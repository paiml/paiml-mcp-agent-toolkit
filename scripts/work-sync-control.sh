#!/usr/bin/env bash
# work-sync-control.sh — prove `pmat work sync --check-only` can fail before trusting it
# (goal-mode.md §5, §7.1 "CONTROL FIRST"; PMAT-720).
#
#   scripts/work-sync-control.sh <path-to-pmat-binary>
#
# A gate that cannot fail is theater. Every arm feeds the command a snapshot from
# a file, so nothing here touches GitHub, and asserts BOTH the exit code and the
# JSON report — the exit is what a pipeline acts on, the report is what proves the
# verdict came from the finding it names:
#
#   arm 1  RED   a planted ORPHAN-ROADMAP, ORPHAN-GITHUB and COLLISION   exit 1, each named in the report
#   arm 2  GREEN a bijection                                             exit 0, coherent=true
#   arm 3  DRY   --dry-run on arm 1's fixture                            exit 0, the roadmap's bytes unchanged,
#                                                                        the plan names the two writes and the two COLLISION skips
#   arm 4  RT    this repository's docs/roadmaps/roadmap.yaml through a real github-to-yaml write:
#                one orphan issue becomes GH-999999 and every pre-existing line is byte-identical (§4.2).
#                This arm proves the SERIALIZER, not the fixers (arms 1-3 do): its snapshot names none of
#                the real issues on purpose, so every real item is skipped and the only write is the
#                appended control item — a real issue in that snapshot would let the arm modify the
#                repository's own entries, which is not what a control is for.
#
# Exit: 0 every arm behaved · 1 an arm did not (named on stderr) · 2 usage,
# missing binary, jq absent. The binary is an ARGUMENT, never resolved from PATH.
set -uo pipefail

PMAT="${1:-}"
if [ -z "$PMAT" ] || [ ! -x "$PMAT" ]; then
  echo "work-sync-control: usage: $0 <path-to-pmat-binary> (got '${PMAT}')" >&2
  exit 2
fi
command -v jq >/dev/null 2>&1 || { echo "work-sync-control: jq is required" >&2; exit 2; }
here="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d "${TMPDIR:-/tmp}/work-sync-control.XXXXXX")"
trap 'rm -rf "${work:?}"' EXIT

# mk <dir> <items (yaml, \n-escaped)> <issues (json, comma-joined)>
mk() {
  mkdir -p "$1/docs/roadmaps"
  printf "roadmap_version: '1.0'\ngithub_enabled: true\ngithub_repo: paiml/control\nroadmap:\n%b" "$2" > "$1/docs/roadmaps/roadmap.yaml"
  printf '{"repo":"paiml/control","taken_at":"2026-09-09T12:00:00Z","issues":[%s],"milestones":[{"title":"3.42.0","state":"open"}]}' "$3" > "$1/snapshot.json"
}
# run <dir> <flags…> — sets RC; the JSON report lands in $work/out.json
run() {
  RC=0
  "$PMAT" work sync --path "$1" --snapshot "$1/snapshot.json" --format json "${@:2}" > "$work/out.json" 2> "$work/err" || RC=$?
}
fail_arm() {
  echo "work-sync-control: ARM $1 FAILED — $2" >&2
  echo "  exit=$RC" >&2
  echo "--- report" >&2; head -c 2000 "$work/out.json" >&2; echo >&2
  echo "--- stderr" >&2; head -20 "$work/err" >&2
  exit 1
}

ITEM_A='- id: A\n  github_issue: null\n  item_type: task\n  title: alpha\n  status: planned\n'
ITEM_B='- id: B\n  github_issue: 2\n  item_type: task\n  title: beta\n  status: planned\n'
ITEM_M0='- id: M0\n  github_issue: 612\n  item_type: task\n  title: zero\n  status: planned\n'
ITEM_M1='- id: M1\n  github_issue: 612\n  item_type: task\n  title: one\n  status: planned\n'
ISSUE_2='{"number":2,"title":"beta","state":"open","labels":[],"updated_at":"2026-09-09T12:00:00Z"}'
ISSUE_9='{"number":9,"title":"nine","state":"open","labels":[],"milestone":"3.42.0","updated_at":"2026-09-09T12:00:00Z"}'
ISSUE_612='{"number":612,"title":"macs","state":"closed","state_reason":"completed","labels":[],"updated_at":"2026-09-09T12:00:00Z"}'

echo "work-sync-control: pmat=$("$PMAT" --version 2>/dev/null | head -1)"

# ── arm 1: RED ───────────────────────────────────────────────────────────────
red="$work/red"
mk "$red" "$ITEM_A$ITEM_B$ITEM_M0$ITEM_M1" "$ISSUE_2,$ISSUE_9,$ISSUE_612"
run "$red" --check-only
[ "$RC" -eq 1 ] || fail_arm 1 "planted defects must exit 1"
jq -e '.coherent == false' "$work/out.json" >/dev/null || fail_arm 1 "coherent must be false"
jq -e '[.findings[] | select(.class=="COLLISION" and .number==612 and .ids==["M0","M1"])] | length == 1' "$work/out.json" >/dev/null || fail_arm 1 "the COLLISION on #612 must name M0 and M1"
jq -e '[.findings[] | select(.class=="ORPHAN-ROADMAP" and .id=="A" and .reason=="no-issue")] | length == 1' "$work/out.json" >/dev/null || fail_arm 1 "A must be ORPHAN-ROADMAP (no-issue)"
jq -e '[.findings[] | select(.class=="ORPHAN-GITHUB" and .number==9)] | length == 1' "$work/out.json" >/dev/null || fail_arm 1 "#9 must be ORPHAN-GITHUB"
jq -e '(.findings | length) == 3 and .matched == 1' "$work/out.json" >/dev/null || fail_arm 1 "exactly 3 findings and 1 matched pair (B)"
echo "work-sync-control: arm 1 RED   — COLLISION #612 (M0, M1), ORPHAN-ROADMAP A, ORPHAN-GITHUB #9 named (exit 1)"

# ── arm 2: GREEN ─────────────────────────────────────────────────────────────
green="$work/green"
mk "$green" "$ITEM_B" "$ISSUE_2"
run "$green" --check-only
[ "$RC" -eq 0 ] || fail_arm 2 "a bijection must exit 0"
jq -e '.coherent == true and .matched == 1 and (.findings | length) == 0' "$work/out.json" >/dev/null || fail_arm 2 "coherent, matched 1, no findings"
echo "work-sync-control: arm 2 GREEN — bijection coherent (exit 0)"

# ── arm 3: DRY ───────────────────────────────────────────────────────────────
before="$(sha256sum "$red/docs/roadmaps/roadmap.yaml")"
run "$red" --direction full --dry-run
[ "$RC" -eq 0 ] || fail_arm 3 "a dry run must exit 0"
[ "$(sha256sum "$red/docs/roadmaps/roadmap.yaml")" = "$before" ] || fail_arm 3 "a dry run must not write the roadmap"
jq -e '.dry_run == true and .applied == 0
       and ([.actions[] | select(.action=="create-issue" and .id=="A")] | length) == 1
       and ([.actions[] | select(.action=="create-item" and .number==9)] | length) == 1
       and ([.actions[] | select(.action=="skip" and (.reason | test("COLLISION")))] | length) == 2' "$work/out.json" >/dev/null \
  || fail_arm 3 "the plan must name create-issue A, create-item #9 and two COLLISION skips"
echo "work-sync-control: arm 3 DRY   — plan names the writes, nothing written (exit 0)"

# ── arm 4: RT — the real roadmap round-trips ─────────────────────────────────
rt="$work/rt"
mkdir -p "$rt/docs/roadmaps"
cp "$here/docs/roadmaps/roadmap.yaml" "$rt/docs/roadmaps/roadmap.yaml"
cp "$rt/docs/roadmaps/roadmap.yaml" "$work/orig.yaml"
orig_lines="$(wc -l < "$work/orig.yaml")"
printf '{"repo":"paiml/control","taken_at":"2026-09-09T12:00:00Z","issues":[{"number":999999,"title":"control orphan","state":"open","labels":[],"milestone":"3.42.0","updated_at":"2026-09-09T12:00:00Z"}],"milestones":[]}' > "$rt/snapshot.json"
run "$rt" --direction github-to-yaml
[ "$RC" -eq 0 ] || fail_arm 4 "github-to-yaml on the real roadmap must exit 0"
jq -e '.applied == 1' "$work/out.json" >/dev/null || fail_arm 4 "exactly one roadmap change (the GH-999999 item); every other finding is a skip"
grep -q '^- id: GH-999999$' "$rt/docs/roadmaps/roadmap.yaml" || fail_arm 4 "GH-999999 must be appended"
head -n "$orig_lines" "$rt/docs/roadmaps/roadmap.yaml" > "$work/prefix.yaml"
if ! diff -q "$work/prefix.yaml" "$work/orig.yaml" >/dev/null; then
  diff "$work/prefix.yaml" "$work/orig.yaml" | head -20 >&2
  fail_arm 4 "the $orig_lines pre-existing lines must round-trip byte-for-byte (§4.2) — an entry was hand-wrapped in a form the serializer does not produce; canonicalise it once with any pmat work command that saves the roadmap (a no-op \`pmat work edit <id> -s <its current status>\` will do)"
fi
echo "work-sync-control: arm 4 RT    — $orig_lines lines of the real roadmap round-tripped byte-for-byte; GH-999999 appended"

echo "work-sync-control: all 4 arms behaved — --check-only can fail and can pass; every write is planned before it happens"
