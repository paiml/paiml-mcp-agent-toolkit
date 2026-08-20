#!/usr/bin/env bash
# Golden trace capture for the pmat CLI dispatcher (PMAT-611).
#
# Writes two renacer-native baselines. Each is a directory holding
# manifest.json + syscalls.trace + timing.stats:
#
#   .renacer/                consumed by the dogfood gate, which runs
#                              renacer validate --baseline .renacer -- <bin> --version
#   golden_traces/baseline/  consumed by `pmat work complete`, which runs
#                              renacer validate --baseline golden_traces/baseline \
#                                  --ignore-timing -- ./target/release/pmat --help
#
# Extra dispatcher flows (version, list, hooks status) land in
# golden_traces/baseline/extra/ for operator inspection; no gate reads them.
#
# ---------------------------------------------------------------------------
# PORTABILITY -- why this script never names the binary directly
# ---------------------------------------------------------------------------
# renacer copies the traced argv verbatim into manifest.json. Passing an
# absolute path therefore pins the committed baseline to one machine's home
# directory, which is what `pmat analyze hardcoded-paths` used to flag in all
# four manifests here:
#
#     "command": ["/home/<user>/src/paiml-mcp-agent-toolkit/target/release/pmat", ...]
#
# A baseline naming one user's home can only ever validate on that machine, so
# it is evidence of nothing. Cargo's own executable path is no fix: it is
# absolute too, and when CARGO_TARGET_DIR is redirected it does not even sit
# under the repo (`target/` here is a symlink to a DIFFERENT directory than the
# one cargo builds into, so the checked-in `target/release/pmat` can be a stale
# binary from an older build).
#
# So: resolve the binary from cargo's own JSON output (authoritative, never
# hand-written), expose it through a repo-relative symlink, and trace through
# that symlink. Every manifest then records the same machine-independent argv
# on every machine:
#
#     "command": ["./.renacer-bin", "--version"]
#
# ---------------------------------------------------------------------------
# WHAT THIS GATE DOES AND DOES NOT PROVE (renacer 0.10.2)
# ---------------------------------------------------------------------------
# `renacer validate --generate` writes a 36-byte header-only syscalls.trace
# (magic RNTR ... RTNR, zero records) and an all-zero timing.stats, even though
# it traces and prints hundreds of syscalls to the terminal. With no records
# stored, `validate --baseline` cannot compare anything: it returns 0 for ANY
# command. Measured on this tree -- `/bin/true` validates clean against a
# `pmat --version` baseline.
#
# The gate therefore distinguishes exactly two states:
#   exit 2  no baseline present
#   exit 0  a well-formed baseline is present
# It does NOT prove syscall behaviour is unchanged. Do not read a green renacer
# gate as behavioural equivalence until renacer persists trace records.
#
# Usage: scripts/capture_golden_traces.sh
set -euo pipefail

# Every mkdir/rm -rf below is rooted at a path derived from `git rev-parse`.
# That is normally trustworthy, but a worktree path containing '..' (or an empty
# result) would send `rm -rf` outside the repository, so each path is checked
# before it is used.
validate_repo_path() {
    case "$1" in
        "" | *..*)
            echo "Refusing unsafe path: '$1'" >&2
            return 1
            ;;
    esac
}

REPO_ROOT=$(git rev-parse --show-toplevel)
validate_repo_path "$REPO_ROOT" || exit 1
cd "$REPO_ROOT" || exit 1

if ! command -v renacer >/dev/null 2>&1; then
    echo "renacer not found in PATH. Install:  cargo install renacer --locked" >&2
    exit 1
fi

# --- resolve the binary from cargo, never by hand ---------------------------
# `cargo build --message-format=json` reports the executable it actually
# produced, which is the only trustworthy answer when CARGO_TARGET_DIR is
# redirected. Build errors are surfaced rather than swallowed.
echo "Building release binary..."
BUILD_JSON=$(cargo build --release --bin pmat --message-format=json)
REAL_BIN=$(printf '%s' "$BUILD_JSON" \
    | sed -n 's/.*"executable":"\([^"]*pmat\)".*/\1/p' | tail -1)

if [ -z "$REAL_BIN" ] || [ ! -x "$REAL_BIN" ]; then
    echo "cargo build --release --bin pmat produced no executable" >&2
    exit 1
fi

# --- repo-relative shim, so recorded argv carries no machine path -----------
SHIM=".renacer-bin"
validate_repo_path "$REPO_ROOT/$SHIM" || exit 1
ln -sfn "$REAL_BIN" "$SHIM"
# shellcheck disable=SC2064  # expand REPO_ROOT/SHIM now, not at trap time
trap "rm -f '$REPO_ROOT/$SHIM'" EXIT
TRACE_BIN="./$SHIM"

RENACER_VERSION=$(renacer --version | awk '{print $2}')
PMAT_VERSION=$("$TRACE_BIN" --version | awk 'NR==1{print $2}')
echo "Capturing golden traces (pmat $PMAT_VERSION, renacer $RENACER_VERSION)"
echo "  tracing $TRACE_BIN -> $REAL_BIN"

# generate <dir> <args...>  -- refresh one baseline directory
generate() {
    local dir=$1
    shift
    validate_repo_path "$dir" || return 1
    if [ -z "$dir" ]; then
        echo "Refusing to clear an empty capture directory" >&2
        return 1
    fi
    echo "  [$dir] $TRACE_BIN $*"
    rm -rf "${dir:?}"
    mkdir -p "$dir"
    renacer validate --generate "$dir" -- "$TRACE_BIN" "$@" >/dev/null
}

# Baseline read by the dogfood gate (`--baseline .renacer -- <bin> --version`).
generate ".renacer" --version

# Baseline read by `pmat work complete` (traces `--help`).
generate "golden_traces/baseline" --help

# Extra dispatcher flows, for operator inspection only.
generate "golden_traces/baseline/extra/version" --version
generate "golden_traces/baseline/extra/list" list --format json
generate "golden_traces/baseline/extra/hooks_status" hooks status

# --- source-correlated JSON trace ------------------------------------------
# renacer 0.6.5 wrote its "DWARF debug info loaded from <abs path>" banner to
# stdout, which both pinned this artifact to one machine AND made line 1
# invalid JSON. 0.10.2 sends that banner to stderr, so only the tracee's own
# stdout has to be trimmed: drop everything before renacer's top-level '{'.
# Safe for `--version` specifically, whose output is three lines of plain text
# with no column-0 brace to confuse the match.
SRC_TRACE="golden_traces/pmat_version_source.json"
validate_repo_path "$SRC_TRACE" || exit 1
echo "  [$SRC_TRACE] renacer -s --format json"
renacer -s --format json -- "$TRACE_BIN" --version 2>/dev/null \
    | sed -n '/^{$/,$p' > "$SRC_TRACE"

if ! head -c 1 "$SRC_TRACE" | grep -q '{'; then
    echo "Source trace capture produced no JSON object" >&2
    exit 1
fi

echo ""
echo "Done. Baselines refreshed:"
ls -1 .renacer
echo "---"
ls -1 golden_traces/baseline
