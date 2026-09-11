#!/usr/bin/env bash
# spec-review-control.sh — prove CB-2111 can fail before trusting it
# (goal-mode.md §7.1 "CONTROL FIRST"; PMAT-1299, goal-mode step 7).
#
#   scripts/spec-review-control.sh <path-to-pmat-binary>
#
# CB-2111 (invariant E.1, goal-mode.md §6): every `status: active` spec under
# docs/specifications/ has a review artifact, docs/audits/spec-<slug>-review.json,
# whose spec_sha256 is the spec's sha256 NOW, whose plan carries a non-empty
# sha256, whose lanes cover every required role (quality, architecture,
# security, crux, adversarial, and vendor:<name> for each vendor the spec's
# front-matter names), every lane PASS, and partial false. An unrecognised role
# is an error, never an extra lane. The rule reads files and never invokes a
# model, so every arm is offline. `pmat spec review --record <json>` validates
# a review exactly as the rule will and stages it; it never produces one.
#
# A gate that cannot fail is theater. This script plants the defects the rule
# exists to catch in a throwaway git repository and asserts the verdict on
# every arm, the RED arms AND the GREEN ones, because a rule that always fails
# would pass the RED arms for the wrong reason:
#
#   arm 1  GREEN a.md active, a current five-role review, a plan recorded       exit 0, Pass reviewed 1, exempt none
#   arm 2  RED   §7 falsifier: append one space to a.md                         exit 1, Fail STALE-REVIEW a.md, and NOT NO-REVIEW
#   arm 3  RED   the review file deleted                                         exit 1, Fail NO-REVIEW a.md naming docs/audits/spec-a-review.json, the recorder named
#   arm 4  RED   the review file is not JSON                                     exit 1, Fail BAD-REVIEW a.md
#   arm 5  RED   `partial` misspelt (`partail: true`, no `partial`)              exit 1, Fail BAD-REVIEW a.md naming partial: a typo must not default a partial review to complete
#   arm 6  RED   the review names docs/specifications/b.md                       exit 1, Fail SPEC-MISMATCH a.md
#   arm 7  RED   no plan; then a plan whose sha256 is empty                      exit 1, Fail NO-PLAN a.md, both times
#   arm 8  RED   the crux lane missing                                           exit 1, Fail MISSING-ROLE a.md `crux`, and no other role missing
#   arm 9  RED   vendors: [cuda] with the five lanes; then vendor:cuda PASS added  exit 1 MISSING-ROLE `vendor:cuda`; then exit 0 Pass
#   arm 10 RED   the security lane says FAIL                                     exit 1, Fail LANE-NOT-PASS a.md `security` FAIL
#   arm 11 RED   partial: true                                                   exit 1, Fail PARTIAL a.md
#   arm 12 RED   a sixth lane with role `style`                                  exit 1, Fail UNKNOWN-ROLE a.md `style`, and NOT MISSING-ROLE
#   arm 13 GREEN h.md historical and s.md superseded, no reviews, beside reviewed a.md  exit 0, Pass, both named exempt
#   arm 14 RED   c.md with no front-matter beside reviewed a.md                  exit 1, Fail UNJUDGEABLE c.md: never skipped
#   arm 15 RED   a.md with no review beside c.md with no front-matter            exit 1, "2 finding(s) — NO-REVIEW 1, UNJUDGEABLE 1: NO-REVIEW …a.md:": path order
#   arm 16 RED   ten active specs with no review                                 exit 1, "10 finding(s) — NO-REVIEW 10:" and "(+2 more)": the ninth is counted
#   arm 17 RECORD `pmat spec review --record`: a stale review refused, nothing written or staged; a current one written byte for byte and staged, then the gate passes; a review naming README.md, a `..` path or a `.` path refused; a symlink at the artifact path refused and nothing written through it; an artifact path git ignores refused before any write; a directory outside any git work tree refused; a review missing a front-matter vendor's lane refused; a hard link at the artifact path replaced, never written through; a spec path with a carriage return refused; a locked index reported as written and not staged
#   arm 18 SKIP  no docs/specifications, in a repository whose history never held one  exit 0, Skip
#   arm 19 N/M   docs/specifications committed and then deleted                 exit 1, Fail not_measured: "committed and is now gone" (§12)
#   arm 20 THIS TREE this repository's own specs, and its own review artifacts (none today)
#                                                                                exit 1, "N finding(s) — NO-REVIEW N:" with N = the active specs, counted from the tree
#   arm 21 RED   a second quality lane                                           exit 1, Fail DUPLICATE-LANE a.md `quality` — a copy is not a reviewer (§6.1)
#   arm 22 RED   a vendor:made-up lane on a spec whose front-matter names no vendor   exit 1, Fail EXTRA-LANE a.md `vendor:made-up`
#   arm 23 RED   lanes without executor                                          exit 1, Fail BAD-REVIEW a.md naming executor — every §6.1 field is required
#   arm 24 RED   a plan sha256 of not-a-hash                                     exit 1, Fail NO-PLAN a.md — a sha256 is 64 hex digits
#   arm 25 GREEN vendors: [nvidia cuda] with a vendor:nvidia cuda PASS lane      exit 0, Pass reviewed 1 — a vendor named with a space can be reviewed
#   arm 26 RED   a/b.md and a-b.md, both active, share one artifact path          exit 1, Fail SLUG-COLLISION for both, naming the other, and NOT NO-REVIEW
#   arm 27 RED   agreed: false with every lane PASS                              exit 1, Fail NOT-AGREED a.md — a review that says its quorum did not agree
#
# Arm 20 is the withheld-step measurement (goal-mode.md §11 step 7, doctrine
# 6): it runs the rule on THIS tree's specs, copied into the fixture, and
# asserts what the tree measures today — every active spec NO-REVIEW, because
# no review has been recorded yet. The direct CB-2111 step in CI cannot land
# until they are; the first review recorded changes this arm first, so the
# flip cannot happen unnoticed. N is computed from the tree, never written
# down here.
#
# Each arm asserts BOTH the process exit code and the rule's entry in the JSON
# report: the exit code is what CI acts on, the entry is what proves the
# verdict came from this rule. A missing entry is under-discovery and fails
# the control (exit 2). Every needle is the RENDERED finding
# (`CLASS docs/specifications/<file>:`), never a class word alone.
#
# Exit: 0 every arm behaved · 1 an arm did not (named on stderr) · 2 usage,
# missing binary, or a report the script could not read.
#
# The binary is an ARGUMENT, never resolved from PATH: the control must judge
# the pmat built from this tree, not whichever one is installed.
set -uo pipefail

PMAT="${1:-}"
if [ -z "$PMAT" ] || [ ! -f "$PMAT" ] || [ ! -x "$PMAT" ]; then
  echo "spec-review-control: usage: $0 <path-to-pmat-binary> (got '${PMAT}')" >&2
  exit 2
fi
for tool in jq git sha256sum cmp; do
  command -v "$tool" >/dev/null 2>&1 || { echo "spec-review-control: $tool is required" >&2; exit 2; }
done

here="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d "${TMPDIR:-/tmp}/spec-review-control.XXXXXX")"
trap 'rm -rf "${work:?}"' EXIT
repo="$work/repo"
report="$work/report.json"
specs="$repo/docs/specifications"
audits="$repo/docs/audits"

# Host git configuration is kept out of the fixture.
fgit() {
  GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null \
  GIT_AUTHOR_NAME=pmat1299 GIT_AUTHOR_EMAIL=pmat1299@example.invalid \
  GIT_COMMITTER_NAME=pmat1299 GIT_COMMITTER_EMAIL=pmat1299@example.invalid \
    git -C "$repo" -c commit.gpgsign=false "$@"
}
# fresh — a new fixture repository: no remote, no history.
fresh() {
  rm -rf "${repo:?}"
  mkdir -p "$specs" "$audits"
  fgit init -q -b master
}
# write_spec <rel-path> <status> [vendors]   goal-mode.md's own header shape
write_spec() {
  mkdir -p "$(dirname "$specs/$1")"
  printf -- '---\nepic: null\nstatus: %s\nvendors: %s\n---\n\n# A spec\n' "$2" "${3:-[]}" >"$specs/$1"
}
# write_raw_spec <rel-path> <text>   the file verbatim, for malformed fronts
write_raw_spec() {
  mkdir -p "$(dirname "$specs/$1")"
  printf '%s\n' "$2" >"$specs/$1"
}
# artifact <rel-path>   docs/audits/spec-<slug>-review.json, the slug the path
# under docs/specifications with .md dropped and / flattened to -
artifact() {
  local s=${1%.md}
  printf '%s/spec-%s-review.json' "$audits" "${s//\//-}"
}
FIVE=(quality:PASS architecture:PASS security:PASS crux:PASS adversarial:PASS)
PLAN='{"tool":"claude-plan","ref":"plan.md","sha256":"64879f7d6b960a01909762d911a32d4582c20010c5641ee90278b644a9e3b525"}'
# lanes <role:VERDICT>...   a vendor role keeps its own colon: vendor:cuda:PASS
lanes() {
  printf '%s\n' "$@" | jq -R 'split(":") as $parts | {role: ($parts | .[:-1] | join(":")), executor: "human", verdict: ($parts | last), summary: "read it"}' | jq -cs .
}
# review_json <rel-path> <lanes-json> [plan-json|null] [partial] [spec-named]
# — the review as the spec reads NOW (its sha256 taken at call time)
review_json() {
  local rel=$1 ls=$2 plan=${3:-$PLAN} partial=${4:-false} named=${5:-docs/specifications/$1} sha
  sha=$(sha256sum "$specs/$rel" | cut -d' ' -f1)
  jq -cn --arg spec "$named" --arg sha "$sha" --argjson lanes "$ls" --argjson plan "$plan" --argjson partial "$partial" \
    '{spec: $spec, spec_sha256: $sha, lanes: $lanes, agreed: true, partial: $partial} + (if $plan == null then {} else {plan: $plan} end)'
}
# write_review <rel-path> [role:VERDICT...]   default: the five, all PASS
write_review() {
  local rel=$1
  shift
  [ "$#" -gt 0 ] || set -- "${FIVE[@]}"
  review_json "$rel" "$(lanes "$@")" >"$(artifact "$rel")"
}

# One run of the gate against the fixture. Sets RC (process exit). Then
# `judge <PREFIX>` sets STATUS and MESSAGE from the entry whose name starts
# with the prefix; a report with no such entry is exit 2.
run_gate() {
  RC=0
  "$PMAT" comply check --checks "${CHECKS:-CB-2111}" --path "$repo" --format json >"$report" 2>"$work/stderr" || RC=$?
}
judge() {
  local entry
  entry=$(jq -c --arg p "$1" '[.checks[] | select(.name | startswith($p))] | first // empty' "$report" 2>/dev/null || true)
  if [ -z "$entry" ]; then
    echo "spec-review-control: the report carries no '$1' entry (exit $RC) — under-discovery is a finding, not a pass" >&2
    echo "--- stderr" >&2; head -20 "$work/stderr" >&2
    echo "--- report" >&2; head -c 2000 "$report" >&2; echo >&2
    exit 2
  fi
  STATUS=$(printf '%s' "$entry" | jq -r '.status')
  MESSAGE=$(printf '%s' "$entry" | jq -r '.message')
}
fail_arm() {
  echo "spec-review-control: ARM $1 FAILED — $2" >&2
  echo "  exit=$RC status=${STATUS:-unjudged}" >&2
  echo "  message=${MESSAGE:-}" >&2
  exit 1
}
# needs <arm> <needle>...  — every needle must appear in MESSAGE
needs() {
  local arm=$1 n
  shift
  for n in "$@"; do
    case "$MESSAGE" in *"$n"*) ;; *) fail_arm "$arm" "the message must name '$n'";; esac
  done
}
# lacks <arm> <needle>...  — no needle may appear in MESSAGE
lacks() {
  local arm=$1 n
  shift
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
  local arm=$1 rule=$2 status=$3
  shift 3
  judge "$rule"
  [ "$STATUS" = "$status" ] || fail_arm "$arm" "$rule must be $status"
  needs "$arm" "$@"
}
not_measured() {
  case "$MESSAGE" in "not_measured:"*) ;; *) fail_arm "$1" "the message must start with not_measured:";; esac
}
measured() {
  case "$MESSAGE" in "not_measured:"*) fail_arm "$1" "the message must NOT start with not_measured:";; esac
}
# record <review-file>   one `pmat spec review --record` run; sets RRC
record() {
  RRC=0
  "$PMAT" spec review --record "$1" --path "$repo" >"$work/record.out" 2>"$work/record.err" || RRC=$?
}

fresh
echo "spec-review-control: fixture $repo (no remote, offline); pmat=$("$PMAT" --version 2>/dev/null | head -1)"

# arm 1: GREEN — a current five-role review with a plan
write_spec a.md active
write_review a.md
run_gate
[ "$RC" -eq 0 ] || fail_arm 1 "an active spec with a current five-role review must exit 0"
expect 1 CB-2111 Pass "reviewed 1" "exempt: none" "1 spec(s)"
echo "spec-review-control: arm 1 GREEN — a.md with a current five-role review: Pass reviewed 1 (exit 0)"

# arm 2: RED — the §7 falsifier: append one space to the spec
printf ' ' >>"$specs/a.md"
run_gate
[ "$RC" -eq 1 ] || fail_arm 2 "appending one space to a reviewed spec must exit 1"
expect 2 CB-2111 Fail "STALE-REVIEW docs/specifications/a.md:"
measured 2
lacks 2 "NO-REVIEW"
echo "spec-review-control: arm 2 RED   — one space appended to a.md: STALE-REVIEW (exit 1)"

# arm 3: RED — the review deleted
write_spec a.md active
rm -f "$(artifact a.md)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 3 "an active spec with no review must exit 1"
expect 3 CB-2111 Fail "NO-REVIEW docs/specifications/a.md: no docs/audits/spec-a-review.json" "pmat spec review --record"
echo "spec-review-control: arm 3 RED   — no review file: NO-REVIEW, the recorder named (exit 1)"

# arm 4: RED — the review is not JSON
printf '{not json' >"$(artifact a.md)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 4 "a review that does not parse must exit 1"
expect 4 CB-2111 Fail "BAD-REVIEW docs/specifications/a.md:"
echo "spec-review-control: arm 4 RED   — unparseable review: BAD-REVIEW (exit 1)"

# arm 5: RED — `partial` misspelt: a typo must not default a partial review to complete
review_json a.md "$(lanes "${FIVE[@]}")" | jq -c 'del(.partial) + {partail: true}' >"$(artifact a.md)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 5 "a review without a partial key must exit 1"
expect 5 CB-2111 Fail "BAD-REVIEW docs/specifications/a.md:" "partial"
echo "spec-review-control: arm 5 RED   — partial misspelt: BAD-REVIEW naming partial, never a default (exit 1)"

# arm 6: RED — the review names another spec
review_json a.md "$(lanes "${FIVE[@]}")" "$PLAN" false docs/specifications/b.md >"$(artifact a.md)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 6 "a review of another spec must exit 1"
expect 6 CB-2111 Fail "SPEC-MISMATCH docs/specifications/a.md: the review names docs/specifications/b.md"
echo "spec-review-control: arm 6 RED   — the review names b.md: SPEC-MISMATCH (exit 1)"

# arm 7: RED — no plan, then a plan whose sha256 is empty
review_json a.md "$(lanes "${FIVE[@]}")" null >"$(artifact a.md)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 7 "a review with no plan must exit 1"
expect 7 CB-2111 Fail "NO-PLAN docs/specifications/a.md:"
review_json a.md "$(lanes "${FIVE[@]}")" '{"tool":"claude-plan","ref":"plan.md","sha256":""}' >"$(artifact a.md)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 7 "a review whose plan sha256 is empty must exit 1"
expect 7 CB-2111 Fail "NO-PLAN docs/specifications/a.md:"
echo "spec-review-control: arm 7 RED   — no plan, then an empty plan sha256: NO-PLAN both times (exit 1)"

# arm 8: RED — the crux lane missing
write_review a.md quality:PASS architecture:PASS security:PASS adversarial:PASS
run_gate
[ "$RC" -eq 1 ] || fail_arm 8 "a review with no crux lane must exit 1"
expect 8 CB-2111 Fail "MISSING-ROLE docs/specifications/a.md: no \`crux\` lane"
starts 8 "1 finding(s) — MISSING-ROLE 1:"
echo "spec-review-control: arm 8 RED   — no crux lane: MISSING-ROLE crux, only (exit 1)"

# arm 9: RED then GREEN — a front-matter vendor adds vendor:<name> to the required roles (§4.3)
write_spec a.md active '[cuda]'
write_review a.md
run_gate
[ "$RC" -eq 1 ] || fail_arm 9 "vendors: [cuda] with only the five lanes must exit 1"
expect 9 CB-2111 Fail "MISSING-ROLE docs/specifications/a.md: no \`vendor:cuda\` lane"
write_review a.md "${FIVE[@]}" vendor:cuda:PASS
run_gate
[ "$RC" -eq 0 ] || fail_arm 9 "vendors: [cuda] with a vendor:cuda PASS lane must exit 0"
expect 9 CB-2111 Pass "reviewed 1"
write_spec a.md active
write_review a.md
echo "spec-review-control: arm 9 RED+GREEN — vendors: [cuda] needs a vendor:cuda lane, and passes with one (exit 1, then 0)"

# arm 10: RED — a lane that is not PASS
write_review a.md quality:PASS architecture:PASS security:FAIL crux:PASS adversarial:PASS
run_gate
[ "$RC" -eq 1 ] || fail_arm 10 "a review with a FAIL lane must exit 1"
expect 10 CB-2111 Fail "LANE-NOT-PASS docs/specifications/a.md: the \`security\` lane says FAIL"
write_review a.md quality:pass architecture:PASS security:PASS crux:PASS adversarial:PASS
run_gate
[ "$RC" -eq 1 ] || fail_arm 10 "a lane whose verdict is lowercase pass must exit 1 — the verdict is the exact word PASS"
expect 10 CB-2111 Fail "LANE-NOT-PASS docs/specifications/a.md: the \`quality\` lane says pass"
echo "spec-review-control: arm 10 RED  — the security lane says FAIL: LANE-NOT-PASS (exit 1)"

# arm 11: RED — partial: true
review_json a.md "$(lanes "${FIVE[@]}")" "$PLAN" true >"$(artifact a.md)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 11 "a partial review must exit 1"
expect 11 CB-2111 Fail "PARTIAL docs/specifications/a.md:"
echo "spec-review-control: arm 11 RED  — partial: true: PARTIAL (exit 1)"

# arm 12: RED — an unrecognised role is an error, never an extra lane (§6.1)
write_review a.md "${FIVE[@]}" style:PASS
run_gate
[ "$RC" -eq 1 ] || fail_arm 12 "a review with a lane whose role is not in the closed set must exit 1"
expect 12 CB-2111 Fail "UNKNOWN-ROLE docs/specifications/a.md: \`style\`"
lacks 12 "MISSING-ROLE"
echo "spec-review-control: arm 12 RED  — a style lane: UNKNOWN-ROLE, never an extra lane (exit 1)"

# arm 13: GREEN — historical and superseded are exempt, and named
write_review a.md
write_spec h.md historical
write_spec s.md superseded
run_gate
[ "$RC" -eq 0 ] || fail_arm 13 "exempt specs with no review beside a reviewed active one must exit 0"
expect 13 CB-2111 Pass "reviewed 1" "docs/specifications/h.md (historical)" "docs/specifications/s.md (superseded)"
rm -f "$specs/h.md" "$specs/s.md"
echo "spec-review-control: arm 13 GREEN — h.md historical and s.md superseded exempt and named (exit 0)"

# arm 14: RED — a spec whose front-matter does not parse cannot say which roles it needs
write_raw_spec c.md '# c has no front-matter'
run_gate
[ "$RC" -eq 1 ] || fail_arm 14 "a spec with no front-matter must exit 1"
expect 14 CB-2111 Fail "UNJUDGEABLE docs/specifications/c.md:"
starts 14 "1 finding(s) — UNJUDGEABLE 1:"
echo "spec-review-control: arm 14 RED  — c.md with no front-matter: UNJUDGEABLE, never skipped (exit 1)"

# arm 15: RED — findings render in path order
rm -f "$(artifact a.md)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 15 "an unreviewed a.md beside an unjudgeable c.md must exit 1"
expect 15 CB-2111 Fail "; UNJUDGEABLE docs/specifications/c.md:"
starts 15 "2 finding(s) — NO-REVIEW 1, UNJUDGEABLE 1: NO-REVIEW docs/specifications/a.md:"
echo "spec-review-control: arm 15 RED  — NO-REVIEW a.md then UNJUDGEABLE c.md: path order (exit 1)"

# arm 16: RED — more than eight findings: the ninth and tenth are counted, not dropped
rm -rf "${specs:?}" "${audits:?}"
mkdir -p "$specs" "$audits"
for i in 0 1 2 3 4 5 6 7 8 9; do write_spec "s0$i.md" active; done
run_gate
[ "$RC" -eq 1 ] || fail_arm 16 "ten active specs with no review must exit 1"
expect 16 CB-2111 Fail "(+2 more)" "NO-REVIEW docs/specifications/s00.md:" "NO-REVIEW docs/specifications/s07.md:"
starts 16 "10 finding(s) — NO-REVIEW 10:"
lacks 16 "NO-REVIEW docs/specifications/s08.md:"
echo "spec-review-control: arm 16 RED  — ten NO-REVIEW specs: eight rendered, +2 more counted (exit 1)"

# arm 17: RECORD — `pmat spec review --record` validates, then stages; it never produces (§6.3)
rm -rf "${specs:?}" "${audits:?}"
mkdir -p "$specs" "$audits"
write_spec a.md active
review_json a.md "$(lanes "${FIVE[@]}")" >"$work/review.json"
printf ' ' >>"$specs/a.md"
record "$work/review.json"
[ "$RRC" -ne 0 ] || fail_arm 17 "recording a review of a spec edited since must exit non-zero"
grep -q "STALE-REVIEW docs/specifications/a.md:" "$work/record.err" || fail_arm 17 "the refusal must print the finding: $(head -3 "$work/record.err")"
[ ! -e "$(artifact a.md)" ] || fail_arm 17 "a refused review must write nothing"
[ -z "$(fgit diff --cached --name-only)" ] || fail_arm 17 "a refused review must stage nothing"
write_spec a.md active
record "$work/review.json"
[ "$RRC" -eq 0 ] || fail_arm 17 "recording a current review must exit 0: $(head -3 "$work/record.err")"
cmp -s "$work/review.json" "$(artifact a.md)" || fail_arm 17 "the review must be recorded byte for byte"
[ "$(fgit diff --cached --name-only)" = "docs/audits/spec-a-review.json" ] || fail_arm 17 "the recorded review, and nothing else, must be staged"
grep -q "recorded docs/audits/spec-a-review.json" "$work/record.out" || fail_arm 17 "stdout must name the artifact"
run_gate
[ "$RC" -eq 0 ] || fail_arm 17 "the gate must pass the review --record staged"
expect 17 CB-2111 Pass "reviewed 1"
review_json a.md "$(lanes "${FIVE[@]}")" "$PLAN" false README.md >"$work/readme-review.json"
record "$work/readme-review.json"
[ "$RRC" -ne 0 ] || fail_arm 17 "a review naming a file outside docs/specifications must be refused"
grep -q "not a spec under docs/specifications" "$work/record.err" || fail_arm 17 "the refusal must say why: $(head -3 "$work/record.err")"
for named in docs/specifications/../README.md docs/specifications/./a.md; do
  review_json a.md "$(lanes "${FIVE[@]}")" "$PLAN" false "$named" >"$work/dot-review.json"
  record "$work/dot-review.json"
  [ "$RRC" -ne 0 ] || fail_arm 17 "a review naming $named must be refused"
  grep -q "not a spec under docs/specifications" "$work/record.err" || fail_arm 17 "the refusal of $named must say why"
done
rm -f "$(artifact a.md)"
echo ORIGINAL >"$work/victim.txt"
ln -s "$work/victim.txt" "$(artifact a.md)"
record "$work/review.json"
[ "$RRC" -ne 0 ] || fail_arm 17 "recording through a symlinked artifact path must be refused"
grep -q "is a symlink" "$work/record.err" || fail_arm 17 "the refusal must name the symlink: $(head -3 "$work/record.err")"
[ "$(cat "$work/victim.txt")" = ORIGINAL ] || fail_arm 17 "a refused record must not write through the symlink"
rm -f "$(artifact a.md)"
write_spec b.md active
review_json b.md "$(lanes "${FIVE[@]}")" >"$work/review-b.json"
printf 'docs/audits/spec-b-review.json\n' >"$repo/.gitignore"
record "$work/review-b.json"
[ "$RRC" -ne 0 ] || fail_arm 17 "recording where git ignores the artifact must be refused"
grep -q "cannot be staged" "$work/record.err" || fail_arm 17 "the refusal must say it cannot be staged: $(head -3 "$work/record.err")"
[ ! -e "$(artifact b.md)" ] || fail_arm 17 "a review git ignores must not be written"
rm -f "$repo/.gitignore" "$specs/b.md"
mkdir -p "$work/nogit/docs/specifications"
cp "$specs/a.md" "$work/nogit/docs/specifications/a.md"
RRC=0
GIT_CEILING_DIRECTORIES="$work" "$PMAT" spec review --record "$work/review.json" --path "$work/nogit" >"$work/record.out" 2>"$work/record.err" || RRC=$?
[ "$RRC" -ne 0 ] || fail_arm 17 "recording outside a git work tree must be refused"
grep -q "cannot be staged" "$work/record.err" || fail_arm 17 "the refusal outside a work tree must say it cannot be staged: $(head -3 "$work/record.err")"
[ ! -e "$work/nogit/docs/audits" ] || fail_arm 17 "nothing may be written outside a git work tree"
write_spec v.md active '[cuda]'
review_json v.md "$(lanes "${FIVE[@]}")" >"$work/review-v.json"
record "$work/review-v.json"
[ "$RRC" -ne 0 ] || fail_arm 17 "recording a review missing the front-matter vendor's lane must be refused"
grep -q "MISSING-ROLE docs/specifications/v.md: no \`vendor:cuda\` lane" "$work/record.err" || fail_arm 17 "the refusal must name the missing vendor lane: $(head -3 "$work/record.err")"
[ ! -e "$(artifact v.md)" ] || fail_arm 17 "a refused review must not be written"
rm -f "$specs/v.md"
echo ORIGINAL >"$work/victim2.txt"
ln "$work/victim2.txt" "$(artifact a.md)"
record "$work/review.json"
[ "$RRC" -eq 0 ] || fail_arm 17 "a hard link at the artifact path must be replaced, not refused: $(head -3 "$work/record.err")"
[ "$(cat "$work/victim2.txt")" = ORIGINAL ] || fail_arm 17 "record must never write through a hard link to a file outside the project"
cmp -s "$work/review.json" "$(artifact a.md)" || fail_arm 17 "the review must replace the hard link byte for byte"
review_json a.md "$(lanes "${FIVE[@]}")" "$PLAN" false "$(printf 'docs/specifications/a\rb.md')" >"$work/cr-review.json"
record "$work/cr-review.json"
[ "$RRC" -ne 0 ] || fail_arm 17 "a spec path holding a carriage return must be refused"
grep -q "not a spec under docs/specifications" "$work/record.err" || fail_arm 17 "the refusal of a carriage return must say why"
rm -f "$(artifact a.md)"
: >"$repo/.git/index.lock"
record "$work/review.json"
rm -f "$repo/.git/index.lock"
[ "$RRC" -ne 0 ] || fail_arm 17 "a failed git add must be reported, not swallowed"
grep -q "written but NOT staged" "$work/record.err" || fail_arm 17 "the refusal must say the file is written and not staged: $(head -3 "$work/record.err")"
echo "spec-review-control: arm 17 RECORD — stale refused (nothing written or staged); current recorded byte-identical and staged, and the gate passes it; README.md, .. and . refused; a symlink refused, nothing written through it; an ignored path and a non-git directory refused before any write"

# arm 18: SKIP — no docs/specifications in a repository whose history never held one
fresh
rm -rf "${specs:?}"
run_gate
[ "$RC" -eq 0 ] || fail_arm 18 "a project with no docs/specifications must exit 0"
expect 18 CB-2111 Skip "no docs/specifications"
echo "spec-review-control: arm 18 SKIP — no docs/specifications ever: Skip (exit 0)"

# arm 19: not measured — the directory was committed and then deleted (§12)
mkdir -p "$specs"
write_spec a.md active
write_review a.md
fgit add -A && fgit commit -q -m "specs"
fgit rm -r -q docs/specifications
[ ! -e "$specs" ] || fail_arm 19 "git rm must remove the directory"
run_gate
[ "$RC" -eq 1 ] || fail_arm 19 "a docs/specifications that was committed and deleted must exit 1 — deleting a gate's input is not a way of passing it"
expect 19 CB-2111 Fail "committed and is now gone"
not_measured 19
echo "spec-review-control: arm 19 N/M  — committed-then-deleted docs/specifications reported not_measured (exit 1)"

# arm 20: THIS TREE — the withheld-step measurement. This repository's own
# specs, copied into the fixture, beside its own review artifacts: none today.
fresh
rm -rf "${specs:?}"
cp -R "$here/docs/specifications" "$specs"
reviews=$(find "$here/docs/audits" -maxdepth 1 -name 'spec-*-review.json' 2>/dev/null | wc -l | tr -d ' ')
[ "$reviews" -eq 0 ] || fail_arm 20 "this tree now holds $reviews spec review artifact(s): re-measure, flip the direct CB-2111 step once every active spec's review is current (PMAT-FLIP), and retire this arm"
total=0
active=0
while IFS= read -r -d '' f; do
  total=$((total + 1))
  if [ "$(head -1 "$f")" = "---" ] && sed -n '2,/^---$/p' "$f" | grep -qE '^status:[[:space:]]*active([[:space:]]|$)'; then
    active=$((active + 1))
  fi
done < <(find "$here/docs/specifications" -name '*.md' -print0)
[ "$total" -gt 0 ] || { echo "spec-review-control: arm 20 found no *.md under $here/docs/specifications — the control is running in the wrong tree" >&2; exit 2; }
run_gate
[ "$RC" -eq 1 ] || fail_arm 20 "this tree's $active active specs must exit 1 today — none has a review yet (goal-mode.md §11 step 7 is withheld)"
expect 20 CB-2111 Fail "pmat spec review --record"
starts 20 "$active finding(s) — NO-REVIEW $active:"
echo "spec-review-control: arm 20 THIS TREE — $total specs, $active active, every one NO-REVIEW: the direct step stays withheld until they are reviewed (exit 1)"

# arm 21: RED — a second lane of one role is a copy, not a reviewer (§6.1)
fresh
write_spec a.md active
write_review a.md "${FIVE[@]}" quality:PASS
run_gate
[ "$RC" -eq 1 ] || fail_arm 21 "a review with two quality lanes must exit 1"
expect 21 CB-2111 Fail "DUPLICATE-LANE docs/specifications/a.md: \`quality\`"
echo "spec-review-control: arm 21 RED  — a second quality lane: DUPLICATE-LANE (exit 1)"

# arm 22: RED — a closed-set role this spec does not require is an extra lane
write_review a.md "${FIVE[@]}" vendor:made-up:PASS
run_gate
[ "$RC" -eq 1 ] || fail_arm 22 "a vendor lane the front-matter does not name must exit 1"
expect 22 CB-2111 Fail "EXTRA-LANE docs/specifications/a.md: \`vendor:made-up\`"
echo "spec-review-control: arm 22 RED  — vendor:made-up with no vendor in the front-matter: EXTRA-LANE (exit 1)"

# arm 23: RED — every §6.1 field is required: lanes without executor do not parse
review_json a.md "$(lanes "${FIVE[@]}" | jq -c 'map(del(.executor))')" >"$(artifact a.md)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 23 "lanes without executor must exit 1"
expect 23 CB-2111 Fail "BAD-REVIEW docs/specifications/a.md:" "executor"
for field in agreed lanes; do
  review_json a.md "$(lanes "${FIVE[@]}")" | jq -c "del(.$field)" >"$(artifact a.md)"
  run_gate
  [ "$RC" -eq 1 ] || fail_arm 23 "a review without $field must exit 1"
  expect 23 CB-2111 Fail "BAD-REVIEW docs/specifications/a.md:" "$field"
done
echo "spec-review-control: arm 23 RED  — lanes without executor: BAD-REVIEW naming the field (exit 1)"

# arm 24: RED — a plan sha256 that is not 64 hex digits
review_json a.md "$(lanes "${FIVE[@]}")" '{"tool":"claude-plan","ref":"plan.md","sha256":"not-a-hash"}' >"$(artifact a.md)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 24 "a plan sha256 that is not 64 hex digits must exit 1"
expect 24 CB-2111 Fail "NO-PLAN docs/specifications/a.md:"
echo "spec-review-control: arm 24 RED  — plan sha256 not-a-hash: NO-PLAN (exit 1)"

# arm 25: GREEN — a vendor named with a space can be reviewed
write_spec a.md active '[nvidia cuda]'
write_review a.md "${FIVE[@]}" "vendor:nvidia cuda:PASS"
run_gate
[ "$RC" -eq 0 ] || fail_arm 25 "vendors: [nvidia cuda] with a vendor:nvidia cuda PASS lane must exit 0"
expect 25 CB-2111 Pass "reviewed 1"
echo "spec-review-control: arm 25 GREEN — vendors: [nvidia cuda] reviewed by a vendor:nvidia cuda lane: Pass (exit 0)"

# arm 26: RED — two active specs sharing an artifact path are named as a collision
rm -rf "${specs:?}" "${audits:?}"
mkdir -p "$specs" "$audits"
write_spec a/b.md active
write_spec a-b.md active
run_gate
[ "$RC" -eq 1 ] || fail_arm 26 "two active specs sharing an artifact path must exit 1"
expect 26 CB-2111 Fail "SLUG-COLLISION docs/specifications/a-b.md:" "SLUG-COLLISION docs/specifications/a/b.md:" "shares docs/audits/spec-a-b-review.json with docs/specifications/a/b.md"
lacks 26 "NO-REVIEW"
echo "spec-review-control: arm 26 RED  — a/b.md and a-b.md share one artifact path: SLUG-COLLISION for both (exit 1)"

# arm 27: RED — a review that records agreed: false says its quorum did not agree
rm -rf "${specs:?}" "${audits:?}"
mkdir -p "$specs" "$audits"
write_spec a.md active
review_json a.md "$(lanes "${FIVE[@]}")" | jq -c '.agreed = false' >"$(artifact a.md)"
run_gate
[ "$RC" -eq 1 ] || fail_arm 27 "a review that records agreed: false must exit 1"
expect 27 CB-2111 Fail "NOT-AGREED docs/specifications/a.md:"
echo "spec-review-control: arm 27 RED  — agreed: false with every lane PASS: NOT-AGREED (exit 1)"

echo "spec-review-control: all 27 arms behaved — CB-2111 can fail, can pass, says why, names its exemptions, refuses a deleted input, --record validates before it stages, and this tree measures NO-REVIEW on every active spec"
