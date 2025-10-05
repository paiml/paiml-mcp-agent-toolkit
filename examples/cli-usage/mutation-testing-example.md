# Mutation Testing Example

This example demonstrates how to use PMAT's mutation testing feature to measure test suite quality.

## Quick Start

### 1. Basic Mutation Testing

Test a single Rust file:

```bash
pmat analyze mutate --path src/calculator.rs
```

**Output:**
```
🧬 Mutation Testing
Path: src/calculator.rs
Operators: AOR, ROR, COR, UOR (default)

📝 Generating mutants...
✅ Generated 12 mutants

🧪 Running tests on mutants...
  [1/12] Testing mutant AOR_a3f1b2c...
    ✅ Killed (342ms)
  [2/12] Testing mutant ROR_d4e5f6a...
    ❌ Survived (298ms)
  ...

✅ Mutation testing complete!
   Mutation score: 75.00%
   9 mutants killed, 3 survived
```

### 2. Specify Mutation Operators

Choose which operators to use:

```bash
pmat analyze mutate \
  --path src/lib.rs \
  --operators AOR,ROR,COR,UOR,CRR,SDL
```

### 3. Set Quality Threshold

Fail if mutation score is below 80%:

```bash
pmat analyze mutate \
  --path src/lib.rs \
  --min-score 0.80
```

Exit code will be non-zero if score < 80%.

## Example Code to Test

Create `examples/calculator.rs`:

```rust
/// Add two numbers
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Subtract two numbers
pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

/// Check if number is positive
pub fn is_positive(n: i32) -> bool {
    n > 0
}

/// Check if two numbers are equal
pub fn are_equal(a: i32, b: i32) -> bool {
    a == b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(0, 0), 0);
        assert_eq!(add(-1, 1), 0);
    }

    #[test]
    fn test_subtract() {
        assert_eq!(subtract(5, 3), 2);
        assert_eq!(subtract(0, 0), 0);
    }

    #[test]
    fn test_is_positive() {
        assert!(is_positive(1));
        assert!(!is_positive(0));
    }

    // Note: are_equal has NO tests!
    // This will be caught by mutation testing
}
```

### Run Mutation Testing

```bash
pmat analyze mutate --path examples/calculator.rs
```

**Expected Results:**
- `add()`: High mutation score (well tested)
- `subtract()`: Medium score (missing edge cases)
- `is_positive()`: Medium score (missing negative test)
- `are_equal()`: **0% score** - no tests! All mutants survive.

## Understanding Results

### Killed Mutants ✅
When tests catch the mutation, the mutant is "killed" (good!):
```
Original: a + b
Mutant:   a - b  ← Test fails, mutant killed ✅
```

### Survived Mutants ❌
When tests pass despite the mutation, it survived (bad - test gap!):
```
Original: n > 0
Mutant:   n >= 0  ← Tests still pass, mutant survived ❌
```

This reveals: You're not testing the `n == 0` edge case!

### Mutation Score Formula
```
Mutation Score = Killed / (Total - CompileErrors - Timeouts)
```

## Real-World Example: Workspace Crates

Test files in workspace crates:

```bash
# Test a specific crate
pmat analyze mutate --path crates/my-crate/src/validator.rs

# Smart filtering automatically runs only relevant tests
# For crates/my-crate/src/validator.rs → runs tests matching 'validator'
```

## Performance Tips

PMAT uses **smart test filtering** for speed:

- **File**: `server/src/services/mutation/types.rs`
- **Filter**: Runs tests in `services::mutation` module only
- **Result**: 20× faster than running entire test suite

### Before (v2.134.0):
```
cargo test --lib  # ALL tests, every mutant
→ 120s per mutant ❌
```

### After (v2.135.0):
```
cargo test --lib -- services::mutation  # Only relevant tests
→ 24s per mutant ✅ (5× speedup)
```

## Troubleshooting

### File Corruption (Fixed in v2.136.0)

**Problem:** Files corrupted with unformatted code on one line.

**Solution:**
```bash
# Restore corrupted file
git checkout -- path/to/corrupted/file.rs

# Upgrade to latest version
cargo install pmat --version 2.137.0

# Verify version
pmat --version  # Should be ≥2.136.0
```

### Low Mutation Score

If score is below expectations:

1. **Review survived mutants** in the report
2. **Add missing test cases** for weak spots
3. **Improve assertion coverage** (test more edge cases)
4. **Use specific operators** to focus on certain mutation types

### Timeout Issues

If mutations timeout:

1. Check test suite performance
2. Use `--workers` for parallel execution (future feature)
3. Filter to specific operators with `--operators`

## Advanced Usage

### JSON Output

Get machine-readable results:

```bash
pmat analyze mutate \
  --path src/lib.rs \
  --format json \
  --output mutation-report.json
```

### CI/CD Integration

Add to `.github/workflows/test.yml`:

```yaml
- name: Mutation Testing
  run: |
    pmat analyze mutate \
      --path src/lib.rs \
      --min-score 0.75 \
      --format json \
      --output mutation-results.json
```

### Pre-commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
# Run mutation testing on changed files

CHANGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$')

for file in $CHANGED_FILES; do
  echo "Testing $file..."
  pmat analyze mutate --path "$file" --min-score 0.70 || exit 1
done
```

## Mutation Operators Explained

### AOR - Arithmetic Operator Replacement
```rust
a + b  →  a - b, a * b, a / b, a % b
```

### ROR - Relational Operator Replacement
```rust
a < b  →  a <= b, a > b, a >= b, a == b, a != b
```

### COR - Conditional Operator Replacement
```rust
a && b  →  a || b
a || b  →  a && b
```

### UOR - Unary Operator Replacement
```rust
!condition  →  condition
-value      →  value
```

### CRR - Constant Replacement
```rust
0      →  1
true   →  false
"text" →  ""
```

### SDL - Statement Deletion
```rust
let x = calculate();  →  // deleted
process(x);
```

## Benchmarks

PMAT vs cargo-mutants (pforge validator.rs):

| Tool | Mutants | Time | Per Mutant | Score |
|------|---------|------|------------|-------|
| **PMAT** | 28 | 10.8s | 0.39s | 21.43% |
| cargo-mutants | 4 | 31s | 7.75s | 100% |

**PMAT is 20× faster** and generates **7× more mutants** (better coverage)!

## Next Steps

1. **Run on your codebase**: `pmat analyze mutate --path src/lib.rs`
2. **Analyze results**: Look for survived mutants
3. **Add tests**: Cover the gaps revealed by mutations
4. **Improve score**: Aim for 75-85% mutation score
5. **Integrate CI/CD**: Enforce minimum mutation score

## Resources

- [Full Mutation Testing Guide](../../docs/mutation-testing.md)
- [GitHub Issue #64](https://github.com/paiml/paiml-mcp-agent-toolkit/issues/64) - File corruption bug (FIXED)
- [Five Whys Analysis](../../FIVE_WHYS_ANALYSIS.md) - Root cause fix documentation
