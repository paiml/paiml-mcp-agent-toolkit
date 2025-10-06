# TICKET-PMAT-6020: Connect health_check MCP Tool

**Sprint:** Sprint 22 - MCP Phase 2
**Priority:** P1 - High
**Estimated Effort:** 2-3 hours
**Actual Effort:** 2 hours
**Status**: GREEN ✅
**Created:** 2025-10-06
**Completed:** 2025-10-06

## Problem Statement

The `health_check` MCP tool returns mock health data. Agents cannot run real project health checks via MCP.

## Solution

Refactor `handle_maintain_health()` to extract `run_health_checks_internal()` that returns `HealthReport`. MCP handler calls this internal function, benefiting from parallel execution added in PMAT-6010!

## Implementation

### Refactored Health Handler

```rust
/// Run health checks and return report (internal, reusable)
/// (TICKET-PMAT-6020)
pub async fn run_health_checks_internal(
    project_dir: &PathBuf,
    quick: bool,
    all: bool,
    check_build: bool,
    check_tests: bool,
    check_coverage: bool,
    check_complexity: bool,
    check_satd: bool,
) -> Result<HealthReport> {
    let checks_to_run = determine_checks_to_run(...);

    let mut check_types = Vec::new();
    // Build check list based on flags

    // Run checks in parallel (PMAT-6010)
    let checks = run_checks_parallel(project_dir, check_types).await?;

    let summary = calculate_summary(&checks);
    Ok(HealthReport {
        healthy: summary.failed == 0,
        checks,
        summary,
    })
}

/// CLI wrapper
pub async fn handle_maintain_health(...) -> Result<()> {
    let report = run_health_checks_internal(...).await?;
    print_health_report(&report, &format)?;
    if !report.healthy {
        std::process::exit(1);
    }
    Ok(())
}
```

### MCP Handler

```rust
async fn health_check_internal(&self, params: Value) -> Result<Value> {
    let project_dir = params.get("project_dir")...;
    let quick = params.get("quick")...;
    // Extract all check flags

    let report = run_health_checks_internal(
        &PathBuf::from(project_dir),
        quick, all,
        check_build, check_tests, check_coverage,
        check_complexity, check_satd,
    ).await?;

    Ok(json!({
        "project_dir": project_dir,
        "healthy": report.healthy,
        "checks": report.checks,
        "summary": report.summary,
        "message": if report.healthy { "Passed" } else { "Failed" }
    }))
}
```

## Key Features

1. **Parallel Execution:** Reuses PMAT-6010 parallel health checks
2. **All Check Types:** Build, tests, coverage, complexity, SATD
3. **Quick Mode:** Fast subset of checks
4. **Structured Results:** `HealthReport` with detailed check data

## Acceptance Criteria

- [x] Extracted `run_health_checks_internal()` function
- [x] MCP handler calls real health checks
- [x] Uses parallel execution from PMAT-6010
- [x] Returns actual HealthReport with all checks
- [x] CLI wrapper maintains existing behavior
- [x] Error handling with McpOperationResult
- [x] Code compiles successfully

## Files Modified

1. **server/src/cli/handlers/health_handler.rs**
   - Added `run_health_checks_internal()` (+50 lines)
   - Refactored `handle_maintain_health()` to call internal version

2. **server/src/contracts/mcp_impl.rs**
   - Replaced mock `handle_health_check()`
   - Added `health_check_internal()` (+60 lines)

## Usage Examples

**Quick check:**
```json
{
  "project_dir": ".",
  "quick": true
}
```

**Specific checks:**
```json
{
  "project_dir": ".",
  "check_build": true,
  "check_tests": true
}
```

**All checks:**
```json
{
  "project_dir": "."
}
```

## Impact

**Before:** Mock health data
**After:**
- ✅ Real build checks
- ✅ Real test execution
- ✅ Real coverage analysis
- ✅ Parallel execution (14-40% faster!)
- ✅ Detailed check results

---

*Completed: October 6, 2025*
*Sprint 22 - MCP Phase 2*
