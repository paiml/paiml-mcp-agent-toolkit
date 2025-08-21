# Quality Standards

*Reference: SPECIFICATION.md Sections 31-33*

## Overview

PMAT enforces extreme quality standards through automated gates and zero-tolerance policies. This document defines our quality requirements and enforcement mechanisms.

## Zero SATD Policy (Section 31)

### Definition

Self-Admitted Technical Debt (SATD) includes any comment containing:
- `TODO`, `FIXME`, `HACK`, `XXX`, `BUG`
- `KLUDGE`, `REFACTOR`, `DEPRECATED`
- Phrases like "temporary", "for now", "quick fix"

### Enforcement

```rust
// Compile-time enforcement
#[cfg(any(feature = "strict", not(debug_assertions)))]
compile_error_if!(satd_count() > 0, "SATD detected - zero tolerance policy");
```

### Pre-commit Hooks

```bash
# Automatically blocks commits with SATD
./scripts/pre-commit
```

### Exceptions

No exceptions. All SATD must be:
1. Converted to GitHub issues
2. Removed from code
3. Tracked in roadmap if needed

## Complexity Limits (Section 32)

### Thresholds

| Metric | Maximum | Enforcement |
|--------|---------|-------------|
| Cyclomatic Complexity | 20 | Hard fail |
| Cognitive Complexity | 15 | Hard fail |
| Function Length | 50 lines | Warning |
| File Length | 500 lines | Warning |
| Nesting Depth | 5 levels | Hard fail |

### Measurement

Using McCabe (1976) for cyclomatic and Campbell (2018) for cognitive:

```bash
# Check single file
pmat analyze complexity src/main.rs --max-cyclomatic 20

# Check entire project
pmat quality-gate --max-complexity-p99 20
```

### Refactoring

When complexity exceeds limits:

```bash
# Automatic refactoring
pmat refactor auto --file src/complex.rs --target-complexity 15

# Manual guidance
pmat refactor suggest --file src/complex.rs
```

## Documentation Requirements (Section 33)

### Mandatory Documentation

Every public item requires:
1. **Summary**: One-line description
2. **Details**: Extended explanation if needed
3. **Examples**: At least one usage example
4. **Safety**: For unsafe code
5. **Panics**: Conditions that cause panics
6. **Errors**: Possible error returns

### Documentation Coverage

```rust
#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
```

### Doctest Requirements

All examples must be executable:

```rust
/// Analyzes code complexity
/// 
/// # Example
/// ```
/// use pmat::analyze_complexity;
/// 
/// let code = "fn main() { println!(\"Hello\"); }";
/// let result = analyze_complexity(code);
/// assert!(result.cyclomatic <= 1);
/// ```
pub fn analyze_complexity(code: &str) -> ComplexityResult {
    // ...
}
```

Run doctests:
```bash
make test-doc
```

## Quality Gate Integration

### Configuration

In `pmat.toml`:
```toml
[quality]
max_cyclomatic = 20
max_cognitive = 15
max_satd = 0
min_coverage = 80
require_docs = true
```

### CI/CD Integration

```yaml
# .github/workflows/quality.yml
- name: Quality Gate
  run: |
    pmat quality-gate --fail-on-violation
    make lint
    make test
```

### Pre-commit Enforcement

Automatic quality checks on every commit:
1. SATD detection
2. Complexity analysis
3. Lint violations
4. Test failures

## Continuous Improvement

### Metrics Tracking

Track quality trends in `docs/execution/velocity.json`:
- Complexity reduction over time
- SATD elimination rate
- Coverage improvements
- Defect density

### Regular Audits

Weekly quality reviews:
```bash
# Generate quality report
pmat analyze quality --format markdown > docs/quality/weekly-report.md

# Identify improvement areas
pmat analyze lint-hotspot --top-files 5
```

## Escalation

Quality violations block:
1. Local commits (pre-commit hook)
2. CI builds (quality gate)
3. Releases (release checklist)

No overrides without:
1. Architecture review
2. Documented exception
3. Remediation plan

## Related Documentation

- [Quality Gates](../execution/quality-gates.md)
- [Toyota Way Principles](../CLAUDE.md)
- [SPECIFICATION.md Section 31-33](../SPECIFICATION.md#31-zero-satd-policy)