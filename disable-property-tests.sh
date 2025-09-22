#!/bin/bash
# Temporarily disable property tests for coverage

echo "Temporarily disabling property tests to fix memory issues..."

# Add #[cfg(not(coverage))] before property test modules
find server/src -name "*.rs" -exec grep -l "mod property_tests" {} \; | while read file; do
    echo "Disabling property tests in $file..."
    sed -i 's/^mod property_tests/#[cfg(not(coverage))] mod property_tests/g' "$file"
    sed -i 's/^pub mod property_tests/#[cfg(not(coverage))] pub mod property_tests/g' "$file"
done

echo "Done!"