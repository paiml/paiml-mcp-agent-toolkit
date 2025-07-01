#!/bin/bash
# AUTOMATED OVERNIGHT CODE REPAIR STATE MACHINE
# Executes 8-12 hour autonomous refactoring pipeline with state persistence
# Generates actionable checklist while applying fixes via RefactorStateMachine

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create necessary directories
mkdir -p .refactor_state
mkdir -p docs/bugs

# Function to log with timestamp
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Function to check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check if pmat binary exists
    if ! command -v pmat &> /dev/null; then
        log "${YELLOW}Warning: pmat not in PATH. Using cargo run instead.${NC}"
        PMAT_CMD="cargo run --release --"
    else
        PMAT_CMD="pmat"
    fi
    
    # Check if make commands work
    if ! make lint &> /dev/null; then
        log "${RED}Error: 'make lint' failed. Please fix linting issues first.${NC}"
        exit 1
    fi
    
    log "${GREEN}Prerequisites check passed.${NC}"
}

# Function to create refactor config
create_refactor_config() {
    cat > .refactor_state/config.json <<'EOF'
{
  "state_machine": {
    "initial_state": "analyzing",
    "checkpoint_interval": 300,
    "max_iterations": 10000,
    "transitions": [
      {"from": "analyzing", "to": "planning", "condition": "metrics_collected"},
      {"from": "planning", "to": "executing", "condition": "violations_prioritized"},
      {"from": "executing", "to": "validating", "condition": "refactor_applied"},
      {"from": "validating", "to": "analyzing", "condition": "tests_pass"},
      {"from": "validating", "to": "reverting", "condition": "tests_fail"},
      {"from": "reverting", "to": "planning", "condition": "rollback_complete"}
    ]
  },
  "metrics": {
    "complexity_cyclomatic": 10,
    "complexity_cognitive": 15,
    "tdg_threshold": 15.0,
    "satd_auto_fix": true,
    "dead_code_remove": true,
    "duplicate_threshold": 0.85
  },
  "operations": {
    "extract_function": {"min_lines": 10, "max_complexity": 8},
    "inline_variable": {"single_use": true},
    "remove_dead_code": {"confidence": 0.95},
    "fix_satd": {"patterns": ["TODO", "FIXME", "HACK", "XXX"]},
    "reduce_nesting": {"max_depth": 3},
    "split_module": {"max_lines": 500}
  },
  "validation": {
    "run_tests": "make test-fast",
    "check_coverage": "cargo llvm-cov report --json --output-path coverage.json",
    "lint_check": "make lint",
    "rollback_on_failure": true
  },
  "output": {
    "deep_context": "./deep_context_state.json",
    "checklist": "docs/bugs/prepare-release.md",
    "state_log": "refactor_state.log",
    "metrics_snapshot": "metrics_evolution.jsonl"
  }
}
EOF
}

# Function to run the refactor server
run_refactor_server() {
    log "Starting refactor server in batch mode..."
    
    # Create a wrapper script for the actual command
    cat > .refactor_state/run_command.sh <<EOF
#!/bin/bash
$PMAT_CMD refactor serve \\
  --mode batch \\
  --config .refactor_state/config.json \\
  --project . \\
  --parallel 8 \\
  --memory-limit 16384 \\
  --batch-size 50 \\
  --priority "tdg_score DESC, complexity DESC, churn DESC" \\
  --checkpoint-dir .refactor_state \\
  --resume \\
  --auto-commit "refactor: automated fix via state machine [skip ci]" \\
  --max-runtime 43200
EOF
    
    chmod +x .refactor_state/run_command.sh
    
    # Run with nohup
    nohup .refactor_state/run_command.sh 2>&1 | tee refactor_overnight.log &
    
    # Save PID for monitoring
    echo $! > .refactor_state/refactor.pid
    
    log "${GREEN}Refactor server started with PID $(cat .refactor_state/refactor.pid)${NC}"
}

# Function to monitor progress
monitor_progress() {
    log "To monitor progress:"
    echo "  tail -f refactor_overnight.log | grep -E 'STATE:|FIXED:|ERROR:'"
    echo ""
    log "To interrupt safely:"
    echo "  kill -SIGUSR1 \$(cat .refactor_state/refactor.pid)"
    echo ""
    log "To check state:"
    echo "  cat .refactor_state/current_state.json"
}

# Main execution
main() {
    log "=== AUTOMATED OVERNIGHT CODE REPAIR STATE MACHINE ==="
    log "This will execute an 8-12 hour autonomous refactoring pipeline"
    echo ""
    
    check_prerequisites
    create_refactor_config
    
    # Run the actual refactor serve command
    log "Starting automated refactoring pipeline..."
    
    # First, update the prepare-release checklist
    ./scripts/pre-release-analysis.sh || true
    
    # Then start the refactor server
    $PMAT_CMD refactor serve \
        --config .refactor_state/config.json \
        --project . \
        --parallel 8 \
        --memory-limit 16384 \
        --batch-size 50 \
        --priority "tdg_score DESC, complexity DESC, churn DESC" \
        --checkpoint-dir .refactor_state \
        --resume \
        --auto-commit "refactor: automated fix via state machine [skip ci]" \
        --max-runtime 43200 \
        2>&1 | tee refactor_overnight.log &
    
    # Save PID for monitoring
    echo $! > .refactor_state/refactor.pid
    
    log "${GREEN}Refactor server started with PID $(cat .refactor_state/refactor.pid)${NC}"
    
    # Create initial state file
    cat > .refactor_state/current_state.json <<EOF
{
  "state": "ANALYZING",
  "started": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "pid": $(cat .refactor_state/refactor.pid),
  "checkpoint_dir": ".refactor_state",
  "files_processed": 0,
  "refactors_applied": 0,
  "current_file": null
}
EOF
    
    monitor_progress
}

# Run main function
main "$@"