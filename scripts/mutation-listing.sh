#!/usr/bin/env bash
# GH-1846 — write the listing artifact the mutation gate reads when no outcomes.json exists.
#
# Contract (src/services/mutation_gate.rs, MUTANTS_LISTING_FILE / MUTANTS_LISTING_DIFF_FILE):
#   <outdir>/mutants-listing.json  `cargo mutants --list --json --in-diff <diff>` output; empty stdout -> `[]`
#   <outdir>/mutants-listing.diff  byte-equal copy of <diff>, the diff the gate judges
# Usage: scripts/mutation-listing.sh <diff> [outdir]   (outdir defaults to the current directory)
# CARGO_MUTANTS_BIN overrides the cargo-mutants command (the selftest plants a stub).
set -euo pipefail

diff="${1:-}"
outdir="${2:-.}"
[ -f "$diff" ] || { echo "mutation-listing: '$diff' is not a readable diff" >&2; exit 2; }
case "$outdir" in *..*) echo "mutation-listing: outdir must not contain .." >&2; exit 2 ;; esac
mkdir -p "$outdir"

# Capture first: a failing listing must not leave a half-written `[]` behind.
listing="$(${CARGO_MUTANTS_BIN:-cargo mutants} --list --json --in-diff "$diff")" || {
    echo "mutation-listing: cargo mutants --list failed; no listing written" >&2
    exit 1
}
rm -f "${outdir:?}/mutants-listing.json" "${outdir:?}/mutants-listing.diff"
if [ -z "${listing//[[:space:]]/}" ]; then
    listing='[]'
fi
printf '%s\n' "$listing" >"$outdir/mutants-listing.json"
cp "$diff" "$outdir/mutants-listing.diff"
