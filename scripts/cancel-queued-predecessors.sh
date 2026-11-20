#!/usr/bin/env bash
# GH-1318: cancel-in-progress does not cancel a run still `queued`; the new run then sits
# pending behind it. Force-cancel every queued/waiting/pending run of the same workflow for
# the same PR whose head_sha differs from the current one.
# Usage: cancel-queued-predecessors.sh REPO PR_NUMBER HEAD_SHA WORKFLOW_FILE
#        cancel-queued-predecessors.sh --self-test
# GH_BIN overrides the gh binary (the self-test plants a fake one).
set -euo pipefail
GH="${GH_BIN:-gh}"

run() {
  local repo=$1 pr=$2 sha=$3 wf=$4 id
  local ids
  ids=$("$GH" api "repos/$repo/actions/workflows/$wf/runs?event=pull_request&per_page=100&status=queued" \
    --jq ".workflow_runs[] | select(.head_sha != \"$sha\") | select(any(.pull_requests[]?; .number == $pr)) | .id")
  for id in $ids; do
    echo "force-cancelling queued predecessor $id of PR #$pr"
    "$GH" api -X POST "repos/$repo/actions/runs/$id/force-cancel" >/dev/null
  done
}

self_test() {
  local d; d=$(mktemp -d); trap 'rm -rf "$d"' RETURN
  cat >"$d/gh" <<'FAKE'
#!/usr/bin/env bash
# plant: runs 1 (old sha, PR 7), 2 (current sha), 3 (old sha, other PR 8) all queued
if [ "$1" = api ] && [ "$2" = -X ]; then echo "$4" >>"$LOG"; exit 0; fi
shift; url=$1; shift
case $url in *status=queued*) ;; *) exit 9;; esac
jq -r '.workflow_runs[] | select(.head_sha != "NEW") | select(any(.pull_requests[]?; .number == 7)) | .id' <<'J'
{"workflow_runs":[
 {"id":1,"head_sha":"OLD","pull_requests":[{"number":7}]},
 {"id":2,"head_sha":"NEW","pull_requests":[{"number":7}]},
 {"id":3,"head_sha":"OLD","pull_requests":[{"number":8}]}]}
J
FAKE
  chmod +x "$d/gh"
  : >"$d/log"
  LOG="$d/log" GH_BIN="$d/gh" bash "$0" o/r 7 NEW ci.yml >/dev/null
  [ "$(cat "$d/log")" = "repos/o/r/actions/runs/1/force-cancel" ] || { echo "self-test FAIL: $(cat "$d/log")"; exit 1; }
  echo "self-test ok"
}

if [ "${1:-}" = --self-test ]; then self_test; else run "$@"; fi
