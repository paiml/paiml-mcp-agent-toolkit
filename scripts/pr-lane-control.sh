#!/usr/bin/env bash
# pr-lane-control.sh — PMAT-1313. The PR lane and the pre-release lane are not the
# same lane, and this proves the difference is what it claims to be.
#
#   arm 1 GREEN  ci.yml does NOT enable nextest, and says in the file why not
#   arm 2 RED    a workflow that enables nextest without lifting the thread cap is refused
#   arm 3 GREEN  .config/nextest.toml carries no default-filter (the fast lane runs everything)
#   arm 4 RED    a config that reintroduces a default-filter is refused
#   arm 5 GREEN  the coverage ratchet file is tracked, parses, and is where ci.yml says
#   arm 6 RED    the gate's own arithmetic fails a floor above a fixture's coverage
#   arm 7 GREEN  the gate's own arithmetic passes a floor at the fixture's coverage
#   arm 8 RED    the gate refuses an lcov with no line records
#
# Arms 2 and 4 are the ones that matter: without them this script asserts that
# a file contains the string somebody just wrote into it, which is theater. Each
# builds a fixture carrying the defect and requires the predicate to REFUSE it.
#
# PMAT-1314 retired the former arms 5, 6 and 7 (which asserted that the fast
# lane's now-deleted default-filter named exactly two tests that exist, and
# that nothing double-excluded them from the pre-release lane). Those two
# tests hung from a real drain-backstop bug, not an order dependency; once
# fixed, there is nothing left to exclude and nothing left for those arms to
# check. See `.config/nextest.toml` and
# `src/mcp_pmcp/simple_unified_server.rs`.
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
nextest_has_no_default_filter() {  # $1 = nextest config
  ! grep -qE '^default-filter' "$1"
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
nextest_has_no_default_filter "$NT" \
  || fail_arm 3 "the local fast lane must not carry a default-filter — PMAT-1314 fixed the two tests' real defect (an uncancellable 300s drain backstop) instead of hiding them"
echo "pr-lane-control: arm 3 GREEN — no default-filter, the fast lane runs everything"

# ── arm 4 (falsifier) ─────────────────────────────────────────────────────────
printf '[profile.default]\ndefault-filter = "not test(=some::excluded::test)"\n' > "$T/with-filter.toml"
if nextest_has_no_default_filter "$T/with-filter.toml"; then
  fail_arm 4 "a config that reintroduces a default-filter must be refused"
fi
echo "pr-lane-control: arm 4 RED   — a config reintroducing a default-filter is refused"

# ── arms 5–8: the coverage ratchet ──────────────────────────────────────────
# ci.yml sets coverage_min and coverage_baseline_file; sovereign-ci.yml's
# `Enforce coverage floor` step reads lcov DA records and fails below
# max(min, baseline). None of that is visible from this repository, so these arms
# reproduce the step's arithmetic on a fixture and require it to answer both ways.
BL=$(grep -oE "coverage_baseline_file: '[^']+'" "$WF" | sed "s/.*: '//; s/'$//")
[ -n "$BL" ] || fail_arm 5 "ci.yml must name coverage_baseline_file"
git ls-files --error-unmatch "$BL" >/dev/null 2>&1 \
  || fail_arm 5 "$BL must be TRACKED — .pmat/ is gitignored, so a baseline there never reaches CI and the ratchet is coverage_min alone"
BASE=$(tr -dc '0-9.' < "$BL" | head -c 16)
[ -n "$BASE" ] || fail_arm 5 "$BL must hold a number"
echo "pr-lane-control: arm 5 GREEN — the ratchet file $BL is tracked and reads $BASE"

# the gate's arithmetic, as sovereign-ci.yml writes it: covered = DA records with hits>0
gate_pct() { awk -F'[:,]' '/^DA:/ { t++; if ($3 > 0) c++ } END { if (t==0) print "EMPTY"; else printf "%.2f", (c/t)*100 }' "$1"; }
gate_verdict() {  # $1 lcov, $2 floor → 0 pass, 1 fail (mirrors: exit 1 iff pct < floor, or empty)
  local p; p=$(gate_pct "$1"); [ "$p" = EMPTY ] && return 1
  awk -v p="$p" -v f="$2" 'BEGIN { exit (p < f) ? 1 : 0 }'
}
printf 'SF:a.rs\nDA:1,1\nDA:2,1\nDA:3,1\nDA:4,0\nend_of_record\n' > "$T/fixture.lcov"   # 75.00%
if gate_verdict "$T/fixture.lcov" 80; then fail_arm 6 "a 75% fixture must FAIL an 80 floor"; fi
echo "pr-lane-control: arm 6 RED   — 75% under an 80 floor is refused"
gate_verdict "$T/fixture.lcov" 75 || fail_arm 7 "a 75% fixture must PASS a 75 floor"
echo "pr-lane-control: arm 7 GREEN — 75% at a 75 floor passes"
: > "$T/empty.lcov"
if gate_verdict "$T/empty.lcov" 0; then fail_arm 8 "an lcov with no DA records must be refused even at floor 0"; fi
echo "pr-lane-control: arm 8 RED   — empty coverage data is refused"

[ "$FAIL" -eq 0 ] || { echo "pr-lane-control: FAILED"; exit 1; }
echo "pr-lane-control: all 8 arms behaved — nextest is off with its measurement recorded, the local fast lane carries no default-filter, and cargo test runs everything"
