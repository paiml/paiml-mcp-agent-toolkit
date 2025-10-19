# Sprint 41b: Test Triage Plan - EXTREME TDD + FAST

**Sprint**: 41b (2025-10-19)
**Objective**: Re-enable 15-20 ignored tests (from 83 total)
**Methodology**: EXTREME TDD + FAST (Mutation, Property, Fuzz, PMAT)
**Estimated Time**: 6-8 hours

## Executive Summary

We have 83 documented ignored/failing tests across 15 categories. Based on CLAUDE.md documentation and actual test analysis, we'll triage them into 3 categories and focus on re-enabling Category A (quick wins).

## Test Inventory (83 Total)

### By Category (from CLAUDE.md)
1. Language-Specific Tests: 4 tests
2. Infrastructure Tests: 7 tests
3. Binary Integration Tests: 1 test
4. End-to-End Tests: 4 tests
5. CLI and Quality Tests: 2 tests
6. Annotation TDD Tests: 7 tests
7. Unified Quality Framework: 14 tests
8. Language Detection Tests: 5 tests
9. Enhanced Naming Tests: 6 tests
10. Unified Context Tests: 4 tests
11. TypeScript/JavaScript Tests: 3 tests
12. Real-World/Performance Tests: 5 tests
13. Integration Tests: 1 test
14. Timeout Integration Tests: 3 tests
15. Ruchy Parser Tests: 10 tests

**Failing Tests** (documented separately): 14 tests
- Configuration Service: 1 test
- Deep WASM Service: 3 tests
- Defect Report Service: 5 tests
- CLI Integration: 3 tests
- Mutation Test: 1 test
- Kotlin Test: 1 test

## Triage Categories

### Category A: Quick Wins (Target: 15-20 tests)
**Criteria**: Low effort, high value, likely already working or simple fixes
**Estimated Time**: 20-40 min each

**Priority 1 - Likely Already Passing** (Test them first!):
1. ✅ Language Regression Tests (6 tests) - **ALREADY PASSING** (Sprint 36)
   - test_c_deep_context_analysis
   - test_cpp_deep_context_analysis
   - test_bash_deep_context_analysis
   - test_php_deep_context_analysis
   - test_swift_deep_context_analysis
   - test_wasm_deep_context_analysis

2. Infrastructure Tests (subset - 3 tests):
   - test_concurrent_access - May need tokio runtime setup
   - test_config_from_file - Likely needs test fixture
   - test_operation_profiling - Likely needs test data

3. CLI and Quality Tests (2 tests):
   - test_optional_argument_coercion - Type coercion test
   - test_complexity_violation_detection - Quality check test

4. Integration Tests (1 test):
   - test_context_markdown_output - Output format may have changed

**Total Category A Estimate**: 12 tests (6 likely passing + 6 quick fixes)

### Category B: Medium Effort (Defer to Sprint 42)
**Criteria**: Require implementation or significant setup
**Estimated Time**: 1-3 hours each

1. Unified Quality Framework (14 tests) - Property-based tests
2. Language Detection Tests (5 tests) - Need detection logic fixes
3. Enhanced Naming Tests (6 tests) - Require Phase 2 implementation
4. Unified Context Tests (4 tests) - Require implementation
5. TypeScript/JavaScript Tests (3 tests) - Need implementation

**Total Category B**: 32 tests

### Category C: Future Work (Defer to Later Sprints)
**Criteria**: Require major features, binary dependencies, or Phase 2 work
**Estimated Time**: 4+ hours each or blocked by dependencies

1. Annotation TDD Tests (7 tests) - Require pmat binary
2. Timeout Integration Tests (3 tests) - Require binary
3. Binary Integration Test (1 test) - Compilation timeout
4. Ruchy Parser Tests (10 tests) - Require ruchy-ast feature
5. Real-World/Performance Tests (5 tests) - Require proper setup
6. End-to-End Tests (4 tests) - AST parsing issues
7. Language-Specific (4 tests) - Kotlin/WASM complexity
8. Infrastructure (4 tests) - TDG dashboard/profiling

**Total Category C**: 38 tests

## Sprint 41b Execution Plan

### Phase 1: Verification (30 min)
**Objective**: Quickly test Category A Priority 1 to confirm they're passing

```bash
# Test language regression tests
cargo test language_regression_tests:: --lib -- --ignored

# Expected: 6/6 passing (based on Sprint 36 work)
```

### Phase 2: Re-enable Passing Tests (30 min)
**Objective**: Remove #[ignore] from tests that pass

For each passing test:
1. Find source file
2. Remove `#[ignore]` annotation
3. Add comment: `// Re-enabled Sprint 41b - verified passing`
4. Run test individually to confirm
5. Apply FAST methodology (property test if applicable)

### Phase 3: Fix Quick Wins (2-3 hours)
**Objective**: Fix 6-8 additional tests from Category A

For each test:
1. **RED**: Run test, understand failure
2. **GREEN**: Make minimal fix to pass
3. **REFACTOR**: Clean up code
4. **FAST**: Run mutation/property tests
5. **PMAT**: Analyze complexity with `pmat analyze`

### Phase 4: Documentation (30 min)
**Objective**: Update CLAUDE.md and create completion report

## Success Criteria

| Criterion | Target | Status |
|-----------|--------|--------|
| Tests Re-enabled | 15-20 | 📋 Pending |
| Category A Coverage | 100% | 📋 Pending |
| All Re-enabled Tests Pass | 100% | 📋 Pending |
| FAST Verification | 100% | 📋 Pending |
| Documentation Updated | Yes | 📋 Pending |

## EXTREME TDD + FAST Methodology

### For Each Test Re-enable:

**RED Phase**:
```bash
# Run test to see failure
cargo test test_name --lib -- --exact --nocapture
```

**GREEN Phase**:
```bash
# Make minimal fix
# Re-run test
cargo test test_name --lib -- --exact
```

**REFACTOR Phase**:
```bash
# Clean up code
# Ensure still passing
cargo test test_name --lib -- --exact
```

**FAST Phase**:
```bash
# Mutation testing (if applicable)
cargo mutants --file path/to/file.rs --timeout 60

# Property testing (if applicable - proptest)
# Already included in some tests

# Fuzzing (if applicable - cargo-fuzz)
# Defer to specific cases

# PMAT Analysis
pmat analyze path/to/file.rs --format json
```

## Risk Mitigation

### Risk 1: Tests Still Fail
**Mitigation**: Move to Category B, don't force it
**Impact**: Low - we have 83 tests, plenty of candidates

### Risk 2: Time Overrun
**Mitigation**: Stop at 15 tests if approaching 6 hours
**Impact**: Medium - defer remaining to Sprint 42

### Risk 3: Tests Break Other Tests
**Mitigation**: Run full test suite after each batch of 5
**Impact**: High - use `cargo test --lib` frequently

## Next Actions

1. ✅ Create triage plan (this document)
2. 📋 Run Phase 1: Verification
3. 📋 Execute Phase 2: Re-enable passing tests
4. 📋 Execute Phase 3: Fix quick wins
5. 📋 Execute Phase 4: Documentation
6. 📋 Create Sprint 41b completion report

---

**Status**: PLANNED
**Next**: Execute Phase 1 (Verification)
**Estimated Completion**: 2025-10-19 EOD
