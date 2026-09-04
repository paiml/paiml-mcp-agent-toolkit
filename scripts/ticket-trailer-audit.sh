#!/usr/bin/env bash
# ticket-trailer-audit — AD-07 acceptance: every commit on a branch carries a Pmat-Ticket trailer naming an
# in-progress ticket (comply check CB-1340), and `pmat work link` records commits and PRs on the ticket.
#   PMAT=<binary> scripts/ticket-trailer-audit.sh
# Exit 0 = every leg green; 1 = a leg red.
set -euo pipefail
PMAT=${PMAT:-pmat}
red=0; leg(){ if [ "$2" = 0 ]; then echo "  ✓ $1"; else echo "  ✗ $1"; red=1; fi; }
echo "ticket-trailer-audit (AD-07) — $($PMAT --version 2>/dev/null | head -1)"
T=$(mktemp -d); [ -n "$T" ] && [ "$T" != "/" ] || exit 2
cleanup() { if [ -n "${T:-}" ] && [ "$T" != "/" ] && [ -d "$T" ]; then find "$T" -type f -delete; find "$T" -depth -type d -empty -delete; fi; }
trap cleanup EXIT
mk_repo(){ # $1 dir: a git repository on master with a roadmap holding PMAT-1 (in progress) and PMAT-2 (completed)
  mkdir -p "$1/docs/roadmaps"; ( cd "$1" && git init -q --template= -b master && git config user.email t@t && git config user.name t && git config core.hooksPath /dev/null )
  cat > "$1/docs/roadmaps/roadmap.yaml" <<'Y'
roadmap_version: '1.0'
github_enabled: false
github_repo: fx/fx
roadmap:
- id: PMAT-1
  github_issue: null
  item_type: task
  title: in progress
  status: in_progress
  priority: medium
  assigned_to: null
  created: 2026-09-04T00:00:00Z
  updated: 2026-09-04T00:00:00Z
  spec: null
  acceptance_criteria: []
  phases: []
  subtasks: []
  estimated_effort: null
  labels: []
  notes: null
- id: PMAT-2
  github_issue: null
  item_type: task
  title: completed
  status: completed
  priority: medium
  assigned_to: null
  created: 2026-09-04T00:00:00Z
  updated: 2026-09-04T00:00:00Z
  spec: null
  acceptance_criteria: []
  phases: []
  subtasks: []
  estimated_effort: null
  labels: []
  notes: null
Y
  ( cd "$1" && echo a > a.txt && git add . && git commit -qm "init" && git switch -q -c PMAT-1-work )
}
commit(){ ( cd "$1" && echo "$2" >> a.txt && git commit -qam "$2" ); }
trailer_commit(){ ( cd "$1" && echo "$2" >> a.txt && git commit -qa -F - <<M
$2

Pmat-Ticket: $3
M
) ; }
cb(){ ( cd "$1" && "$PMAT" comply check --format json 2>/dev/null || true ) | python3 -c '
import json,sys
d=json.load(sys.stdin)
for c in d.get("checks") or []:
    if "cb-1340" in str(c.get("name","")).lower():
        st=str(c.get("status","")).lower()
        print(("pass" if st in ("pass","passed","ok") else "fail")+" "+str(c.get("message") or "")[:160]); sys.exit(0)
print("absent"); sys.exit(0)'; }
# ---- 1: every commit trailered → CB-1340 passes
R="$T/ok"; mk_repo "$R"; trailer_commit "$R" "one" PMAT-1; trailer_commit "$R" "two" PMAT-1
out=$(cb "$R"); case "$out" in pass*) leg "CB-1340: a branch whose commits all carry Pmat-Ticket → pass" 0;; *) leg "CB-1340: all trailered → pass (got: ${out:0:60})" 1;; esac
# ---- 2: one untrailered commit → fails naming its sha
commit "$R" "three, no trailer"; SHA=$(git -C "$R" rev-parse --short HEAD)
out=$(cb "$R"); case "$out" in fail*"$SHA"*) leg "CB-1340: an untrailered commit fails naming $SHA" 0;; *) leg "CB-1340: untrailered → fail naming sha (got: ${out:0:80})" 1;; esac
# ---- 3: a trailer naming a completed ticket → fails
R2="$T/done"; mk_repo "$R2"; trailer_commit "$R2" "one" PMAT-2
out=$(cb "$R2"); case "$out" in fail*) leg "CB-1340: a trailer naming a completed ticket fails" 0;; *) leg "CB-1340: completed ticket → fail (got: ${out:0:60})" 1;; esac
# ---- 4: on the default branch the check does not fire on history (control)
out=$( cd "$R2" && git switch -q master && cb "$R2" ); case "$out" in pass*) leg "CB-1340 control: on master the check passes (nothing to judge)" 0;; *) leg "CB-1340 control on master (got: ${out:0:60})" 1;; esac
# ---- 5: pmat work link records a commit and a PR on the ticket, and annotate shows them
R3="$T/link"; mk_repo "$R3"; trailer_commit "$R3" "one" PMAT-1; SHA3=$(git -C "$R3" rev-parse HEAD)
if ( cd "$R3" && "$PMAT" work link PMAT-1 --commit "$SHA3" >/dev/null 2>&1 && "$PMAT" work link PMAT-1 --pr 42 >/dev/null 2>&1 ); then
  if grep -q "${SHA3:0:12}" "$R3/docs/roadmaps/roadmap.yaml" && grep -qE 'pr: *42|#42|pull.*42' "$R3/docs/roadmaps/roadmap.yaml"; then leg "work link: the commit and the PR are recorded on the ticket" 0; else leg "work link: recorded on the ticket (roadmap lacks them)" 1; fi
  if ( cd "$R3" && "$PMAT" work annotate PMAT-1 2>/dev/null | grep -q "${SHA3:0:7}" ); then leg "work annotate shows the linked commit" 0; else leg "work annotate shows the linked commit" 1; fi
else leg "work link: subcommand exists and accepts --commit / --pr" 1; leg "work annotate shows the linked commit (skipped: no link)" 1; fi
[ "$red" = 0 ] && echo "GREEN" || { echo "RED"; exit 1; }
