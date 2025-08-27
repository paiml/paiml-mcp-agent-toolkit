# Refactoring Plan: Split stubs.rs into Logical Modules

## Problem Statement
- **File**: `server/src/cli/stubs.rs`
- **Size**: 7,522 lines, 210 functions
- **Complexity**: Max cyclomatic 26, Max cognitive 53
- **Issue**: Massive single file violating single responsibility principle

## Proposed Module Structure

### 1. `handlers/tdg_handler.rs`
- `handle_analyze_tdg()`
- TDG analysis related functions
- ~200 lines

### 2. `handlers/makefile_handler.rs`
- `handle_analyze_makefile()`
- Makefile analysis and linting functions
- ~300 lines

### 3. `handlers/provability_handler.rs`
- `handle_analyze_provability()`
- `calculate_provability_score()`
- Proof annotation handlers
- ~250 lines

### 4. `handlers/defect_handler.rs`
- `handle_analyze_defect_prediction()`
- Defect prediction and formatting
- ~300 lines

### 5. `handlers/coverage_handler.rs`
- `handle_analyze_incremental_coverage()`
- Coverage analysis and reporting
- ~400 lines

### 6. `handlers/churn_handler.rs`
- `handle_analyze_churn()`
- Code churn analysis
- ~200 lines

### 7. `handlers/satd_handler.rs`
- `handle_analyze_satd()`
- `check_satd()`
- SATD detection and formatting
- ~350 lines

### 8. `handlers/dag_handler.rs`
- `handle_analyze_dag()`
- Dependency graph analysis
- ~200 lines

### 9. `handlers/comprehensive_handler.rs`
- `handle_analyze_comprehensive()`
- Comprehensive analysis orchestration
- ~400 lines

### 10. `quality_gate/checks.rs`
- `check_complexity()`
- `check_dead_code()`
- `check_entropy()`
- `check_duplicates()`
- `check_security()`
- All quality check functions
- ~800 lines

### 11. `quality_gate/formatting.rs`
- `format_quality_gate_output()`
- All QG formatting functions (JSON, Markdown, JUnit, etc.)
- ~600 lines

### 12. `quality_gate/runner.rs`
- `handle_quality_gate()`
- `run_project_checks()`
- `run_single_file_checks()`
- Quality gate orchestration
- ~500 lines

### 13. `serve_handler.rs`
- `handle_serve()`
- Server-related functions
- ~300 lines

### 14. `utils/formatting.rs`
- Common formatting utilities
- Table generation
- Output helpers
- ~400 lines

### 15. `utils/analysis.rs`
- Common analysis utilities
- File filtering
- Pattern matching
- ~300 lines

## Implementation Steps

### Phase 1: Create Module Structure
1. Create `handlers/` subdirectory
2. Create `quality_gate/` subdirectory
3. Create `utils/` subdirectory

### Phase 2: Extract and Move Functions
1. Move each handler to its respective module
2. Update imports and dependencies
3. Create proper module exports

### Phase 3: Update References
1. Update `mod.rs` to include new modules
2. Update all callers to use new module paths
3. Fix any circular dependencies

### Phase 4: Testing
1. Run all tests to ensure no breakage
2. Verify all handlers still work
3. Check that quality gates pass

### Phase 5: Cleanup
1. Delete the original `stubs.rs`
2. Update documentation
3. Commit with detailed message

## Expected Benefits

### Complexity Reduction
- **Before**: Single file with 210 functions, max complexity 26
- **After**: 15 focused modules, max complexity per module <15

### Maintainability
- **Before**: 7,522 lines in one file
- **After**: Average 400 lines per module

### Testing
- Each module can be tested independently
- Easier to identify and fix issues

### Code Organization
- Clear separation of concerns
- Logical grouping of related functionality
- Easier navigation and understanding

## Success Metrics
- [ ] All tests pass
- [ ] No function exceeds complexity 15
- [ ] No module exceeds 800 lines
- [ ] Quality gates pass
- [ ] Zero SATD introduced

## Timeline
- Estimated: 4-6 hours
- Priority: High (biggest technical debt source)

## Risk Mitigation
- Create comprehensive tests before refactoring
- Use git branches for safe experimentation
- Refactor incrementally, testing after each step