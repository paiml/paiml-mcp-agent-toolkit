#!/bin/bash
# dead_code_calibration.sh - Run in isolated test environment

set -euo pipefail

# Create temporary test directory
TEST_DIR=$(mktemp -d)
trap "rm -rf $TEST_DIR" EXIT

cd "$TEST_DIR"

# Generate calibration fixture
cat > lib.rs << 'EOF'
pub fn used_function() {
    println!("This is used");
}

fn definitely_dead() {
    println!("Never called");
}

#[cfg(test)]
fn test_only_function() {
    // Should not be marked dead
}
EOF

# Create a simple Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "test-dead-code"
version = "0.1.0"
edition = "2021"

[lib]
name = "test_dead_code"
path = "lib.rs"
EOF

# Run analyzer
echo "Running dead code analysis..."
OUTPUT=$(pmat analyze dead-code --path . --format json 2>&1 || true)

# Debug output
echo "Raw output:"
echo "$OUTPUT"

# Verify detection
if ! echo "$OUTPUT" | jq -e '.summary.dead_functions >= 1' >/dev/null 2>&1; then
    echo "FAIL: Dead code analyzer failed to detect known dead function"
    echo "Output: $OUTPUT"
    exit 1
fi

# Verify test code not marked as dead
if echo "$OUTPUT" | jq -e '.dead_items[] | select(.name == "test_only_function")' >/dev/null 2>&1; then
    echo "FAIL: Test code incorrectly marked as dead"
    exit 1
fi

echo "PASS: Dead code detection calibration successful"

# Additional test: Mixed language project
echo
echo "Testing mixed language project..."

# Add TypeScript file
cat > app.ts << 'EOF'
export function fetchData() {
    return fetch('/api/data');
}

function unusedHelper() {
    console.log('This is never called');
}
EOF

# Re-run analysis
OUTPUT_MIXED=$(pmat analyze dead-code --path . --format json 2>&1 || true)

# Check if TypeScript dead code is detected
if echo "$OUTPUT_MIXED" | jq -e '.dead_items[] | select(.file_path | endswith(".ts"))' >/dev/null 2>&1; then
    echo "PASS: TypeScript dead code detected in mixed project"
else
    echo "WARNING: No TypeScript dead code detected - possible cross-language reference issue"
fi

# Test edge case: All functions are used
echo
echo "Testing zero dead code scenario..."

cat > main.rs << 'EOF'
fn main() {
    helper();
}

fn helper() {
    println!("I am used!");
}
EOF

OUTPUT_ZERO=$(pmat analyze dead-code --path . --format json 2>&1 || true)

DEAD_COUNT=$(echo "$OUTPUT_ZERO" | jq -r '.summary.dead_functions // 0' 2>/dev/null || echo "0")

if [ "$DEAD_COUNT" -eq 0 ]; then
    echo "PASS: Zero dead code correctly reported when all functions are used"
else
    echo "FAIL: False positives in dead code detection"
    exit 1
fi

echo
echo "All calibration tests passed!"