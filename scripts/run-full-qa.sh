#!/bin/bash
# Full QA Checklist Re-run Script

set -e

echo "=== PMAT QA Checklist Re-run ==="
# Identify the tree under test, not the wall clock: the revision is what makes
# two runs of this report comparable.
echo "Revision: $(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
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

# Function to test a command.
# Usage: test_command <description> <expected> <command> [args...]
# The command is taken as argv and executed directly, so it is never re-parsed
# by a shell (the previous `eval "$cmd"` was).
test_command() {
    local desc="$1"
    local expected="$2"
    shift 2

    echo "Testing: $desc"
    echo "Command: $*"

    # Capture the real exit code via an `if` (exempt from `set -e`), not a
    # `cmd || true` that unconditionally forces $? to 0 before it can be read.
    if output=$("$@" 2>&1); then
        rc=0
    else
        rc=$?
    fi

    case "$expected" in
        "success")
            if [[ $rc -eq 0 ]] && [[ "$output" != *"not yet implemented"* ]] && [[ "$output" != *"not implemented"* ]]; then
                echo "✅ PASS"
                PASSED=$((PASSED + 1))
            else
                echo "❌ FAIL: Command failed or not implemented"
                FAILED=$((FAILED + 1))
            fi
            ;;
        "not-impl")
            if [[ "$output" == *"not yet implemented"* ]] || [[ "$output" == *"not implemented"* ]]; then
                echo "⚠️  NOT IMPLEMENTED (expected)"
                NOT_IMPL=$((NOT_IMPL + 1))
            else
                echo "❌ FAIL: Should show not implemented message"
                FAILED=$((FAILED + 1))
            fi
            ;;
        *)
            if [[ "$output" == *"$expected"* ]]; then
                echo "✅ PASS"
                PASSED=$((PASSED + 1))
            else
                echo "❌ FAIL: Expected '$expected' in output"
                FAILED=$((FAILED + 1))
            fi
            ;;
    esac
    echo ""
}

# Run all tests from QA checklist
echo "=== Basic Commands ==="
test_command "Help" "Professional project quantitative" "$PMAT_BIN" --help
test_command "Version" "paiml-mcp-agent-toolkit" "$PMAT_BIN" --version

echo "=== Analysis Commands ==="
test_command "Complexity Analysis" "Analyzing.*project complexity" "$PMAT_BIN" analyze complexity
test_command "SATD Analysis" "Analyzing self-admitted technical debt" "$PMAT_BIN" analyze satd
test_command "Dead Code Analysis" "files analyzed" "$PMAT_BIN" analyze dead-code
test_command "DAG Generation" "Generating dependency analysis graph" "$PMAT_BIN" analyze dag call-graph
test_command "DAG with target-nodes" "Generating dependency analysis graph" "$PMAT_BIN" analyze dag call-graph --target-nodes 50
test_command "Deep Context" "Analyzing project context" "$PMAT_BIN" analyze deep-context
test_command "TDG Analysis" "Calculating Technical Debt Gradient" "$PMAT_BIN" analyze tdg
test_command "Churn Analysis" "Analyzing code churn" "$PMAT_BIN" analyze churn
test_command "Duplicates" "Detecting code duplicates" "$PMAT_BIN" analyze duplicates
test_command "Big-O Analysis" "not-impl" "$PMAT_BIN" analyze big-o
test_command "Defect Prediction" "Predicting potential defects" "$PMAT_BIN" analyze defect-prediction
test_command "Proof Annotations" "not-impl" "$PMAT_BIN" analyze proof-annotations
test_command "Incremental Coverage" "not-impl" "$PMAT_BIN" analyze incremental-coverage --base-branch main
test_command "Symbol Table" "not-impl" "$PMAT_BIN" analyze symbol-table
test_command "Name Similarity" "not-impl" "$PMAT_BIN" analyze name-similarity test
test_command "Graph Metrics" "not-impl" "$PMAT_BIN" analyze graph-metrics
test_command "Comprehensive" "not-impl" "$PMAT_BIN" analyze comprehensive
test_command "Provability" "Analyzing code provability" "$PMAT_BIN" analyze provability

# Makefile analysis
if [ -f "Makefile" ]; then
    test_command "Makefile Analysis" "Quality Score:" "$PMAT_BIN" analyze makefile Makefile
fi

echo "=== Generation Commands ==="
rm -rf test-scaffold-project
test_command "Scaffold Rust" "Project scaffolded successfully" "$PMAT_BIN" scaffold test-scaffold-project --toolchain rust
if [ -d "test-scaffold-project" ]; then
    file_count=$(find test-scaffold-project -type f | wc -l)
    if [ $file_count -gt 0 ]; then
        echo "✅ Scaffold created $file_count files"
        PASSED=$((PASSED + 1))
    else
        echo "❌ Scaffold created no files"
        FAILED=$((FAILED + 1))
    fi
    rm -rf test-scaffold-project
fi

echo "=== Other Commands ==="
test_command "Context" "Files" "$PMAT_BIN" context
test_command "Tokenize" "not-impl" "$PMAT_BIN" tokenize README.md
test_command "Explain" "not-impl" "$PMAT_BIN" explain
test_command "Refactor" "not-impl" "$PMAT_BIN" refactor extract-function test.rs:10-20 new_func
test_command "Quality Gate" "not-impl" "$PMAT_BIN" quality-gate
test_command "Serve" "not-impl" "$PMAT_BIN" serve
test_command "Chat" "not-impl" "$PMAT_BIN" chat test
test_command "Report" "not-impl" "$PMAT_BIN" report quality
test_command "Search" "not-impl" "$PMAT_BIN" search TODO
test_command "Diff" "not-impl" "$PMAT_BIN" diff main feature-branch
test_command "Config Get" "not-impl" "$PMAT_BIN" config get max_file_size
test_command "Diagnose" "Checking system dependencies" "$PMAT_BIN" diagnose

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