# Sprint 62 Day 1 Progress - Enhanced Mutant Details

**Date**: October 27, 2025 (continuation)
**Status**: In Progress (Type Extension Phase)
**Target**: Add code snippets to mutation testing output

---

## What Was Accomplished

### 1. Extended `Mutant` Struct with Code Snippet Fields

**File Modified**: `server/src/services/mutation/types.rs`

**Changes Made**:
```rust
pub struct Mutant {
    // ... existing fields ...

    /// Original code snippet (before mutation) - Sprint 62
    #[serde(default)]
    pub original_code_snippet: Option<String>,

    /// Mutated code snippet (after mutation) - Sprint 62
    #[serde(default)]
    pub mutated_code_snippet: Option<String>,
}
```

**Status**: ⚠️ Partially complete - causes compilation errors in existing code

---

## Issues Discovered

### Compilation Errors

Adding new fields to `Mutant` struct causes missing field errors in 7+ locations where `Mutant` is instantiated:

```
error[E0063]: missing fields `mutated_code_snippet` and `original_code_snippet`
in initializer of `services::mutation::types::Mutant`
```

**Affected Files**:
- `server/src/services/mutation/rust_mutation_generator.rs`
- `server/src/services/mutation/python_mutation_generator.rs`
- `server/src/services/mutation/typescript_mutation_generator.rs`
- `server/src/services/mutation/cpp_mutation_generator.rs`
- `server/src/services/mutation/go_mutation_generator.rs`
- `server/src/services/mutation/state.rs`
- Test files (equivalent_detector_tests.rs, ml_integration_tests.rs, etc.)

---

## Next Steps (Revised Approach)

### Option 1: Fix All Mutant Instantiations (Recommended for Sprint 62)

**Tasks**:
1. Add `..Default::default()` to all Mutant struct initializations
2. Implement `Default` trait for `Mutant` with `None` for new fields
3. Populate code snippets during mutant generation (optional)

**Pros**:
- Proper integration with mutation engine
- Code snippets available at generation time
- No duplication of file reading

**Cons**:
- Requires modifying 7+ files
- More extensive testing needed

**Estimated Time**: 2-3 hours

### Option 2: Extract Code Snippets in Output Functions (Alternative)

**Tasks**:
1. Revert `Mutant` struct changes
2. Read source files in `output_text()`, `output_json()`, `output_markdown()`
3. Extract code snippets on-demand using `SourceLocation`

**Pros**:
- No changes to mutation generation logic
- Backward compatible
- Simpler implementation

**Cons**:
- File I/O duplication (read source file multiple times)
- Code snippets only available in output, not in `MutationResult`

**Estimated Time**: 1-2 hours

---

## Recommended Implementation Plan

### Phase 1: Implement `Default` Trait for `Mutant`

```rust
impl Default for Mutant {
    fn default() -> Self {
        Self {
            id: String::new(),
            original_file: PathBuf::new(),
            mutated_source: String::new(),
            location: SourceLocation {
                line: 0,
                column: 0,
                end_line: 0,
                end_column: 0,
            },
            operator: MutationOperatorType::None,
            hash: String::new(),
            status: MutantStatus::Pending,
            original_code_snippet: None,
            mutated_code_snippet: None,
        }
    }
}
```

### Phase 2: Fix All Mutant Instantiations

For each file that creates `Mutant` structs, add:

```rust
Mutant {
    id,
    original_file,
    mutated_source,
    location,
    operator,
    hash,
    status,
    ..Default::default()  // <-- Add this line
}
```

**Files to Update** (7 files minimum):
1. `server/src/services/mutation/rust_mutation_generator.rs`
2. `server/src/services/mutation/python_mutation_generator.rs`
3. `server/src/services/mutation/typescript_mutation_generator.rs`
4. `server/src/services/mutation/cpp_mutation_generator.rs`
5. `server/src/services/mutation/go_mutation_generator.rs`
6. `server/src/services/mutation/state.rs`
7. Test files

### Phase 3: Enhance Output Functions

Modify `server/src/cli/handlers/mutate.rs` to display code snippets:

**For Survived Mutants** (test gaps):
```
Survived Mutants (needs test coverage):

1. src/utils/path_validator.rs:150:20
   Operator: ArithmeticReplacement
   Original: if count + 1 > MAX_DEPTH
   Mutated:  if count - 1 > MAX_DEPTH
   Time: 0.42s
```

---

## Decision Point

**Question**: Which approach should we take?

**Recommendation**: **Option 1** (Fix All Mutant Instantiations)
- More robust long-term solution
- Aligns with Sprint 62 goals
- Enables future features (e.g., code snippet caching)

**Alternative**: **Option 2** (Extract in Output Functions)
- Faster implementation for Day 1
- Less invasive changes
- Good enough for initial release

---

## Files Modified So Far

### Modified (1 file)
- ✅ `server/src/services/mutation/types.rs` - Added optional code snippet fields

### To Be Created/Modified
- [ ] `server/src/services/mutation/types.rs` - Add `Default` impl
- [ ] 7+ mutation generator files - Add `..Default::default()`
- [ ] `server/src/cli/handlers/mutate.rs` - Enhance output functions
- [ ] `CHANGELOG.md` - Document v2.175.0 changes

---

## Quality Gates

### Before Proceeding
- [ ] All compilation errors resolved
- [ ] `cargo check --lib` passes
- [ ] `cargo test --lib` passes (at least existing tests)

### Before Committing
- [ ] Clippy passes: `cargo clippy --lib`
- [ ] Format checked: `cargo fmt --check`
- [ ] Build succeeds: `cargo build --release --bin pmat`

---

## Context for Next Session

**Current State**:
- `Mutant` struct extended with optional code snippet fields
- Compilation fails due to missing fields in 7+ locations
- Need to decide between Option 1 (fix all) vs Option 2 (extract on-demand)

**Next Immediate Task**:
- Implement `Default` trait for `Mutant`
- Fix all Mutant instantiations with `..Default::default()`
- Verify compilation succeeds

**Sprint 62 Remaining Tasks** (after Day 1):
- Day 2: `--failures-only` flag and color coding
- Day 3: Multi-file testing and documentation

---

## References

- **Sprint 62 Kickoff**: `docs/execution/SPRINT-62-KICKOFF.md`
- **Mutation Types**: `server/src/services/mutation/types.rs`
- **Mutation Handler**: `server/src/cli/handlers/mutate.rs`
- **Sprint 61 Completion**: `docs/execution/SPRINT-61-PROGRESS.md`
