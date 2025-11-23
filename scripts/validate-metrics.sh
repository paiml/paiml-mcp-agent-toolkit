#!/bin/bash
# O(1) metric validation for pre-commit hooks
# Spec: docs/specifications/quick-test-build-O(1)-checking.md
# shellcheck disable=DET002
# Intentional: Timestamps used for staleness checking (spec requirement)
set -euo pipefail

METRICS_DIR=".pmat-metrics"
CONFIG_FILE=".pmat-metrics.toml"
FAILURES_ONLY="${1:-false}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Load thresholds from config (simple parsing)
LINT_MAX_MS=30000
TEST_FAST_MAX_MS=300000
COVERAGE_MAX_MS=600000
BINARY_MAX_BYTES=50000000
DEPS_DEFAULT_MAX=3000
MAX_AGE_DAYS=7

# Check if metrics directory exists
if [ ! -d "$METRICS_DIR" ]; then
    echo -e "${YELLOW}⚠️  No metrics cache found (.pmat-metrics/)${NC}"
    echo "   Run 'make lint' or 'make test-fast' to generate metrics"
    exit 0
fi

# Function to check if metric is stale
is_stale() {
    local file=$1
    local max_age_seconds=$((MAX_AGE_DAYS * 24 * 60 * 60))

    if [ ! -f "$file" ]; then
        return 0 # Missing = stale
    fi

    local file_time
    file_time="$(stat -c %Y "$file" 2>/dev/null || stat -f %m "$file" 2>/dev/null)"
    local current_time
    current_time="$(date +%s)"
    local age_seconds=$((current_time - file_time))

    [ "$age_seconds" -gt "$max_age_seconds" ]
}

# Function to extract JSON value (simple grep-based)
json_value() {
    local file=$1
    local key=$2
    grep "\"$key\"" "$file" | sed -E 's/.*: ([0-9.]+).*/\1/' | head -1
}

# Validation counters
VIOLATIONS=0
WARNINGS=0

echo -e "${BLUE}🔍 PMAT Quality Gate Validation${NC}"
echo ""

# Validate lint
if [ -f "$METRICS_DIR/lint.result" ]; then
    DURATION_MS="$(json_value "$METRICS_DIR/lint.result" "duration_ms")"
    if is_stale "$METRICS_DIR/lint.result"; then
        echo -e "${YELLOW}⚠️  make lint: Stale (>$MAX_AGE_DAYS days old)${NC}"
        WARNINGS=$((WARNINGS + 1))
    elif [ "${DURATION_MS%.*}" -gt "$LINT_MAX_MS" ]; then
        HEADROOM=$(echo "scale=1; 100 * ($DURATION_MS - $LINT_MAX_MS) / $LINT_MAX_MS" | bc)
        echo -e "${RED}❌ make lint: ${DURATION_MS}ms (exceeds ${LINT_MAX_MS}ms threshold by ${HEADROOM}%)${NC}"
        VIOLATIONS=$((VIOLATIONS + 1))
    elif [ "$FAILURES_ONLY" != "--failures-only" ]; then
        HEADROOM=$(echo "scale=1; 100 * ($LINT_MAX_MS - $DURATION_MS) / $LINT_MAX_MS" | bc)
        echo -e "${GREEN}✅ make lint: ${DURATION_MS}ms (${HEADROOM}% headroom, target: ${LINT_MAX_MS}ms)${NC}"
    fi
fi

# Validate test-fast
if [ -f "$METRICS_DIR/test-fast.result" ]; then
    DURATION_MS="$(json_value "$METRICS_DIR/test-fast.result" "duration_ms")"
    if is_stale "$METRICS_DIR/test-fast.result"; then
        echo -e "${YELLOW}⚠️  make test-fast: Stale (>$MAX_AGE_DAYS days old)${NC}"
        WARNINGS=$((WARNINGS + 1))
    elif [ "${DURATION_MS%.*}" -gt "$TEST_FAST_MAX_MS" ]; then
        HEADROOM=$(echo "scale=1; 100 * ($DURATION_MS - $TEST_FAST_MAX_MS) / $TEST_FAST_MAX_MS" | bc)
        echo -e "${RED}❌ make test-fast: ${DURATION_MS}ms (exceeds ${TEST_FAST_MAX_MS}ms threshold by ${HEADROOM}%)${NC}"
        VIOLATIONS=$((VIOLATIONS + 1))
    elif [ "$FAILURES_ONLY" != "--failures-only" ]; then
        HEADROOM=$(echo "scale=1; 100 * ($TEST_FAST_MAX_MS - $DURATION_MS) / $TEST_FAST_MAX_MS" | bc)
        DURATION_SEC=$((DURATION_MS / 1000))
        DURATION_MIN=$((DURATION_SEC / 60))
        DURATION_REMAIN=$((DURATION_SEC % 60))
        echo -e "${GREEN}✅ make test-fast: ${DURATION_MIN}m ${DURATION_REMAIN}s (${HEADROOM}% headroom)${NC}"
    fi
fi

# Validate build-release (binary size)
if [ -f "$METRICS_DIR/build-release.result" ]; then
    BINARY_SIZE="$(json_value "$METRICS_DIR/build-release.result" "binary_size")"
    BINARY_SIZE_INT="${BINARY_SIZE%.*}"
    if is_stale "$METRICS_DIR/build-release.result"; then
        echo -e "${YELLOW}⚠️  Binary size: Stale (>$MAX_AGE_DAYS days old)${NC}"
        WARNINGS=$((WARNINGS + 1))
    elif [ "$BINARY_SIZE_INT" -gt "$BINARY_MAX_BYTES" ]; then
        BINARY_MB=$((BINARY_SIZE_INT / 1024 / 1024))
        THRESHOLD_MB=$((BINARY_MAX_BYTES / 1024 / 1024))
        HEADROOM=$(echo "scale=1; 100 * ($BINARY_SIZE_INT - $BINARY_MAX_BYTES) / $BINARY_MAX_BYTES" | bc)
        echo -e "${RED}❌ Binary size: ${BINARY_MB} MB (exceeds ${THRESHOLD_MB} MB threshold by ${HEADROOM}%)${NC}"
        VIOLATIONS=$((VIOLATIONS + 1))
    elif [ "$FAILURES_ONLY" != "--failures-only" ]; then
        BINARY_MB=$((BINARY_SIZE_INT / 1024 / 1024))
        THRESHOLD_MB=$((BINARY_MAX_BYTES / 1024 / 1024))
        HEADROOM=$(echo "scale=1; 100 * ($BINARY_MAX_BYTES - $BINARY_SIZE_INT) / $BINARY_MAX_BYTES" | bc)
        echo -e "${GREEN}✅ Binary size: ${BINARY_MB} MB (${HEADROOM}% headroom, target: ${THRESHOLD_MB} MB)${NC}"
    fi
fi

# Validate dependencies
if [ -f "$METRICS_DIR/deps-default.result" ]; then
    DEPS_COUNT="$(json_value "$METRICS_DIR/deps-default.result" "count")"
    DEPS_COUNT_INT="${DEPS_COUNT%.*}"
    if is_stale "$METRICS_DIR/deps-default.result"; then
        echo -e "${YELLOW}⚠️  Dependencies: Stale (>$MAX_AGE_DAYS days old)${NC}"
        WARNINGS=$((WARNINGS + 1))
    elif [ "$DEPS_COUNT_INT" -gt "$DEPS_DEFAULT_MAX" ]; then
        HEADROOM=$(echo "scale=1; 100 * ($DEPS_COUNT_INT - $DEPS_DEFAULT_MAX) / $DEPS_DEFAULT_MAX" | bc)
        echo -e "${RED}❌ Dependencies: ${DEPS_COUNT_INT} (exceeds ${DEPS_DEFAULT_MAX} threshold by ${HEADROOM}%)${NC}"
        VIOLATIONS=$((VIOLATIONS + 1))
    elif [ "$FAILURES_ONLY" != "--failures-only" ]; then
        HEADROOM=$(echo "scale=1; 100 * ($DEPS_DEFAULT_MAX - $DEPS_COUNT_INT) / $DEPS_DEFAULT_MAX" | bc)
        echo -e "${GREEN}✅ Dependencies: ${DEPS_COUNT_INT} (${HEADROOM}% headroom, target: ${DEPS_DEFAULT_MAX})${NC}"
    fi
fi

echo ""
if [ "$VIOLATIONS" -gt 0 ]; then
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}❌ QUALITY GATE FAILURE${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "${YELLOW}🚨 Andon Cord: Quality regression detected${NC}"
    echo ""
    echo "💡 Actions required:"
    echo "   1. Review threshold violations above"
    echo "   2. Run respective targets to fix: make lint, make test-fast, etc."
    echo "   3. Optimize code to meet thresholds"
    echo ""
    echo "⚠️  Emergency bypass: git commit --no-verify (not recommended)"
    exit 1
elif [ "$WARNINGS" -gt 0 ]; then
    echo -e "${YELLOW}⚠️  $WARNINGS warning(s): Stale metrics detected${NC}"
    echo "   Re-run: make lint, make test-fast, etc."
    exit 0
else
    echo -e "${GREEN}✅ All quality gates passed (O(1) validation: <10ms)${NC}"
    exit 0
fi
