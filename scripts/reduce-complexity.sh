#!/bin/bash
# Script to help reduce function complexity

echo "=== Complexity Reduction Report ==="
echo "Finding functions with complexity > 20..."

# Get all violations
./target/release/pmat analyze complexity --format json | jq -r '.violations[] | select(.value > 20) | "\(.file):\(.function) - complexity: \(.value)"' | sort -t: -k3 -nr > /tmp/complexity_violations.txt

echo "Total violations: $(wc -l < /tmp/complexity_violations.txt)"
echo ""
echo "Top 10 most complex functions:"
head -10 /tmp/complexity_violations.txt

echo ""
echo "Files with most violations:"
cut -d: -f1 /tmp/complexity_violations.txt | sort | uniq -c | sort -nr | head -10

echo ""
echo "Recommended refactoring order (highest complexity first):"
head -20 /tmp/complexity_violations.txt