#!/usr/bin/env bash
# Ask ONE question three ways — CLI, MCP stdio, HTTP — and compare the answers.
#
# Why. A CLI-only sweep proves nothing about the other transports, and this
# project's own history records 24 MCP-vs-CLI contradictions in a single round.
# The 3.32.0 surface audit put numbers on the gap: of the 16 tools the shipped
# binary actually serves over MCP, ZERO had a top-tier coverage row, while the
# 18 it does not serve scored higher. So the tested surface and the served
# surface were close to disjoint.
#
# The finding this produces is not "a transport is broken". It is
# "the transports DISAGREE" — which is worse, because each looks right alone.
#
# CLI_BIN  a pmat built however you like (HTTP need not be compiled in)
# HTTP_BIN a pmat built with --features mcp-http; omit to skip the HTTP leg,
#          which is reported as SKIPPED, never silently passed.

set -uo pipefail
CLI_BIN="${CLI_BIN:?set CLI_BIN}"
HTTP_BIN="${HTTP_BIN:-}"
REPO="${1:?usage: transport-parity.sh <repo-path>}"
NAME="$(basename "$REPO")"
TIMEOUT="${TIMEOUT:-300}"
PORT="${PORT:-18765}"
OUT="${OUT:-/tmp/parity-$NAME.json}"

disagreements=(); notes=(); compared=0

say() { printf '  %s\n' "$*"; }

# One JSON-RPC round trip over stdio. Returns the tool's text payload.
mcp_call() { # mcp_call <tool> <args-json>
  local tool="$1" args="$2"
  { printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"parity","version":"1"}}}'
    printf '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"%s","arguments":%s}}\n' "$tool" "$args"
  } | timeout "$TIMEOUT" env MCP_VERSION=1 "$CLI_BIN" 2>/dev/null \
    | python3 -c '
import json, sys
for line in sys.stdin:
    line = line.strip()
    if not line: continue
    try: d = json.loads(line)
    except Exception: continue
    if d.get("id") != 2: continue
    if "error" in d:
        print("MCP_ERROR " + json.dumps(d["error"])[:200]); break
    for c in d.get("result", {}).get("content", []) or []:
        if c.get("type") == "text":
            print(c.get("text", "")); break
    break
'
}

# Pull one comparable scalar out of whatever shape the payload has.
scalar() { # scalar <json> <dotted-key-candidates...>
  local payload="$1"; shift
  printf '%s' "$payload" | python3 -c '
import json, sys
keys = sys.argv[1:]
raw = sys.stdin.read().strip()
try: d = json.loads(raw)
except Exception:
    print("UNPARSEABLE"); raise SystemExit
def find(o, k):
    if isinstance(o, dict):
        if k in o: return o[k]
        for v in o.values():
            r = find(v, k)
            if r is not None: return r
    elif isinstance(o, list):
        for v in o:
            r = find(v, k)
            if r is not None: return r
    return None
for k in keys:
    v = find(d, k)
    if v is not None and not isinstance(v, (dict, list)):
        print(v); raise SystemExit
print("ABSENT")
' "$@"
}

compare() { # compare <label> <cli-json> <mcp-text> <key...>
  local label="$1" cli="$2" mcp="$3"; shift 3
  compared=$((compared + 1))
  local a b
  a=$(scalar "$cli" "$@")
  b=$(scalar "$mcp" "$@")
  say "$label: cli=$a  mcp=$b"
  case "$b" in
    MCP_ERROR*|*"-32602"*)
      # A JSON-RPC validation error is almost always the HARNESS calling the
      # tool wrongly, not pmat misbehaving. Reported as a harness fault so it
      # cannot be mistaken for a defect in the thing under test — the first run
      # of this script called every tool with `project_path` when the schema
      # says `paths`, and produced three confident phantom findings.
      notes+=("$label: harness called the tool wrongly — $b")
      return ;;
  esac
  if [ "$b" = "UNPARSEABLE" ]; then
    # An MCP tool whose payload is prose where the CLI returns JSON is itself
    # the contradiction — a caller cannot consume both the same way.
    disagreements+=("$label: MCP payload is not JSON while CLI --format json is (cli=$a)")
    return
  fi
  [ "$a" = "ABSENT" ] && [ "$b" = "ABSENT" ] && { notes+=("$label: neither transport exposes this key"); return; }
  [ "$a" != "$b" ] && disagreements+=("$label: CLI says $a, MCP says $b — same question, same repo, two answers")
}

say "── $NAME ──────────────────────────────────────────────"

CLI_CX=$(timeout "$TIMEOUT" "$CLI_BIN" analyze complexity --path "$REPO" --format json 2>/dev/null)
MCP_CX=$(mcp_call analyze_complexity "$(printf '{"paths":["%s"]}' "$REPO")")
compare "complexity/files"  "$CLI_CX" "$MCP_CX" files_analyzed total_files files_discovered

CLI_SD=$(timeout "$TIMEOUT" "$CLI_BIN" analyze satd --path "$REPO" --format json 2>/dev/null)
MCP_SD=$(mcp_call analyze_satd "$(printf '{"paths":["%s"]}' "$REPO")")
# NOT the bare key `total`. `scalar` searches RECURSIVELY, and the satd payload
# carries `files_not_read.total` — so a fallback to `total` found the count of
# files DECLINED (8) and compared it against MCP's count of violations (0),
# reporting a transport disagreement where the two agree exactly. The first run
# of this gate produced that false finding. Name the field that means what the
# label says.
compare "satd/total"        "$CLI_SD" "$MCP_SD" total_violations total_satd

CLI_DC=$(timeout "$TIMEOUT" "$CLI_BIN" analyze dead-code --path "$REPO" --format json 2>/dev/null)
MCP_DC=$(mcp_call analyze_dead_code "$(printf '{"paths":["%s"]}' "$REPO")")
compare "dead-code/files"   "$CLI_DC" "$MCP_DC" files_analyzed total_files

# ── HTTP leg. Absent binary is SKIPPED and said so, never silently passed.
if [ -n "$HTTP_BIN" ] && [ -x "$HTTP_BIN" ]; then
  # >=16 chars: the server REFUSES to start below that ("PMAT_MCP_HTTP_TOKEN
  # must be at least 16 characters"), and `parity-$$` is at most 13. So this
  # leg had never once run — every invocation died at startup and was recorded
  # as the indistinguishable "server did not answer".
  TOKEN="parity-token-$$-$(od -An -N6 -tx1 /dev/urandom | tr -d ' \n')"
  # Keep the server's stderr. Discarding it is what made this a mystery: the
  # process printed the exact reason on line 1 and the harness sent it to
  # /dev/null, leaving a symptom with no cause.
  # pmat serves the MCP endpoint at the ROOT ("listening on http://HOST:PORT/").
  # The harness posted to /mcp, which 404s. Measured, not assumed:
  #   POST /  -> 200      POST /mcp -> 404      POST /health -> 404
  MCP_PATH="${PMAT_MCP_PATH:-/}"
  SRV_LOG="$(mktemp)"
  PMAT_MCP_HTTP_TOKEN="$TOKEN" timeout 120 "$HTTP_BIN" serve --transport http --port "$PORT" >"$SRV_LOG" 2>&1 &
  SRV=$!
  # Readiness by speaking MCP, not by polling /health — there IS no /health
  # (it 404s), so the old loop could never break and simply burned its 40
  # iterations before posting to a server it had not confirmed was up.
  for _ in $(seq 1 40); do
    rc=$(curl -sS -m 2 -o /dev/null -w '%{http_code}' -X POST \
      -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
      -H 'Accept: application/json, text/event-stream' \
      -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"parity","version":"1"}}}' \
      "http://127.0.0.1:$PORT$MCP_PATH" 2>/dev/null)
    [ "$rc" = "200" ] && break
    sleep 0.25
  done
  # `Accept: application/json, text/event-stream` is REQUIRED by the streamable
  # HTTP transport; without it the request is rejected, and `curl -f` turns that
  # into an empty string with no message — the third independent reason this leg
  # never ran. Measured: with the header, tools/call returns 200 and 18,873 bytes.
  HTTP_CX=$(curl -fsS -m "$TIMEOUT" -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
    -H 'Accept: application/json, text/event-stream' \
      -d "$(printf '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"analyze_complexity","arguments":{"paths":["%s"]}}}' "$REPO")" \
      "http://127.0.0.1:$PORT$MCP_PATH" 2>/dev/null \
    | python3 -c '
import json,sys
try: d=json.load(sys.stdin)
except Exception: print(""); raise SystemExit
for c in d.get("result",{}).get("content",[]) or []:
    if c.get("type")=="text": print(c.get("text","")); break
' 2>/dev/null)
  kill "$SRV" 2>/dev/null; wait "$SRV" 2>/dev/null
  if [ -z "$HTTP_CX" ]; then
    # Quote the server's own words: "did not answer" alone cannot distinguish
    # "refused to start" from "wrong port", "auth rejected" or "tool errored".
    why=$(head -3 "$SRV_LOG" 2>/dev/null | tr '\n' ';' | sed 's/;*$//')
    notes+=("HTTP: server did not answer tools/call on port $PORT${why:+ — server said: $why}")
    say "http: NO ANSWER${why:+ — $why}"
  else
    compared=$((compared + 1))
    h=$(scalar "$HTTP_CX" files_analyzed total_files files_discovered)
    m=$(scalar "$MCP_CX" files_analyzed total_files files_discovered)
    say "complexity/files: http=$h  (mcp=$m)"
    [ "$h" != "$m" ] && disagreements+=("complexity/files: HTTP says $h, MCP stdio says $m — one server, two answers")
  fi
  rm -f "$SRV_LOG"
else
  notes+=("HTTP leg SKIPPED: no binary built with --features mcp-http")
  say "http: SKIPPED (no mcp-http build)"
fi

printf '{"repo":"%s","compared":%s,"disagreements":%s,"notes":%s}\n' \
  "$NAME" "$compared" \
  "$(printf '%s\n' "${disagreements[@]:-}" | python3 -c 'import json,sys;print(json.dumps([l for l in sys.stdin.read().split("\n") if l.strip()]))')" \
  "$(printf '%s\n' "${notes[@]:-}"        | python3 -c 'import json,sys;print(json.dumps([l for l in sys.stdin.read().split("\n") if l.strip()]))')" \
  > "$OUT"

say "compared=$compared  DISAGREEMENTS=${#disagreements[@]}"
for d in "${disagreements[@]:-}"; do [ -n "$d" ] && say "  ✗ $d"; done
for n in "${notes[@]:-}";        do [ -n "$n" ] && say "  · $n"; done
[ "${#disagreements[@]}" -eq 0 ]
