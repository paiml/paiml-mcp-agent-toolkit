# Release Notes - v0.28.12

## 🐛 Bug Fixes

### Quality Gate Improvements

This release fixes critical issues with the quality gate and complexity analysis commands:

#### Issue #30: quality-gate doesn't show checks
- **Fixed**: Quality gate now displays which checks are being run
- **Added**: Clear "📋 Checks to run:" section showing all active checks
- **Impact**: Users can now see exactly what quality gate is checking

#### Issue #32: --max-cyclomatic doesn't affect report
- **Fixed**: Custom complexity thresholds now properly affect violation detection
- **Added**: New `aggregate_results_with_thresholds` function that respects user-provided thresholds
- **Impact**: `pmat analyze complexity --max-cyclomatic 15` now correctly identifies violations based on the custom threshold

#### Issue #29: quality-gate doesn't find any violations
- **Fixed**: Quality gate now uses proper AST-based complexity analysis instead of heuristics
- **Removed**: Eliminated `estimate_cyclomatic_complexity` and `estimate_cognitive_complexity` heuristic functions
- **Impact**: Quality gate now accurately detects complexity violations using the same engine as `analyze complexity`

### 🏗️ Code Quality Improvements

#### Zero Tolerance for Heuristics (NEW Rule)
Added to CLAUDE.md:
- **Rule 6**: NEVER Use Simple Heuristics - Always use proper AST-based analysis
- **Rule 7**: NEVER Duplicate Core Logic - ONE implementation per feature shared across all providers

#### Technical Details
- Quality gate `check_complexity` now calls the unified `analyze_project_files` function
- All complexity analysis uses the same AST-based engine
- Removed duplicate implementations and heuristic approximations

### 📚 Documentation Updates

#### New Examples
- `examples/quality_gate_thresholds.rs` - Demonstrates configurable thresholds in action

#### Updated Functions
- `aggregate_results_with_thresholds` - Now includes comprehensive doctests
- Property tests added for threshold functionality
- Unit tests for quality gate with custom thresholds

### 🧪 Testing

Added comprehensive tests:
- Property tests for custom threshold behavior
- Unit tests for quality gate single file checks
- Integration tests for threshold overrides
- All tests pass with proper AST analysis

### 🔧 API Changes

The `aggregate_results_with_thresholds` function signature:
```rust
pub fn aggregate_results_with_thresholds(
    file_metrics: Vec<FileComplexityMetrics>,
    max_cyclomatic: Option<u16>,  // Custom threshold
    max_cognitive: Option<u16>,   // Custom threshold
) -> ComplexityReport
```

### 🚀 Usage

```bash
# Analyze with custom thresholds (now works correctly!)
pmat analyze complexity --max-cyclomatic 15 --max-cognitive 10

# Quality gate with custom complexity limits
pmat quality-gate --max-complexity 15 --fail-on-violation

# Example output shows violations based on YOUR thresholds
```

### 🙏 Acknowledgments

This release addresses critical functionality issues reported in:
- Issue #32: Custom thresholds not affecting reports
- Issue #29: Quality gate not finding violations

These fixes ensure that pmat provides accurate, configurable quality analysis for all users.

---

**Remember**: No heuristics, no shortcuts, no duplicate implementations. Only proper AST-based analysis following the Toyota Way principles.