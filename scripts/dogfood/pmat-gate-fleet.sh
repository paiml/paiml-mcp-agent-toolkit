#!/usr/bin/env bash
# RELEASE GATE — the release binary against real sibling repositories.
#
# SKILL.md §11. Every other gate tests pmat against fixtures pmat's own authors
# wrote, so the fixture and the code share an author and confirm each other.
# These repos do not: they are 10 to 60,980 files of code nobody wrote to make
# pmat look good.
#
# Takes no arguments: the runner executes declared gates bare.
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HARNESS="$HERE/pmat-fleet-dogfood.sh"
SRC="${PMAT_FLEET_SRC:-$HOME/src}"
BIN="${PMAT_BIN:-$PWD/target/debug/pmat}"

[ -x "$BIN" ] || { echo "no pmat binary at $BIN — build first"; exit 1; }
[ -r "$HARNESS" ] || { echo "harness missing: $HARNESS"; exit 1; }

# The roster the clean room declares. Absent repos are NAMED, never silently
# dropped — a sweep over an empty set is this protocol's signature false green.
ROSTER="aprender forjar depyler bashrs duende rmedia copia pepita pzsh cohete pforge"
present=0; missing=""; defects=0; probed=0

for r in $ROSTER; do
    d="$SRC/$r"
    if [ ! -d "$d/.git" ]; then missing="$missing $r"; continue; fi
    present=$((present + 1))
    out=$(BIN="$BIN" TIMEOUT=600 OUT="$(mktemp)" timeout 900 bash "$HARNESS" "$d" 2>&1)
    rc=$?
    probed=$((probed + 8))
    if [ $rc -ne 0 ]; then
        defects=$((defects + 1))
        printf '  ✗ %s\n' "$r"
        printf '%s\n' "$out" | grep -E '✗ PMAT:' | sed 's/^/     /'
    else
        printf '  ✓ %s\n' "$r"
    fi
done

[ -n "$missing" ] && echo "  not present (not probed):$missing"

# A gate that probed nothing has not passed. Refuse rather than report success
# over an empty set.
if [ "$present" -eq 0 ]; then
    echo "no sibling repo found under $SRC — this gate measured NOTHING, which is"
    echo "not the same as finding nothing. Set PMAT_FLEET_SRC or clone the fleet."
    exit 1
fi

echo "  fleet: $present repo(s), ~$probed probe(s), $defects with PMAT-DEFECTs"
[ "$defects" -eq 0 ]
