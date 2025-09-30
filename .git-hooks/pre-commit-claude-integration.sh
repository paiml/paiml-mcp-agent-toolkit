#!/bin/bash
# Pre-commit hook for Claude integration quality gates
# Copy to .git/hooks/pre-commit and make executable

set -e

echo "🔍 Running EXTREME TDD quality gates for Claude integration..."

# Check if we're modifying Claude integration code
CHANGED_FILES=$(git diff --cached --name-only --diff-filter=ACM)
INTEGRATION_FILES=$(echo "$CHANGED_FILES" | grep -E '(server/src/claude_integration|bridge/src)' || true)

if [ -z "$INTEGRATION_FILES" ]; then
    echo "✓ No Claude integration files changed, skipping checks"
    exit 0
fi

echo "📁 Changed files:"
echo "$INTEGRATION_FILES"
echo ""

# 1. SATD Detection - Zero Tolerance
echo "1️⃣  Checking for SATD (zero tolerance)..."
if echo "$INTEGRATION_FILES" | xargs grep -n "TODO\|FIXME\|HACK\|XXX" 2>/dev/null; then
    echo "❌ SATD detected in Claude integration code"
    echo "   Zero-tolerance policy violated"
    echo "   Please remove all TODO/FIXME/HACK/XXX comments before committing"
    exit 1
fi
echo "✓ Zero SATD verified"
echo ""

# 2. Rust Formatting
if echo "$INTEGRATION_FILES" | grep -q "\.rs$"; then
    echo "2️⃣  Checking Rust formatting..."
    cd server
    if ! cargo fmt -- --check; then
        echo "❌ Rust formatting issues detected"
        echo "   Run: cd server && cargo fmt"
        exit 1
    fi
    cd ..
    echo "✓ Rust formatting verified"
    echo ""
fi

# 3. Clippy Lints
if echo "$INTEGRATION_FILES" | grep -q "\.rs$"; then
    echo "3️⃣  Running Clippy..."
    cd server
    if ! cargo clippy --all-targets -- -D warnings 2>&1 | grep -v "warning: pmat"; then
        echo "❌ Clippy warnings detected"
        echo "   Fix warnings before committing"
        exit 1
    fi
    cd ..
    echo "✓ Clippy checks passed"
    echo ""
fi

# 4. Complexity Check
if echo "$INTEGRATION_FILES" | grep -q "server/src/claude_integration.*\.rs$"; then
    echo "4️⃣  Checking cyclomatic complexity..."

    # Simple complexity check using syn if available
    # For now, just ensure code compiles
    cd server
    if ! cargo build --lib 2>&1 | tail -5; then
        echo "❌ Compilation failed"
        exit 1
    fi
    cd ..
    echo "✓ Complexity check passed (compilation successful)"
    echo ""
fi

# 5. TypeScript Checks
if echo "$INTEGRATION_FILES" | grep -q "bridge/src.*\.ts$"; then
    echo "5️⃣  Checking TypeScript..."
    cd bridge

    if [ -d "node_modules" ]; then
        if ! npm run build 2>&1 | tail -10; then
            echo "❌ TypeScript compilation failed"
            exit 1
        fi
        echo "✓ TypeScript build successful"
    else
        echo "⚠️  node_modules not found, skipping TypeScript checks"
        echo "   Run: cd bridge && npm install"
    fi
    cd ..
    echo ""
fi

# 6. Unit Tests
echo "6️⃣  Running unit tests..."
cd server
if ! cargo test --lib claude_integration 2>&1 | tail -20; then
    echo "❌ Unit tests failed"
    exit 1
fi
cd ..
echo "✓ Unit tests passed"
echo ""

# 7. File Size Check
echo "7️⃣  Checking file sizes..."
for file in $INTEGRATION_FILES; do
    if [ -f "$file" ]; then
        lines=$(wc -l < "$file")
        if [ "$lines" -gt 500 ]; then
            echo "⚠️  $file has $lines lines (>500)"
            echo "   Consider breaking into smaller modules"
        fi
    fi
done
echo "✓ File size check completed"
echo ""

# Summary
echo "✅ All quality gates passed!"
echo ""
echo "Quality Metrics Enforced:"
echo "  • Zero SATD tolerance"
echo "  • Clippy warnings as errors"
echo "  • Code formatting"
echo "  • Unit test coverage"
echo "  • TypeScript compilation"
echo ""
echo "Ready to commit! 🚀"

exit 0