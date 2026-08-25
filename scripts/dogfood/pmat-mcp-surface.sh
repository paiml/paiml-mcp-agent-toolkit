#!/usr/bin/env bash
# Does the MCP SURFACE work — over stdio and over HTTP — tool by tool?
#
# Why this exists alongside the parity harness. Parity asks ONE question three
# ways and compares the answers; it proves the transports AGREE about the three
# things it asks. It cannot prove the surface WORKS. A tool that is advertised
# in `tools/list` and returns a JSON-RPC error on every input passes the parity
# gate untouched, because parity never calls it. Sixteen of the nineteen tools
# were in exactly that position: served, and never once invoked by a gate.
#
# The project owner's mandate is "mcp mode is critical and non-optional". This
# harness is what makes that assertion falsifiable:
#
#   1. stdio initializes and answers tools/list
#   2. HTTP starts, and answers tools/list AT THE ROOT PATH
#   3. the two tool sets are IDENTICAL, set-wise (HTTP returns them in HashMap
#      order — compare sorted sets, never the raw order, or this flaps)
#   4. EVERY advertised tool is INVOCABLE on BOTH transports against a real
#      fixture: it returns a non-empty result, not an error and not a hang
#
# FAIL-CLOSED. There is no leg here that can be skipped. `mcp-http` is in the
# DEFAULT feature set as of 3.32.0, so one binary serves all three surfaces and
# "no HTTP binary" has no legitimate cause. The parity harness treats HTTP_BIN
# as optional — correct for a harness run by hand against an arbitrary build,
# and the reason its HTTP leg had never once run while the gate stayed green.
# HTTP_BIN is REQUIRED here.
#
# WHAT COUNTS AS A DEFECT, stated so the numbers can be argued with. Every tool
# is called with arguments that SATISFY ITS OWN DECLARED inputSchema, against a
# fixture that is a real git repo with real Rust in it. Under that contract any
# JSON-RPC error is counted as a failure — including -32602 "validation error",
# which elsewhere would be a legitimate argument-validation response. The one
# case that arose was `analyze_vacuous_tests`, which errors with
# "no #[test] functions found" on a fixture that has none. That was fixed by
# giving the fixture a test — not by exempting the tool. An exemption list is
# how a tool that errors on everything ends up permanently excused.
#
# ENV
#   STDIO_BIN  pmat binary to drive over MCP stdio            (required)
#   HTTP_BIN   pmat binary to run as an MCP HTTP server       (required)
#   PORT       HTTP port                                      (default 18791)
#   TIMEOUT    per-call seconds; exceeded is HANG, not error  (default 120)
#   OUT        receipt JSON path                              (default under /tmp)
#
# EXIT  0 = every leg ran and every tool answered on both transports.

set -uo pipefail

STDIO_BIN="${STDIO_BIN:?set STDIO_BIN — the pmat binary to drive over MCP stdio}"
HTTP_BIN="${HTTP_BIN:?set HTTP_BIN — the pmat binary to serve MCP over HTTP. There is deliberately no skip: mcp-http ships in default features, so an absent HTTP leg is the defect, not a configuration}"
# Ask the OS for a free port rather than hard-coding one. Measured with 18791
# deliberately occupied, this gate goes RED with the server's own words —
# "Address already in use (os error 98)" — which is correct fail-closed
# behaviour and the WRONG verdict: a busy port is not a pmat defect. Nine
# agents share this machine and CI runs jobs side by side, so a fixed port
# turns a release gate into a coin flip. PORT is still honoured when set.
PORT="${PORT:-$(python3 -c '
import socket
s = socket.socket()
s.bind(("127.0.0.1", 0))
print(s.getsockname()[1])
s.close()
')}"
[ -n "$PORT" ] || { echo "could not obtain a port to serve MCP over HTTP on"; exit 1; }
TIMEOUT="${TIMEOUT:-120}"
OUT="${OUT:-/tmp/pmat-mcp-surface.json}"
# pmat serves MCP at the ROOT. Measured on 3.32.0, not assumed:
#   POST /  -> 200      POST /mcp -> 404      POST /health -> 404
MCP_PATH="${PMAT_MCP_PATH:-/}"

[ -x "$STDIO_BIN" ] || { echo "STDIO_BIN is not executable: $STDIO_BIN"; exit 1; }
[ -x "$HTTP_BIN" ]  || { echo "HTTP_BIN is not executable: $HTTP_BIN"; exit 1; }

WORK="$(mktemp -d)"
# bashrs SEC011, and it is right: `rm -rf "$WORK"` in the cleanup trap is only
# safe while WORK is a directory we made. An unchecked `mktemp -d` failure
# leaves it empty, and an empty expansion is exactly the shape that turns a
# cleanup into an incident. Check it once, here, rather than reasoning about it
# at every use.
if [ -z "$WORK" ] || [ ! -d "$WORK" ]; then
    echo "mktemp -d produced no usable work directory (got '${WORK}') — refusing to run"
    exit 1
fi
FIX="$WORK/fixture"
RESULTS="$WORK/results.tsv"
NOTES="$WORK/notes.txt"
: > "$RESULTS"
: > "$NOTES"
SRV=""

cleanup() {
    if [ -n "$SRV" ]; then
        kill "$SRV" 2>/dev/null
        wait "$SRV" 2>/dev/null
    fi
    if [ -n "${WORK:-}" ] && [ -d "$WORK" ]; then
        rm -rf "$WORK"
    fi
}
trap cleanup EXIT INT TERM

say()  { printf '  %s\n' "$*"; }
note() { printf '%s\n' "$*" >> "$NOTES"; }

# ── receipt. Written on EVERY exit path, including the fatal ones, because a
# gate that reads the receipt must be able to tell "the harness failed early"
# from "the harness never ran".
receipt() {
    STDIO_LIST="$WORK/stdio.tools" HTTP_LIST="$WORK/http.tools" \
    RESULTS="$RESULTS" NOTES="$NOTES" OUT="$OUT" python3 - <<'PY'
import json, os

def lines(p):
    try:
        with open(p) as f:
            return [x.strip() for x in f if x.strip()]
    except OSError:
        return []

stdio = lines(os.environ["STDIO_LIST"])
http  = lines(os.environ["HTTP_LIST"])
inv = []
for row in lines(os.environ["RESULTS"]):
    parts = row.split("\t")
    while len(parts) < 4:
        parts.append("")
    inv.append({"transport": parts[0], "tool": parts[1],
                "verdict": parts[2], "detail": parts[3]})
doc = {
    "stdio_tools": sorted(stdio),
    "http_tools": sorted(http),
    "stdio_count": len(stdio),
    "http_count": len(http),
    "only_stdio": sorted(set(stdio) - set(http)),
    "only_http": sorted(set(http) - set(stdio)),
    "invocations": inv,
    "failures": [i for i in inv if i["verdict"] != "OK"],
    "notes": lines(os.environ["NOTES"]),
}
with open(os.environ["OUT"], "w") as f:
    json.dump(doc, f, indent=1)
PY
}

fatal() {
    note "$*"
    say "✗ $*"
    receipt
    exit 1
}

# ── classify one JSON-RPC reply.
# Handles JSONL (stdio emits one object per line) and SSE-framed bodies
# ("data: {...}"), because the streamable HTTP transport is entitled to either.
classify() { # classify <reply-file> <rc>
    REPLY="$1" RC="$2" python3 - <<'PY'
import json, os, sys

rc = os.environ["RC"]
if rc == "124" or rc == "28":          # timeout(1) / curl --max-time
    print("HANG\tno reply before the per-call timeout expired (rc=%s)" % rc)
    sys.exit(0)
try:
    with open(os.environ["REPLY"]) as f:
        raw = f.read()
except OSError as e:
    print("NOREPLY\tcould not read the reply file: %s" % e)
    sys.exit(0)

reply = None
for line in raw.splitlines():
    line = line.strip()
    if line.startswith("data:"):
        line = line[5:].strip()
    if not line:
        continue
    try:
        d = json.loads(line)
    except Exception:
        continue
    if isinstance(d, dict) and d.get("id") == 2:
        reply = d
        break
if reply is None:
    try:
        d = json.loads(raw)
        if isinstance(d, dict):
            reply = d
    except Exception:
        pass
if reply is None:
    print("NOREPLY\tno JSON-RPC reply with id=2 in %d byte(s) of output" % len(raw))
    sys.exit(0)

if "error" in reply:
    e = reply["error"] or {}
    msg = str(e.get("message", "")).replace("\n", " ")[:300]
    print("ERROR\tcode=%s %s" % (e.get("code"), msg))
    sys.exit(0)

res = reply.get("result") or {}
text = "".join(c.get("text", "") for c in (res.get("content") or [])
               if c.get("type") == "text")
if res.get("isError"):
    print("ISERROR\tresult.isError=true :: %s" % text.replace("\n", " ")[:300])
elif not text.strip():
    # An advertised tool that answers with nothing is not working. This is the
    # shape a stub returns, and it is indistinguishable from success unless
    # something asserts on the payload.
    print("EMPTY\tresult carried no text content")
else:
    print("OK\t%d bytes" % len(text))
PY
}

TAB=$'\t'   # every tab in this file is spelled, never typed: a literal tab in
            # source is invisible in review and an editor that expands it silently
            # breaks the receipt's field split.
record() { printf '%s\t%s\t%s\n' "$1" "$2" "$3" >> "$RESULTS"; }

# ── fixture: a real git repo, small enough that every tool answers in under a
# second. Built with git PLUMBING (hash-object / mktree / commit-tree /
# update-ref) rather than `git add` + `git commit`, for a measured reason: this
# machine has a global core hooksPath, so `git commit` in a throwaway fixture
# fires the operator's own pre-commit quality gates. Plumbing runs no hooks, and
# with a pinned author date the commit sha is reproducible.
build_fixture() {
    mkdir -p "$FIX/src" || fatal "could not create the fixture directory"
    cat > "$FIX/Cargo.toml" <<'EOF'
[package]
name = "surface-fixture"
version = "0.1.0"
edition = "2021"
EOF
    # The contents are chosen so each analyzer has something real to find:
    # a branch (complexity), a TODO (satd), a call edge (dag), a #[test]
    # (vacuous-tests, which errors outright on a tree with none).
    cat > "$FIX/src/main.rs" <<'EOF'
// TODO: this SATD marker exists so satd analysis has a real finding
fn helper(n: u32) -> u32 {
    if n > 10 { n * 2 } else { n + 1 }
}

fn main() {
    println!("{}", helper(3));
}

#[cfg(test)]
mod tests {
    use super::helper;

    #[test]
    fn helper_doubles_above_ten() {
        assert_eq!(helper(11), 22);
    }
}
EOF
    local log="$WORK/git.log" b_toml b_main t_src t_root commit
    git -C "$FIX" init -q > "$log" 2>&1 || fatal "git init failed in the fixture: $(tr '\n' ';' < "$log")"
    b_toml=$(git -C "$FIX" hash-object -w Cargo.toml 2>>"$log")
    b_main=$(git -C "$FIX" hash-object -w src/main.rs 2>>"$log")
    t_src=$(printf '100644 blob %s\tmain.rs\n' "$b_main" | git -C "$FIX" mktree 2>>"$log")
    t_root=$(printf '100644 blob %s\tCargo.toml\n040000 tree %s\tsrc\n' "$b_toml" "$t_src" \
             | git -C "$FIX" mktree 2>>"$log")
    commit=$(GIT_AUTHOR_DATE='2020-01-01T00:00:00Z' GIT_COMMITTER_DATE='2020-01-01T00:00:00Z' \
             git -C "$FIX" -c user.email=fixture@example.invalid -c user.name=fixture \
                 commit-tree "$t_root" -m fixture 2>>"$log")
    [ -n "$commit" ] || fatal "could not build the fixture commit: $(tr '\n' ';' < "$log")"
    git -C "$FIX" update-ref HEAD "$commit" 2>>"$log" \
        || fatal "could not point HEAD at the fixture commit: $(tr '\n' ';' < "$log")"
    # Populate the INDEX from the tree. `commit-tree` writes history and leaves
    # the index empty, and several tools enumerate their input with
    # `git ls-files`, which reads the index and NOT HEAD. Without this the
    # fixture has a commit, a clean `git log`, and zero tracked files — and
    # analyze_hardcoded_paths / analyze_vacuous_tests correctly refuse to
    # report a clean result over nothing. That refusal is the right behaviour;
    # the bug was in the fixture, and this is the line that fixes it.
    git -C "$FIX" read-tree HEAD 2>>"$log" \
        || fatal "could not read the fixture tree into the index: $(tr '\n' ';' < "$log")"
    [ -n "$(git -C "$FIX" ls-files)" ] \
        || fatal "the fixture index is empty — \`git ls-files\` lists nothing, so every tool that enumerates tracked files would be measuring an empty set"
    # git_operation runs `git rev-parse HEAD` and fails on a repo with no
    # commits, so this check is the difference between testing the tool and
    # testing the fixture.
    git -C "$FIX" rev-parse HEAD >/dev/null 2>>"$log" \
        || fatal "the fixture repo has no HEAD: $(tr '\n' ';' < "$log")"
}

# ── the argument recipe for each tool, taken from the tool's own inputSchema.
# A tool with NO recipe is a FAILURE, never a skip: "advertised, and the gate
# does not know how to call it" is precisely the hole this gate was built to
# close, and a skip would let a new tool arrive permanently untested.
args_for() {
    case "$1" in
    analyze_big_o|analyze_complexity|analyze_dag|analyze_dead_code|\
    analyze_deep_context|analyze_satd|generate_context|quality_gate|scaffold_project)
        printf '{"paths":[%s]}' "$PJ" ;;
    analyze_hardcoded_paths|analyze_reachability|analyze_vacuous_tests)
        printf '{"project_path":%s}' "$PJ" ;;
    git_operation)
        printf '{"path":%s}' "$PJ" ;;
    pdmt_deterministic_todos)
        printf '{"requirements":["add a health endpoint"],"project_name":"surface-fixture"}' ;;
    pmat_index_stats)
        printf '{}' ;;
    pmat_query_code)
        printf '{"query":"helper","limit":3}' ;;
    pmat_get_function)
        printf '{"function_id":"src/main.rs::helper"}' ;;
    pmat_find_similar)
        printf '{"function_id":"src/main.rs::helper","limit":3}' ;;
    quality_proxy)
        printf '{"operation":"write","file_path":%s,"content":"pub fn ok() -> u32 { 1 }\\n","mode":"advisory"}' "$PJ_PROBE" ;;
    *)
        return 1 ;;
    esac
}

INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"mcp-surface","version":"1"}}}'

# ── stdio. One process per call, so a tool that hangs is attributed to that
# tool instead of killing the whole session and being reported as nineteen
# failures. No pipeline: `$?` after a pipe is the LAST command's status, which
# is how a dead server reads as a clean parse.
stdio_rpc() { # stdio_rpc <slug> <json-line>
    local slug="$1" body="$2" rc
    printf '%s\n%s\n' "$INIT" "$body" > "$WORK/req-$slug.jsonl"
    ( cd "$FIX" && exec timeout "$TIMEOUT" env MCP_VERSION=1 "$STDIO_BIN" ) \
        < "$WORK/req-$slug.jsonl" > "$WORK/stdio-$slug.json" 2> "$WORK/stdio-$slug.err"
    rc=$?
    printf '%s' "$rc" > "$WORK/stdio-$slug.rc"
    return "$rc"
}

# ── HTTP. Never `curl -f`: it turns every 4xx into an empty string with no
# message, which is how "the server rejected the missing Accept header" was
# read for an hour as "the server did not answer". Assert on the code instead.
http_rpc() { # http_rpc <slug> <json-body>
    local slug="$1" body="$2" code rc
    code=$(curl -sS -m "$TIMEOUT" -o "$WORK/http-$slug.json" -w '%{http_code}' -X POST \
        -H "Authorization: Bearer $TOKEN" \
        -H 'Content-Type: application/json' \
        -H 'Accept: application/json, text/event-stream' \
        --data-binary "$body" \
        "http://127.0.0.1:$PORT$MCP_PATH" 2> "$WORK/http-$slug.err")
    rc=$?
    printf '%s' "$rc" > "$WORK/http-$slug.rc"
    printf '%s' "$code" > "$WORK/http-$slug.code"
    return "$rc"
}

# ── run ────────────────────────────────────────────────────────────────────
say "── MCP surface ───────────────────────────────────────"
build_fixture
PJ=$(python3 -c 'import json,sys; print(json.dumps(sys.argv[1]))' "$FIX")
PJ_PROBE=$(python3 -c 'import json,sys; print(json.dumps(sys.argv[1] + "/probe_written.rs"))' "$FIX")
say "fixture: $FIX @ $(git -C "$FIX" rev-parse --short HEAD)"

# ── leg 1: stdio tools/list
stdio_rpc list '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
STDIO_RC=$?
python3 - "$WORK/stdio-list.json" > "$WORK/stdio.tools" <<'PY'
import json, sys
for line in open(sys.argv[1]):
    line = line.strip()
    if not line:
        continue
    try:
        d = json.loads(line)
    except Exception:
        continue
    if d.get("id") == 2:
        for t in (d.get("result") or {}).get("tools", []) or []:
            print(t["name"])
        break
PY
STDIO_N=$(wc -l < "$WORK/stdio.tools" | tr -d ' ')
if [ "$STDIO_N" -eq 0 ]; then
    # Quote the process's own words. "the server did not answer" cannot
    # distinguish "refused to start" from "wrong flag" from "tool errored",
    # and the reason is nearly always on line 1 of the stderr we just kept.
    why=$(head -3 "$WORK/stdio-list.err" 2>/dev/null | tr '\n' ';' | sed 's/;*$//')
    fatal "MCP stdio answered tools/list with ZERO tools (rc=$STDIO_RC)${why:+ — the process said: $why}"
fi
say "stdio: $STDIO_N tool(s)"

# ── leg 2: HTTP tools/list at the root
# >= 16 characters or the server REFUSES TO START ("must be at least 16
# characters; got 13"). The parity harness used `parity-$$`, which is 13, so its
# HTTP leg died at startup on every run it ever made and was recorded as the
# indistinguishable "server did not answer". Assert the length here rather than
# trusting the expression.
TOKEN="pmat-mcp-surface-$(od -An -N8 -tx1 /dev/urandom | tr -d ' \n')"
[ "${#TOKEN}" -ge 16 ] || fatal "generated token is ${#TOKEN} chars; the server requires >= 16 and would refuse to start"

SRV_LOG="$WORK/server.log"
( cd "$FIX" && exec env PMAT_MCP_HTTP_TOKEN="$TOKEN" "$HTTP_BIN" serve --transport http --port "$PORT" ) \
    > "$SRV_LOG" 2>&1 &
SRV=$!

# Readiness by SPEAKING MCP. Not by polling /health — there is no /health, it
# 404s, so a readiness loop keyed on it can never break and simply burns its
# iterations before posting to a server nobody confirmed was up.
READY=""
for _ in $(seq 1 60); do
    if ! kill -0 "$SRV" 2>/dev/null; then break; fi
    code=$(curl -sS -m 2 -o "$WORK/http-init.json" -w '%{http_code}' -X POST \
        -H "Authorization: Bearer $TOKEN" \
        -H 'Content-Type: application/json' \
        -H 'Accept: application/json, text/event-stream' \
        --data-binary "$INIT" \
        "http://127.0.0.1:$PORT$MCP_PATH" 2> "$WORK/http-init.err")
    if [ "$code" = "200" ]; then READY=yes; break; fi
    sleep 0.25
done
if [ -z "$READY" ]; then
    why=$(head -5 "$SRV_LOG" 2>/dev/null | tr '\n' ';' | sed 's/;*$//')
    curlwhy=$(head -2 "$WORK/http-init.err" 2>/dev/null | tr '\n' ';' | sed 's/;*$//')
    fatal "MCP over HTTP never answered initialize at http://127.0.0.1:$PORT$MCP_PATH (last code=${code:-none})${why:+ — server said: $why}${curlwhy:+ — curl said: $curlwhy}"
fi

# Auth is MANDATORY on this surface: below 16 chars the server refuses to start
# rather than serve unauthenticated, because pmcp serves every request when no
# auth provider is wired. Assert the 401 so a future build cannot quietly drop
# the check and still pass every other line of this gate.
NOAUTH=$(curl -sS -m 5 -o /dev/null -w '%{http_code}' -X POST \
    -H 'Content-Type: application/json' \
    -H 'Accept: application/json, text/event-stream' \
    --data-binary "$INIT" "http://127.0.0.1:$PORT$MCP_PATH" 2> "$WORK/http-noauth.err")
if [ "$NOAUTH" != "401" ]; then
    note "unauthenticated POST $MCP_PATH returned $NOAUTH, expected 401 — the HTTP surface is not requiring its bearer token"
    say "✗ unauthenticated POST $MCP_PATH -> $NOAUTH (expected 401)"
    AUTH_OK=""
else
    AUTH_OK=yes
    say "auth: unauthenticated POST $MCP_PATH -> 401"
fi

http_rpc list '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
HTTP_RC=$?
HTTP_CODE=$(cat "$WORK/http-list.code" 2>/dev/null)
python3 - "$WORK/http-list.json" > "$WORK/http.tools" <<'PY'
import json, sys
raw = open(sys.argv[1]).read()
d = None
for line in raw.splitlines():
    line = line.strip()
    if line.startswith("data:"):
        line = line[5:].strip()
    if not line:
        continue
    try:
        obj = json.loads(line)
    except Exception:
        continue
    if isinstance(obj, dict) and obj.get("id") == 2:
        d = obj
        break
for t in ((d or {}).get("result") or {}).get("tools", []) or []:
    print(t["name"])
PY
HTTP_N=$(wc -l < "$WORK/http.tools" | tr -d ' ')
if [ "$HTTP_N" -eq 0 ]; then
    why=$(head -5 "$SRV_LOG" 2>/dev/null | tr '\n' ';' | sed 's/;*$//')
    fatal "MCP over HTTP answered tools/list with ZERO tools (http=$HTTP_CODE curl_rc=$HTTP_RC)${why:+ — server said: $why}"
fi
say "http:  $HTTP_N tool(s) at $MCP_PATH (http=$HTTP_CODE)"

# ── the two sets must be IDENTICAL. Sorted sets, because HTTP returns the
# tools in HashMap order and a raw-order comparison would flap forever.
ONLY_STDIO=$(comm -23 <(sort -u "$WORK/stdio.tools") <(sort -u "$WORK/http.tools") | tr '\n' ' ' | sed 's/ *$//')
ONLY_HTTP=$(comm -13 <(sort -u "$WORK/stdio.tools") <(sort -u "$WORK/http.tools") | tr '\n' ' ' | sed 's/ *$//')
SETS_OK=yes
if [ -n "$ONLY_STDIO" ]; then
    note "served over stdio but NOT over HTTP: $ONLY_STDIO"
    say "✗ stdio-only tools: $ONLY_STDIO"
    SETS_OK=""
fi
if [ -n "$ONLY_HTTP" ]; then
    note "served over HTTP but NOT over stdio: $ONLY_HTTP"
    say "✗ http-only tools: $ONLY_HTTP"
    SETS_OK=""
fi

# ── every advertised tool, on both transports.
FAILED=0
while read -r tool; do
    [ -n "$tool" ] || continue
    if ! ARGS=$(args_for "$tool"); then
        record stdio "$tool" "NO_RECIPE${TAB}no argument recipe in args_for() — a newly advertised tool this gate cannot call is untested, which is the state this gate exists to refuse. Add its recipe from its inputSchema."
        record http  "$tool" "NO_RECIPE${TAB}no argument recipe in args_for()"
        say "✗ $tool: NO RECIPE — add one to args_for() from its inputSchema"
        FAILED=$((FAILED + 1))
        continue
    fi
    body=$(printf '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"%s","arguments":%s}}' "$tool" "$ARGS")

    stdio_rpc "$tool" "$body"
    src=$?
    sverdict=$(classify "$WORK/stdio-$tool.json" "$src")
    case "$sverdict" in
        OK*) ;;
        *)   serr=$(head -2 "$WORK/stdio-$tool.err" 2>/dev/null | tr '\n' ';' | sed 's/;*$//')
             sverdict="$sverdict${serr:+ | stderr: $serr}"
             FAILED=$((FAILED + 1)) ;;
    esac
    printf 'stdio\t%s\t%s\n' "$tool" "$sverdict" >> "$RESULTS"

    http_rpc "$tool" "$body"
    hrc=$?
    hcode=$(cat "$WORK/http-$tool.code" 2>/dev/null)
    hverdict=$(classify "$WORK/http-$tool.json" "$hrc")
    if [ "$hcode" != "200" ]; then
        hverdict="HTTP_$hcode${TAB}transport returned $hcode, not 200"
    fi
    case "$hverdict" in
        OK*) ;;
        *)   herr=$(head -2 "$WORK/http-$tool.err" 2>/dev/null | tr '\n' ';' | sed 's/;*$//')
             hverdict="$hverdict${herr:+ | curl: $herr}"
             FAILED=$((FAILED + 1)) ;;
    esac
    printf 'http\t%s\t%s\n' "$tool" "$hverdict" >> "$RESULTS"

    printf '  %-26s stdio=%-9s http=%s\n' "$tool" "${sverdict%%"$TAB"*}" "${hverdict%%"$TAB"*}"
done < <(sort -u "$WORK/stdio.tools" "$WORK/http.tools")

receipt

say "surface: stdio=$STDIO_N http=$HTTP_N  invocation failures=$FAILED"
if [ "$FAILED" -gt 0 ]; then
    grep -v "${TAB}OK${TAB}" "$RESULTS" | while IFS="$TAB" read -r tr_ tool verdict detail; do
        say "  ✗ $tr_ $tool: $verdict ${detail:-}"
    done
fi

[ -n "$SETS_OK" ] && [ -n "$AUTH_OK" ] && [ "$FAILED" -eq 0 ]
