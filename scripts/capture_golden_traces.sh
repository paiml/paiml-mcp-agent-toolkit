#!/usr/bin/env bash
# Golden trace capture for pmat CLI dispatcher (PMAT-611).
#
# Writes a renacer-native baseline (manifest.json + syscalls.trace +
# timing.stats) at `golden_traces/baseline/`. The `pmat work complete`
# quality gate runs:
#
#   renacer validate --baseline golden_traces/baseline --ignore-timing \
#       -- ./target/release/pmat --help
#
# so this script captures the same flow. Extra dispatcher flows (version,
# list, hooks status) are captured into subdirectories for operator use;
# they are not consulted by the work-complete gate but make regressions in
# unrelated code paths easy to inspect with `renacer validate`.
#
# Usage: scripts/capture_golden_traces.sh
#
# bashrs-disable: SEC010 SEC011 SEC014 SC2114 SC2320
#   SEC010/SEC014: $REPO_ROOT comes from `git rev-parse` (controlled source)
#   SEC011/SC2114: `rm -rf` targets use `${var:?}` guard below
#   SC2320: positional parameter passthrough is intentional
set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel)
BASELINE="$REPO_ROOT/golden_traces/baseline"
BIN="$REPO_ROOT/target/release/pmat"

if [[ ! -x "$BIN" ]]; then
    echo "Building release binary..."
    (cd "$REPO_ROOT" && cargo build --release --bin pmat)
fi

if ! command -v renacer >/dev/null 2>&1; then
    echo "renacer not found in PATH. Install:  cargo install renacer --locked" >&2
    exit 1
fi

RENACER_VERSION=$(renacer --version | awk '{print $2}')
PMAT_VERSION=$("$BIN" --version | awk '{print $2}')

echo "Capturing golden traces (pmat $PMAT_VERSION, renacer $RENACER_VERSION)..."

# Primary baseline: pmat --help (validated by `pmat work complete`).
rm -rf "${BASELINE:?}"
mkdir -p "$BASELINE"
echo "  [primary] $BIN --help"
renacer validate --generate "$BASELINE" -- "$BIN" --help >/dev/null

# Extra dispatcher flows for operator inspection; ignored by work-complete.
capture_extra() {
    local name=$1
    shift
    local dir="$BASELINE/extra/$name"
    echo "  [$name] $*"
    rm -rf "${dir:?}"
    mkdir -p "$dir"
    renacer validate --generate "$dir" -- "$@" >/dev/null
}

capture_extra version      "$BIN" --version
capture_extra list         "$BIN" list --format json
capture_extra hooks_status "$BIN" hooks status

echo ""
echo "Done. Baseline refreshed at: $BASELINE"
ls -1 "$BASELINE"
