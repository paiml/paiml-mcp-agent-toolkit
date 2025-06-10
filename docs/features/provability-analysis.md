# Provability Analysis

## Overview

The Provability Analysis feature provides lightweight formal verification capabilities for code properties. It uses abstract interpretation and property domain analysis to prove certain properties about code without full formal methods overhead. This enables developers to gain confidence in critical code properties while maintaining practical analysis times.

## Core Concepts

### Property Domains

The analyzer supports several property domains:

1. **Nullability Analysis** - Proves absence of null pointer dereferences
2. **Alias Analysis** - Tracks pointer aliasing and ownership
3. **Range Analysis** - Proves integer bounds and overflow safety
4. **Initialization Analysis** - Ensures variables are initialized before use
5. **Taint Analysis** - Tracks data flow from untrusted sources
6. **Lifetime Analysis** - Proves memory safety in Rust-like systems

### Confidence Levels

Each property annotation includes a confidence score:

- **High (90-100%)** - Property can be proven with high certainty
- **Medium (70-89%)** - Property likely holds but with some assumptions
- **Low (50-69%)** - Property may hold but requires manual verification

## Command Usage

### Basic Analysis

```bash
# Analyze current directory
pmat analyze proof-annotations

# Specific path with high confidence only
pmat analyze proof-annotations \
  --project-path /path/to/project \
  --high-confidence-only

# Include detailed evidence
pmat analyze proof-annotations --include-evidence
```

### Filtering Options

```bash
# Filter by property type
pmat analyze proof-annotations --property-type nullability

# Filter by verification method
pmat analyze proof-annotations --verification-method abstract-interpretation

# Clear cache for fresh analysis
pmat analyze proof-annotations --clear-cache
```

## Output Formats

### Summary Format (Default)

```
Provability Analysis Results
================================================================================

High Confidence Properties (95%+)
─────────────────────────────────
src/core/parser.rs::parse_input
  ✓ Nullability: Parameter 'input' is never null
  ✓ Bounds: Index 'i' always within array bounds
  ✓ Initialization: All paths initialize 'result'

src/utils/validator.rs::check_range
  ✓ Overflow: No integer overflow possible
  ✓ Range: Return value always in [0, 100]

Medium Confidence Properties (70-94%)
────────────────────────────────────
src/handlers/request.rs::process
  ? Aliasing: No mutable aliasing detected (85%)
  ? Taint: User input properly sanitized (78%)

Total functions analyzed: 234
Properties proven: 156 (66.7%)
Average confidence: 84.3%
```

### JSON Format

```json
{
  "analysis_results": [
    {
      "file": "src/core/parser.rs",
      "function": "parse_input",
      "properties": [
        {
          "type": "nullability",
          "property": "parameter_non_null",
          "parameter": "input",
          "confidence": 0.98,
          "evidence": {
            "type": "precondition_check",
            "location": {"line": 15, "column": 8},
            "description": "Explicit null check at function entry"
          }
        },
        {
          "type": "bounds",
          "property": "array_access_safe",
          "confidence": 0.95,
          "evidence": {
            "type": "loop_invariant",
            "invariant": "0 <= i < array.len()",
            "verified_by": "induction"
          }
        }
      ]
    }
  ],
  "summary": {
    "total_functions": 234,
    "functions_with_properties": 178,
    "total_properties": 523,
    "proven_properties": 412,
    "property_distribution": {
      "nullability": 145,
      "bounds": 89,
      "initialization": 67,
      "aliasing": 45,
      "overflow": 34,
      "taint": 32
    }
  }
}
```

## Property Types

### Nullability Properties

Proves that pointers/references are non-null:

```rust
// Proven: input is never null
fn process(input: &str) -> Result<Data> {
    // Analyzer proves this can't panic
    let first_char = input.chars().next().unwrap();
    // ...
}

// Proven: Option is checked before use
fn safe_divide(a: i32, b: Option<i32>) -> Option<i32> {
    b.map(|divisor| a / divisor)  // No division by zero
}
```

### Bounds Properties

Proves array/slice accesses are within bounds:

```rust
// Proven: No out-of-bounds access
fn sum_array(arr: &[i32]) -> i32 {
    let mut sum = 0;
    for i in 0..arr.len() {
        sum += arr[i];  // Proven safe
    }
    sum
}

// Proven: Substring bounds are valid
fn extract_middle(s: &str, start: usize, len: usize) -> &str {
    if start + len <= s.len() {
        &s[start..start + len]  // Proven safe
    } else {
        ""
    }
}
```

### Overflow Properties

Proves absence of integer overflow:

```rust
// Proven: No overflow in factorial
fn factorial(n: u32) -> Option<u32> {
    if n > 12 {
        return None;  // Prevents overflow
    }
    
    let mut result = 1u32;
    for i in 2..=n {
        result *= i;  // Proven: no overflow due to guard
    }
    Some(result)
}
```

### Initialization Properties

Proves variables are initialized before use:

```rust
// Proven: result always initialized
fn find_max(values: &[i32]) -> Option<i32> {
    let mut result;
    
    if values.is_empty() {
        return None;
    }
    
    result = values[0];  // Always executed if not empty
    
    for &val in &values[1..] {
        if val > result {
            result = val;
        }
    }
    
    Some(result)  // Proven: result is initialized
}
```

### Aliasing Properties

Proves absence of problematic aliasing:

```rust
// Proven: No mutable aliasing
fn swap_elements(arr: &mut [i32], i: usize, j: usize) {
    if i != j && i < arr.len() && j < arr.len() {
        // Proven: arr[i] and arr[j] don't alias
        let temp = arr[i];
        arr[i] = arr[j];
        arr[j] = temp;
    }
}
```

### Taint Properties

Tracks data flow from untrusted sources:

```python
# Proven: SQL injection safe
def get_user(user_id: str) -> User:
    # Proven: user_id is sanitized before use
    sanitized_id = sanitize_input(user_id)
    query = f"SELECT * FROM users WHERE id = ?"
    return db.execute(query, [sanitized_id])

# Proven: XSS safe
def render_comment(comment: str) -> str:
    # Proven: HTML escaping prevents XSS
    escaped = html.escape(comment)
    return f"<div class='comment'>{escaped}</div>"
```

## Evidence Types

### Direct Evidence

```json
{
  "type": "direct_proof",
  "description": "Explicit check proves property",
  "code_snippet": "if ptr != null { use(ptr) }",
  "confidence": 1.0
}
```

### Inductive Evidence

```json
{
  "type": "loop_invariant",
  "invariant": "0 <= i <= array.length",
  "initialization": "i = 0",
  "preservation": "i++",
  "confidence": 0.95
}
```

### Flow-Sensitive Evidence

```json
{
  "type": "path_analysis",
  "paths_analyzed": 12,
  "paths_proven": 12,
  "description": "All execution paths maintain property",
  "confidence": 0.92
}
```

### Compositional Evidence

```json
{
  "type": "function_contract",
  "callee": "validate_input",
  "postcondition": "return != null",
  "propagated_property": "input validated",
  "confidence": 0.88
}
```

## Integration with Other Tools

### Deep Context Integration

```bash
# Include provability in deep context
pmat analyze deep-context \
  --include-provability \
  --min-confidence 0.8
```

### Refactoring Integration

```bash
# Preserve proven properties during refactoring
pmat refactor serve \
  --preserve-properties \
  --property-confidence 0.9
```

### CI/CD Integration

```yaml
- name: Verify Critical Properties
  run: |
    pmat analyze proof-annotations \
      --property-type nullability \
      --high-confidence-only \
      --format json > properties.json
    
    # Fail if critical properties can't be proven
    if [ $(jq '.summary.proven_properties' properties.json) -lt 50 ]; then
      echo "::error::Failed to prove required properties"
      exit 1
    fi
```

## Advanced Features

### Custom Properties

Define domain-specific properties:

```json
{
  "custom_properties": [
    {
      "name": "monetary_precision",
      "description": "Monetary calculations maintain precision",
      "pattern": "Money\\s*\\*|/\\s*",
      "verification": "range_analysis",
      "constraints": ["no_precision_loss", "no_overflow"]
    }
  ]
}
```

### Incremental Analysis

Leverage caching for faster re-analysis:

```bash
# First run builds cache
pmat analyze proof-annotations

# Subsequent runs are incremental
pmat analyze proof-annotations  # Only analyzes changed files
```

### Property Composition

Combine simple properties into complex guarantees:

```
Memory Safety = Nullability ∧ Bounds ∧ Initialization ∧ No-Aliasing
Security = Taint-Free ∧ Bounds ∧ Input-Validation
```

## Performance Characteristics

### Analysis Speed

- **Small functions (<50 LOC)**: <10ms
- **Medium functions (50-200 LOC)**: 10-50ms
- **Large functions (>200 LOC)**: 50-200ms
- **Cache hit**: <1ms

### Memory Usage

- **Per-function state**: ~10KB
- **Cache overhead**: ~100KB per file
- **Peak memory**: O(largest function)

### Scalability

- Linear in codebase size
- Parallelizable across files
- Incremental analysis support

## Limitations

### Decidability

Some properties are undecidable in general:

- Arbitrary loop termination
- Complex pointer arithmetic
- Recursive data structure properties
- Concurrency properties

### Precision

The analysis is necessarily conservative:

- May report "unknown" for provable properties
- Assumes worst-case for external functions
- Limited inter-procedural analysis depth

### Language Support

Property support varies by language:

| Language | Nullability | Bounds | Overflow | Aliasing | Taint |
|----------|------------|---------|----------|----------|-------|
| Rust     | ✓          | ✓       | ✓        | ✓        | ✓     |
| C/C++    | ✓          | ✓       | ✓        | Partial  | ✓     |
| Python   | ✓          | Partial | N/A      | Limited  | ✓     |
| TypeScript| ✓         | Partial | N/A      | Limited  | ✓     |

## Best Practices

### 1. Focus on Critical Code

Prioritize proving properties for:
- Security-sensitive functions
- Core algorithms
- API boundaries
- Error handling paths

### 2. Write Provable Code

Structure code to be amenable to analysis:

```rust
// Harder to prove
fn complex_logic(data: &[u8]) -> Option<u32> {
    let mut result = None;
    for i in 0..data.len() {
        if some_complex_condition(i, data) {
            result = Some(calculate(data, i));
            break;
        }
    }
    result
}

// Easier to prove
fn simple_logic(data: &[u8]) -> Option<u32> {
    data.iter()
        .position(|&b| is_target(b))
        .map(|i| calculate_at(data, i))
}
```

### 3. Document Assumptions

Make implicit assumptions explicit:

```rust
/// Assumes: input.len() > 0
/// Proves: No panic, returns value in range [0, 255]
fn process_bytes(input: &[u8]) -> u8 {
    assert!(!input.is_empty());
    // Analysis can now prove properties
    input.iter().fold(0u8, |acc, &b| acc.saturating_add(b))
}
```

### 4. Incremental Adoption

Start with high-value properties:

1. Null safety in critical paths
2. Bounds checking in parsers
3. Overflow prevention in calculations
4. Taint tracking in user input handling

## Related Commands

- `pmat analyze complexity` - Identify complex code needing verification
- `pmat analyze deep-context` - Include provability in analysis
- `pmat refactor interactive` - Refactor while preserving properties
- `pmat analyze defect-prediction` - Correlate properties with defects