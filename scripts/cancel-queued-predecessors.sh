#!/usr/bin/env bash
# Refs #1318 — `cancel-in-progress: true` leaves a QUEUED predecessor alive, and
# the new run of the same concurrency group sits `pending` with 0 jobs behind it.
# Force-cancel every run of this workflow for the same PR whose head_sha differs
# and whose status is `queued` (plain `gh run cancel` was measured ineffective on
# a queued run; POST .../force-cancel is what moved it).
#
# usage: cancel-queued-predecessors.sh <repo> <workflow-file> <branch> <current-sha>
# env:   CQP_RUNS_JSON  (test seam) read the run list from this file instead of the API
#        CQP_DRY_RUN=1  print the ids that would be force-cancelled, cancel nothing
set -euo pipefail
repo=$1 workflow=$2 branch=$3 sha=$4

if [ -n "${CQP_RUNS_JSON:-}" ]; then
  runs=$(cat "$CQP_RUNS_JSON")
else
  runs=$(gh api "repos/$repo/actions/workflows/$workflow/runs?branch=$branch&status=queued&per_page=100")
fi

ids=$(jq -r --arg sha "$sha" \
  '.workflow_runs[] | select(.status == "queued" and .head_sha != $sha) | .id' <<<"$runs")

for id in $ids; do
  if [ "${CQP_DRY_RUN:-0}" = 1 ]; then
    echo "would force-cancel $id"
  else
    echo "force-cancelling queued predecessor $id"
    gh api -X POST "repos/$repo/actions/runs/$id/force-cancel" >/dev/null
  fi
done
