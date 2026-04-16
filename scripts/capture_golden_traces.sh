#!/usr/bin/env bash
# Golden trace capture for pmat CLI dispatcher (PMAT-611).
#
# Produces JSON syscall traces + strace-style summary for representative
# dispatcher flows. Validation against these baselines is governed by
# `renacer.toml` (key: `[semantic_equivalence] baseline_dir`).
#
# Usage: scripts/capture_golden_traces.sh
#
# bashrs-disable: SC1020 SC1140 SEC010 SEC014 REL002
#   SC1020/SC1140: false positives on JSON brackets inside heredoc
#   SEC010/SEC014: $REPO_ROOT comes from `git rev-parse` (controlled source)
#   REL002: temp file cleanup handled by trap
set -euo pipefail

trap 'rm -rf "${_tmp:-}"' EXIT

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

mkdir -p "$BASELINE"

# Capture one flow: write JSON trace and strace-style summary.
# Renacer writes program stdout, then strace summary (to stderr with -c),
# then JSON trace at end. We split by redirecting stderr to summary file
# and filtering the first non-JSON line(s) out of stdout.
capture() {
    local name=$1
    shift
    echo "  [$name]: $*"
    _tmp=$(mktemp)
    renacer --format json -c -- "$@" >"$_tmp" 2>"$BASELINE/${name}_summary.txt" || true
    # Drop program-stdout preamble (everything before the first '{').
    awk '/^\{/{found=1} found' "$_tmp" >"$BASELINE/${name}.json"
    rm -f "$_tmp"
    unset _tmp
}

echo "Capturing golden traces (pmat $PMAT_VERSION, renacer $RENACER_VERSION)..."
capture pmat_version     "$BIN" --version
capture pmat_help        "$BIN" --help
capture pmat_list        "$BIN" list --format json
capture pmat_hooks_status "$BIN" hooks status

# Refresh manifest with current versions.
cat >"$BASELINE/manifest.json" <<JSON
{
  "version": "1.1.0",
  "renacer_version": "$RENACER_VERSION",
  "pmat_version": "$PMAT_VERSION",
  "platform": {
    "os": "$(uname -s | tr '[:upper:]' '[:lower:]')",
    "arch": "$(uname -m)"
  },
  "traces": [
    {"name": "pmat_version",      "command": ["pmat", "--version"],                "description": "Shortest dispatcher path (no subcommand match)."},
    {"name": "pmat_help",         "command": ["pmat", "--help"],                   "description": "Clap help dispatch."},
    {"name": "pmat_list",         "command": ["pmat", "list", "--format", "json"], "description": "Commands::List -> handlers::handle_list."},
    {"name": "pmat_hooks_status", "command": ["pmat", "hooks", "status"],          "description": "Commands::Hooks -> handlers::handle_hooks_command."}
  ],
  "tolerance": {
    "timing_percent": 15.0,
    "syscall_sequence": "fuzzy",
    "argument_match": "fuzzy"
  },
  "regenerate": "scripts/capture_golden_traces.sh"
}
JSON

echo ""
echo "Done. Baseline refreshed at: $BASELINE"
ls -1 "$BASELINE"
