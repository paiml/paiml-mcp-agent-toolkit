#!/usr/bin/env bash
# GH-1318: cancel-in-progress does not cancel a run still `queued`; the new run then sits
# pending behind it. Force-cancel every queued, waiting or pending run (one paginated GET per status)
# of the same workflow for the same PR whose head_sha is not the PR's LIVE head. The live head is
# read from the API, not from the event payload, so a delayed older trigger can never cancel the
# newest run, and the run for the live head is never cancelled.
# Exit codes: 0 done, 1 a force-cancel failed, 2 usage, 3 not_measured (a read failed or was empty/invalid).
# Usage: cancel-queued-predecessors.sh REPO PR_NUMBER HEAD_SHA WORKFLOW_FILE
#        cancel-queued-predecessors.sh --self-test
# GH_BIN overrides the gh binary (the self-test plants a fake one).
set -euo pipefail
GH="${GH_BIN:-gh}"

run() {
  [ "$#" -eq 4 ] || { echo "usage: $0 REPO PR_NUMBER HEAD_SHA WORKFLOW_FILE" >&2; return 2; }
  local repo=$1 pr=$2 sha=$3 wf=$4 id st live raw ids out
  case "$pr" in ''|*[!0-9]*) echo "usage: PR_NUMBER must be digits" >&2; return 2 ;; esac
  live=$("$GH" api "repos/$repo/pulls/$pr" --jq .head.sha) || { echo "not_measured: live head of PR #$pr unreadable" >&2; return 3; }
  case "$live" in ''|*[!0-9a-fA-F]*) echo "not_measured: live head of PR #$pr empty or invalid" >&2; return 3 ;; esac
  for st in queued waiting pending; do
    raw=$("$GH" api --paginate "repos/$repo/actions/workflows/$wf/runs?event=pull_request&per_page=100&status=$st") \
      || { echo "not_measured: list of $st runs failed" >&2; return 3; }
    ids=$(jq -rs --arg live "$live" --argjson pr "$pr" \
      'if all(.[]; (.workflow_runs|type)=="array") and length>0
       then [.[].workflow_runs[] | select(.head_sha != $live) | select(any(.pull_requests[]?; .number == $pr)) | .id][] else error("invalid") end' \
      <<<"$raw") || { echo "not_measured: list of $st runs empty or invalid" >&2; return 3; }
    while IFS= read -r id; do
      [ -n "$id" ] || continue
      echo "force-cancelling $st predecessor $id of PR #$pr (live head ${live:0:7}, trigger ${sha:0:7})"
      if ! out=$("$GH" api -X POST "repos/$repo/actions/runs/$id/force-cancel" 2>&1); then
        case "$out" in
          *"(HTTP 409)"*) echo "run $id already moving (409), continuing" ;;
          *) echo "force-cancel $id failed: $out" >&2; return 1 ;;
        esac
      fi
    done <<<"$ids"
  done
}

TMPD=
cleanup() { if [ -n "$TMPD" ]; then rm -rf "${TMPD:?}"; fi; }

# Fake gh: pulls/7 -> live head NEW. Run lists are served from $SCEN by status; page 2 only with --paginate.
# SCEN=normal|empty|fail500|bad
fake_gh() {
  cat >"$1/gh" <<'FAKE'
#!/usr/bin/env bash
if [ "$1" = api ] && [ "$2" = -X ]; then
  case $4 in
    */runs/1409/force-cancel) echo "gh: Internal Server Error (HTTP 500) on $4" >&2; exit 1;;
    */runs/4/force-cancel) echo "gh: Cannot cancel a run that is not queued (HTTP 409)" >&2; exit 1;;
  esac
  echo "$4" >>"$LOG"; exit 0
fi
shift
pag=0; if [ "$1" = --paginate ]; then pag=1; shift; fi
url=$1
case $url in */pulls/7) echo feed; exit 0;; esac
case $SCEN in empty) exit 0;; bad) echo 'not json'; exit 0;; esac
case $url in
  *status=queued*) ;;
  *status=waiting*) echo '{"workflow_runs":[{"id":5,"head_sha":"dead","pull_requests":[{"number":7}]}]}'; exit 0;;
  *status=pending*) echo '{"workflow_runs":[{"id":6,"head_sha":"dead","pull_requests":[{"number":7}]}]}'; exit 0;;
  *) exit 9;;
esac
if [ "$SCEN" = fail500 ]; then
  echo '{"workflow_runs":[{"id":1409,"head_sha":"dead","pull_requests":[{"number":7}]}]}'; exit 0
fi
echo '{"workflow_runs":[
 {"id":4,"head_sha":"dead","pull_requests":[{"number":7}]},
 {"id":1,"head_sha":"dead","pull_requests":[{"number":7}]},
 {"id":2,"head_sha":"feed","pull_requests":[{"number":7}]},
 {"id":3,"head_sha":"dead","pull_requests":[{"number":8}]}]}'
[ "$pag" = 1 ] && echo '{"workflow_runs":[{"id":9,"head_sha":"dead","pull_requests":[{"number":7}]}]}'
exit 0
FAKE
  chmod +x "$1/gh"
}

self_test() {
  local d rc fails=0; d=$(mktemp -d); TMPD="$d"
  trap cleanup EXIT
  fake_gh "$d"
  expect() { # name want_rc want_log trigger_sha scen
    : >"$d/log"; rc=0
    LOG="$d/log" SCEN=$5 GH_BIN="$d/gh" bash "$0" o/r 7 "$4" ci.yml >/dev/null 2>&1 || rc=$?
    local got; got=$(sed 's#repos/o/r/actions/runs/##;s#/force-cancel##' "$d/log" | tr '\n' ' ')
    if [ "$rc" != "$2" ] || [ "$got" != "$3" ]; then echo "self-test FAIL [$1]: rc=$rc (want $2) cancelled='$got' (want '$3')"; fails=1; fi
  }
  expect normal 0 "1 9 5 6 " feed normal            # pages followed, waiting+pending covered, 409 tolerated, newest/other PR kept
  expect stale-trigger 0 "1 9 5 6 " dead normal     # delayed older trigger still never cancels the live-head run (2)
  expect empty-output 3 "" feed empty               # empty body is not_measured, not success
  expect invalid-output 3 "" feed bad
  expect http500-with-409-in-id 1 "" feed fail500   # a 500 on run 1409 is a failure, not a 409
  rc=0; bash "$0" o/r 7 >/dev/null 2>&1 || rc=$?
  [ "$rc" = 2 ] || { echo "self-test FAIL [usage]: rc=$rc (want 2)"; fails=1; }
  [ "$fails" = 0 ] || exit 1
  echo "self-test ok"
}

if [ "${1:-}" = --self-test ]; then self_test; else run "$@"; fi
