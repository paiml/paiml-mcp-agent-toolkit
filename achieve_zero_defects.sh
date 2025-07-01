#!/bin/bash
# Script to systematically achieve zero defects

set -e

echo "🎯 ZERO DEFECTS ACHIEVEMENT SCRIPT"
echo "=================================="

# Step 1: Fix all lint warnings
echo -e "\n📝 Step 1: Fixing lint warnings..."
cd server
cargo fix --allow-dirty --allow-staged || true
cd ..

# Step 2: Remove or document SATD
echo -e "\n🧹 Step 2: Finding SATD items..."
./target/release/pmat analyze satd --format json > satd_report.json
SATD_COUNT=$(jq '.summary.total_satd_items // 0' satd_report.json)
echo "Found $SATD_COUNT SATD items"

# Step 3: Check complexity violations
echo -e "\n🔍 Step 3: Checking complexity..."
./target/release/pmat analyze complexity --max-cyclomatic 10 --format json > complexity_report.json
HIGH_COMPLEXITY=$(jq '[.violations[] | select(.cyclomatic > 20)] | length' complexity_report.json)
echo "Found $HIGH_COMPLEXITY functions with complexity > 20"

# Step 4: Run automated refactoring
echo -e "\n🤖 Step 4: Running automated refactoring..."
if [ "$SATD_COUNT" -gt "0" ] || [ "$HIGH_COMPLEXITY" -gt "0" ]; then
    echo "Running pmat refactor auto to fix issues..."
    ./target/release/pmat refactor auto \
        --max-iterations 1 \
        --format summary \
        --checkpoint .refactor_checkpoints/zero_defects.json || true
else
    echo "✅ No major defects found!"
fi

# Step 5: Final quality check
echo -e "\n📊 Step 5: Final quality check..."
echo "=== Current Status ==="
echo -n "SATD items: "
./target/release/pmat analyze satd --format json 2>/dev/null | jq '.summary.total_satd_items // 0'
echo -n "High complexity functions: "
./target/release/pmat analyze complexity --max-cyclomatic 20 --format json 2>/dev/null | jq '[.violations[] | select(.cyclomatic > 20)] | length'
echo -n "Lint warnings: "
make lint 2>&1 | grep -c "warning:" || echo "0"

echo -e "\n✅ Zero defects script complete!"
echo "Next steps:"
echo "1. Review the refactor auto output if any"
echo "2. Manually fix remaining issues"
echo "3. Run 'make test' to ensure everything works"