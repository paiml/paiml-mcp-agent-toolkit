#!/usr/bin/env bash
# ticket-release-control.sh — prove CB-2112 and CB-2114 can fail before
# trusting them (goal-mode.md §7.1 "CONTROL FIRST"; PMAT-724, goal-mode step 5).
#
#   scripts/ticket-release-control.sh <path-to-pmat-binary>
#
# A gate that cannot fail is theater. This script plants the defects the two
# rules exist to catch in a throwaway project — a roadmap and a GitHub
# snapshot file handed to the rules on the command line, so no arm touches the
# network — and asserts the verdict on every arm, the RED arms AND the GREEN
# ones, because a rule that always fails would pass the RED arms for the wrong
# reason:
#
#   CB-2112 (invariant A: every open item names an open issue numbered as its tail)
#   arm 1  RED   §7 falsifier: github_issue is null                          exit 1, Fail, NO-ISSUE PMAT-001
#   arm 2  GREEN PMAT-001 names open #1                                      exit 0, Pass, linked 1
#   arm 3  RED   #1 is closed                                                exit 1, Fail, ISSUE-CLOSED #1
#   arm 4  RED   PMAT-001 names open #7 — the number is not the tail         exit 1, Fail, TAIL-MISMATCH #7
#   arm 5  RED   #1 carries the no-roadmap label                             exit 1, Fail, ISSUE-EXCLUDED no-roadmap
#   CB-2114 (invariants B/F1: every open item carries release:, the milestone exists, the issue is on it)
#   arm 6  RED   §7 falsifier: release: removed                              exit 1, Fail, NO-RELEASE PMAT-001
#   arm 7  GREEN release "3.41.0", milestone 3.41.0 exists, #1 is on it      exit 0, Pass, bound 1
#   arm 8  RED   no milestone titled 3.41.0                                  exit 1, Fail, NO-MILESTONE 3.41.0
#   arm 9  RED   #1 is on milestone 3.42.0                                   exit 1, Fail, NOT-ON-MILESTONE 3.42.0
#   arm 10 RED   release "v3.41.0" — the key is the bare string (§4.1)       exit 1, Fail, PREFIXED v3.41.0
#   shared preamble (both rules, one run)
#   arm 11 N/M   the snapshot named on the command line does not exist       exit 1, both Fail, messages start not_measured:
#   arm 12 SKIP  no roadmap, never committed                                 exit 0, both Skip, name docs/roadmaps/roadmap.yaml
#   arm 13 N/M   the roadmap was committed and then deleted                  exit 1, both Fail, not_measured: "committed and is now gone"
#   arm 14 N/M   a snapshot path committed in .pmat.yaml (the bypass token)  exit 1, the named rule Fail not_measured naming .pmat.yaml and --github-snapshot
#   arm 15 GREEN a completed item with no issue and no release               exit 0, both Pass, "0 open item" — scope is §4.2's
#
# Each arm asserts BOTH the process exit code and the rule's entry in the JSON
# report: the exit code is what CI acts on, the entry is what proves the
# verdict came from this rule and not from another one. A missing entry is
# under-discovery and fails the control (exit 2) — "we found no CB-2112 row"
# must never render as "CB-2112 passed".
#
# Every run selects BOTH rules (`--checks CB-2112,CB-2114`) so an arm that
# plants a CB-2112 defect also proves CB-2114 did not fire on it, and vice
# versa: a defect must be caught by the rule that owns it.
#
# Exit: 0 every arm behaved · 1 an arm did not (named on stderr) · 2 usage,
# missing binary, or a report the script could not read.
#
# The binary is an ARGUMENT, never resolved from PATH: the control must judge
# the pmat built from this tree, not whichever one is installed.
set -uo pipefail

PMAT="${1:-}"
if [ -z "$PMAT" ] || [ ! -x "$PMAT" ]; then
  echo "ticket-release-control: usage: $0 <path-to-pmat-binary> (got '${PMAT}')" >&2
  exit 2
fi
command -v jq >/dev/null 2>&1 || { echo "ticket-release-control: jq is required" >&2; exit 2; }
command -v git >/dev/null 2>&1 || { echo "ticket-release-control: git is required" >&2; exit 2; }

work="$(mktemp -d "${TMPDIR:-/tmp}/ticket-release-control.XXXXXX")"
trap 'rm -rf "${work:?}"' EXIT
repo="$work/repo"
report="$work/report.json"

# The rules read no clock: the timestamps are fixed values.
when="2026-09-08T00:00:00Z"

# The fixture is a git repository with NO remote, so nothing here can fall back
# to a live `gh` call: the rules read the snapshot named on the command line
# or report that they could not. Host git configuration is kept out of it.
mkdir -p "$repo/docs/roadmaps"
GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null git -C "$repo" init -q -b master

# write_roadmap <item-yaml>...   one argument per item (`$(...)` strips the
# trailing newline, so the newline between items is added HERE).
write_roadmap() {
  {
    printf 'roadmap_version: "1.0"\ngithub_enabled: true\ngithub_repo: paiml/fixture\nroadmap:\n'
    for it in "$@"; do printf '%s\n' "$it"; done
  } >"$repo/docs/roadmaps/roadmap.yaml"
}
# item <id> <title> <status> <issue|null> <release|->   ("-" writes no release: line,
# exactly as the serializer's skip_serializing_if does)
item() {
  printf '  - id: %s\n    title: %s\n    status: %s\n    github_issue: %s\n    updated: %s\n' "$1" "$2" "$3" "$4" "$when"
  if [ "$5" != "-" ]; then printf '    release: "%s"\n' "$5"; fi
}
# issue <number> <title> <open|closed> <labels-json-array> <milestone-title|->
issue() {
  local ms=null
  if [ "$5" != "-" ]; then ms="\"$5\""; fi
  jq -cn --argjson n "$1" --arg t "$2" --arg s "$3" --argjson l "$4" --argjson m "$ms" --arg u "$when" \
    '{number:$n, title:$t, state:$s, labels:$l, milestone:$m, updated_at:$u}'
}
# write_snapshot <issue-json>...   (MILESTONES, if set, is a comma-separated
# list of open milestone titles; otherwise the snapshot carries none)
write_snapshot() {
  local ms='[]'
  if [ -n "${MILESTONES:-}" ]; then
    ms=$(printf '%s' "$MILESTONES" | jq -Rc 'split(",") | map({title:., state:"open"})')
  fi
  printf '%s\n' "$@" | jq -s --argjson ms "$ms" '{repo:"paiml/fixture", taken_at:"2026-09-10T00:00:00Z", issues:., milestones:$ms}' >"$repo/snapshot.json"
  rm -f "$repo/.pmat.yaml"
}

# One run of the gate against the fixture, both rules selected, judging the
# fixture's snapshot through the command-line flag unless the caller passes
# its own arguments. Sets RC (process exit). Then `judge <RULE>` sets STATUS
# and MESSAGE from that rule's entry; a report with no such entry is exit 2.
run_gate() {
  RC=0
  if [ "$#" -eq 0 ]; then set -- --github-snapshot "$repo/snapshot.json"; fi
  "$PMAT" comply check --checks CB-2112,CB-2114 --path "$repo" --format json "$@" >"$report" 2>"$work/stderr" || RC=$?
}
judge() {
  local entry
  entry=$(jq -c --arg p "$1" '[.checks[] | select(.name | startswith($p))] | first // empty' "$report" 2>/dev/null || true)
  if [ -z "$entry" ]; then
    echo "ticket-release-control: the report carries no $1 entry (exit $RC) — under-discovery is a finding, not a pass" >&2
    echo "--- stderr" >&2; head -20 "$work/stderr" >&2
    echo "--- report" >&2; head -c 2000 "$report" >&2; echo >&2
    exit 2
  fi
  STATUS=$(printf '%s' "$entry" | jq -r '.status')
  MESSAGE=$(printf '%s' "$entry" | jq -r '.message')
}

fail_arm() {
  echo "ticket-release-control: ARM $1 FAILED — $2" >&2
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
# expect <arm> <RULE> <Pass|Fail|Skip> <needle>...
expect() {
  local arm=$1 rule=$2 status=$3; shift 3
  judge "$rule"
  [ "$STATUS" = "$status" ] || fail_arm "$arm" "$rule must be $status"
  needs "$arm" "$@"
}
not_measured() {
  case "$MESSAGE" in "not_measured:"*) ;; *) fail_arm "$1" "the $2 message must start with not_measured:";; esac
}

echo "ticket-release-control: fixture $repo (no remote, snapshot on the command line); pmat=$("$PMAT" --version 2>/dev/null | head -1)"

# ── CB-2112 ────────────────────────────────────────────────────────────────────
# arm 1: RED — the §7 falsifier: null one github_issue
write_roadmap "$(item PMAT-001 'planned work' planned null 3.41.0)"
MILESTONES=3.41.0 write_snapshot
run_gate
[ "$RC" -eq 1 ] || fail_arm 1 "an open item with no issue must exit 1"
expect 1 CB-2112 Fail "NO-ISSUE" "PMAT-001"
echo "ticket-release-control: arm 1 RED   — PMAT-001 with github_issue null refused by CB-2112 (exit 1)"

# arm 2: GREEN — PMAT-001 names open #1 (on its milestone, so CB-2114 is quiet too)
write_roadmap "$(item PMAT-001 'planned work' planned 1 3.41.0)"
MILESTONES=3.41.0 write_snapshot "$(issue 1 'planned work' open '[]' 3.41.0)"
run_gate
[ "$RC" -eq 0 ] || fail_arm 2 "a linked, bound item must exit 0"
expect 2 CB-2112 Pass "linked 1" "snapshot.json"
expect 2 CB-2114 Pass "bound 1"
echo "ticket-release-control: arm 2 GREEN — PMAT-001 <-> open #1 on milestone 3.41.0: both rules pass (exit 0)"

# arm 3: RED — #1 is closed
MILESTONES=3.41.0 write_snapshot "$(issue 1 'planned work' closed '[]' 3.41.0)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 3 "an open item naming a closed issue must exit 1"
expect 3 CB-2112 Fail "ISSUE-CLOSED" "#1"
echo "ticket-release-control: arm 3 RED   — closed #1 refused by CB-2112 (exit 1)"

# arm 4: RED — the number is not the tail
write_roadmap "$(item PMAT-001 'planned work' planned 7 3.41.0)"
MILESTONES=3.41.0 write_snapshot "$(issue 7 'planned work' open '[]' 3.41.0)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 4 "an item naming an issue that is not its tail must exit 1"
expect 4 CB-2112 Fail "TAIL-MISMATCH" "PMAT-001" "#7"
expect 4 CB-2114 Pass "bound 1"
echo "ticket-release-control: arm 4 RED   — PMAT-001 naming #7 refused by CB-2112 alone (exit 1)"

# arm 5: RED — #1 carries the no-roadmap label
write_roadmap "$(item PMAT-001 'planned work' planned 1 3.41.0)"
MILESTONES=3.41.0 write_snapshot "$(issue 1 'planned work' open '["no-roadmap"]' 3.41.0)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 5 "an item naming a no-roadmap issue must exit 1"
expect 5 CB-2112 Fail "ISSUE-EXCLUDED" "no-roadmap"
echo "ticket-release-control: arm 5 RED   — no-roadmap-labelled #1 refused by CB-2112 (exit 1)"

# ── CB-2114 ────────────────────────────────────────────────────────────────────
# arm 6: RED — the §7 falsifier: remove one release:
write_roadmap "$(item PMAT-001 'planned work' planned 1 -)"
MILESTONES=3.41.0 write_snapshot "$(issue 1 'planned work' open '[]' 3.41.0)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 6 "an open item with no release: must exit 1"
expect 6 CB-2114 Fail "NO-RELEASE" "PMAT-001" "github-to-yaml"
expect 6 CB-2112 Pass "linked 1"
echo "ticket-release-control: arm 6 RED   — PMAT-001 without release: refused by CB-2114 alone (exit 1)"

# arm 7: GREEN — release 3.41.0, the milestone exists, #1 is on it
write_roadmap "$(item PMAT-001 'planned work' planned 1 3.41.0)"
run_gate
[ "$RC" -eq 0 ] || fail_arm 7 "a bound item must exit 0"
expect 7 CB-2114 Pass "bound 1" "snapshot.json"
echo "ticket-release-control: arm 7 GREEN — release 3.41.0 bound to its milestone and issue (exit 0)"

# arm 8: RED — no milestone titled 3.41.0
MILESTONES=3.42.0 write_snapshot "$(issue 1 'planned work' open '[]' 3.41.0)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 8 "a release with no milestone of that title must exit 1"
expect 8 CB-2114 Fail "NO-MILESTONE" "3.41.0"
echo "ticket-release-control: arm 8 RED   — release 3.41.0 with no such milestone refused (exit 1)"

# arm 9: RED — #1 is on 3.42.0, not 3.41.0
MILESTONES=3.41.0,3.42.0 write_snapshot "$(issue 1 'planned work' open '[]' 3.42.0)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 9 "an issue on another milestone must exit 1"
expect 9 CB-2114 Fail "NOT-ON-MILESTONE" "3.42.0" "3.41.0"
echo "ticket-release-control: arm 9 RED   — #1 on milestone 3.42.0 while release says 3.41.0 refused (exit 1)"

# arm 10: RED — a v-prefixed release
write_roadmap "$(item PMAT-001 'planned work' planned 1 v3.41.0)"
MILESTONES=3.41.0 write_snapshot "$(issue 1 'planned work' open '[]' 3.41.0)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 10 "a v-prefixed release must exit 1 (§4.1: the key is the bare string)"
expect 10 CB-2114 Fail "PREFIXED" "v3.41.0"
echo "ticket-release-control: arm 10 RED  — release v3.41.0 refused (exit 1)"

# ── shared preamble ────────────────────────────────────────────────────────────
# arm 11: not measured — the snapshot named on the command line does not exist
write_roadmap "$(item PMAT-001 'planned work' planned 1 3.41.0)"
rm -f "$repo/snapshot.json"
run_gate
[ "$RC" -eq 1 ] || fail_arm 11 "a snapshot the rules cannot read must exit 1, never pass"
for r in CB-2112 CB-2114; do expect 11 "$r" Fail "snapshot.json"; not_measured 11 "$r"; done
echo "ticket-release-control: arm 11 N/M  — missing snapshot reported not_measured by both rules (exit 1)"

# arm 12: skip — no roadmap, never committed
rm -f "$repo/docs/roadmaps/roadmap.yaml"
run_gate
[ "$RC" -eq 0 ] || fail_arm 12 "a project with no roadmap must exit 0"
for r in CB-2112 CB-2114; do expect 12 "$r" Skip "docs/roadmaps/roadmap.yaml"; done
echo "ticket-release-control: arm 12 SKIP — no roadmap, never committed, both rules skip naming the file (exit 0)"

# arm 13: not measured — the roadmap was committed and then deleted
fgit() {
  GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null \
  GIT_AUTHOR_NAME=pmat724 GIT_AUTHOR_EMAIL=pmat724@example.invalid \
  GIT_COMMITTER_NAME=pmat724 GIT_COMMITTER_EMAIL=pmat724@example.invalid \
    git -C "$repo" -c commit.gpgsign=false "$@"
}
write_roadmap "$(item PMAT-001 'planned work' planned 1 3.41.0)"
MILESTONES=3.41.0 write_snapshot "$(issue 1 'planned work' open '[]' 3.41.0)"
fgit add -A && fgit commit -q -m "roadmap"
fgit rm -q docs/roadmaps/roadmap.yaml
run_gate
[ "$RC" -eq 1 ] || fail_arm 13 "a roadmap that was committed and deleted must exit 1 — deleting a gate's input is not a way of passing it"
for r in CB-2112 CB-2114; do expect 13 "$r" Fail "committed and is now gone"; not_measured 13 "$r"; done
echo "ticket-release-control: arm 13 N/M  — committed-then-deleted roadmap reported not_measured by both rules (exit 1)"

# arm 14: not measured — a snapshot path committed in .pmat.yaml is refused, per rule key
fgit checkout -q HEAD -- docs/roadmaps/roadmap.yaml
for key in cb-2112 cb-2114; do
  rule=$(printf '%s' "$key" | tr '[:lower:]' '[:upper:]')
  printf 'comply:\n  checks:\n    %s:\n      options:\n        snapshot: snapshot.json\n' "$key" >"$repo/.pmat.yaml"
  run_gate --checks CB-2112,CB-2114
  [ "$RC" -eq 1 ] || fail_arm 14 "a snapshot path in .pmat.yaml under $key must be refused (exit 1), never read"
  expect 14 "$rule" Fail ".pmat.yaml" "--github-snapshot" "$key"; not_measured 14 "$rule"
done
rm -f "$repo/.pmat.yaml"
echo "ticket-release-control: arm 14 N/M  — snapshot path committed in .pmat.yaml refused under each rule's key (exit 1)"

# arm 15: GREEN — scope: a completed item with nothing is out of both rules' scope
write_roadmap "$(item PMAT-001 'shipped' completed null -)"
write_snapshot
run_gate
[ "$RC" -eq 0 ] || fail_arm 15 "a roadmap of only terminal items must exit 0"
for r in CB-2112 CB-2114; do expect 15 "$r" Pass "0 open item"; done
echo "ticket-release-control: arm 15 GREEN — a completed item is out of scope for both rules (exit 0)"

echo "ticket-release-control: all 15 arms behaved — CB-2112 and CB-2114 can fail, can pass, say why, and refuse their own bypasses"
