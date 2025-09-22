#!/bin/bash

# PMAT Agent System - Comprehensive Quality Check Script
# This script runs all quality checks before release

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Quality report file
REPORT_FILE="quality_report_$(date +%Y%m%d_%H%M%S).md"

echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   PMAT Agent System - Pre-Release Quality Check${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════${NC}\n"

# Initialize report
echo "# PMAT Agent System Quality Report" > $REPORT_FILE
echo "Generated: $(date)" >> $REPORT_FILE
echo "" >> $REPORT_FILE

# Function to add section to report
add_to_report() {
    echo "$1" >> $REPORT_FILE
}

# Function to check command result
check_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓ $2 passed${NC}"
        add_to_report "✅ **$2**: PASSED"
    else
        echo -e "${RED}✗ $2 failed${NC}"
        add_to_report "❌ **$2**: FAILED"
        FAILED_CHECKS+=1
    fi
}

FAILED_CHECKS=0

# Change to server directory
cd server

# ═══════════════════════════════════════════════════════
# 1. Build Check
# ═══════════════════════════════════════════════════════
echo -e "\n${YELLOW}[1/10] Running Build Check...${NC}"
add_to_report "\n## 1. Build Check"

cargo build --release 2>&1 | tee build.log | tail -5
BUILD_RESULT=$?
check_result $BUILD_RESULT "Build"

if [ $BUILD_RESULT -ne 0 ]; then
    add_to_report "\`\`\`"
    tail -20 build.log >> $REPORT_FILE
    add_to_report "\`\`\`"
fi

# ═══════════════════════════════════════════════════════
# 2. Test Suite
# ═══════════════════════════════════════════════════════
echo -e "\n${YELLOW}[2/10] Running Test Suite...${NC}"
add_to_report "\n## 2. Test Suite"

cargo test --all 2>&1 | tee test.log
TEST_RESULT=$?

# Count test results
TEST_PASSED=$(grep -c "test result: ok" test.log || echo "0")
TEST_COUNT=$(grep "running" test.log | tail -1 | grep -oE '[0-9]+ test' | grep -oE '[0-9]+' || echo "0")

echo -e "Tests: ${GREEN}$TEST_PASSED passed${NC} out of $TEST_COUNT"
add_to_report "- Total tests: $TEST_COUNT"
add_to_report "- Passed: $TEST_PASSED"
check_result $TEST_RESULT "Test Suite"

# ═══════════════════════════════════════════════════════
# 3. Test Coverage
# ═══════════════════════════════════════════════════════
echo -e "\n${YELLOW}[3/10] Running Coverage Analysis...${NC}"
add_to_report "\n## 3. Test Coverage"

# Install cargo-llvm-cov if not present
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "Installing cargo-llvm-cov..."
    cargo install cargo-llvm-cov
fi

# Run coverage
cargo llvm-cov --all-features --workspace --ignore-filename-regex="tests/|bin/" --lcov --output-path lcov.info 2>&1 | tee coverage.log || true

# Extract coverage percentage from the summary
COVERAGE=$(cargo llvm-cov report --summary-only 2>&1 | grep -oE '[0-9]+\.[0-9]+%' | tail -1 || echo "0%")
COVERAGE_NUM=$(echo $COVERAGE | grep -oE '[0-9]+' | head -1 || echo "0")

echo -e "Coverage: ${BLUE}$COVERAGE${NC}"
add_to_report "- **Coverage**: $COVERAGE"

if [ "$COVERAGE_NUM" -gt 80 ]; then
    add_to_report "- ✅ Coverage exceeds 80% threshold"
else
    add_to_report "- ⚠️  Coverage below 80% threshold"
    FAILED_CHECKS+=1
fi

# ═══════════════════════════════════════════════════════
# 4. Clippy Lints
# ═══════════════════════════════════════════════════════
echo -e "\n${YELLOW}[4/10] Running Clippy Lints...${NC}"
add_to_report "\n## 4. Clippy Analysis"

cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee clippy.log
CLIPPY_RESULT=$?

# Count warnings and errors
CLIPPY_WARNINGS=$(grep -c "warning:" clippy.log || echo "0")
CLIPPY_ERRORS=$(grep -c "error:" clippy.log || echo "0")

echo -e "Clippy: ${YELLOW}$CLIPPY_WARNINGS warnings${NC}, ${RED}$CLIPPY_ERRORS errors${NC}"
add_to_report "- Warnings: $CLIPPY_WARNINGS"
add_to_report "- Errors: $CLIPPY_ERRORS"

if [ $CLIPPY_ERRORS -eq 0 ]; then
    add_to_report "- ✅ No clippy errors"
else
    add_to_report "- ❌ Clippy errors found"
    FAILED_CHECKS+=1
fi

# ═══════════════════════════════════════════════════════
# 5. Format Check
# ═══════════════════════════════════════════════════════
echo -e "\n${YELLOW}[5/10] Running Format Check...${NC}"
add_to_report "\n## 5. Code Formatting"

cargo fmt -- --check 2>&1 | tee fmt.log
FMT_RESULT=$?
check_result $FMT_RESULT "Format Check"

# ═══════════════════════════════════════════════════════
# 6. Security Audit
# ═══════════════════════════════════════════════════════
echo -e "\n${YELLOW}[6/10] Running Security Audit...${NC}"
add_to_report "\n## 6. Security Audit"

# Install cargo-audit if not present
if ! command -v cargo-audit &> /dev/null; then
    echo "Installing cargo-audit..."
    cargo install cargo-audit
fi

cargo audit 2>&1 | tee audit.log || true

# Count vulnerabilities
VULN_COUNT=$(grep -c "Vulnerability" audit.log || echo "0")

echo -e "Vulnerabilities found: ${RED}$VULN_COUNT${NC}"
add_to_report "- Vulnerabilities: $VULN_COUNT"

if [ $VULN_COUNT -eq 0 ]; then
    add_to_report "- ✅ No known vulnerabilities"
else
    add_to_report "- ⚠️  Vulnerabilities detected"
    grep "Vulnerability" audit.log | head -5 >> $REPORT_FILE
fi

# ═══════════════════════════════════════════════════════
# 7. Dependency Check
# ═══════════════════════════════════════════════════════
echo -e "\n${YELLOW}[7/10] Running Dependency Check...${NC}"
add_to_report "\n## 7. Dependencies"

# Count dependencies
DEP_COUNT=$(cargo tree | grep -v "│" | grep -v "├" | grep -v "└" | wc -l)
DIRECT_DEPS=$(grep -c "^[a-z]" Cargo.toml | grep -v "#" || echo "0")

echo -e "Dependencies: ${BLUE}$DIRECT_DEPS direct, $DEP_COUNT total${NC}"
add_to_report "- Direct dependencies: $DIRECT_DEPS"
add_to_report "- Total dependencies: $DEP_COUNT"

# Check for outdated
if command -v cargo-outdated &> /dev/null; then
    OUTDATED=$(cargo outdated | grep -c "^" || echo "0")
    add_to_report "- Outdated dependencies: $OUTDATED"
fi

# ═══════════════════════════════════════════════════════
# 8. SATD Detection (Self-Admitted Technical Debt)
# ═══════════════════════════════════════════════════════
echo -e "\n${YELLOW}[8/10] Checking for Technical Debt...${NC}"
add_to_report "\n## 8. Technical Debt Analysis"

# Search for SATD patterns
TODO_COUNT=$(grep -r "TODO" src/ --include="*.rs" | wc -l || echo "0")
FIXME_COUNT=$(grep -r "FIXME" src/ --include="*.rs" | wc -l || echo "0")
HACK_COUNT=$(grep -r "HACK" src/ --include="*.rs" | wc -l || echo "0")
XXX_COUNT=$(grep -r "XXX" src/ --include="*.rs" | wc -l || echo "0")
UNWRAP_COUNT=$(grep -r "\.unwrap()" src/ --include="*.rs" | wc -l || echo "0")
PANIC_COUNT=$(grep -r "panic!" src/ --include="*.rs" | wc -l || echo "0")

TOTAL_SATD=$((TODO_COUNT + FIXME_COUNT + HACK_COUNT + XXX_COUNT))

echo -e "SATD Items: ${YELLOW}$TOTAL_SATD${NC} (TODO: $TODO_COUNT, FIXME: $FIXME_COUNT, HACK: $HACK_COUNT)"
echo -e "Unsafe patterns: unwrap(): ${YELLOW}$UNWRAP_COUNT${NC}, panic!: ${YELLOW}$PANIC_COUNT${NC}"

add_to_report "- TODO comments: $TODO_COUNT"
add_to_report "- FIXME comments: $FIXME_COUNT"
add_to_report "- HACK comments: $HACK_COUNT"
add_to_report "- XXX comments: $XXX_COUNT"
add_to_report "- .unwrap() calls: $UNWRAP_COUNT"
add_to_report "- panic! calls: $PANIC_COUNT"

if [ $TOTAL_SATD -eq 0 ]; then
    add_to_report "- ✅ Zero SATD (Zero tolerance achieved!)"
else
    add_to_report "- ⚠️  SATD items found"
    FAILED_CHECKS+=1
fi

# ═══════════════════════════════════════════════════════
# 9. Complexity Analysis
# ═══════════════════════════════════════════════════════
echo -e "\n${YELLOW}[9/10] Running Complexity Analysis...${NC}"
add_to_report "\n## 9. Code Complexity"

# Count lines of code
LOC=$(find src -name "*.rs" -exec wc -l {} + | tail -1 | awk '{print $1}')
FILE_COUNT=$(find src -name "*.rs" | wc -l)
AVG_FILE_SIZE=$((LOC / FILE_COUNT))

# Find largest files
LARGEST_FILE=$(find src -name "*.rs" -exec wc -l {} + | sort -rn | head -2 | tail -1)

echo -e "Total LOC: ${BLUE}$LOC${NC} in $FILE_COUNT files"
echo -e "Average file size: ${BLUE}$AVG_FILE_SIZE${NC} lines"

add_to_report "- Total lines of code: $LOC"
add_to_report "- Number of files: $FILE_COUNT"
add_to_report "- Average file size: $AVG_FILE_SIZE lines"
add_to_report "- Largest file: $LARGEST_FILE"

# Check for overly complex functions (using simple heuristic)
LONG_FUNCTIONS=$(grep -n "^fn " src/**/*.rs | while read line; do
    file=$(echo $line | cut -d: -f1)
    start=$(echo $line | cut -d: -f2)
    end=$(grep -n "^}" $file | awk -v s="$start" '$1 > s {print $1; exit}' | cut -d: -f1)
    if [ ! -z "$end" ]; then
        length=$((end - start))
        if [ $length -gt 50 ]; then
            echo "$file:$start (${length} lines)"
        fi
    fi
done | wc -l || echo "0")

add_to_report "- Functions > 50 lines: $LONG_FUNCTIONS"

if [ $LONG_FUNCTIONS -gt 10 ]; then
    add_to_report "- ⚠️  Many long functions detected"
fi

# ═══════════════════════════════════════════════════════
# 10. Documentation Coverage
# ═══════════════════════════════════════════════════════
echo -e "\n${YELLOW}[10/10] Checking Documentation...${NC}"
add_to_report "\n## 10. Documentation"

# Check for missing docs
cargo doc --no-deps 2>&1 | tee doc.log > /dev/null

# Count public items without docs (simplified check)
MISSING_DOCS=$(grep -c "missing documentation" doc.log || echo "0")

echo -e "Missing documentation: ${YELLOW}$MISSING_DOCS items${NC}"
add_to_report "- Missing documentation: $MISSING_DOCS items"

if [ $MISSING_DOCS -eq 0 ]; then
    add_to_report "- ✅ All public items documented"
else
    add_to_report "- ⚠️  Some public items lack documentation"
fi

# ═══════════════════════════════════════════════════════
# Final Summary
# ═══════════════════════════════════════════════════════
echo -e "\n${BLUE}════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}                    QUALITY SUMMARY${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"

add_to_report "\n## Summary"

if [ $FAILED_CHECKS -eq 0 ]; then
    echo -e "\n${GREEN}✅ ALL QUALITY CHECKS PASSED!${NC}"
    echo -e "${GREEN}The system is ready for release.${NC}"
    add_to_report "\n### ✅ **ALL QUALITY CHECKS PASSED**"
    add_to_report "The PMAT Agent System meets all quality standards and is ready for release."
else
    echo -e "\n${RED}❌ $FAILED_CHECKS QUALITY CHECKS FAILED${NC}"
    echo -e "${YELLOW}Please address the issues before release.${NC}"
    add_to_report "\n### ⚠️  **$FAILED_CHECKS QUALITY CHECKS NEED ATTENTION**"
    add_to_report "Please review and address the issues identified above."
fi

# Add metrics summary
add_to_report "\n### Key Metrics"
add_to_report "- 📊 Test Coverage: $COVERAGE"
add_to_report "- 🧪 Tests Passed: $TEST_PASSED/$TEST_COUNT"
add_to_report "- 📝 Lines of Code: $LOC"
add_to_report "- 🔍 SATD Items: $TOTAL_SATD"
add_to_report "- ⚠️  Clippy Warnings: $CLIPPY_WARNINGS"
add_to_report "- 🔒 Security Vulnerabilities: $VULN_COUNT"

echo -e "\n📄 Quality report saved to: ${BLUE}$REPORT_FILE${NC}"

# Clean up log files
rm -f build.log test.log coverage.log clippy.log fmt.log audit.log doc.log

exit $FAILED_CHECKS