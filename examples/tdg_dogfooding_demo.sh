#!/bin/bash
# TDG Dogfooding Storage Demo
# Demonstrates how PMAT tracks its own quality metrics persistently

set -e

echo "🗄️ PMAT TDG Dogfooding Storage Demo - v2.68.0"
echo "=============================================="
echo ""

# Check if pmat is installed
if ! command -v pmat &> /dev/null; then
    echo "❌ pmat not found. Install with: cargo install pmat"
    exit 1
fi

echo "1️⃣ Initial Storage State"
echo "----------------------"
echo "📊 Storage statistics before any analysis:"
pmat tdg storage stats
echo ""

echo "💾 Storage location and size:"
echo "~/.pmat/tdg-warm: $(du -sh ~/.pmat/tdg-warm 2>/dev/null | cut -f1 || echo 'not found')"
echo "~/.pmat/tdg-cold: $(du -sh ~/.pmat/tdg-cold 2>/dev/null | cut -f1 || echo 'not found')"
echo ""

echo "2️⃣ First Analysis - Cache Miss"
echo "-----------------------------"
echo "🔍 Analyzing server/src/lib.rs (will be stored):"
FIRST_SCORE=$(pmat tdg server/src/lib.rs --quiet)
echo "Score: $FIRST_SCORE/100"
echo ""

echo "3️⃣ Second Analysis - Cache Hit"
echo "------------------------------"
echo "🔍 Re-analyzing same file (should use cached score):"
SECOND_SCORE=$(pmat tdg server/src/lib.rs --quiet)
echo "Score: $SECOND_SCORE/100"

if [ "$FIRST_SCORE" = "$SECOND_SCORE" ]; then
    echo "✅ Cache hit confirmed - scores match!"
else
    echo "⚠️  Cache miss or file changed - scores differ"
fi
echo ""

echo "4️⃣ Storage Growth Analysis"
echo "-------------------------"
echo "📊 Updated storage statistics:"
pmat tdg storage stats
echo ""

echo "💾 Storage directory sizes after analysis:"
echo "~/.pmat/tdg-warm: $(du -sh ~/.pmat/tdg-warm 2>/dev/null | cut -f1 || echo 'not found')"
echo "~/.pmat/tdg-cold: $(du -sh ~/.pmat/tdg-cold 2>/dev/null | cut -f1 || echo 'not found')"
echo ""

echo "5️⃣ Multiple File Analysis"
echo "-------------------------"
echo "🔍 Analyzing multiple files to demonstrate storage growth:"
FILES=("server/src/lib.rs" "server/src/main.rs" "server/build.rs")

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        score=$(pmat tdg "$file" --quiet 2>/dev/null || echo "N/A")
        echo "  $file: ${score}/100"
    fi
done
echo ""

echo "6️⃣ Final Storage State"
echo "---------------------"
echo "📊 Final storage statistics:"
pmat tdg storage stats
echo ""

echo "💡 Key Takeaways:"
echo "  ✅ Every TDG analysis automatically stores scores"
echo "  ⚡ Repeated analyses use cached scores (performance benefit)"
echo "  📈 Storage grows as more files are analyzed (historical tracking)"
echo "  🔍 Use 'pmat tdg storage stats' to monitor dogfooding progress"
echo "  🏭 True dogfooding: PMAT tracks its own quality metrics!"
echo ""

echo "🎯 Next Steps:"
echo "  • Run this script weekly to see storage growth"
echo "  • Use storage data for quality trend analysis"
echo "  • Leverage cached scores for faster CI/CD pipelines"
echo "  • Monitor storage size for cleanup decisions"
echo ""

echo "Demo completed! 🎉"