#!/bin/bash
# MCP Stateful Refactor Server Demo
# This script demonstrates how to interact with the stateful MCP refactor server

set -e

echo "=== MCP Stateful Refactor Server Demo ==="
echo

# Function to send JSON-RPC request and pretty print response
send_request() {
    local method=$1
    local params=$2
    local id=$3
    
    local request="{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":$id}"
    
    echo "→ Sending request:"
    echo "$request" | jq .
    
    echo "← Response:"
    echo "$request" | PMAT_REFACTOR_MCP=1 timeout 5s pmat 2>/dev/null | jq . || echo "Error: Request failed"
    echo
}

# Create test files for demonstration
TEST_DIR=$(mktemp -d)
echo "Creating test files in $TEST_DIR..."

cat > "$TEST_DIR/complex_function.rs" << 'EOF'
fn calculate_metrics(data: &[i32]) -> Result<Metrics, Error> {
    let mut sum = 0;
    let mut count = 0;
    let mut min = i32::MAX;
    let mut max = i32::MIN;
    
    for value in data {
        if *value > 0 {
            sum += value;
            count += 1;
            if *value < min {
                min = *value;
            }
            if *value > max {
                max = *value;
            }
        } else {
            if *value == 0 {
                continue;
            } else {
                return Err(Error::NegativeValue);
            }
        }
    }
    
    if count == 0 {
        return Err(Error::NoData);
    }
    
    let average = sum / count;
    
    // TODO: Add variance calculation
    // FIXME: Handle overflow in sum
    
    Ok(Metrics {
        sum,
        count,
        average,
        min,
        max,
    })
}
EOF

echo "Test file created: $TEST_DIR/complex_function.rs"
echo

# Demo 1: Start a refactoring session
echo "=== Demo 1: Starting Refactoring Session ==="
send_request "refactor.start" "{\"targets\":[\"$TEST_DIR/complex_function.rs\"],\"config\":{\"target_complexity\":10,\"remove_satd\":true}}" 1

# Demo 2: Get current state
echo "=== Demo 2: Getting Current State ==="
send_request "refactor.getState" "{}" 2

# Demo 3: Advance to next iteration
echo "=== Demo 3: Advancing to Next Iteration ==="
send_request "refactor.nextIteration" "{}" 3

# Demo 4: Get state after advancement
echo "=== Demo 4: Getting State After Advancement ==="
send_request "refactor.getState" "{}" 4

# Demo 5: Stop the session
echo "=== Demo 5: Stopping Session ==="
send_request "refactor.stop" "{}" 5

# Demo 6: Try to get state after stop (should fail)
echo "=== Demo 6: Attempting to Get State After Stop (Should Fail) ==="
send_request "refactor.getState" "{}" 6

# Cleanup
echo "Cleaning up test directory..."
rm -rf "$TEST_DIR"

echo "Demo complete!"