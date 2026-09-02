#!/bin/bash
S=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/e1ae0d23-4b84-4273-be07-926cd668e2d3/scratchpad/research
M=$S/m
BIN=/mnt/nvme-raid0/coverage/paiml-mcp-agent-toolkit/release/pmat
REPO=/home/noah/src/paiml-mcp-agent-toolkit
FIX=$S/fixture-small
run() { # name dir cmd...
  local name=$1; shift; local dir=$1; shift
  cd "$dir" || return
  { echo "cmd: $*"; echo "dir: $dir"; date -Is; uptime; } > "$M/$name.meta"
  timeout 600 /usr/bin/time -v -o "$M/$name.time" "$@" > "$M/$name.out" 2> "$M/$name.err"
  echo "exit=$?" >> "$M/$name.meta"
  echo "== $name done $(grep -E 'Elapsed' "$M/$name.time")" >> "$S/measure.progress"
}
: > "$S/measure.progress"
# --- startup-only ---
run version1 "$REPO" "$BIN" --version
run version2 "$REPO" "$BIN" --version
run help1 "$REPO" "$BIN" --help
# --- small fixture ---
run fix_complexity "$FIX" "$BIN" analyze complexity --format json
run fix_satd "$FIX" "$BIN" analyze satd --format json
run fix_deadcode "$FIX" "$BIN" analyze dead-code --format json
run fix_complexity2 "$FIX" "$BIN" analyze complexity --format json
# --- repo ---
run repo_query1 "$REPO" "$BIN" query "error handling" --limit 5
run repo_query2 "$REPO" "$BIN" query "error handling" --limit 5
run repo_complexity "$REPO" "$BIN" analyze complexity --format json
run repo_satd "$REPO" "$BIN" analyze satd --format json
run repo_deadcode "$REPO" "$BIN" analyze dead-code --format json
run repo_tdg "$REPO" "$BIN" tdg . --format json
run repo_context "$REPO" "$BIN" context --format llm-optimized --output "$S/ctx.md"
run repo_verify "$REPO" "$BIN" verify --skip clippy,tests --format json
run repo_qualitygate "$REPO" "$BIN" quality-gate --format json
run repo_score "$REPO" "$BIN" score
echo "ALL DONE $(date -Is)" >> "$S/measure.progress"
