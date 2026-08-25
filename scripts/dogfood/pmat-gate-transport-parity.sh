#!/usr/bin/env bash
# RELEASE GATE — one question asked over CLI, MCP stdio and HTTP.
#
# SKILL.md §12. A CLI-only sweep proves nothing about the other transports, and
# this project's history records 24 MCP-vs-CLI contradictions in a single round.
# The finding this produces is not "a transport is broken" — it is "the
# transports DISAGREE", which is worse, because each looks right alone.
#
# THE HTTP LEG IS MANDATORY HERE. The harness treats HTTP_BIN as optional and
# reports a missing one as SKIPPED — correct for a harness someone runs by hand
# against an arbitrary build. But this gate never set it, and the gate's verdict
# keys on `disagreements` alone, so a skipped leg produced a GREEN release gate
# that had never spoken HTTP. That is this project's own signature defect —
# absence rendered as success — sitting in the gate meant to catch it.
#
# `mcp-http` is in the DEFAULT feature set as of 3.32.0, so the one binary
# serves all three surfaces and a skip has no legitimate cause. This gate
# therefore asserts the leg RAN, and fails when it did not.
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HARNESS="$HERE/pmat-transport-parity.sh"
SRC="${PMAT_FLEET_SRC:-$HOME/src}"
# Ask cargo where it builds. This workspace redirects `target-dir` off-site, so
# `$PWD/target/debug/pmat` is a STALE COPY left over from before the redirect —
# 43 minutes behind the real binary when this was found. A release gate that
# measures yesterday's binary is worse than no gate: it reports on code that is
# not being shipped. Never hand-write this path.
if [ -z "${PMAT_BIN:-}" ]; then
    TARGET_DIR=$(cargo metadata --no-deps --format-version 1 2>/dev/null \
        | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])' 2>/dev/null)
    BIN="${TARGET_DIR:-$PWD/target}/debug/pmat"
else
    BIN="$PMAT_BIN"
fi

[ -x "$BIN" ] || { echo "no pmat binary at $BIN — build first"; exit 1; }
[ -r "$HARNESS" ] || { echo "harness missing: $HARNESS"; exit 1; }

# Small, fast trees: parity is about AGREEMENT, not about scale.
TARGETS="cohete pzsh copia"
present=0; disagreements=0; http_legs=0; http_failures=0

# One receipt per target, all removed on any exit path. A gate killed mid-run
# used to leave them behind, and a stale receipt is worse than none: the next
# reader cannot tell it from this run's.
RECEIPTS=()
cleanup() {
    if [ "${#RECEIPTS[@]}" -gt 0 ]; then
        rm -f "${RECEIPTS[@]}"
    fi
}
trap cleanup EXIT INT TERM

for r in $TARGETS; do
    d="$SRC/$r"
    [ -d "$d/.git" ] || continue
    present=$((present + 1))
    receipt="$(mktemp)"; RECEIPTS+=("$receipt")
    # HTTP_BIN is the SAME binary: mcp-http ships in default features.
    out=$(CLI_BIN="$BIN" HTTP_BIN="$BIN" TIMEOUT=300 OUT="$receipt" \
          timeout 900 bash "$HARNESS" "$d" 2>&1)
    rc=$?
    if [ $rc -ne 0 ]; then
        disagreements=$((disagreements + 1))
        printf '%s\n' "$out" | grep -E '✗' | sed 's/^/  /'
    fi

    # Did the HTTP leg actually speak? The harness records a note when it did
    # not. Read the receipt rather than the human-readable log: the log is for
    # people, the JSON is the evidence.
    note=$(python3 -c '
import json,sys
try:
    d=json.load(open(sys.argv[1]))
except Exception as e:
    print("receipt unreadable: %s" % e); raise SystemExit
bad=[n for n in d.get("notes") or [] if "HTTP" in n]
print(bad[0] if bad else "")
' "$receipt" 2>/dev/null)
    if [ -n "$note" ]; then
        http_failures=$((http_failures + 1))
        printf '  ✗ %s: %s\n' "$r" "$note"
    else
        http_legs=$((http_legs + 1))
    fi
done

if [ "$present" -eq 0 ]; then
    echo "no target repo present — this gate measured NOTHING. Set PMAT_FLEET_SRC."
    exit 1
fi

# A parity gate that never spoke HTTP has not tested parity. Refuse.
if [ "$http_legs" -eq 0 ]; then
    echo "  the HTTP leg ran on ZERO of $present repo(s) — this gate compared two"
    echo "  transports, not three, and 'no disagreement' over a surface never"
    echo "  contacted is not a pass. mcp-http is in default features; a skip here"
    echo "  means the server failed to start or answer, which is itself the defect."
    exit 1
fi

echo "  parity: $present repo(s), $disagreements with transport disagreements," \
     "$http_legs/$present HTTP legs answered"
[ "$disagreements" -eq 0 ] && [ "$http_failures" -eq 0 ]
