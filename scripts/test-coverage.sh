#!/bin/bash
# Selective LLVM-Cov instrumentation for stratified test architecture
# Optimized for fast feedback and precise coverage metrics

set -euo pipefail

COVERAGE_DIR="target/coverage"
BASELINE_COVERAGE="${BASELINE_COVERAGE:-baseline-coverage.json}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create coverage directory
mkdir -p "$COVERAGE_DIR"

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to check if a feature is enabled
has_feature() {
    cargo metadata --format-version 1 | jq -r '.packages[].features' | grep -q "$1" || return 1
}

# Clean previous coverage data
print_status "$YELLOW" "🧹 Cleaning previous coverage data..."
cargo llvm-cov clean

# Fast unit coverage (core development feedback)
print_status "$GREEN" "🚀 Running unit tests with coverage..."
cargo llvm-cov --test unit_core \
    --html --output-dir "$COVERAGE_DIR/unit" \
    --ignore-filename-regex="(demo|testing|tests)/" \
    --include-build-script \
    -- --test-threads=1

# Service integration coverage (if feature enabled)
if cargo test --test services_integration --no-run 2>/dev/null; then
    print_status "$GREEN" "🔧 Running service integration tests with coverage..."
    cargo llvm-cov --test services_integration \
        --features integration-tests \
        --html --output-dir "$COVERAGE_DIR/services" \
        --include="src/services/*" \
        --exclude="src/services/cache/*" \
        -- --test-threads=4
fi

# Protocol adapter coverage (if feature enabled)
if cargo test --test protocol_adapters --no-run 2>/dev/null; then
    print_status "$GREEN" "🌐 Running protocol adapter tests with coverage..."
    cargo llvm-cov --test protocol_adapters \
        --features integration-tests \
        --html --output-dir "$COVERAGE_DIR/protocols" \
        --include="src/unified_protocol/*" \
        -- --test-threads=2
fi

# Critical path coverage (high-complexity functions)
print_status "$GREEN" "🎯 Analyzing critical path coverage..."
cargo llvm-cov --test services_integration \
    --features integration-tests \
    --json --output-path "$COVERAGE_DIR/critical.json" \
    --include="src/services/deep_context.rs" \
    --include="src/services/dead_code_analyzer.rs" \
    --include="src/cli/handlers/utility_handlers.rs" \
    -- --nocapture || true

# Generate unified coverage report
print_status "$GREEN" "📊 Generating unified coverage report..."
cargo llvm-cov report --lcov --output-path "$COVERAGE_DIR/lcov.info"

# Generate JSON summary for analysis
cargo llvm-cov report --json --output-path "$COVERAGE_DIR/summary.json"

# Coverage quality gates check
print_status "$YELLOW" "🔍 Checking coverage quality gates..."

# Extract coverage percentage
COVERAGE_PCT=$(cargo llvm-cov report --json | jq '.data[0].totals.lines.percent')
COVERAGE_INT=${COVERAGE_PCT%.*}

# Check if coverage meets threshold
COVERAGE_THRESHOLD="${COVERAGE_THRESHOLD:-85}"
if [ "$COVERAGE_INT" -lt "$COVERAGE_THRESHOLD" ]; then
    print_status "$RED" "❌ Coverage ${COVERAGE_PCT}% is below threshold of ${COVERAGE_THRESHOLD}%"
    exit_code=1
else
    print_status "$GREEN" "✅ Coverage ${COVERAGE_PCT}% meets threshold of ${COVERAGE_THRESHOLD}%"
    exit_code=0
fi

# Generate coverage diff for PR validation (CI only)
if [[ -n "${CI:-}" ]]; then
    print_status "$YELLOW" "📈 Generating coverage diff for CI..."
    
    if [ -f "$BASELINE_COVERAGE" ]; then
        # Create a simple coverage diff report
        python3 - <<EOF
import json
import sys

with open('$BASELINE_COVERAGE', 'r') as f:
    baseline = json.load(f)
    
with open('$COVERAGE_DIR/summary.json', 'r') as f:
    current = json.load(f)

baseline_pct = baseline['data'][0]['totals']['lines']['percent']
current_pct = current['data'][0]['totals']['lines']['percent']
diff = current_pct - baseline_pct

if diff < -2.0:
    print(f"❌ Coverage regression: {baseline_pct:.2f}% → {current_pct:.2f}% ({diff:+.2f}%)")
    sys.exit(1)
else:
    print(f"✅ Coverage change: {baseline_pct:.2f}% → {current_pct:.2f}% ({diff:+.2f}%)")
EOF
    fi
fi

# Generate HTML report summary
print_status "$GREEN" "📄 Generating HTML report index..."
cat > "$COVERAGE_DIR/index.html" <<EOF
<!DOCTYPE html>
<html>
<head>
    <title>Coverage Report Summary</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #333; }
        .report-link { 
            display: inline-block; 
            margin: 10px; 
            padding: 10px 20px; 
            background: #007bff; 
            color: white; 
            text-decoration: none; 
            border-radius: 5px; 
        }
        .report-link:hover { background: #0056b3; }
        .coverage-summary {
            background: #f8f9fa;
            padding: 20px;
            border-radius: 5px;
            margin: 20px 0;
        }
        .metric { 
            display: inline-block; 
            margin: 10px 20px; 
        }
        .metric-value { 
            font-size: 24px; 
            font-weight: bold; 
            color: #28a745; 
        }
    </style>
</head>
<body>
    <h1>Stratified Test Coverage Report</h1>
    
    <div class="coverage-summary">
        <h2>Overall Coverage</h2>
        <div class="metric">
            <div class="metric-value">${COVERAGE_PCT}%</div>
            <div>Line Coverage</div>
        </div>
    </div>
    
    <h2>Detailed Reports</h2>
    <a href="unit/index.html" class="report-link">Unit Tests</a>
    <a href="services/index.html" class="report-link">Service Integration</a>
    <a href="protocols/index.html" class="report-link">Protocol Adapters</a>
    
    <h2>Test Execution Times</h2>
    <ul>
        <li>Unit Tests: &lt; 10s</li>
        <li>Service Integration: &lt; 30s</li>
        <li>Protocol Tests: &lt; 45s</li>
    </ul>
</body>
</html>
EOF

# Print summary
print_status "$GREEN" "📋 Coverage Summary:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Overall Coverage: ${COVERAGE_PCT}%"
echo "Report Location: $COVERAGE_DIR/index.html"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Open coverage report in browser if not in CI
if [[ -z "${CI:-}" ]] && command -v xdg-open &> /dev/null; then
    print_status "$YELLOW" "🌐 Opening coverage report in browser..."
    xdg-open "$COVERAGE_DIR/index.html" || true
elif [[ -z "${CI:-}" ]] && command -v open &> /dev/null; then
    print_status "$YELLOW" "🌐 Opening coverage report in browser..."
    open "$COVERAGE_DIR/index.html" || true
fi

exit $exit_code