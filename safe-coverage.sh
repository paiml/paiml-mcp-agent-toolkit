#!/bin/bash
# Safe coverage generation script

echo "🔍 Generating test coverage report safely..."
echo "========================================="

# Check system load first
LOAD=$(uptime | awk -F'load average:' '{print $2}' | awk -F, '{print $1}' | xargs)
CORES=$(nproc)

echo "System load: $LOAD (on $CORES cores)"

# Use minimal resources
export RUST_TEST_THREADS=2
export CARGO_BUILD_JOBS=4

echo "Running tests with coverage..."

# Run with safe parameters
cargo llvm-cov test \
    --lib \
    --no-fail-fast \
    -- --test-threads=2 \
    2>&1 | tail -50

echo ""
echo "Generating coverage summary..."

# Get summary
cargo llvm-cov report --summary-only 2>&1 | grep "TOTAL" || echo "Failed to generate summary"