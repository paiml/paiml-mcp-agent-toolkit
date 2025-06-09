#!/bin/bash
# Start the overnight repair process

echo "🚀 Starting Overnight Code Repair State Machine"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📊 Current Code Quality Status:"
./target/release/pmat analyze complexity --project-path server/src/cli --max-cyclomatic 20 --format summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🔧 Starting refactor server with:"
echo "  - Max runtime: 12 hours (43200 seconds)"
echo "  - Parallel workers: 8"
echo "  - Memory limit: 16GB"
echo "  - Auto-commit enabled"
echo ""

# Start the refactor server
nohup ./target/release/pmat refactor serve \
  --refactor-mode batch \
  --config refactor-config.json \
  --project . \
  --parallel 8 \
  --memory-limit 16384 \
  --batch-size 50 \
  --checkpoint-dir .refactor_state \
  --resume \
  --auto-commit "refactor: automated fix via state machine [skip ci]" \
  --max-runtime 43200 \
  > refactor_overnight_full.log 2>&1 &

PID=$!
echo "✅ Refactor server started with PID: $PID"
echo ""
echo "📝 Commands to monitor progress:"
echo "  tail -f refactor_overnight_full.log"
echo "  ps -p $PID"
echo "  kill -SIGUSR1 $PID  # Safe interrupt"
echo ""
echo "📋 View reports:"
echo "  cat docs/bugs/prepare-release.md"
echo "  cat .refactor_state/checkpoint.json | jq '.current'"
echo ""
echo "The system will now run autonomously for up to 12 hours."
echo "Check refactor_overnight_full.log for detailed progress."