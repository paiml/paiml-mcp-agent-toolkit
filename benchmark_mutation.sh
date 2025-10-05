#!/bin/bash
# Benchmark PMAT vs cargo-mutants
# Compares mutation testing performance and accuracy

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}Mutation Testing Benchmark${NC}"
echo -e "${BLUE}PMAT vs cargo-mutants${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# Check if cargo-mutants is installed
if ! command -v cargo-mutants &> /dev/null; then
    echo -e "${YELLOW}cargo-mutants not found. Installing...${NC}"
    cargo install cargo-mutants
fi

# Build PMAT first
echo -e "${GREEN}Building PMAT...${NC}"
cargo build --release
echo ""

# Test file to benchmark (choose a small file with tests)
TEST_FILE="server/src/services/mutation/types.rs"

if [ ! -f "$TEST_FILE" ]; then
    echo -e "${RED}Test file not found: $TEST_FILE${NC}"
    exit 1
fi

echo -e "${BLUE}Test file: $TEST_FILE${NC}"
echo -e "${BLUE}Working directory: $(pwd)${NC}"
echo ""

# Benchmark PMAT
echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}Running PMAT Mutation Testing${NC}"
echo -e "${GREEN}================================${NC}"
PMAT_START=$(date +%s)

./target/release/pmat analyze mutate \
    --path "$TEST_FILE" \
    --operators AOR,ROR,COR,UOR \
    --format json \
    --output pmat_results.json 2>&1 | tee pmat_output.txt

PMAT_END=$(date +%s)
PMAT_TIME=$((PMAT_END - PMAT_START))

echo ""
echo -e "${GREEN}PMAT completed in ${PMAT_TIME}s${NC}"
echo ""

# Benchmark cargo-mutants
echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}Running cargo-mutants${NC}"
echo -e "${GREEN}================================${NC}"
MUTANTS_START=$(date +%s)

cargo mutants --file "$TEST_FILE" \
    --timeout 600 \
    --no-times \
    --output mutants_results.json 2>&1 | tee mutants_output.txt || true

MUTANTS_END=$(date +%s)
MUTANTS_TIME=$((MUTANTS_END - MUTANTS_START))

echo ""
echo -e "${GREEN}cargo-mutants completed in ${MUTANTS_TIME}s${NC}"
echo ""

# Parse results
echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}Results Comparison${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# PMAT results
if [ -f pmat_results.json ]; then
    PMAT_TOTAL=$(jq -r '.total_mutants // 0' pmat_results.json)
    PMAT_KILLED=$(jq -r '.killed // 0' pmat_results.json)
    PMAT_SURVIVED=$(jq -r '.survived // 0' pmat_results.json)
    PMAT_SCORE=$(jq -r '.mutation_score // 0' pmat_results.json)
    PMAT_SCORE_PCT=$(echo "$PMAT_SCORE * 100" | bc -l | xargs printf "%.2f")

    echo -e "${GREEN}PMAT Results:${NC}"
    echo "  Total mutants: $PMAT_TOTAL"
    echo "  Killed: $PMAT_KILLED"
    echo "  Survived: $PMAT_SURVIVED"
    echo "  Mutation score: ${PMAT_SCORE_PCT}%"
    echo "  Execution time: ${PMAT_TIME}s"
else
    echo -e "${RED}PMAT results file not found${NC}"
    PMAT_TOTAL=0
    PMAT_KILLED=0
    PMAT_SURVIVED=0
    PMAT_SCORE=0
fi

echo ""

# cargo-mutants results (parse from output since JSON format may differ)
MUTANTS_CAUGHT=$(grep -oP 'caught \K\d+' mutants_output.txt | head -1 || echo "0")
MUTANTS_MISSED=$(grep -oP 'missed \K\d+' mutants_output.txt | head -1 || echo "0")
MUTANTS_TOTAL=$((MUTANTS_CAUGHT + MUTANTS_MISSED))

if [ "$MUTANTS_TOTAL" -gt 0 ]; then
    MUTANTS_SCORE=$(echo "scale=4; $MUTANTS_CAUGHT / $MUTANTS_TOTAL" | bc)
    MUTANTS_SCORE_PCT=$(echo "$MUTANTS_SCORE * 100" | bc -l | xargs printf "%.2f")
else
    MUTANTS_SCORE=0
    MUTANTS_SCORE_PCT="0.00"
fi

echo -e "${GREEN}cargo-mutants Results:${NC}"
echo "  Total mutants: $MUTANTS_TOTAL"
echo "  Killed (caught): $MUTANTS_CAUGHT"
echo "  Survived (missed): $MUTANTS_MISSED"
echo "  Mutation score: ${MUTANTS_SCORE_PCT}%"
echo "  Execution time: ${MUTANTS_TIME}s"

echo ""
echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}Performance Comparison${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# Speed comparison
if [ "$MUTANTS_TIME" -gt 0 ]; then
    SPEEDUP=$(echo "scale=2; $MUTANTS_TIME / $PMAT_TIME" | bc)
    echo -e "Speed: PMAT is ${GREEN}${SPEEDUP}x${NC} vs cargo-mutants"
else
    echo -e "Speed: ${YELLOW}cargo-mutants did not complete${NC}"
fi

# Accuracy comparison
SCORE_DIFF=$(echo "scale=2; ($PMAT_SCORE - $MUTANTS_SCORE) * 100" | bc)
echo -e "Mutation score difference: ${SCORE_DIFF} percentage points"

if [ "$PMAT_TOTAL" -gt 0 ] && [ "$MUTANTS_TOTAL" -gt 0 ]; then
    COVERAGE=$(echo "scale=2; ($PMAT_TOTAL / $MUTANTS_TOTAL) * 100" | bc)
    echo -e "Mutant coverage: PMAT found ${COVERAGE}% of cargo-mutants mutants"
fi

echo ""
echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}Conclusion${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# Determine winner
if [ "$PMAT_TIME" -lt "$MUTANTS_TIME" ]; then
    echo -e "⚡ ${GREEN}PMAT is faster${NC} (${PMAT_TIME}s vs ${MUTANTS_TIME}s)"
else
    echo -e "⚡ ${YELLOW}cargo-mutants is faster${NC} (${MUTANTS_TIME}s vs ${PMAT_TIME}s)"
fi

# Compare accuracy (within 5% is "just as good")
SCORE_DIFF_ABS=$(echo "$SCORE_DIFF" | tr -d '-')
if (( $(echo "$SCORE_DIFF_ABS < 5" | bc -l) )); then
    echo -e "✅ ${GREEN}Mutation scores are equivalent${NC} (within 5%)"
else
    echo -e "⚠️  ${YELLOW}Mutation scores differ by ${SCORE_DIFF_ABS}%${NC}"
fi

echo ""
echo -e "${BLUE}Results saved to:${NC}"
echo "  - pmat_results.json"
echo "  - pmat_output.txt"
echo "  - mutants_output.txt"
