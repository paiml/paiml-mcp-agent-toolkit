//! Project health check handlers
//!
//! This module provides functionality for checking overall project health
//! by running multiple quality checks and generating consolidated reports.

use crate::cli::colors;
use crate::cli::OutputFormat;
use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;
use tokio::task::JoinSet;

/// Configuration for health checks
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    pub quick: bool,
    pub all: bool,
    pub check_build: bool,
    pub check_tests: bool,
    pub check_coverage: bool,
    pub check_complexity: bool,
    pub check_satd: bool,
}

impl HealthCheckConfig {
    /// Create config from individual flags
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(
        quick: bool,
        all: bool,
        check_build: bool,
        check_tests: bool,
        check_coverage: bool,
        check_complexity: bool,
        check_satd: bool,
    ) -> Self {
        Self {
            quick,
            all,
            check_build,
            check_tests,
            check_coverage,
            check_complexity,
            check_satd,
        }
    }
}

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
/// Status of check operation.
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

/// Check type for parallel execution (TICKET-PMAT-6010)
#[derive(Debug, Clone, Copy)]
enum CheckType {
    Build,
    Tests,
    Coverage,
    Complexity,
    Satd,
}

/// Run health checks and return report (internal, reusable)
/// (TICKET-PMAT-6020)
///
/// # Complexity
/// - Time: O(n) where n is project size
/// - Cyclomatic: 7
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn run_health_checks_internal(
    project_dir: &PathBuf,
    config: &HealthCheckConfig,
) -> Result<HealthReport> {
    // Determine which checks to run based on flags
    let checks_to_run = determine_checks_to_run(
        config.quick,
        config.all,
        config.check_build,
        config.check_tests,
        config.check_coverage,
        config.check_complexity,
        config.check_satd,
    );

    // Build list of check types to run (TICKET-PMAT-6010: parallel execution)
    let mut check_types = Vec::new();
    if checks_to_run.build {
        check_types.push(CheckType::Build);
    }
    if checks_to_run.tests {
        check_types.push(CheckType::Tests);
    }
    if checks_to_run.coverage {
        check_types.push(CheckType::Coverage);
    }
    if checks_to_run.complexity {
        check_types.push(CheckType::Complexity);
    }
    if checks_to_run.satd {
        check_types.push(CheckType::Satd);
    }

    // Run checks in parallel (TICKET-PMAT-6010)
    let checks = run_checks_parallel(project_dir, check_types).await?;

    let summary = calculate_summary(&checks);
    let report = HealthReport {
        // `failed == 0` also holds when every check SKIPPED, so a run that
        // measured nothing at all reported "✨ Project is healthy!" and exited
        // 0 — `pmat maintain health` in an empty directory (Build: "No
        // Cargo.toml found", 1 skipped, 0 of everything else) passed. Health
        // has to be observed: at least one check must have reached a verdict.
        healthy: summary.total_checks > summary.skipped && summary.failed == 0,
        checks,
        summary,
    };

    Ok(report)
}

/// Handle project health check command (CLI wrapper)
/// (TICKET-PMAT-6001, PMAT-6010)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_maintain_health(
    project_dir: PathBuf,
    format: OutputFormat,
    config: HealthCheckConfig,
) -> Result<()> {
    let report = run_health_checks_internal(&project_dir, &config).await?;

    print_health_report(&report, &format)?;

    if !report.healthy {
        std::process::exit(1);
    }

    Ok(())
}

include!("health_handler_output.rs");
include!("health_handler_checks.rs");
include!("health_handler_tests.rs");
