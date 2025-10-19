# Sprint 43 Phase 2: Re-enable Passing Tests - Execution Plan

**Sprint**: 43
**Phase**: 2 (Re-enable)
**Date**: 2025-10-19
**Status**: READY TO EXECUTE
**Prerequisites**: Phase 1 ✅ Complete (17+ passing tests identified)

## Objective

Re-enable 17+ passing tests discovered in Phase 1 by removing `#[ignore]` annotations.

## Discovered Passing Tests (17 total)

### Batch 1: Claude Integration Tests (3 tests)
1. `claude_integration::sandbox::sandbox_escape_tests::test_filesystem_isolation`
   - **File**: `server/src/claude_integration/sandbox.rs:142`
   - **Current reason**: "Requires operational bridge binary for sandbox testing"
   - **Actual status**: ✅ PASSING
   - **Action**: Remove `#[ignore]`, update reason to "Re-enabled Sprint 43 - verified passing"

2. `claude_integration::tests::integration_tests::test_end_to_end_message_round_trip`
   - **File**: `server/src/claude_integration/tests.rs` (find)
   - **Action**: Remove `#[ignore]`

3. `claude_integration::tests::red_phase_integration_tests::test_claude_bridge_must_initialize_within_500ms`
   - **File**: `server/src/claude_integration/tests.rs` (find)
   - **Action**: Remove `#[ignore]`

### Batch 2: CLI Property Tests (4 tests)
4. `cli::analysis_utilities_property_tests::test_dead_code_percentage_invariants`
5. `cli::analysis_utilities_property_tests::test_dead_code_threshold_property`
6. `cli::analysis_utilities_property_tests::test_entropy_monotonicity`
7. `cli::analysis_utilities_property_tests::test_provability_score_bounds`
   - **File**: `server/src/cli/analysis_utilities.rs` (property tests module)
   - **Action**: Remove `#[ignore]` from all 4 tests

### Batch 3: CLI Commands Test (1 test)
8. `cli::commands::tests::test_cli_parse_empty`
   - **File**: `server/src/cli/commands.rs`
   - **Action**: Remove `#[ignore]`

### Batch 4: Graph Tests (2 tests)
9. `graph::tests::builder_tests::tests::test_build_from_small_workspace`
10. `graph::tests::builder_tests::tests::test_incremental_graph_update`
    - **File**: `server/src/graph/builder.rs` (tests module)
    - **Action**: Remove `#[ignore]` from both

### Batch 5: Integration Tests (6 tests)
11. `maintenance::git::tests::integration_get_current_commit`
    - **File**: `server/src/maintenance/git.rs`

12. `mcp_pmcp::discovery::integration_tests::discovery_integration_tests::test_initialization_performance`
    - **File**: `server/src/mcp_pmcp/discovery.rs`

13. `mcp_pmcp::quality_proxy_handler::tests::test_quality_proxy_handle`
    - **File**: `server/src/mcp_pmcp/quality_proxy_handler.rs`

14. `quality::gates::tests::integration_execute_all_gates`
    - **File**: `server/src/quality/gates.rs`

15. `quality::gates::tests::integration_execute_clippy`
    - **File**: `server/src/quality/gates.rs`

16. `roadmap::parser::tests::test_roundtrip_parsing`
    - **File**: `server/src/roadmap/parser.rs`

17. `scaffold::ci::tests::integration_workflow_installation`
    - **File**: `server/src/scaffold/ci.rs`

## Execution Steps

### Step 1: Find all files with #[ignore] annotations
```bash
grep -r "#\[ignore" server/src --include="*.rs" -l | sort > /tmp/sprint43_ignore_files.txt
```

### Step 2: For each passing test (Batch 1-5):
```bash
# Example for test_filesystem_isolation
# Before:
#[ignore = "Requires operational bridge binary for sandbox testing"]
fn test_filesystem_isolation() {

# After:
// Re-enabled Sprint 43 Phase 1 - verified passing (previously ignored: bridge binary requirement resolved)
fn test_filesystem_isolation() {
```

**Pattern**:
1. Read the source file
2. Find the `#[ignore ...]` line(s) before the test
3. Replace with comment documenting Sprint 43 re-enable
4. Remove `#[ignore]` annotation entirely

### Step 3: Verify each batch
After each batch of 5 tests:
```bash
# Run the specific tests
cargo test <batch_tests> --lib -- --exact

# Run full test suite to check for regressions
cargo test --lib 2>&1 | grep "test result:"
```

**Expected**: No new failures, passing count increases by 5.

### Step 4: Final verification
After all 17 tests re-enabled:
```bash
cargo test --lib 2>&1 | grep "test result:"
```

**Expected**:
- Before: 4342 passed, 28 failed, 136 ignored
- After: 4359 passed, 28 failed, 119 ignored (+17 passing, -17 ignored)

## Safety Checks

### Check 1: Test in isolation first
```bash
cargo test test_filesystem_isolation --lib -- --exact
```
**Expected**: `test result: ok. 1 passed`

### Check 2: Run batch together
```bash
cargo test claude_integration:: --lib
```
**Expected**: All claude_integration tests pass (including newly re-enabled)

### Check 3: Full suite after each batch
```bash
cargo test --lib 2>&1 | grep -E "(passed|failed|ignored)"
```
**Expected**: Passing count increases, no new failures

## Risk Mitigation

### Risk 1: Test fails when run with full suite
**Mitigation**: Re-add `#[ignore]` with note about concurrency issues
**Example**: Like Sprint 42 language regression tests (flaky when concurrent)

### Risk 2: Test causes regressions in other tests
**Mitigation**: Run full suite after each batch
**Rollback**: Git revert batch if regressions detected

### Risk 3: Test reason was valid (actually requires binary)
**Mitigation**: Check test implementation before removing `#[ignore]`
**Action**: If test truly requires binary, document reason and keep ignored

## Time Estimate

- **Batch 1-5 execution**: 1-2 hours (conservative)
  - Find files: 5 min
  - Update 17 files: 45 min (5 min per file × 17 ÷ 2 for batching)
  - Verify each batch: 30 min (6 min per batch × 5)
  - Final verification: 10 min

- **Total**: 1.5 hours

## Success Criteria

| Criterion | Target | Measurement |
|-----------|--------|-------------|
| Tests re-enabled | 17 | Count of `#[ignore]` removed |
| New passing tests | +17 | 4359 - 4342 |
| Ignored tests | -17 | 119 (from 136) |
| No new failures | 0 | Failed count stays at 28 |
| No regressions | 0 | No existing passing tests fail |

## Files to Modify (17 files estimated)

```
server/src/claude_integration/sandbox.rs
server/src/claude_integration/tests.rs
server/src/cli/analysis_utilities.rs (property tests)
server/src/cli/commands.rs
server/src/graph/builder.rs (tests)
server/src/maintenance/git.rs
server/src/mcp_pmcp/discovery.rs
server/src/mcp_pmcp/quality_proxy_handler.rs
server/src/quality/gates.rs
server/src/roadmap/parser.rs
server/src/scaffold/ci.rs
```

## Next Steps After Phase 2

1. ✅ Create Sprint 43 Phase 2 completion document
2. 📋 Update CLAUDE.md with new test counts
3. 📋 Phase 3 (optional): Investigate 9 failing tests with Five Whys
4. 📋 Phase 4: Create Sprint 43 overall completion summary
5. 📋 Commit all changes with comprehensive message

## Decision Point

**Option A: Execute Phase 2 now** (1.5 hours)
- Re-enable all 17 tests
- Achieve Sprint 43 primary goal (15-20 tests)
- Update documentation

**Option B: Execute Phase 2 in next session**
- Save current progress (Phase 1 complete)
- User reviews Phase 1 discovery document
- Continue with fresh context window

**Option C: Partial execution** (30 min)
- Re-enable first batch (5 tests) only
- Verify no issues
- Defer remaining 12 to next session

## Recommendation

**Option A** if context window allows (currently at 92K/200K).
**Option B** if approaching token limits or user wants to review first.
**Option C** as conservative middle ground.

---

**Sprint**: 43
**Phase**: 2 (Plan)
**Status**: READY
**Prerequisites**: Phase 1 ✅ Complete
**Next**: Execute re-enable or await user decision
