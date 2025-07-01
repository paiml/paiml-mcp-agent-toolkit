#!/bin/bash
set -e

echo "🎌 KAIZEN LINT FIX - Toyota Way Zero Defects"
echo "============================================"
echo ""
echo "This script systematically fixes the largest collections of lint issues."
echo "Following the principle of continuous improvement (Kaizen)."
echo ""

# Function to count remaining lint issues
count_lint_issues() {
    make lint 2>&1 | grep -E "error:|warning:" | wc -l || echo "0"
}

# Initial count
INITIAL_COUNT=$(count_lint_issues)
echo "📊 Initial lint issues: $INITIAL_COUNT"
echo ""

# Step 1: Fix missing panic documentation (1265 instances)
echo "Step 1: Fixing missing panic documentation..."
echo "----------------------------------------------"
chmod +x fix-panic-docs.ts
./fix-panic-docs.ts
echo ""

# Step 2: Fix missing error documentation (745 instances)
echo "Step 2: Fixing missing error documentation..."
echo "----------------------------------------------"
chmod +x fix-error-docs.ts
./fix-error-docs.ts
echo ""

# Step 3: Fix format string appends (339 instances)
echo "Step 3: Fixing format string appends..."
echo "----------------------------------------"
chmod +x fix-format-string-append.ts
./fix-format-string-append.ts
echo ""

# Step 4: Run cargo fmt to ensure consistent formatting
echo "Step 4: Running cargo fmt..."
echo "-----------------------------"
cd server && cargo fmt && cd ..
echo "✓ Code formatted"
echo ""

# Step 5: Try cargo clippy fix for remaining issues
echo "Step 5: Running cargo clippy --fix for safe fixes..."
echo "-----------------------------------------------------"
cd server
cargo clippy --fix --allow-dirty --allow-staged -- \
    -W clippy::uninlined_format_args \
    -W clippy::redundant_field_names \
    -W clippy::use_self \
    -W clippy::redundant_closure \
    -W clippy::single_char_pattern || true
cd ..
echo "✓ Clippy fixes applied"
echo ""

# Final count
FINAL_COUNT=$(count_lint_issues)
FIXED_COUNT=$((INITIAL_COUNT - FINAL_COUNT))

echo "📊 KAIZEN RESULTS:"
echo "=================="
echo "  - Initial issues: $INITIAL_COUNT"
echo "  - Fixed issues: $FIXED_COUNT"
echo "  - Remaining issues: $FINAL_COUNT"
echo "  - Improvement: $(( (FIXED_COUNT * 100) / INITIAL_COUNT ))%"
echo ""

# Show breakdown of remaining issues
if [ "$FINAL_COUNT" -gt 0 ]; then
    echo "📋 Top 10 remaining issue types:"
    echo "---------------------------------"
    make lint 2>&1 | grep -E "error:|warning:" | sed -E 's/.*: (.*)/\1/' | sort | uniq -c | sort -rn | head -10
    echo ""
    echo "These issues require more specific fixes."
fi

echo "✅ Kaizen lint fix process complete!"
echo ""
echo "Next steps:"
echo "1. Review changes: git diff"
echo "2. Run tests: make test-fast"
echo "3. Continue fixing remaining issues"