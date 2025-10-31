# Bug Report: Function Count Always Shows Zero Despite Functions Detected

**Date**: 2025-10-31
**Reporter**: User feedback
**Severity**: Medium → ✅ FIXED
**Component**: Context generation - function detection
**Status**: GREEN phase complete (5/5 tests passing)
**Progress**: Sprint 79 Phase 2 - FIXED

## Description

When running `pmat context`, the output always lists `File Complexity: 1 | Functions: 0` even when there are functions inside the analyzed files and when functions are correctly reported in the detailed output.

## Steps to Reproduce

```bash
pmat context
# Look at a file that has functions
```

## Example Case

File: `assetgen/src/transcriber/auto_transcriber.rs`

Expected: Should show actual function count (e.g., `Functions: 5`)
Actual: Shows `Functions: 0`

But the detailed output does list functions with their complexity metrics.

## Expected Behavior

The function count should reflect the actual number of functions detected in each file:

```
### ./assetgen/src/transcriber/auto_transcriber.rs
File Complexity: 15 | Functions: 5

- **Function**: `new` [complexity: 3] [cognitive: 2]
- **Function**: `transcribe` [complexity: 8] [cognitive: 5]
- **Function**: `process_batch` [complexity: 4] [cognitive: 3]
...
```

## Analysis

Possible causes:

1. **Display Bug**: Function count is calculated but not displayed correctly
2. **Calculation Bug**: Function counting logic always returns 0
3. **Aggregation Bug**: Functions detected but count not aggregated to file level
4. **Default Value**: Using default/placeholder value instead of actual count

## Impact

- Users can't quickly see function density per file
- Misleading information (functions exist but count shows 0)
- Reduces usefulness of summary metrics

## Files to Investigate

- `server/src/cli/handlers/context.rs` - Context generation
- `server/src/services/simple_deep_context.rs` or `server/src/services/deep_context.rs` - Function counting
- Markdown output formatting for file statistics

## Suggested Fix

Ensure function count is properly calculated and displayed:

```rust
struct FileStats {
    complexity: u32,
    function_count: usize,  // Ensure this is populated
}

// When outputting:
writeln!(output, "File Complexity: {} | Functions: {}",
    stats.complexity,
    stats.function_count)?;
```

## Test Case

```rust
#[test]
fn test_function_count_reflects_actual_functions() {
    let result = generate_context("./fixtures/rust-project");
    assert!(result.contains("Functions: 5"));  // Not "Functions: 0"
}
```

## Fix Applied

**Root Cause**: Path matching failure in `utility_handlers.rs:350`. The code used `file.path.ends_with(&f.path)` which failed when file paths were in different formats (relative vs absolute).

**Solution**: Implemented comprehensive path matching with 4 strategies + fallback:

1. **Strategy 1**: Exact string match
2. **Strategy 2**: `ends_with` for relative paths
3. **Strategy 3**: File name comparison
4. **Strategy 4**: Canonical path comparison
5. **Fallback**: Count functions from `file.items` if complexity report missing

**Files Modified**:
- `server/src/cli/handlers/utility_handlers.rs` - Added improved path matching logic
- `server/tests/bug_007_function_count_tests.rs` - Added 5 comprehensive tests (100% passing)

**Test Coverage**: 5/5 tests passing
- ✅ `test_function_count_reflects_actual_functions`
- ✅ `test_function_count_zero_when_no_functions`
- ✅ `test_function_count_per_file`
- ✅ `test_function_count_includes_all_types`
- ✅ `test_function_count_in_summary`
