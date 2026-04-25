#![cfg_attr(coverage_nightly, coverage(off))]
//! Project Diagnostic handlers for Rust project analysis
//!
//! Provides the `pmat project-diag` command for diagnosing Rust projects.
//! Implements 20 diagnostic checks across 5 categories, matching lltop Tab 8.
//!
//! Categories:
//! - Cargo Config (6 checks): Edition, Resolver, Dependencies, LTO, Workspace lints, Workspace deps
//! - Dependencies (3 checks): Target dir size, Cargo.lock, Audit config
//! - Build Performance (4 checks): Cargo config, Incremental builds, Codegen units, Build system
//! - Code Quality (4 checks): Clippy config, Rustfmt config, Tests present, README
//! - Advanced (3 checks): MSRV defined, Benchmarks, CI configured

use crate::cli::colors;
use crate::cli::commands::ProjectDiagOutputFormat;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Configuration for project diagnostics command
pub struct ProjectDiagConfig {
    pub path: PathBuf,
    pub format: ProjectDiagOutputFormat,
    pub category: Option<String>,
    pub failures_only: bool,
    pub output: Option<PathBuf>,
    pub quiet: bool,
}

/// Health status for a diagnostic check
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Check passed (green)
    Green,
    /// Check has warnings (yellow)
    Yellow,
    /// Check failed (red)
    Red,
    /// Check was skipped
    Skip,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Green => write!(f, "GREEN"),
            HealthStatus::Yellow => write!(f, "YELLOW"),
            HealthStatus::Red => write!(f, "RED"),
            HealthStatus::Skip => write!(f, "SKIP"),
        }
    }
}

/// Single diagnostic check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCheck {
    pub name: String,
    pub category: String,
    pub status: HealthStatus,
    pub message: String,
    pub score: f64,
    pub max_score: f64,
}

/// Complete diagnostic report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub project_path: String,
    pub total_score: f64,
    pub max_score: f64,
    pub percentage: f64,
    pub overall_status: HealthStatus,
    pub checks: Vec<DiagnosticCheck>,
    pub categories: Vec<CategorySummary>,
}

/// Summary for a category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub name: String,
    pub passed: usize,
    pub warned: usize,
    pub failed: usize,
    pub total: usize,
    pub score: f64,
    pub max_score: f64,
}

/// Handle the project-diag command
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_project_diag(config: ProjectDiagConfig) -> Result<()> {
    // Validate path
    if !config.path.exists() {
        anyhow::bail!("Path not found: {}", config.path.display());
    }

    if !config.path.is_dir() {
        anyhow::bail!("Path is not a directory: {}", config.path.display());
    }

    // Verify it's a Rust project
    if !config.path.join("Cargo.toml").exists() {
        anyhow::bail!(
            "Not a Rust project: no Cargo.toml found at {}",
            config.path.display()
        );
    }

    // Run all diagnostics
    let report = run_diagnostics(&config.path, &config.category)?;

    // Format output
    let output_text = match config.format {
        ProjectDiagOutputFormat::Summary => format_summary(&report, config.failures_only),
        ProjectDiagOutputFormat::Json => format_json(&report)?,
        ProjectDiagOutputFormat::Markdown => format_markdown(&report, config.failures_only),
        ProjectDiagOutputFormat::Andon => format_andon(&report),
    };

    // Write output
    if let Some(output_path) = &config.output {
        std::fs::write(output_path, &output_text)?;
        if !config.quiet {
            println!("Diagnostic report written to: {}", output_path.display());
        }
    } else {
        print!("{}", output_text);
    }

    Ok(())
}

/// Run all 20 diagnostic checks
fn run_diagnostics(
    project_path: &Path,
    category_filter: &Option<String>,
) -> Result<DiagnosticReport> {
    let mut checks = Vec::new();

    // Cargo Config category (6 checks)
    if should_include_category("cargo", category_filter) {
        checks.push(check_edition_2021(project_path));
        checks.push(check_resolver_v2(project_path));
        checks.push(check_dependency_count(project_path));
        checks.push(check_lto_enabled(project_path));
        checks.push(check_workspace_lints(project_path));
        checks.push(check_workspace_deps(project_path));
    }

    // Dependencies category (3 checks)
    if should_include_category("deps", category_filter) {
        checks.push(check_target_dir_size(project_path));
        checks.push(check_cargo_lock(project_path));
        checks.push(check_audit_config(project_path));
    }

    // Build Performance category (4 checks)
    if should_include_category("build", category_filter) {
        checks.push(check_cargo_config(project_path));
        checks.push(check_incremental_builds(project_path));
        checks.push(check_codegen_units(project_path));
        checks.push(check_build_system(project_path));
    }

    // Code Quality category (4 checks)
    if should_include_category("quality", category_filter) {
        checks.push(check_clippy_config(project_path));
        checks.push(check_rustfmt_config(project_path));
        checks.push(check_tests_present(project_path));
        checks.push(check_readme(project_path));
    }

    // Advanced category (3 checks)
    if should_include_category("advanced", category_filter) {
        checks.push(check_msrv_defined(project_path));
        checks.push(check_benchmarks(project_path));
        checks.push(check_ci_configured(project_path));
    }

    // Calculate totals
    let total_score: f64 = checks.iter().map(|c| c.score).sum();
    let max_score: f64 = checks.iter().map(|c| c.max_score).sum();
    let percentage = if max_score > 0.0 {
        (total_score / max_score) * 100.0
    } else {
        0.0
    };

    // Determine overall status
    let overall_status = if percentage >= 85.0 {
        HealthStatus::Green
    } else if percentage >= 60.0 {
        HealthStatus::Yellow
    } else {
        HealthStatus::Red
    };

    // Build category summaries
    let categories = build_category_summaries(&checks);

    Ok(DiagnosticReport {
        project_path: project_path.display().to_string(),
        total_score,
        max_score,
        percentage,
        overall_status,
        checks,
        categories,
    })
}

fn should_include_category(category: &str, filter: &Option<String>) -> bool {
    match filter {
        None => true,
        Some(f) => f.to_lowercase() == category,
    }
}

fn build_category_summaries(checks: &[DiagnosticCheck]) -> Vec<CategorySummary> {
    let categories = [
        "Cargo Config",
        "Dependencies",
        "Build Performance",
        "Code Quality",
        "Advanced",
    ];

    categories
        .iter()
        .filter_map(|&cat| {
            let cat_checks: Vec<_> = checks.iter().filter(|c| c.category == cat).collect();
            if cat_checks.is_empty() {
                return None;
            }

            let passed = cat_checks
                .iter()
                .filter(|c| c.status == HealthStatus::Green)
                .count();
            let warned = cat_checks
                .iter()
                .filter(|c| c.status == HealthStatus::Yellow)
                .count();
            let failed = cat_checks
                .iter()
                .filter(|c| c.status == HealthStatus::Red)
                .count();
            let score: f64 = cat_checks.iter().map(|c| c.score).sum();
            let max_score: f64 = cat_checks.iter().map(|c| c.max_score).sum();

            Some(CategorySummary {
                name: cat.to_string(),
                passed,
                warned,
                failed,
                total: cat_checks.len(),
                score,
                max_score,
            })
        })
        .collect()
}

// Cargo Config Checks (6) + Dependencies Checks (3)
include!("project_diag_cargo_checks.rs");

// Build Performance Checks (4) + Code Quality Checks (4)
include!("project_diag_build_quality_checks.rs");

// Advanced Checks (3) + Helper Functions + Output Formatters
include!("project_diag_advanced_formatters.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_check_creation() {
        let check = DiagnosticCheck {
            name: "Test".to_string(),
            category: "Test Category".to_string(),
            status: HealthStatus::Green,
            message: "OK".to_string(),
            score: 5.0,
            max_score: 5.0,
        };
        assert_eq!(check.name, "Test");
        assert_eq!(check.status, HealthStatus::Green);
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(format!("{}", HealthStatus::Green), "GREEN");
        assert_eq!(format!("{}", HealthStatus::Yellow), "YELLOW");
        assert_eq!(format!("{}", HealthStatus::Red), "RED");
    }

    #[test]
    fn test_run_diagnostics_on_pmat() {
        // Run diagnostics on the pmat project itself
        let result = run_diagnostics(std::path::Path::new("."), &None);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(report.total_score > 0.0);
        assert!(report.max_score > 0.0);
    }

    // ─────────────────────────────────────────────────────────────────────
    // project_diag_advanced_formatters.rs: format_summary / format_json /
    // format_markdown / format_andon dispatchers + helper checks.
    // ─────────────────────────────────────────────────────────────────────

    fn make_check(name: &str, status: HealthStatus) -> DiagnosticCheck {
        DiagnosticCheck {
            name: name.to_string(),
            category: "Test".to_string(),
            status,
            message: format!("{name}-msg"),
            score: 5.0,
            max_score: 5.0,
        }
    }

    fn make_category(
        name: &str,
        passed: usize,
        warned: usize,
        failed: usize,
        total: usize,
    ) -> CategorySummary {
        CategorySummary {
            name: name.to_string(),
            passed,
            warned,
            failed,
            total,
            score: passed as f64 * 5.0,
            max_score: total as f64 * 5.0,
        }
    }

    fn make_report(checks: Vec<DiagnosticCheck>, status: HealthStatus) -> DiagnosticReport {
        DiagnosticReport {
            project_path: "/test/path".to_string(),
            total_score: 80.0,
            max_score: 100.0,
            percentage: 80.0,
            overall_status: status,
            categories: vec![
                make_category("Cat1", 2, 1, 1, 4),
                make_category("Cat2", 4, 0, 0, 4),
            ],
            checks,
        }
    }

    fn report_with_all_4_status_arms() -> DiagnosticReport {
        make_report(
            vec![
                make_check("G", HealthStatus::Green),
                make_check("Y", HealthStatus::Yellow),
                make_check("R", HealthStatus::Red),
                make_check("S", HealthStatus::Skip),
            ],
            HealthStatus::Yellow,
        )
    }

    // ── format_summary: 4 status arms × failures_only toggle ──

    #[test]
    fn test_format_summary_includes_all_4_status_arms() {
        let r = format_summary(&report_with_all_4_status_arms(), false);
        // All 4 check names appear in the output.
        assert!(r.contains("G - "));
        assert!(r.contains("Y - "));
        assert!(r.contains("R - "));
        assert!(r.contains("S - "));
    }

    #[test]
    fn test_format_summary_failures_only_skips_green_checks() {
        let r = format_summary(&report_with_all_4_status_arms(), true);
        // Green check skipped, others retained.
        assert!(!r.contains("G - "));
        assert!(r.contains("Y - "));
        assert!(r.contains("R - "));
        assert!(r.contains("S - "));
    }

    #[test]
    fn test_format_summary_overall_status_arms() {
        // Each HealthStatus → distinct overall icon; verify all branches reachable.
        for st in [
            HealthStatus::Green,
            HealthStatus::Yellow,
            HealthStatus::Red,
            HealthStatus::Skip,
        ] {
            let r = format_summary(&make_report(vec![], st), false);
            assert!(r.contains("Overall:"));
        }
    }

    // ── format_json: smoke (parsable) ──

    #[test]
    fn test_format_json_returns_parsable_json() {
        let r = format_json(&report_with_all_4_status_arms()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&r).unwrap();
        assert_eq!(parsed["project_path"], "/test/path");
        assert!(parsed["checks"].is_array());
    }

    // ── format_markdown: badge arms + failures_only + emoji arms ──

    #[test]
    fn test_format_markdown_emits_status_badge_per_overall_status() {
        for (st, badge) in [
            (HealthStatus::Green, "status-healthy-green"),
            (HealthStatus::Yellow, "status-warning-yellow"),
            (HealthStatus::Red, "status-critical-red"),
            (HealthStatus::Skip, "status-skipped-gray"),
        ] {
            let r = format_markdown(&make_report(vec![], st), false);
            assert!(r.contains(badge), "badge for {st:?} missing");
        }
    }

    #[test]
    fn test_format_markdown_includes_emoji_for_each_check_status_arm() {
        let r = format_markdown(&report_with_all_4_status_arms(), false);
        assert!(r.contains("✅"));
        assert!(r.contains("⚠️"));
        assert!(r.contains("❌"));
        assert!(r.contains("⏭️"));
    }

    #[test]
    fn test_format_markdown_failures_only_skips_green_rows() {
        let r = format_markdown(&report_with_all_4_status_arms(), true);
        assert!(!r.contains("✅"));
        assert!(r.contains("⚠️"));
    }

    // ── format_andon: progress-bar color arms + failures section gating ──

    #[test]
    fn test_format_andon_no_failures_emits_production_ready_banner() {
        let r = format_andon(&make_report(
            vec![make_check("ok", HealthStatus::Green)],
            HealthStatus::Green,
        ));
        assert!(r.contains("No critical issues - production ready"));
    }

    #[test]
    fn test_format_andon_with_red_check_triggers_andon_cord() {
        let r = format_andon(&make_report(
            vec![make_check("broken", HealthStatus::Red)],
            HealthStatus::Red,
        ));
        assert!(r.contains("ANDON CORD TRIGGERED"));
        // Failed check name appears in the alert section.
        assert!(r.contains("broken"));
    }

    #[test]
    fn test_format_andon_caps_failures_list_at_5() {
        let mut checks = vec![];
        for i in 0..10 {
            checks.push(make_check(&format!("fail{i}"), HealthStatus::Red));
        }
        let r = format_andon(&make_report(checks, HealthStatus::Red));
        // First 5 included.
        assert!(r.contains("fail0"));
        assert!(r.contains("fail4"));
        // 6th+ excluded.
        assert!(!r.contains("fail5"));
        assert!(!r.contains("fail9"));
    }

    #[test]
    fn test_format_andon_progress_bar_color_arms() {
        // ≥85.0 → GREEN, ≥60.0 → YELLOW, else RED. Verify all three reachable.
        let mut high = make_report(vec![], HealthStatus::Green);
        high.percentage = 90.0;
        format_andon(&high);
        let mut mid = make_report(vec![], HealthStatus::Yellow);
        mid.percentage = 70.0;
        format_andon(&mid);
        let mut low = make_report(vec![], HealthStatus::Red);
        low.percentage = 40.0;
        format_andon(&low);
    }

    #[test]
    fn test_format_andon_category_light_arms_red_yellow_green() {
        // failed > 0 → red light, warned > 0 → yellow, else green.
        let mut report = make_report(vec![], HealthStatus::Yellow);
        report.categories = vec![
            make_category("FailCat", 0, 0, 1, 1), // red light
            make_category("WarnCat", 0, 1, 0, 1), // yellow light
            make_category("PassCat", 1, 0, 0, 1), // green light
        ];
        let r = format_andon(&report);
        assert!(r.contains("FailCat"));
        assert!(r.contains("WarnCat"));
        assert!(r.contains("PassCat"));
    }
}
