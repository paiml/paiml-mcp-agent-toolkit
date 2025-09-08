#!/bin/bash
# Test script to verify TDG is dogfooding by storing file scores

echo "=== TDG Dogfooding Verification Test ==="
echo ""

# Create a test file
TEST_FILE="/tmp/tdg_test_$(date +%s).rs"
cat > "$TEST_FILE" << 'EOF'
fn main() {
    println!("Hello, TDG!");
    let x = 42;
    let y = x * 2;
    println!("Result: {}", y);
}
EOF

echo "1. Analyzing test file: $TEST_FILE"
./target/debug/pmat tdg "$TEST_FILE"

echo ""
echo "2. Analyzing same file again (should use cached score):"
./target/debug/pmat tdg "$TEST_FILE"

echo ""
echo "3. Modifying file and re-analyzing:"
cat >> "$TEST_FILE" << 'EOF'

fn complex_function() {
    for i in 0..10 {
        for j in 0..10 {
            if i * j > 50 {
                println!("Complex: {} * {} = {}", i, j, i * j);
            }
        }
    }
}
EOF

./target/debug/pmat tdg "$TEST_FILE"

echo ""
echo "4. Cleanup"
rm "$TEST_FILE"

echo ""
echo "=== Test Complete ==="
echo "If scores were consistent on second run and changed after modification,"
echo "then TDG is successfully dogfooding by storing file scores!"