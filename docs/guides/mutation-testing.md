# Mutation Testing with PMAT - User Guide

Comprehensive guide to mutation testing with PMAT (Polyglot Multi-language Analysis Toolkit).

## Table of Contents

- [What is Mutation Testing?](#what-is-mutation-testing)
- [Why Mutation Testing?](#why-mutation-testing)
- [Getting Started](#getting-started)
- [Basic Usage](#basic-usage)
- [Multi-Language Support](#multi-language-support)
- [Output Formats](#output-formats)
- [Advanced Features](#advanced-features)
- [Interpreting Results](#interpreting-results)
- [Workflow Integration](#workflow-integration)
- [Troubleshooting](#troubleshooting)
- [FAQ](#faq)

---

## What is Mutation Testing?

**Mutation testing** is a technique to evaluate the quality of your test suite by introducing small, deliberate changes (mutations) to your source code and checking whether your tests can detect them.

### Key Concepts

**Mutation**: A small syntactic change to source code
- Example: Change `+` to `-`, `>` to `>=`, `return x` to `return null`

**Mutant**: A version of your code with one mutation applied
- **Killed**: Test suite detected the mutation (GOOD!)
- **Survived**: Test suite did not detect the mutation (Test gap!)
- **Compile Error**: Mutation caused invalid syntax
- **Timeout**: Mutation caused infinite loop

**Mutation Score**: Quality metric for test suite
```
Mutation Score = (Killed Mutants / Total Valid Mutants) × 100%
```

### Example

**Original Code**:
```rust
fn max(a: i32, b: i32) -> i32 {
    if a > b {  // <-- PMAT will mutate this
        return a;
    }
    return b;
}
```

**Mutation**: Change `>` to `>=`
```rust
fn max(a: i32, b: i32) -> i32 {
    if a >= b {  // <-- Mutated
        return a;
    }
    return b;
}
```

**Test That Catches It**:
```rust
#[test]
fn test_max() {
    assert_eq!(max(5, 3), 5);  // Still passes
    assert_eq!(max(2, 8), 8);  // Still passes
    assert_eq!(max(4, 4), 4);  // FAILS with >= mutation!
}
```

Without the `max(4, 4)` test case, the mutation would **survive**, indicating a test gap.

---

## Why Mutation Testing?

### Beyond Code Coverage

Code coverage measures which lines are executed by tests, but doesn't measure test quality.

**100% code coverage ≠ 100% test quality**

Example with 100% coverage but poor tests:
```rust
fn divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        return None;  // <-- Covered by test
    }
    Some(a / b)
}

#[test]
fn test_divide() {
    let _ = divide(10, 0);  // 100% coverage, but no assertions!
}
```

Mutation testing reveals this test is ineffective:
- Mutate `b == 0` to `b != 0` → Test still passes (mutation survives!)
- Mutate `return None` to `return Some(0)` → Test still passes!

### Benefits

1. **Finds Test Gaps**: Reveals untested edge cases
2. **Improves Test Quality**: Forces meaningful assertions
3. **Prevents Regression**: Ensures tests detect real bugs
4. **Complements Coverage**: Works alongside line/branch coverage
5. **Language Agnostic**: PMAT supports Rust, Python, TypeScript, and more

---

## Getting Started

### Installation

```bash
# Install PMAT
cargo install pmat

# Verify installation
pmat --version
```

### Your First Mutation Test

1. **Create a simple function**:

`src/calculator.rs`:
```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
```

2. **Run mutation testing**:

```bash
pmat mutate --target src/calculator.rs
```

3. **Review results**:

```
Mutation Testing Results
========================
Total Mutants: 5
Killed: 3 (60.0%)
Survived: 2 (40.0%)

Mutation Score: 60.0%

Survived Mutants:
-----------------
1. src/calculator.rs:2:5 - Changed + to -
2. src/calculator.rs:2:5 - Changed + to *
```

4. **Improve tests** to kill survived mutants:

```rust
#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);
    assert_eq!(add(-1, 1), 0);   // Kills - mutation
    assert_eq!(add(0, 0), 0);     // Kills * mutation
}
```

5. **Re-run mutation testing**:

```bash
pmat mutate --target src/calculator.rs
```

```
Mutation Score: 100.0% ✓
```

---

## Basic Usage

### Command Syntax

```bash
pmat mutate --target <FILE_OR_DIR> [OPTIONS]
```

### Common Commands

**Single file**:
```bash
pmat mutate --target src/lib.rs
```

**Directory**:
```bash
pmat mutate --target src/
```

**With threshold** (fail if score < 85%):
```bash
pmat mutate --target src/ --threshold 85
```

**Failures only** (CI/CD optimization):
```bash
pmat mutate --target src/ --failures-only
```

**JSON output**:
```bash
pmat mutate --target src/ --output-format json > results.json
```

**Parallel execution** (4 threads):
```bash
pmat mutate --target src/ --jobs 4
```

**Timeout per mutant** (10 seconds):
```bash
pmat mutate --target src/ --timeout 10
```

---

## Multi-Language Support

PMAT automatically detects language from file extension.

### Supported Languages

| Language   | Extensions | Auto-Detect | Example |
|------------|------------|-------------|---------|
| **Rust**   | `.rs`      | ✅          | `pmat mutate --target src/lib.rs` |
| **Python** | `.py`      | ✅          | `pmat mutate --target src/calculator.py` |
| **TypeScript** | `.ts`, `.tsx` | ✅   | `pmat mutate --target src/app.ts` |
| **JavaScript** | `.js`, `.jsx` | ✅   | `pmat mutate --target src/app.js` |
| **Go**     | `.go`      | ✅          | `pmat mutate --target main.go` |
| **C++**    | `.cpp`, `.cc`, `.cxx` | ✅ | `pmat mutate --target src/main.cpp` |

### Language-Specific Examples

**Rust**:
```bash
pmat mutate --target src/lib.rs --threshold 90
```

**Python**:
```bash
pmat mutate --target src/calculator.py --threshold 80
```

**TypeScript**:
```bash
pmat mutate --target src/calculator.ts --threshold 85
```

### Explicit Language Selection

Override auto-detection:
```bash
pmat mutate --target script.txt --language python
```

---

## Output Formats

PMAT supports three output formats.

### Text (Default)

Color-coded terminal output:

```bash
pmat mutate --target src/lib.rs
```

**Output**:
```
Mutation Testing Results
========================
Total Mutants: 50
Killed: 45 (90.0%)
Survived: 3 (6.0%)
Compile Errors: 2 (4.0%)

Mutation Score: 90.0%

✓ Killed: src/lib.rs:10:5 - Changed + to -
✗ Survived: src/lib.rs:15:9 - Changed > to >=
```

Colors:
- 🟢 Green: Killed mutants
- 🔴 Red: Survived mutants
- 🟡 Yellow: Compile errors, timeouts
- 🔵 Cyan: File paths, operators

### JSON

Machine-readable format for CI/CD:

```bash
pmat mutate --target src/ --output-format json > results.json
```

**Output**:
```json
{
  "mutation_score": 90.0,
  "total_mutants": 50,
  "killed": 45,
  "survived": 3,
  "compile_errors": 2,
  "timeouts": 0,
  "mutants": [
    {
      "location": "src/lib.rs:10:5",
      "mutation": "Changed + to -",
      "status": "Killed",
      "original_code": "a + b",
      "mutated_code": "a - b"
    }
  ]
}
```

### Markdown

Human-readable reports for pull requests:

```bash
pmat mutate --target src/ --output-format markdown > MUTATION_REPORT.md
```

**Output**:
```markdown
# Mutation Testing Report

**Mutation Score**: 90.0%

## Summary

| Metric | Count | Percentage |
|--------|-------|------------|
| Total Mutants | 50 | 100% |
| Killed | 45 | 90.0% |
| Survived | 3 | 6.0% |
| Compile Errors | 2 | 4.0% |

## Survived Mutants

| Location | Mutation | Code |
|----------|----------|------|
| src/lib.rs:15:9 | Changed > to >= | `if a > b` |
```

---

## Advanced Features

### Failures-Only Mode

Show only survived mutants (CI/CD optimization):

```bash
pmat mutate --target src/ --failures-only
```

**Use Case**: Reduce noise in CI logs by showing only what needs attention.

### Threshold Enforcement

Fail command if mutation score below threshold:

```bash
pmat mutate --target src/ --threshold 85
```

Exit codes:
- `0`: Passed (score >= threshold)
- `1`: Failed (score < threshold)

**Use Case**: Quality gate in CI/CD pipelines.

### Parallel Execution

Speed up mutation testing with multiple threads:

```bash
pmat mutate --target src/ --jobs 8
```

**Default**: Number of CPU cores
**Recommendation**: Use 2-4 jobs to avoid resource contention

### Timeout Control

Prevent infinite loops from hanging tests:

```bash
pmat mutate --target src/ --timeout 30
```

**Default**: 60 seconds per mutant
**Recommendation**: Set to 2-3× your average test runtime

### Combining Options

Production CI/CD command:

```bash
pmat mutate \
  --target src/ \
  --threshold 85 \
  --failures-only \
  --output-format json \
  --jobs 4 \
  --timeout 30 \
  > mutation-results.json
```

---

## Interpreting Results

### Mutation Score Thresholds

| Score | Quality Level | Action |
|-------|---------------|--------|
| **90-100%** | Excellent | Maintain quality |
| **80-89%** | Good | Improve critical paths |
| **70-79%** | Acceptable | Review test coverage |
| **Below 70%** | Poor | Significant gaps exist |

### Survived Mutants Analysis

**Example Survived Mutant**:
```
src/lib.rs:15:9 - Changed > to >=
Original: if a > b
Mutated:  if a >= b
```

**Why it Survived**: Missing test case for equality condition

**Fix**: Add test case:
```rust
#[test]
fn test_max_equal() {
    assert_eq!(max(5, 5), 5);  // Tests a == b case
}
```

### Common Mutation Patterns

**Boundary Conditions**:
- `>` → `>=`, `<` → `<=`
- **Fix**: Test boundary values (e.g., 0, -1, max values)

**Arithmetic Operators**:
- `+` → `-`, `*`, `/`
- **Fix**: Test with diverse inputs (positive, negative, zero)

**Boolean Logic**:
- `&&` → `||`, `true` → `false`
- **Fix**: Test all logical paths

**Return Values**:
- `return x` → `return null`, `return 0`
- **Fix**: Assert exact return values

---

## Workflow Integration

### Local Development

```bash
# Before committing
cargo test  # Verify tests pass
pmat mutate --target src/ --failures-only  # Check test quality
```

### Pre-Commit Hook

`.git/hooks/pre-commit`:
```bash
#!/bin/bash
pmat mutate --target src/ --threshold 80 --failures-only
if [ $? -ne 0 ]; then
    echo "Mutation score below 80%, commit blocked"
    exit 1
fi
```

### CI/CD Integration

See dedicated guides:
- [GitHub Actions Integration](../ci-cd/github-actions-integration.md)
- [GitLab CI Integration](../ci-cd/gitlab-ci-integration.md)
- [Jenkins Integration](../ci-cd/jenkins-integration.md)

### Pull Request Workflow

1. Developer runs mutation testing locally
2. CI pipeline runs full mutation testing on PR
3. Bot comments mutation score on PR
4. Reviewer checks survived mutants
5. Developer adds tests to kill survived mutants
6. Merge when mutation score meets threshold

---

## Troubleshooting

### Long Test Runtime

**Symptom**: Mutation testing takes hours

**Solutions**:
```bash
# Reduce timeout
pmat mutate --target src/ --timeout 10

# Increase parallelism
pmat mutate --target src/ --jobs 8

# Test specific module
pmat mutate --target src/auth/ --failures-only
```

### High Memory Usage

**Symptom**: Out of memory errors

**Solutions**:
```bash
# Reduce parallelism
pmat mutate --target src/ --jobs 1

# Test smaller modules separately
pmat mutate --target src/module1.rs
pmat mutate --target src/module2.rs
```

### Flaky Tests

**Symptom**: Different mutation scores on each run

**Solutions**:
1. Fix non-deterministic tests (timestamps, random values)
2. Use fixed seeds for random generators
3. Mock external dependencies

### Compile Errors

**Symptom**: Many mutants show compile errors

**Cause**: PMAT's mutations sometimes create invalid syntax

**Solution**: This is expected. Only **valid mutants** count toward mutation score.

### Timeouts

**Symptom**: Many mutants timeout

**Solutions**:
```bash
# Increase timeout
pmat mutate --target src/ --timeout 120

# Investigate infinite loops in code
# Check for while loops without proper exit conditions
```

---

## FAQ

### Q: How long does mutation testing take?

**A**: Depends on test suite size and code complexity.
- Small project (100 tests): 5-10 minutes
- Medium project (500 tests): 30-60 minutes
- Large project (2000+ tests): 2-4 hours

**Optimization**: Use `--failures-only` and parallel execution.

### Q: What's a good mutation score target?

**A**: Depends on code criticality:
- **Authentication/Security**: 95-100%
- **Business Logic**: 85-95%
- **Utilities**: 80-85%
- **UI Components**: 70-80%

### Q: Should I aim for 100% mutation score?

**A**: Not always. Some mutations may be:
- **Equivalent**: Behaviorally identical to original
- **Infeasible**: Impossible to kill without brittle tests
- **Low Value**: Testing error messages, logging

Aim for **85-95%** for most codebases.

### Q: How does mutation testing differ from code coverage?

**A**:
- **Code Coverage**: Measures which lines are executed
- **Mutation Testing**: Measures if tests detect changes

You can have 100% coverage with zero mutations killed!

### Q: Can PMAT mutate test files?

**A**: No. PMAT only mutates source code, not tests. Mutating tests would be counterproductive.

### Q: How does PMAT compare to other mutation testing tools?

**A**:
- **cargo-mutants** (Rust only): PMAT supports multi-language
- **Stryker** (JavaScript): PMAT has tighter CLI integration
- **PITest** (Java): PMAT is faster for small projects

### Q: What if a mutant can't be killed?

**A**: Evaluate whether:
1. Test is missing (add test)
2. Mutation is equivalent (ignore it)
3. Code is dead/unreachable (remove it)

### Q: Can I exclude files from mutation testing?

**A**: Yes, specify exact targets:
```bash
pmat mutate --target src/important_module.rs
```

Or use shell globbing:
```bash
for file in src/{auth,api,core}/*.rs; do
    pmat mutate --target "$file"
done
```

### Q: Does mutation testing work with integration tests?

**A**: Yes! PMAT runs your test suite (unit + integration) for each mutant.

### Q: How do I interpret "Compile Error" mutants?

**A**: These are mutations that create invalid syntax. They're excluded from the mutation score calculation.

---

## Additional Resources

### Example Projects
- [Rust Mutation Testing Example](../../examples/rust-mutation-testing/)
- [Python Mutation Testing Example](../../examples/python-mutation-testing/)
- [TypeScript Mutation Testing Example](../../examples/typescript-mutation-testing/)

### Guides
- [Mutation Testing Best Practices](./mutation-testing-best-practices.md)
- [Mutation Testing API Reference](./mutation-testing-api-reference.md)

### CI/CD Integration
- [GitHub Actions Integration](../ci-cd/github-actions-integration.md)
- [GitLab CI Integration](../ci-cd/gitlab-ci-integration.md)
- [Jenkins Integration](../ci-cd/jenkins-integration.md)

### Further Reading
- [Mutation Testing: An Empirical Evaluation (ACM)](https://dl.acm.org/doi/10.1145/3183440)
- [PMAT Main Documentation](../../README.md)

---

**Version**: v2.177.0
**Last Updated**: October 28, 2025
**Sprint**: Sprint 64 Day 3
