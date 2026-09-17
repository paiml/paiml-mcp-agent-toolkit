#!/usr/bin/env bash
# PMAT-1305 (#1305; duplicate report #1284) — reproduce the `ci / test` flake of
# a_crate_that_does_not_compile_is_reported_as_not_measured OUTSIDE CI.
#
# The CI shape it re-creates, read off the job logs of runs 35209474539 and
# 34562657315: `ci / test` and `ci / coverage` run at the same time and mount the
# same per-PR directory as CARGO_TARGET_DIR=/workspace/target. The coverage job's
# dead-code tests inherit that variable, so their nested `cargo check` blocks on
# the lock the test job holds while it compiles its test binary. The lock is
# released, both processes run the dead-code tests within a second of each
# other, and the coverage job's COMPILABLE `fx` fixture is checked after the test
# job wrote its UNCOMPILABLE `fx` source and before that source was checked —
# the same package name, so the same fingerprint, and a newer one. cargo calls
# the broken crate fresh, replays the empty diagnostics and exits 0.
#
# usage: repro-pmat-1305-shared-target-race.sh <lib-test-binary> <iterations> [aligned|free|single]
#   aligned  (default) two processes share a target dir and start while its
#            build lock is held, as in CI; the lock is released under both
#   free     two processes share a target dir, no lock held
#   single   one process per iteration (no concurrent writer: the control)
#
# Prints one line: repro: mode=<m> iterations=<n> red=<k> ; exits 0 whatever k
# is — a count is the result, not a verdict. Exit 2 on bad arguments.
set -euo pipefail

BIN=${1:-}
N=${2:-}
MODE=${3:-aligned}
if [ -z "$BIN" ] || [ ! -x "$BIN" ] || ! [[ "$N" =~ ^[1-9][0-9]*$ ]]; then
  echo "usage: $0 <lib-test-binary> <iterations> [aligned|free|single]" >&2
  exit 2
fi
case "$MODE" in aligned | free | single) ;; *)
  echo "unknown mode: $MODE" >&2
  exit 2
  ;;
esac

MOD=cli::analysis_utilities::dead_code_outcome_tests
TESTS=(
  "$MOD::a_crate_that_does_not_compile_is_reported_as_not_measured"
  "$MOD::the_same_crate_compiling_is_measured_with_no_disclosure"
  "$MOD::a_directory_without_a_manifest_is_not_applicable_not_unmeasured"
)
WORK=$(mktemp -d "${REPRO_DIR:-${TMPDIR:-/tmp}}/pmat-1305-race.XXXXXX")
TARGET="$WORK/target"
mkdir -p "$TARGET/debug"

run_suite() { # run_suite <log>
  env -u RUST_MIN_STACK CARGO_TARGET_DIR="$TARGET" "$BIN" --exact "${TESTS[@]}" --test-threads=8 >"$1" 2>&1 || true
}

red=0
for i in $(seq 1 "$N"); do
  pids=()
  if [ "$MODE" = aligned ]; then
    flock "$TARGET/debug/.cargo-lock" flock "$TARGET/debug/.cargo-build-lock" sleep 2 &
    hold=$!
    sleep 0.2
  fi
  if [ "$MODE" = single ]; then
    run_suite "$WORK/$i-test.log" &
    pids+=($!)
  else
    run_suite "$WORK/$i-test.log" &
    pids+=($!)
    run_suite "$WORK/$i-coverage.log" &
    pids+=($!)
  fi
  [ "$MODE" = aligned ] && wait "$hold"
  wait "${pids[@]}"
  for log in "$WORK/$i"-*.log; do
    if grep -q "a_crate_that_does_not_compile_is_reported_as_not_measured \.\.\. FAILED" "$log"; then
      red=$((red + 1))
      echo "RED iteration=$i log=$log"
    fi
  done
done
echo "repro: mode=$MODE iterations=$N red=$red work=$WORK"
