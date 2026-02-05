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

// ============================================================================
// Cargo Config Checks (6)
// ============================================================================

fn check_edition_2021(project_path: &Path) -> DiagnosticCheck {
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let (status, score, message) = if content.contains("edition = \"2021\"") {
        (
            HealthStatus::Green,
            5.0,
            "Edition 2021 configured".to_string(),
        )
    } else if content.contains("edition = \"2024\"") {
        (
            HealthStatus::Green,
            5.0,
            "Edition 2024 configured".to_string(),
        )
    } else if content.contains("edition") {
        (
            HealthStatus::Yellow,
            2.0,
            "Older edition configured - consider upgrading to 2021+".to_string(),
        )
    } else {
        (
            HealthStatus::Red,
            0.0,
            "No edition specified - defaults to 2015".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Edition 2021+".to_string(),
        category: "Cargo Config".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_resolver_v2(project_path: &Path) -> DiagnosticCheck {
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let (status, score, message) = if content.contains("resolver = \"2\"") {
        (
            HealthStatus::Green,
            5.0,
            "Resolver v2 explicitly configured".to_string(),
        )
    } else if content.contains("edition = \"2021\"") || content.contains("edition = \"2024\"") {
        // Edition 2021+ implies resolver v2
        (
            HealthStatus::Green,
            5.0,
            "Resolver v2 via edition 2021+".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            2.0,
            "Using legacy resolver - add resolver = \"2\"".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Resolver v2".to_string(),
        category: "Cargo Config".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_dependency_count(project_path: &Path) -> DiagnosticCheck {
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    // Count dependencies
    let mut count = 0;
    let mut in_deps = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[dependencies]"
            || trimmed == "[dev-dependencies]"
            || trimmed == "[build-dependencies]"
        {
            in_deps = true;
            continue;
        }
        if in_deps && trimmed.starts_with('[') {
            in_deps = false;
        }
        if in_deps && !trimmed.starts_with('#') && trimmed.contains('=') {
            count += 1;
        }
    }

    let (status, score, message) = if count <= 20 {
        (
            HealthStatus::Green,
            5.0,
            format!("{} dependencies (excellent)", count),
        )
    } else if count <= 50 {
        (
            HealthStatus::Yellow,
            3.0,
            format!("{} dependencies (consider reducing)", count),
        )
    } else {
        (
            HealthStatus::Red,
            0.0,
            format!("{} dependencies (too many)", count),
        )
    };

    DiagnosticCheck {
        name: "Dependencies <= 50".to_string(),
        category: "Cargo Config".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_lto_enabled(project_path: &Path) -> DiagnosticCheck {
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let has_lto = content.contains("lto = true")
        || content.contains("lto = \"thin\"")
        || content.contains("lto = \"fat\"");

    let (status, score, message) = if has_lto {
        (
            HealthStatus::Green,
            5.0,
            "LTO enabled for release builds".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "LTO not configured - add lto = true to [profile.release]".to_string(),
        )
    };

    DiagnosticCheck {
        name: "LTO Enabled".to_string(),
        category: "Cargo Config".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_workspace_lints(project_path: &Path) -> DiagnosticCheck {
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let has_workspace_lints = content.contains("[workspace.lints");

    let (status, score, message) = if has_workspace_lints {
        (
            HealthStatus::Green,
            5.0,
            "Workspace-level lints configured".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "No workspace lints - add [workspace.lints.rust] section".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Workspace Lints".to_string(),
        category: "Cargo Config".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_workspace_deps(project_path: &Path) -> DiagnosticCheck {
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let has_workspace_deps = content.contains("[workspace.dependencies]");

    let (status, score, message) = if has_workspace_deps {
        (
            HealthStatus::Green,
            5.0,
            "Workspace dependencies configured".to_string(),
        )
    } else if content.contains("[workspace]") {
        (
            HealthStatus::Yellow,
            2.0,
            "Workspace exists but no shared dependencies".to_string(),
        )
    } else {
        (
            HealthStatus::Skip,
            0.0,
            "Single-crate project (N/A)".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Workspace Deps".to_string(),
        category: "Cargo Config".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

// ============================================================================
// Dependencies Checks (3)
// ============================================================================

fn check_target_dir_size(project_path: &Path) -> DiagnosticCheck {
    let target_path = project_path.join("target");

    if !target_path.exists() {
        return DiagnosticCheck {
            name: "Target Dir <= 10GB".to_string(),
            category: "Dependencies".to_string(),
            status: HealthStatus::Green,
            message: "No target directory (clean state)".to_string(),
            score: 5.0,
            max_score: 5.0,
        };
    }

    // Calculate size (best effort)
    let size_bytes = dir_size(&target_path).unwrap_or(0);
    let size_gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    let (status, score, message) = if size_gb <= 2.0 {
        (
            HealthStatus::Green,
            5.0,
            format!("{:.1} GB (excellent)", size_gb),
        )
    } else if size_gb <= 5.0 {
        (
            HealthStatus::Green,
            4.0,
            format!("{:.1} GB (good)", size_gb),
        )
    } else if size_gb <= 10.0 {
        (
            HealthStatus::Yellow,
            2.0,
            format!("{:.1} GB (consider cargo clean)", size_gb),
        )
    } else {
        (
            HealthStatus::Red,
            0.0,
            format!("{:.1} GB (run cargo clean)", size_gb),
        )
    };

    DiagnosticCheck {
        name: "Target Dir <= 10GB".to_string(),
        category: "Dependencies".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_cargo_lock(project_path: &Path) -> DiagnosticCheck {
    let cargo_lock = project_path.join("Cargo.lock");

    let (status, score, message) = if cargo_lock.exists() {
        (
            HealthStatus::Green,
            5.0,
            "Cargo.lock present (reproducible builds)".to_string(),
        )
    } else {
        (
            HealthStatus::Red,
            0.0,
            "No Cargo.lock - run cargo build to generate".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Cargo.lock Present".to_string(),
        category: "Dependencies".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_audit_config(project_path: &Path) -> DiagnosticCheck {
    let deny_toml = project_path.join("deny.toml");
    let audit_toml = project_path.join(".cargo").join("audit.toml");

    let (status, score, message) = if deny_toml.exists() {
        (
            HealthStatus::Green,
            5.0,
            "cargo-deny configured (deny.toml)".to_string(),
        )
    } else if audit_toml.exists() {
        (
            HealthStatus::Green,
            4.0,
            "cargo-audit configured".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "No audit config - add deny.toml for security scanning".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Audit Config".to_string(),
        category: "Dependencies".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

// ============================================================================
// Build Performance Checks (4)
// ============================================================================

fn check_cargo_config(project_path: &Path) -> DiagnosticCheck {
    let config_toml = project_path.join(".cargo").join("config.toml");
    let config_legacy = project_path.join(".cargo").join("config");

    let (status, score, message) = if config_toml.exists() {
        (
            HealthStatus::Green,
            5.0,
            ".cargo/config.toml present".to_string(),
        )
    } else if config_legacy.exists() {
        (
            HealthStatus::Yellow,
            3.0,
            ".cargo/config exists (rename to config.toml)".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "No .cargo/config.toml - consider adding build config".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Cargo Config".to_string(),
        category: "Build Performance".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_incremental_builds(project_path: &Path) -> DiagnosticCheck {
    // Incremental is on by default for dev builds
    let config_toml = project_path.join(".cargo").join("config.toml");
    let cargo_toml = project_path.join("Cargo.toml");

    // Check if explicitly disabled
    let config_content = std::fs::read_to_string(&config_toml).unwrap_or_default();
    let cargo_content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let (status, score, message) = if config_content.contains("incremental = false")
        || cargo_content.contains("incremental = false")
    {
        (
            HealthStatus::Red,
            0.0,
            "Incremental builds disabled".to_string(),
        )
    } else if config_content.contains("incremental = true") {
        (
            HealthStatus::Green,
            5.0,
            "Incremental builds explicitly enabled".to_string(),
        )
    } else {
        (
            HealthStatus::Green,
            4.0,
            "Incremental builds enabled (default)".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Incremental Builds".to_string(),
        category: "Build Performance".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_codegen_units(project_path: &Path) -> DiagnosticCheck {
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let (status, score, message) = if content.contains("codegen-units = 1") {
        (
            HealthStatus::Green,
            5.0,
            "codegen-units = 1 (maximum optimization)".to_string(),
        )
    } else if content.contains("codegen-units") {
        (
            HealthStatus::Yellow,
            3.0,
            "codegen-units configured (not optimal)".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "No codegen-units - add codegen-units = 1 to [profile.release]".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Codegen Units".to_string(),
        category: "Build Performance".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_build_system(project_path: &Path) -> DiagnosticCheck {
    let makefile = project_path.join("Makefile");
    let justfile = project_path.join("justfile");
    let justfile_cap = project_path.join("Justfile");
    let build_rs = project_path.join("build.rs");

    let mut found = Vec::new();
    if makefile.exists() {
        found.push("Makefile");
    }
    if justfile.exists() || justfile_cap.exists() {
        found.push("justfile");
    }
    if build_rs.exists() {
        found.push("build.rs");
    }

    let (status, score, message) = if !found.is_empty() {
        (
            HealthStatus::Green,
            5.0,
            format!("Build automation: {}", found.join(", ")),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "No build automation - add Makefile or justfile".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Build System".to_string(),
        category: "Build Performance".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

// ============================================================================
// Code Quality Checks (4)
// ============================================================================

fn check_clippy_config(project_path: &Path) -> DiagnosticCheck {
    let clippy_toml = project_path.join(".clippy.toml");
    let clippy_toml_alt = project_path.join("clippy.toml");
    let cargo_toml = project_path.join("Cargo.toml");
    let cargo_content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let has_workspace_clippy = cargo_content.contains("[workspace.lints.clippy]")
        || cargo_content.contains("[lints.clippy]");

    let (status, score, message) = if clippy_toml.exists() || clippy_toml_alt.exists() {
        (
            HealthStatus::Green,
            5.0,
            ".clippy.toml configured".to_string(),
        )
    } else if has_workspace_clippy {
        (
            HealthStatus::Green,
            5.0,
            "Clippy lints in Cargo.toml".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "No Clippy config - add [lints.clippy] or .clippy.toml".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Clippy Config".to_string(),
        category: "Code Quality".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_rustfmt_config(project_path: &Path) -> DiagnosticCheck {
    let rustfmt_toml = project_path.join("rustfmt.toml");
    let rustfmt_toml_alt = project_path.join(".rustfmt.toml");

    let (status, score, message) = if rustfmt_toml.exists() || rustfmt_toml_alt.exists() {
        (HealthStatus::Green, 5.0, "rustfmt configured".to_string())
    } else {
        (
            HealthStatus::Yellow,
            2.0,
            "No rustfmt.toml - using defaults".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Rustfmt Config".to_string(),
        category: "Code Quality".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_tests_present(project_path: &Path) -> DiagnosticCheck {
    let tests_dir = project_path.join("tests");
    let src_dir = project_path.join("src");

    // Check for tests/ directory
    let has_integration_tests = tests_dir.exists() && tests_dir.is_dir();

    // Check for #[test] or #[cfg(test)] in src/
    let has_unit_tests = if src_dir.exists() {
        has_test_annotations(&src_dir)
    } else {
        false
    };

    let (status, score, message) = if has_integration_tests && has_unit_tests {
        (
            HealthStatus::Green,
            5.0,
            "Both unit and integration tests present".to_string(),
        )
    } else if has_integration_tests || has_unit_tests {
        (HealthStatus::Yellow, 3.0, "Some tests present".to_string())
    } else {
        (HealthStatus::Red, 0.0, "No tests found".to_string())
    };

    DiagnosticCheck {
        name: "Tests Present".to_string(),
        category: "Code Quality".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_readme(project_path: &Path) -> DiagnosticCheck {
    let readme = project_path.join("README.md");
    let readme_alt = project_path.join("README");

    let (status, score, message) = if readme.exists() {
        let content = std::fs::read_to_string(&readme).unwrap_or_default();
        if content.len() > 500 {
            (
                HealthStatus::Green,
                5.0,
                "Comprehensive README.md present".to_string(),
            )
        } else {
            (
                HealthStatus::Yellow,
                3.0,
                "README.md exists but is short".to_string(),
            )
        }
    } else if readme_alt.exists() {
        (
            HealthStatus::Yellow,
            2.0,
            "README exists (consider README.md)".to_string(),
        )
    } else {
        (
            HealthStatus::Red,
            0.0,
            "No README - add README.md".to_string(),
        )
    };

    DiagnosticCheck {
        name: "README".to_string(),
        category: "Code Quality".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

// ============================================================================
// Advanced Checks (3)
// ============================================================================

fn check_msrv_defined(project_path: &Path) -> DiagnosticCheck {
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let (status, score, message) = if content.contains("rust-version") {
        (
            HealthStatus::Green,
            5.0,
            "MSRV defined (rust-version field)".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "No MSRV - add rust-version to Cargo.toml".to_string(),
        )
    };

    DiagnosticCheck {
        name: "MSRV Defined".to_string(),
        category: "Advanced".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_benchmarks(project_path: &Path) -> DiagnosticCheck {
    let benches_dir = project_path.join("benches");
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let has_bench_dir = benches_dir.exists() && benches_dir.is_dir();
    let has_criterion = content.contains("criterion");

    let (status, score, message) = if has_bench_dir && has_criterion {
        (
            HealthStatus::Green,
            5.0,
            "Criterion benchmarks configured".to_string(),
        )
    } else if has_bench_dir {
        (
            HealthStatus::Yellow,
            3.0,
            "Benchmarks present (consider Criterion)".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "No benchmarks - add benches/ directory".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Benchmarks".to_string(),
        category: "Advanced".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_ci_configured(project_path: &Path) -> DiagnosticCheck {
    let github_workflows = project_path.join(".github").join("workflows");
    let gitlab_ci = project_path.join(".gitlab-ci.yml");
    let jenkinsfile = project_path.join("Jenkinsfile");

    let (status, score, message) = if github_workflows.exists() && github_workflows.is_dir() {
        let workflow_count = std::fs::read_dir(&github_workflows)
            .map(|entries| entries.filter_map(|e| e.ok()).count())
            .unwrap_or(0);
        if workflow_count >= 3 {
            (
                HealthStatus::Green,
                5.0,
                format!("{} GitHub Actions workflows", workflow_count),
            )
        } else if workflow_count > 0 {
            (
                HealthStatus::Yellow,
                3.0,
                format!("{} GitHub Actions workflow(s)", workflow_count),
            )
        } else {
            (
                HealthStatus::Yellow,
                1.0,
                "Empty .github/workflows directory".to_string(),
            )
        }
    } else if gitlab_ci.exists() {
        (HealthStatus::Green, 5.0, "GitLab CI configured".to_string())
    } else if jenkinsfile.exists() {
        (
            HealthStatus::Green,
            5.0,
            "Jenkins pipeline configured".to_string(),
        )
    } else {
        (
            HealthStatus::Red,
            0.0,
            "No CI configured - add .github/workflows/".to_string(),
        )
    };

    DiagnosticCheck {
        name: "CI Configured".to_string(),
        category: "Advanced".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn dir_size(path: &Path) -> std::io::Result<u64> {
    let mut total = 0u64;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                total += dir_size(&path).unwrap_or(0);
            } else {
                total += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    Ok(total)
}

fn has_test_annotations(dir: &Path) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "rs").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains("#[test]") || content.contains("#[cfg(test)]") {
                        return true;
                    }
                }
            } else if path.is_dir() && has_test_annotations(&path) {
                return true;
            }
        }
    }
    false
}

// ============================================================================
// Output Formatters
// ============================================================================

fn format_summary(report: &DiagnosticReport, failures_only: bool) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "\n  Project Diagnostics: {}\n",
        report.project_path
    ));
    output.push_str(&format!("  {}\n\n", "=".repeat(60)));

    // Overall score
    let status_icon = match report.overall_status {
        HealthStatus::Green => "[GREEN]",
        HealthStatus::Yellow => "[YELLOW]",
        HealthStatus::Red => "[RED]",
        HealthStatus::Skip => "[SKIP]",
    };
    output.push_str(&format!(
        "  Overall: {} {:.1}/{:.1} ({:.1}%)\n\n",
        status_icon, report.total_score, report.max_score, report.percentage
    ));

    // Category summaries
    for cat in &report.categories {
        output.push_str(&format!("  {} [{}/{}]\n", cat.name, cat.passed, cat.total));
    }
    output.push('\n');

    // Individual checks
    output.push_str("  Checks:\n");
    output.push_str(&format!("  {}\n", "-".repeat(60)));

    for check in &report.checks {
        if failures_only && check.status == HealthStatus::Green {
            continue;
        }

        let icon = match check.status {
            HealthStatus::Green => "[OK]",
            HealthStatus::Yellow => "[WARN]",
            HealthStatus::Red => "[FAIL]",
            HealthStatus::Skip => "[SKIP]",
        };

        output.push_str(&format!("  {} {} - {}\n", icon, check.name, check.message));
    }

    output.push('\n');
    output
}

fn format_json(report: &DiagnosticReport) -> Result<String> {
    serde_json::to_string_pretty(report).map_err(|e| anyhow::anyhow!(e))
}

fn format_markdown(report: &DiagnosticReport, failures_only: bool) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "# Project Diagnostics: {}\n\n",
        report.project_path
    ));

    // Overall status
    let badge = match report.overall_status {
        HealthStatus::Green => "![Status](https://img.shields.io/badge/status-healthy-green)",
        HealthStatus::Yellow => "![Status](https://img.shields.io/badge/status-warning-yellow)",
        HealthStatus::Red => "![Status](https://img.shields.io/badge/status-critical-red)",
        HealthStatus::Skip => "![Status](https://img.shields.io/badge/status-skipped-gray)",
    };
    output.push_str(&format!("{}\n\n", badge));
    output.push_str(&format!(
        "**Score:** {:.1}/{:.1} ({:.1}%)\n\n",
        report.total_score, report.max_score, report.percentage
    ));

    // Category table
    output.push_str("## Categories\n\n");
    output.push_str("| Category | Passed | Warned | Failed | Score |\n");
    output.push_str("|----------|--------|--------|--------|-------|\n");
    for cat in &report.categories {
        output.push_str(&format!(
            "| {} | {} | {} | {} | {:.1}/{:.1} |\n",
            cat.name, cat.passed, cat.warned, cat.failed, cat.score, cat.max_score
        ));
    }
    output.push('\n');

    // Checks table
    output.push_str("## Checks\n\n");
    output.push_str("| Status | Check | Message |\n");
    output.push_str("|--------|-------|--------|\n");
    for check in &report.checks {
        if failures_only && check.status == HealthStatus::Green {
            continue;
        }
        let emoji = match check.status {
            HealthStatus::Green => "✅",
            HealthStatus::Yellow => "⚠️",
            HealthStatus::Red => "❌",
            HealthStatus::Skip => "⏭️",
        };
        output.push_str(&format!(
            "| {} | {} | {} |\n",
            emoji, check.name, check.message
        ));
    }

    output
}

fn format_andon(report: &DiagnosticReport) -> String {
    let mut output = String::new();

    // Andon-style visualization (Toyota Way)
    output.push('\n');
    output.push_str("  ╔══════════════════════════════════════════════════════════════╗\n");
    output.push_str("  ║                    PROJECT DIAGNOSTICS                       ║\n");
    output.push_str("  ║                      (Andon Board)                           ║\n");
    output.push_str("  ╠══════════════════════════════════════════════════════════════╣\n");

    // Score display
    let bar_width = 40;
    let filled = ((report.percentage / 100.0) * bar_width as f64) as usize;
    let empty = bar_width - filled;
    let progress_bar = format!("{}{}", "#".repeat(filled), "-".repeat(empty));

    output.push_str(&format!(
        "  ║  Score: [{progress_bar}] {:.1}%  ║\n",
        report.percentage
    ));
    output.push_str("  ╠══════════════════════════════════════════════════════════════╣\n");

    // Category lights
    for cat in &report.categories {
        let light = if cat.failed > 0 {
            "[RED]  "
        } else if cat.warned > 0 {
            "[YELLOW]"
        } else {
            "[GREEN]"
        };
        output.push_str(&format!(
            "  ║  {} {:20} {}/{} checks passed          ║\n",
            light, cat.name, cat.passed, cat.total
        ));
    }

    output.push_str("  ╠══════════════════════════════════════════════════════════════╣\n");

    // Failed checks (Andon cord triggers)
    let failures: Vec<_> = report
        .checks
        .iter()
        .filter(|c| c.status == HealthStatus::Red)
        .collect();

    if failures.is_empty() {
        output.push_str("  ║  No critical issues - production ready                       ║\n");
    } else {
        output.push_str("  ║  ANDON CORD TRIGGERED - Issues require attention:            ║\n");
        for check in failures.iter().take(5) {
            output.push_str(&format!("  ║    - {:<54} ║\n", check.name));
        }
    }

    output.push_str("  ╚══════════════════════════════════════════════════════════════╝\n");
    output.push('\n');

    output
}

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
}
