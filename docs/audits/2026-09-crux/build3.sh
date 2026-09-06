#!/bin/bash
S=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/e1ae0d23-4b84-4273-be07-926cd668e2d3/scratchpad/research
cd "${REPO:-$HOME/src/paiml-mcp-agent-toolkit}" || exit 99
stat -c '%y %n' .git/index .git/HEAD > "$S/build3.gitstat.before"
{ date -Is; uptime; } > "$S/build3.start"
CARGO_LOG=cargo::core::compiler::fingerprint=info /usr/bin/time -v -o "$S/build3.time" cargo build --release --message-format=json > "$S/build3.json" 2> "$S/build3.stderr"
{ echo "exit=$?"; date -Is; uptime; } > "$S/build3.exit"
stat -c '%y %n' .git/index .git/HEAD > "$S/build3.gitstat.after"
