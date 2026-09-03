#!/usr/bin/env bash
# CRUX-10 — quality_check_content (formerly quality_proxy) audit (spec section 8.10, issue 1151).
# S selects the live branch (rename vs write) from tools/list; B0-B2 are branch-independent;
# the WRITE branch runs only if `operation` still carries "write"; the RENAME branch otherwise.
# Every leg inspects the filesystem after the call and reads only schema-declared fields.
# PMAT=<binary> overrides the pmat used.
set -euo pipefail
fail(){ echo "FAIL: $*"; exit 1; }
PMAT=${PMAT:-$(command -v pmat)}

mcp(){ { printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"crux10","version":"0"}}}'
        printf '%s\n' '{"jsonrpc":"2.0","method":"notifications/initialized"}'
        jq -cn --arg n "$1" --argjson a "$2" '{jsonrpc:"2.0",id:2,method:"tools/call",params:{name:$n,arguments:$a}}'
      } | "${MCP_RUNNER[@]}" --mode mcp 2>/dev/null | grep '^{' \
        | jq -c 'select(.id==2)|if .error then {error:.error} else (.result.content[0].text|fromjson) end'; }
# Run the server from another directory WITHOUT `cd`: `env -C` sets the child's cwd
# (GNU coreutils), which is what W5's scoping probe needs. Default: no cwd change.
MCP_RUNNER=("$PMAT")
in_dir(){ MCP_RUNNER=(env -C "$1" "$PMAT"); }
no_dir(){ MCP_RUNNER=("$PMAT"); }
mcplist(){ { printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"crux10","version":"0"}}}'
             printf '%s\n' '{"jsonrpc":"2.0","method":"notifications/initialized"}'
             printf '%s\n' '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
           } | "${MCP_RUNNER[@]}" --mode mcp 2>/dev/null | grep '^{' | jq -c 'select(.id==2)|.result'; }

T=$(mktemp -d); OUT=$(mktemp -d); ROOT=$(mktemp -d); mkdir -p "$ROOT/src"
cleanup(){ for d in "${T:-}" "${OUT:-}" "${ROOT:-}"; do case "$d" in "${TMPDIR:-/tmp}"/tmp.*|/tmp/tmp.*) [ -d "$d" ] && rm -rf -- "$d";; esac; done; }; trap cleanup EXIT
printf '%b' '/// d\npub fn ok(){}\n'     > "$T/.good"     # the client's bytes, and the only
printf '%b' '// TODO: x\npub fn f(){}\n' > "$T/.bad"      # constants this test pins
OK_SHA=$(sha256sum "$T/.good"|cut -c1-64); OK_LEN=$(stat -c%s "$T/.good")
EMPTY_SHA=$(printf '' | sha256sum | cut -c1-64)
# The request shape follows the branch (set after S): the write branch still
# carries `operation:"write"`; the rename branch has no `operation` at all, and
# sending one is exactly what R1 proves is refused. req_stale() is that probe.
req(){ if [ "$WRITE_IN_ENUM" = true ]; then req_stale "$@"; else jq -cn --arg p "$1" --rawfile c "$2" --arg m "$3" '{file_path:$p,content:$c,mode:$m}'; fi; }
req_stale(){ jq -cn --arg p "$1" --rawfile c "$2" --arg m "$3" '{operation:"write",file_path:$p,content:$c,mode:$m}'; }
sha(){ sha256sum "$1" 2>/dev/null | cut -c1-64 || true; }
exists(){ test -e "$1" && echo true || echo false; }

# ---- S (permanent): branch selector. Exactly one branch is live; deleting the tool fails HERE.
LIST=$(mcplist)
# Prefer the live name; on the rename branch the alias is covered by R2, not by B0-B2.
NAME=$(printf '%s' "$LIST" | jq -r '[.tools[].name]|if index("quality_check_content") then "quality_check_content" elif index("quality_proxy") then "quality_proxy" else "" end')
[ -n "$NAME" ] || fail "S: neither quality_proxy nor quality_check_content is served"
printf '%s' "$LIST" | jq -e --arg n "$NAME" '[.tools[]|select(.name==$n)]|length==1' >/dev/null || fail "S: tool not uniquely served"
WRITE_IN_ENUM=$(printf '%s' "$LIST" | jq -r --arg n "$NAME" '[.tools[]|select(.name==$n)][0].inputSchema.properties.operation.enum // [] | index("write") != null')
echo "S: tool=$NAME write_in_enum=$WRITE_IN_ENUM client_sha=$OK_SHA len=$OK_LEN"

# ---- B0 (permanent, branch-independent): the response must DISCLOSE what it did.
r=$(mcp "$NAME" "$(req "$T/B0.rs" "$T/.good" strict)")
printf '%s' "$r" | jq -e 'has("written") and (.written|type=="boolean")' >/dev/null || fail "B0: no boolean 'written' in the response"
printf '%s' "$r" | jq -e --argjson o "$(exists "$T/B0.rs")" '.written == $o' >/dev/null || fail "B0: 'written' contradicts the filesystem"

# ---- B1 (permanent, branch-independent): advisory must not launder a failing verdict.
r=$(mcp "$NAME" "$(req "$T/ADV.rs" "$T/.bad" advisory)")
printf '%s' "$r" | jq -e '.quality_report.passed == false' >/dev/null || fail "B1 setup: bad fixture graded as passing"
printf '%s' "$r" | jq -e '.status != "accepted"' >/dev/null || fail "B1: advisory returned accepted while passed==false"

# ---- B2 (permanent, branch-independent): client quality_config may only tighten; count==list.
r=$(mcp "$NAME" "$(req "$T/CFG.rs" "$T/.bad" strict | jq -c '. + {quality_config:{max_complexity:9999,allow_satd:true,require_docs:false}}')")
printf '%s' "$r" | jq -e '.quality_report.passed == false' >/dev/null || fail "B2: client config loosened the project gate"
printf '%s' "$r" | jq -e '([.quality_report.violations[]|select(.type=="satd")]|length) == .quality_report.metrics.satd_count' >/dev/null || fail "B2: satd_count contradicts violations[]"

if [ "$WRITE_IN_ENUM" = true ]; then
# ===================== WRITE BRANCH — one session, conjunctive =====================
# W1 (permanent): the accepted write lands, as the CLIENT's bytes.
r=$(mcp "$NAME" "$(req "$T/OK.rs" "$T/.good" strict)")
printf '%s' "$r" | jq -e '.status == "accepted" and .written == true' >/dev/null || fail "W1: not accepted, or no written:true"
test -f "$T/OK.rs"                        || fail "W1: accepted write created no file"
[ "$(sha "$T/OK.rs")" = "$OK_SHA" ]       || fail "W1: file bytes are not the client's bytes"
[ "$(stat -c%s "$T/OK.rs")" = "$OK_LEN" ] || fail "W1: wrong length"
[ "$(sha "$T/OK.rs")" != "$EMPTY_SHA" ]   || fail "W1: empty-file game — file sha == sha256(\"\")"
printf '%s' "$r" | jq -j '.final_content' | cmp -s - "$T/.good" || fail "W1: final_content diverges from the client's bytes"
# W2 (conjunctive; vacuous alone): refusal writes nothing, then the SAME path must accept.
r=$(mcp "$NAME" "$(req "$T/BAD.rs" "$T/.bad" strict)")
printf '%s' "$r" | jq -e '.status == "rejected" and .written == false' >/dev/null || fail "W2: refusal not reported as rejected+written:false"
test ! -e "$T/BAD.rs" || fail "W2: rejected write created a file"
r=$(mcp "$NAME" "$(req "$T/BAD.rs" "$T/.good" strict)")
[ "$(sha "$T/BAD.rs")" = "$OK_SHA" ] || fail "W2 control: that path never accepts — the feature does not write at all"
# W3 (conjunctive): advisory refuses to write failing content, still writes passing content.
r=$(mcp "$NAME" "$(req "$T/ADV2.rs" "$T/.bad" advisory)")
printf '%s' "$r" | jq -e '.written == false' >/dev/null || fail "W3: advisory did not disclose written:false"
test ! -e "$T/ADV2.rs" || fail "W3: advisory wrote failing content"
r=$(mcp "$NAME" "$(req "$T/ADV2.rs" "$T/.good" advisory)")
[ "$(sha "$T/ADV2.rs")" = "$OK_SHA" ] || fail "W3 control: advisory never writes"
# W4 (conjunctive): no-clobber on refusal, real clobber on acceptance.
printf 'pre\n' > "$T/EXIST.rs"; before=$(sha "$T/EXIST.rs")
r=$(mcp "$NAME" "$(req "$T/EXIST.rs" "$T/.bad" strict)")
printf '%s' "$r" | jq -e '.written == false' >/dev/null || fail "W4: did not disclose written:false on refusal"
[ "$(sha "$T/EXIST.rs")" = "$before" ] || fail "W4: rejected write clobbered an existing file"
r=$(mcp "$NAME" "$(req "$T/EXIST.rs" "$T/.good" strict)")
[ "$(sha "$T/EXIST.rs")" = "$OK_SHA" ] || fail "W4 control: the accepted overwrite never happened"
# W5 (conjunctive): the write is scoped to the project root.
for p in "$OUT/ESCAPE.rs" "$ROOT/src/../../ESCAPE2.rs"; do
  in_dir "$ROOT"; r=$(mcp "$NAME" "$(req "$p" "$T/.good" strict)"); no_dir
  printf '%s' "$r" | jq -e '.written == false' >/dev/null || fail "W5: no written:false for an out-of-root path ($p)"
  test ! -e "$p" || fail "W5: created $p outside the project root"
done
else
# ============ RENAME BRANCH — live since PMAT-640; the selector chose W before it ============
r=$(mcp "$NAME" "$(req_stale "$T/R1.rs" "$T/.good" strict)")
printf '%s' "$r" | jq -e '.error.code == -32602' >/dev/null || fail "R1: operation=write still reaches a handler"
test ! -e "$T/R1.rs" || fail "R1: a refused operation created a file"
printf '%s' "$LIST" | jq -e '[.tools[].name]|(index("quality_proxy") != null) and (index("quality_check_content") != null)' >/dev/null || fail "R2: the one-release deprecated alias is not served"
a=$(mcp quality_proxy "$(req "$T/R2.rs" "$T/.good" strict)"); b=$(mcp quality_check_content "$(req "$T/R2.rs" "$T/.good" strict)")
[ "$a" = "$b" ] || fail "R2: alias and new name return different payloads"
printf '%s' "$a" | jq -e 'has("written") and .written == false' >/dev/null || fail "R3: no written:false disclosure"
printf '%s' "$LIST" | jq -e --arg n "$NAME" '[.tools[]|select(.name==$n)][0].inputSchema.properties.operation.enum // [] | map(select(["write","edit","append"]|index(.))) | length == 0' >/dev/null || fail "R5: a mutation value survives in the operation enum"
grep -qi 'writes files' docs/mcp/TOOLS.md && fail "R4: docs/mcp/TOOLS.md still asserts a write"
printf '%s' "$LIST" | jq -e --arg n "$NAME" '[.tools[]|select(.name==$n)][0].description|test("write|edit|append";"i")|not' >/dev/null || fail "R4: the live description still asserts a mutation"
fi
echo "PASS"
