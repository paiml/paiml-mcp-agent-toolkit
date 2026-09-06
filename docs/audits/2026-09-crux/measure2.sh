#!/bin/bash
S=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/e1ae0d23-4b84-4273-be07-926cd668e2d3/scratchpad/research
M=$S/m
BIN="${PMAT_BIN:-$(cargo metadata --format-version 1 --no-deps | jq -r .target_directory)/release/pmat}"
REPO="${REPO:-$HOME/src/paiml-mcp-agent-toolkit}"
run() {
  local name=$1; shift; local dir=$1; shift
  cd "$dir" || return
  { echo "cmd: $*"; echo "dir: $dir"; date -Is; uptime; } > "$M/$name.meta"
  timeout 900 /usr/bin/time -v -o "$M/$name.time" "$@" > "$M/$name.out" 2> "$M/$name.err"
  echo "exit=$?" >> "$M/$name.meta"
  echo "== $name exit done $(grep -E 'Elapsed|Maximum resident' "$M/$name.time" | tr '\n' ' ')" >> "$S/measure.progress"
}
echo "MEASURE2 START $(date -Is)" >> "$S/measure.progress"
run repo_complexity   "$REPO" "$BIN" analyze complexity --format json
run repo_satd         "$REPO" "$BIN" analyze satd --format json
run repo_deadcode     "$REPO" "$BIN" analyze dead-code --format json
run repo_tdg          "$REPO" "$BIN" tdg . --format json
run repo_verify       "$REPO" "$BIN" verify --skip clippy,tests --format json
run repo_qualitygate  "$REPO" "$BIN" quality-gate --format json
run repo_context      "$REPO" "$BIN" context --format llm-optimized --output "$S/ctx.md"
run repo_dag          "$REPO" "$BIN" analyze dag --format json
run repo_duplicates   "$REPO" "$BIN" analyze duplicates --format json
run repo_score        "$REPO" "$BIN" score
echo "MEASURE2 ALL DONE $(date -Is)" >> "$S/measure.progress"
