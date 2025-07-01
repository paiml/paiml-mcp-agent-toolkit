#!/bin/bash
set -e

echo "🚀 MASTER LINT FIXER FOR PAIML-MCP-AGENT-TOOLKIT"
echo "================================================"
echo ""
echo "This script will systematically fix all 5744 lint errors."
echo "Following the Toyota Way principle of 'Zero Defects'."
echo ""
echo "Press Ctrl+C to cancel, or Enter to continue..."
read -r

# Step 1: Create a git checkpoint
echo "Step 1: Creating git checkpoint..."
if git diff --quiet && git diff --staged --quiet; then
    echo "  Working directory is clean"
else
    echo "  Creating WIP commit for safety..."
    git add -A
    git commit -m "WIP: Before comprehensive lint fixes" || true
fi
echo ""

# Step 2: Run the comprehensive bash fixes
echo "Step 2: Running comprehensive automated fixes..."
./fix-all-comprehensive.sh
echo ""

# Step 3: Run the intelligent documentation fixer
echo "Step 3: Adding intelligent documentation..."
deno run --allow-read --allow-write fix-docs-intelligent.ts
echo ""

# Step 4: Run any remaining specific fixes
echo "Step 4: Running remaining specific fixes..."
deno run --allow-read --allow-write fix-all-remaining-lint.ts
echo ""

# Step 5: Final formatting pass
echo "Step 5: Final formatting..."
cd server && cargo fmt && cd ..
echo ""

# Step 6: Show results
echo "Step 6: Analyzing results..."
echo "=============================="

# Try to run make lint and count issues
REMAINING=$(make lint 2>&1 | grep -E "error:|warning:" | wc -l || echo "0")

echo ""
echo "📊 FINAL RESULTS:"
echo "  - Started with: 5744 issues"
echo "  - Remaining: $REMAINING issues"
echo "  - Fixed: $((5744 - REMAINING)) issues"
echo "  - Success rate: $(( (5744 - REMAINING) * 100 / 5744 ))%"
echo ""

if [ "$REMAINING" -eq 0 ]; then
    echo "🎉 SUCCESS! All lint issues have been fixed!"
    echo ""
    echo "Next steps:"
    echo "1. Review changes: git diff"
    echo "2. Run tests: make test-fast"
    echo "3. Commit changes: git add -A && git commit -m 'fix: resolve all 5744 lint issues'"
else
    echo "⚠️  There are still $REMAINING issues remaining."
    echo ""
    echo "Top remaining issues:"
    make lint 2>&1 | grep -E "error:|warning:" | sed -E 's/.*: (.*)/\1/' | sort | uniq -c | sort -rn | head -10 || true
    echo ""
    echo "These likely require manual intervention."
    echo "Run 'make lint' to see the full list."
fi

echo ""
echo "✅ Lint fixing process complete!"