#!/usr/bin/env bash
# CI script to check for dependency duplicates
# GH-89: Prevents regression of duplicate dependency count
#
# Usage: ./scripts/check_dependency_duplicates.sh
# Exit codes: 0 = pass, 1 = too many duplicates

set -euo pipefail

# Maximum allowed unique duplicate packages
MAX_DUPLICATES=28

cd "$(dirname "$0")/../server" || exit 1

echo "🔍 Checking dependency duplicates..."

# Count unique duplicate packages
DUPLICATE_COUNT=$(cargo tree -d 2>/dev/null | grep -E "^[a-z]" | sed 's/ v.*//' | sort -u | wc -l)

echo "📊 Found $DUPLICATE_COUNT unique duplicate packages (max allowed: $MAX_DUPLICATES)"

if [ "$DUPLICATE_COUNT" -gt "$MAX_DUPLICATES" ]; then
    echo "❌ ERROR: Too many duplicate packages!"
    echo ""
    echo "Duplicates found:"
    cargo tree -d 2>/dev/null | grep -E "^[a-z]" | sed 's/ v.*//' | sort -u
    echo ""
    echo "To fix:"
    echo "  1. Update dependencies in Cargo.toml"
    echo "  2. Run 'cargo update' to resolve versions"
    echo "  3. Or document new duplicates in tests/dependency_duplicates_test.rs"
    exit 1
fi

echo "✅ Dependency duplicate check passed"
exit 0
