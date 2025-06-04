#!/bin/bash
# complexity_distribution.sh - Analyze existing metrics

set -euo pipefail

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    echo "Python 3 is required for distribution analysis"
    exit 1
fi

# Check if jq is available
if ! command -v jq &> /dev/null; then
    echo "jq is required for JSON processing"
    exit 1
fi

# Get path from argument or use current directory
PATH_TO_ANALYZE="${1:-.}"

echo "Analyzing complexity distribution for: $PATH_TO_ANALYZE"
echo

# Extract raw complexity data
echo "Extracting complexity metrics..."
pmat analyze complexity --path "$PATH_TO_ANALYZE" --format json | \
  jq -r '.files[].functions[] | [.name, .cyclomatic, .cognitive] | @csv' > complexity_raw.csv

# Check if we got any data
if [ ! -s complexity_raw.csv ]; then
    echo "ERROR: No complexity data found"
    exit 1
fi

# Count functions
FUNCTION_COUNT=$(wc -l < complexity_raw.csv)
echo "Found $FUNCTION_COUNT functions"
echo

# Calculate distribution metrics using existing tools
python3 -c "
import csv
import math
from collections import Counter

with open('complexity_raw.csv', 'r') as f:
    reader = csv.reader(f)
    data = list(reader)
    cyclomatic_values = [int(row[1]) for row in data]
    cognitive_values = [int(row[2]) for row in data]

if not cyclomatic_values:
    print('ERROR: No complexity values found')
    exit(1)

# Calculate entropy for cyclomatic complexity
counter = Counter(cyclomatic_values)
total = len(cyclomatic_values)
entropy = -sum((count/total) * math.log2(count/total) 
               for count in counter.values())

# Calculate percentiles
cyclomatic_values.sort()
p50 = cyclomatic_values[len(cyclomatic_values)//2]
p90 = cyclomatic_values[int(len(cyclomatic_values)*0.9)]
p99 = cyclomatic_values[int(len(cyclomatic_values)*0.99)] if len(cyclomatic_values) > 100 else cyclomatic_values[-1]

# Calculate mean and standard deviation
mean = sum(cyclomatic_values) / len(cyclomatic_values)
variance = sum((x - mean) ** 2 for x in cyclomatic_values) / len(cyclomatic_values)
std_dev = math.sqrt(variance)
cv = (std_dev / mean * 100) if mean > 0 else 0

# Count complexity buckets
simple = sum(1 for v in cyclomatic_values if v <= 5)
moderate = sum(1 for v in cyclomatic_values if 6 <= v <= 15)
complex = sum(1 for v in cyclomatic_values if 16 <= v <= 30)
very_complex = sum(1 for v in cyclomatic_values if v > 30)

print('=== Cyclomatic Complexity Distribution ===')
print(f'Entropy: {entropy:.2f}')
print(f'Mean: {mean:.2f}, Std Dev: {std_dev:.2f}, CV: {cv:.1f}%')
print(f'P50: {p50}, P90: {p90}, P99: {p99}')
print()
print('=== Complexity Buckets ===')
print(f'Simple (1-5): {simple} ({simple/total*100:.1f}%)')
print(f'Moderate (6-15): {moderate} ({moderate/total*100:.1f}%)')
print(f'Complex (16-30): {complex} ({complex/total*100:.1f}%)')
print(f'Very Complex (>30): {very_complex} ({very_complex/total*100:.1f}%)')
print()
print(f'Functions > 10: {sum(1 for v in cyclomatic_values if v > 10)} ({sum(1 for v in cyclomatic_values if v > 10)/total*100:.1f}%)')

# Verification checks
print()
print('=== Verification Results ===')

issues = []

if entropy < 2.0 and len(cyclomatic_values) > 100:
    issues.append(f'WARNING: Low complexity entropy ({entropy:.2f}) detected')

if cv < 30.0 and len(cyclomatic_values) > 50:
    issues.append(f'WARNING: Low complexity variation (CV={cv:.1f}%) - possible parser issue')

complex_ratio = sum(1 for v in cyclomatic_values if v > 10) / total
if complex_ratio < 0.05 and len(cyclomatic_values) > 100:
    issues.append(f'WARNING: Suspiciously few complex functions ({complex_ratio*100:.1f}%)')

# Check for suspicious patterns
unique_values = len(set(cyclomatic_values))
if unique_values < 5 and len(cyclomatic_values) > 20:
    issues.append(f'WARNING: Only {unique_values} unique complexity values found')

if issues:
    for issue in issues:
        print(issue)
else:
    print('✓ All distribution checks passed')

# Output top complex functions
print()
print('=== Top 10 Most Complex Functions ===')
with open('complexity_raw.csv', 'r') as f:
    reader = csv.reader(f)
    sorted_data = sorted(reader, key=lambda x: int(x[1]), reverse=True)[:10]
    for i, row in enumerate(sorted_data, 1):
        print(f'{i}. {row[0]}: Cyclomatic={row[1]}, Cognitive={row[2]}')
"

# Clean up
rm -f complexity_raw.csv

echo
echo "Analysis complete"