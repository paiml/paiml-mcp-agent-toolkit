#!/usr/bin/env bash
# GH-1318: cancel-in-progress does not cancel a run still `queued`; the new run then sits
# pending behind it. Force-cancel every queued, waiting or pending run (one paginated GET per status)
# of the same workflow for the same PR whose head_sha is not the PR's LIVE head. The live head is
# read from the API, not from the event payload, so a delayed older trigger can never cancel the
# newest run, and the run for the live head is never cancelled.
# FLEET RULE GH-1/GH-2: the rate limit is checked first (below 1000 remaining => not_measured, no
# calls made); the PR head is read through GraphQL, the run lists/status re-reads (not in GraphQL)
# and the force-cancel write go through REST.
# Exit codes: 0 done, 1 a force-cancel failed, 2 usage, 3 not_measured (a read failed or was empty/invalid).
# Usage: cancel-queued-predecessors.sh REPO PR_NUMBER HEAD_SHA WORKFLOW_FILE
#        cancel-queued-predecessors.sh --self-test
# GH_BIN overrides the gh binary (the self-test plants a fake one).
set -euo pipefail
GH="${GH_BIN:-gh}"

# GH-1: skip (not_measured) when fewer than 1000 core or graphql calls remain, or the budget is unreadable.
check_budget() {
  local rem
  rem=$("$GH" api rate_limit --jq '[.resources.core.remaining,.resources.graphql.remaining]|min') \
    || { echo "not_measured: rate limit unreadable" >&2; return 3; }
  case "$rem" in ''|*[!0-9]*) echo "not_measured: rate limit empty or invalid" >&2; return 3 ;; esac
  if [ "$rem" -lt 1000 ]; then echo "not_measured: only $rem API calls remain (GH-1 floor 1000)" >&2; return 3; fi
}

# Live head of the PR via GraphQL (GH-2), hex-validated. Returns 3 (not_measured) on a failed or invalid read.
read_live() {
  local l
  l=$("$GH" api graphql -f query='query($o:String!,$n:String!,$p:Int!){repository(owner:$o,name:$n){pullRequest(number:$p){headRefOid}}}' \
    -f o="${1%%/*}" -f n="${1#*/}" -F p="$2" --jq .data.repository.pullRequest.headRefOid) || { echo "not_measured: live head of PR #$2 unreadable" >&2; return 3; }
  case "$l" in ''|*[!0-9a-fA-F]*) echo "not_measured: live head of PR #$2 empty or invalid" >&2; return 3 ;; esac
  printf '%s\n' "$l"
}

run() {
  [ "$#" -eq 4 ] || { echo "usage: $0 REPO PR_NUMBER HEAD_SHA WORKFLOW_FILE" >&2; return 2; }
  local repo=$1 pr=$2 sha=$3 wf=$4 id hs st live now raw ids out cur
  case "$pr" in ''|*[!0-9]*) echo "usage: PR_NUMBER must be digits" >&2; return 2 ;; esac
  check_budget || return 3
  live=$(read_live "$repo" "$pr") || return 3
  for st in queued waiting pending; do
    raw=$("$GH" api --paginate "repos/$repo/actions/workflows/$wf/runs?event=pull_request&per_page=100&status=$st") \
      || { echo "not_measured: list of $st runs failed" >&2; return 3; }
    ids=$(jq -rs --arg live "$live" --arg st "$st" --argjson pr "$pr" \
      'if all(.[]; (.workflow_runs|type)=="array") and length>0
       then [.[].workflow_runs[] | select(.status == $st) | select(.head_sha != $live) | select(any(.pull_requests[]?; .number == $pr)) | "\(.id) \(.head_sha)"][] else error("invalid") end' \
      <<<"$raw") || { echo "not_measured: list of $st runs empty or invalid" >&2; return 3; }
    while read -r id hs; do
      [ -n "$id" ] || continue
      # TOCTOU: a push may land between the list and this cancel. Re-read the live head and never
      # cancel the run that belongs to it.
      now=$(read_live "$repo" "$pr") || return 3
      if [ "$hs" = "$now" ]; then echo "skipping run $id: it is the live head now"; continue; fi
      # The run may have started since it was listed: only a run still queued/waiting/pending is cancelled.
      cur=$("$GH" api "repos/$repo/actions/runs/$id" --jq .status) || { echo "not_measured: status of run $id unreadable" >&2; return 3; }
      case "$cur" in
        ''|null) echo "not_measured: status of run $id empty or null" >&2; return 3 ;;
        queued|waiting|pending) ;;
        *) echo "skipping run $id: status is now '$cur', not queued"; continue ;;
      esac
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
# SCEN=normal|empty|fail500|bad|pullsfail|runsfail|nonhex|race|inprog|flip
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
if [ "$1" = rate_limit ]; then
  case $SCEN in ratefail) echo "gh: HTTP 500" >&2; exit 1;; rateempty) exit 0;; ratelow) echo 999;; *) echo 4000;; esac
  exit 0
fi
pag=0; if [ "$1" = --paginate ]; then pag=1; shift; fi
url=$1
case $url in */pulls/*) echo "gh: REST PR read forbidden (GH-2)" >&2; exit 9;; esac
if [ "$url" = graphql ]; then url=repos/o/r/pulls/7; set -- "$url" "${@: -2:1}" "${@: -1}"; fi
case $url in */pulls/7)
  [ "$SCEN" = pullsfail ] && { echo feed; echo "gh: API rate limit exceeded (HTTP 403)" >&2; exit 1; }
  n=$(cat "$LOG.cnt" 2>/dev/null || echo 0); echo $((n+1)) >"$LOG.cnt"
  [ "$SCEN" = livefail2 ] && [ "$n" -ge 1 ] && { echo "gh: HTTP 403" >&2; exit 1; }
  h=feed
  case $SCEN in race) if [ "$n" = 0 ]; then h=aaaa; else h=bbbb; fi;; nonhex) h=null;; esac
  if [ "$2" = --jq ]; then
    case $3 in .data.repository.pullRequest.headRefOid) echo "$h";; *) echo '{}';; esac
  else echo "{\"head\":{\"sha\":\"$h\"}}"; fi
  exit 0;; esac
case $url in */actions/runs/*)
  id=${url##*/}
  case $SCEN in
    statusfail) echo "gh: HTTP 429" >&2; exit 1;;
    statusempty) exit 0;;
    statusnull) echo null; exit 0;;
  esac
  [ "$2" = --jq ] && [ "$3" = .status ] || { echo "{\"status\":\"in_progress\"}"; exit 0; }
  case $id in 77) [ "$SCEN" = flip ] && echo in_progress || echo queued;; 5) echo waiting;; 6) echo pending;; *) echo queued;; esac
  exit 0;; esac
case $SCEN in empty) exit 0;; runsfail) echo "gh: HTTP 429" >&2; exit 1;; race) echo '{"workflow_runs":[{"id":101,"status":"queued","head_sha":"aaaa","pull_requests":[{"number":7}]},{"id":102,"status":"queued","head_sha":"bbbb","pull_requests":[{"number":7}]}]}'; exit 0;; bad) echo 'not json'; exit 0;; esac
base=repos/o/r/actions/workflows/ci.yml/runs?event=pull_request\&per_page=100\&status=
case $url in
  "${base}queued") ;;
  "${base}waiting") echo '{"workflow_runs":[{"id":5,"status":"waiting","head_sha":"dead","pull_requests":[{"number":7}]}]}'; exit 0;;
  "${base}pending") echo '{"workflow_runs":[{"id":6,"status":"pending","head_sha":"dead","pull_requests":[{"number":7}]}]}'; exit 0;;
  *) exit 9;;
esac
if [ "$SCEN" = inprog ]; then
  echo '{"workflow_runs":[{"id":77,"status":"in_progress","head_sha":"dead","pull_requests":[{"number":7}]}]}'; exit 0
fi
if [ "$SCEN" = mismatch ]; then
  echo '{"workflow_runs":[{"id":88,"status":"completed","head_sha":"dead","pull_requests":[{"number":7}]}]}'; exit 0
fi
if [ "$SCEN" = flip ]; then
  echo '{"workflow_runs":[{"id":77,"status":"queued","head_sha":"dead","pull_requests":[{"number":7}]}]}'; exit 0
fi
if [ "$SCEN" = fail500 ]; then
  echo '{"workflow_runs":[{"id":1409,"status":"queued","head_sha":"dead","pull_requests":[{"number":7}]}]}'; exit 0
fi
echo '{"workflow_runs":[
 {"id":4,"status":"queued","head_sha":"dead","pull_requests":[{"number":7}]},
 {"id":1,"status":"queued","head_sha":"dead","pull_requests":[{"number":7}]},
 {"id":2,"status":"queued","head_sha":"feed","pull_requests":[{"number":7}]},
 {"id":3,"status":"queued","head_sha":"dead","pull_requests":[{"number":8}]}]}'
[ "$pag" = 1 ] && echo '{"workflow_runs":[{"id":9,"status":"queued","head_sha":"dead","pull_requests":[{"number":7}]}]}'
exit 0
FAKE
  chmod +x "$1/gh"
}

self_test() {
  local d rc fails=0; d=$(mktemp -d); TMPD="$d"
  trap cleanup EXIT
  fake_gh "$d"
  expect() { # name want_rc want_log trigger_sha scen
    : >"$d/log"; rm -f "${d:?}/log.cnt"; rc=0
    LOG="$d/log" SCEN=$5 GH_BIN="$d/gh" bash "$0" o/r 7 "$4" ci.yml >/dev/null 2>&1 || rc=$?
    local got; got=$(sed 's#repos/o/r/actions/runs/##;s#/force-cancel##' "$d/log" | tr '\n' ' ')
    if [ "$rc" != "$2" ] || [ "$got" != "$3" ]; then echo "self-test FAIL [$1]: rc=$rc (want $2) cancelled='$got' (want '$3')"; fails=1; fi
  }
  expect normal 0 "1 9 5 6 " feed normal            # pages followed, waiting+pending covered, 409 tolerated, newest/other PR kept
  expect stale-trigger 0 "1 9 5 6 " dead normal     # delayed older trigger still never cancels the live-head run (2)
  expect empty-output 3 "" feed empty               # empty body is not_measured, not success
  expect invalid-output 3 "" feed bad
  expect http500-with-409-in-id 1 "" feed fail500   # a 500 on run 1409 is a failure, not a 409
  expect pulls-read-fails 3 "" feed pullsfail      # HTTP 403 on the live-head read is not_measured
  expect runs-read-fails 3 "" feed runsfail         # HTTP 429 on a run list is not_measured
  expect live-head-not-hex 3 "" feed nonhex         # `null` is not a sha
  expect push-lands-mid-run 0 "" aaaa race      # newest push (bbbb) lands after the first read: its run 102 is kept
  expect in-progress-listed 0 "5 6 " feed inprog        # a run listed as in_progress is never cancelled
  expect started-after-list 0 "5 6 " feed flip          # listed queued, in_progress by cancel time: kept
  expect rate-below-floor 3 "" feed ratelow         # 999 remaining: no calls, not_measured (GH-1)
  expect rate-unreadable 3 "" feed ratefail
  expect rate-empty 3 "" feed rateempty
  expect live-reread-fails 3 "" feed livefail2      # the per-cancel head re-read failing is not_measured, never fail-open
  expect status-reread-fails 3 "" feed statusfail   # failed per-run status read: not_measured, no cancel
  expect status-reread-empty 3 "" feed statusempty
  expect status-reread-null 3 "" feed statusnull
  expect status-mismatch-listed 0 "5 6 " feed mismatch  # a listed run whose status != the queried one is never cancelled
  rc=0; bash "$0" o/r 7 >/dev/null 2>&1 || rc=$?
  [ "$rc" = 2 ] || { echo "self-test FAIL [usage]: rc=$rc (want 2)"; fails=1; }
  [ "$fails" = 0 ] || exit 1
  echo "self-test ok"
}

if [ "${1:-}" = --self-test ]; then self_test; else run "$@"; fi
