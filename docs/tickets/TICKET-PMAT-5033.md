# TICKET-PMAT-5033: Add `pmat maintain health` CLI Command

**Status**: GREEN
**Priority**: P1
**Complexity**: 4
**Estimated Time**: 60 minutes
**Dependencies**: Existing analysis infrastructure, TICKET-PMAT-5032
**Sprint**: Sprint 19 - CLI Integration & Dogfooding

## Objective

Add `pmat maintain health` subcommand to check overall project health by running multiple quality checks (build, tests, coverage, complexity, SATD) and generating a consolidated health report. This provides a single command to assess project quality.

## Success Criteria

- [ ] `pmat maintain health` runs all health checks and generates report
- [ ] Individual checks can be enabled/disabled with flags
- [ ] Exit code 0 if healthy, 1 if issues found
- [ ] Output formats: table (default), json, yaml
- [ ] All quality gates pass (complexity <10, coverage >80%, no SATD)

## Current State

**Already Exists:**
- MaintainCommands::Health variant in commands.rs
- Analysis infrastructure (complexity, SATD, coverage, etc.)
- Quality gate system

**Missing:**
- Handler function `handle_maintain_health()`
- Health check orchestration logic
- Consolidated reporting

## Test Strategy

### Unit Tests
- [ ] `test_health_check_all_passing` - All checks pass
- [ ] `test_health_check_with_failures` - Some checks fail
- [ ] `test_health_check_selective` - Run only selected checks
- [ ] `test_health_report_formatting` - Output formats correct

### Integration Tests
- [ ] `integration_health_check_real_project` - Run on PMAT itself
- [ ] `integration_health_exit_codes` - Verify exit code behavior

## Quality Gates

- [ ] Cyclomatic complexity <10 for all functions
- [ ] Cognitive complexity <15 for all functions
- [ ] Line coverage >80%
- [ ] Branch coverage >80%
- [ ] 0 SATD violations
- [ ] 0 clippy warnings
- [ ] All tests pass

## Implementation Plan

### Phase 1: Create Health Check Types

```rust
// server/src/cli/handlers/health_handler.rs

use anyhow::Result;
use crate::cli::OutputFormat;
use serde::Serialize;
use std::path::PathBuf;

/// Health check result
#[derive(Debug, Serialize)]
pub struct HealthReport {
    pub healthy: bool,
    pub checks: Vec<HealthCheck>,
    pub summary: HealthSummary,
}

/// Individual health check
#[derive(Debug, Serialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

/// Health summary
#[derive(Debug, Serialize)]
pub struct HealthSummary {
    pub total_checks: usize,
    pub passed: usize,
    pub warned: usize,
    pub failed: usize,
    pub skipped: usize,
}
```

### Phase 2: Implement Health Checks

```rust
/// Handle project health check command
///
/// # Complexity
/// - Time: O(n) where n is project size
/// - Cyclomatic: 8
pub async fn handle_maintain_health(
    project_dir: PathBuf,
    format: OutputFormat,
    check_build: bool,
    check_tests: bool,
    check_coverage: bool,
    check_complexity: bool,
    check_satd: bool,
) -> Result<()> {
    let mut checks = Vec::new();

    if check_build {
        checks.push(run_build_check(&project_dir).await?);
    }

    if check_tests {
        checks.push(run_test_check(&project_dir).await?);
    }

    if check_coverage {
        checks.push(run_coverage_check(&project_dir).await?);
    }

    if check_complexity {
        checks.push(run_complexity_check(&project_dir).await?);
    }

    if check_satd {
        checks.push(run_satd_check(&project_dir).await?);
    }

    let summary = calculate_summary(&checks);
    let report = HealthReport {
        healthy: summary.failed == 0,
        checks,
        summary,
    };

    print_health_report(&report, &format)?;

    if !report.healthy {
        std::process::exit(1);
    }

    Ok(())
}

/// Run build health check
async fn run_build_check(project_dir: &PathBuf) -> Result<HealthCheck> {
    // Check if Cargo.toml exists
    let cargo_toml = project_dir.join("Cargo.toml");

    if !cargo_toml.exists() {
        return Ok(HealthCheck {
            name: "Build".to_string(),
            status: CheckStatus::Skip,
            message: "No Cargo.toml found".to_string(),
            details: None,
        });
    }

    // Try to build
    let output = tokio::process::Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(project_dir)
        .output()
        .await?;

    if output.status.success() {
        Ok(HealthCheck {
            name: "Build".to_string(),
            status: CheckStatus::Pass,
            message: "Project builds successfully".to_string(),
            details: None,
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(HealthCheck {
            name: "Build".to_string(),
            status: CheckStatus::Fail,
            message: "Build failed".to_string(),
            details: Some(stderr.lines().take(5).collect::<Vec<_>>().join("\n")),
        })
    }
}

/// Run test health check
async fn run_test_check(project_dir: &PathBuf) -> Result<HealthCheck> {
    let output = tokio::process::Command::new("cargo")
        .arg("test")
        .arg("--quiet")
        .arg("--no-fail-fast")
        .current_dir(project_dir)
        .output()
        .await?;

    if output.status.success() {
        Ok(HealthCheck {
            name: "Tests".to_string(),
            status: CheckStatus::Pass,
            message: "All tests passing".to_string(),
            details: None,
        })
    } else {
        Ok(HealthCheck {
            name: "Tests".to_string(),
            status: CheckStatus::Fail,
            message: "Some tests failing".to_string(),
            details: None,
        })
    }
}

/// Run coverage health check
async fn run_coverage_check(project_dir: &PathBuf) -> Result<HealthCheck> {
    // Use cargo llvm-cov to get coverage
    let output = tokio::process::Command::new("cargo")
        .arg("llvm-cov")
        .arg("--quiet")
        .arg("--summary-only")
        .current_dir(project_dir)
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            // Parse coverage percentage from output
            // Expected format: "TOTAL   1234   1000   80.0%"
            let coverage = parse_coverage_percentage(&stdout);

            let status = if coverage >= 80.0 {
                CheckStatus::Pass
            } else if coverage >= 60.0 {
                CheckStatus::Warn
            } else {
                CheckStatus::Fail
            };

            Ok(HealthCheck {
                name: "Coverage".to_string(),
                status,
                message: format!("Coverage: {:.1}%", coverage),
                details: Some(format!("Target: ≥80%, Current: {:.1}%", coverage)),
            })
        }
        _ => Ok(HealthCheck {
            name: "Coverage".to_string(),
            status: CheckStatus::Skip,
            message: "cargo-llvm-cov not available".to_string(),
            details: Some("Install with: cargo install cargo-llvm-cov".to_string()),
        }),
    }
}

/// Run complexity health check
async fn run_complexity_check(project_dir: &PathBuf) -> Result<HealthCheck> {
    use crate::services::complexity::calculate_complexity_for_directory;

    let result = calculate_complexity_for_directory(project_dir).await;

    match result {
        Ok(stats) => {
            let max_complexity = stats
                .functions
                .iter()
                .map(|f| f.cyclomatic_complexity)
                .max()
                .unwrap_or(0);

            let status = if max_complexity <= 10 {
                CheckStatus::Pass
            } else if max_complexity <= 15 {
                CheckStatus::Warn
            } else {
                CheckStatus::Fail
            };

            Ok(HealthCheck {
                name: "Complexity".to_string(),
                status,
                message: format!("Max complexity: {}", max_complexity),
                details: Some(format!(
                    "Target: ≤10, Current max: {}, Avg: {:.1}",
                    max_complexity, stats.average_complexity
                )),
            })
        }
        Err(e) => Ok(HealthCheck {
            name: "Complexity".to_string(),
            status: CheckStatus::Fail,
            message: format!("Complexity check failed: {}", e),
            details: None,
        }),
    }
}

/// Run SATD health check
async fn run_satd_check(project_dir: &PathBuf) -> Result<HealthCheck> {
    use crate::services::satd::detect_satd_in_directory;

    let satd_items = detect_satd_in_directory(project_dir).await?;

    let status = if satd_items.is_empty() {
        CheckStatus::Pass
    } else if satd_items.len() <= 5 {
        CheckStatus::Warn
    } else {
        CheckStatus::Fail
    };

    Ok(HealthCheck {
        name: "SATD".to_string(),
        status,
        message: format!("{} SATD items found", satd_items.len()),
        details: if satd_items.is_empty() {
            None
        } else {
            Some(format!(
                "Found {} TODO/FIXME/HACK comments",
                satd_items.len()
            ))
        },
    })
}

/// Parse coverage percentage from llvm-cov output
fn parse_coverage_percentage(output: &str) -> f64 {
    for line in output.lines() {
        if line.contains("TOTAL") {
            // Expected format: "TOTAL   1234   1000   80.0%"
            if let Some(pct_str) = line.split_whitespace().last() {
                if let Some(num_str) = pct_str.strip_suffix('%') {
                    if let Ok(pct) = num_str.parse::<f64>() {
                        return pct;
                    }
                }
            }
        }
    }
    0.0
}

/// Calculate summary from checks
fn calculate_summary(checks: &[HealthCheck]) -> HealthSummary {
    let mut summary = HealthSummary {
        total_checks: checks.len(),
        passed: 0,
        warned: 0,
        failed: 0,
        skipped: 0,
    };

    for check in checks {
        match check.status {
            CheckStatus::Pass => summary.passed += 1,
            CheckStatus::Warn => summary.warned += 1,
            CheckStatus::Fail => summary.failed += 1,
            CheckStatus::Skip => summary.skipped += 1,
        }
    }

    summary
}

/// Print health report
fn print_health_report(report: &HealthReport, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(report)?);
        }
        OutputFormat::Yaml => {
            print_health_yaml(report);
        }
        OutputFormat::Table => {
            print_health_table(report);
        }
    }
    Ok(())
}

/// Print health report as table
fn print_health_table(report: &HealthReport) {
    let overall_icon = if report.healthy { "✅" } else { "❌" };
    eprintln!("{} Project Health Report\n", overall_icon);

    for check in &report.checks {
        let icon = match check.status {
            CheckStatus::Pass => "✅",
            CheckStatus::Warn => "⚠️ ",
            CheckStatus::Fail => "❌",
            CheckStatus::Skip => "⏭️ ",
        };

        eprintln!("{} {}: {}", icon, check.name, check.message);
        if let Some(details) = &check.details {
            eprintln!("   {}", details);
        }
    }

    eprintln!("\n📊 Summary:");
    eprintln!("   Total:   {}", report.summary.total_checks);
    eprintln!("   Passed:  {}", report.summary.passed);
    eprintln!("   Warned:  {}", report.summary.warned);
    eprintln!("   Failed:  {}", report.summary.failed);
    eprintln!("   Skipped: {}", report.summary.skipped);

    if report.healthy {
        eprintln!("\n✨ Project is healthy!");
    } else {
        eprintln!("\n⚠️  Project has {} issue(s)", report.summary.failed);
    }
}

/// Print health report as YAML
fn print_health_yaml(report: &HealthReport) {
    println!("healthy: {}", report.healthy);
    println!("checks:");
    for check in &report.checks {
        println!("  - name: {}", check.name);
        println!("    status: {:?}", check.status);
        println!("    message: {}", check.message);
        if let Some(details) = &check.details {
            println!("    details: {}", details);
        }
    }
    println!("summary:");
    println!("  total_checks: {}", report.summary.total_checks);
    println!("  passed: {}", report.summary.passed);
    println!("  warned: {}", report.summary.warned);
    println!("  failed: {}", report.summary.failed);
    println!("  skipped: {}", report.summary.skipped);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_summary_all_pass() {
        let checks = vec![
            HealthCheck {
                name: "Test1".to_string(),
                status: CheckStatus::Pass,
                message: "OK".to_string(),
                details: None,
            },
            HealthCheck {
                name: "Test2".to_string(),
                status: CheckStatus::Pass,
                message: "OK".to_string(),
                details: None,
            },
        ];

        let summary = calculate_summary(&checks);
        assert_eq!(summary.total_checks, 2);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 0);
    }

    #[test]
    fn test_calculate_summary_mixed() {
        let checks = vec![
            HealthCheck {
                name: "Test1".to_string(),
                status: CheckStatus::Pass,
                message: "OK".to_string(),
                details: None,
            },
            HealthCheck {
                name: "Test2".to_string(),
                status: CheckStatus::Warn,
                message: "Warning".to_string(),
                details: None,
            },
            HealthCheck {
                name: "Test3".to_string(),
                status: CheckStatus::Fail,
                message: "Failed".to_string(),
                details: None,
            },
        ];

        let summary = calculate_summary(&checks);
        assert_eq!(summary.total_checks, 3);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.warned, 1);
        assert_eq!(summary.failed, 1);
    }

    #[test]
    fn test_parse_coverage_valid() {
        let output = "Filename                      Regions    Missed Regions     Cover   Functions  Missed Functions  Executed\n\
                      TOTAL   1234   234   81.0%";
        let coverage = parse_coverage_percentage(output);
        assert_eq!(coverage, 81.0);
    }

    #[test]
    fn test_parse_coverage_invalid() {
        let output = "No coverage data";
        let coverage = parse_coverage_percentage(output);
        assert_eq!(coverage, 0.0);
    }
}
```

### Phase 3: Update Command Arguments

```rust
// server/src/cli/commands.rs

/// Validate project health (TICKET-PMAT-5033)
Health {
    /// Project directory
    #[arg(long, default_value = ".")]
    project_dir: PathBuf,

    /// Output format
    #[arg(long, value_enum, default_value = "table")]
    format: OutputFormat,

    /// Check build status
    #[arg(long, default_value = "true")]
    check_build: bool,

    /// Check tests
    #[arg(long, default_value = "true")]
    check_tests: bool,

    /// Check coverage
    #[arg(long, default_value = "true")]
    check_coverage: bool,

    /// Check complexity
    #[arg(long, default_value = "true")]
    check_complexity: bool,

    /// Check SATD
    #[arg(long, default_value = "true")]
    check_satd: bool,
},
```

### Phase 4: Wire Up Handler

```rust
// server/src/cli/command_structure.rs

MaintainCommands::Health {
    project_dir,
    format,
    check_build,
    check_tests,
    check_coverage,
    check_complexity,
    check_satd,
} => {
    super::handlers::handle_maintain_health(
        project_dir,
        format,
        check_build,
        check_tests,
        check_coverage,
        check_complexity,
        check_satd,
    )
    .await
}
```

## Complexity Analysis

Functions with complexity:
- `handle_maintain_health`: CC=7 (5 check flags + healthy check + exit)
- `run_build_check`: CC=3 (exists check, success check)
- `run_coverage_check`: CC=4 (match, coverage thresholds)
- `calculate_summary`: CC=5 (4 status variants + loop)

All functions under CC=10 threshold ✓

## Verification Commands

```bash
# Run all health checks
pmat maintain health

# Run specific checks only
pmat maintain health --check-build --no-check-tests

# Output in JSON
pmat maintain health --format json

# Check specific project
pmat maintain health --project-dir /path/to/project

# CI integration
pmat maintain health && echo "Healthy!" || echo "Issues found"
```

## Files to Create/Modify

### New Files
- `server/src/cli/handlers/health_handler.rs` - Health check implementation

### Modified Files
- `server/src/cli/commands.rs` - Update Health variant with check flags
- `server/src/cli/handlers/mod.rs` - Export health handler
- `server/src/cli/command_structure.rs` - Wire up Health handler
- `server/src/cli/command_dispatcher.rs` - Wire up Health dispatcher

## Risk Assessment

**Medium Risk:**
- Running external commands (cargo check, cargo test)
- Parsing command output might be fragile
- Long-running operations

**Mitigation:**
- Timeout on external commands
- Graceful degradation if tools not available
- Skip checks that can't run rather than fail
- Clear error messages

## Notes

This command enables comprehensive health checks in a single invocation:

**Use Cases:**
1. **CI/CD**: Single command to validate all quality aspects
2. **Pre-commit**: Check health before pushing
3. **Sprint reviews**: Assess overall project quality
4. **Onboarding**: New devs can check setup

**Value:**
- Consolidates multiple checks into one command
- Consistent quality assessment
- Actionable feedback with details
- Exit codes for automation

**Integration:**
- Works alongside `pmat quality-gates` (more detailed)
- Complements `pmat maintain roadmap` (project tracking)
- Quick health overview vs. detailed gate enforcement

**TDD Cycle Duration**: Estimated 60 minutes for RED → GREEN → REFACTOR
