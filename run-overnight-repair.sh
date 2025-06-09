#!/bin/bash
# AUTOMATED OVERNIGHT CODE REPAIR STATE MACHINE
# Executes 8-12 hour autonomous refactoring pipeline with state persistence
# Generates actionable checklist while applying fixes via RefactorStateMachine

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
PROJECT_DIR="."
CHECKPOINT_DIR=".refactor_state"
CONFIG_FILE="refactor-config.json"
LOG_FILE="refactor_overnight.log"
MAX_RUNTIME=43200  # 12 hours

# Check if pmat is available
if command -v pmat &> /dev/null; then
    PMAT_CMD="pmat"
else
    echo -e "${YELLOW}Building release binary...${NC}"
    cargo build --release
    PMAT_CMD="./target/release/pmat"
fi

# Create necessary directories
mkdir -p "$CHECKPOINT_DIR"
mkdir -p docs/bugs

echo -e "${BLUE}=== AUTOMATED OVERNIGHT CODE REPAIR STATE MACHINE ===${NC}"
echo "Configuration:"
echo "  Project: $PROJECT_DIR"
echo "  Config: $CONFIG_FILE"
echo "  Checkpoint: $CHECKPOINT_DIR"
echo "  Max Runtime: $MAX_RUNTIME seconds ($(($MAX_RUNTIME / 3600)) hours)"
echo ""

# Run pre-release analysis first
echo -e "${GREEN}Running pre-release analysis...${NC}"
./scripts/pre-release-analysis.sh || true

# Check for existing checkpoint
if [ -f "$CHECKPOINT_DIR/checkpoint.json" ]; then
    echo -e "${YELLOW}Found existing checkpoint. Resuming...${NC}"
    RESUME_FLAG="--resume"
else
    echo -e "${GREEN}Starting fresh refactoring session...${NC}"
    RESUME_FLAG=""
fi

# Start the refactor server
echo -e "${GREEN}Starting refactor server in batch mode...${NC}"
nohup $PMAT_CMD refactor serve \
  --refactor-mode batch \
  --config "$CONFIG_FILE" \
  --project "$PROJECT_DIR" \
  --parallel 8 \
  --memory-limit 16384 \
  --batch-size 50 \
  --priority "tdg_score DESC, complexity DESC, churn DESC" \
  --checkpoint-dir "$CHECKPOINT_DIR" \
  $RESUME_FLAG \
  --auto-commit "refactor: automated fix via state machine [skip ci]" \
  --max-runtime $MAX_RUNTIME \
  2>&1 | tee "$LOG_FILE" &

# Save PID for monitoring
REFACTOR_PID=$!
echo $REFACTOR_PID > "$CHECKPOINT_DIR/refactor.pid"

echo -e "${GREEN}Refactor server started with PID $REFACTOR_PID${NC}"
echo ""
echo "STATE MACHINE PHASES:"
echo "  1. ANALYZING: Collects all metrics (SATD, TDG, complexity, dead code)"
echo "  2. PLANNING: Prioritizes violations, generates fix sequence"
echo "  3. EXECUTING: Applies refactorings incrementally with AST transforms"
echo "  4. VALIDATING: Runs tests, checks coverage remains >80%"
echo "  5. REVERTING: Rolls back failed transformations"
echo ""
echo -e "${BLUE}Monitor progress:${NC}"
echo "  tail -f $LOG_FILE | grep -E 'STATE:|FIXED:|ERROR:'"
echo ""
echo -e "${BLUE}Check current state:${NC}"
echo "  cat $CHECKPOINT_DIR/current_state.json"
echo ""
echo -e "${BLUE}Interrupt safely:${NC}"
echo "  kill -SIGUSR1 $REFACTOR_PID"
echo ""
echo -e "${BLUE}View updated checklist:${NC}"
echo "  cat docs/bugs/prepare-release.md"
echo ""
echo -e "${GREEN}The refactor server is now running in the background.${NC}"
echo "It will automatically fix code quality issues for up to $(($MAX_RUNTIME / 3600)) hours."
echo "Check $LOG_FILE for detailed progress."