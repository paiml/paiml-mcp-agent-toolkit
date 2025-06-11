#!/bin/bash
# Kaizen Overnight Runner - Main entry point for overnight optimization

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║           KAIZEN OVERNIGHT OPTIMIZATION SYSTEM                     ║${NC}"
echo -e "${CYAN}║                                                                    ║${NC}"
echo -e "${CYAN}║  This system will run overnight to optimize your codebase         ║${NC}"
echo -e "${CYAN}║  Target: 50% compilation time improvement                         ║${NC}"
echo -e "${CYAN}║  Method: Automated AST transformations and profiling              ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════════════╝${NC}"
echo

# Configuration
WORK_DIR="/home/noah/src/paiml-mcp-agent-toolkit"
SERVER_DIR="$WORK_DIR/server"
LOG_FILE="$WORK_DIR/kaizen_overnight_$(date +%Y%m%d_%H%M%S).log"

# Ensure we're in the right directory
cd "$WORK_DIR"

# Log function
log() {
    echo "[$(date +%H:%M:%S)] $1" | tee -a "$LOG_FILE"
}

# Pre-flight checks
log "Running pre-flight checks..."

# 1. Check git status
if [[ -n $(git status --porcelain) ]]; then
    log "WARNING: Uncommitted changes detected. Creating backup branch..."
    git stash push -m "Kaizen optimization backup $(date +%Y%m%d_%H%M%S)"
fi

# 2. Create optimization branch
BRANCH_NAME="perf/kaizen-overnight-$(date +%Y%m%d-%H%M)"
git checkout -b "$BRANCH_NAME" 2>/dev/null || {
    log "Branch already exists, using existing branch"
    git checkout "$BRANCH_NAME"
}

# 3. Measure baseline
log "Measuring baseline performance..."
cd "$SERVER_DIR"
cargo clean >/dev/null 2>&1
BASELINE_START=$(date +%s)
cargo build --release 2>&1 | tail -5
BASELINE_END=$(date +%s)
BASELINE_TIME=$((BASELINE_END - BASELINE_START))
log "Baseline build time: ${BASELINE_TIME}s"

# 4. Start monitoring in background
log "Starting monitoring dashboard..."
"$WORK_DIR/kaizen-dashboard.sh" &
MONITOR_PID=$!

# 5. Run continuous optimization
log "Starting continuous optimization..."
log "This will run for up to 8 hours or until 50% improvement is achieved"
log "Check $LOG_FILE for detailed progress"

# Run the optimization with proper error handling
set +e
"$WORK_DIR/kaizen-continuous-monitor.sh" 2>&1 | tee -a "$LOG_FILE"
OPTIMIZATION_EXIT_CODE=$?
set -e

# 6. Generate final report
log "Generating final report..."

# Measure final performance
cargo clean >/dev/null 2>&1
FINAL_START=$(date +%s)
cargo build --release 2>&1 | tail -5
FINAL_END=$(date +%s)
FINAL_TIME=$((FINAL_END - FINAL_START))

IMPROVEMENT=$(echo "scale=2; (($BASELINE_TIME - $FINAL_TIME) * 100) / $BASELINE_TIME" | bc)

# Create summary
cat > "$WORK_DIR/KAIZEN_OVERNIGHT_SUMMARY.md" << EOF
# Kaizen Overnight Optimization Summary

Date: $(date)
Branch: $BRANCH_NAME

## Results
- Baseline Build Time: ${BASELINE_TIME}s
- Final Build Time: ${FINAL_TIME}s
- **Total Improvement: ${IMPROVEMENT}%**

## Changes Applied
$(git log --oneline "$BRANCH_NAME" --not main | head -10)

## Files Modified
$(git diff --stat main)

## Next Steps
1. Review changes: \`git diff main\`
2. Run tests: \`cd server && cargo test --all-features\`
3. If tests pass, merge: \`git checkout main && git merge $BRANCH_NAME\`

## Log File
Full details available in: $LOG_FILE
EOF

# 7. Cleanup
kill $MONITOR_PID 2>/dev/null || true

# 8. Final message
echo
echo -e "${GREEN}=== OPTIMIZATION COMPLETE ===${NC}"
echo "Total Improvement: ${IMPROVEMENT}%"
echo "Summary saved to: KAIZEN_OVERNIGHT_SUMMARY.md"
echo
echo "To review changes:"
echo "  git diff main"
echo
echo "To merge optimizations:"
echo "  git checkout main"
echo "  git merge $BRANCH_NAME"

# Return success if we achieved any improvement
if (( $(echo "$IMPROVEMENT > 0" | bc -l) )); then
    exit 0
else
    exit 1
fi