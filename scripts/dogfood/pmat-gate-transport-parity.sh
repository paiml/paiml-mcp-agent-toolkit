#!/usr/bin/env bash
# RELEASE GATE — one question asked over CLI, MCP stdio and HTTP.
#
# SKILL.md §12. A CLI-only sweep proves nothing about the other transports, and
# this project's history records 24 MCP-vs-CLI contradictions in a single round.
# The finding this produces is not "a transport is broken" — it is "the
# transports DISAGREE", which is worse, because each looks right alone.
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HARNESS="$HERE/pmat-transport-parity.sh"
SRC="${PMAT_FLEET_SRC:-$HOME/src}"
BIN="${PMAT_BIN:-$PWD/target/debug/pmat}"

[ -x "$BIN" ] || { echo "no pmat binary at $BIN — build first"; exit 1; }
[ -r "$HARNESS" ] || { echo "harness missing: $HARNESS"; exit 1; }

# Small, fast trees: parity is about AGREEMENT, not about scale.
TARGETS="cohete pzsh copia"
present=0; disagreements=0

for r in $TARGETS; do
    d="$SRC/$r"
    [ -d "$d/.git" ] || continue
    present=$((present + 1))
    out=$(CLI_BIN="$BIN" TIMEOUT=300 OUT="$(mktemp)" timeout 900 bash "$HARNESS" "$d" 2>&1)
    if [ $? -ne 0 ]; then
        disagreements=$((disagreements + 1))
        printf '%s\n' "$out" | grep -E '✗' | sed 's/^/  /'
    fi
done

if [ "$present" -eq 0 ]; then
    echo "no target repo present — this gate measured NOTHING. Set PMAT_FLEET_SRC."
    exit 1
fi

echo "  parity: $present repo(s), $disagreements with transport disagreements"
[ "$disagreements" -eq 0 ]
