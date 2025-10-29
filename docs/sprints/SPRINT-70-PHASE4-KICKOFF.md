# Sprint 70 Phase 4: Comprehensive Testing - KICKOFF

**Phase**: 4/7
**Goal**: Validate parser fix with comprehensive test coverage
**Duration**: 2-3 hours
**Status**: 🚀 STARTING
**Date**: October 29, 2025

---

## Context

### Current State

**Completed in Session 4**:
- ✅ Phase 2 parser rewritten for actual cargo-mutants v25.3.1 format
- ✅ Phase 3 backend updated for directory-based output
- ✅ End-to-end workflow validated manually (5 mutants, 80% score)

**Problem**:
- Existing tests use deprecated `from_json()` API
- Tests marked with `#[ignore]` and haven't been updated
- Need comprehensive coverage for edge cases
- No integration tests with real cargo-mutants output

**Solution**:
- Update all tests to use new `from_output_dir()` API
- Add integration tests with real cargo-mutants output files
- Cover edge cases: empty projects, timeouts, large projects
- Remove `#[ignore]` from passing tests

---

## Phase Goal

**Primary**: Ensure parser fix has thorough test coverage before moving to documentation.

**Success Criteria**:
- ✅ All tests use new `from_output_dir()` API
- ✅ Integration tests with real cargo-mutants output
- ✅ Edge cases covered (empty, all caught, timeouts, large projects)
- ✅ No `#[ignore]` on passing tests
- ✅ 100% test pass rate
- ✅ Zero test warnings

---

## Implementation Plan

### Task 1: Create Test Fixtures (30 min)

**Goal**: Real cargo-mutants output for integration tests

**Steps**:
1. Create directory structure:
   ```
   server/tests/fixtures/cargo-mutants-output/
   ├── empty/              # No mutants found
   ├── all-caught/         # 100% score
   ├── some-missed/        # 80% score (current test case)
   ├── with-timeout/       # Contains timeout mutants
   ├── unviable/           # Contains unviable mutants
   └── large-project/      # >50 mutants
   ```

2. Copy actual `outcomes.json` from real cargo-mutants runs
3. Sanitize if needed (remove absolute paths)
4. Document fixture sources in README

**Acceptance Criteria**:
- ✅ 5-6 test fixtures with real cargo-mutants output
- ✅ Each fixture has `outcomes.json` at minimum
- ✅ Fixtures cover all outcome types
- ✅ README documents fixture sources

### Task 2: Update Unit Tests (45 min)

**Goal**: Migrate existing tests from `from_json()` to `from_output_dir()`

**Files to Update**:
- `server/tests/mutate_command_tests.rs` (12 tests)
- `server/tests/cargo_mutants_integration_test.rs` (integration tests)

**Changes Needed**:

**Before**:
```rust
#[test]
#[ignore]
fn test_cargo_mutants_backend_executes_and_parses() {
    let mock_json = r#"{"mutants": [...]}"#;
    let report = CargoMutantsReport::from_json(mock_json).unwrap();
    assert_eq!(report.mutants.len(), 1);
}
```

**After**:
```rust
#[test]
fn test_cargo_mutants_backend_executes_and_parses() {
    let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/some-missed");
    let report = CargoMutantsReport::from_output_dir(&fixture).unwrap();
    assert_eq!(report.mutants.len(), 5);
    assert_eq!(report.count_by_outcome(MutantOutcome::Caught), 4);
    assert_eq!(report.count_by_outcome(MutantOutcome::Missed), 1);
}
```

**Acceptance Criteria**:
- ✅ All tests use `from_output_dir()` with fixture paths
- ✅ No tests use deprecated `from_json()`
- ✅ Assertions match fixture contents
- ✅ Remove `#[ignore]` from updated tests
- ✅ All tests compile

### Task 3: Add Edge Case Tests (45 min)

**Goal**: Cover scenarios not in current test suite

**New Tests**:

1. **Empty Project** (no mutants):
   ```rust
   #[test]
   fn test_empty_project_no_mutants() {
       let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/empty");
       let report = CargoMutantsReport::from_output_dir(&fixture).unwrap();
       assert_eq!(report.mutants.len(), 0);
       assert_eq!(report.mutation_score(), 0.0);
   }
   ```

2. **Perfect Score** (all caught):
   ```rust
   #[test]
   fn test_all_mutants_caught() {
       let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/all-caught");
       let report = CargoMutantsReport::from_output_dir(&fixture).unwrap();
       assert_eq!(report.mutation_score(), 100.0);
       assert_eq!(report.count_by_outcome(MutantOutcome::Missed), 0);
   }
   ```

3. **Timeout Handling**:
   ```rust
   #[test]
   fn test_timeout_mutants() {
       let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/with-timeout");
       let report = CargoMutantsReport::from_output_dir(&fixture).unwrap();
       assert!(report.count_by_outcome(MutantOutcome::Timeout) > 0);
   }
   ```

4. **Unviable Mutants**:
   ```rust
   #[test]
   fn test_unviable_mutants() {
       let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/unviable");
       let report = CargoMutantsReport::from_output_dir(&fixture).unwrap();
       assert!(report.count_by_outcome(MutantOutcome::Unviable) > 0);
   }
   ```

5. **Large Project**:
   ```rust
   #[test]
   fn test_large_project_performance() {
       let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/large-project");
       let start = Instant::now();
       let report = CargoMutantsReport::from_output_dir(&fixture).unwrap();
       let elapsed = start.elapsed();

       assert!(report.mutants.len() > 50);
       assert!(elapsed.as_millis() < 100); // Parsing should be fast
   }
   ```

6. **Missing File Handling**:
   ```rust
   #[test]
   fn test_missing_outcomes_json() {
       let fixture = PathBuf::from("tests/fixtures/nonexistent");
       let result = CargoMutantsReport::from_output_dir(&fixture);
       assert!(result.is_err());
       assert!(result.unwrap_err().to_string().contains("outcomes.json"));
   }
   ```

**Acceptance Criteria**:
- ✅ 6+ new edge case tests
- ✅ All edge cases covered
- ✅ Error handling tested
- ✅ Performance validated
- ✅ All tests passing

### Task 4: Integration Test Enhancement (30 min)

**Goal**: Ensure integration tests use real output

**File**: `server/tests/cargo_mutants_integration_test.rs`

**Updates**:
1. Replace mock JSON with fixture paths
2. Test full workflow: detection → execution → parsing → display
3. Verify statistics calculation accuracy
4. Test threshold enforcement

**Example**:
```rust
#[test]
fn test_end_to_end_workflow_with_threshold() {
    let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/some-missed");

    // Parse report
    let report = CargoMutantsReport::from_output_dir(&fixture).unwrap();

    // Verify statistics
    assert_eq!(report.mutation_score(), 80.0);

    // Test threshold enforcement
    let threshold = 70.0;
    assert!(report.mutation_score() >= threshold, "Score below threshold");

    let threshold_fail = 85.0;
    assert!(report.mutation_score() < threshold_fail, "Should fail threshold");
}
```

**Acceptance Criteria**:
- ✅ Integration tests use fixtures
- ✅ End-to-end workflow tested
- ✅ Statistics accuracy verified
- ✅ Threshold logic tested
- ✅ All integration tests passing

### Task 5: Verification & Cleanup (30 min)

**Goal**: Final validation and cleanup

**Steps**:
1. Run full test suite: `cargo test --workspace`
2. Verify 100% pass rate
3. Check for warnings: `cargo test --workspace 2>&1 | grep warning`
4. Run clippy on tests: `cargo clippy --tests`
5. Remove any remaining `#[ignore]` attributes
6. Update test documentation
7. Verify test coverage metrics

**Acceptance Criteria**:
- ✅ All tests passing (0 failures)
- ✅ Zero test warnings
- ✅ Zero clippy issues
- ✅ No ignored tests (unless documented)
- ✅ Test documentation updated

---

## Files to Create/Modify

### New Files
- `server/tests/fixtures/cargo-mutants-output/*/outcomes.json` (5-6 fixtures)
- `server/tests/fixtures/cargo-mutants-output/README.md` (fixture documentation)

### Modified Files
- `server/tests/mutate_command_tests.rs` (update 12 tests)
- `server/tests/cargo_mutants_integration_test.rs` (update integration tests)
- `server/src/services/mutation/json_parser.rs` (possible test additions)

**Estimated Total**: ~200 lines (test updates + fixtures)

---

## Testing Strategy

### Test Categories

1. **Unit Tests** (fast, isolated):
   - JSON parsing logic
   - Outcome mapping
   - Statistics calculation
   - Error handling

2. **Integration Tests** (slower, real fixtures):
   - Full workflow with fixtures
   - Threshold enforcement
   - Statistics display

3. **Edge Case Tests** (corner cases):
   - Empty projects
   - Perfect scores
   - Timeouts and unviable mutants
   - Large projects

### Test Execution Plan

**Fast Tests** (unit):
```bash
cargo test json_parser --lib
```

**Integration Tests**:
```bash
cargo test cargo_mutants_integration --test '*'
```

**All Tests**:
```bash
cargo test --workspace
```

---

## Success Criteria

### Functional
- ✅ All tests use new `from_output_dir()` API
- ✅ Integration tests with real cargo-mutants output
- ✅ Edge cases covered (6+ new tests)
- ✅ 100% test pass rate
- ✅ No ignored tests (except documented reasons)

### Quality
- ✅ Zero test warnings
- ✅ Zero clippy issues in tests
- ✅ Test execution time <5 seconds
- ✅ Clear test names and documentation

### Coverage
- ✅ All outcome types tested (Caught, Missed, Timeout, Unviable)
- ✅ Error conditions tested
- ✅ Performance validated (large projects)
- ✅ Threshold logic tested

---

## Potential Issues

### Issue 1: Creating Realistic Fixtures

**Problem**: Need real cargo-mutants output for fixtures.

**Solution**: Use `/tmp/mutants-test/mutants.out/mutants.out/outcomes.json` as base, modify for edge cases.

### Issue 2: Test Flakiness

**Problem**: Tests might be sensitive to exact mutant counts.

**Solution**: Use range assertions or focus on percentages/ratios.

### Issue 3: Large Fixture Size

**Problem**: Large project fixtures might bloat repository.

**Solution**: Use minimal but representative fixtures (10-20 mutants), document generation process.

---

## Timeline

**Total Estimated Time**: 2-3 hours

**Breakdown**:
- Task 1 (Fixtures): 30 min
- Task 2 (Update Tests): 45 min
- Task 3 (Edge Cases): 45 min
- Task 4 (Integration): 30 min
- Task 5 (Verification): 30 min

**Target Completion**: Same session (October 29, 2025)

---

## Next Phase

After Phase 4 completion:
- **Phase 5**: Documentation (user guide, troubleshooting)
- **Phase 6**: Performance validation (benchmarks, large projects)
- **Phase 7**: Release preparation (CHANGELOG, version bump)

---

## References

- Session 4 fix: Commits 4fe05dcf, acf3c233
- Test fixtures: `/tmp/mutants-test/mutants.out/mutants.out/`
- Phase 3 completion: `docs/sprints/SPRINT-70-PHASE3-COMPLETION.md`
- Phase 4 will be: `docs/sprints/SPRINT-70-PHASE4-COMPLETION.md`
