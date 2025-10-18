#!/bin/bash
# Fast pmat-book validation script
# Runs critical chapter tests in parallel for speed
# Exit immediately on first failure (fail-fast)

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

BOOK_DIR="${PMAT_BOOK_DIR:-/home/noah/src/pmat-book}"
PARALLEL_JOBS="${PMAT_BOOK_JOBS:-4}"

# Check if pmat-book exists
if [ ! -d "$BOOK_DIR" ]; then
    echo -e "${YELLOW}⚠️  pmat-book not found at $BOOK_DIR${NC}"
    echo -e "${YELLOW}   Set PMAT_BOOK_DIR env var or skip with: git commit --no-verify${NC}"
    exit 0  # Don't fail if book doesn't exist
fi

echo -e "${YELLOW}📚 Validating pmat-book (parallel, fail-fast)${NC}"
echo ""

# Critical chapters that MUST pass (covers all major functionality)
CRITICAL_CHAPTERS=(
    "13"  # Multi-language support (CRITICAL)
    "14"  # Quality-driven development
    "05"  # Analyze command suite
    "07"  # Quality gate
)

# Specific test files to run for each chapter (if needed)
declare -A SPECIFIC_TESTS
SPECIFIC_TESTS[13]="test_language_examples.sh"  # Use main multi-lang test, not minimal

# Track failures
FAILED_TESTS=()
PASSED_TESTS=()

# Function to run a single chapter test
run_chapter_test() {
    local ch=$1
    local test_dir="$BOOK_DIR/tests/ch$ch"

    if [ ! -d "$test_dir" ]; then
        echo -e "${YELLOW}⚠️  Chapter $ch test directory not found${NC}"
        return 0
    fi

    # Reconstruct SPECIFIC_TESTS array from exported variables in subshell
    declare -A SPECIFIC_TESTS
    eval "SPECIFIC_TESTS[13]=\"\${SPECIFIC_TESTS_13:-}\""

    # Check if we have a specific test file for this chapter
    local test_scripts=""
    if [ -n "${SPECIFIC_TESTS[$ch]:-}" ]; then
        test_scripts="$test_dir/${SPECIFIC_TESTS[$ch]}"
        if [ ! -f "$test_scripts" ]; then
            echo -e "${YELLOW}⚠️  Specific test ${SPECIFIC_TESTS[$ch]} not found for Chapter $ch${NC}"
            test_scripts=""
        fi
    fi

    # Otherwise, find all test scripts
    if [ -z "$test_scripts" ]; then
        test_scripts=$(find "$test_dir" -name "test_*.sh" -type f 2>/dev/null || true)
    fi

    if [ -z "$test_scripts" ]; then
        echo -e "${YELLOW}⚠️  No test scripts found for Chapter $ch${NC}"
        return 0
    fi

    # Run each test script
    for script in $test_scripts; do
        local script_name=$(basename "$script")
        if timeout 60 bash "$script" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ Ch$ch: $script_name${NC}"
            PASSED_TESTS+=("Ch$ch:$script_name")
        else
            echo -e "${RED}❌ Ch$ch: $script_name FAILED${NC}"
            FAILED_TESTS+=("Ch$ch:$script_name")
            return 1  # Fail fast
        fi
    done

    return 0
}

# Run tests in parallel with fail-fast
export -f run_chapter_test
export BOOK_DIR
export GREEN RED YELLOW NC
export -a PASSED_TESTS FAILED_TESTS
# Export the associative array for use in subshells
for key in "${!SPECIFIC_TESTS[@]}"; do
    export "SPECIFIC_TESTS_$key=${SPECIFIC_TESTS[$key]}"
done

# Use xargs for parallel execution with fail-fast
printf "%s\n" "${CRITICAL_CHAPTERS[@]}" | \
    xargs -P "$PARALLEL_JOBS" -I {} bash -c 'run_chapter_test "$@"' _ {}

EXIT_CODE=$?

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ QUALITY GATE PASSED: pmat-book validation${NC}"
    echo -e "${GREEN}   ${#CRITICAL_CHAPTERS[@]} critical chapters validated${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 0
else
    echo -e "${RED}❌ QUALITY GATE FAILED: pmat-book validation${NC}"
    echo -e "${RED}   One or more critical tests failed${NC}"
    echo ""
    echo "To bypass (NOT RECOMMENDED):"
    echo "  git commit --no-verify"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 1
fi
