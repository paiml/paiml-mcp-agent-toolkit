# PMAT Ignored Test Triage Report

Generated: 2025-11-24

## Executive Summary

**Total Ignored Tests: 721** (across 593 test functions)

This is a **SYSTEMATIC ISSUE** requiring prioritized, phased approach.

## Statistics by Category

### Category A: DELETE (DEPRECATED) - 22 tests (3%)
Tests for deprecated functionality replaced by external tools.

**Files**:
- `unit_openai_embeddings.rs` (16 tests) - Use assetsearch instead
- `trueno_db_integration_tests.rs` (4 tests) - Deprecated
- `route_analyze_command_refactor_test.rs` (1 test) - Refactored
- `cargo_mutants_integration_test.rs` (1 test) - Replaced

**Action**: DELETE immediately (low risk)
**Effort**: 1 hour

### Category C: DEFER (Pending Migration/Features) - 169+ tests (23%)
Tests blocked by external dependencies or feature migrations.

**Subcategories**:
1. **Semantic Search Migration** (~82 tests)
   - Pending migration to `assetsearch` (external MCP tool)
   - Files: unit_hybrid_search.rs (27), unit_semantic_search_engine.rs (18),
     cli_semantic_integration.rs (18), unit_mcp_semantic_tools.rs (21)

2. **Mutation Testing** (~38 tests)
   - Feature-gated or pending cargo-mutants integration
   - Files: mutation_integration_tests.rs (19), mutation_handler_unit_tests.rs (9),
     mutate_command_tests.rs (9)

3. **CI Integration** (~20 tests)
   - TDG CI integration pending
   - Files: tdg_ci_integration_tests.rs (20)

4. **Other Features** (~29 tests)
   - Various feature flags and external dependencies

**Action**: Document blockers, track in roadmap
**Effort**: 2 hours documentation

### Category B: FIX (No Obvious Blocker) - ~530 tests (74%)
Tests that appear fixable but need investigation.

**Top Files** (investigation priority):
1. `repo_score/models_tests.rs` (24 tests) - Unit tests
2. `cli_acceptance/test_analyze_commands.rs` (22 tests) - CLI tests
3. `cli_acceptance/test_additional_commands.rs` (21 tests) - CLI tests
4. `cli_integration_tests.rs` (21 tests) - Integration tests
5. `cli_comprehensive_tests.rs` (19 tests) - Comprehensive tests
6. `prompt_integration_tests.rs` (19 tests) - Integration tests
7. `progress_reporting_tests.rs` (15 tests) - Unit tests
8. `tdg/baseline.rs` (15 tests) - TDG unit tests
9. `mcp_docs_enforcement.rs` (14 tests) - MCP tests
10. `scaffold/tests.rs` (14 tests) - Scaffold tests

**Action**: Apply EXTREME TDD selectively
**Effort**: 5-10 days (phased approach)

## Recommendations

### Phase 1: Quick Wins (1-2 hours)
1. **Delete Category A** (22 tests) - 1 hour
2. **Measure baseline coverage** - 30 min

**Impact**: Reduce ignored count by 3%, establish baseline

### Phase 2: High-Value Fixes (3-5 days)
Fix unit tests in core modules (prioritize by module importance):
1. `repo_score/models_tests.rs` (24 tests) - Core scoring
2. `tdg/baseline.rs` (15 tests) - TDG foundation
3. `scaffold/tests.rs` (14 tests) - Scaffolding
4. `progress_reporting_tests.rs` (15 tests) - UX

**Target**: +50 tests fixed, +5-8% coverage

### Phase 3: Integration Tests (5-10 days)
Fix CLI and integration tests systematically:
1. CLI acceptance tests (~43 tests)
2. CLI integration tests (~40 tests)
3. Prompt integration tests (19 tests)

**Target**: +100 tests fixed, +10-15% coverage

### Phase 4: Long Tail (ongoing)
Remaining ~400 tests - triage individually, apply EXTREME TDD selectively.

### Phase 5: Document Category C (2 hours)
Update CLAUDE.md with migration tracking for deferred tests.

## Critical Insight

**721 ignored tests suggests**:
- Historical technical debt accumulation
- Rapid feature development without test maintenance
- Possible CI/CD gaps (tests ignored instead of fixed)
- Opportunity for systematic improvement

**Don't try to fix all at once** - this will take weeks.
**DO apply EXTREME TDD to high-value subsets** - immediate impact.

## Coverage Impact Estimate

| Phase | Tests Fixed | Estimated Coverage Gain | Effort |
|-------|-------------|-------------------------|--------|
| Baseline | 0 | 0% (measure current) | 30 min |
| Phase 1 (Delete) | -22 (delete) | +0-1% | 1 hour |
| Phase 2 (High-Value) | +50 | +5-8% | 3-5 days |
| Phase 3 (Integration) | +100 | +10-15% | 5-10 days |
| Phase 4 (Long Tail) | +400 | +15-25% | 20-30 days |
| **Total** | **+528** | **+30-49%** | **25-45 days** |

**Realistic Target**: Phase 1 (delete + baseline) achievable this session (1.5 hours).
**Ambitious Target**: Phase 1-2 (8% gain) achievable in 3-5 days.
**Full 95% Coverage**: Requires Phase 1-4 + new tests for uncovered code (1-2 months).

## Session Plan (This Session)

**Start with Phase 1** - immediate impact, low risk:

1. ✅ Triage report created
2. Delete deprecated tests (4 files, 22 tests)
3. Measure baseline coverage with `cargo llvm-cov`
4. Commit results
5. Update CLAUDE.md with findings

**Time**: 1.5 hours total
**Impact**: Cleaner test suite, baseline established
