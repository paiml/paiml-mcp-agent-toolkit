#!/usr/bin/env bash
# roadmap-coherence-control.sh — prove CB-2115 can fail before trusting it
# (goal-mode.md §7.1 "CONTROL FIRST"; PMAT-722, goal-mode step 4).
#
#   scripts/roadmap-coherence-control.sh <path-to-pmat-binary>
#
# A gate that cannot fail is theater. This script plants the defects CB-2115
# exists to catch in a throwaway project — a roadmap, a GitHub snapshot file,
# and a `.pmat.yaml` pointing the rule at that file so no arm touches the
# network — and asserts the verdict on every arm, the RED arms AND the GREEN
# ones, because a rule that always fails would pass the RED arms for the wrong
# reason:
#
#   arm 1  RED   §7 falsifier: the linked issue is closed, the item is open   exit 1, Fail, ORPHAN-ROADMAP PMAT-001 #1 closed
#   arm 2  GREEN the same pair, the issue open, the titles equal              exit 0, Pass, matched 1
#   arm 3  RED   two open items name #1                                       exit 1, Fail, COLLISION #1 PMAT-001 PMAT-002
#   arm 4  RED   an open, unlabelled issue #2 that no item names              exit 1, Fail, ORPHAN-GITHUB #2
#   arm 5  RED   titles disagree, both sides older than the grace window     exit 1, Fail, DRIFT title
#   arm 6  GREEN the same disagreement, the issue touched just now            exit 0, Pass, tolerated 1
#   arm 7  N/M   the snapshot file named on the command line does not exist    exit 1, Fail, message starts not_measured:
#   arm 8  SKIP  no roadmap, never committed                                  exit 0, Skip, names docs/roadmaps/roadmap.yaml
#   arm 9  N/M   the roadmap was committed and then deleted                   exit 1, Fail, not_measured: "committed and is now gone"
#   arm 10 N/M   a snapshot path committed in .pmat.yaml (the bypass token)   exit 1, Fail, not_measured: names .pmat.yaml and --github-snapshot
#   arm 11 RED   grace_minutes: 10 makes a 30-minute-old disagreement a DRIFT exit 1, Fail — then the same pair passes without the option
#
# Each arm asserts BOTH the process exit code and the CB-2115 entry in the JSON
# report: the exit code is what CI acts on, the entry is what proves the verdict
# came from this rule and not from another one. A missing entry is
# under-discovery and fails the control (exit 2) — "we found no CB-2115 row"
# must never render as "CB-2115 passed".
#
# Arm 6 is the one that proves §5.2's tolerance is a bound in TIME: the same
# disagreement as arm 5 passes only because one side moved inside the window.
# Arm 11 proves the option is READ (a rule that hard-coded 60 survives every
# other arm — the quorum on PMAT-722 named that mutant). Arms 9 and 10 are the
# input-deletion and bypass-token findings of the same quorum.
#
# The snapshot reaches the rule on the COMMAND LINE (`--github-snapshot`),
# never through .pmat.yaml: a path committed in the tree would let every CI
# run judge a fixture instead of GitHub, and arm 10 asserts it is refused.
#
# Exit: 0 every arm behaved · 1 an arm did not (named on stderr) · 2 usage,
# missing binary, or a report the script could not read.
#
# The binary is an ARGUMENT, never resolved from PATH: the control must judge
# the pmat built from this tree, not whichever one is installed.
set -uo pipefail

PMAT="${1:-}"
if [ -z "$PMAT" ] || [ ! -x "$PMAT" ]; then
  echo "roadmap-coherence-control: usage: $0 <path-to-pmat-binary> (got '${PMAT}')" >&2
  exit 2
fi
command -v jq >/dev/null 2>&1 || { echo "roadmap-coherence-control: jq is required" >&2; exit 2; }
command -v git >/dev/null 2>&1 || { echo "roadmap-coherence-control: git is required" >&2; exit 2; }

work="$(mktemp -d "${TMPDIR:-/tmp}/roadmap-coherence-control.XXXXXX")"
trap 'rm -rf "${work:?}"' EXIT
repo="$work/repo"
report="$work/report.json"

# Fixture timestamps against the REAL clock, on purpose: §5.2's tolerance is a
# bound in time, and arms 5/6 prove it by moving one side of the same pair
# inside and outside the window. Nothing built from them is an artifact.
old="$(date -u -d '2 days ago' +%Y-%m-%dT%H:%M:%SZ)"   # bashrs disable-line=DET002
now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"                   # bashrs disable-line=DET002

# The fixture is a git repository with NO remote, so nothing here can fall back
# to a live `gh` call: the rule reads the snapshot the `.pmat.yaml` names or
# reports that it could not. Host git configuration is kept out of it.
mkdir -p "$repo/docs/roadmaps"
GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null git -C "$repo" init -q -b master

# write_roadmap <item-yaml>...   one argument per item (`$(...)` strips the
# trailing newline, so the newline between items is added HERE, never inside
# an argument — two items on one line was measured as "mapping values are not
# allowed in this context" and a not_measured verdict for the wrong reason).
write_roadmap() {
  {
    printf 'roadmap_version: "1.0"\ngithub_enabled: true\ngithub_repo: paiml/fixture\nroadmap:\n'
    for it in "$@"; do printf '%s\n' "$it"; done
  } >"$repo/docs/roadmaps/roadmap.yaml"
}
# item <id> <title> <status> <issue|null> <updated>
item() {
  printf '  - id: %s\n    title: %s\n    status: %s\n    github_issue: %s\n    updated: %s\n' "$1" "$2" "$3" "$4" "$5"
}
# issue <number> <title> <open|closed> <labels-json-array> <updated_at>
issue() {
  jq -cn --argjson n "$1" --arg t "$2" --arg s "$3" --argjson l "$4" --arg u "$5" \
    '{number:$n, title:$t, state:$s, labels:$l, updated_at:$u}'
}
# write_snapshot <issue-json>...   (GRACE, if set, becomes the only .pmat.yaml option)
write_snapshot() {
  printf '%s\n' "$@" | jq -s '{repo:"paiml/fixture", taken_at:"2026-09-10T00:00:00Z", issues:., milestones:[]}' >"$repo/snapshot.json"
  rm -f "$repo/.pmat.yaml"
  if [ -n "${GRACE:-}" ]; then
    printf 'comply:\n  checks:\n    cb-2115:\n      options:\n        grace_minutes: %s\n' "$GRACE" >"$repo/.pmat.yaml"
  fi
}

# One run of the gate against the fixture, judging the fixture's snapshot
# through the command-line flag unless the caller passes its own arguments.
# Sets RC (process exit), STATUS and MESSAGE (the CB-2115 entry). A report with
# no CB-2115 entry is exit 2.
run_gate() {
  RC=0
  if [ "$#" -eq 0 ]; then set -- --github-snapshot "$repo/snapshot.json"; fi
  "$PMAT" comply check --checks CB-2115 --path "$repo" --format json "$@" >"$report" 2>"$work/stderr" || RC=$?
  local entry
  entry=$(jq -c '[.checks[] | select(.name | startswith("CB-2115"))] | first // empty' "$report" 2>/dev/null || true)
  if [ -z "$entry" ]; then
    echo "roadmap-coherence-control: the report carries no CB-2115 entry (exit $RC) — under-discovery is a finding, not a pass" >&2
    echo "--- stderr" >&2; head -20 "$work/stderr" >&2
    echo "--- report" >&2; head -c 2000 "$report" >&2; echo >&2
    exit 2
  fi
  STATUS=$(printf '%s' "$entry" | jq -r '.status')
  MESSAGE=$(printf '%s' "$entry" | jq -r '.message')
}

fail_arm() {
  echo "roadmap-coherence-control: ARM $1 FAILED — $2" >&2
  echo "  exit=$RC status=$STATUS" >&2
  echo "  message=$MESSAGE" >&2
  exit 1
}
# needs <arm> <needle>...  — every needle must appear in MESSAGE
needs() {
  local arm=$1; shift
  for n in "$@"; do
    case "$MESSAGE" in *"$n"*) ;; *) fail_arm "$arm" "the message must name '$n'";; esac
  done
}

echo "roadmap-coherence-control: fixture $repo (no remote, snapshot via .pmat.yaml); pmat=$("$PMAT" --version 2>/dev/null | head -1)"

# ── arm 1: RED — the §7 falsifier: close one linked issue, leave the item open ──
write_roadmap "$(item PMAT-001 'planned work' planned 1 "$old")"
write_snapshot "$(issue 1 'planned work' closed '[]' "$old")"
run_gate
[ "$RC" -eq 1 ] || fail_arm 1 "an open item naming a closed issue must exit 1"
[ "$STATUS" = "Fail" ] || fail_arm 1 "CB-2115 must be Fail"
needs 1 "ORPHAN-ROADMAP" "PMAT-001" "#1" "closed"
echo "roadmap-coherence-control: arm 1 RED   — open item PMAT-001 naming closed #1 refused (exit 1)"

# ── arm 2: GREEN — the same pair, open, titles equal ───────────────────────────
write_snapshot "$(issue 1 'planned work' open '[]' "$old")"
run_gate
[ "$RC" -eq 0 ] || fail_arm 2 "a bijection must exit 0"
[ "$STATUS" = "Pass" ] || fail_arm 2 "CB-2115 must be Pass"
needs 2 "matched 1" "snapshot.json"
echo "roadmap-coherence-control: arm 2 GREEN — one item, one open issue, in bijection (exit 0)"

# ── arm 3: RED — two open items name #1 ─────────────────────────────────────────
write_roadmap "$(item PMAT-001 'first' planned 1 "$old")" "$(item PMAT-002 'second' planned 1 "$old")"
write_snapshot "$(issue 1 'first' open '[]' "$old")"
run_gate
[ "$RC" -eq 1 ] || fail_arm 3 "a collision must exit 1"
[ "$STATUS" = "Fail" ] || fail_arm 3 "CB-2115 must be Fail on a COLLISION"
needs 3 "COLLISION" "#1" "PMAT-001" "PMAT-002"
echo "roadmap-coherence-control: arm 3 RED   — COLLISION #1 (PMAT-001, PMAT-002) refused (exit 1)"

# ── arm 4: RED — an open unlabelled issue no item names ────────────────────────
write_roadmap "$(item PMAT-001 'planned work' planned 1 "$old")"
write_snapshot "$(issue 1 'planned work' open '[]' "$old")" "$(issue 2 'stray' open '[]' "$old")"
run_gate
[ "$RC" -eq 1 ] || fail_arm 4 "an orphan issue must exit 1"
[ "$STATUS" = "Fail" ] || fail_arm 4 "CB-2115 must be Fail on an ORPHAN-GITHUB"
needs 4 "ORPHAN-GITHUB" "#2"
echo "roadmap-coherence-control: arm 4 RED   — ORPHAN-GITHUB #2 refused (exit 1)"

# ── arm 5: RED — a title disagreement older than the grace window ──────────────
write_roadmap "$(item PMAT-001 'old title' planned 1 "$old")"
write_snapshot "$(issue 1 'new title' open '[]' "$old")"
run_gate
[ "$RC" -eq 1 ] || fail_arm 5 "a disagreement past the grace window must exit 1"
[ "$STATUS" = "Fail" ] || fail_arm 5 "CB-2115 must be Fail on DRIFT"
needs 5 "DRIFT" "PMAT-001" "title"
echo "roadmap-coherence-control: arm 5 RED   — DRIFT on title, 2 days old, refused (exit 1)"

# ── arm 6: GREEN — the same disagreement, the issue touched inside the window ──
write_snapshot "$(issue 1 'new title' open '[]' "$now")"
run_gate
[ "$RC" -eq 0 ] || fail_arm 6 "a disagreement inside the grace window must exit 0 (§5.2: a bound in time)"
[ "$STATUS" = "Pass" ] || fail_arm 6 "CB-2115 must be Pass while the window is open"
needs 6 "tolerated 1"
echo "roadmap-coherence-control: arm 6 GREEN — the same DRIFT, issue updated now, tolerated (exit 0)"

# ── arm 7: not measured — the snapshot named on the command line does not exist ─
rm -f "$repo/snapshot.json"
run_gate
[ "$RC" -eq 1 ] || fail_arm 7 "a snapshot the rule cannot read must exit 1, never pass"
[ "$STATUS" = "Fail" ] || fail_arm 7 "CB-2115 must be Fail when its input is unreadable"
case "$MESSAGE" in "not_measured:"*) ;; *) fail_arm 7 "the message must start with not_measured:";; esac
needs 7 "snapshot.json"
echo "roadmap-coherence-control: arm 7 N/M   — missing snapshot reported not_measured (exit 1)"

# ── arm 8: skip — no roadmap, never committed ──────────────────────────────────
rm -f "$repo/docs/roadmaps/roadmap.yaml" "$repo/.pmat.yaml"
run_gate
[ "$RC" -eq 0 ] || fail_arm 8 "a project with no roadmap must exit 0"
[ "$STATUS" = "Skip" ] || fail_arm 8 "CB-2115 must be Skip with a reason when there is no roadmap"
needs 8 "docs/roadmaps/roadmap.yaml"
echo "roadmap-coherence-control: arm 8 SKIP  — no roadmap, never committed, reported as a skip that names the file (exit 0)"

# ── arm 9: not measured — the roadmap was committed and then deleted ───────────
fgit() {
  GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null \
  GIT_AUTHOR_NAME=cb2115 GIT_AUTHOR_EMAIL=cb2115@example.invalid \
  GIT_COMMITTER_NAME=cb2115 GIT_COMMITTER_EMAIL=cb2115@example.invalid \
    git -C "$repo" -c commit.gpgsign=false "$@"
}
write_roadmap "$(item PMAT-001 'planned work' planned 1 "$old")"
write_snapshot "$(issue 1 'planned work' open '[]' "$old")"
fgit add -A && fgit commit -q -m "roadmap"
fgit rm -q docs/roadmaps/roadmap.yaml
run_gate
[ "$RC" -eq 1 ] || fail_arm 9 "a roadmap that was committed and deleted must exit 1 — deleting a gate's input is not a way of passing it"
[ "$STATUS" = "Fail" ] || fail_arm 9 "CB-2115 must be Fail when its committed input is gone"
case "$MESSAGE" in "not_measured:"*) ;; *) fail_arm 9 "the message must start with not_measured:";; esac
needs 9 "committed and is now gone"
echo "roadmap-coherence-control: arm 9 N/M   — committed-then-deleted roadmap reported not_measured (exit 1)"

# ── arm 10: not measured — a snapshot path committed in .pmat.yaml is refused ──
fgit checkout -q HEAD -- docs/roadmaps/roadmap.yaml
printf 'comply:\n  checks:\n    cb-2115:\n      options:\n        snapshot: snapshot.json\n' >"$repo/.pmat.yaml"
run_gate --checks CB-2115
[ "$RC" -eq 1 ] || fail_arm 10 "a snapshot path in .pmat.yaml must be refused (exit 1), never read"
[ "$STATUS" = "Fail" ] || fail_arm 10 "CB-2115 must be Fail on a committed snapshot path"
case "$MESSAGE" in "not_measured:"*) ;; *) fail_arm 10 "the message must start with not_measured:";; esac
needs 10 ".pmat.yaml" "--github-snapshot"
echo "roadmap-coherence-control: arm 10 N/M  — snapshot path committed in .pmat.yaml refused (exit 1)"

# ── arm 11: RED — grace_minutes is read: 10 makes a 30-minute disagreement a DRIFT ─
recent="$(date -u -d '30 minutes ago' +%Y-%m-%dT%H:%M:%SZ)"   # bashrs disable-line=DET002
write_roadmap "$(item PMAT-001 'old title' planned 1 "$recent")"
GRACE=10 write_snapshot "$(issue 1 'new title' open '[]' "$recent")"
run_gate
[ "$RC" -eq 1 ] || fail_arm 11 "grace_minutes: 10 must make a 30-minute-old disagreement a finding (exit 1)"
[ "$STATUS" = "Fail" ] || fail_arm 11 "CB-2115 must be Fail under grace_minutes: 10"
needs 11 "DRIFT" "PMAT-001"
write_snapshot "$(issue 1 'new title' open '[]' "$recent")"
run_gate
[ "$RC" -eq 0 ] || fail_arm 11 "the same pair under the default 60-minute grace must exit 0 — the option was read both ways"
[ "$STATUS" = "Pass" ] || fail_arm 11 "CB-2115 must be Pass under the default grace"
echo "roadmap-coherence-control: arm 11 RED  — grace_minutes: 10 refused the 30-minute DRIFT that the default tolerates (exit 1, then 0)"

echo "roadmap-coherence-control: all 11 arms behaved — CB-2115 can fail, can pass, says why, and refuses its own bypasses"
