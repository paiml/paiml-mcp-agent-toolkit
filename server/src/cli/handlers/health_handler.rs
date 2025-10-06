//! Project health check handlers
//!
//! This module provides functionality for checking overall project health
//! by running multiple quality checks and generating consolidated reports.

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

/// Handle project health check command
///
/// # Complexity
/// - Time: O(n) where n is project size
/// - Cyclomatic: 7
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
async fn run_complexity_check(_project_dir: &PathBuf) -> Result<HealthCheck> {
    // Simplified: Just return skip for now
    // Full implementation would use complexity analysis service
    Ok(HealthCheck {
        name: "Complexity".to_string(),
        status: CheckStatus::Skip,
        message: "Complexity check not yet implemented".to_string(),
        details: Some("Use 'pmat analyze complexity' for detailed analysis".to_string()),
    })
}

/// Run SATD health check
async fn run_satd_check(_project_dir: &PathBuf) -> Result<HealthCheck> {
    // Simplified: Just return skip for now
    // Full implementation would use SATD detection service
    Ok(HealthCheck {
        name: "SATD".to_string(),
        status: CheckStatus::Skip,
        message: "SATD check not yet implemented".to_string(),
        details: Some("Use 'pmat analyze satd' for detailed analysis".to_string()),
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

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn summary_totals_match(passed in 0u32..100, warned in 0u32..100, failed in 0u32..100, skipped in 0u32..100) {
            let mut checks = Vec::new();

            for _ in 0..passed {
                checks.push(HealthCheck {
                    name: "Pass".to_string(),
                    status: CheckStatus::Pass,
                    message: "OK".to_string(),
                    details: None,
                });
            }

            for _ in 0..warned {
                checks.push(HealthCheck {
                    name: "Warn".to_string(),
                    status: CheckStatus::Warn,
                    message: "Warning".to_string(),
                    details: None,
                });
            }

            for _ in 0..failed {
                checks.push(HealthCheck {
                    name: "Fail".to_string(),
                    status: CheckStatus::Fail,
                    message: "Failed".to_string(),
                    details: None,
                });
            }

            for _ in 0..skipped {
                checks.push(HealthCheck {
                    name: "Skip".to_string(),
                    status: CheckStatus::Skip,
                    message: "Skipped".to_string(),
                    details: None,
                });
            }

            let summary = calculate_summary(&checks);

            prop_assert_eq!(summary.total_checks, checks.len());
            prop_assert_eq!(summary.passed, passed as usize);
            prop_assert_eq!(summary.warned, warned as usize);
            prop_assert_eq!(summary.failed, failed as usize);
            prop_assert_eq!(summary.skipped, skipped as usize);
        }
    }
}
