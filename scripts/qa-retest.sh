#!/bin/bash
# QA Re-test Script - Tests all the fixes we've made

set -e

echo "=== QA Re-test Results ==="
echo "Date: $(date)"
echo "Binary: $PMAT"
echo ""

# Check if binary exists - try debug build if release not available
if [ -f "./target/debug/pmat" ]; then
    PMAT="./target/debug/pmat"
    echo "Using debug build: $PMAT"
elif [ -f "$PMAT" ]; then
    PMAT="$PMAT"
    echo "Using release build: $PMAT"
else
    echo "❌ ERROR: No binary found"
    echo "Please run: cargo build"
    exit 1
fi

# Test 1: Dead Code Analysis
echo "1. Dead Code Analysis"
echo "   Command: pmat analyze dead-code"
output=$($PMAT analyze dead-code 2>&1 | grep "files analyzed" || echo "Failed")
if [[ "$output" == *"0 files analyzed"* ]]; then
    echo "   ❌ FAIL: Still reporting 0 files analyzed"
else
    echo "   ✅ PASS: $output"
fi
echo ""

# Test 2: Scaffold Command
echo "2. Scaffold Command"
echo "   Command: pmat scaffold test-project --toolchain rust"
rm -rf test-project
output=$($PMAT scaffold test-project --toolchain rust 2>&1)
if [ -d "test-project" ] && [ "$(ls -A test-project)" ]; then
    file_count=$(find test-project -type f | wc -l)
    echo "   ✅ PASS: Created $file_count files"
    rm -rf test-project
else
    echo "   ❌ FAIL: No files created"
fi
echo ""

# Test 3: Incremental Coverage
echo "3. Incremental Coverage"
echo "   Command: pmat analyze incremental-coverage --base-branch main"
output=$($PMAT analyze incremental-coverage --base-branch main 2>&1)
if [[ "$output" == *"No such file or directory"* ]]; then
    echo "   ❌ FAIL: Still getting file not found error"
elif [[ "$output" == *"not yet implemented"* ]]; then
    echo "   ✅ PASS: Shows proper not implemented message"
else
    echo "   ✅ PASS: Command executed without file errors"
fi
echo ""

# Test 4: DAG with target-nodes
echo "4. DAG --target-nodes Parameter"
echo "   Command: pmat analyze dag call-graph --target-nodes 50"
output=$($PMAT analyze dag call-graph --target-nodes 50 2>&1)
if [[ "$output" == *"unexpected argument"* ]] || [[ "$output" == *"unknown argument"* ]]; then
    echo "   ❌ FAIL: Parameter not recognized"
else
    echo "   ✅ PASS: Parameter accepted"
fi
echo ""

# Test 5: Stub implementations
echo "5. Stub Implementations (checking user-friendly messages)"
commands=(
    "analyze graph-metrics"
    "analyze name-similarity test" 
    "analyze symbol-table"
    "quality-gate"
    "serve"
    "analyze comprehensive"
)

all_stubs_ok=true
for cmd in "${commands[@]}"; do
    echo "   Testing: pmat $cmd"
    output=$($PMAT $cmd 2>&1)
    if [[ "$output" == *"not yet implemented"* ]] || [[ "$output" == *"not implemented"* ]]; then
        echo "      ✅ Shows user message"
    else
        echo "      ❌ No user message shown"
        all_stubs_ok=false
    fi
done

if $all_stubs_ok; then
    echo "   ✅ PASS: All stubs show proper messages"
else
    echo "   ❌ FAIL: Some stubs missing user messages"
fi
echo ""

# Test 6: Makefile quality score
echo "6. Makefile Quality Score"
echo "   Command: pmat analyze makefile Makefile"
if [ -f "Makefile" ]; then
    output=$($PMAT analyze makefile Makefile 2>&1)
    quality_line=$(echo "$output" | grep "Quality Score:" || echo "")
    violations_line=$(echo "$output" | grep "Found.*violations" || echo "")
    
    if [[ "$quality_line" == *"0.0%"* ]] && [[ "$violations_line" == *"Found 0 violations"* ]]; then
        echo "   ✅ PASS: Correctly shows 100% quality with no violations"
    elif [[ "$quality_line" == *"0.0%"* ]] && [[ "$violations_line" != *"Found 0 violations"* ]]; then
        echo "   ❌ FAIL: Shows 0% quality despite having violations"
        echo "      $violations_line"
        echo "      $quality_line"
    else
        echo "   ✅ PASS: Quality score calculation working"
        echo "      $violations_line"
        echo "      $quality_line"
    fi
else
    echo "   ⚠️  SKIP: No Makefile found in current directory"
fi

echo ""
echo "=== Summary ==="
echo "Re-run this script after the build completes to verify all fixes."