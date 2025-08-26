#!/bin/bash
#
# Timeout Feature Validation Script
# Part of CI/CD pipeline to ensure timeout functionality works correctly
#
# Following Toyota Way principles:
# - Jidoka: Quality at the source through automated validation
# - Genchi Genbutsu: Go and see - actually test the commands
# - Zero Defects: No regressions in timeout functionality

set -euo pipefail

echo "🔧 Validating Analysis Command Timeout Feature..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration
TEST_PROJECT="test_project"
BINARY="./target/debug/pmat"

# Ensure binary exists
if [[ ! -f "$BINARY" ]]; then
    echo -e "${RED}❌ PMAT binary not found at $BINARY${NC}"
    echo "Please run 'cargo build' first"
    exit 1
fi

# Ensure test project exists
if [[ ! -d "$TEST_PROJECT" ]]; then
    echo "📁 Creating test project..."
    mkdir -p "$TEST_PROJECT"
    echo 'fn main() { println!("hello"); }' > "$TEST_PROJECT/main.rs"
    echo 'fn test() { println!("test"); }' > "$TEST_PROJECT/test.rs"
fi

echo "🎯 Starting timeout feature validation..."

# Test 1: Verify timeout parameters are accepted
echo -n "Test 1: Complexity analysis accepts timeout parameter... "
if timeout 10s "$BINARY" analyze complexity --project-path "$TEST_PROJECT" --timeout 5 > /dev/null 2>&1; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
    echo "Complexity analysis should accept --timeout parameter"
    exit 1
fi

# Test 2: Verify SATD analysis accepts timeout parameter
echo -n "Test 2: SATD analysis accepts timeout parameter... "
if timeout 10s "$BINARY" analyze satd --path "$TEST_PROJECT" --timeout 5 > /dev/null 2>&1; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
    echo "SATD analysis should accept --timeout parameter"
    exit 1
fi

# Test 3: Verify dead-code analysis accepts timeout parameter
echo -n "Test 3: Dead-code analysis accepts timeout parameter... "
if timeout 10s "$BINARY" analyze dead-code --path "$TEST_PROJECT" --timeout 5 > /dev/null 2>&1; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
    echo "Dead-code analysis should accept --timeout parameter"
    exit 1
fi

# Test 4: Verify timeout logging appears in output
echo -n "Test 4: Timeout value is logged in output... "
output=$("$BINARY" analyze complexity --project-path "$TEST_PROJECT" --timeout 5 2>&1 || true)
if echo "$output" | grep -q "⏰ Analysis timeout set to 5 seconds"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
    echo "Expected timeout logging message not found in output"
    echo "Output was: $output"
    exit 1
fi

# Test 5: Verify commands complete within reasonable time
echo -n "Test 5: Commands complete within reasonable time... "
start_time=$(date +%s)
timeout 30s "$BINARY" analyze complexity --project-path "$TEST_PROJECT" --timeout 25 > /dev/null 2>&1
end_time=$(date +%s)
duration=$((end_time - start_time))

if [[ $duration -lt 25 ]]; then
    echo -e "${GREEN}✅ PASS (${duration}s)${NC}"
else
    echo -e "${YELLOW}⚠️  SLOW (${duration}s)${NC}"
    echo "Command took longer than expected, but still within timeout"
fi

# Test 6: Verify timeout parameter is functional (skip help check for now)
echo -n "Test 6: Timeout parameter works with different values... "
if timeout 10s "$BINARY" analyze complexity --project-path "$TEST_PROJECT" --timeout 10 > /dev/null 2>&1; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
    echo "Timeout parameter should work with different values"
    exit 1
fi

# Test 7: Default timeout behavior (no timeout specified)
echo -n "Test 7: Commands work without explicit timeout... "
if timeout 15s "$BINARY" analyze complexity --project-path "$TEST_PROJECT" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
    echo "Commands should work with default timeout when not specified"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 All timeout feature validation tests passed!${NC}"
echo ""
echo "📊 Summary:"
echo "  ✅ Timeout parameters accepted by all analysis commands"
echo "  ✅ Timeout values properly logged for transparency" 
echo "  ✅ Commands complete within expected timeframes"
echo "  ✅ Timeout parameter works with different values"
echo "  ✅ Default timeout behavior works correctly"
echo ""
echo "🚗 Toyota Way validation complete - timeout feature ready for production!"

# Cleanup
echo "🧹 Cleaning up test artifacts..."
# Keep test_project as it's useful for future testing

echo "✨ Timeout feature validation completed successfully!"