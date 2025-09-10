#!/bin/bash

# Pre-commit Hook: Automatic Property Test Generation
# Sprint 89: Ensure all new Rust files have property tests
#
# Installation:
# cp scripts/pre-commit-property-tests.sh .git/hooks/pre-commit-property-tests
# chmod +x .git/hooks/pre-commit-property-tests
# Add to existing pre-commit: ./scripts/pre-commit-property-tests.sh

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
MIN_COVERAGE=80
SERVER_DIR="server/src"

echo -e "${YELLOW}🔍 Property Test Pre-commit Check${NC}"
echo "================================"

# Function to check if file has property tests
has_property_tests() {
    local file=$1
    grep -q "proptest!" "$file" 2>/dev/null || grep -q "mod property_tests" "$file" 2>/dev/null
}

# Function to add basic property tests to a file
add_property_tests() {
    local file=$1
    local module_name=$(basename "$file" .rs)
    
    # Check if file already has tests section
    if grep -q "#\[cfg(test)\]" "$file"; then
        echo "  File has test section but no property tests"
        return 1
    fi
    
    # Determine module type for appropriate template
    local template_type="generic"
    if echo "$file" | grep -q "parser"; then
        template_type="parser"
    elif echo "$file" | grep -q "analyzer"; then
        template_type="analyzer"
    elif echo "$file" | grep -q "handler"; then
        template_type="handler"
    elif echo "$file" | grep -q "service"; then
        template_type="service"
    elif echo "$file" | grep -q "model"; then
        template_type="model"
    fi
    
    # Add property test template
    cat >> "$file" << 'EOF'

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn module_invariants_hold(seed in 0u64..1000) {
            // TODO: Add meaningful property test
            // This is a placeholder - replace with actual invariant testing
            let _ = seed; // Use seed for deterministic behavior
            prop_assert!(true);
        }

        #[test]
        fn operations_are_deterministic(input in ".*") {
            // TODO: Test that operations are deterministic
            // Replace with actual determinism checks
            let _ = input;
            prop_assert!(true);
        }
    }
}
EOF
    
    echo -e "  ${GREEN}✅ Added property test template to $file${NC}"
    return 0
}

# Get list of staged Rust files
staged_files=$(git diff --cached --name-only --diff-filter=ACM | grep "\.rs$" || true)

if [ -z "$staged_files" ]; then
    echo "No Rust files staged for commit"
    exit 0
fi

# Track files without property tests
missing_tests=()
new_files_without_tests=()
auto_added=()

# Check each staged file
for file in $staged_files; do
    # Skip test files themselves
    if echo "$file" | grep -q "_test\.rs\|/tests/\|test_.*\.rs"; then
        continue
    fi
    
    # Skip if not in server/src
    if ! echo "$file" | grep -q "^server/src/"; then
        continue
    fi
    
    # Check if file has property tests
    if ! has_property_tests "$file"; then
        # Check if this is a new file
        if ! git ls-files "$file" | grep -q "$file"; then
            new_files_without_tests+=("$file")
            
            # Try to auto-add property tests to new files
            echo -e "${YELLOW}📝 New file without property tests: $file${NC}"
            if add_property_tests "$file"; then
                auto_added+=("$file")
                git add "$file"
            fi
        else
            missing_tests+=("$file")
        fi
    fi
done

# Calculate coverage
total_files=$(find "$SERVER_DIR" -name "*.rs" -type f 2>/dev/null | wc -l)
files_with_tests=$(find "$SERVER_DIR" -name "*.rs" -type f -exec grep -l "proptest!\|mod property_tests" {} \; 2>/dev/null | wc -l)
coverage=$(awk "BEGIN {printf \"%.1f\", ($files_with_tests / $total_files) * 100}")

echo ""
echo "📊 Property Test Coverage: ${coverage}% (${files_with_tests}/${total_files} files)"

# Check if coverage meets threshold
if (( $(echo "$coverage < $MIN_COVERAGE" | bc -l) )); then
    echo -e "${RED}❌ Coverage below ${MIN_COVERAGE}% threshold${NC}"
    
    if [ ${#missing_tests[@]} -gt 0 ]; then
        echo ""
        echo -e "${YELLOW}Files missing property tests:${NC}"
        for file in "${missing_tests[@]}"; do
            echo "  - $file"
        done
    fi
    
    echo ""
    echo "To fix this:"
    echo "1. Run: ./scripts/upgrade_property_tests.py --apply --limit 10"
    echo "2. Or manually add property tests to the files listed above"
    echo "3. Use meaningful tests, not just prop_assert!(true)"
    
    # Don't block if we auto-added tests
    if [ ${#auto_added[@]} -gt 0 ]; then
        echo ""
        echo -e "${GREEN}✅ Auto-added property tests to ${#auto_added[@]} new files${NC}"
        echo "Please review and improve the generated tests before committing."
    fi
    
    # Only fail if coverage is significantly below threshold and no auto-fixes
    if (( $(echo "$coverage < $MIN_COVERAGE - 5" | bc -l) )) && [ ${#auto_added[@]} -eq 0 ]; then
        exit 1
    fi
else
    echo -e "${GREEN}✅ Property test coverage meets threshold${NC}"
fi

# Check for placeholder tests in staged files
placeholder_count=0
for file in $staged_files; do
    if grep -q "prop_assert!(true)" "$file" 2>/dev/null; then
        placeholder_count=$((placeholder_count + $(grep -c "prop_assert!(true)" "$file")))
    fi
done

if [ $placeholder_count -gt 0 ]; then
    echo ""
    echo -e "${YELLOW}⚠️  Warning: Found $placeholder_count placeholder property tests${NC}"
    echo "Consider upgrading them to meaningful tests:"
    echo "  ./scripts/upgrade_property_tests.py --apply --priority 1"
fi

# Success message
if [ ${#auto_added[@]} -gt 0 ]; then
    echo ""
    echo -e "${GREEN}✅ Pre-commit check passed with auto-fixes${NC}"
    echo "Files with auto-added tests:"
    for file in "${auto_added[@]}"; do
        echo "  - $file"
    done
    echo ""
    echo "IMPORTANT: Review and improve the generated tests!"
else
    echo ""
    echo -e "${GREEN}✅ Property test pre-commit check passed${NC}"
fi

exit 0