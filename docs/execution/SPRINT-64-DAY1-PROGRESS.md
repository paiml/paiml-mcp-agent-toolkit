# Sprint 64 Day 1 Progress: Testing Infrastructure Analysis

**Sprint**: Sprint 64 - Testing, Examples, and Documentation
**Day**: Day 1 - Testing Infrastructure
**Date**: October 27, 2025
**Status**: 🔍 Analysis Phase Complete
**Target Version**: v2.177.0

---

## Overview

Day 1 focuses on building comprehensive test infrastructure for the mutation testing feature. This document tracks progress on implementing 50+ unit tests, 20+ integration tests, and 10+ property-based tests.

---

## Completed Tasks

### ✅ 1. Handler Structure Analysis

**File Analyzed**: `server/src/cli/handlers/mutate.rs` (280 lines)

**Components Identified**:

#### Main Handler (`handle()` function)
- **Target Validation**: Canonicalize file path, check existence
- **Engine Configuration**: MutationConfig with strategy, max_mutants, threads
- **Mutant Generation**: Generate mutants from file using engine
- **Execution Orchestration**: Parallel or sequential execution
- **Score Calculation**: Calculate mutation score from results
- **Output Formatting**: JSON, Markdown, or Text output
- **Threshold Checking**: Optional mutation score threshold enforcement

#### Execution Functions
1. **`execute_with_progress()`**
   - Parallel mutant execution with progress reporting
   - Uses `tokio::spawn` for background execution
   - 500ms polling interval for progress updates
   - Returns aggregated results

2. **`execute_sequential_with_progress()`**
   - Sequential mutant execution
   - Progress updates after each mutant
   - Useful for debugging and low-thread scenarios

3. **`print_progress()`**
   - Terminal progress bar (40 characters wide)
   - Shows completed/total and percentage
   - Uses `\r` for in-place updates

#### Output Functions
1. **`output_json()`**
   - JSON serialization with `serde_json`
   - Includes code snippets (original + mutated)
   - Supports `failures_only` filtering
   - Filters: Survived, CompileError, Timeout

2. **`output_markdown()`**
   - Markdown-formatted tables
   - Summary statistics
   - Individual mutant details
   - Code snippets in fenced blocks

3. **`output_text()`**
   - Color-coded terminal output (Sprint 62)
   - Green: Killed mutants
   - Red: Survived mutants
   - Yellow: Compile errors, timeouts
   - Cyan: File paths, operator names

4. **`extract_code_snippet()`**
   - Reads source file
   - Extracts lines based on SourceLocation
   - 1-indexed line numbers
   - Handles out-of-bounds gracefully

#### Helper Types
1. **`MutationTestOutput`**
   - Wrapper for JSON serialization
   - Contains `score` and `results` fields

2. **`EnhancedMutationResult`**
   - Extends `MutationResult` with code snippets
   - `original_code_snippet`: Option<String>
   - `mutated_code_snippet`: Option<String>

---

## Test Categories Identified

Based on handler analysis, 50+ tests needed across these categories:

### Category 1: Argument Validation (10 tests)
- ✅ Test target file not found error
- ✅ Test target directory instead of file error
- ✅ Test relative path canonicalization
- ✅ Test symlink resolution
- ✅ Test invalid threshold value (>100)
- ✅ Test negative threshold value
- ✅ Test invalid output format
- ✅ Test jobs parameter (0, 1, max)
- ✅ Test timeout parameter validation
- ✅ Test combined argument validation

### Category 2: Output Format Tests (12 tests)
- ✅ Test JSON output structure
- ✅ Test JSON with failures_only=true
- ✅ Test JSON with failures_only=false
- ✅ Test JSON code snippet inclusion
- ✅ Test Markdown output structure
- ✅ Test Markdown summary table
- ✅ Test Markdown mutant details
- ✅ Test Text output with colors
- ✅ Test Text output without colors (NO_COLOR)
- ✅ Test output format selection (json, markdown, text)
- ✅ Test empty results output
- ✅ Test large results output (>1000 mutants)

### Category 3: Filtering Logic (8 tests)
- ✅ Test failures_only filters Survived
- ✅ Test failures_only filters CompileError
- ✅ Test failures_only filters Timeout
- ✅ Test failures_only excludes Killed
- ✅ Test all mutants passed (no failures)
- ✅ Test all mutants failed (all failures)
- ✅ Test mixed results filtering
- ✅ Test filtering with empty results

### Category 4: Progress Indicators (6 tests)
- ✅ Test progress bar rendering (0%, 50%, 100%)
- ✅ Test progress bar width calculation
- ✅ Test progress with zero total mutants
- ✅ Test progress updates during execution
- ✅ Test parallel progress reporting
- ✅ Test sequential progress reporting

### Category 5: Code Snippet Extraction (8 tests)
- ✅ Test single-line snippet extraction
- ✅ Test multi-line snippet extraction
- ✅ Test out-of-bounds line numbers
- ✅ Test empty file snippet
- ✅ Test file read error handling
- ✅ Test Unicode content extraction
- ✅ Test large file snippet (>10k lines)
- ✅ Test snippet trimming behavior

### Category 6: Error Handling (10 tests)
- ✅ Test file not found error
- ✅ Test permission denied error
- ✅ Test invalid Rust syntax error
- ✅ Test engine initialization failure
- ✅ Test mutant generation failure
- ✅ Test execution timeout
- ✅ Test threshold failure error
- ✅ Test concurrent execution errors
- ✅ Test output serialization errors
- ✅ Test graceful degradation on errors

**Total Planned Unit Tests**: 54 tests

---

## Integration Test Categories Identified

### Category 1: End-to-End Workflow (8 tests)
- ✅ Test complete Rust mutation workflow
- ✅ Test complete Python mutation workflow (requires Python adapter)
- ✅ Test complete TypeScript mutation workflow (requires TS adapter)
- ✅ Test complete JavaScript mutation workflow
- ✅ Test complete Go mutation workflow (requires Go adapter)
- ✅ Test complete C++ mutation workflow (requires C++ adapter)
- ✅ Test multi-file project mutation
- ✅ Test workspace-level mutation

### Category 2: Performance and Scale (6 tests)
- ✅ Test large file (>1000 lines)
- ✅ Test many mutants (>500 mutants)
- ✅ Test parallel execution scaling (1, 2, 4, 8 threads)
- ✅ Test timeout handling
- ✅ Test memory usage bounds
- ✅ Test execution time bounds

### Category 3: Concurrent Execution (4 tests)
- ✅ Test parallel mutant execution correctness
- ✅ Test race condition handling
- ✅ Test resource contention
- ✅ Test graceful shutdown on error

### Category 4: Real-World Scenarios (4 tests)
- ✅ Test mutation of actual PMAT code
- ✅ Test mutation with failing tests
- ✅ Test mutation with no tests
- ✅ Test mutation with flaky tests

**Total Planned Integration Tests**: 22 tests

---

## Property-Based Test Categories Identified

Using `proptest` framework:

### Category 1: Invariants (4 properties)
- ✅ Mutation score always between 0.0 and 1.0
- ✅ Killed mutant count ≤ Total mutant count
- ✅ Sum of status counts equals total mutants
- ✅ Progress percentage never exceeds 100%

### Category 2: Determinism (3 properties)
- ✅ Same input file produces same mutants (given same seed)
- ✅ Mutant order is deterministic
- ✅ Score calculation is deterministic

### Category 3: Output Consistency (3 properties)
- ✅ JSON output is valid JSON
- ✅ Markdown output is valid Markdown
- ✅ All output formats contain same data

### Category 4: Correctness (2 properties)
- ✅ Generated mutants are syntactically valid
- ✅ Mutant locations are within file bounds

**Total Planned Property Tests**: 12 tests

**Grand Total: 88 tests across all categories**

---

## Technical Dependencies

### External Crates Needed
- ✅ `proptest` - Property-based testing (already in dev-dependencies)
- ✅ `tempfile` - Temporary file/directory creation (already available)
- ✅ `tokio-test` - Async test utilities (may need to add)
- ✅ `assert_matches` - Pattern matching in tests (may need to add)

### Test File Structure
```
server/tests/
├── mutation_handler_unit_tests.rs     (54 unit tests)
├── mutation_integration_tests.rs      (22 integration tests)
└── mutation_property_tests.rs         (12 property tests)
```

---

## Implementation Plan

### Phase 1: Unit Tests (Estimated: 2-3 hours)
1. Create `server/tests/mutation_handler_unit_tests.rs`
2. Implement Category 1: Argument Validation (10 tests)
3. Implement Category 2: Output Format Tests (12 tests)
4. Implement Category 3: Filtering Logic (8 tests)
5. Implement Category 4: Progress Indicators (6 tests)
6. Implement Category 5: Code Snippet Extraction (8 tests)
7. Implement Category 6: Error Handling (10 tests)
8. Run tests: `cargo test mutation_handler_unit_tests`

### Phase 2: Integration Tests (Estimated: 2-3 hours)
1. Create `server/tests/mutation_integration_tests.rs`
2. Implement Category 1: End-to-End Workflow (8 tests)
3. Implement Category 2: Performance and Scale (6 tests)
4. Implement Category 3: Concurrent Execution (4 tests)
5. Implement Category 4: Real-World Scenarios (4 tests)
6. Run tests: `cargo test mutation_integration_tests`

### Phase 3: Property-Based Tests (Estimated: 1-2 hours)
1. Create `server/tests/mutation_property_tests.rs`
2. Add `proptest` dependency (if not present)
3. Implement Category 1: Invariants (4 properties)
4. Implement Category 2: Determinism (3 properties)
5. Implement Category 3: Output Consistency (3 properties)
6. Implement Category 4: Correctness (2 properties)
7. Run tests: `cargo test mutation_property_tests`

### Phase 4: Coverage Analysis (Estimated: 30 minutes)
1. Run coverage: `cargo llvm-cov --all-features`
2. Analyze coverage report
3. Identify gaps in coverage
4. Add targeted tests for uncovered code
5. Verify >85% coverage goal achieved

**Total Estimated Time**: 6-9 hours (full day)

---

## Test Implementation Strategy

### Pattern 1: Arrange-Act-Assert
```rust
#[tokio::test]
async fn test_target_file_not_found() {
    // Arrange
    let args = MutateArgs {
        target: PathBuf::from("/nonexistent/file.rs"),
        ..Default::default()
    };
    let server = Arc::new(StatelessTemplateServer::new());

    // Act
    let result = handle(args, server).await;

    // Assert
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Target file not found"));
}
```

### Pattern 2: Property-Based Testing
```rust
proptest! {
    #[test]
    fn mutation_score_always_bounded(results: Vec<MutationResult>) {
        let score = MutationScore::from_results(&results);
        prop_assert!(score.score >= 0.0 && score.score <= 1.0);
    }
}
```

### Pattern 3: Integration Testing
```rust
#[tokio::test]
async fn test_rust_mutation_full_workflow() {
    // Create temporary Rust file
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, "fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();

    // Run mutation testing
    let args = MutateArgs {
        target: file_path.clone(),
        output_format: "json".to_string(),
        ..Default::default()
    };
    let server = Arc::new(StatelessTemplateServer::new());
    let result = handle(args, server).await;

    // Verify success
    assert!(result.is_ok());
}
```

---

## Challenges and Solutions

### Challenge 1: Async Testing
**Problem**: Handler is async, requires tokio runtime
**Solution**: Use `#[tokio::test]` macro for async tests

### Challenge 2: File System Dependencies
**Problem**: Tests need real files, can interfere with each other
**Solution**: Use `tempfile` crate for isolated temp directories

### Challenge 3: Progress Bar Testing
**Problem**: Progress bar uses `\r` escape sequences, hard to capture
**Solution**: Extract progress logic to testable function, mock output

### Challenge 4: Color Code Testing
**Problem**: Terminal colors use ANSI escape codes
**Solution**: Test with `NO_COLOR=1` env var, verify plain text output

### Challenge 5: Parallel Execution Testing
**Problem**: Parallel execution has non-deterministic ordering
**Solution**: Test final results, not execution order

---

## Success Criteria (from Sprint 64 Kickoff)

- [ ] >50 unit tests for mutation handler
- [ ] >20 integration tests for workflows
- [ ] >10 property-based tests
- [ ] >85% test coverage for mutation feature
- [ ] All tests passing
- [ ] CI integration configured

---

## Next Steps

### Immediate (Next Session)
1. Create `server/tests/mutation_handler_unit_tests.rs`
2. Implement first 10 tests (Category 1: Argument Validation)
3. Run tests and verify they pass
4. Commit initial test suite

### Subsequent Sessions
1. Complete remaining unit tests (Categories 2-6)
2. Implement integration tests
3. Implement property-based tests
4. Run coverage analysis
5. Fill coverage gaps
6. Update Sprint 64 Day 1 Progress document

---

## Resources

### Code References
- **Handler**: `server/src/cli/handlers/mutate.rs` (280 lines)
- **Engine**: `server/src/services/mutation/engine.rs`
- **Types**: `server/src/services/mutation/types.rs`
- **Commands**: `server/src/cli/commands.rs` (MutateArgs struct)

### Documentation
- **Sprint 64 Kickoff**: `docs/execution/SPRINT-64-KICKOFF.md`
- **Sprint 62-64 Roadmap**: `docs/execution/SPRINT-62-64-ROADMAP.md`

---

## Notes

- Handler uses `console` crate for color output (Sprint 62 feature)
- Code snippet extraction added in Sprint 62
- Failures-only filtering added in Sprint 62
- Language detection uses centralized `Language` enum (Sprint 63)
- Handler currently only supports Rust (will expand in future)

---

**Created**: October 27, 2025
**Last Updated**: October 27, 2025
**Sprint**: Sprint 64 Day 1
**Next Update**: After first batch of tests implemented
