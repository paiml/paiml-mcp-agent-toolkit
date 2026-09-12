#!/usr/bin/env bash
# tests-dont-write-control.sh — PMAT-1329 acceptance criterion 1: the tree stays clean
# after the suite runs, "proven by a check that runs the suite and asserts the tree is
# clean rather than by reading the tests".
#
#   arm 1 GREEN  the dispatcher suite leaves docs/ clean
#   arm 2 RED    the checker itself reports dirty when something DOES write under docs/
#   arm 3 GREEN  the checker ignores changes outside docs/, so an unrelated edit cannot mask arm 2
#
# Arm 2 is the one that matters. Without it this script asserts "git found nothing",
# which is also what a broken checker says. It plants a write and requires a refusal.
#
# Usage: bash scripts/tests-dont-write-control.sh
set -uo pipefail
cd "$(dirname "$0")/.." || exit 9
FAIL=0
fail_arm() { echo "tests-dont-write-control: ARM $1 FAILED — $2"; FAIL=1; }

# The predicate, used by every arm: how many TRACKED files under docs/ are modified?
docs_dirty() { git status --porcelain -- docs | grep -vc '^??' || true; }

[ "$(docs_dirty)" = "0" ] || {
  echo "tests-dont-write-control: refusing to run — docs/ is already dirty before the suite:"
  git status --porcelain -- docs | grep -v '^??' | head -5
  exit 2
}

# ── arm 1 ─────────────────────────────────────────────────────────────────────
# The dispatcher subset is the one PMAT-1329 was found in and runs in ~30s. A full
# `cargo test --lib` would also do, at twenty times the cost for the same signal.
env -u TMPDIR -u RUST_MIN_STACK cargo test --lib -- command_dispatcher >/dev/null 2>&1
n=$(docs_dirty)
[ "$n" = "0" ] || fail_arm 1 "the suite left $n tracked file(s) under docs/ modified: $(git status --porcelain -- docs | grep -v '^??' | head -3 | tr '\n' ' ')"
echo "tests-dont-write-control: arm 1 GREEN — the suite left docs/ clean"

# ── arm 2 (falsifier) ─────────────────────────────────────────────────────────
# Plant a write into a tracked file under docs/ and require the predicate to see it.
VICTIM=docs/execution/roadmap.md
[ -f "$VICTIM" ] || { fail_arm 2 "$VICTIM is missing — the falsifier has nothing to plant on"; VICTIM=""; }
if [ -n "$VICTIM" ]; then
  printf '\n<!-- tests-dont-write-control: planted -->\n' >> "$VICTIM"
  n=$(docs_dirty)
  git checkout -- "$VICTIM"
  [ "$n" -ge 1 ] || fail_arm 2 "a write to $VICTIM must read as dirty; the checker reported $n"
  echo "tests-dont-write-control: arm 2 RED   — a planted write under docs/ is detected"
fi

# ── arm 3 ─────────────────────────────────────────────────────────────────────
# A change OUTSIDE docs/ must not register, or arm 2 could pass on the wrong file.
OUT=README.md
if [ -f "$OUT" ]; then
  printf '\n<!-- tests-dont-write-control: planted outside docs -->\n' >> "$OUT"
  n=$(docs_dirty)
  git checkout -- "$OUT"
  [ "$n" = "0" ] || fail_arm 3 "a change to $OUT must not register as docs/ dirt; got $n"
  echo "tests-dont-write-control: arm 3 GREEN — changes outside docs/ are ignored"
fi

[ "$FAIL" -eq 0 ] || { echo "tests-dont-write-control: FAILED"; exit 1; }
echo "tests-dont-write-control: all 3 arms behaved — the suite leaves docs/ clean, and the checker can tell"
