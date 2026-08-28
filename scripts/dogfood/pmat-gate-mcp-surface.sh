#!/usr/bin/env bash
# RELEASE GATE — the MCP surface itself, over stdio AND over HTTP.
#
# "mcp mode is critical and non-optional" is the standing instruction for this
# crate. Until this gate existed, nothing in the release protocol tested it as a
# SURFACE. The parity gate compares CLI/stdio/HTTP answers to three analyze
# questions — real, and orthogonal — but it calls three of the nineteen served
# tools. The other sixteen were advertised in `tools/list` and never invoked by
# any gate, so a tool that errored on every input shipped green.
#
# This gate asserts four things the parity gate cannot:
#
#   1. `pmat --mode mcp` (MCP_VERSION=1) initializes and answers tools/list
#   2. `pmat serve --transport http` starts and answers tools/list at the ROOT
#   3. the two tool sets are IDENTICAL — any tool in one and not the other is
#      named in the failure
#   4. EVERY advertised tool is INVOCABLE on BOTH transports against a real
#      fixture — a result, not an error, not a hang, not an empty payload
#
# ── THE TOOL-COUNT RATCHET ─────────────────────────────────────────────────
# 19 tools are served, over each transport, as of 3.32.0. Nothing else in the
# tree fails if that silently becomes 12. Reproduce the number yourself — this
# is the exact command, and it prints 19 today:
#
#   printf '%s\n%s\n' \
#     '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"c","version":"1"}}}' \
#     '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
#   | MCP_VERSION=1 pmat \
#   | python3 -c 'import json,sys; [print(len(json.loads(l)["result"]["tools"])) for l in sys.stdin if json.loads(l).get("id")==2]'
#
# The HTTP server prints its own count on line 3 of startup ("tools: 19").
#
# The ratchet is EXACT, not a floor. A drop is a regression; a RISE is a new
# tool that arrived without an argument recipe in the harness, i.e. advertised
# and untested — which is the condition this gate was built to refuse. Either
# direction fails, and the fix for a legitimate rise is to add the recipe and
# raise this number in the same commit.
EXPECTED_TOOLS="${EXPECTED_TOOLS:-19}"

# Takes no arguments: the runner executes declared gates bare.
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HARNESS="$HERE/pmat-mcp-surface.sh"

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
[ -r "$HARNESS" ] || { echo "harness missing: $HARNESS — a declared gate whose harness is gone is a deleted gate, not a satisfied one"; exit 1; }

RECEIPT="$(mktemp)"
HLOG="$(mktemp)"
cleanup() { rm -f "$RECEIPT" "$HLOG"; }
trap cleanup EXIT INT TERM

# ONE binary serves both transports: mcp-http is in the DEFAULT feature set as
# of 3.32.0. STDIO_BIN and HTTP_BIN are the same file on purpose — if they ever
# have to differ, the shipped binary does not serve MCP over HTTP, which is
# itself the finding.
#
# No pipe here. `$?` after a pipe is the LAST command's status, and reading a
# harness's exit code through `| tee` is how six of this project's gates
# reported the status of `tee`.
STDIO_BIN="$BIN" HTTP_BIN="$BIN" OUT="$RECEIPT" TIMEOUT="${TIMEOUT:-120}" \
    timeout "${GATE_TIMEOUT:-600}" bash "$HARNESS" > "$HLOG" 2>&1
rc=$?
cat "$HLOG"

if [ "$rc" -eq 124 ]; then
    echo "  ✗ the MCP surface harness did not finish within ${GATE_TIMEOUT:-600}s — a hung"
    echo "    transport is a failure, never a skip. Its output up to the hang is above."
    exit 1
fi

# Read the RECEIPT, not the human-readable log. The log is for people; the JSON
# is the evidence, and a verdict keyed on prose is a verdict keyed on wording.
read -r STDIO_N HTTP_N ONLY_STDIO ONLY_HTTP FAILS <<EOF
$(python3 - "$RECEIPT" <<'PY'
import json, sys
try:
    with open(sys.argv[1]) as f:
        d = json.load(f)
except Exception:
    # No receipt at all means the harness died before it could write one. That
    # is a failure with an unknown cause, and it must never read as zero
    # failures over zero tools.
    print("UNREADABLE UNREADABLE - - -")
    raise SystemExit
print(d.get("stdio_count", -1),
      d.get("http_count", -1),
      ",".join(d.get("only_stdio") or []) or "-",
      ",".join(d.get("only_http") or []) or "-",
      len(d.get("failures") or []))
PY
)
EOF

if [ "$STDIO_N" = "UNREADABLE" ]; then
    echo "  ✗ the harness wrote no readable receipt (exit=$rc) — this gate measured NOTHING."
    echo "    Its output is above; that is the only evidence there is."
    exit 1
fi

# The command a human runs to check the number themselves, printed with the
# failure so it does not have to be reconstructed from this file. A QUOTED
# heredoc, because every character of it is literal: the `\n` must survive as
# two characters, and the trailing ` \` must not be read as a line
# continuation. This block was `echo` until shellcheck's SC2028 pointed out
# that `echo` is not required to leave either alone.
repro_command() {
    sed "s|PMAT_BINARY|$BIN|" <<'EOF'
      printf '%s\n%s\n' \
        '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"c","version":"1"}}}' \
        '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
      | MCP_VERSION=1 PMAT_BINARY \
      | python3 -c 'import json,sys; [print(len(json.loads(l)["result"]["tools"])) for l in sys.stdin if json.loads(l).get("id")==2]'
EOF
}

# ── the ratchet. Checked here, in the gate, so the number and the command that
# reproduces it sit next to each other where a human reading the failure can
# act on it without opening the harness.
ratchet_ok=yes
for pair in "stdio:$STDIO_N" "http:$HTTP_N"; do
    t="${pair%%:*}"; n="${pair##*:}"
    if [ "$n" != "$EXPECTED_TOOLS" ]; then
        ratchet_ok=""
        echo "  ✗ $t serves $n tool(s); this gate is ratcheted at $EXPECTED_TOOLS."
        if [ "$n" -lt "$EXPECTED_TOOLS" ] 2>/dev/null; then
            echo "    Tools were DROPPED from the $t surface. Reproduce the live count with:"
        else
            echo "    Tools were ADDED to the $t surface but no argument recipe was added to"
            echo "    $HARNESS, so the new one is advertised and untested. Add the recipe"
            echo "    from its inputSchema, then raise EXPECTED_TOOLS in the same commit."
            echo "    Reproduce the live count with:"
        fi
        repro_command
    fi
done

[ "$ONLY_STDIO" = "-" ] || echo "  ✗ served over stdio but NOT over HTTP: $ONLY_STDIO"
[ "$ONLY_HTTP" = "-" ]  || echo "  ✗ served over HTTP but NOT over stdio: $ONLY_HTTP"

echo "  mcp-surface: stdio=$STDIO_N http=$HTTP_N tools (ratchet $EXPECTED_TOOLS)," \
     "$FAILS failed invocation(s) of $(( (STDIO_N + HTTP_N) )) attempted"

# Every clause must hold. The harness's own exit code is one of them: it is the
# only one that knows about hangs, empty payloads and a missing argument recipe.
[ "$rc" -eq 0 ] \
    && [ -n "$ratchet_ok" ] \
    && [ "$ONLY_STDIO" = "-" ] \
    && [ "$ONLY_HTTP" = "-" ] \
    && [ "$FAILS" = "0" ]
