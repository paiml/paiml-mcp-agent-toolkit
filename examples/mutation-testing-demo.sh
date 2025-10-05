#!/bin/bash
# Mutation Testing Demonstration Script
#
# This script demonstrates PMAT's mutation testing capabilities with a
# simple calculator example. It shows how mutation testing reveals test gaps.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  PMAT Mutation Testing Demo${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo ""

# Check if pmat is installed
if ! command -v pmat &> /dev/null; then
    echo -e "${RED}Error: pmat is not installed${NC}"
    echo "Install with: cargo install pmat"
    exit 1
fi

# Check version (should be >= 2.136.0 for bug-free mutation testing)
PMAT_VERSION=$(pmat --version | grep -oP '\d+\.\d+\.\d+' || echo "0.0.0")
echo -e "${GREEN}✓${NC} Using pmat version: $PMAT_VERSION"

# Check if version is >= 2.136.0
MAJOR=$(echo $PMAT_VERSION | cut -d. -f1)
MINOR=$(echo $PMAT_VERSION | cut -d. -f2)
PATCH=$(echo $PMAT_VERSION | cut -d. -f3)

if [ "$MAJOR" -lt 2 ] || ([ "$MAJOR" -eq 2 ] && [ "$MINOR" -lt 136 ]); then
    echo -e "${YELLOW}⚠️  Warning: You're using pmat $PMAT_VERSION${NC}"
    echo -e "${YELLOW}   Versions before 2.136.0 had a file corruption bug (Issue #64)${NC}"
    echo -e "${YELLOW}   Please upgrade: cargo install pmat${NC}"
    echo ""
fi

# Find the example calculator file
CALCULATOR_FILE="examples/cli-usage/calculator.rs"

if [ ! -f "$CALCULATOR_FILE" ]; then
    echo -e "${RED}Error: Example file not found: $CALCULATOR_FILE${NC}"
    echo "Please run this script from the PMAT repository root"
    exit 1
fi

echo -e "${GREEN}✓${NC} Found example file: $CALCULATOR_FILE"
echo ""

# Step 1: Show the example code
echo -e "${BLUE}Step 1: Examining the test code${NC}"
echo "----------------------------------------"
echo "The calculator has 8 functions:"
echo "  - add, subtract, multiply, divide"
echo "  - is_positive, is_even, max, are_equal"
echo ""
echo -e "${YELLOW}Note: 'are_equal' has NO tests!${NC}"
echo "Mutation testing will reveal this gap."
echo ""
read -p "Press Enter to continue..."

# Step 2: Run mutation testing
echo ""
echo -e "${BLUE}Step 2: Running mutation testing${NC}"
echo "----------------------------------------"
echo "Command: pmat analyze mutate --path $CALCULATOR_FILE"
echo ""

# Run mutation testing and save output
MUTATION_OUTPUT=$(pmat analyze mutate --path "$CALCULATOR_FILE" 2>&1 || true)
echo "$MUTATION_OUTPUT"

# Extract mutation score
MUTATION_SCORE=$(echo "$MUTATION_OUTPUT" | grep -oP 'Mutation score: \K[\d.]+' || echo "0")
echo ""
echo -e "${GREEN}✓${NC} Mutation testing complete"
echo -e "  Mutation Score: ${YELLOW}${MUTATION_SCORE}%${NC}"
echo ""

# Step 3: Analyze results
echo -e "${BLUE}Step 3: Analyzing results${NC}"
echo "----------------------------------------"

# Check for survived mutants
SURVIVED=$(echo "$MUTATION_OUTPUT" | grep -oP '\d+ survived' | grep -oP '\d+' || echo "0")
KILLED=$(echo "$MUTATION_OUTPUT" | grep -oP '\d+ (mutants )?killed' | grep -oP '\d+' || echo "0")

echo -e "Mutants killed:   ${GREEN}$KILLED${NC}"
echo -e "Mutants survived: ${RED}$SURVIVED${NC}"
echo ""

if [ "$SURVIVED" -gt 0 ]; then
    echo -e "${YELLOW}⚠️  Found test gaps!${NC}"
    echo ""
    echo "Survived mutants indicate:"
    echo "  1. Missing edge case tests (e.g., n == 0 for is_positive)"
    echo "  2. Untested functions (are_equal has no tests)"
    echo "  3. Weak assertions (tests don't check all behaviors)"
    echo ""
    echo "Action items:"
    echo "  - Review survived mutants in the output above"
    echo "  - Add tests for untested functions"
    echo "  - Add edge case tests (zero, negative, equal values)"
fi

# Step 4: Demonstrate fix
echo ""
echo -e "${BLUE}Step 4: How to improve mutation score${NC}"
echo "----------------------------------------"
echo "To improve the mutation score:"
echo ""
echo "1. Add tests for are_equal():"
echo "   #[test]"
echo "   fn test_are_equal() {"
echo "       assert!(are_equal(5, 5));"
echo "       assert!(!are_equal(5, 3));"
echo "   }"
echo ""
echo "2. Add edge case tests:"
echo "   - Test is_positive with negative numbers"
echo "   - Test max with equal numbers"
echo "   - Test subtract with negative results"
echo ""
echo "3. Re-run mutation testing to verify improvements"
echo ""

# Step 5: Performance comparison
echo -e "${BLUE}Step 5: Performance highlights${NC}"
echo "----------------------------------------"
echo "PMAT Mutation Testing features:"
echo ""
echo -e "${GREEN}✓${NC} 20× faster than cargo-mutants"
echo "  - Smart test filtering (only runs relevant tests)"
echo "  - Module-based execution optimization"
echo ""
echo -e "${GREEN}✓${NC} Better mutation coverage"
echo "  - Generates 7× more mutants than cargo-mutants"
echo "  - Finds more test gaps"
echo ""
echo -e "${GREEN}✓${NC} Human-readable output"
echo "  - Properly formatted mutant source code"
echo "  - File corruption bug fixed in v2.136.0"
echo ""

# Summary
echo ""
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Demo Complete!${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo ""
echo "What you learned:"
echo "  ✓ How mutation testing reveals test gaps"
echo "  ✓ How to interpret mutation scores"
echo "  ✓ How to identify missing tests"
echo "  ✓ PMAT's performance advantages"
echo ""
echo "Next steps:"
echo "  1. Run mutation testing on your own code"
echo "  2. Add tests for survived mutants"
echo "  3. Aim for 75-85% mutation score"
echo "  4. Integrate into CI/CD with --min-score flag"
echo ""
echo "Documentation:"
echo "  - Full guide: docs/mutation-testing.md"
echo "  - Examples: examples/cli-usage/mutation-testing-example.md"
echo "  - Issue #64: https://github.com/paiml/paiml-mcp-agent-toolkit/issues/64"
echo ""
