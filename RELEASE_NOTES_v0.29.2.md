# Release Notes v0.29.2

## Bug Fixes

### Critical: Fixed --max-cyclomatic Flag Not Filtering Correctly (#32)

**Issue**: The `--max-cyclomatic` and `--max-cognitive` flags were not properly filtering files. Files with all functions below the specified threshold were still being included in the results.

**Root Cause**: The filtering logic was applied after the `top_files` limit, meaning files with low complexity could still appear if they were among the "top N" files by total complexity.

**Fix**: Added proper filtering logic that:
- Filters files to only include those with at least one function exceeding the threshold
- Applies this filtering BEFORE the `top_files` limit is applied
- Works correctly with both `--max-cyclomatic` and `--max-cognitive` flags

**Example**:
```bash
# Before fix: Would show files with complexity below 50
pmat analyze complexity --max-cyclomatic 50

# After fix: Only shows files with at least one function > 50 complexity
pmat analyze complexity --max-cyclomatic 50
```

### Fixed QualityGateResults Default Implementation

Fixed test failure where `QualityGateResults::default()` was incorrectly setting `passed` to `false`. Now correctly defaults to `true` when there are no violations.

## Testing

- Added comprehensive property-based tests for complexity threshold filtering
- Added integration tests to verify the filtering behavior
- Added doctests with examples demonstrating correct filtering behavior

## Developer Notes

The fix is implemented in `server/src/cli/handlers/complexity_handlers.rs` lines 251-265. The filtering logic ensures that only files containing functions that exceed the specified thresholds are included in the analysis results.