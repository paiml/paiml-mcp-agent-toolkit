# TICKET-3002: Refactor High Entropy Violations

**Sprint**: 13 - Technical Debt Reduction
**Priority**: High
**Estimated Effort**: 5 hours
**Status**: Ready for Development
**Methodology**: EXTREME TDD

## Problem Statement

52 code entropy violations detected across the codebase, indicating functions with high information density that are difficult to understand and maintain. These violations reduce code quality and increase the risk of bugs.

### Current State

**Top Offenders:**
1. `unified_quality/enforcer.rs`: 15 violations (entropy: 8.9-12.5)
2. `unified_quality/enhanced_parser.rs`: 8 violations (entropy: 7.8-11.2)
3. `services/simple_deep_context.rs`: 6 violations (entropy: 8.1-9.7)
4. `tdg/analyzer_simple.rs`: 5 violations
5. Other files: 18 violations across 12 files

**Entropy Thresholds:**
- **Low**: < 7.0 (acceptable)
- **Medium**: 7.0-8.5 (needs attention)
- **High**: 8.5-10.0 (must fix)
- **Critical**: > 10.0 (immediate action required)

## Goal

Reduce entropy violations by 80% (from 52 to <11) using the Extract Method pattern and other refactoring techniques while maintaining 100% test coverage.

## Implementation Strategy

### Phase 1: Identify High-Entropy Functions (1 hour)

**Deliverables:**
- Run `pmat analyze complexity --path server/src` and capture all entropy violations
- Create CSV report with: file, function, entropy score, lines of code
- Prioritize by entropy score (highest first)

**RED Phase Tests:**
```rust
#[test]
fn red_test_entropy_report_captures_all_violations() {
    // Test that we can identify all 52 violations
}

#[test]
fn red_test_entropy_prioritization_by_score() {
    // Test that violations are sorted by entropy score
}
```

### Phase 2: Refactor `unified_quality/enforcer.rs` (1.5 hours)

**Target**: 15 violations → 3 violations (80% reduction)

**Refactoring Techniques:**
1. **Extract Method**: Break down large functions into smaller, focused functions
2. **Extract Variable**: Replace complex expressions with named variables
3. **Replace Conditional with Polymorphism**: Use trait objects for complex branching
4. **Introduce Parameter Object**: Group related parameters

**RED Phase Tests:**
```rust
#[test]
fn red_test_enforcer_functions_below_entropy_threshold() {
    // After refactoring, all functions should have entropy < 8.5
}

#[test]
fn red_test_enforcer_maintains_correctness() {
    // Property test: refactored code produces same results
}

#[test]
fn red_test_enforcer_maintains_performance() {
    // Benchmark: refactored code is not slower
}
```

**Example Refactoring:**

Before (High Entropy):
```rust
fn evaluate_quality(files: Vec<File>) -> Report {
    let mut report = Report::new();
    for file in files {
        if file.ext == "rs" {
            if file.lines > 1000 {
                report.add_warning("Large file");
                if file.complexity > 50 {
                    report.add_error("High complexity in large file");
                }
            }
            let satd = detect_satd(&file);
            if satd.count > 5 {
                report.add_warning("High SATD");
            }
        }
    }
    report
}
```

After (Low Entropy):
```rust
fn evaluate_quality(files: Vec<File>) -> Report {
    files.into_iter()
        .flat_map(|file| evaluate_file_quality(file))
        .collect()
}

fn evaluate_file_quality(file: File) -> Vec<Violation> {
    let mut violations = Vec::new();
    violations.extend(check_file_size(&file));
    violations.extend(check_complexity(&file));
    violations.extend(check_satd(&file));
    violations
}

fn check_file_size(file: &File) -> Option<Violation> {
    (file.lines > 1000).then(|| Violation::warning("Large file"))
}

fn check_complexity(file: &File) -> Option<Violation> {
    (file.complexity > 50).then(|| Violation::error("High complexity"))
}

fn check_satd(file: &File) -> Option<Violation> {
    let satd = detect_satd(file);
    (satd.count > 5).then(|| Violation::warning("High SATD"))
}
```

### Phase 3: Refactor `unified_quality/enhanced_parser.rs` (1 hour)

**Target**: 8 violations → 2 violations (75% reduction)

**Focus Areas:**
- Complex parsing logic with nested conditionals
- Large pattern matching expressions
- Tightly coupled parsing and analysis

**RED Phase Tests:**
```rust
#[test]
fn red_test_parser_functions_below_entropy_threshold() {
    // All functions should have entropy < 8.5
}

#[test]
fn red_test_parser_outputs_match_original() {
    // Property test: refactored parser produces identical AST
}
```

### Phase 4: Refactor `services/simple_deep_context.rs` (1 hour)

**Target**: 6 violations → 1 violation (83% reduction)

**Key Function:**
- `generate_deep_context`: CC=45, Cog=112, High Entropy
  - Extract language-specific analysis into separate functions
  - Extract markdown generation into separate module
  - Use builder pattern for context construction

**RED Phase Tests:**
```rust
#[test]
fn red_test_deep_context_functions_below_entropy_threshold() {
    // All functions should have entropy < 8.5
}

#[test]
fn red_test_deep_context_output_identical() {
    // Integration test: refactored code produces same markdown
}
```

### Phase 5: Address Remaining Files (0.5 hours)

**Target**: 23 violations → 5 violations (78% reduction)

**Files:**
- `tdg/analyzer_simple.rs`: 5 violations
- 12 other files: 18 violations

**Strategy:**
- Focus on functions with entropy > 10.0 first
- Apply Extract Method pattern consistently
- Document remaining acceptable violations

## EXTREME TDD Requirements

### RED Phase (Write Failing Tests First)

For each file being refactored:
1. **Entropy Test**: Function entropy must be < 8.5
2. **Correctness Test**: Refactored code produces identical output
3. **Performance Test**: No regression in performance
4. **Property Test**: Use proptest for edge cases

### GREEN Phase (Minimal Implementation)

1. Refactor code to pass all tests
2. No premature optimization
3. Focus on reducing entropy through simplification

### REFACTOR Phase (Code Quality)

1. Ensure consistent naming conventions
2. Add inline documentation for complex logic
3. Remove any duplication introduced during GREEN phase

## Success Criteria

### Must Have
- [ ] Entropy violations reduced from 52 to <11 (80% reduction)
- [ ] All refactored functions have entropy < 8.5
- [ ] 100% of existing tests still pass
- [ ] New tests for refactored code (minimum 3 per file)
- [ ] No performance regression (benchmark comparison)

### Should Have
- [ ] Code coverage maintained or improved
- [ ] Function cyclomatic complexity reduced as side effect
- [ ] Better separation of concerns in refactored modules

### Nice to Have
- [ ] 90% reduction in entropy violations (< 6 violations)
- [ ] All functions have entropy < 7.0
- [ ] Improved documentation with code examples

## Metrics

### Before Refactoring
```
Total Violations: 52
Critical (>10.0): 8
High (8.5-10.0): 12
Medium (7.0-8.5): 32
```

### Target After Refactoring
```
Total Violations: <11
Critical (>10.0): 0
High (8.5-10.0): 0
Medium (7.0-8.5): <11
```

### Measurement Commands
```bash
# Generate entropy report before
pmat analyze complexity --path server/src --output before.json

# After refactoring
pmat analyze complexity --path server/src --output after.json

# Compare
diff before.json after.json
```

## Risk Assessment

### Low Risk
- ✅ Extract Method is a safe refactoring technique
- ✅ Tests ensure correctness is maintained
- ✅ Can be done incrementally (one file at a time)

### Medium Risk
- ⚠️ Performance regression if not careful with abstractions
- ⚠️ May introduce new complexity if over-engineered

### Mitigation Strategies
1. **Benchmark Before/After**: Use criterion for precise measurements
2. **Incremental Commits**: Commit after each file is refactored
3. **Pair Review**: Have another developer review refactoring decisions
4. **Rollback Plan**: Keep old functions as deprecated for one release

## Timeline

**Total Estimated Time**: 5 hours

| Phase | Task | Time | Dependencies |
|-------|------|------|--------------|
| 1 | Identify Violations | 1h | None |
| 2 | Refactor enforcer.rs | 1.5h | Phase 1 |
| 3 | Refactor enhanced_parser.rs | 1h | Phase 1 |
| 4 | Refactor simple_deep_context.rs | 1h | Phase 1 |
| 5 | Address Remaining Files | 0.5h | Phase 1 |

**Sprint Duration**: 1 day with focus

## References

### Documentation
- [Code Entropy Definition](https://en.wikipedia.org/wiki/Entropy_(information_theory))
- [Martin Fowler's Refactoring Catalog](https://refactoring.com/catalog/)
- [Extract Method Pattern](https://refactoring.guru/extract-method)

### Related Tickets
- TICKET-3003: Refactor High Complexity Functions
- TICKET-3004: Clean Up Dead Code and SATD

---

**Created**: 2025-09-30
**Assigned**: TBD
**Methodology**: EXTREME TDD
**Sprint**: 13