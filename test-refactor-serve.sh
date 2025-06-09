#!/bin/bash
# Test script for refactor serve command

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo "Testing refactor serve command..."

# Create test directories
mkdir -p .test_refactor_state
mkdir -p test_output

# Create a minimal test config
cat > .test_refactor_state/test_config.json <<'EOF'
{
  "target_complexity": 15,
  "remove_satd": true,
  "max_function_lines": 50,
  "parallel_workers": 2,
  "memory_limit_mb": 1024,
  "batch_size": 10,
  "priority_expression": "complexity DESC",
  "auto_commit_template": "test: refactor {file}",
  "thresholds": {
    "cyclomatic_warn": 10,
    "cyclomatic_error": 20,
    "cognitive_warn": 15,
    "cognitive_error": 30,
    "tdg_warn": 1.5,
    "tdg_error": 2.0
  },
  "strategies": {
    "prefer_functional": true,
    "use_early_returns": true,
    "extract_helpers": true
  }
}
EOF

# Test 1: Check help text
echo -e "\n${GREEN}Test 1: Check help for refactor serve${NC}"
cargo run --release -- refactor serve --help || echo -e "${RED}Help command failed${NC}"

# Test 2: Run with config file (limited runtime)
echo -e "\n${GREEN}Test 2: Run with config file (5 second test)${NC}"
timeout 5s cargo run --release -- refactor serve \
    --config .test_refactor_state/test_config.json \
    --project server/src \
    --checkpoint-dir .test_refactor_state \
    --max-runtime 5 \
    2>&1 | tee test_output/serve_test.log || true

# Test 3: Check if checkpoint was created
echo -e "\n${GREEN}Test 3: Check checkpoint creation${NC}"
if [ -f .test_refactor_state/checkpoint.json ]; then
    echo "✅ Checkpoint file created"
    cat .test_refactor_state/checkpoint.json | head -5
else
    echo "❌ No checkpoint file found"
fi

# Test 4: Test resume functionality
echo -e "\n${GREEN}Test 4: Test resume functionality${NC}"
timeout 3s cargo run --release -- refactor serve \
    --resume \
    --checkpoint-dir .test_refactor_state \
    --max-runtime 3 \
    2>&1 | tee -a test_output/serve_test.log || true

# Cleanup
echo -e "\n${GREEN}Cleaning up test files...${NC}"
rm -rf .test_refactor_state test_output

echo -e "\n${GREEN}Test complete!${NC}"