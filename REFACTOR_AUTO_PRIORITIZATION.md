# Refactor Auto Prioritization Update

## Summary

Updated `pmat refactor auto` to implement the correct prioritization order with enhanced progress reporting.

## Prioritization Order (NEW)

1. **PRIORITY 1: Lint Violations** 
   - Fix all clippy lint violations first (highest count)
   - Uses existing lint-hotspot analysis

2. **PRIORITY 2: Build Errors**
   - If no lint violations remain, check if build passes
   - If build fails, analyze compilation errors using `cargo build`
   - Target files with most compilation errors

3. **PRIORITY 3: Coverage < 80%**
   - If build passes, find any file with coverage < 80%
   - Simple check: any file below threshold gets refactored

4. **PRIORITY 4: Enforce Extreme Quality**
   - Only when all files have ≥ 80% coverage
   - Check for complexity > 10
   - Check for SATD items
   - Other extreme quality standards

## Key Changes Made

### 1. Added BuildFixes Phase
```rust
pub enum RefactorPhase {
    Initialization,
    LintFixes,
    BuildFixes,      // NEW
    ComplexityReduction,
    SatdCleanup,
    CoverageDriven,
    QualityValidation,
    Complete,
}
```

### 2. Enhanced Progress Reporting
- Visual progress bar showing overall completion %
- Per-category completion percentages (lint, complexity, SATD, coverage)
- Current phase indication
- Files completed/remaining count
- Estimated time remaining
- Progress updates at start and end of each iteration

### 3. New Helper Functions
- `select_coverage_or_extreme_target()` - Handles priority 3 & 4
- `find_file_with_low_coverage()` - Finds any file < 80% coverage
- `select_extreme_quality_target()` - Enforces extreme standards

### 4. Improved File Selection Logic
- Clear priority order enforcement
- Build check integrated into main flow
- Coverage check simplified (any file < 80%)
- Extreme quality only after coverage met

## Progress Display Example

```
📊 Progress Update - Iteration 5/100
🎯 Overall: 45.2% [█████████░░░░░░░░░░░]
   📊 Lint: 75.0% | 🔧 Complexity: 60.0% | 🧹 SATD: 100.0% | 📈 Coverage: 15.0%
   🔄 Phase: CoverageDriven
   📁 Files: 12 completed, 38 remaining

🎯 PRIORITY 3: Checking for files with coverage < 80%...
   Current overall coverage: 65.3%
   📊 src/handlers.rs has 45.2% coverage (< 80%)
```

## Verification During Refactoring

After each file refactor, the system ALWAYS verifies:
1. ✅ 80% coverage achieved for the file
2. ✅ All `pmat enforce extreme` standards met
3. ✅ Build still passes

Files that don't meet ALL criteria are NOT marked as completed and will be retried.

## Testing Commands

```bash
# Run with detailed progress
pmat refactor auto --format detailed

# Check current progress
pmat refactor status

# Dry run to see what would be refactored
pmat refactor auto --dry-run
```

## Benefits

1. **Clear Prioritization**: Always fix compilation errors before coverage
2. **Better Progress Visibility**: Know exactly where you are in the process
3. **Efficient Coverage Improvement**: Target any file < 80% immediately
4. **Quality Enforcement**: Extreme standards only after basics are met