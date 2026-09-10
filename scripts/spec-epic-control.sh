#!/usr/bin/env bash
# spec-epic-control.sh — prove CB-2110 can fail before trusting it
# (goal-mode.md §7.1 "CONTROL FIRST"; PMAT-728, goal-mode step 6).
#
#   scripts/spec-epic-control.sh <path-to-pmat-binary>
#
# CB-2110 (invariant E, goal-mode.md §4.3): every file under
# docs/specifications/ opens with YAML front-matter (`epic:`, `status:`,
# `vendors:`), and every `status: active` spec names in `epic:` an OPEN GitHub
# issue labelled `epic` that has at least one sub-issue — membership is
# GitHub's native sub-issue relation, read into the snapshot for every issue
# labelled epic, never a label or a title convention. `status: historical`
# and `status: superseded` take a spec off the epic leg, never off the parse
# leg, and every verdict names the exempt specs so the escape hatch is
# visible. The parse leg is offline; the snapshot is loaded only when an
# active spec names an epic. CB-148, the rule this one supersedes, prints
# RETIRED for one minor version and judges nothing (§11).
#
# A gate that cannot fail is theater. This script plants the defects the rule
# exists to catch in a throwaway project — specs, a minimal roadmap naming the
# repository, and a GitHub snapshot file handed to the rule on the command
# line, so no arm touches the network — and asserts the verdict on every arm,
# the RED arms AND the GREEN ones, because a rule that always fails would pass
# the RED arms for the wrong reason:
#
#   arm 1  RED   §7 falsifier: the active spec's epic: line deleted            exit 1, Fail NO-EPIC docs/specifications/a.md, the fixer named
#   arm 2  GREEN a.md active epic 7; #7 open, labelled epic, sub_issues 1     exit 0, Pass linked 1, exempt none, snapshot from the file
#   arm 3  RED   #7 is closed                                                  exit 1, Fail EPIC-CLOSED a.md names #7
#   arm 4  RED   #7 is absent from the snapshot                                exit 1, Fail EPIC-ABSENT a.md names #7
#   arm 5  RED   #7 open but labelled ["bug","Epic"] — the label is lowercase  exit 1, Fail NOT-AN-EPIC a.md names #7
#   arm 6  RED   #7 has sub_issues 0                                           exit 1, Fail NO-SUB-ISSUES a.md #7
#   arm 7  N/M   #7 carries no sub_issues key at all                           exit 1, Fail not_measured: SUB-ISSUES-UNMEASURED a.md — and NOT NO-SUB-ISSUES
#   arm 8  RED   components/c.md with no front-matter (a.md still bound)      exit 1, Fail NO-FRONT-MATTER components/c.md — and NOT NO-EPIC for it
#   arm 9  RED   h.md historical with epic: x                                  exit 1, Fail BAD-FRONT-MATTER h.md — the parse leg judges a historical spec
#   arm 10 GREEN h.md historical + s.md superseded epic 99 + a.md bound        exit 0, Pass, both exempt specs named; then #7 closed: Fail, exempt list kept, #99 never judged
#   arm 11 SKIP  no docs/specifications directory at all                       exit 0, Skip naming docs/specifications
#   arm 12 N/M   a.md names epic 7, no --github-snapshot, no repository        exit 1, Fail not_measured: no GitHub repository resolves — exempt clause kept
#   arm 13 N/M   a snapshot path committed in .pmat.yaml (the bypass token)    exit 1, Fail not_measured naming .pmat.yaml, --github-snapshot, cb-2110
#   arm 14 RED   a.md epic null, no --github-snapshot, no repository           exit 1, Fail NO-EPIC a.md, measured — the parse leg needs no network
#   arm 15 RETIRED `--checks CB-148` on the arm-2 fixture                       exit 0, the row "CB-148: RETIRED — superseded by CB-2110" is Skip naming goal-mode.md §11
#   arm 16 THIS TREE this repository's own docs/specifications, epics #1017/#1018/#1019 open with 0 sub-issues
#                                                                              exit 1, Fail "N finding(s) — NO-EPIC N:" with N = the *.md count, exempt none
#   arm 17 RED   ten active specs s00..s09 with epic null                      exit 1, Fail "10 finding(s) — NO-EPIC 10:" and "(+2 more)" — the ninth is counted, not dropped
#
# Arm 16 is the withheld-step measurement (goal-mode.md §11 step 6, doctrine
# 6): it runs the rule on THIS tree's specs, copied into the fixture, and
# asserts what the tree measures today — every spec NO-EPIC, because the
# epics do not exist yet. The direct CB-2110 step in CI cannot land until
# they do; when an operator creates them, fills `epic:` and flips the step,
# this arm is what changes first, so the flip cannot happen unnoticed. N is
# computed from the tree, never written down here.
#
# Each arm asserts BOTH the process exit code and the rule's entry in the JSON
# report: the exit code is what CI acts on, the entry is what proves the
# verdict came from this rule and not from another one. A missing entry is
# under-discovery and fails the control (exit 2) — "we found no CB-2110 row"
# must never render as "CB-2110 passed". Every needle is the RENDERED finding
# (`CLASS docs/specifications/<file>:`), never a class word alone and never
# the header count: on PMAT-724 a header that enumerated every class made
# `contains("PREFIXED")` true on any failure.
#
# Exit: 0 every arm behaved · 1 an arm did not (named on stderr) · 2 usage,
# missing binary, or a report the script could not read.
#
# The binary is an ARGUMENT, never resolved from PATH: the control must judge
# the pmat built from this tree, not whichever one is installed.
set -uo pipefail

PMAT="${1:-}"
if [ -z "$PMAT" ] || [ ! -f "$PMAT" ] || [ ! -x "$PMAT" ]; then
  echo "spec-epic-control: usage: $0 <path-to-pmat-binary> (got '${PMAT}')" >&2
  exit 2
fi
command -v jq >/dev/null 2>&1 || { echo "spec-epic-control: jq is required" >&2; exit 2; }
command -v git >/dev/null 2>&1 || { echo "spec-epic-control: git is required" >&2; exit 2; }

here="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d "${TMPDIR:-/tmp}/spec-epic-control.XXXXXX")"
trap 'rm -rf "${work:?}"' EXIT
repo="$work/repo"
report="$work/report.json"
specs="$repo/docs/specifications"

# The rule reads no clock: the timestamps are fixed values.
when="2026-09-08T00:00:00Z"

# The fixture is a git repository with NO remote, so nothing here can fall back
# to a live `gh` call: the rule reads the snapshot named on the command line
# or reports that it could not. Host git configuration is kept out of it.
mkdir -p "$repo/docs/roadmaps" "$specs"
GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null git -C "$repo" init -q -b master

# write_roadmap — the minimal roadmap the rule reads the repository from
# (`github_repo`, the same hint the roadmap rules use). REPO, if set,
# overrides it (arms 12, 14: `null`, so no repository resolves).
write_roadmap() {
  printf 'roadmap_version: "1.0"\ngithub_enabled: true\ngithub_repo: %s\nroadmap: []\n' "${REPO:-paiml/fixture}" >"$repo/docs/roadmaps/roadmap.yaml"
}
# write_spec <rel-path> <epic|null|-> <status>   ("-" writes no epic: line at
# all — the §7 falsifier deletes the line, it does not null it)
write_spec() {
  mkdir -p "$(dirname "$specs/$1")"
  {
    printf -- '---\n'
    if [ "$2" != "-" ]; then printf 'epic: %s\n' "$2"; fi
    printf 'status: %s\nvendors: []\n---\n\n# A spec\n' "$3"
  } >"$specs/$1"
}
# write_raw_spec <rel-path> <text>   the file verbatim, for malformed fronts
write_raw_spec() {
  mkdir -p "$(dirname "$specs/$1")"
  printf '%s\n' "$2" >"$specs/$1"
}
# issue <number> <open|closed> <labels-json-array> <sub_issues|->   ("-" omits
# the sub_issues key — what a snapshot written before the field looks like:
# not measured, never zero)
issue() {
  if [ "$4" = "-" ]; then
    jq -cn --argjson n "$1" --arg s "$2" --argjson l "$3" --arg u "$when" \
      '{number:$n, title:("issue " + ($n|tostring)), state:$s, labels:$l, milestone:null, updated_at:$u}'
  else
    jq -cn --argjson n "$1" --arg s "$2" --argjson l "$3" --argjson si "$4" --arg u "$when" \
      '{number:$n, title:("issue " + ($n|tostring)), state:$s, labels:$l, milestone:null, updated_at:$u, sub_issues:$si}'
  fi
}
# write_snapshot <issue-json>...
write_snapshot() {
  printf '%s\n' "$@" | jq -s '{repo:"paiml/fixture", taken_at:"2026-09-10T00:00:00Z", issues:., milestones:[]}' >"$repo/snapshot.json"
  rm -f "$repo/.pmat.yaml"
}

# One run of the gate against the fixture, the rule selected (CHECKS, if set,
# selects another — arm 15), judging the fixture's snapshot through the
# command-line flag unless the caller passes its own arguments. Sets RC
# (process exit). Then `judge <PREFIX>` sets STATUS and MESSAGE from the entry
# whose name starts with the prefix; a report with no such entry is exit 2.
run_gate() {
  RC=0
  if [ "$#" -eq 0 ]; then set -- --github-snapshot "$repo/snapshot.json"; fi
  "$PMAT" comply check --checks "${CHECKS:-CB-2110}" --path "$repo" --format json "$@" >"$report" 2>"$work/stderr" || RC=$?
}
judge() {
  local entry
  entry=$(jq -c --arg p "$1" '[.checks[] | select(.name | startswith($p))] | first // empty' "$report" 2>/dev/null || true)
  if [ -z "$entry" ]; then
    echo "spec-epic-control: the report carries no '$1' entry (exit $RC) — under-discovery is a finding, not a pass" >&2
    echo "--- stderr" >&2; head -20 "$work/stderr" >&2
    echo "--- report" >&2; head -c 2000 "$report" >&2; echo >&2
    exit 2
  fi
  STATUS=$(printf '%s' "$entry" | jq -r '.status')
  MESSAGE=$(printf '%s' "$entry" | jq -r '.message')
}

fail_arm() {
  echo "spec-epic-control: ARM $1 FAILED — $2" >&2
  echo "  exit=$RC status=${STATUS:-unjudged}" >&2
  echo "  message=${MESSAGE:-}" >&2
  exit 1
}
# needs <arm> <needle>...  — every needle must appear in MESSAGE
needs() {
  local arm=$1 n; shift
  for n in "$@"; do
    case "$MESSAGE" in *"$n"*) ;; *) fail_arm "$arm" "the message must name '$n'";; esac
  done
}
# lacks <arm> <needle>...  — no needle may appear in MESSAGE (a class that did
# not fire must not be named; a spec off the epic leg must not be judged on it)
lacks() {
  local arm=$1 n; shift
  for n in "$@"; do
    case "$MESSAGE" in *"$n"*) fail_arm "$arm" "the message must NOT contain '$n'";; esac
  done
}
# starts <arm> <prefix>  — MESSAGE must begin with the prefix (the header)
starts() {
  case "$MESSAGE" in "$2"*) ;; *) fail_arm "$1" "the message must start with '$2'";; esac
}
# expect <arm> <RULE-PREFIX> <Pass|Fail|Skip> <needle>...
expect() {
  local arm=$1 rule=$2 status=$3; shift 3
  judge "$rule"
  [ "$STATUS" = "$status" ] || fail_arm "$arm" "$rule must be $status"
  needs "$arm" "$@"
}
not_measured() {
  case "$MESSAGE" in "not_measured:"*) ;; *) fail_arm "$1" "the message must start with not_measured:";; esac
}
measured() {
  case "$MESSAGE" in "not_measured:"*) fail_arm "$1" "the message must NOT start with not_measured: — this leg needs no input it could fail to read";; esac
}

echo "spec-epic-control: fixture $repo (no remote, snapshot on the command line); pmat=$("$PMAT" --version 2>/dev/null | head -1)"

# ── the parse leg ──────────────────────────────────────────────────────────────
# arm 1: RED — the §7 falsifier: delete the active spec's epic: line
write_roadmap
write_spec a.md - active
write_snapshot
run_gate
[ "$RC" -eq 1 ] || fail_arm 1 "an active spec with no epic: line must exit 1"
expect 1 CB-2110 Fail "NO-EPIC docs/specifications/a.md:" "create it on GitHub"
echo "spec-epic-control: arm 1 RED   — a.md active with its epic: line deleted: NO-EPIC, the fixer named (exit 1)"

# ── the epic leg ───────────────────────────────────────────────────────────────
# arm 2: GREEN — a.md names #7; #7 is open, labelled epic, has one sub-issue
write_spec a.md 7 active
write_snapshot "$(issue 7 open '["epic"]' 1)"
run_gate
[ "$RC" -eq 0 ] || fail_arm 2 "an active spec bound to an open epic with a sub-issue must exit 0"
expect 2 CB-2110 Pass "linked 1" "exempt from the epic leg: none" "snapshot: file"
echo "spec-epic-control: arm 2 GREEN — a.md <-> open #7 labelled epic with 1 sub-issue: Pass linked 1 (exit 0)"

# arm 3: RED — #7 is closed
write_snapshot "$(issue 7 closed '["epic"]' 1)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 3 "an active spec naming a closed epic must exit 1"
expect 3 CB-2110 Fail "EPIC-CLOSED docs/specifications/a.md: names #7"
echo "spec-epic-control: arm 3 RED   — closed #7 refused: EPIC-CLOSED (exit 1)"

# arm 4: RED — #7 is absent from the snapshot
write_snapshot
run_gate
[ "$RC" -eq 1 ] || fail_arm 4 "an active spec naming an issue the snapshot does not carry must exit 1"
expect 4 CB-2110 Fail "EPIC-ABSENT docs/specifications/a.md: names #7"
echo "spec-epic-control: arm 4 RED   — absent #7 refused: EPIC-ABSENT (exit 1)"

# arm 5: RED — #7 is open but not labelled epic (the label is the lowercase word; "Epic" is not it)
write_snapshot "$(issue 7 open '["bug","Epic"]' 1)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 5 "an active spec naming an open issue that is not labelled epic must exit 1"
expect 5 CB-2110 Fail "NOT-AN-EPIC docs/specifications/a.md: names #7"
echo "spec-epic-control: arm 5 RED   — #7 labelled bug and Epic, not epic, refused: NOT-AN-EPIC (exit 1)"

# arm 6: RED — #7 has no sub-issue
write_snapshot "$(issue 7 open '["epic"]' 0)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 6 "an epic with zero sub-issues must exit 1 — a spec's tickets are its epic's sub-issues"
expect 6 CB-2110 Fail "NO-SUB-ISSUES docs/specifications/a.md: #7 has no sub-issue"
echo "spec-epic-control: arm 6 RED   — #7 with 0 sub-issues refused: NO-SUB-ISSUES (exit 1)"

# arm 7: not measured — the snapshot carries no sub_issues key for #7: not zero, unknown
write_snapshot "$(issue 7 open '["epic"]' -)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 7 "a sub-issue count the snapshot does not carry must exit 1, never read as zero and never as pass"
expect 7 CB-2110 Fail "SUB-ISSUES-UNMEASURED docs/specifications/a.md:"; not_measured 7
lacks 7 "NO-SUB-ISSUES"
echo "spec-epic-control: arm 7 N/M   — #7 without a sub_issues key reported not_measured: SUB-ISSUES-UNMEASURED, not NO-SUB-ISSUES (exit 1)"

# ── the parse leg, beside a bound spec ─────────────────────────────────────────
# arm 8: RED — a second spec with no front-matter, in a subdirectory
write_snapshot "$(issue 7 open '["epic"]' 1)"
write_raw_spec components/c.md '# C

A spec with no front-matter.'
run_gate
[ "$RC" -eq 1 ] || fail_arm 8 "a spec with no front-matter must exit 1"
expect 8 CB-2110 Fail "NO-FRONT-MATTER docs/specifications/components/c.md:"
lacks 8 "NO-EPIC docs/specifications/components/c.md"
rm -f "$specs/components/c.md"
echo "spec-epic-control: arm 8 RED   — components/c.md without front-matter refused: NO-FRONT-MATTER, not also NO-EPIC (exit 1)"

# arm 9: RED — a historical spec whose epic: is not a number: the parse leg judges it
write_raw_spec h.md '---
epic: x
status: historical
---
# H'
run_gate
[ "$RC" -eq 1 ] || fail_arm 9 "a historical spec with a malformed epic: must exit 1 — status exempts the epic leg, never the parse leg"
expect 9 CB-2110 Fail 'BAD-FRONT-MATTER docs/specifications/h.md: epic: "x" is neither an issue number nor null'
rm -f "$specs/h.md"
echo "spec-epic-control: arm 9 RED   — historical h.md with epic: x refused: BAD-FRONT-MATTER (exit 1)"

# arm 10: GREEN — historical and superseded specs are off the epic leg and named on every verdict
write_spec h.md null historical
write_spec s.md 99 superseded
run_gate
[ "$RC" -eq 0 ] || fail_arm 10 "a bound active spec beside a historical and a superseded one must exit 0"
expect 10 CB-2110 Pass "exempt from the epic leg: docs/specifications/h.md (historical), docs/specifications/s.md (superseded)" "3 spec(s)" "linked 1"
write_snapshot "$(issue 7 closed '["epic"]' 1)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 10 "the same fixture with #7 closed must exit 1"
expect 10 CB-2110 Fail "EPIC-CLOSED docs/specifications/a.md: names #7" "exempt from the epic leg: docs/specifications/h.md (historical), docs/specifications/s.md (superseded)"
lacks 10 "#99"
rm -f "$specs/h.md" "$specs/s.md"
echo "spec-epic-control: arm 10 GREEN — h.md (historical) and s.md (superseded) named exempt on Pass and on Fail; #99 never judged (exit 0, then 1)"

# ── the preamble ───────────────────────────────────────────────────────────────
# arm 11: skip — no docs/specifications at all
rm -rf "${specs:?}"
write_snapshot "$(issue 7 open '["epic"]' 1)"
run_gate
[ "$RC" -eq 0 ] || fail_arm 11 "a project with no docs/specifications must exit 0"
expect 11 CB-2110 Skip "no docs/specifications"
mkdir -p "$specs"
echo "spec-epic-control: arm 11 SKIP  — no docs/specifications directory: Skip naming it (exit 0)"

# arm 12: not measured — an active spec names an epic, no snapshot on the
# command line, and no repository resolves (github_repo null, no remote):
# the epic leg needs an input it cannot get. The exempt clause is still there.
REPO=null write_roadmap
write_spec a.md 7 active
run_gate --checks CB-2110
[ "$RC" -eq 1 ] || fail_arm 12 "an epic the rule cannot look up must exit 1 — a missing input is not a pass"
expect 12 CB-2110 Fail "no GitHub repository resolves" "exempt from the epic leg:"; not_measured 12
echo "spec-epic-control: arm 12 N/M   — epic 7 named, no snapshot, no repository: not_measured, exempt clause kept (exit 1)"

# arm 13: not measured — a snapshot path committed in .pmat.yaml is refused, never read
write_roadmap
write_snapshot "$(issue 7 open '["epic"]' 1)"
printf 'comply:\n  checks:\n    cb-2110:\n      options:\n        snapshot: snapshot.json\n' >"$repo/.pmat.yaml"
run_gate
[ "$RC" -eq 1 ] || fail_arm 13 "a snapshot path in .pmat.yaml under cb-2110 must be refused (exit 1), never read"
expect 13 CB-2110 Fail ".pmat.yaml" "--github-snapshot" "cb-2110"; not_measured 13
rm -f "$repo/.pmat.yaml"
echo "spec-epic-control: arm 13 N/M   — snapshot path committed in .pmat.yaml refused under cb-2110 (exit 1)"

# arm 14: RED — the parse leg needs no network: epic null, no snapshot, no repository, still NO-EPIC and measured
REPO=null write_roadmap
write_spec a.md null active
run_gate --checks CB-2110
[ "$RC" -eq 1 ] || fail_arm 14 "an active spec with epic null must exit 1 with no snapshot and no repository — the parse leg is offline"
expect 14 CB-2110 Fail "NO-EPIC docs/specifications/a.md:" "snapshot: not needed"; measured 14
echo "spec-epic-control: arm 14 RED   — epic null with no snapshot and no repository: NO-EPIC, measured, snapshot not needed (exit 1)"

# ── the retired rule ───────────────────────────────────────────────────────────
# arm 15: RETIRED — CB-148 prints its retirement and judges nothing (§11: retired, not patched)
write_roadmap
write_spec a.md 7 active
write_snapshot "$(issue 7 open '["epic"]' 1)"
CHECKS=CB-148 run_gate
[ "$RC" -eq 0 ] || fail_arm 15 "a run selecting the retired CB-148 must exit 0 — it judges nothing"
expect 15 "CB-148: RETIRED — superseded by CB-2110" Skip "goal-mode.md §11"
echo "spec-epic-control: arm 15 RETIRED — CB-148 prints 'RETIRED — superseded by CB-2110' as Skip, naming goal-mode.md §11 (exit 0)"

# ── this tree ──────────────────────────────────────────────────────────────────
# arm 16: THIS TREE — the withheld-step measurement. This repository's own
# specs, copied into the fixture beside a snapshot of the three epic issues
# that exist today (#1017/#1018/#1019: open, labelled epic, 0 sub-issues).
# Every spec reads NO-EPIC; N is counted from the tree, never written here.
rm -rf "${specs:?}"
cp -R "$here/docs/specifications" "$specs"
count=$(find "$here/docs/specifications" -name '*.md' | wc -l | tr -d ' ')
[ "$count" -gt 0 ] || { echo "spec-epic-control: arm 16 found no *.md under $here/docs/specifications — the control is running in the wrong tree" >&2; exit 2; }
write_snapshot "$(issue 1017 open '["epic"]' 0)" "$(issue 1018 open '["epic"]' 0)" "$(issue 1019 open '["epic"]' 0)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 16 "this tree's $count specs must exit 1 today — the epics do not exist yet (goal-mode.md §11 step 6 is withheld); if they now do, flip the direct step (PMAT-729) and retire this arm"
expect 16 CB-2110 Fail "exempt from the epic leg: none"
starts 16 "$count finding(s) — NO-EPIC $count:"
echo "spec-epic-control: arm 16 THIS TREE — $count specs, all NO-EPIC, none exempt: the direct step stays withheld until the epics exist (exit 1)"

# arm 17: RED — more than eight findings: the ninth and tenth are counted, not dropped
rm -rf "${specs:?}"
for i in 0 1 2 3 4 5 6 7 8 9; do write_spec "s0$i.md" null active; done
write_snapshot
run_gate
[ "$RC" -eq 1 ] || fail_arm 17 "ten active specs with epic null must exit 1"
expect 17 CB-2110 Fail "(+2 more)" "NO-EPIC docs/specifications/s00.md:" "NO-EPIC docs/specifications/s07.md:"
starts 17 "10 finding(s) — NO-EPIC 10:"
lacks 17 "NO-EPIC docs/specifications/s08.md:"
echo "spec-epic-control: arm 17 RED   — ten NO-EPIC specs: eight rendered, +2 more counted (exit 1)"

echo "spec-epic-control: all 17 arms behaved — CB-2110 can fail, can pass, says why, names its exemptions, refuses its own bypass, and this tree measures NO-EPIC on every spec"
