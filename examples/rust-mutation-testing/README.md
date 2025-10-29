# Rust Mutation Testing Example

This example demonstrates how to use PMAT for mutation testing on a Rust project.

## Overview

Mutation testing is a technique to evaluate the quality of your test suite by introducing small changes (mutations) to your code and checking if your tests detect them.

**Mutation Score** = (Killed Mutants / Total Valid Mutants) × 100%

- **Killed**: Test suite detected the mutation (good!)
- **Survived**: Test suite did not detect the mutation (test gap!)
- **CompileError**: Mutation caused invalid syntax
- **Timeout**: Mutation caused infinite loop

## Project Structure

```
rust-mutation-testing/
├── Cargo.toml          # Package configuration
├── src/
│   └── lib.rs          # Calculator library with tests
└── README.md           # This file
```

## Getting Started

### 1. Run Tests

First, verify that all tests pass:

```bash
cd examples/rust-mutation-testing
cargo test
```

You should see:
```
running 8 tests
test tests::test_add ... ok
test tests::test_subtract ... ok
test tests::test_multiply ... ok
test tests::test_divide ... ok
test tests::test_is_even ... ok
test tests::test_max ... ok
test tests::test_factorial ... ok
test tests::test_is_prime ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### 2. Run Mutation Testing

Use PMAT to perform mutation testing on the library:

```bash
# Install pmat if not already installed
cargo install pmat

# Run mutation testing on the library
pmat mutate --target src/lib.rs
```

### 3. Analyze Results

PMAT will generate mutants and run your tests against each one. Example output:

```
Mutation Testing Results
========================
Total Mutants: 45
Killed: 40 (88.9%)
Survived: 3 (6.7%)
Compile Errors: 2 (4.4%)
Timeouts: 0 (0.0%)

Mutation Score: 88.9%
```

### 4. Review Survived Mutants (Test Gaps)

Use `--failures-only` to see only survived mutants:

```bash
pmat mutate --target src/lib.rs --failures-only
```

This shows which mutations your tests failed to catch, indicating gaps in test coverage.

## Example: Detecting Test Gaps

### Original Code
```rust
pub fn max(a: i32, b: i32) -> i32 {
    if a > b {  // Mutation: change > to >=
        a
    } else {
        b
    }
}
```

### Mutation
```rust
pub fn max(a: i32, b: i32) -> i32 {
    if a >= b {  // Mutated: > changed to >=
        a
    } else {
        b
    }
}
```

### Test That Catches It
```rust
#[test]
fn test_max() {
    assert_eq!(max(5, 3), 5);  // Would pass with either > or >=
    assert_eq!(max(2, 8), 8);  // Would pass with either > or >=
    assert_eq!(max(4, 4), 4);  // CRITICAL: Catches the >= mutation!
}
```

Without the `max(4, 4)` test case, the `> to >=` mutation would **survive**, indicating a test gap.

## Output Formats

### Text (Default)
```bash
pmat mutate --target src/lib.rs
```

Color-coded terminal output with summary and individual mutant details.

### JSON
```bash
pmat mutate --target src/lib.rs --output-format json > results.json
```

Machine-readable JSON for CI/CD integration.

### Markdown
```bash
pmat mutate --target src/lib.rs --output-format markdown > results.md
```

Human-readable report with tables and statistics.

## Advanced Usage

### Control Concurrency
```bash
# Use 4 parallel jobs
pmat mutate --target src/lib.rs --jobs 4
```

### Set Timeout
```bash
# Timeout individual tests after 10 seconds
pmat mutate --target src/lib.rs --timeout 10
```

### Enforce Mutation Score Threshold
```bash
# Fail if mutation score is below 85%
pmat mutate --target src/lib.rs --threshold 85
```

This will exit with code 1 if the mutation score is below 85%, useful for CI/CD quality gates.

## CI/CD Integration

### GitHub Actions

Create `.github/workflows/mutation-testing.yml`:

```yaml
name: Mutation Testing

on: [push, pull_request]

jobs:
  mutation-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install pmat
        run: cargo install pmat

      - name: Run mutation tests
        run: |
          cd examples/rust-mutation-testing
          pmat mutate --target src/lib.rs --failures-only --threshold 80
```

### GitLab CI

Add to `.gitlab-ci.yml`:

```yaml
mutation-testing:
  image: rust:latest
  stage: test
  script:
    - cargo install pmat
    - cd examples/rust-mutation-testing
    - pmat mutate --target src/lib.rs --output-format json > mutation_results.json
  artifacts:
    reports:
      junit: mutation_results.json
```

## Best Practices

1. **Run mutation testing regularly** - Integrate into CI/CD pipeline
2. **Set reasonable thresholds** - 80-90% mutation score is excellent
3. **Focus on critical code** - Test business logic thoroughly
4. **Review survived mutants** - Each one represents a potential bug
5. **Use `--failures-only`** - Reduces noise in large codebases
6. **Test edge cases** - Boundary values catch more mutations
7. **Combine with coverage** - High code coverage + high mutation score = robust tests

## Understanding Mutation Operators

PMAT applies various mutation operators:

### Arithmetic Operators
- `+` → `-`, `*`, `/`
- `-` → `+`, `*`, `/`
- `*` → `+`, `-`, `/`
- `/` → `+`, `-`, `*`

### Relational Operators
- `==` → `!=`, `<`, `>`, `<=`, `>=`
- `<` → `<=`, `>`, `>=`, `==`, `!=`
- `>` → `>=`, `<`, `<=`, `==`, `!=`

### Conditional Operators
- `&&` → `||`
- `||` → `&&`
- `!` → (removed)

### Boundary Values
- `<` → `<=`
- `>` → `>=`
- `0` → `1`, `-1`

## Example Results

```
╔═══════════════════════════════════════════╗
║     Mutation Testing Summary              ║
╠═══════════════════════════════════════════╣
║ Total Mutants:        45                  ║
║ Killed:               40 (88.9%)          ║
║ Survived:             3 (6.7%)            ║
║ Compile Errors:       2 (4.4%)            ║
║ Timeouts:             0 (0.0%)            ║
╠═══════════════════════════════════════════╣
║ Mutation Score:       88.9% ✓             ║
╚═══════════════════════════════════════════╝
```

## Resources

- **PMAT Repository**: https://github.com/paiml/paiml-mcp-agent-toolkit
- **Crates.io**: https://crates.io/crates/pmat
- **Documentation**: `server/README.md`
- **Mutation Testing Paper**: [Mutation Testing: An Empirical Evaluation](https://dl.acm.org/doi/10.1145/3183440) *(requires ACM subscription)*

## License

This example is part of the PMAT project and is provided as-is for demonstration purposes.
