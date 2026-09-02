#!/bin/bash
S=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/e1ae0d23-4b84-4273-be07-926cd668e2d3/scratchpad/research
cd /home/noah/src/paiml-mcp-agent-toolkit || exit 99
{ date -Is; uptime; } > "$S/build1.start"
/usr/bin/time -v -o "$S/build1.time" cargo build --release --message-format=json > "$S/build1.json" 2> "$S/build1.stderr"
{ echo "exit=$?"; date -Is; uptime; } > "$S/build1.exit"
{ date -Is; uptime; } > "$S/build2.start"
/usr/bin/time -v -o "$S/build2.time" cargo build --release --message-format=json > "$S/build2.json" 2> "$S/build2.stderr"
{ echo "exit=$?"; date -Is; uptime; } > "$S/build2.exit"
