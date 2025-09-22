#!/bin/bash
# Re-enable all tests for full coverage

echo "Re-enabling all tests..."

# Remove #[ignore] attributes we added
find server/src -name "*.rs" -exec grep -l "#\[ignore\] // Memory allocation issues" {} \; | while read file; do
    echo "Re-enabling tests in $file..."
    sed -i '/#\[ignore\] \/\/ Memory allocation issues/d' "$file"
done

# Re-enable property tests
find server/src -name "*.rs" -exec grep -l "#\[cfg(not(coverage))\] mod property_tests" {} \; | while read file; do
    echo "Re-enabling property tests in $file..."
    sed -i 's/#\[cfg(not(coverage))\] mod property_tests/mod property_tests/g' "$file"
    sed -i 's/#\[cfg(not(coverage))\] pub mod property_tests/pub mod property_tests/g' "$file"
done

echo "Done!"