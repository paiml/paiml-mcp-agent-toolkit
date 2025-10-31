# Bug Report: Incorrect Line Numbers After File Extraction (Issue #67)

**Date**: 2025-10-31
**Reporter**: GitHub Issue #67
**Severity**: Critical → ✅ FIXED (Already implemented)
**Component**: `pmat analyze complexity` - Line number reporting
**Status**: GREEN phase complete - Tests passing

## Description

When analyzing functions extracted from one file to another, `pmat analyze complexity` reports line numbers from the OLD file location instead of the ACTUAL file being analyzed.

## Example

**File extraction**:
- Original: `utils.rs:500` (function `parse_rust_attribute_arguments`)
- Extracted to: `attributes.rs:148` (same function, new location)

**Command**:
```bash
pmat analyze complexity --file src/frontend/parser/utils_helpers/attributes.rs
```

**Expected Output**:
```
parse_rust_attribute_arguments (line 148-198) - Cyclomatic: 6, Cognitive: 13
```

**Actual Output**:
```
parse_rust_attribute_arguments (line 500-550) - Cyclomatic: 6, Cognitive: 13
```

**Problem**: File is only 214 lines, but pmat reports line 500-550 (impossible!)

## Impact

- **CRITICAL**: Blocks commits with false complexity violations
- Pre-commit hooks fail on working code due to stale line numbers
- Cannot proceed with refactoring work
- Trust issue: Users lose confidence in pmat accuracy

## Root Cause Analysis

**Hypothesis 1**: Cache not being invalidated on file move/extraction
- Clearing `~/.pmat/tdg-warm/*` did NOT fix issue
- Suggests problem is not simple cache staleness

**Hypothesis 2**: Analysis uses git history or function name lookup
- pmat may be matching function by NAME across files
- Then reporting OLD line numbers from git history

**Hypothesis 3**: AST analysis re-uses metadata from wrong file
- Complexity analysis correct (CC=6, Cognitive=13)
- But line number metadata comes from wrong source

## Investigation Needed

1. How does `--file` parameter work in complexity analysis?
2. Does it use git to find function history?
3. Does it cache function metadata by name?
4. Is there a function name → line number mapping that's stale?

## Test Requirements (RED Phase)

1. Test that analyzing extracted file reports correct line numbers
2. Test that function name doesn't cause cross-file pollution
3. Test that cache doesn't contain stale line numbers
4. Test `--force-refresh` flag (needs to be added)
5. Test file-scoped analysis (no cross-file lookups)

## Suggested Fix

**Option 1: File-scoped analysis (RECOMMENDED)**
```rust
// When --file is specified, analyze ONLY that file
// Never look at git history or other files
pub fn analyze_complexity_for_file(file_path: &Path) -> Result<Vec<FunctionComplexity>> {
    // 1. Parse the ACTUAL file at file_path
    let source = fs::read_to_string(file_path)?;

    // 2. Extract functions with line numbers from THIS source
    let functions = extract_functions_with_positions(&source)?;

    // 3. Return complexities with line numbers from THIS file
    // NO cross-file lookups, NO git history, NO caching by function name
    Ok(functions)
}
```

**Option 2: Add --force-refresh flag**
```rust
#[arg(long)]
force_refresh: bool,

if force_refresh {
    // Bypass ALL caches
    // Analyze from scratch
}
```

**Option 3: Invalidate cache on file modification**
```rust
// Cache key should include file modification time
let cache_key = format!("{file_path}:{mtime}");
```

## Files to Investigate

- `server/src/cli/handlers/complexity.rs` - Complexity CLI handler
- `server/src/services/complexity_analyzer.rs` - Complexity analysis
- `server/src/services/ast/` - AST parsing (line number source)
- Caching logic for complexity results

## Workaround

None currently available. Users must:
1. Accept false violations
2. Skip pre-commit hooks (`--no-verify`)
3. Wait for fix

## TDD Approach

**Sprint**: Bug Fix Sprint (Critical Line Numbers)
**Version**: v2.190.0
**Methodology**: Extreme TDD (RED → GREEN → REFACTOR → COMMIT)

---

## Fix Applied

**Root Cause**: TDG cache keyed by content hash - unchanged when functions moved between files

**Solution**: Implemented `analyze_file_complexity_uncached()` function to bypass cache

**Commit**: 9cbdd3c5 - "fix: Issue #67 - Accurate line numbers for extracted functions (EXTREME TDD)"

**Files Modified**:
- `server/src/services/complexity.rs` - Added uncached analysis function
- `server/src/services/complexity_file_extraction_tests.rs` - Comprehensive tests (377 lines)

**Test Results**: ✅ ALL TESTS PASSING
```bash
test services::complexity_file_extraction_tests::red_phase_tests::test_file_extraction_line_numbers_accurate ... ok
```

**Implementation**:
```rust
/// Analyze file complexity without using cache
/// Ensures fresh line numbers for extracted/moved functions
pub async fn analyze_file_complexity_uncached(
    file_path: &Path,
    content: Option<&str>,
) -> Result<FileComplexityMetrics> {
    // Read file or use provided content
    let source = match content {
        Some(c) => c.to_string(),
        None => fs::read_to_string(file_path).await?,
    };

    // Parse AST and extract functions with CURRENT line numbers
    let functions = extract_functions_with_positions(&source)?;

    // Return fresh complexity metrics from THIS file
    Ok(FileComplexityMetrics {
        file_path: file_path.to_path_buf(),
        functions,
        total_complexity: calculate_total_complexity(&functions),
    })
}
```

**Impact**:
- ✅ Accurate line numbers for extracted functions
- ✅ No false pre-commit violations
- ✅ Refactoring work unblocked
- ✅ Cache doesn't pollute fresh analysis

**Status Updates**:
- 2025-10-31: Bug report created
- 2025-10-31: DISCOVERED - Already fixed in commit 9cbdd3c5
- 2025-10-31: Tests verified passing, issue ready to close
