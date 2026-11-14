#!/usr/bin/env bash
# pr-lane-control.sh — PMAT-1313. The PR lane and the pre-release lane are not the
# same lane, and this proves the difference is what it claims to be.
#
#   arm 1 GREEN  ci.yml does NOT enable nextest, and says in the file why not
#   arm 2 RED    a workflow that enables nextest without lifting the thread cap is refused
#   arm 3 GREEN  the fast lane has no default-filter, so it excludes no test (PMAT-1314)
#   arm 4 GREEN  the tests PMAT-1314 re-admitted still exist, and no `--skip` drops them
#   arm 5 RED    an exclusion naming a test that does not exist is refused
#   arm 6 RED    a planted default-filter of ANY form (=exact, /regex/, bare) is seen by arm 3
#   arm 7 RED    a Makefile or workflow that `--skip`s a re-admitted test is refused
#   arm 8 GREEN  the coverage ratchet file is tracked, parses, and is where ci.yml says
#   arm 9 RED    the gate's own arithmetic fails a floor above a fixture's coverage
#   arm 10 GREEN the gate's own arithmetic passes a floor at the fixture's coverage
#   arm 11 RED   the gate refuses an lcov with no line records
#
# Arms 2, 5, 6, 7, 9 and 11 are the ones that matter: without them this script asserts that
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
fast_lane_filters() {     # $1 = nextest config; prints every live default-filter line
  # ANY default-filter excludes tests from every profile that inherits it, whatever
  # filterset form it uses: test(=exact), test(/regex/), bare test(substr), package().
  # Matching only one form is how `not test(/eof_drain/)` read as "excludes nothing".
  grep -E '^[[:space:]]*default-filter[[:space:]]*=' "$1"
}
test_exists() {           # $1 = full test path; true iff its fn is in the tree
  local fn=${1##*::}
  grep -rqE "fn ${fn}\(" src --include='*.rs'
}
# The tests PMAT-1314 brought back into the fast lane. Once no filter excludes them,
# the only way left to stop running them is a `--skip` in the slow lane.
READMITTED=(
  mcp_pmcp::simple_unified_server::eof_drain_tests::eof_does_not_signal_while_a_request_is_in_flight
  mcp_pmcp::simple_unified_server::eof_drain_tests::waits_for_every_outstanding_request
)
skipped_readmitted() {    # $1 = file that may carry cargo-test skips; prints each skipped test
  # `cargo test -- --skip P` drops every test whose path CONTAINS P, so
  # `--skip eof_drain` or `--skip simple_unified_server` drops both, not just the
  # exact fn name the old arm looked for.
  local p t
  while read -r p; do
    [ -n "$p" ] || continue
    for t in "${READMITTED[@]}"; do
      case "$t" in *"$p"*) echo "$t (--skip $p)" ;; esac
    done
  done < <(grep -oE -- '--skip[= ]+[^[:space:]\\"'"'"'`]+' "$1" | sed -E 's/^--skip[= ]+//')
}
slow_lane_files() {       # every file that can carry a cargo-test --skip, minus this one
  printf '%s\n' Makefile scripts/gate.sh
  find .github/workflows -name '*.yml' -type f | sort
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
# PMAT-1314 removed the two-test exclusion: the "hang" was a 300 s uncancellable
# DRAIN_BACKSTOP sleep that the tests now run on a paused clock. The fast lane
# excludes nothing, so any default-filter, in any filterset form, is a regression.
N=$(fast_lane_filters "$NT" | grep -c . )
[ "$N" -eq 0 ] || fail_arm 3 "the fast lane must exclude no test (PMAT-1314), found $N default-filter line(s): $(fast_lane_filters "$NT")"
echo "pr-lane-control: arm 3 GREEN — the fast lane has no default-filter, so it excludes no test"

# ── arm 4 ─────────────────────────────────────────────────────────────────────
# Arm 7's skip check is keyed on these names; a rename would make it vacuous.
for t in "${READMITTED[@]}"; do
  test_exists "$t" || fail_arm 4 "re-admitted test is not in the tree: $t (update READMITTED, or arm 7 guards nothing)"
done
while read -r f; do
  s=$(skipped_readmitted "$f")
  [ -z "$s" ] || fail_arm 4 "$f skips a test PMAT-1314 re-admitted — then the slow lane no longer runs it: $s"
done < <(slow_lane_files)
echo "pr-lane-control: arm 4 GREEN — both re-admitted tests exist and no slow-lane file skips them"

# ── arm 5 ─────────────────────────────────────────────────────────────────────
if test_exists "a::b::this_test_was_renamed_or_deleted_and_the_filter_was_not"; then
  fail_arm 5 "the existence predicate must refuse a test that is not in the tree"
fi
echo "pr-lane-control: arm 5 RED   — an exclusion naming a missing test is refused"

# ── arm 6 (falsifier) ─────────────────────────────────────────────────────────
# A filter that brings an exclusion back must be counted by the predicate arm 3 uses,
# in every filterset form; a commented-out one must not be.
for form in "not test(=a::b::c)" "not test(/eof_drain/)" "not test(eof_drain)" \
            "not (test(=a::b) + test(/x/))" "not package(pmat)"; do
  printf '[profile.default]\n  default-filter = %s\n' "'$form'" > "$T/nextest.toml"
  [ "$(fast_lane_filters "$T/nextest.toml" | grep -c .)" -eq 1 ] \
    || fail_arm 6 "fast_lane_filters must see a planted default-filter '$form', or arm 3 passes it"
done
printf "# default-filter = 'not test(=a::b::c)'\n" > "$T/nextest.toml"
[ "$(fast_lane_filters "$T/nextest.toml" | grep -c .)" -eq 0 ] \
  || fail_arm 6 "a commented-out default-filter excludes nothing and must not be counted"
echo "pr-lane-control: arm 6 RED   — a planted default-filter of every form is seen by arm 3's predicate"

# ── arm 7 (falsifier) ─────────────────────────────────────────────────────────
# Each planted skip drops a re-admitted test from `cargo test`; each must be refused.
for p in eof_does_not_signal_while_a_request_is_in_flight eof_drain \
         simple_unified_server::eof_drain_tests::waits_for_every_outstanding_request; do
  for form in "--skip $p" "--skip=$p"; do
    printf 'test-x:\n\tcargo test --lib -- --skip libsql %s \\\n\t\t--skip cli_integration_tests\n' "$form" > "$T/Makefile"
    [ -n "$(skipped_readmitted "$T/Makefile")" ] \
      || fail_arm 7 "a Makefile with '$form' skips a re-admitted test and must be refused"
  done
done
printf 'test-x:\n\tcargo test --lib -- --skip libsql --skip cli_integration_tests\n' > "$T/Makefile"
[ -z "$(skipped_readmitted "$T/Makefile")" ] \
  || fail_arm 7 "skips that name no re-admitted test must not be refused"
echo "pr-lane-control: arm 7 RED   — skipping a re-admitted test from the slow lane is refused"

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
echo "pr-lane-control: all 11 arms behaved — nextest is off with its measurement recorded, the local fast lane has no default-filter, and no slow-lane file skips the tests PMAT-1314 re-admitted"
