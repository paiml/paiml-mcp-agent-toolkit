# Big-O Complexity Analysis

## Overview

The Big-O analysis feature provides automated algorithmic complexity detection for functions across multiple programming languages. It uses pattern matching and control flow analysis to determine time complexity with confidence scores and supporting evidence.

## Command Usage

### Basic Usage

```bash
# Analyze all functions in the current directory
pmat analyze big-o

# Analyze specific path
pmat analyze big-o --project-path /path/to/project

# Filter by complexity threshold
pmat analyze big-o --min-complexity "O(n^2)"

# JSON output for tool integration
pmat analyze big-o --format json
```

### Advanced Options

```bash
# Include only specific patterns
pmat analyze big-o --include "**/*.rs" --exclude "**/tests/**"

# Show detailed evidence
pmat analyze big-o --include-evidence

# Filter by confidence level
pmat analyze big-o --min-confidence 0.8

# Limit results
pmat analyze big-o --top-k 20
```

## Output Formats

### Summary Format (Default)

```
Big-O Complexity Analysis Results
================================================================================

src/services/analyzer.rs::process_data
  Complexity: O(n^2)
  Confidence: 85%
  Evidence: Nested loops over input parameter

src/utils/sort.rs::merge_sort
  Complexity: O(n log n)
  Confidence: 95%
  Evidence: Divide-and-conquer recursion pattern

Total functions analyzed: 1,234
Functions with detected complexity: 89
Average confidence: 78%
```

### JSON Format

```json
{
  "analysis_results": [
    {
      "file": "src/services/analyzer.rs",
      "function": "process_data",
      "complexity": "O(n^2)",
      "confidence": 0.85,
      "evidence": [
        {
          "type": "nested_loops",
          "description": "Nested iteration over 'items' collection",
          "line_numbers": [45, 47]
        }
      ],
      "metrics": {
        "lines_of_code": 125,
        "cyclomatic_complexity": 15,
        "loop_depth": 2
      }
    }
  ],
  "summary": {
    "total_functions": 1234,
    "analyzed_functions": 89,
    "complexity_distribution": {
      "O(1)": 450,
      "O(log n)": 23,
      "O(n)": 234,
      "O(n log n)": 12,
      "O(n^2)": 8,
      "O(n^3)": 2
    }
  }
}
```

## Complexity Detection Patterns

### Constant Time - O(1)
- Direct array/hash access
- Simple arithmetic operations
- Fixed iterations

### Logarithmic - O(log n)
- Binary search patterns
- Tree traversal with pruning
- Divide-and-conquer with single branch

### Linear - O(n)
- Single loop over input
- Linear recursion
- Single-pass algorithms

### Linearithmic - O(n log n)
- Efficient sorting algorithms
- Divide-and-conquer with merge
- Tree construction

### Quadratic - O(n²)
- Nested loops over same input
- All-pairs comparisons
- Matrix operations

### Cubic - O(n³)
- Triple nested loops
- 3D matrix operations
- Certain graph algorithms

### Exponential - O(2^n)
- Recursive branching without memoization
- Power set generation
- Combinatorial explosion

## Integration with Other Tools

### Complexity Analysis Integration

```bash
# Combine with cyclomatic complexity
pmat analyze complexity --format json > complexity.json
pmat analyze big-o --format json > big-o.json

# Use with refactoring engine
pmat refactor serve --priority "big_o_complexity"
```

### CI/CD Integration

```yaml
# GitHub Actions example
- name: Check Big-O Complexity
  run: |
    pmat analyze big-o --min-complexity "O(n^2)" --format json > big-o.json
    if [ $(jq '.summary.high_complexity_count' big-o.json) -gt 0 ]; then
      echo "::error::Found functions with O(n²) or higher complexity"
      exit 1
    fi
```

## Evidence Types

### Loop Analysis
- Single loops: O(n)
- Nested loops: O(n²), O(n³)
- Loop with logarithmic bounds: O(log n)

### Recursion Patterns
- Linear recursion: O(n)
- Binary recursion: O(2^n)
- Tail recursion: Often O(n)
- Divide-and-conquer: O(n log n)

### Data Structure Operations
- Array access: O(1)
- HashMap operations: O(1) average
- Tree operations: O(log n) balanced
- List operations: O(n)

### Algorithm Recognition
- Known sorting algorithms
- Graph traversal patterns
- Dynamic programming
- Greedy algorithms

## Confidence Scoring

Confidence scores indicate how certain the analysis is:

- **95-100%**: Known algorithm patterns or simple structures
- **80-94%**: Clear patterns with minor ambiguity
- **60-79%**: Complex patterns with some uncertainty
- **Below 60%**: Best effort estimate, manual review recommended

## Best Practices

### For Development

1. **Regular Analysis**: Run Big-O analysis as part of code review
2. **Focus on Hot Paths**: Prioritize optimization of frequently called functions
3. **Verify Critical Functions**: Manually review low-confidence results
4. **Track Trends**: Monitor complexity changes over time

### For CI/CD

1. **Set Thresholds**: Define acceptable complexity limits
2. **Gate on Regression**: Prevent complexity increases
3. **Exemption List**: Document acceptable high-complexity functions
4. **Trend Reporting**: Track complexity metrics over releases

## Limitations

- **Dynamic Behavior**: Cannot analyze runtime-dependent complexity
- **Data-Dependent**: Assumes worst-case unless patterns indicate otherwise
- **Language Features**: Some language-specific optimizations may not be detected
- **External Calls**: Cannot analyze complexity of external function calls

## Examples

### Python Example

```python
def find_duplicates(arr):  # Detected: O(n²)
    duplicates = []
    for i in range(len(arr)):
        for j in range(i + 1, len(arr)):
            if arr[i] == arr[j]:
                duplicates.append(arr[i])
    return duplicates
```

### Rust Example

```rust
fn binary_search(arr: &[i32], target: i32) -> Option<usize> {  // Detected: O(log n)
    let mut left = 0;
    let mut right = arr.len();
    
    while left < right {
        let mid = left + (right - left) / 2;
        match arr[mid].cmp(&target) {
            Ordering::Equal => return Some(mid),
            Ordering::Less => left = mid + 1,
            Ordering::Greater => right = mid,
        }
    }
    None
}
```

### TypeScript Example

```typescript
function mergeSort(arr: number[]): number[] {  // Detected: O(n log n)
    if (arr.length <= 1) return arr;
    
    const mid = Math.floor(arr.length / 2);
    const left = mergeSort(arr.slice(0, mid));
    const right = mergeSort(arr.slice(mid));
    
    return merge(left, right);
}
```

## Related Commands

- `pmat analyze complexity` - Cyclomatic and cognitive complexity
- `pmat refactor interactive` - Reduce complexity interactively
- `pmat analyze deep-context` - Comprehensive analysis including Big-O
- `pmat analyze defect-prediction` - Correlate complexity with defects