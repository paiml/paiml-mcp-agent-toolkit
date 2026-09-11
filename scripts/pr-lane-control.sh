#!/usr/bin/env bash
# pr-lane-control.sh — PMAT-1313. The PR lane and the pre-release lane are not the
# same lane, and this proves the difference is what it claims to be.
#
#   arm 1 GREEN  ci.yml takes the fast lane on a pull request AND ONLY THERE
#   arm 2 RED    a workflow that turns nextest on unconditionally is refused
#   arm 3 GREEN  the fast lane's exclusion names exactly two tests
#   arm 4 GREEN  both named tests still exist in the tree
#   arm 5 RED    an exclusion naming a test that does not exist is refused
#   arm 6 GREEN  nothing excludes those two from the pre-release lane
#   arm 7 RED    a Makefile that skips one of them from `cargo test` is refused
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
fast_lane_is_pr_only() {  # $1 = workflow file
  grep -qF "use_nextest: \${{ github.event_name == 'pull_request' }}" "$1"
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
fast_lane_is_pr_only "$WF" \
  || fail_arm 1 "ci.yml must pass use_nextest gated on github.event_name == 'pull_request'"
echo "pr-lane-control: arm 1 GREEN — the fast lane is the pull-request lane"

# ── arm 2 (falsifier) ─────────────────────────────────────────────────────────
T=$(mktemp -d); trap 'rm -rf "${T:?}"' EXIT
sed "s/use_nextest: \${{ github.event_name == 'pull_request' }}/use_nextest: true/" "$WF" > "$T/always.yml"
if fast_lane_is_pr_only "$T/always.yml"; then
  fail_arm 2 "a workflow that turns nextest on for every event must be refused — master would stop running the tests nextest cannot"
fi
echo "pr-lane-control: arm 2 RED   — unconditional nextest is refused"

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

[ "$FAIL" -eq 0 ] || { echo "pr-lane-control: FAILED"; exit 1; }
echo "pr-lane-control: all 7 arms behaved — the fast lane is the PR lane, it excludes exactly two named tests that exist, and the pre-release lane still runs them"
