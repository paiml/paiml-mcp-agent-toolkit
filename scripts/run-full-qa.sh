#!/bin/bash
# Full QA Checklist Re-run Script

set -e

echo "=== PMAT QA Checklist Re-run ==="
echo "Date: $(date)"
echo "Version: $($PMAT_BIN --version 2>/dev/null || echo 'Binary not found')"
echo ""

# Check binary exists
# Try release build first, fall back to debug
if [ -f "$PMAT_BIN" ]; then
    PMAT_BIN="$PMAT_BIN"
elif [ -f "./target/debug/pmat" ]; then
    PMAT_BIN="./target/debug/pmat"
    echo "⚠️  Using debug build"
else
    echo "❌ ERROR: Binary not found. Please run: cargo build"
    exit 1
fi

# Initialize counters
PASSED=0
FAILED=0
NOT_IMPL=0

# Function to test a command
test_command() {
    local desc="$1"
    local cmd="$2"
    local expected="$3"
    
    echo "Testing: $desc"
    echo "Command: $cmd"
    
    output=$(eval "$cmd" 2>&1 || true)
    
    case "$expected" in
        "success")
            if [[ $? -eq 0 ]] && [[ "$output" != *"not yet implemented"* ]] && [[ "$output" != *"not implemented"* ]]; then
                echo "✅ PASS"
                ((PASSED++))
            else
                echo "❌ FAIL: Command failed or not implemented"
                ((FAILED++))
            fi
            ;;
        "not-impl")
            if [[ "$output" == *"not yet implemented"* ]] || [[ "$output" == *"not implemented"* ]]; then
                echo "⚠️  NOT IMPLEMENTED (expected)"
                ((NOT_IMPL++))
            else
                echo "❌ FAIL: Should show not implemented message"
                ((FAILED++))
            fi
            ;;
        *)
            if [[ "$output" == *"$expected"* ]]; then
                echo "✅ PASS"
                ((PASSED++))
            else
                echo "❌ FAIL: Expected '$expected' in output"
                ((FAILED++))
            fi
            ;;
    esac
    echo ""
}

# Run all tests from QA checklist
echo "=== Basic Commands ==="
test_command "Help" "$PMAT_BIN --help" "Professional project quantitative"
test_command "Version" "$PMAT_BIN --version" "paiml-mcp-agent-toolkit"

echo "=== Analysis Commands ==="
test_command "Complexity Analysis" "$PMAT_BIN analyze complexity" "Analyzing.*project complexity"
test_command "SATD Analysis" "$PMAT_BIN analyze satd" "Analyzing self-admitted technical debt"
test_command "Dead Code Analysis" "$PMAT_BIN analyze dead-code" "files analyzed"
test_command "DAG Generation" "$PMAT_BIN analyze dag call-graph" "Generating dependency analysis graph"
test_command "DAG with target-nodes" "$PMAT_BIN analyze dag call-graph --target-nodes 50" "Generating dependency analysis graph"
test_command "Deep Context" "$PMAT_BIN analyze deep-context" "Analyzing project context"
test_command "TDG Analysis" "$PMAT_BIN analyze tdg" "Calculating Technical Debt Gradient"
test_command "Churn Analysis" "$PMAT_BIN analyze churn" "Analyzing code churn"
test_command "Duplicates" "$PMAT_BIN analyze duplicates" "Detecting code duplicates"
test_command "Big-O Analysis" "$PMAT_BIN analyze big-o" "not-impl"
test_command "Defect Prediction" "$PMAT_BIN analyze defect-prediction" "Predicting potential defects"
test_command "Proof Annotations" "$PMAT_BIN analyze proof-annotations" "not-impl"
test_command "Incremental Coverage" "$PMAT_BIN analyze incremental-coverage --base-branch main" "not-impl"
test_command "Symbol Table" "$PMAT_BIN analyze symbol-table" "not-impl"
test_command "Name Similarity" "$PMAT_BIN analyze name-similarity test" "not-impl"
test_command "Graph Metrics" "$PMAT_BIN analyze graph-metrics" "not-impl"
test_command "Comprehensive" "$PMAT_BIN analyze comprehensive" "not-impl"
test_command "Provability" "$PMAT_BIN analyze provability" "Analyzing code provability"

# Makefile analysis
if [ -f "Makefile" ]; then
    test_command "Makefile Analysis" "$PMAT_BIN analyze makefile Makefile" "Quality Score:"
fi

echo "=== Generation Commands ==="
rm -rf test-scaffold-project
test_command "Scaffold Rust" "$PMAT_BIN scaffold test-scaffold-project --toolchain rust" "Project scaffolded successfully"
if [ -d "test-scaffold-project" ]; then
    file_count=$(find test-scaffold-project -type f | wc -l)
    if [ $file_count -gt 0 ]; then
        echo "✅ Scaffold created $file_count files"
        ((PASSED++))
    else
        echo "❌ Scaffold created no files"
        ((FAILED++))
    fi
    rm -rf test-scaffold-project
fi

echo "=== Other Commands ==="
test_command "Context" "$PMAT_BIN context" "Files"
test_command "Tokenize" "$PMAT_BIN tokenize README.md" "not-impl"
test_command "Explain" "$PMAT_BIN explain" "not-impl"
test_command "Refactor" "$PMAT_BIN refactor extract-function test.rs:10-20 new_func" "not-impl"
test_command "Quality Gate" "$PMAT_BIN quality-gate" "not-impl"
test_command "Serve" "$PMAT_BIN serve" "not-impl"
test_command "Chat" "$PMAT_BIN chat test" "not-impl"
test_command "Report" "$PMAT_BIN report quality" "not-impl"
test_command "Search" "$PMAT_BIN search TODO" "not-impl"
test_command "Diff" "$PMAT_BIN diff main feature-branch" "not-impl"
test_command "Config Get" "$PMAT_BIN config get max_file_size" "not-impl"
test_command "Diagnose" "$PMAT_BIN diagnose" "Checking system dependencies"

echo ""
echo "=== SUMMARY ==="
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Not Implemented: $NOT_IMPL"
echo "Total: $((PASSED + FAILED + NOT_IMPL))"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "✅ All implemented features are working correctly!"
    echo "Pass rate: $((PASSED * 100 / (PASSED + FAILED)))% (excluding not-implemented)"
else
    echo "❌ Some tests failed. Please review the output above."
    echo "Pass rate: $((PASSED * 100 / (PASSED + FAILED)))% (excluding not-implemented)"
fi