# Sprint 39 Remaining Work - Backlog Tickets

**Date Created**: October 23, 2025
**Sprint**: Sprint 39 (Quality & Coverage Enhancement)
**Status**: DEFERRED TO BACKLOG
**Total Estimated Time**: 18-25 hours

## Context

Sprint 39 was declared SUBSTANTIALLY COMPLETE (75-85%) on October 23, 2025. The following tickets represent remaining work deferred to the backlog for future sprints.

**Completed in Sprint 39**:
- ✅ Priority 1: Language regression tests (6 tests fixed)
- ✅ Priority 2: Known failing tests (11/14 tests fixed, 79%)
- 🟡 Priority 4: Mutation testing setup and blocker documentation

**Deferred to Backlog** (5 tickets, 18-25 hours):
- Priorities 3, 4 (completion), 5, 6, 7

---

## Ticket 1: Re-enable Ignored Tests (Priority 3)

**ID**: PMAT-BACKLOG-001
**Title**: Re-enable 117 Ignored Tests
**Priority**: MEDIUM
**Estimated Time**: 10-15 hours
**Complexity**: HIGH

### Description

Currently 117 tests are marked with `#[ignore]` attribute across multiple categories. These tests need to be reviewed, fixed, and re-enabled to improve test coverage.

### Background

- Down from 137 ignored tests (Sprint 44 re-enabled 20 tests)
- Tests span multiple categories: language-specific, infrastructure, E2E, TDD, etc.
- Full list documented in `CLAUDE.md` (see "Test Coverage" section)

### Scope

**Phase 1: High Priority** (20 tests, 4-5 hours)
- 4 language-specific tests (Kotlin, WASM)
- 7 infrastructure tests (memory manager, TDG profiler, web dashboard)
- 7 annotation TDD tests (require pmat binary)
- 2 CLI and quality tests

**Phase 2: Medium Priority** (25 tests, 3-5 hours)
- 14 unified quality framework tests
- 4 end-to-end tests
- 5 language detection tests
- 1 binary integration test
- 1 integration test

**Phase 3: Lower Priority** (24 tests, 3-5 hours)
- 6 enhanced naming tests
- 4 unified context tests
- 3 TypeScript/JavaScript tests
- 5 real-world/performance tests
- 3 timeout integration tests
- 3 Ruchy parser tests (implement feature first)

**Not Re-enabling** (48 tests)
- 7 Ruchy parser RED tests (unimplemented feature)
- 3 E2E binary tests (correctly ignored - require binary)
- Others may remain ignored after investigation

### Acceptance Criteria

- [ ] All Phase 1 tests (20) reviewed and fixed/re-enabled
- [ ] All Phase 2 tests (25) reviewed and fixed/re-enabled
- [ ] All Phase 3 tests (24) reviewed and fixed/re-enabled
- [ ] Ignored test count reduced from 117 to <50
- [ ] All re-enabled tests passing in CI/CD
- [ ] Documentation updated in CLAUDE.md

### Dependencies

- None (can start immediately)

### References

- `CLAUDE.md` - Full list of ignored tests
- Sprint 44 precedent: Re-enabled 20 tests (100% passing)

---

## Ticket 2: Complete Mutation Testing (Priority 4)

**ID**: PMAT-BACKLOG-002
**Title**: Complete Mutation Testing for Hallucination Detector
**Priority**: HIGH
**Estimated Time**: 7-10 hours (4-6 hours refactoring + 3-4 hours testing)
**Complexity**: MEDIUM

### Description

Complete mutation testing implementation blocked by path-dependent test infrastructure. Requires refactoring 12 remaining tests to use TempDir pattern, then running mutation testing baseline.

### Background

**Already Completed**:
- ✅ Installed cargo-mutants v25.3.1
- ✅ Identified 98 mutants in hallucination_detector.rs (719 lines)
- ✅ Fixed 4 path-dependent tests in path_validator.rs
- ✅ Documented blocker in `docs/execution/SPRINT-39-PRIORITY-4-MUTATION-TESTING.md`

**Blocker**: 12 remaining tests fail when run from `/tmp/` (cargo-mutants working directory)

### Scope

**Phase 1: Refactor Path-Dependent Tests** (4-6 hours)

Fix 12 tests using TempDir pattern (same approach as 4 path_validator tests):

**Service Layer Tests** (6 tests):
1. `services::configuration_service::tests::test_service_lifecycle`
2. `services::deep_wasm::service::tests::test_analyze_minimal_request`
3. `services::deep_wasm::service::tests::test_analyze_ruchy_file`
4. `services::deep_wasm::tests::integration_tests::test_end_to_end_minimal_analysis`
5. `services::mutation::rust_adapter::tests::test_find_cargo_root`
6. `tests::cli_integration_full::tests::test_cli_context_generation`

**Defect Report Service Tests** (5 tests):
7. `services::defect_report_service::integration_tests::tests::test_csv_formatting`
8. `services::defect_report_service::integration_tests::tests::test_defect_report_generation`
9. `services::defect_report_service::integration_tests::tests::test_json_formatting`
10. `services::defect_report_service::integration_tests::tests::test_markdown_formatting`
11. `services::defect_report_service::integration_tests::tests::test_text_formatting`

**File Discovery Test** (1 test):
12. `tests::cli_integration_full::tests::test_custom_ignore_patterns`

**Phase 2: Run Mutation Testing** (3-4 hours)

```bash
# Run baseline tests (should pass after Phase 1)
cargo mutants --file server/src/services/hallucination_detector.rs --output mutants-out

# Analyze mutation score
# Target: >70% mutation score
```

**Phase 3: Add Tests for Surviving Mutants** (included in Phase 2 time)

- Identify mutants that survive (not caught by tests)
- Add test cases for uncaught edge cases
- Re-run mutation testing to verify improvement

### Acceptance Criteria

- [ ] All 12 path-dependent tests refactored with TempDir pattern
- [ ] Baseline mutation tests pass (0 failures)
- [ ] Mutation testing run completes successfully
- [ ] Mutation score >70% for hallucination_detector.rs
- [ ] Test cases added for surviving mutants
- [ ] Final mutation report documented

### Dependencies

- None (can start immediately)

### References

- `docs/execution/SPRINT-39-PRIORITY-4-MUTATION-TESTING.md` - Comprehensive blocker analysis
- `server/src/utils/path_validator.rs` (lines 178-257) - TempDir pattern examples

---

## Ticket 3: Property-Based Testing (Priority 5)

**ID**: PMAT-BACKLOG-003
**Title**: Add Property-Based Tests for Critical Paths
**Priority**: LOW
**Estimated Time**: 2-3 hours
**Complexity**: MEDIUM

### Description

Add property-based tests using `proptest` crate (already in dependencies) to validate invariants hold for all inputs across critical code paths.

### Scope

**Target Areas**:

1. **Language Detection** (`cli::language_detection`):
   - Property: Same file extension always maps to same language
   - Property: JavaScript detection consistent across naming conventions
   - Property: TypeScript detection consistent across naming conventions

2. **Complexity Analysis** (`services::complexity_analyzer`):
   - Property: Complexity score always non-negative
   - Property: More control flow = higher complexity
   - Property: Empty file = zero complexity

3. **File Classification** (`cli::handlers::context_handlers`):
   - Property: All files get classified
   - Property: Test files detected correctly
   - Property: No file classified as multiple primary types

### Implementation Example

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn complexity_never_negative(code in ".*") {
        let result = analyze_complexity(&code);
        prop_assert!(result.complexity >= 0.0);
    }

    #[test]
    fn same_extension_same_language(ext in "[a-z]{2,4}") {
        let lang1 = detect_language(&format!("file.{}", ext));
        let lang2 = detect_language(&format!("other.{}", ext));
        prop_assert_eq!(lang1, lang2);
    }
}
```

### Acceptance Criteria

- [ ] Property tests added for language detection (3 properties)
- [ ] Property tests added for complexity analysis (3 properties)
- [ ] Property tests added for file classification (3 properties)
- [ ] All property tests passing
- [ ] Property tests run in CI/CD
- [ ] Documentation updated with property testing approach

### Dependencies

- None (proptest already in dependencies)

### References

- `proptest` crate documentation
- Existing property tests in `server/src/unified_quality/enhanced_parser.rs`

---

## Ticket 4: Fuzz Testing (Priority 6)

**ID**: PMAT-BACKLOG-004
**Title**: Set Up Fuzz Testing for Parsers
**Priority**: LOW
**Estimated Time**: 2-3 hours (setup) + 24 hours (run time)
**Complexity**: MEDIUM

### Description

Set up fuzz testing using `cargo-fuzz` to find parser crashes and edge cases through automated input generation.

### Scope

**Target Parsers**:
1. JavaScript/TypeScript parser
2. Rust parser (tree-sitter)
3. Python parser
4. WASM parser

**Implementation**:

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Create fuzz targets
cargo fuzz init
cargo fuzz add javascript_parser
cargo fuzz add rust_parser
cargo fuzz add python_parser
cargo fuzz add wasm_parser

# Run fuzzing (24-hour corpus generation)
cargo fuzz run javascript_parser -- -max_total_time=86400
cargo fuzz run rust_parser -- -max_total_time=86400
cargo fuzz run python_parser -- -max_total_time=86400
cargo fuzz run wasm_parser -- -max_total_time=86400
```

### Success Criteria

- [ ] Zero crashes after 24 hours of fuzzing per parser
- [ ] Corpus of 1000+ valid inputs generated per parser
- [ ] Edge cases discovered and added as regression tests
- [ ] Fuzz tests integrated into CI/CD (shorter runs)

### Acceptance Criteria

- [ ] cargo-fuzz installed and configured
- [ ] Fuzz targets created for 4 parsers
- [ ] 24-hour fuzz runs completed for all parsers
- [ ] Crash reports analyzed and fixed
- [ ] Regression tests added for discovered edge cases
- [ ] Corpus committed to repo
- [ ] Documentation updated with fuzz testing approach

### Dependencies

- None (can start immediately)

### References

- `cargo-fuzz` documentation
- Rust Fuzz Book: https://rust-fuzz.github.io/book/

---

## Ticket 5: pmat Self-Validation (Priority 7)

**ID**: PMAT-BACKLOG-005
**Title**: Run pmat Quality Gates on pmat Codebase (Dogfooding)
**Priority**: LOW
**Estimated Time**: 1-2 hours
**Complexity**: LOW

### Description

Validate that pmat's codebase meets its own quality standards by running pmat quality gates on itself (dogfooding).

### Rationale

- Validates pmat works correctly on real-world codebases
- Identifies quality improvements in pmat's own code
- Demonstrates pmat's capabilities
- Builds confidence in quality gate accuracy

### Scope

**Commands to Run**:

```bash
# 1. Generate deep context for pmat
cd /home/noah/src/paiml-mcp-agent-toolkit
pmat context --output pmat_deep_context.md --format llm-optimized

# 2. Validate README for hallucinations
pmat validate-readme \
    --targets README.md CLAUDE.md GEMINI.md AGENT.md \
    --deep-context pmat_deep_context.md \
    --fail-on-contradiction \
    --verbose

# 3. Analyze pmat's own complexity
pmat analyze complexity --path server/src \
    --output pmat_complexity_report.json

# 4. Check for SATD annotations
pmat analyze satd --path server/src

# 5. Run quality gate
pmat quality-gate --path server/src \
    --checks docs-accuracy,complexity,satd
```

### Expected Outcomes

- Zero hallucinations in documentation (README, CLAUDE, GEMINI, AGENT)
- Complexity violations identified and documented
- SATD annotations tracked
- Quality improvements identified
- Self-validation report generated

### Acceptance Criteria

- [ ] Deep context generated successfully
- [ ] README validation passes (zero contradictions)
- [ ] Complexity analysis completed
- [ ] SATD analysis completed
- [ ] Quality gate report generated
- [ ] Violations documented in issues
- [ ] Self-validation report created in `docs/quality/`

### Dependencies

- pmat binary built (use `make build` or `cargo build --release`)

### References

- Sprint 38: `validate-readme` command implementation
- Sprint 37: Hallucination detection system
- CLAUDE.md: Documentation accuracy enforcement section

---

## Summary

**Total Backlog**: 5 tickets, 18-25 hours estimated

| Ticket | Priority | Estimated Time | Complexity |
|--------|----------|----------------|------------|
| PMAT-BACKLOG-001 | MEDIUM | 10-15 hours | HIGH |
| PMAT-BACKLOG-002 | HIGH | 7-10 hours | MEDIUM |
| PMAT-BACKLOG-003 | LOW | 2-3 hours | MEDIUM |
| PMAT-BACKLOG-004 | LOW | 2-3 hours | MEDIUM |
| PMAT-BACKLOG-005 | LOW | 1-2 hours | LOW |

**Recommended Prioritization** (if continuing Sprint 39 work):

1. **PMAT-BACKLOG-002** (HIGH): Complete mutation testing (unblocks quality validation)
2. **PMAT-BACKLOG-001** (MEDIUM): Re-enable ignored tests (improves coverage)
3. **PMAT-BACKLOG-005** (LOW): Self-validation (quick win, demonstrates capability)
4. **PMAT-BACKLOG-003** (LOW): Property-based testing (improves test quality)
5. **PMAT-BACKLOG-004** (LOW): Fuzz testing (discovers edge cases)

**Alternative**: Move to Sprint 48 (new feature work) and address these tickets opportunistically.

---

**Document Status**: Ready for stakeholder review
**Created By**: Sprint 39 completion process
**Last Updated**: October 23, 2025
