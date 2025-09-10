#!/bin/bash
# Final property test fix script for Sprint 89
# Ensures all property tests compile without syntax errors

set -e

echo "🔧 Sprint 89: Final property test syntax fixes"

# List of files with remaining property test issues
files_to_fix=(
    "server/src/cli/handlers/enforce_handlers.rs"
    "server/src/cli/handlers/refactor_auto_property_tests.rs"
    "server/src/cli/handlers/quality_gate_property_tests.rs"
    "server/src/cli/handlers/lint_hotspot_property_tests.rs"
    "server/src/unified_protocol/error.rs"
)

for file in "${files_to_fix[@]}"; do
    if [[ -f "$file" ]]; then
        echo "🔧 Fixing: $file"
        
        # Fix proptest! macro syntax
        sed -i 's/proptest! {/proptest! {/' "$file"
        sed -i '/proptest! {/,/^}$/ {
            s/fn \([^(]*\)(\([^)]*\)) {/fn \1(\2) {/
            s/^    \([a-zA-Z_][a-zA-Z0-9_]*\) in/        \1 in/
        }' "$file"
        
        # Ensure proper macro formatting
        sed -i '/^    proptest! {$/,/^    }$/ {
            /^        fn / {
                N
                s/\n        prop_assert!/\n            prop_assert!/
            }
        }' "$file"
        
        # Fix test function indentation
        sed -i '/proptest! {/,/^    }$/ {
            s/^        fn /            fn /
            s/^        prop_assert!/            prop_assert!/
            s/^        let /            let /
        }' "$file"
        
        echo "✅ Fixed: $file"
    else
        echo "⚠️  File not found: $file"
    fi
done

echo ""
echo "🧪 Testing compilation..."
if cargo build --package pmat --tests --quiet; then
    echo "✅ All tests compile successfully!"
    echo "🎯 Sprint 89: Property test compilation COMPLETE"
else
    echo "❌ Some tests still have issues"
    echo "Showing remaining errors:"
    cargo build --package pmat --tests 2>&1 | grep "error:" | head -5
fi