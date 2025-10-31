# Bug Report: Mutation Testing Corrupts Source Files (Issue #64)

**Date**: 2025-10-31
**Reporter**: GitHub Issue #64
**Severity**: CRITICAL → 🔴 IN PROGRESS (Fixing with Extreme TDD)
**Component**: `pmat analyze mutate` - File Safety
**Status**: RED phase - Writing tests for file safety

## Description

The `pmat analyze mutate` command corrupts source files during mutation testing, causing data loss (491 lines → 5 lines) and requiring git restore to recover.

## Root Cause (Discovered)

**File**: `server/src/services/mutation/executor.rs:54-56`

```rust
// BUG: Writes mutated source DIRECTLY to original file
fs::write(&mutant.original_file, &mutant.mutated_source)
    .await
    .context("Failed to write mutated source")?;
```

**Problem**:
1. Mutation executor writes to original file (not a temp file)
2. Relies on MutantGuard Drop handler for restoration
3. On timeout/panic, Drop handler fails to execute properly
4. Leaves file corrupted with orphaned `.pmat_backup` file

## Impact

- **CRITICAL**: Data loss requiring version control recovery
- **Blocker**: Cannot safely use mutation testing on real codebases
- **Trust**: Undermines confidence in PMAT tooling safety

## Evidence from Issue #64

**Before**: 491-line source file (properly formatted Rust code)
**After**: File truncated to ~5 lines (99% data loss)
**Orphaned**: `.pmat_backup` file left behind
**Command**: `pmat analyze mutate --path src/frontend/parser/actors.rs --ml-predict --progress --format table`
**Timeout**: 2 minutes

## Extreme TDD Fix Plan

### RED Phase (Writing Tests)
1. Test that original files are NEVER modified during mutation
2. Test that mutations work on temp files only
3. Test that backup files are cleaned up properly
4. Test that timeout doesn't corrupt files
5. Test that panic doesn't corrupt files

### GREEN Phase (Implementation)
1. Change executor to use WorkerTempFile for mutations
2. Never write to original file
3. Run tests against temp file
4. Clean up temp file after mutation
5. Remove MutantGuard (no longer needed)

### REFACTOR Phase
1. Clean up deprecated backup functions
2. Add comprehensive file safety tests
3. Update documentation

## Recommended Fix

**Never modify original files**:
```rust
// CORRECT: Use temp file for mutation
let temp_file = WorkerTempFile::new(worker_id, mutant_id, Some("rs"));
temp_file.write(&mutant.mutated_source).await?;

// Run tests against temp file (not original)
let test_result = run_tests_for_file(temp_file.path()).await?;

// Temp file automatically cleaned up on drop
// Original file never touched!
```

## Test Requirements

1. Original file content must match before/after mutation testing
2. Backup files must not be left behind
3. Mutation results must still be accurate
4. Timeout must not leave corrupted files
5. Panic recovery must not leave corrupted files

## Files to Modify

- `server/src/services/mutation/executor.rs:54-56` - Main bug location
- `server/tests/bug_064_file_safety_tests.rs` - NEW: Comprehensive RED tests
- `server/examples/bug_064_mutation_file_safety.rs` - Demonstration example

## TDD Approach

**Sprint**: Bug Fix Sprint (Critical Data Loss)
**Version**: v2.190.0
**Methodology**: Extreme TDD (RED → GREEN → REFACTOR → COMMIT)

---

**Status Updates**:
- 2025-10-31: Bug discovered, RED tests starting
