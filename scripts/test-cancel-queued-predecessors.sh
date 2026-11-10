#!/usr/bin/env bash
# Refs #1318 — selection logic of cancel-queued-predecessors.sh (no network).
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
tmp=$(mktemp); trap 'rm -f "$tmp"' EXIT
cat >"$tmp" <<'J'
{"workflow_runs":[
 {"id":1,"status":"queued","head_sha":"old1"},
 {"id":2,"status":"queued","head_sha":"cur"},
 {"id":3,"status":"in_progress","head_sha":"old2"},
 {"id":4,"status":"queued","head_sha":"old3"}]}
J
out=$(CQP_RUNS_JSON=$tmp CQP_DRY_RUN=1 "$here/cancel-queued-predecessors.sh" o/r ci.yml br cur)
want=$'would force-cancel 1\nwould force-cancel 4'
[ "$out" = "$want" ] || { echo "FAIL: got: $out"; exit 1; }
# empty list cancels nothing
echo '{"workflow_runs":[]}' >"$tmp"
out=$(CQP_RUNS_JSON=$tmp CQP_DRY_RUN=1 "$here/cancel-queued-predecessors.sh" o/r ci.yml br cur)
[ -z "$out" ] || { echo "FAIL: empty list produced: $out"; exit 1; }
echo "ok"
