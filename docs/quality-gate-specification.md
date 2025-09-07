# Quality Gate Specification

## Overview

The quality-gate command is the **single source of truth** for code quality checks in PMAT. It acts as a unified interface that delegates to specialized analysis tools while ensuring consistent behavior across all interfaces (CLI, MCP, and services).

## Core Principles

### 1. Single Source of Truth
- Quality-gate is the canonical way to check code quality
- All other tools (analyze complexity, analyze satd, etc.) provide raw analysis
- Quality-gate applies thresholds and determines pass/fail status
- This ensures consistency across all interfaces

### 2. Threshold-Based Violation Counting
- **CRITICAL**: Only functions/items that EXCEED thresholds are counted as violations
- A function with complexity 10 is NOT a violation if threshold is 10
- A function with complexity 11 IS a violation if threshold is 10
- This applies to all quality checks (complexity, dead code, entropy, etc.)

## Implementation Details

### Complexity Checking

The complexity check uses the following logic:

```rust
fn function_exceeds_complexity_threshold(
    func: &FunctionComplexity,
    max_complexity: u32,
) -> bool {
    func.metrics.cyclomatic > max_complexity as u16
}
```

Key points:
- Uses `>` not `>=` for threshold comparison
- Only functions exceeding the threshold are counted
- Provides clear violation messages with actual vs. allowed values

### Output Format

When running quality checks, the output includes:
1. Initial status message
2. List of checks to run
3. Progress indicators for each check
4. Violation count for each check
5. Final pass/fail status

Example output:
```
🔍 Running quality gate checks on file: src/main.rs...

📋 Checks to run:
  ✓ Complexity analysis
  ✓ Self-admitted technical debt (SATD)

📄 Analyzing single file: src/main.rs
  🔍 Checking complexity... 1 violations found
  🔍 Checking SATD... 0 violations found

✅ Quality gate PASSED
```

## Testing

### Unit Tests
- `quality_gate_complexity_test.rs`: Verifies correct violation counting
  - Tests that only functions exceeding threshold are counted
  - Tests zero violations when all functions are below threshold

### Integration Tests
- `quality_gate_integration_test.rs`: End-to-end testing
  - Verifies quality-gate matches expected behavior
  - Tests threshold boundaries (equal to vs. exceeding)
  - Tests multiple check types

## Command Usage

### Basic Usage
```bash
# Check single file with default thresholds
pmat quality-gate --file src/main.rs

# Check with specific complexity threshold
pmat quality-gate --file src/main.rs --checks complexity --max-complexity-p99 10

# Check entire project
pmat quality-gate --checks all
```

### Available Checks
- `complexity`: Cyclomatic complexity analysis
- `satd`: Self-admitted technical debt detection
- `dead-code`: Unused code detection
- `security`: Security vulnerability scanning
- `entropy`: Code entropy analysis
- `duplicates`: Duplicate code detection
- `all`: Run all available checks

### Thresholds
- `--max-complexity-p99`: Maximum allowed cyclomatic complexity (default: 20)
- `--max-dead-code`: Maximum allowed dead code percentage (default: 15.0)
- `--min-entropy`: Minimum required code entropy (default: 0.5)

## Sprint 78 Fix

As part of Sprint 78 quality restoration, a critical bug was fixed where quality-gate was counting ALL analyzed functions as violations instead of only those exceeding thresholds. This was causing incorrect violation counts (e.g., reporting 531 violations when 0 functions actually exceeded the threshold).

The fix ensures:
1. Only threshold violations are counted
2. Clear output showing violation counts
3. Consistent behavior across all check types
4. Proper delegation to underlying analysis tools

## Future Improvements

While the current implementation is correct, future enhancements could include:
- Support for multiple checks in a single command
- Configurable output formats (JSON, SARIF, etc.)
- Baseline support for tracking quality trends
- Integration with CI/CD pipelines
- Custom quality profiles