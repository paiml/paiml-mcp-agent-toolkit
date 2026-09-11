#!/usr/bin/env bash
# pr-lane-control.sh — PMAT-1313. The PR lane and the pre-release lane are not the
# same lane, and this proves the difference is what it claims to be.
#
#   arm 1 GREEN  ci.yml does NOT enable nextest, and says in the file why not
#   arm 2 RED    a workflow that enables nextest without lifting the thread cap is refused
#   arm 3 GREEN  the fast lane's exclusion names exactly two tests
#   arm 4 GREEN  both named tests still exist in the tree
#   arm 5 RED    an exclusion naming a test that does not exist is refused
#   arm 6 GREEN  nothing excludes those two from the pre-release lane
#   arm 7 RED    a Makefile that skips one of them from `cargo test` is refused
#   arm 8 GREEN  the coverage ratchet file is tracked, parses, and is where ci.yml says
#   arm 9 RED    the gate's own arithmetic fails a floor above a fixture's coverage
#   arm 10 GREEN the gate's own arithmetic passes a floor at the fixture's coverage
#   arm 11 RED   the gate refuses an lcov with no line records
#
# Arms 2, 5 and 7 are the ones that matter: without them this script asserts that
# a file contains the string somebody just wrote into it, which is theater. Each
# builds a fixture carrying the defect and requires the predicate to REFUSE it.
#
# Usage: bash scripts/pr-lane-control.sh
set -uo pipefail
cd "$(dirname "$0")/.." || exit 9
FAIL=0
fail_arm() { echo "pr-lane-control: ARM $1 FAILED — $2"; FAIL=1; }

WF=.github/workflows/ci.yml
NT=.config/nextest.toml

# The predicates, as functions, so a fixture can be judged by the SAME code.
nextest_is_off_and_explained() {  # $1 = workflow file
  # OFF is not enough: the next person to try this must find the measurement that
  # turned it off, in the file, or they will turn it back on and slow CI down again.
  grep -qE '^ *use_nextest: false *$' "$1" \
    && grep -qF 'NEXTEST_TEST_THREADS=4' "$1" \
    && grep -qF 'PMAT-1315' "$1"
}
excluded_tests() {        # $1 = nextest config; prints one test path per line
  grep -oE 'test\(=[^)]+\)' "$1" | sed 's/^test(=//; s/)$//'
}
test_exists() {           # $1 = full test path; true iff its fn is in the tree
  local fn=${1##*::}
  grep -rqE "fn ${fn}\(" src --include='*.rs'
}
not_skipped_in_slow_lane() {  # $1 = file that may carry cargo-test skips
  local t
  while read -r t; do
    [ -n "$t" ] || continue
    grep -qE -- "--skip[= ]+${t##*::}" "$1" && return 1
  done < <(excluded_tests "$NT")
  return 0
}

# ── arm 1 ─────────────────────────────────────────────────────────────────────
nextest_is_off_and_explained "$WF" \
  || fail_arm 1 "ci.yml must set use_nextest: false AND carry the measurement that says why — the 4-thread cap and PMAT-1315"
echo "pr-lane-control: arm 1 GREEN — nextest is off, and the file says why"

# ── arm 2 (falsifier) ─────────────────────────────────────────────────────────
T=$(mktemp -d); trap 'rm -rf "${T:?}"' EXIT
sed 's/^ *use_nextest: false *$/      use_nextest: true/' "$WF" > "$T/on.yml"
if nextest_is_off_and_explained "$T/on.yml"; then
  fail_arm 2 "a workflow that enables nextest must be refused while sovereign-ci.yml caps it at 4 threads — measured SLOWER than cargo test on this suite"
fi
echo "pr-lane-control: arm 2 RED   — enabling nextest under the thread cap is refused"

# ── arm 3 ─────────────────────────────────────────────────────────────────────
N=$(excluded_tests "$NT" | grep -c . )
[ "$N" -eq 2 ] || fail_arm 3 "the fast lane's exclusion must name exactly two tests, found $N"
echo "pr-lane-control: arm 3 GREEN — the exclusion names exactly 2 tests"

# ── arm 4 ─────────────────────────────────────────────────────────────────────
while read -r t; do
  [ -n "$t" ] || continue
  test_exists "$t" || fail_arm 4 "excluded test does not exist in the tree: $t (a filter naming a renamed test excludes nothing, silently)"
done < <(excluded_tests "$NT")
echo "pr-lane-control: arm 4 GREEN — every excluded test exists"

# ── arm 5 (falsifier) ─────────────────────────────────────────────────────────
if test_exists "a::b::this_test_was_renamed_or_deleted_and_the_filter_was_not"; then
  fail_arm 5 "the existence predicate must refuse a test that is not in the tree"
fi
echo "pr-lane-control: arm 5 RED   — an exclusion naming a missing test is refused"

# ── arm 6 ─────────────────────────────────────────────────────────────────────
for f in Makefile "$WF"; do
  not_skipped_in_slow_lane "$f" \
    || fail_arm 6 "$f skips a test the fast lane already excludes — then NOTHING runs it"
done
echo "pr-lane-control: arm 6 GREEN — the pre-release lane still runs both"

# ── arm 7 (falsifier) ─────────────────────────────────────────────────────────
first=$(excluded_tests "$NT" | head -1); first=${first##*::}
printf 'test-x:\n\tcargo test --lib -- --skip %s\n' "$first" > "$T/Makefile"
if not_skipped_in_slow_lane "$T/Makefile"; then
  fail_arm 7 "a Makefile that also skips an excluded test must be refused"
fi
echo "pr-lane-control: arm 7 RED   — double-exclusion is refused"

# ── arms 8–11: the coverage ratchet ──────────────────────────────────────────
# ci.yml sets coverage_min and coverage_baseline_file; sovereign-ci.yml's
# `Enforce coverage floor` step reads lcov DA records and fails below
# max(min, baseline). None of that is visible from this repository, so these arms
# reproduce the step's arithmetic on a fixture and require it to answer both ways.
BL=$(grep -oE "coverage_baseline_file: '[^']+'" "$WF" | sed "s/.*: '//; s/'$//")
[ -n "$BL" ] || fail_arm 8 "ci.yml must name coverage_baseline_file"
git ls-files --error-unmatch "$BL" >/dev/null 2>&1 \
  || fail_arm 8 "$BL must be TRACKED — .pmat/ is gitignored, so a baseline there never reaches CI and the ratchet is coverage_min alone"
BASE=$(tr -dc '0-9.' < "$BL" | head -c 16)
[ -n "$BASE" ] || fail_arm 8 "$BL must hold a number"
echo "pr-lane-control: arm 8 GREEN — the ratchet file $BL is tracked and reads $BASE"

# the gate's arithmetic, as sovereign-ci.yml writes it: covered = DA records with hits>0
gate_pct() { awk -F'[:,]' '/^DA:/ { t++; if ($3 > 0) c++ } END { if (t==0) print "EMPTY"; else printf "%.2f", (c/t)*100 }' "$1"; }
gate_verdict() {  # $1 lcov, $2 floor → 0 pass, 1 fail (mirrors: exit 1 iff pct < floor, or empty)
  local p; p=$(gate_pct "$1"); [ "$p" = EMPTY ] && return 1
  awk -v p="$p" -v f="$2" 'BEGIN { exit (p < f) ? 1 : 0 }'
}
printf 'SF:a.rs\nDA:1,1\nDA:2,1\nDA:3,1\nDA:4,0\nend_of_record\n' > "$T/fixture.lcov"   # 75.00%
if gate_verdict "$T/fixture.lcov" 80; then fail_arm 9 "a 75% fixture must FAIL an 80 floor"; fi
echo "pr-lane-control: arm 9 RED   — 75% under an 80 floor is refused"
gate_verdict "$T/fixture.lcov" 75 || fail_arm 10 "a 75% fixture must PASS a 75 floor"
echo "pr-lane-control: arm 10 GREEN — 75% at a 75 floor passes"
: > "$T/empty.lcov"
if gate_verdict "$T/empty.lcov" 0; then fail_arm 11 "an lcov with no DA records must be refused even at floor 0"; fi
echo "pr-lane-control: arm 11 RED   — empty coverage data is refused"

[ "$FAIL" -eq 0 ] || { echo "pr-lane-control: FAILED"; exit 1; }
echo "pr-lane-control: all 11 arms behaved — nextest is off with its measurement recorded, the local fast lane excludes exactly two named tests that exist, and cargo test still runs them"
