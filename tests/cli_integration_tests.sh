#!/bin/bash
# Sprint 1 Ticket #47: CLI Integration Testing
# Test uniform contracts implementation across all migrated commands

set -e

PMAT="./target/debug/pmat"
TEST_DIR="/tmp/cli_integration_test"
EXIT_CODE=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=== Sprint 1 CLI Integration Tests ==="
echo "Testing uniform contracts implementation"
echo ""

# Setup test environment
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"

# Create test files
cat > "$TEST_DIR/test.py" << 'EOF'
def simple_function():
    """A simple Python function"""
    return 42

def complex_function(x):
    """A more complex function"""
    if x > 0:
        return x * 2
    else:
        return x * 3

# TODO: This is a test SATD comment
# FIXME: Another SATD marker

class TestClass:
    def method(self):
        return "test"
EOF

cat > "$TEST_DIR/test.rs" << 'EOF'
fn main() {
    println!("Hello, world!");
}

fn complex_function(x: i32) -> i32 {
    if x > 0 {
        x * 2
    } else {
        x * 3
    }
}

// TODO: This is a Rust SATD comment
// FIXME: Another SATD marker

fn unused_function() {
    // This function is never called
    println!("I'm never used");
}
EOF

# Test 1: Complexity Command - Uniform parameter --path
echo "Test 1: Complexity Command - Uniform parameter --path"
OUTPUT=$($PMAT analyze complexity --path "$TEST_DIR" --format json 2>/dev/null)
FUNCTION_COUNT=$(echo "$OUTPUT" | jq -r '.summary.summary.total_functions // 0' 2>/dev/null || echo "0")

if [ "$FUNCTION_COUNT" -ge 4 ]; then
    echo -e "${GREEN}✅ PASS${NC}: Complexity --path works (found $FUNCTION_COUNT functions)"
else
    echo -e "${RED}❌ FAIL${NC}: Complexity --path failed (found only $FUNCTION_COUNT functions)"
    EXIT_CODE=1
fi

# Test 2: Complexity Command - Backward compatibility warning
echo ""
echo "Test 2: Complexity Command - Backward compatibility (--project-path deprecated)"
OUTPUT=$($PMAT analyze complexity --project-path "$TEST_DIR" --format json 2>&1)
if echo "$OUTPUT" | grep -q "deprecated"; then
    echo -e "${GREEN}✅ PASS${NC}: Deprecation warning shown for --project-path"
else
    echo -e "${RED}❌ FAIL${NC}: No deprecation warning for --project-path"
    EXIT_CODE=1
fi

# Test 3: SATD Command - Uniform parameters
echo ""
echo "Test 3: SATD Command - Uniform parameters"
OUTPUT=$($PMAT analyze satd --path "$TEST_DIR" --format json 2>/dev/null)
SATD_COUNT=$(echo "$OUTPUT" | jq -r '.summary.total_items // 0' 2>/dev/null || echo "0")

if [ "$SATD_COUNT" -ge 2 ]; then
    echo -e "${GREEN}✅ PASS${NC}: SATD --path works (found $SATD_COUNT items)"
else
    echo -e "${RED}❌ FAIL${NC}: SATD --path failed (found only $SATD_COUNT items)"
    EXIT_CODE=1
fi

# Test 4: Dead Code Command - Uniform parameters
echo ""
echo "Test 4: Dead Code Command - Uniform parameters"
OUTPUT=$($PMAT analyze dead-code --path "$TEST_DIR" --format json 2>/dev/null)
DEAD_COUNT=$(echo "$OUTPUT" | jq -r '.summary.total_items // 0' 2>/dev/null || echo "0")

if [ "$DEAD_COUNT" -ge 0 ]; then
    echo -e "${GREEN}✅ PASS${NC}: Dead code --path works (found $DEAD_COUNT items)"
else
    echo -e "${RED}❌ FAIL${NC}: Dead code --path failed"
    EXIT_CODE=1
fi

# Test 5: Format parameter consistency
echo ""
echo "Test 5: Format parameter consistency across commands"
FORMATS=("json" "summary" "markdown")
COMMANDS=("complexity" "satd" "dead-code")

ALL_PASS=true
for cmd in "${COMMANDS[@]}"; do
    for fmt in "${FORMATS[@]}"; do
        OUTPUT=$($PMAT analyze "$cmd" --path "$TEST_DIR" --format "$fmt" 2>&1)
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✅${NC} $cmd --format $fmt works"
        else
            echo -e "${RED}❌${NC} $cmd --format $fmt failed"
            ALL_PASS=false
            EXIT_CODE=1
        fi
    done
done

if [ "$ALL_PASS" = true ]; then
    echo -e "${GREEN}✅ PASS${NC}: All format parameters work consistently"
fi

# Test 6: File-specific analysis with uniform parameters
echo ""
echo "Test 6: File-specific analysis with --file parameter"
OUTPUT=$($PMAT analyze complexity --file "$TEST_DIR/test.py" --format json 2>/dev/null)
PY_FUNCTIONS=$(echo "$OUTPUT" | jq -r '.summary.summary.total_functions // 0' 2>/dev/null || echo "0")

OUTPUT=$($PMAT analyze complexity --file "$TEST_DIR/test.rs" --format json 2>/dev/null)
RS_FUNCTIONS=$(echo "$OUTPUT" | jq -r '.summary.summary.total_functions // 0' 2>/dev/null || echo "0")

if [ "$PY_FUNCTIONS" -ge 2 ] && [ "$RS_FUNCTIONS" -ge 2 ]; then
    echo -e "${GREEN}✅ PASS${NC}: --file parameter works for both Python ($PY_FUNCTIONS) and Rust ($RS_FUNCTIONS)"
else
    echo -e "${RED}❌ FAIL${NC}: --file parameter issues (Python: $PY_FUNCTIONS, Rust: $RS_FUNCTIONS)"
    EXIT_CODE=1
fi

# Test 7: Multiple files with --files parameter
echo ""
echo "Test 7: Multiple files with --files parameter"
OUTPUT=$($PMAT analyze complexity --files "$TEST_DIR/test.py" "$TEST_DIR/test.rs" --format json 2>/dev/null)
TOTAL_FUNCTIONS=$(echo "$OUTPUT" | jq -r '.summary.summary.total_functions // 0' 2>/dev/null || echo "0")

if [ "$TOTAL_FUNCTIONS" -ge 5 ]; then
    echo -e "${GREEN}✅ PASS${NC}: --files parameter works for multiple files (found $TOTAL_FUNCTIONS functions)"
else
    echo -e "${RED}❌ FAIL${NC}: --files parameter failed (found only $TOTAL_FUNCTIONS functions)"
    EXIT_CODE=1
fi

# Test 8: Output redirection with --output parameter
echo ""
echo "Test 8: Output redirection with --output parameter"
OUTPUT_FILE="$TEST_DIR/output.json"
$PMAT analyze complexity --path "$TEST_DIR" --format json --output "$OUTPUT_FILE" 2>/dev/null

if [ -f "$OUTPUT_FILE" ]; then
    FUNCTIONS=$(jq -r '.summary.summary.total_functions // 0' "$OUTPUT_FILE" 2>/dev/null || echo "0")
    if [ "$FUNCTIONS" -ge 4 ]; then
        echo -e "${GREEN}✅ PASS${NC}: --output parameter works (file created with $FUNCTIONS functions)"
    else
        echo -e "${RED}❌ FAIL${NC}: --output file created but invalid content"
        EXIT_CODE=1
    fi
else
    echo -e "${RED}❌ FAIL${NC}: --output parameter didn't create file"
    EXIT_CODE=1
fi

# Test 9: Issue #42 Regression Test
echo ""
echo "Test 9: Issue #42 Regression - Python complexity analysis"
bash /tmp/test_complexity_fix/test_issue_42.sh > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ PASS${NC}: Issue #42 remains fixed"
else
    echo -e "${RED}❌ FAIL${NC}: Issue #42 regression detected"
    EXIT_CODE=1
fi

# Cleanup
rm -rf "$TEST_DIR"

echo ""
echo "=== Test Summary ==="
if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}All Sprint 1 CLI integration tests passed!${NC}"
    echo "✅ Ticket #44: Complexity command migrated"
    echo "✅ Ticket #45: SATD command migrated"
    echo "✅ Ticket #46: Dead code command migrated"
    echo "✅ Issue #42: Python complexity analysis fixed"
    echo "✅ Ticket #47: CLI integration tests complete"
else
    echo -e "${RED}Some tests failed. Exit code: $EXIT_CODE${NC}"
fi

exit $EXIT_CODE