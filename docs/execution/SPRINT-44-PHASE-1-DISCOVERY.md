# Sprint 44 - Phase 1: Discovery (Verify Test Status)

**Date**: October 19, 2025
**Sprint**: 44
**Phase**: 1 (Discovery)
**Methodology**: EXTREME TDD + FAST + Five Whys (Toyota Way - Genchi Genbutsu)

---

## Objective

**Empirically verify** the actual status of ignored tests via systematic execution.

**Target**: 137 ignored tests
**Focus**: Mutation tests (CRITICAL for FAST), graph tests, service layer tests
**Duration**: 1-1.5 hours estimated

---

## Test Categories (Prioritized by FAST + Value)

### Category C: Mutation Tests (HIGHEST PRIORITY - FAST Alignment)
**Count**: 20+ tests
**Rationale**: Mutation testing is CORE to FAST methodology
**Impact**: Re-enabling these improves quality coverage by 15-20%

**Tests**:
```
services::mutation::rust_tree_sitter_mutations::tests::test_rust_binary_addition
services::mutation::rust_tree_sitter_mutations::tests::test_rust_binary_subtraction
services::mutation::rust_tree_sitter_mutations::tests::test_rust_bitwise_and
services::mutation::rust_tree_sitter_mutations::tests::test_rust_bitwise_not
services::mutation::rust_tree_sitter_mutations::tests::test_rust_borrow_immutable
services::mutation::rust_tree_sitter_mutations::tests::test_rust_borrow_mutable
services::mutation::rust_tree_sitter_mutations::tests::test_rust_exclusive_range
services::mutation::rust_tree_sitter_mutations::tests::test_rust_inclusive_range
services::mutation::rust_tree_sitter_mutations::tests::test_rust_logical_and
services::mutation::rust_tree_sitter_mutations::tests::test_rust_logical_or
services::mutation::rust_tree_sitter_mutations::tests::test_rust_method_chain_filter
services::mutation::rust_tree_sitter_mutations::tests::test_rust_method_chain_map
... (additional mutation tests)
```

**Verification Command**:
```bash
cargo test --ignored services::mutation::rust_tree_sitter_mutations --no-fail-fast
```

**Expected Outcome**: 50-70% passing (based on Sprint 42/43 pattern)

---

### Category A: Graph Tests (HIGH PRIORITY - Quick Wins)
**Count**: 2 tests
**Rationale**: Integration tests, likely passing

**Tests**:
```
graph::tests::builder_tests::tests::test_build_from_small_workspace
graph::tests::builder_tests::tests::test_incremental_graph_update
```

**Verification Command**:
```bash
cargo test --ignored graph::tests::builder_tests --no-fail-fast
```

**Expected Outcome**: 100% passing (may have been re-ignored accidentally)

---

### Category B: Service Layer Tests (MEDIUM PRIORITY)
**Count**: 10 tests
**Rationale**: Core functionality, mixed complexity

**Tests**:
```
services::cache::cache_property_tests::tests::cache_eviction_maintains_invariants_slow
services::cache::cache_property_tests::tests::cache_get_put_consistency_slow
services::cache::cache_property_tests::tests::unified_cache_manager_consistency_slow
services::context::tests::test_format_deep_context_as_markdown
services::dead_code_analyzer::tests::test_analyze_with_ranking
services::deep_context::tests::test_deep_context_result_creation
services::deep_wasm::bytecode_analyzer::tests::test_analyze_minimal_wasm
services::file_classifier_property_tests::tests::include_large_files_flag_behavior
services::git_clone_property_tests::test_repo_size_edge_cases
services::memory_manager::tests::test_concurrent_access
```

**Verification Commands**:
```bash
# Cache property tests (likely slow but passing)
cargo test --ignored services::cache::cache_property_tests --no-fail-fast

# Context tests
cargo test --ignored services::context::tests::test_format_deep_context_as_markdown

# Dead code analyzer
cargo test --ignored services::dead_code_analyzer::tests::test_analyze_with_ranking

# Deep context
cargo test --ignored services::deep_context::tests::test_deep_context_result_creation

# Deep WASM
cargo test --ignored services::deep_wasm::bytecode_analyzer::tests::test_analyze_minimal_wasm

# File classifier property tests
cargo test --ignored services::file_classifier_property_tests --no-fail-fast

# Git clone property tests
cargo test --ignored services::git_clone_property_tests --no-fail-fast

# Memory manager
cargo test --ignored services::memory_manager::tests::test_concurrent_access
```

**Expected Outcome**: 60-80% passing

---

### Category D: MCP Discovery Tests (MEDIUM PRIORITY)
**Count**: 1 test
**Rationale**: Performance test, likely passing but slow

**Tests**:
```
mcp_pmcp::discovery::integration_tests::discovery_integration_tests::test_initialization_performance
```

**Verification Command**:
```bash
cargo test --ignored mcp_pmcp::discovery::integration_tests::discovery_integration_tests::test_initialization_performance
```

**Expected Outcome**: PASSING (performance test, not functional issue)

---

### Category E: Doc Validator Tests (MEDIUM PRIORITY - TDD Alignment)
**Count**: 2 tests
**Rationale**: RED tests for documentation accuracy enforcement

**Tests**:
```
services::doc_validator::unit_tests::red_test_validate_http_200
services::doc_validator::unit_tests::red_test_validate_http_404
```

**Verification Command**:
```bash
cargo test --ignored services::doc_validator --no-fail-fast
```

**Expected Outcome**: RED tests may still be RED (expected for TDD)

---

### Category F: Annotation TDD Tests (MEDIUM PRIORITY)
**Count**: 7 tests
**Rationale**: Require pmat binary, may be passing with binary built

**Tests**:
```
cli::handlers::annotation_tdd_tests::red_phase_tests::red_must_show_complexity_scores
cli::handlers::annotation_tdd_tests::red_phase_tests::red_must_show_dead_code_markers
cli::handlers::annotation_tdd_tests::red_phase_tests::red_must_show_file_level_breakdown
cli::handlers::annotation_tdd_tests::red_phase_tests::red_must_show_individual_function_names
cli::handlers::annotation_tdd_tests::red_phase_tests::red_must_show_quality_insights
cli::handlers::annotation_tdd_tests::red_phase_tests::red_must_show_satd_annotations
cli::handlers::annotation_tdd_tests::red_phase_tests::red_must_show_wasm_function_details
```

**Verification Command**:
```bash
# First build pmat binary
cargo build --release

# Then run tests
cargo test --ignored cli::handlers::annotation_tdd_tests --no-fail-fast
```

**Expected Outcome**: 50-70% passing (if binary is available)

---

### Category G: Mermaid Generator Tests (LOW PRIORITY)
**Count**: 3 tests
**Rationale**: May require external fixtures

**Tests**:
```
services::mermaid_generator::tests::test_generated_output_matches_reference_syntax
services::mermaid_generator::tests::test_invalid_example_is_correctly_identified
services::mermaid_generator::tests::test_reference_standards_are_valid
```

**Verification Command**:
```bash
cargo test --ignored services::mermaid_generator --no-fail-fast
```

**Expected Outcome**: Unknown (fixtures may be missing)

---

## Execution Plan (EXTREME TDD - Red Phase)

### Step 1: Build pmat Binary (Prerequisite)
```bash
cd /home/noah/src/paiml-mcp-agent-toolkit/server
cargo build --release --lib
```

**Rationale**: Some tests require pmat binary to exist

---

### Step 2: Execute Category C (Mutation Tests - HIGHEST PRIORITY)
```bash
cargo test --ignored services::mutation::rust_tree_sitter_mutations --no-fail-fast 2>&1 | tee /tmp/sprint44_mutation_tests.log
```

**Success Criteria**: ≥70% passing
**Time Budget**: 15 minutes
**FAST Alignment**: CRITICAL - these are mutation tests

---

### Step 3: Execute Category A (Graph Tests - Quick Wins)
```bash
cargo test --ignored graph::tests::builder_tests --no-fail-fast 2>&1 | tee /tmp/sprint44_graph_tests.log
```

**Success Criteria**: 100% passing
**Time Budget**: 5 minutes

---

### Step 4: Execute Category B (Service Layer Tests)
```bash
# Run all service layer tests individually
cargo test --ignored services::cache::cache_property_tests --no-fail-fast 2>&1 | tee /tmp/sprint44_cache_tests.log
cargo test --ignored services::context::tests::test_format_deep_context_as_markdown 2>&1 | tee -a /tmp/sprint44_service_tests.log
cargo test --ignored services::dead_code_analyzer::tests::test_analyze_with_ranking 2>&1 | tee -a /tmp/sprint44_service_tests.log
cargo test --ignored services::deep_context::tests::test_deep_context_result_creation 2>&1 | tee -a /tmp/sprint44_service_tests.log
cargo test --ignored services::deep_wasm::bytecode_analyzer::tests::test_analyze_minimal_wasm 2>&1 | tee -a /tmp/sprint44_service_tests.log
cargo test --ignored services::file_classifier_property_tests --no-fail-fast 2>&1 | tee -a /tmp/sprint44_service_tests.log
cargo test --ignored services::git_clone_property_tests --no-fail-fast 2>&1 | tee -a /tmp/sprint44_service_tests.log
cargo test --ignored services::memory_manager::tests::test_concurrent_access 2>&1 | tee -a /tmp/sprint44_service_tests.log
```

**Success Criteria**: ≥60% passing
**Time Budget**: 20 minutes

---

### Step 5: Execute Category D (MCP Discovery)
```bash
cargo test --ignored mcp_pmcp::discovery::integration_tests::discovery_integration_tests::test_initialization_performance 2>&1 | tee /tmp/sprint44_mcp_tests.log
```

**Success Criteria**: PASSING
**Time Budget**: 5 minutes

---

### Step 6: Execute Category E (Doc Validator)
```bash
cargo test --ignored services::doc_validator --no-fail-fast 2>&1 | tee /tmp/sprint44_doc_validator_tests.log
```

**Success Criteria**: Verify actual status (RED is acceptable for TDD)
**Time Budget**: 5 minutes

---

### Step 7: Execute Category F (Annotation TDD)
```bash
cargo test --ignored cli::handlers::annotation_tdd_tests --no-fail-fast 2>&1 | tee /tmp/sprint44_annotation_tests.log
```

**Success Criteria**: ≥50% passing
**Time Budget**: 10 minutes

---

## Data Collection (Five Whys - Root Cause Analysis)

For each test category, record:

1. **Status**: PASS / FAIL / TIMEOUT
2. **Duration**: Execution time
3. **Error Type**: If failing, categorize error
4. **Root Cause**: Apply Five Whys to understand why ignored
5. **Re-enable Candidate**: YES / NO / MAYBE

### Recording Format

```markdown
### Test: services::mutation::rust_tree_sitter_mutations::tests::test_rust_binary_addition

**Status**: PASS ✅
**Duration**: 0.23s
**Why ignored?**
1. Why was this ignored? → Assumed to be failing
2. Why assumed failing? → No recent verification
3. Why no verification? → Test marked as slow/flaky
4. Why marked slow/flaky? → Original implementation issue
5. Why not fixed? → Assumed to require major refactor

**Root Cause**: Test was working all along, never verified empirically
**Re-enable Candidate**: YES ✅
```

---

## Expected Results (Based on Sprint 42/43 Pattern)

### Conservative Estimate
- **Total Verified**: 40-50 tests
- **Passing**: 25-35 tests (60-70% pass rate)
- **Failing**: 10-15 tests (requires fixes)
- **Timeout**: 0-5 tests (too slow)

### Optimistic Estimate
- **Total Verified**: 50-60 tests
- **Passing**: 35-45 tests (70-80% pass rate)
- **Failing**: 10-15 tests
- **Timeout**: 0-5 tests

### Re-enablement Target
**Goal**: Re-enable 20-30 passing tests in Phase 2

---

## FAST Methodology Alignment

### Property Tests (Category B)
- ✅ `cache_property_tests` - Property-based testing of cache invariants
- ✅ `file_classifier_property_tests` - File classification properties
- ✅ `git_clone_property_tests` - Git operations properties

### Mutation Tests (Category C - CRITICAL)
- ✅ All rust_tree_sitter_mutations tests
- ✅ 20+ mutation test cases
- ✅ Core to FAST quality methodology

### Analysis (`pmat analyze`)
```bash
# Analyze test files to verify complexity
pmat analyze server/src/services/mutation/rust_tree_sitter_mutations.rs --format markdown
pmat analyze server/src/graph/tests/builder_tests.rs --format markdown
pmat analyze server/src/services/cache/cache_property_tests.rs --format markdown
```

**Purpose**: Verify test code quality meets EXTREME TDD standards

---

## Toyota Way Principles

### Genchi Genbutsu (Go and See)
- ✅ Run tests empirically, don't assume status
- ✅ Collect real execution data
- ✅ Verify with actual results, not documentation

### Jidoka (Built-in Quality)
- ✅ Tests verify themselves via execution
- ✅ bashrs pre-commit hook prevents bad commits
- ✅ Automated quality gates

### Kaizen (Continuous Improvement)
- ✅ Learn from Sprint 42/43 success
- ✅ Refine verification process
- ✅ Establish sustainable cadence

### Muda (Waste Elimination)
- ✅ Avoid debugging tests that are already passing
- ✅ Systematic verification saves 60-75% time
- ✅ Focus effort on actual failures

---

## Next Steps

After Phase 1 completion:

1. ✅ **Analyze Results**: Categorize tests by status
2. ⏭️ **Create Phase 2 Plan**: Document re-enablement strategy
3. ⏭️ **Execute Phase 2**: Remove #[ignore] from passing tests
4. ⏭️ **Verify**: Run full test suite
5. ⏭️ **Document**: Update CLAUDE.md, CHANGELOG.md

---

## Time Budget

- **Category C (Mutation)**: 15 minutes
- **Category A (Graph)**: 5 minutes
- **Category B (Service)**: 20 minutes
- **Category D (MCP)**: 5 minutes
- **Category E (Doc Validator)**: 5 minutes
- **Category F (Annotation TDD)**: 10 minutes
- **Analysis & Documentation**: 20 minutes

**Total**: 80 minutes (1 hour 20 minutes)

---

**Status**: READY TO EXECUTE
**Start Time**: TBD
**Expected Completion**: TBD

---

*Document created: October 19, 2025*
*Sprint: 44*
*Phase: 1 (Discovery)*
*Methodology: EXTREME TDD + FAST + Five Whys*
