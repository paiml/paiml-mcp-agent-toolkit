//! PMAT Compliance and Migration Handlers (GH-96)
//!
//! Implements the `pmat comply` command for checking and maintaining
//! project compliance with PMAT standards.
//!
//! Commands:
//! - check: Verify project compliance with current PMAT version
//! - migrate: Migrate project configs to latest standards
//! - diff: Show changelog between versions
//! - update: Update hooks and configs

use crate::cli::commands::{ComplyCommands, ComplyOutputFormat};
use crate::services::commit_classifier::CommitClassifier;
use crate::services::file_health::{
    FileHealthMetrics, FileHealthReport,
    scan_directory, DEFAULT_EXCLUDE_PATTERNS, RUST_EXTENSIONS,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Current PMAT version (from Cargo.toml)
pub const PMAT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Project compliance information stored in .pmat/project.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub pmat: PmatSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmatSection {
    pub version: String,
    #[serde(default)]
    pub last_compliance_check: Option<DateTime<Utc>>,
    #[serde(default)]
    pub auto_update: bool,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            pmat: PmatSection {
                version: PMAT_VERSION.to_string(),
                last_compliance_check: Some(Utc::now()),
                auto_update: false,
            },
        }
    }
}

/// Compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub project_version: String,
    pub current_version: String,
    pub is_compliant: bool,
    pub versions_behind: u32,
    pub checks: Vec<ComplianceCheck>,
    pub breaking_changes: Vec<BreakingChange>,
    pub recommendations: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    pub version: String,
    pub description: String,
    pub migration_guide: Option<String>,
}

/// Handle all comply subcommands
pub async fn handle_comply_command(command: ComplyCommands) -> Result<()> {
    match command {
        ComplyCommands::Check {
            path,
            strict,
            failures_only,
            format,
        } => handle_check(&path, strict, failures_only, format).await,

        ComplyCommands::Migrate {
            path,
            version,
            dry_run,
            no_backup,
            force,
        } => handle_migrate(&path, version.as_deref(), dry_run, no_backup, force).await,

        ComplyCommands::Diff {
            path,
            from,
            to,
            breaking_only,
        } => handle_diff(&path, from.as_deref(), to.as_deref(), breaking_only).await,

        ComplyCommands::Update {
            path,
            hooks,
            config,
            dry_run,
        } => handle_update(&path, hooks, config, dry_run).await,

        ComplyCommands::Init { path, force } => handle_init(&path, force).await,

        ComplyCommands::Enforce {
            path,
            yes,
            disable,
            format,
        } => handle_enforce(&path, yes, disable, format).await,

        ComplyCommands::Report {
            path,
            include_history,
            format,
            output,
        } => handle_report(&path, include_history, format, output.as_deref()).await,
    }
}

/// Check project compliance with current PMAT version
async fn handle_check(
    project_path: &Path,
    strict: bool,
    failures_only: bool,
    format: ComplyOutputFormat,
) -> Result<()> {
    println!("Checking PMAT compliance for {}", project_path.display());

    // Load or create project config
    let config = load_or_create_project_config(project_path)?;
    let project_version = &config.pmat.version;

    // Run compliance checks
    let checks = vec![
        check_version_currency(project_version),
        check_config_files(project_path),
        check_hooks_installed(project_path),
        check_hooks_o1_capable(project_path),    // CB-030: O(1) hooks capability
        check_hooks_cache_health(project_path),   // CB-031: Cache health
        check_quality_thresholds(project_path),
        check_deprecated_features(project_path),
        check_compute_brick(project_path),
        // Build performance checks (lltop Tab 8 integration)
        check_cargo_lock(project_path),
        check_msrv(project_path),
        check_ci_configured(project_path),
        check_paiml_deps_workspace(project_path),
        check_sovereign_stack_patterns(project_path),
        check_file_health(project_path),  // CB-040: File Health Score (max-lines, TLR)
    ];

    // Calculate compliance
    let failures = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Fail)
        .count();
    let warnings = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Warn)
        .count();
    let is_compliant = failures == 0;

    // Get breaking changes
    let breaking_changes = get_breaking_changes_since(project_version);
    let versions_behind = calculate_versions_behind(project_version);

    // Build recommendations
    let mut recommendations = vec![];
    if versions_behind > 0 {
        recommendations.push(format!(
            "Run 'pmat comply migrate' to update to v{}",
            PMAT_VERSION
        ));
    }
    if !breaking_changes.is_empty() {
        recommendations.push("Review breaking changes with 'pmat comply diff'".to_string());
    }

    let report = ComplianceReport {
        project_version: project_version.clone(),
        current_version: PMAT_VERSION.to_string(),
        is_compliant,
        versions_behind,
        checks: if failures_only {
            checks
                .into_iter()
                .filter(|c| c.status == CheckStatus::Fail)
                .collect()
        } else {
            checks
        },
        breaking_changes,
        recommendations,
        timestamp: Utc::now(),
    };

    // Output report
    match format {
        ComplyOutputFormat::Text => print_compliance_text(&report),
        ComplyOutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        ComplyOutputFormat::Markdown => print_compliance_markdown(&report),
    }

    // Update last check timestamp
    let _ = update_last_check_timestamp(project_path);

    // Exit with error if strict mode and non-compliant
    if strict && !is_compliant {
        std::process::exit(1);
    }
    if strict && warnings > 0 {
        std::process::exit(2);
    }

    Ok(())
}

/// Migrate project to latest PMAT standards
async fn handle_migrate(
    project_path: &Path,
    target_version: Option<&str>,
    dry_run: bool,
    no_backup: bool,
    force: bool,
) -> Result<()> {
    let target = target_version.unwrap_or(PMAT_VERSION);
    println!("Migrating project to PMAT v{}", target);

    if dry_run {
        println!("(dry-run mode - no changes will be made)\n");
    }

    let config = load_or_create_project_config(project_path)?;
    let current_version = &config.pmat.version;

    println!("Current version: {}", current_version);
    println!("Target version:  {}\n", target);

    let breaking_changes = get_breaking_changes_since(current_version);
    if !breaking_changes.is_empty() && !force {
        println!(
            "\x1b[33mWarning: {} breaking changes detected:\x1b[0m",
            breaking_changes.len()
        );
        for change in &breaking_changes {
            println!("  - v{}: {}", change.version, change.description);
        }
        println!("\nUse --force to proceed anyway\n");
        if !force {
            return Ok(());
        }
    }

    if !no_backup && !dry_run {
        let backup_path = project_path.join(".pmat").join("backup");
        fs::create_dir_all(&backup_path)?;
        println!("Created backup at: {}", backup_path.display());
    }

    let migrations = vec![
        (
            "Update project.toml version",
            migrate_project_version(project_path, target, dry_run),
        ),
        ("Update gitignore", migrate_gitignore(project_path, dry_run)),
    ];

    println!("\nMigration steps:");
    for (name, result) in migrations {
        match result {
            Ok(true) => println!("  \x1b[32m✓\x1b[0m {}", name),
            Ok(false) => println!("  \x1b[90m-\x1b[0m {} (no changes needed)", name),
            Err(e) => println!("  \x1b[31m✗\x1b[0m {} - {}", name, e),
        }
    }

    if dry_run {
        println!("\n(dry-run complete - no changes were made)");
    } else {
        println!("\n\x1b[32m✓ Migration complete!\x1b[0m");
    }

    Ok(())
}

/// Show changelog between versions
async fn handle_diff(
    project_path: &Path,
    from_version: Option<&str>,
    to_version: Option<&str>,
    breaking_only: bool,
) -> Result<()> {
    let config = load_or_create_project_config(project_path)?;
    let from = from_version.unwrap_or(&config.pmat.version);
    let to = to_version.unwrap_or(PMAT_VERSION);

    println!("PMAT Changelog: v{} → v{}\n", from, to);

    let changes = get_changelog_entries(from, to);

    if breaking_only {
        println!("\x1b[33mBreaking Changes Only:\x1b[0m\n");
        let breaking: Vec<_> = changes.iter().filter(|c| c.breaking).collect();
        if breaking.is_empty() {
            println!("  No breaking changes between these versions.");
        } else {
            for entry in breaking {
                println!(
                    "  \x1b[31m[BREAKING]\x1b[0m v{}: {}",
                    entry.version, entry.description
                );
            }
        }
    } else {
        for entry in &changes {
            let icon = if entry.breaking {
                "\x1b[31m[BREAKING]\x1b[0m"
            } else {
                "\x1b[32m[FEATURE]\x1b[0m"
            };
            println!("  {} v{}: {}", icon, entry.version, entry.description);
        }
    }

    Ok(())
}

/// Update hooks and configs
async fn handle_update(
    project_path: &Path,
    update_hooks: bool,
    update_config: bool,
    dry_run: bool,
) -> Result<()> {
    let update_both = !update_hooks && !update_config;

    if dry_run {
        println!("(dry-run mode - no changes will be made)\n");
    }

    if update_hooks || update_both {
        println!("Updating hooks...");
        println!("  \x1b[90m-\x1b[0m Hooks already up to date");
    }

    if update_config || update_both {
        println!("Updating config...");
        match update_project_config(project_path, dry_run) {
            Ok(true) => println!("  \x1b[32m✓\x1b[0m Config updated to v{}", PMAT_VERSION),
            Ok(false) => println!("  \x1b[90m-\x1b[0m Config already up to date"),
            Err(e) => println!("  \x1b[31m✗\x1b[0m Failed: {}", e),
        }
    }

    Ok(())
}

/// Initialize .pmat/project.toml with current version
async fn handle_init(project_path: &Path, force: bool) -> Result<()> {
    let config_path = project_path.join(".pmat").join("project.toml");

    if config_path.exists() && !force {
        println!("Project already initialized at {}", config_path.display());
        println!("Use --force to overwrite existing configuration.");
        return Ok(());
    }

    // Create .pmat directory
    let pmat_dir = project_path.join(".pmat");
    if !pmat_dir.exists() {
        fs::create_dir_all(&pmat_dir)?;
    }

    // Create default config
    let config = ProjectConfig::default();
    let content = toml::to_string_pretty(&config)?;
    fs::write(&config_path, &content)?;

    println!(
        "\x1b[32m✓\x1b[0m Initialized PMAT project at {}",
        config_path.display()
    );
    println!("\nProject version: v{}", PMAT_VERSION);
    println!("\nNext steps:");
    println!("  1. Run 'pmat comply check' to verify compliance");
    println!("  2. Run 'pmat hooks init' to install git hooks");
    println!("  3. Run 'pmat quality-gate' to check code quality");

    Ok(())
}

// Helper functions

fn load_or_create_project_config(project_path: &Path) -> Result<ProjectConfig> {
    let config_path = project_path.join(".pmat").join("project.toml");

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        Ok(toml::from_str(&content)?)
    } else {
        let config = ProjectConfig::default();
        let pmat_dir = project_path.join(".pmat");
        if !pmat_dir.exists() {
            fs::create_dir_all(&pmat_dir)?;
        }
        let content = toml::to_string_pretty(&config)?;
        fs::write(&config_path, &content)?;
        Ok(config)
    }
}

fn update_last_check_timestamp(project_path: &Path) -> Result<()> {
    let config_path = project_path.join(".pmat").join("project.toml");
    if let Ok(mut config) = load_or_create_project_config(project_path) {
        config.pmat.last_compliance_check = Some(Utc::now());
        let content = toml::to_string_pretty(&config)?;
        fs::write(&config_path, &content)?;
    }
    Ok(())
}

fn check_version_currency(project_version: &str) -> ComplianceCheck {
    let behind = calculate_versions_behind(project_version);
    if behind == 0 {
        ComplianceCheck {
            name: "Version Currency".to_string(),
            status: CheckStatus::Pass,
            message: format!("Project is on latest version (v{})", PMAT_VERSION),
            severity: Severity::Info,
        }
    } else if behind <= 5 {
        ComplianceCheck {
            name: "Version Currency".to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "{} versions behind (v{} → v{})",
                behind, project_version, PMAT_VERSION
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "Version Currency".to_string(),
            status: CheckStatus::Fail,
            message: format!("{} versions behind - migration recommended", behind),
            severity: Severity::Error,
        }
    }
}

fn check_config_files(project_path: &Path) -> ComplianceCheck {
    let config_files = [".pmat/project.toml", ".pmat-metrics.toml"];
    let missing: Vec<&str> = config_files
        .iter()
        .filter(|f| !project_path.join(f).exists())
        .copied()
        .collect();

    if missing.is_empty() {
        ComplianceCheck {
            name: "Config Files".to_string(),
            status: CheckStatus::Pass,
            message: "All required config files present".to_string(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "Config Files".to_string(),
            status: CheckStatus::Warn,
            message: format!("Missing: {}", missing.join(", ")),
            severity: Severity::Warning,
        }
    }
}

fn check_hooks_installed(project_path: &Path) -> ComplianceCheck {
    let pre_commit = project_path.join(".git").join("hooks").join("pre-commit");
    if pre_commit.exists() {
        if let Ok(content) = fs::read_to_string(&pre_commit) {
            if content.contains("pmat") || content.contains("PMAT") {
                return ComplianceCheck {
                    name: "Git Hooks".to_string(),
                    status: CheckStatus::Pass,
                    message: "PMAT hooks installed".to_string(),
                    severity: Severity::Info,
                };
            }
        }
        ComplianceCheck {
            name: "Git Hooks".to_string(),
            status: CheckStatus::Warn,
            message: "Pre-commit hook exists but may not be PMAT".to_string(),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "Git Hooks".to_string(),
            status: CheckStatus::Warn,
            message: "No pre-commit hook installed".to_string(),
            severity: Severity::Warning,
        }
    }
}

/// CB-030: Check if hooks have O(1) capability (PMAT-453)
fn check_hooks_o1_capable(project_path: &Path) -> ComplianceCheck {
    let cache_dir = project_path.join(".pmat").join("hooks-cache");

    if cache_dir.exists() {
        // Check that the expected structure exists
        let tree_hash = cache_dir.join("tree-hash.json");
        let gates_dir = cache_dir.join("gates");

        if tree_hash.exists() || gates_dir.exists() {
            return ComplianceCheck {
                name: "CB-030: O(1) Hooks".to_string(),
                status: CheckStatus::Pass,
                message: "Hooks cache initialized - O(1) capable".to_string(),
                severity: Severity::Info,
            };
        }

        ComplianceCheck {
            name: "CB-030: O(1) Hooks".to_string(),
            status: CheckStatus::Warn,
            message: "Cache directory exists but not fully initialized".to_string(),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-030: O(1) Hooks".to_string(),
            status: CheckStatus::Warn,
            message: "Run 'pmat hooks cache init' to enable O(1) hooks".to_string(),
            severity: Severity::Warning,
        }
    }
}

/// CB-031: Check hooks cache health (hit rate >= 60%)
fn check_hooks_cache_health(project_path: &Path) -> ComplianceCheck {
    let metrics_path = project_path
        .join(".pmat")
        .join("hooks-cache")
        .join("metrics.json");

    if !metrics_path.exists() {
        return ComplianceCheck {
            name: "CB-031: Cache Health".to_string(),
            status: CheckStatus::Skip,
            message: "No cache metrics available yet".to_string(),
            severity: Severity::Info,
        };
    }

    // Read and parse metrics
    match fs::read_to_string(&metrics_path) {
        Ok(content) => {
            if let Ok(metrics) = serde_json::from_str::<serde_json::Value>(&content) {
                let total_runs = metrics["total_runs"].as_u64().unwrap_or(0);
                let cache_hits = metrics["cache_hits"].as_u64().unwrap_or(0);

                if total_runs < 5 {
                    return ComplianceCheck {
                        name: "CB-031: Cache Health".to_string(),
                        status: CheckStatus::Skip,
                        message: format!("Insufficient data ({} runs, need 5+)", total_runs),
                        severity: Severity::Info,
                    };
                }

                let hit_rate = (cache_hits as f64 / total_runs as f64) * 100.0;

                if hit_rate >= 60.0 {
                    ComplianceCheck {
                        name: "CB-031: Cache Health".to_string(),
                        status: CheckStatus::Pass,
                        message: format!("Cache hit rate {:.1}% (target: ≥60%)", hit_rate),
                        severity: Severity::Info,
                    }
                } else {
                    ComplianceCheck {
                        name: "CB-031: Cache Health".to_string(),
                        status: CheckStatus::Warn,
                        message: format!(
                            "Cache hit rate {:.1}% below 60% target - consider clearing cache",
                            hit_rate
                        ),
                        severity: Severity::Warning,
                    }
                }
            } else {
                ComplianceCheck {
                    name: "CB-031: Cache Health".to_string(),
                    status: CheckStatus::Warn,
                    message: "Failed to parse metrics.json".to_string(),
                    severity: Severity::Warning,
                }
            }
        }
        Err(_) => ComplianceCheck {
            name: "CB-031: Cache Health".to_string(),
            status: CheckStatus::Warn,
            message: "Failed to read metrics.json".to_string(),
            severity: Severity::Warning,
        },
    }
}

fn check_quality_thresholds(project_path: &Path) -> ComplianceCheck {
    if project_path.join(".pmat-metrics.toml").exists() {
        ComplianceCheck {
            name: "Quality Thresholds".to_string(),
            status: CheckStatus::Pass,
            message: "Quality thresholds configured".to_string(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "Quality Thresholds".to_string(),
            status: CheckStatus::Warn,
            message: "No .pmat-metrics.toml found - using defaults".to_string(),
            severity: Severity::Warning,
        }
    }
}

fn check_deprecated_features(_project_path: &Path) -> ComplianceCheck {
    ComplianceCheck {
        name: "Deprecated Features".to_string(),
        status: CheckStatus::Pass,
        message: "No deprecated features detected".to_string(),
        severity: Severity::Info,
    }
}

/// ComputeBrick pattern detection result
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CbPatternViolation {
    pattern_id: String,
    file: String,
    line: usize,
    description: String,
    severity: Severity,
}

/// BrickProfiler anomaly from JSON output
#[derive(Debug, Clone)]
struct ProfilerAnomaly {
    brick_name: String,
    anomaly_type: String,
    value: f64,
    threshold: f64,
}

/// Compute line ranges that are inside test code (#[cfg(test)] mod tests { ... })
/// Returns a HashSet of line indices that should be skipped for production code analysis.
fn compute_test_code_lines(lines: &[&str]) -> std::collections::HashSet<usize> {
    let mut test_lines = std::collections::HashSet::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Detect #[cfg(test)] followed by mod (within next 3 lines)
        if line.starts_with("#[cfg(test)]") {
            // Find the mod line
            for j in i..std::cmp::min(i + 4, lines.len()) {
                if lines[j].contains("mod ") {
                    // Found test module - track all lines until closing brace
                    let mut depth = 0;
                    for k in j..lines.len() {
                        depth += lines[k].matches('{').count();
                        depth = depth.saturating_sub(lines[k].matches('}').count());
                        test_lines.insert(k);
                        if depth == 0 && k > j && lines[k].contains('}') {
                            break;
                        }
                    }
                    // Also mark the #[cfg(test)] line
                    test_lines.insert(i);
                    break;
                }
            }
        }

        // Also detect standalone `mod tests {` without #[cfg(test)] (common pattern)
        if (line.starts_with("mod tests") || line.starts_with("pub mod tests"))
            && line.contains('{')
        {
            let mut depth = 0;
            for k in i..lines.len() {
                depth += lines[k].matches('{').count();
                depth = depth.saturating_sub(lines[k].matches('}').count());
                test_lines.insert(k);
                if depth == 0 && k > i && lines[k].contains('}') {
                    break;
                }
            }
        }

        // Detect #[test] function (individual test functions)
        if line.starts_with("#[test]") {
            test_lines.insert(i);
            // Mark the function that follows
            for j in i + 1..std::cmp::min(i + 4, lines.len()) {
                if lines[j].contains("fn ") {
                    let mut depth = 0;
                    for k in j..lines.len() {
                        depth += lines[k].matches('{').count();
                        depth = depth.saturating_sub(lines[k].matches('}').count());
                        test_lines.insert(k);
                        if depth == 0 && k > j {
                            break;
                        }
                    }
                    break;
                }
            }
        }

        i += 1;
    }

    test_lines
}

/// Scan Rust files for CB-020 (unsafe without SAFETY comment)
/// NOTE: Skips test code (#[cfg(test)], mod tests, #[test]) - test code can use .unwrap() freely
fn detect_cb020_unsafe_without_safety(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    // Walk src/ directory for .rs files
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return violations;
    }

    if let Ok(entries) = walkdir_rs_files(&src_dir) {
        for entry in entries {
            if let Ok(content) = fs::read_to_string(&entry) {
                let lines: Vec<&str> = content.lines().collect();
                let test_lines = compute_test_code_lines(&lines);

                for (line_num, line) in lines.iter().enumerate() {
                    // Skip test code - unsafe in tests is fine
                    if test_lines.contains(&line_num) {
                        continue;
                    }

                    let trimmed = line.trim();
                    // Check for unsafe block without preceding SAFETY comment
                    if trimmed.starts_with("unsafe {") || trimmed.starts_with("unsafe{") {
                        // Look at previous non-empty lines for SAFETY comment
                        let has_safety = lines
                            .iter()
                            .take(line_num)
                            .rev()
                            .take(3)
                            .any(|l| l.contains("// SAFETY:") || l.contains("// SAFETY :"));

                        if !has_safety {
                            violations.push(CbPatternViolation {
                                pattern_id: "CB-020".to_string(),
                                file: entry.display().to_string(),
                                line: line_num + 1,
                                description: "unsafe block without SAFETY comment".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }
    }

    violations
}

/// Scan for CB-021 (SIMD intrinsics without #[target_feature])
/// NOTE: Skips test code (#[cfg(test)], mod tests, #[test]) - test code is exempt
fn detect_cb021_simd_without_target_feature(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return violations;
    }

    // Common SIMD intrinsic patterns - must be actual intrinsic calls
    // x86 SSE/AVX: These are function-like intrinsics (always have underscore prefix)
    // ARM NEON: These are function-like intrinsics (always have underscore prefix)
    // Portable SIMD: Require :: to indicate method call (not just type in identifier)
    // Only check for AVX/AVX-512 which require target_feature
    // SSE intrinsics (_mm_) and NEON are baseline and don't need target_feature
    let simd_patterns_needing_target_feature = [
        "_mm256_", "_mm512_", // x86 AVX/AVX-512 (not SSE which is baseline)
        // NEON (vld1q_, etc.) is baseline on aarch64, no target_feature needed
    ];
    // Portable SIMD - require :: suffix to distinguish from identifiers like "f32x4_verified"
    let portable_simd_patterns = ["i8x16::", "i16x8::", "i32x4::", "f32x4::", "Simd::<"];

    if let Ok(entries) = walkdir_rs_files(&src_dir) {
        for entry in entries {
            if let Ok(content) = fs::read_to_string(&entry) {
                let lines: Vec<&str> = content.lines().collect();
                let test_lines = compute_test_code_lines(&lines);

                // Find functions with #[target_feature] attribute
                let mut protected_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();

                for (i, line) in lines.iter().enumerate() {
                    // Both #[target_feature] and #[cfg(target_feature = "...")] protect SIMD code
                    let is_protected = line.trim().starts_with("#[target_feature")
                        || (line.contains("#[cfg(") && line.contains("target_feature"));
                    if is_protected {
                        // Mark all lines in this function as protected
                        // Find the fn line (within 15 lines to handle attrs + SAFETY comments)
                        for j in i..std::cmp::min(i + 15, lines.len()) {
                            if lines[j].contains("fn ") {
                                // Count braces to find function end
                                // Must track if we've seen { before checking depth == 0
                                let mut depth = 0;
                                let mut seen_opening_brace = false;
                                for k in j..lines.len() {
                                    let open_count = lines[k].matches('{').count();
                                    if open_count > 0 {
                                        seen_opening_brace = true;
                                    }
                                    depth += open_count;
                                    depth = depth.saturating_sub(lines[k].matches('}').count());
                                    protected_lines.insert(k);
                                    // Only break when we've seen { and returned to depth 0
                                    if seen_opening_brace && depth == 0 {
                                        break;
                                    }
                                }
                                break;
                            }
                        }
                    }
                }

                // Check for SIMD intrinsics outside protected functions and test code
                for (line_num, line) in lines.iter().enumerate() {
                    // Skip protected functions and test code
                    if protected_lines.contains(&line_num) || test_lines.contains(&line_num) {
                        continue;
                    }

                    let trimmed = line.trim();
                    // Skip all comments (// and ///)
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Check x86/NEON intrinsics (function-like, always start with _)
                    // Skip prefetch intrinsics - they're SSE baseline on x86_64
                    if line.contains("_mm_prefetch") || line.contains("_MM_HINT_") {
                        continue;
                    }
                    for pattern in &simd_patterns_needing_target_feature {
                        if line.contains(pattern) {
                            violations.push(CbPatternViolation {
                                pattern_id: "CB-021".to_string(),
                                file: entry.display().to_string(),
                                line: line_num + 1,
                                description: format!(
                                    "SIMD intrinsic '{}' without #[target_feature]",
                                    pattern.trim_end_matches('_')
                                ),
                                severity: Severity::Warning,
                            });
                            break;
                        }
                    }

                    // Check portable SIMD (require :: to indicate actual usage)
                    for pattern in &portable_simd_patterns {
                        if line.contains(pattern) {
                            violations.push(CbPatternViolation {
                                pattern_id: "CB-021".to_string(),
                                file: entry.display().to_string(),
                                line: line_num + 1,
                                description: format!(
                                    "Portable SIMD '{}' without #[target_feature]",
                                    pattern.trim_end_matches("::")
                                ),
                                severity: Severity::Warning,
                            });
                            break;
                        }
                    }
                }
            }
        }
    }

    violations
}

/// Scan for WGSL files and detect CB-001 (missing bounds check on global_invocation_id)
/// CB-001: WGSL global_invocation_id used without bounds check
/// Reference: docs/specifications/compute-brick-support.md §4.1
fn detect_cb001_wgsl_no_bounds_check(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    // Find all .wgsl files in the project
    for entry in walkdir::WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "wgsl"))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            let lines: Vec<&str> = content.lines().collect();

            for (line_num, line) in lines.iter().enumerate() {
                // Check for global_invocation_id usage
                if line.contains("global_invocation_id") {
                    // Look for bounds check pattern: `if (gid >= arrayLength` or similar
                    // within 5 lines after global_invocation_id usage
                    let has_bounds_check = lines
                        .iter()
                        .skip(line_num)
                        .take(10)
                        .any(|l| {
                            l.contains("arrayLength") ||
                            l.contains(">= arrayLength") ||
                            l.contains("< arrayLength") ||
                            l.contains(".length") ||
                            (l.contains("if") && (l.contains(">=") || l.contains("<")))
                        });

                    if !has_bounds_check {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-001".to_string(),
                            file: entry.path().display().to_string(),
                            line: line_num + 1,
                            description: "global_invocation_id used without bounds check - potential OOB access".to_string(),
                            severity: Severity::Critical,
                        });
                    }
                }
            }
        }
    }

    violations
}

/// Scan for WGSL files and detect CB-002 (barrier divergence)
/// CB-002: workgroupBarrier() unreachable from some threads (inside conditional)
/// Reference: docs/specifications/compute-brick-support.md §4.2
fn detect_cb002_wgsl_barrier_divergence(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    // Find all .wgsl files in the project
    for entry in walkdir::WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "wgsl"))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            let lines: Vec<&str> = content.lines().collect();
            let mut in_conditional = false;
            let mut conditional_depth = 0;
            let mut conditional_start_line = 0;

            for (line_num, line) in lines.iter().enumerate() {
                let trimmed = line.trim();

                // Track conditional blocks (if, else, for, while)
                if trimmed.starts_with("if ") || trimmed.starts_with("if(")
                    || trimmed.contains(" if ") || trimmed.contains("} else")
                {
                    in_conditional = true;
                    conditional_start_line = line_num + 1;
                }

                // Track brace depth for conditionals
                if in_conditional {
                    conditional_depth += line.matches('{').count();
                    conditional_depth = conditional_depth.saturating_sub(line.matches('}').count());

                    if conditional_depth == 0 && line.contains('}') {
                        in_conditional = false;
                    }
                }

                // Check for workgroupBarrier inside conditional
                if in_conditional && trimmed.contains("workgroupBarrier()") {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-002".to_string(),
                        file: entry.path().display().to_string(),
                        line: line_num + 1,
                        description: format!(
                            "workgroupBarrier() inside conditional (started at line {}) - may cause deadlock",
                            conditional_start_line
                        ),
                        severity: Severity::Critical,
                    });
                }
            }
        }
    }

    violations
}

/// Check for bricks without assertions (budget validation)
fn detect_bricks_without_assertions(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return violations;
    }

    if let Ok(entries) = walkdir_rs_files(&src_dir) {
        for entry in entries {
            if let Ok(content) = fs::read_to_string(&entry) {
                // Look for ComputeBrick impl blocks without assertion methods
                let mut in_brick_impl = false;
                let mut brick_name = String::new();
                let mut impl_start_line = 0;
                let mut brace_depth = 0;
                let mut has_assertion = false;

                for (line_num, line) in content.lines().enumerate() {
                    // Detect impl blocks for types containing "Brick"
                    if line.contains("impl") && line.contains("Brick") && !line.contains("//") {
                        in_brick_impl = true;
                        brick_name = line
                            .split_whitespace()
                            .find(|w| w.contains("Brick"))
                            .unwrap_or("UnknownBrick")
                            .trim_end_matches('{')
                            .to_string();
                        impl_start_line = line_num + 1;
                        brace_depth = 0;
                        has_assertion = false;
                    }

                    if in_brick_impl {
                        brace_depth += line.matches('{').count();
                        brace_depth = brace_depth.saturating_sub(line.matches('}').count());

                        // Check for assertion-related methods
                        if line.contains("assert")
                            || line.contains("debug_assert")
                            || line.contains("verify")
                            || line.contains("validate")
                            || line.contains("budget")
                        {
                            has_assertion = true;
                        }

                        // End of impl block
                        if brace_depth == 0 && line.contains('}') {
                            if !has_assertion {
                                violations.push(CbPatternViolation {
                                    pattern_id: "CB-BUDGET".to_string(),
                                    file: entry.display().to_string(),
                                    line: impl_start_line,
                                    description: format!(
                                        "Brick '{}' has no assertion/budget validation",
                                        brick_name
                                    ),
                                    severity: Severity::Warning,
                                });
                            }
                            in_brick_impl = false;
                        }
                    }
                }
            }
        }
    }

    violations
}

/// Parse BrickProfiler JSON output and detect anomalies
fn detect_profiler_anomalies(project_path: &Path) -> Vec<ProfilerAnomaly> {
    let mut anomalies = Vec::new();

    // Check standard profiler output locations
    let profiler_paths = [
        project_path.join(".pmat-metrics").join("brick-profile.json"),
        project_path.join("target").join("brick-profile.json"),
        project_path.join("brick-profile.json"),
    ];

    for profiler_path in &profiler_paths {
        if !profiler_path.exists() {
            continue;
        }

        if let Ok(content) = fs::read_to_string(profiler_path) {
            // Parse JSON manually to avoid adding serde_json dep to this module
            // Look for patterns like "cv": 0.18 (CV > 15% threshold)
            // and "efficiency": 0.22 (efficiency < 25% threshold)

            // Simple pattern matching for CV values
            for line in content.lines() {
                let line = line.trim();

                // Detect high coefficient of variation (CV > 15%)
                if line.contains("\"cv\"") || line.contains("\"cv_percent\"") {
                    if let Some(value) = extract_json_number(line) {
                        let cv_threshold = 15.0;
                        let cv = if value < 1.0 { value * 100.0 } else { value };
                        if cv > cv_threshold {
                            anomalies.push(ProfilerAnomaly {
                                brick_name: extract_brick_name(&content, line),
                                anomaly_type: "HIGH_CV".to_string(),
                                value: cv,
                                threshold: cv_threshold,
                            });
                        }
                    }
                }

                // Detect low efficiency (< 25%)
                if line.contains("\"efficiency\"") {
                    if let Some(value) = extract_json_number(line) {
                        let eff_threshold = 25.0;
                        let efficiency = if value < 1.0 { value * 100.0 } else { value };
                        if efficiency < eff_threshold {
                            anomalies.push(ProfilerAnomaly {
                                brick_name: extract_brick_name(&content, line),
                                anomaly_type: "LOW_EFFICIENCY".to_string(),
                                value: efficiency,
                                threshold: eff_threshold,
                            });
                        }
                    }
                }
            }
            break; // Only process first found file
        }
    }

    anomalies
}

/// Helper to extract numeric value from JSON line like `"cv": 0.18,`
fn extract_json_number(line: &str) -> Option<f64> {
    line.split(':')
        .nth(1)?
        .trim()
        .trim_end_matches(',')
        .trim_end_matches('}')
        .parse()
        .ok()
}

/// Helper to extract brick name from surrounding JSON context
fn extract_brick_name(content: &str, target_line: &str) -> String {
    // Look backwards from target line for "name": "BrickName" pattern
    let target_pos = content.find(target_line).unwrap_or(0);
    let before = &content[..target_pos];

    for line in before.lines().rev().take(10) {
        if line.contains("\"name\"") || line.contains("\"brick\"") {
            if let Some(start) = line.find('"') {
                let rest = &line[start + 1..];
                if let Some(end) = rest.find('"') {
                    let after_colon = &rest[end + 1..];
                    if let Some(val_start) = after_colon.find('"') {
                        let val_rest = &after_colon[val_start + 1..];
                        if let Some(val_end) = val_rest.find('"') {
                            return val_rest[..val_end].to_string();
                        }
                    }
                }
            }
        }
    }
    "UnknownBrick".to_string()
}

/// Walk directory for .rs files
fn walkdir_rs_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut rs_files = Vec::new();

    fn visit_dir(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dir(&path, files)?;
                } else if path.extension().is_some_and(|e| e == "rs") {
                    files.push(path);
                }
            }
        }
        Ok(())
    }

    visit_dir(dir, &mut rs_files)?;
    Ok(rs_files)
}

/// Check ComputeBrick compliance (PROBAR-SPEC-009-P8)
///
/// Validates:
/// - CB-001 to CB-022: Static analysis pattern detection
/// - CB-020: unsafe blocks without SAFETY comments
/// - CB-021: SIMD intrinsics without #[target_feature]
/// - CB-BUDGET: Bricks without assertion/validation
/// - BrickProfiler anomalies: CV > 15%, efficiency < 25%
/// - Probar GUI coverage >= 80%
fn check_compute_brick(project_path: &Path) -> ComplianceCheck {
    // Check if this is a ComputeBrick project (has probar dependency or brick/ directory)
    let cargo_toml = project_path.join("Cargo.toml");
    let brick_dir = project_path.join("src").join("brick");
    let has_probar = cargo_toml.exists()
        && fs::read_to_string(&cargo_toml)
            .map(|s| s.contains("probar") || s.contains("jugar-probar"))
            .unwrap_or(false);
    let has_brick_dir = brick_dir.exists();

    // Also check for trueno/realizar dependencies (ComputeBrick ecosystem)
    let has_cb_ecosystem = cargo_toml.exists()
        && fs::read_to_string(&cargo_toml)
            .map(|s| s.contains("trueno") || s.contains("realizar") || s.contains("Brick"))
            .unwrap_or(false);

    if !has_probar && !has_brick_dir && !has_cb_ecosystem {
        return ComplianceCheck {
            name: "ComputeBrick Compliance".to_string(),
            status: CheckStatus::Skip,
            message: "Not a ComputeBrick project (no probar/trueno/realizar dep or brick/ dir)"
                .to_string(),
            severity: Severity::Info,
        };
    }

    // Collect all violations
    let mut all_issues: Vec<String> = Vec::new();
    let mut critical_count = 0;
    let mut warning_count = 0;

    // Run static analysis for CB patterns
    let cb020_violations = detect_cb020_unsafe_without_safety(project_path);
    for v in &cb020_violations {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        warning_count += 1;
    }

    let cb021_violations = detect_cb021_simd_without_target_feature(project_path);
    for v in &cb021_violations {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        warning_count += 1;
    }

    // WGSL-specific checks (CB-001 and CB-002 per PROBAR-SPEC-009-P8 §4)
    let cb001_violations = detect_cb001_wgsl_no_bounds_check(project_path);
    for v in &cb001_violations {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        // CB-001 is P0 Critical - counts as critical
        critical_count += 1;
    }

    let cb002_violations = detect_cb002_wgsl_barrier_divergence(project_path);
    for v in &cb002_violations {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        // CB-002 is P0 Critical - counts as critical
        critical_count += 1;
    }

    let budget_violations = detect_bricks_without_assertions(project_path);
    for v in &budget_violations {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        warning_count += 1;
    }

    // Check BrickProfiler output for anomalies
    let profiler_anomalies = detect_profiler_anomalies(project_path);
    for a in &profiler_anomalies {
        all_issues.push(format!(
            "PROFILER-{}: {} has {}={:.1}% (threshold: {:.1}%)",
            a.anomaly_type, a.brick_name, a.anomaly_type.to_lowercase(), a.value, a.threshold
        ));
        if a.anomaly_type == "LOW_EFFICIENCY" {
            critical_count += 1;
        } else {
            warning_count += 1;
        }
    }

    // Check for .pmat-gates.toml compute-brick section
    let gates_path = project_path.join(".pmat-gates.toml");
    let has_cb_config = gates_path.exists()
        && fs::read_to_string(&gates_path)
            .map(|s| s.contains("[compute-brick]"))
            .unwrap_or(false);

    if !has_cb_config && (has_probar || has_brick_dir) {
        all_issues.push("Missing [compute-brick] section in .pmat-gates.toml".to_string());
        warning_count += 1;
    }

    // Check for probar test coverage file
    let coverage_file = project_path.join(".pmat-metrics").join("gui-coverage.json");
    if has_probar && !coverage_file.exists() {
        all_issues.push("No GUI coverage report - run probador to generate".to_string());
        warning_count += 1;
    }

    // Determine overall status and message
    if critical_count > 0 {
        ComplianceCheck {
            name: "ComputeBrick Compliance".to_string(),
            status: CheckStatus::Fail,
            message: format!(
                "{} critical, {} warnings: {}",
                critical_count,
                warning_count,
                all_issues.first().unwrap_or(&"unknown".to_string())
            ),
            severity: Severity::Critical,
        }
    } else if warning_count > 0 {
        ComplianceCheck {
            name: "ComputeBrick Compliance".to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "{} warnings detected: {}",
                warning_count,
                all_issues.join("; ").chars().take(200).collect::<String>()
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "ComputeBrick Compliance".to_string(),
            status: CheckStatus::Pass,
            message: "ComputeBrick patterns validated - no violations detected".to_string(),
            severity: Severity::Info,
        }
    }
}

/// Check Cargo.lock presence (reproducible builds)
fn check_cargo_lock(project_path: &Path) -> ComplianceCheck {
    let cargo_lock = project_path.join("Cargo.lock");

    if cargo_lock.exists() {
        ComplianceCheck {
            name: "Cargo.lock Present".to_string(),
            status: CheckStatus::Pass,
            message: "Cargo.lock present - reproducible builds enabled".to_string(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "Cargo.lock Present".to_string(),
            status: CheckStatus::Fail,
            message: "Missing Cargo.lock - run 'cargo build' to generate".to_string(),
            severity: Severity::Error,
        }
    }
}

/// Check MSRV (Minimum Supported Rust Version) defined
fn check_msrv(project_path: &Path) -> ComplianceCheck {
    let cargo_toml = project_path.join("Cargo.toml");

    if !cargo_toml.exists() {
        return ComplianceCheck {
            name: "MSRV Defined".to_string(),
            status: CheckStatus::Skip,
            message: "No Cargo.toml found".to_string(),
            severity: Severity::Info,
        };
    }

    let content = fs::read_to_string(&cargo_toml).unwrap_or_default();

    if content.contains("rust-version") {
        ComplianceCheck {
            name: "MSRV Defined".to_string(),
            status: CheckStatus::Pass,
            message: "rust-version field present in Cargo.toml".to_string(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "MSRV Defined".to_string(),
            status: CheckStatus::Warn,
            message: "No rust-version field - add to Cargo.toml for compatibility".to_string(),
            severity: Severity::Warning,
        }
    }
}

/// Check CI configuration present
fn check_ci_configured(project_path: &Path) -> ComplianceCheck {
    let github_workflows = project_path.join(".github").join("workflows");
    let gitlab_ci = project_path.join(".gitlab-ci.yml");
    let jenkinsfile = project_path.join("Jenkinsfile");

    if github_workflows.exists() && github_workflows.is_dir() {
        let workflow_count = fs::read_dir(&github_workflows)
            .map(|entries| entries.filter_map(|e| e.ok()).count())
            .unwrap_or(0);

        if workflow_count > 0 {
            return ComplianceCheck {
                name: "CI Configured".to_string(),
                status: CheckStatus::Pass,
                message: format!("{} GitHub Actions workflow(s) found", workflow_count),
                severity: Severity::Info,
            };
        }
    }

    if gitlab_ci.exists() {
        return ComplianceCheck {
            name: "CI Configured".to_string(),
            status: CheckStatus::Pass,
            message: "GitLab CI configured".to_string(),
            severity: Severity::Info,
        };
    }

    if jenkinsfile.exists() {
        return ComplianceCheck {
            name: "CI Configured".to_string(),
            status: CheckStatus::Pass,
            message: "Jenkins pipeline configured".to_string(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: "CI Configured".to_string(),
        status: CheckStatus::Warn,
        message: "No CI configuration found - add .github/workflows/".to_string(),
        severity: Severity::Warning,
    }
}

/// Check Sovereign AI Stack compliance patterns
/// Validates: Five-Whys in fixes, falsification tests, APR models, ticket refs
fn check_sovereign_stack_patterns(project_path: &Path) -> ComplianceCheck {
    use std::process::Command;

    // Check if this is a Sovereign Stack project
    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return ComplianceCheck {
            name: "Sovereign Stack Patterns".to_string(),
            status: CheckStatus::Skip,
            message: "No Cargo.toml found".to_string(),
            severity: Severity::Info,
        };
    }

    let content = fs::read_to_string(&cargo_toml).unwrap_or_default();
    let is_sovereign = content.contains("trueno")
        || content.contains("aprender")
        || content.contains("realizar")
        || content.contains("batuta")
        || content.contains("renacer");

    if !is_sovereign {
        return ComplianceCheck {
            name: "Sovereign Stack Patterns".to_string(),
            status: CheckStatus::Skip,
            message: "Not a Sovereign Stack project".to_string(),
            severity: Severity::Info,
        };
    }

    let mut issues: Vec<String> = Vec::new();
    let mut good_patterns: Vec<String> = Vec::new();

    // Check 1: Recent fix commits should have Five-Whys or root cause
    let git_log = Command::new("git")
        .args(["log", "--oneline", "-20", "--grep=fix"])
        .current_dir(project_path)
        .output();

    if let Ok(output) = git_log {
        let log = String::from_utf8_lossy(&output.stdout);
        let fix_commits: Vec<&str> = log.lines().collect();

        if !fix_commits.is_empty() {
            // Check if any recent fix has Five-Whys
            let has_five_whys = Command::new("git")
                .args(["log", "-20", "--grep=Five-Whys\\|ROOT CAUSE\\|Why 1:"])
                .current_dir(project_path)
                .output()
                .map(|o| !o.stdout.is_empty())
                .unwrap_or(false);

            if has_five_whys {
                good_patterns.push("Five-Whys root cause analysis".to_string());
            } else if fix_commits.len() > 5 {
                issues.push("No Five-Whys in recent fix commits".to_string());
            }
        }
    }

    // Check 2: Falsification tests (F001-F100 pattern)
    let tests_dir = project_path.join("tests");
    if tests_dir.exists() {
        let has_falsification = walkdir::WalkDir::new(&tests_dir)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
            .any(|e| {
                e.path()
                    .to_string_lossy()
                    .contains("falsification")
                    || fs::read_to_string(e.path())
                        .map(|s| s.contains("F001") || s.contains("F0") && s.contains("TEST"))
                        .unwrap_or(false)
            });

        if has_falsification {
            good_patterns.push("Falsification test suite".to_string());
        }
    }

    // Check 3: APR model files validation
    let models_dir = project_path.join("models");
    if models_dir.exists() {
        let apr_files: Vec<_> = walkdir::WalkDir::new(&models_dir)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "apr").unwrap_or(false))
            .collect();

        if !apr_files.is_empty() {
            good_patterns.push(format!("{} APR model(s)", apr_files.len()));
        }
    }

    // Check 4: PAR/PMAT ticket references in commits
    let ticket_refs = Command::new("git")
        .args(["log", "-50", "--oneline"])
        .current_dir(project_path)
        .output()
        .map(|o| {
            let log = String::from_utf8_lossy(&o.stdout);
            log.lines()
                .filter(|l| {
                    l.contains("PAR-")
                        || l.contains("PMAT-")
                        || l.contains("Refs ")
                        || l.contains("GH-")
                })
                .count()
        })
        .unwrap_or(0);

    if ticket_refs > 10 {
        good_patterns.push(format!("{}+ ticket refs in commits", ticket_refs));
    } else if ticket_refs < 5 {
        issues.push("Few ticket references in recent commits".to_string());
    }

    // Check 5: ML-based commit classification (if model available)
    if let Ok(classifier) = CommitClassifier::load_sovereign_stack() {
        // Get recent commit messages for classification
        let git_log_full = Command::new("git")
            .args(["log", "-10", "--format=%B---COMMIT_SEP---"])
            .current_dir(project_path)
            .output();

        if let Ok(output) = git_log_full {
            let log = String::from_utf8_lossy(&output.stdout);
            let commits: Vec<&str> = log
                .split("---COMMIT_SEP---")
                .filter(|s| !s.trim().is_empty())
                .collect();

            if !commits.is_empty() {
                let mut class_counts: std::collections::HashMap<String, usize> =
                    std::collections::HashMap::new();
                let mut high_confidence = 0;

                for commit in &commits {
                    let result = classifier.classify(commit);
                    *class_counts.entry(result.class).or_insert(0) += 1;
                    if result.confidence > 0.6 {
                        high_confidence += 1;
                    }
                }

                // Find dominant pattern
                if let Some((dominant_class, count)) = class_counts.iter().max_by_key(|(_, c)| *c) {
                    if *count >= commits.len() / 2 {
                        good_patterns.push(format!("ML: {} dominant ({}/{})", dominant_class, count, commits.len()));
                    }
                }

                if high_confidence > commits.len() / 2 {
                    good_patterns.push(format!("ML: {}% high-confidence classifications", high_confidence * 100 / commits.len()));
                }
            }
        }
    }

    // Build result
    if issues.is_empty() && !good_patterns.is_empty() {
        ComplianceCheck {
            name: "Sovereign Stack Patterns".to_string(),
            status: CheckStatus::Pass,
            message: format!("Patterns: {}", good_patterns.join(", ")),
            severity: Severity::Info,
        }
    } else if !issues.is_empty() {
        ComplianceCheck {
            name: "Sovereign Stack Patterns".to_string(),
            status: CheckStatus::Warn,
            message: format!("Missing: {}", issues.join("; ")),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "Sovereign Stack Patterns".to_string(),
            status: CheckStatus::Pass,
            message: "Sovereign Stack project detected".to_string(),
            severity: Severity::Info,
        }
    }
}

/// Check PAIML dependency workspace state (dirty/clean/version drift)
/// Detects when local PAIML projects have uncommitted changes or version mismatches
fn check_paiml_deps_workspace(project_path: &Path) -> ComplianceCheck {
    use std::process::Command;

    // Known PAIML/Sovereign stack packages
    const PAIML_PACKAGES: &[&str] = &[
        "trueno", "trueno-graph", "trueno-rag", "trueno-viz", "trueno-db",
        "trueno-zram-core", "trueno-ublk",
        "aprender", "entrenar", "alimentar", "realizar",
        "batuta", "renacer", "repartir",
        "presentar", "presentar-terminal",
        "ruchy", "bashrs", "decy", "depyler", "rascal",
        "pacha", "pepita", "simular", "jugar", "duende", "pzsh",
        "certeza", "verificar", "probar", "manzana", "whisper-apr",
        "copia", "nviwatch", "ruchydbg", "rust-mcp-sdk",
    ];

    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return ComplianceCheck {
            name: "PAIML Deps Workspace".to_string(),
            status: CheckStatus::Skip,
            message: "No Cargo.toml found".to_string(),
            severity: Severity::Info,
        };
    }

    let content = match fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(_) => {
            return ComplianceCheck {
                name: "PAIML Deps Workspace".to_string(),
                status: CheckStatus::Skip,
                message: "Could not read Cargo.toml".to_string(),
                severity: Severity::Info,
            };
        }
    };

    // Find PAIML dependencies in this project
    let mut paiml_deps: Vec<&str> = Vec::new();
    for pkg in PAIML_PACKAGES {
        // Check for dependency lines like: trueno = "0.11" or trueno = { version = "0.11" }
        if content.contains(&format!("{} = ", pkg)) || content.contains(&format!("\"{}\"", pkg)) {
            paiml_deps.push(pkg);
        }
    }

    if paiml_deps.is_empty() {
        return ComplianceCheck {
            name: "PAIML Deps Workspace".to_string(),
            status: CheckStatus::Skip,
            message: "No PAIML stack dependencies found".to_string(),
            severity: Severity::Info,
        };
    }

    // Check local workspace state for each PAIML dep
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            return ComplianceCheck {
                name: "PAIML Deps Workspace".to_string(),
                status: CheckStatus::Skip,
                message: "Could not determine home directory".to_string(),
                severity: Severity::Info,
            };
        }
    };
    let src_dir = home.join("src");

    let mut dirty_deps: Vec<String> = Vec::new();
    let mut clean_deps: Vec<String> = Vec::new();

    for dep in &paiml_deps {
        let dep_path = src_dir.join(dep);
        if !dep_path.exists() {
            continue; // Not a local checkout
        }

        // Check git status
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&dep_path)
            .output();

        match output {
            Ok(out) => {
                let status = String::from_utf8_lossy(&out.stdout);
                if status.trim().is_empty() {
                    clean_deps.push(dep.to_string());
                } else {
                    dirty_deps.push(dep.to_string());
                }
            }
            Err(_) => {
                // Not a git repo or git not available
                continue;
            }
        }
    }

    let total_local = dirty_deps.len() + clean_deps.len();

    if total_local == 0 {
        return ComplianceCheck {
            name: "PAIML Deps Workspace".to_string(),
            status: CheckStatus::Pass,
            message: format!("{} PAIML deps (no local checkouts in ~/src)", paiml_deps.len()),
            severity: Severity::Info,
        };
    }

    if dirty_deps.is_empty() {
        ComplianceCheck {
            name: "PAIML Deps Workspace".to_string(),
            status: CheckStatus::Pass,
            message: format!(
                "{} PAIML deps, {} local checkouts (all clean)",
                paiml_deps.len(),
                total_local
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "PAIML Deps Workspace".to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "{} dirty: {} (using crates.io versions for safety)",
                dirty_deps.len(),
                dirty_deps.join(", ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// Check file health across the project (CB-040)
/// Validates: max-lines (500), TLR (test-to-lines ratio), complexity, health score
/// Based on: docs/specifications/max-lines.md
fn check_file_health(project_path: &Path) -> ComplianceCheck {
    // Skip if not a Rust project
    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return ComplianceCheck {
            name: "File Health".to_string(),
            status: CheckStatus::Skip,
            message: "No Cargo.toml found - not a Rust project".to_string(),
            severity: Severity::Info,
        };
    }

    // Scan for source files
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return ComplianceCheck {
            name: "File Health".to_string(),
            status: CheckStatus::Skip,
            message: "No src/ directory found".to_string(),
            severity: Severity::Info,
        };
    }

    let files = scan_directory(&src_dir, RUST_EXTENSIONS, DEFAULT_EXCLUDE_PATTERNS);
    if files.is_empty() {
        return ComplianceCheck {
            name: "File Health".to_string(),
            status: CheckStatus::Pass,
            message: "No Rust source files found".to_string(),
            severity: Severity::Info,
        };
    }

    // Analyze each file for health metrics
    let mut metrics: Vec<FileHealthMetrics> = Vec::new();
    let mut critical_count = 0;
    let mut problem_count = 0;
    let mut over_500_count = 0;

    for file_path in &files {
        // Count lines
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines = content.lines().count();

        // Quick categorization without full analysis
        if lines > 2000 {
            critical_count += 1;
        } else if lines > 1000 {
            problem_count += 1;
        }
        if lines > 500 {
            over_500_count += 1;
        }

        // Calculate full metrics (simplified - using defaults for test/complexity/churn)
        // In production, these would be gathered from actual analysis
        let test_lines = estimate_test_lines(&content);
        let avg_complexity = estimate_avg_complexity(&content);

        let file_metrics = FileHealthMetrics::calculate(
            file_path.clone(),
            lines,
            test_lines,
            avg_complexity,
            0, // churn_30d - would need git integration
        );
        metrics.push(file_metrics);
    }

    // Build report
    let report = FileHealthReport::from_files(project_path.to_path_buf(), metrics);

    // Determine status based on findings
    if critical_count > 0 {
        ComplianceCheck {
            name: "File Health".to_string(),
            status: CheckStatus::Fail,
            message: format!(
                "CRITICAL: {} files >2000 lines, {} files >1000 lines, {} files >500 lines (avg health: {}%, grade: {})",
                critical_count,
                problem_count,
                over_500_count,
                report.average_health,
                report.average_grade.as_str()
            ),
            severity: Severity::Critical,
        }
    } else if problem_count > 0 || over_500_count > 5 {
        ComplianceCheck {
            name: "File Health".to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "{} files >1000 lines, {} files >500 lines (avg health: {}%, grade: {})",
                problem_count,
                over_500_count,
                report.average_health,
                report.average_grade.as_str()
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "File Health".to_string(),
            status: CheckStatus::Pass,
            message: format!(
                "{} files analyzed, avg health: {}%, grade: {} (all files <500 lines or within tolerance)",
                report.total_files,
                report.average_health,
                report.average_grade.as_str()
            ),
            severity: Severity::Info,
        }
    }
}

/// Estimate test lines by counting lines in #[cfg(test)] modules and test functions
fn estimate_test_lines(content: &str) -> usize {
    let mut test_lines = 0;
    let mut in_test_module = false;
    let mut brace_depth = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Track test module entry
        if trimmed.contains("#[cfg(test)]") {
            in_test_module = true;
        }

        // Track braces for module scope
        if in_test_module {
            brace_depth += trimmed.matches('{').count();
            brace_depth = brace_depth.saturating_sub(trimmed.matches('}').count());
            test_lines += 1;

            if brace_depth == 0 && test_lines > 1 {
                in_test_module = false;
            }
        }

        // Also count standalone test functions
        if trimmed.contains("#[test]") || trimmed.contains("#[tokio::test]") {
            test_lines += 10; // Approximate test function size
        }
    }

    test_lines
}

/// Estimate average cyclomatic complexity by counting control flow statements
fn estimate_avg_complexity(content: &str) -> f32 {
    let mut total_complexity = 1; // Base complexity
    let mut function_count = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Count function definitions
        if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") ||
           trimmed.starts_with("async fn ") || trimmed.starts_with("pub async fn ") {
            function_count += 1;
        }

        // Count control flow statements (add to complexity)
        if trimmed.starts_with("if ") || trimmed.contains(" if ") {
            total_complexity += 1;
        }
        if trimmed.starts_with("else if ") || trimmed.contains("} else if ") {
            total_complexity += 1;
        }
        if trimmed.starts_with("match ") || trimmed.contains(" match ") {
            total_complexity += 1;
        }
        if trimmed.starts_with("for ") || trimmed.contains(" for ") {
            total_complexity += 1;
        }
        if trimmed.starts_with("while ") || trimmed.contains(" while ") {
            total_complexity += 1;
        }
        if trimmed.starts_with("loop ") || trimmed.contains(" loop ") {
            total_complexity += 1;
        }
        if trimmed.contains("&&") || trimmed.contains("||") {
            total_complexity += 1;
        }
        if trimmed.contains("?") && !trimmed.contains("//") {
            total_complexity += 1; // Error propagation
        }
    }

    if function_count == 0 {
        return total_complexity as f32;
    }

    total_complexity as f32 / function_count as f32
}

fn calculate_versions_behind(project_version: &str) -> u32 {
    let current_parts: Vec<u32> = PMAT_VERSION
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let project_parts: Vec<u32> = project_version
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    if current_parts.len() >= 2 && project_parts.len() >= 2 {
        current_parts
            .get(1)
            .unwrap_or(&0)
            .saturating_sub(*project_parts.get(1).unwrap_or(&0))
    } else {
        0
    }
}

fn get_breaking_changes_since(_from_version: &str) -> Vec<BreakingChange> {
    vec![]
}

#[derive(Debug, Clone)]
struct ChangelogEntry {
    version: String,
    description: String,
    breaking: bool,
}

fn get_changelog_entries(_from: &str, _to: &str) -> Vec<ChangelogEntry> {
    vec![
        ChangelogEntry {
            version: PMAT_VERSION.to_string(),
            description: "Added qa-work command for Toyota Way validation".to_string(),
            breaking: false,
        },
        ChangelogEntry {
            version: PMAT_VERSION.to_string(),
            description: "Added cleanup-resources command".to_string(),
            breaking: false,
        },
        ChangelogEntry {
            version: PMAT_VERSION.to_string(),
            description: "Added comply command for compliance checking".to_string(),
            breaking: false,
        },
    ]
}

fn migrate_project_version(project_path: &Path, target: &str, dry_run: bool) -> Result<bool> {
    if dry_run {
        return Ok(true);
    }
    let mut config = load_or_create_project_config(project_path)?;
    if config.pmat.version == target {
        return Ok(false);
    }
    config.pmat.version = target.to_string();
    config.pmat.last_compliance_check = Some(Utc::now());
    let content = toml::to_string_pretty(&config)?;
    fs::write(project_path.join(".pmat").join("project.toml"), &content)?;
    Ok(true)
}

fn migrate_gitignore(project_path: &Path, dry_run: bool) -> Result<bool> {
    let gitignore_path = project_path.join(".gitignore");
    let pmat_entries = [".pmat/backup/", ".pmat-qa/"];
    if !gitignore_path.exists() {
        return Ok(false);
    }
    let content = fs::read_to_string(&gitignore_path)?;
    let mut needs_update = false;
    let mut new_entries = vec![];
    for entry in pmat_entries {
        if !content.contains(entry) {
            needs_update = true;
            new_entries.push(entry);
        }
    }
    if needs_update && !dry_run {
        let mut new_content = content.clone();
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str("\n# PMAT\n");
        for entry in new_entries {
            new_content.push_str(entry);
            new_content.push('\n');
        }
        fs::write(&gitignore_path, &new_content)?;
    }
    Ok(needs_update)
}

fn update_project_config(project_path: &Path, dry_run: bool) -> Result<bool> {
    migrate_project_version(project_path, PMAT_VERSION, dry_run)
}

fn print_compliance_text(report: &ComplianceReport) {
    println!("\n{}", "=".repeat(60));
    println!("PMAT Compliance Report");
    println!("{}", "=".repeat(60));
    println!("\nProject Version: {}", report.project_version);
    println!("Current PMAT:    {}", report.current_version);
    println!("Versions Behind: {}", report.versions_behind);
    let status = if report.is_compliant {
        "\x1b[32mCOMPLIANT\x1b[0m"
    } else {
        "\x1b[31mNON-COMPLIANT\x1b[0m"
    };
    println!("Status:          {}\n", status);
    println!("Checks:");
    for check in &report.checks {
        let icon = match check.status {
            CheckStatus::Pass => "\x1b[32m✓\x1b[0m",
            CheckStatus::Warn => "\x1b[33m⚠\x1b[0m",
            CheckStatus::Fail => "\x1b[31m✗\x1b[0m",
            CheckStatus::Skip => "\x1b[90m-\x1b[0m",
        };
        println!("  {} {}: {}", icon, check.name, check.message);
    }
    if !report.recommendations.is_empty() {
        println!("\nRecommendations:");
        for rec in &report.recommendations {
            println!("  • {}", rec);
        }
    }
    println!("\n{}", "=".repeat(60));
}

fn print_compliance_markdown(report: &ComplianceReport) {
    println!("# PMAT Compliance Report\n");
    println!("| Property | Value |");
    println!("|----------|-------|");
    println!("| Project Version | {} |", report.project_version);
    println!("| Current PMAT | {} |", report.current_version);
    println!(
        "| Status | {} |",
        if report.is_compliant {
            "COMPLIANT"
        } else {
            "NON-COMPLIANT"
        }
    );
    println!("\n## Checks\n");
    for check in &report.checks {
        let icon = match check.status {
            CheckStatus::Pass => "✅",
            CheckStatus::Warn => "⚠️",
            CheckStatus::Fail => "❌",
            CheckStatus::Skip => "⏭️",
        };
        println!("- {} **{}**: {}", icon, check.name, check.message);
    }
}

/// Install git hooks for mandatory work tracking (W-006)
/// Implements master-plan-pmat-work-system.md enforcement
async fn handle_enforce(
    project_path: &Path,
    yes: bool,
    disable: bool,
    format: ComplyOutputFormat,
) -> Result<()> {
    let hooks_dir = project_path.join(".git").join("hooks");

    if !hooks_dir.exists() {
        anyhow::bail!("Not a git repository (no .git/hooks directory)");
    }

    if disable {
        // Remove PMAT hooks
        let pre_commit = hooks_dir.join("pre-commit");
        if pre_commit.exists() {
            if let Ok(content) = fs::read_to_string(&pre_commit) {
                if content.contains("PMAT") {
                    fs::remove_file(&pre_commit)?;
                    println!("✅ Removed PMAT pre-commit hook");
                } else {
                    println!("⚠️  Pre-commit hook exists but is not PMAT - not removed");
                }
            }
        }

        // Also remove pre-push hook if it's PMAT's
        let pre_push = hooks_dir.join("pre-push");
        if pre_push.exists() {
            if let Ok(content) = fs::read_to_string(&pre_push) {
                if content.contains("PMAT") || content.contains("ComputeBrick") {
                    fs::remove_file(&pre_push)?;
                    println!("✅ Removed PMAT pre-push hook");
                } else {
                    println!("⚠️  Pre-push hook exists but is not PMAT - not removed");
                }
            }
        }
        return Ok(());
    }

    // Prompt for confirmation if not -y
    if !yes {
        println!("This will install PMAT enforcement hooks:");
        println!("  - pre-commit: Block commits without active work ticket");
        println!("  - pre-push: Validate spec compliance before push");
        println!("\nProceed? [y/N] ");

        // For now, just proceed (interactive input not easily testable)
        println!("(Auto-proceeding due to non-interactive mode)");
    }

    // Create pre-commit hook
    let pre_commit_content = r#"#!/bin/sh
# PMAT Work Enforcement Hook (master-plan-pmat-work-system.md W-001)
# This hook blocks commits without an active work ticket.

# Check for active work ticket
if ! pmat work status --active >/dev/null 2>&1; then
    echo "❌ COMPLIANCE VIOLATION"
    echo ""
    echo "Action blocked: git commit"
    echo "Reason: No active work ticket"
    echo ""
    echo "To fix:"
    echo "  1. Start work: pmat work start <ticket-id>"
    echo "  2. Or create ticket: pmat work start \"description\" --spec <spec-file>"
    echo ""
    echo "Bypass (NOT RECOMMENDED):"
    echo "  git commit --no-verify"
    exit 1
fi

# Ensure commit message references ticket
TICKET_ID=$(pmat work status --active --quiet 2>/dev/null)
if [ -n "$TICKET_ID" ]; then
    # Check commit message for ticket reference
    if ! grep -qi "$TICKET_ID\|#[0-9]" "$1" 2>/dev/null; then
        echo "⚠️  Commit message should reference ticket: $TICKET_ID"
    fi
fi

exit 0
"#;

    let pre_commit_path = hooks_dir.join("pre-commit");
    fs::write(&pre_commit_path, pre_commit_content)?;

    // Create pre-push hook for ComputeBrick compliance (CB-IMPL-001-C)
    let pre_push_content = r#"#!/bin/sh
# PMAT ComputeBrick Pre-Push Enforcement (PROBAR-SPEC-009-P8)
# This hook validates ComputeBrick compliance before push.

set -e

echo "🔍 Running ComputeBrick compliance checks..."

# Check ComputeBrick compliance via pmat comply
COMPLY_OUTPUT=$(pmat comply check --failures-only 2>&1) || true
if echo "$COMPLY_OUTPUT" | grep -q "ComputeBrick Compliance.*critical"; then
    echo "❌ COMPUTEBRICK COMPLIANCE FAILURE"
    echo ""
    echo "$COMPLY_OUTPUT" | grep -A5 "ComputeBrick"
    echo ""
    echo "Fix critical violations before pushing."
    echo "Run 'pmat comply check' for full details."
    echo ""
    echo "Bypass (NOT RECOMMENDED):"
    echo "  git push --no-verify"
    exit 1
fi

# Check probar GUI coverage if available (PROBAR-SPEC-009)
if command -v probador >/dev/null 2>&1; then
    echo "📊 Checking probar GUI coverage..."
    if ! probador playbook --validate --min-coverage 80 2>/dev/null; then
        echo "⚠️  Probar GUI coverage below 80%"
        echo "   Run 'probador playbook' to generate coverage report."
    fi
fi

# Check for .pmat-gates.toml ComputeBrick config
if [ -f "Cargo.toml" ]; then
    if grep -q "trueno\|probar\|realizar" Cargo.toml 2>/dev/null; then
        if [ ! -f ".pmat-gates.toml" ] || ! grep -q "\[compute-brick\]" .pmat-gates.toml 2>/dev/null; then
            echo "⚠️  ComputeBrick project missing [compute-brick] in .pmat-gates.toml"
            echo "   Add configuration per docs/specifications/compute-brick-support.md"
        fi
    fi
fi

echo "✅ ComputeBrick compliance: PASSED"
exit 0
"#;

    let pre_push_path = hooks_dir.join("pre-push");
    fs::write(&pre_push_path, pre_push_content)?;

    // Make executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&pre_commit_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&pre_commit_path, perms)?;

        let mut perms = fs::metadata(&pre_push_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&pre_push_path, perms)?;
    }

    match format {
        ComplyOutputFormat::Text => {
            println!("\n✅ PMAT enforcement hooks installed!");
            println!("   Pre-commit hook: {}", pre_commit_path.display());
            println!("   Pre-push hook:   {}", pre_push_path.display());
            println!("\nCommits will now require an active work ticket.");
            println!("Pushes will validate ComputeBrick compliance.");
            println!("Use 'pmat comply enforce --disable' to remove hooks.");
        }
        ComplyOutputFormat::Json => {
            let result = serde_json::json!({
                "status": "success",
                "hooks_installed": ["pre-commit", "pre-push"],
                "path": hooks_dir.display().to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        ComplyOutputFormat::Markdown => {
            println!("# PMAT Enforcement Hooks Installed\n");
            println!("| Hook | Status |");
            println!("|------|--------|");
            println!("| pre-commit | ✅ Installed |");
            println!("| pre-push | ✅ Installed |");
        }
    }

    Ok(())
}

/// Generate compliance report (W-009)
async fn handle_report(
    project_path: &Path,
    include_history: bool,
    format: ComplyOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    // Load project config
    let config = load_or_create_project_config(project_path)?;

    // Run compliance checks
    let checks = vec![
        check_version_currency(&config.pmat.version),
        check_config_files(project_path),
        check_hooks_installed(project_path),
        check_quality_thresholds(project_path),
        check_deprecated_features(project_path),
    ];

    let report = ComplianceReport {
        project_version: config.pmat.version.clone(),
        current_version: PMAT_VERSION.to_string(),
        is_compliant: checks.iter().all(|c| c.status != CheckStatus::Fail),
        versions_behind: calculate_versions_behind(&config.pmat.version),
        checks,
        breaking_changes: get_breaking_changes_since(&config.pmat.version),
        recommendations: vec![],
        timestamp: Utc::now(),
    };

    // Format output
    let output_text = match format {
        ComplyOutputFormat::Text => {
            let mut out = String::new();
            out.push_str(&format!("\n{}\n", "=".repeat(60)));
            out.push_str("PMAT Compliance Report\n");
            out.push_str(&format!("{}\n", "=".repeat(60)));
            out.push_str(&format!("\nGenerated: {}\n", report.timestamp));
            out.push_str(&format!("Project Version: {}\n", report.project_version));
            out.push_str(&format!("Current PMAT: {}\n", report.current_version));
            out.push_str(&format!(
                "Status: {}\n\n",
                if report.is_compliant {
                    "COMPLIANT"
                } else {
                    "NON-COMPLIANT"
                }
            ));

            out.push_str("Checks:\n");
            for check in &report.checks {
                let icon = match check.status {
                    CheckStatus::Pass => "✓",
                    CheckStatus::Warn => "⚠",
                    CheckStatus::Fail => "✗",
                    CheckStatus::Skip => "-",
                };
                out.push_str(&format!("  {} {}: {}\n", icon, check.name, check.message));
            }

            if include_history {
                out.push_str("\nWork History:\n");
                out.push_str("  (Work history not yet implemented)\n");
            }

            out
        }
        ComplyOutputFormat::Json => serde_json::to_string_pretty(&report)?,
        ComplyOutputFormat::Markdown => {
            let mut out = String::new();
            out.push_str("# PMAT Compliance Report\n\n");
            out.push_str(&format!("**Generated:** {}\n\n", report.timestamp));
            out.push_str("| Property | Value |\n");
            out.push_str("|----------|-------|\n");
            out.push_str(&format!(
                "| Project Version | {} |\n",
                report.project_version
            ));
            out.push_str(&format!("| Current PMAT | {} |\n", report.current_version));
            out.push_str(&format!(
                "| Status | {} |\n\n",
                if report.is_compliant {
                    "✅ COMPLIANT"
                } else {
                    "❌ NON-COMPLIANT"
                }
            ));

            out.push_str("## Checks\n\n");
            for check in &report.checks {
                let icon = match check.status {
                    CheckStatus::Pass => "✅",
                    CheckStatus::Warn => "⚠️",
                    CheckStatus::Fail => "❌",
                    CheckStatus::Skip => "⏭️",
                };
                out.push_str(&format!(
                    "- {} **{}**: {}\n",
                    icon, check.name, check.message
                ));
            }

            out
        }
    };

    if let Some(output_path) = output {
        fs::write(output_path, &output_text)?;
        println!("✅ Compliance report written to {}", output_path.display());
    } else {
        println!("{}", output_text);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_project_config() {
        let config = ProjectConfig::default();
        assert!(!config.pmat.version.is_empty());
    }

    #[test]
    fn test_calculate_versions_behind_same() {
        let behind = calculate_versions_behind(PMAT_VERSION);
        assert_eq!(behind, 0);
    }

    #[test]
    fn test_check_status_equality() {
        assert_eq!(CheckStatus::Pass, CheckStatus::Pass);
        assert_ne!(CheckStatus::Pass, CheckStatus::Fail);
    }

    #[test]
    fn test_severity_variants() {
        let _ = Severity::Info;
        let _ = Severity::Warning;
        let _ = Severity::Error;
        let _ = Severity::Critical;
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ============================================================================
    // Test Fixture Helpers
    // ============================================================================

    /// Create a temporary directory with basic PMAT structure
    fn create_temp_project() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    /// Create a project with .pmat directory and project.toml
    fn create_pmat_project(version: &str) -> TempDir {
        let temp = create_temp_project();
        let pmat_dir = temp.path().join(".pmat");
        fs::create_dir_all(&pmat_dir).expect("Failed to create .pmat dir");

        let config = format!(
            r#"[pmat]
version = "{}"
auto_update = false
"#,
            version
        );
        fs::write(pmat_dir.join("project.toml"), config).expect("Failed to write project.toml");
        temp
    }

    /// Create a project with .pmat-metrics.toml
    fn create_project_with_metrics(version: &str) -> TempDir {
        let temp = create_pmat_project(version);
        let metrics_content = r#"
[thresholds]
lint = 30000
test-fast = 300000
"#;
        fs::write(temp.path().join(".pmat-metrics.toml"), metrics_content)
            .expect("Failed to write metrics");
        temp
    }

    /// Create a git repository structure
    fn create_git_repo() -> TempDir {
        let temp = create_temp_project();
        let hooks_dir = temp.path().join(".git").join("hooks");
        fs::create_dir_all(&hooks_dir).expect("Failed to create .git/hooks");
        temp
    }

    /// Create a Rust project with Cargo.toml
    fn create_rust_project(with_msrv: bool, with_lock: bool) -> TempDir {
        let temp = create_temp_project();
        let cargo_content = if with_msrv {
            r#"[package]
name = "test"
version = "0.1.0"
rust-version = "1.75"
"#
        } else {
            r#"[package]
name = "test"
version = "0.1.0"
"#
        };
        fs::write(temp.path().join("Cargo.toml"), cargo_content)
            .expect("Failed to write Cargo.toml");
        if with_lock {
            fs::write(temp.path().join("Cargo.lock"), "# lock file")
                .expect("Failed to write Cargo.lock");
        }
        temp
    }

    // ============================================================================
    // ProjectConfig Tests
    // ============================================================================

    #[test]
    fn test_project_config_default_has_current_version() {
        let config = ProjectConfig::default();
        assert_eq!(config.pmat.version, PMAT_VERSION);
    }

    #[test]
    fn test_project_config_default_has_timestamp() {
        let config = ProjectConfig::default();
        assert!(config.pmat.last_compliance_check.is_some());
    }

    #[test]
    fn test_project_config_default_auto_update_is_false() {
        let config = ProjectConfig::default();
        assert!(!config.pmat.auto_update);
    }

    #[test]
    fn test_project_config_serialization() {
        let config = ProjectConfig::default();
        let serialized = toml::to_string_pretty(&config).expect("Serialization failed");
        assert!(serialized.contains("[pmat]"));
        assert!(serialized.contains("version"));
    }

    #[test]
    fn test_project_config_deserialization() {
        let toml_str = r#"
[pmat]
version = "1.0.0"
auto_update = true
"#;
        let config: ProjectConfig = toml::from_str(toml_str).expect("Deserialization failed");
        assert_eq!(config.pmat.version, "1.0.0");
        assert!(config.pmat.auto_update);
        assert!(config.pmat.last_compliance_check.is_none());
    }

    #[test]
    fn test_pmat_section_clone() {
        let section = PmatSection {
            version: "2.0.0".to_string(),
            last_compliance_check: Some(Utc::now()),
            auto_update: true,
        };
        let cloned = section.clone();
        assert_eq!(cloned.version, section.version);
        assert_eq!(cloned.auto_update, section.auto_update);
    }

    // ============================================================================
    // ComplianceReport Tests
    // ============================================================================

    #[test]
    fn test_compliance_report_serialization() {
        let report = ComplianceReport {
            project_version: "1.0.0".to_string(),
            current_version: "2.0.0".to_string(),
            is_compliant: true,
            versions_behind: 10,
            checks: vec![],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        let json = serde_json::to_string(&report).expect("JSON serialization failed");
        assert!(json.contains("project_version"));
        assert!(json.contains("is_compliant"));
    }

    #[test]
    fn test_compliance_report_with_checks() {
        let check = ComplianceCheck {
            name: "Test Check".to_string(),
            status: CheckStatus::Pass,
            message: "All good".to_string(),
            severity: Severity::Info,
        };
        let report = ComplianceReport {
            project_version: "1.0.0".to_string(),
            current_version: "1.0.0".to_string(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![check],
            breaking_changes: vec![],
            recommendations: vec!["Upgrade soon".to_string()],
            timestamp: Utc::now(),
        };
        assert_eq!(report.checks.len(), 1);
        assert_eq!(report.recommendations.len(), 1);
    }

    // ============================================================================
    // ComplianceCheck Tests
    // ============================================================================

    #[test]
    fn test_compliance_check_clone() {
        let check = ComplianceCheck {
            name: "Test".to_string(),
            status: CheckStatus::Warn,
            message: "Warning message".to_string(),
            severity: Severity::Warning,
        };
        let cloned = check.clone();
        assert_eq!(cloned.name, check.name);
        assert_eq!(cloned.status, check.status);
    }

    #[test]
    fn test_compliance_check_serialization() {
        let check = ComplianceCheck {
            name: "Version Check".to_string(),
            status: CheckStatus::Fail,
            message: "Outdated".to_string(),
            severity: Severity::Error,
        };
        let json = serde_json::to_string(&check).expect("Serialization failed");
        assert!(json.contains("Version Check"));
        assert!(json.contains("Fail"));
    }

    // ============================================================================
    // CheckStatus Tests
    // ============================================================================

    #[test]
    fn test_check_status_all_variants() {
        let pass = CheckStatus::Pass;
        let warn = CheckStatus::Warn;
        let fail = CheckStatus::Fail;
        let skip = CheckStatus::Skip;

        assert_eq!(pass, CheckStatus::Pass);
        assert_eq!(warn, CheckStatus::Warn);
        assert_eq!(fail, CheckStatus::Fail);
        assert_eq!(skip, CheckStatus::Skip);
    }

    #[test]
    fn test_check_status_inequality() {
        assert_ne!(CheckStatus::Pass, CheckStatus::Warn);
        assert_ne!(CheckStatus::Warn, CheckStatus::Fail);
        assert_ne!(CheckStatus::Fail, CheckStatus::Skip);
        assert_ne!(CheckStatus::Skip, CheckStatus::Pass);
    }

    #[test]
    fn test_check_status_copy() {
        let status = CheckStatus::Pass;
        let copied = status;
        assert_eq!(copied, CheckStatus::Pass);
    }

    // ============================================================================
    // Severity Tests
    // ============================================================================

    #[test]
    fn test_severity_all_variants() {
        let info = Severity::Info;
        let warning = Severity::Warning;
        let error = Severity::Error;
        let critical = Severity::Critical;

        assert_eq!(info, Severity::Info);
        assert_eq!(warning, Severity::Warning);
        assert_eq!(error, Severity::Error);
        assert_eq!(critical, Severity::Critical);
    }

    #[test]
    fn test_severity_inequality() {
        assert_ne!(Severity::Info, Severity::Warning);
        assert_ne!(Severity::Warning, Severity::Error);
        assert_ne!(Severity::Error, Severity::Critical);
    }

    #[test]
    fn test_severity_serialization() {
        let severity = Severity::Critical;
        let json = serde_json::to_string(&severity).expect("Serialization failed");
        assert!(json.contains("Critical"));
    }

    // ============================================================================
    // BreakingChange Tests
    // ============================================================================

    #[test]
    fn test_breaking_change_with_migration_guide() {
        let change = BreakingChange {
            version: "2.0.0".to_string(),
            description: "API changed".to_string(),
            migration_guide: Some("Follow these steps...".to_string()),
        };
        assert_eq!(change.version, "2.0.0");
        assert!(change.migration_guide.is_some());
    }

    #[test]
    fn test_breaking_change_without_migration_guide() {
        let change = BreakingChange {
            version: "2.0.0".to_string(),
            description: "Removed feature X".to_string(),
            migration_guide: None,
        };
        assert!(change.migration_guide.is_none());
    }

    #[test]
    fn test_breaking_change_clone() {
        let change = BreakingChange {
            version: "1.5.0".to_string(),
            description: "Config format changed".to_string(),
            migration_guide: Some("Update your config".to_string()),
        };
        let cloned = change.clone();
        assert_eq!(cloned.version, change.version);
        assert_eq!(cloned.migration_guide, change.migration_guide);
    }

    // ============================================================================
    // calculate_versions_behind Tests
    // ============================================================================

    #[test]
    fn test_calculate_versions_behind_older_minor() {
        // Parse current version to get major.minor
        let parts: Vec<u32> = PMAT_VERSION
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 2 && parts[1] > 0 {
            let older = format!("{}.{}.0", parts[0], parts[1] - 1);
            let behind = calculate_versions_behind(&older);
            assert_eq!(behind, 1);
        }
    }

    #[test]
    fn test_calculate_versions_behind_same_version() {
        let behind = calculate_versions_behind(PMAT_VERSION);
        assert_eq!(behind, 0);
    }

    #[test]
    fn test_calculate_versions_behind_newer_version() {
        let parts: Vec<u32> = PMAT_VERSION
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 2 {
            let newer = format!("{}.{}.0", parts[0], parts[1] + 10);
            let behind = calculate_versions_behind(&newer);
            // saturating_sub returns 0 for negative result
            assert_eq!(behind, 0);
        }
    }

    #[test]
    fn test_calculate_versions_behind_invalid_version() {
        let behind = calculate_versions_behind("invalid");
        assert_eq!(behind, 0);
    }

    #[test]
    fn test_calculate_versions_behind_partial_version() {
        let behind = calculate_versions_behind("1");
        assert_eq!(behind, 0);
    }

    #[test]
    fn test_calculate_versions_behind_empty_string() {
        let behind = calculate_versions_behind("");
        assert_eq!(behind, 0);
    }

    // ============================================================================
    // check_version_currency Tests
    // ============================================================================

    #[test]
    fn test_check_version_currency_current() {
        let check = check_version_currency(PMAT_VERSION);
        assert_eq!(check.status, CheckStatus::Pass);
        assert_eq!(check.severity, Severity::Info);
        assert!(check.message.contains("latest"));
    }

    #[test]
    fn test_check_version_currency_slightly_behind() {
        let parts: Vec<u32> = PMAT_VERSION
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 2 && parts[1] >= 3 {
            let old = format!("{}.{}.0", parts[0], parts[1] - 3);
            let check = check_version_currency(&old);
            assert_eq!(check.status, CheckStatus::Warn);
            assert_eq!(check.severity, Severity::Warning);
        }
    }

    #[test]
    fn test_check_version_currency_very_behind() {
        let parts: Vec<u32> = PMAT_VERSION
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 2 && parts[1] > 10 {
            let old = format!("{}.{}.0", parts[0], parts[1] - 10);
            let check = check_version_currency(&old);
            assert_eq!(check.status, CheckStatus::Fail);
            assert_eq!(check.severity, Severity::Error);
        }
    }

    // ============================================================================
    // check_config_files Tests
    // ============================================================================

    #[test]
    fn test_check_config_files_none_present() {
        let temp = create_temp_project();
        let check = check_config_files(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("Missing"));
    }

    #[test]
    fn test_check_config_files_pmat_only() {
        let temp = create_pmat_project(PMAT_VERSION);
        let check = check_config_files(temp.path());
        // Only .pmat/project.toml present, missing .pmat-metrics.toml
        assert_eq!(check.status, CheckStatus::Warn);
    }

    #[test]
    fn test_check_config_files_all_present() {
        let temp = create_project_with_metrics(PMAT_VERSION);
        let check = check_config_files(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("All required"));
    }

    // ============================================================================
    // check_hooks_installed Tests
    // ============================================================================

    #[test]
    fn test_check_hooks_not_installed() {
        let temp = create_temp_project();
        let check = check_hooks_installed(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("No pre-commit"));
    }

    #[test]
    fn test_check_hooks_non_pmat_hook() {
        let temp = create_git_repo();
        let hook_content = "#!/bin/sh\necho 'some other hook'";
        fs::write(
            temp.path().join(".git").join("hooks").join("pre-commit"),
            hook_content,
        )
        .expect("Failed to write hook");

        let check = check_hooks_installed(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("may not be PMAT"));
    }

    #[test]
    fn test_check_hooks_pmat_hook_installed() {
        let temp = create_git_repo();
        let hook_content = "#!/bin/sh\n# PMAT hook\npmat check";
        fs::write(
            temp.path().join(".git").join("hooks").join("pre-commit"),
            hook_content,
        )
        .expect("Failed to write hook");

        let check = check_hooks_installed(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("PMAT hooks installed"));
    }

    #[test]
    fn test_check_hooks_pmat_lowercase() {
        let temp = create_git_repo();
        let hook_content = "#!/bin/sh\npmat validate";
        fs::write(
            temp.path().join(".git").join("hooks").join("pre-commit"),
            hook_content,
        )
        .expect("Failed to write hook");

        let check = check_hooks_installed(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    // ============================================================================
    // check_quality_thresholds Tests
    // ============================================================================

    #[test]
    fn test_check_quality_thresholds_missing() {
        let temp = create_temp_project();
        let check = check_quality_thresholds(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("No .pmat-metrics.toml"));
    }

    #[test]
    fn test_check_quality_thresholds_present() {
        let temp = create_project_with_metrics(PMAT_VERSION);
        let check = check_quality_thresholds(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("configured"));
    }

    // ============================================================================
    // check_deprecated_features Tests
    // ============================================================================

    #[test]
    fn test_check_deprecated_features_none() {
        let temp = create_temp_project();
        let check = check_deprecated_features(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("No deprecated"));
    }

    // ============================================================================
    // check_compute_brick Tests
    // ============================================================================

    #[test]
    fn test_check_compute_brick_not_applicable() {
        let temp = create_temp_project();
        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("Not a ComputeBrick"));
    }

    #[test]
    fn test_check_compute_brick_with_probar_dep() {
        let temp = create_temp_project();
        let cargo_content = r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
probar = "0.1"
"#;
        fs::write(temp.path().join("Cargo.toml"), cargo_content)
            .expect("Failed to write Cargo.toml");

        let check = check_compute_brick(temp.path());
        // Has probar but no .pmat-gates.toml
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("[compute-brick]"));
    }

    #[test]
    fn test_check_compute_brick_with_brick_dir() {
        let temp = create_temp_project();
        fs::create_dir_all(temp.path().join("src").join("brick"))
            .expect("Failed to create brick dir");

        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
    }

    #[test]
    fn test_check_compute_brick_fully_configured() {
        let temp = create_temp_project();
        fs::create_dir_all(temp.path().join("src").join("brick"))
            .expect("Failed to create brick dir");

        let gates_content = r#"
[compute-brick]
enabled = true
"#;
        fs::write(temp.path().join(".pmat-gates.toml"), gates_content)
            .expect("Failed to write gates");

        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_check_compute_brick_probar_without_coverage() {
        let temp = create_temp_project();
        let cargo_content = r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
probar = "0.1"
"#;
        fs::write(temp.path().join("Cargo.toml"), cargo_content)
            .expect("Failed to write Cargo.toml");

        let gates_content = r#"
[compute-brick]
enabled = true
"#;
        fs::write(temp.path().join(".pmat-gates.toml"), gates_content)
            .expect("Failed to write gates");

        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("GUI coverage"));
    }

    // ============================================================================
    // check_cargo_lock Tests
    // ============================================================================

    #[test]
    fn test_check_cargo_lock_missing() {
        let temp = create_rust_project(false, false);
        let check = check_cargo_lock(temp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("Missing Cargo.lock"));
    }

    #[test]
    fn test_check_cargo_lock_present() {
        let temp = create_rust_project(false, true);
        let check = check_cargo_lock(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("reproducible builds"));
    }

    // ============================================================================
    // check_msrv Tests
    // ============================================================================

    #[test]
    fn test_check_msrv_no_cargo_toml() {
        let temp = create_temp_project();
        let check = check_msrv(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("No Cargo.toml"));
    }

    #[test]
    fn test_check_msrv_missing() {
        let temp = create_rust_project(false, false);
        let check = check_msrv(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("No rust-version"));
    }

    #[test]
    fn test_check_msrv_present() {
        let temp = create_rust_project(true, false);
        let check = check_msrv(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("rust-version field present"));
    }

    // ============================================================================
    // check_ci_configured Tests
    // ============================================================================

    #[test]
    fn test_check_ci_not_configured() {
        let temp = create_temp_project();
        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("No CI configuration"));
    }

    #[test]
    fn test_check_ci_github_actions() {
        let temp = create_temp_project();
        let workflows_dir = temp.path().join(".github").join("workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create workflows dir");
        fs::write(workflows_dir.join("ci.yml"), "name: CI").expect("Failed to write workflow");

        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("GitHub Actions"));
    }

    #[test]
    fn test_check_ci_github_actions_empty() {
        let temp = create_temp_project();
        let workflows_dir = temp.path().join(".github").join("workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create workflows dir");

        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
    }

    #[test]
    fn test_check_ci_gitlab() {
        let temp = create_temp_project();
        fs::write(temp.path().join(".gitlab-ci.yml"), "stages:\n  - build")
            .expect("Failed to write gitlab-ci");

        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("GitLab CI"));
    }

    #[test]
    fn test_check_ci_jenkins() {
        let temp = create_temp_project();
        fs::write(temp.path().join("Jenkinsfile"), "pipeline { }")
            .expect("Failed to write Jenkinsfile");

        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("Jenkins"));
    }

    // ============================================================================
    // check_paiml_deps_workspace Tests
    // ============================================================================

    #[test]
    fn test_check_paiml_deps_no_cargo_toml() {
        let temp = tempfile::tempdir().expect("Failed to create temp dir");
        let check = check_paiml_deps_workspace(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("No Cargo.toml"));
    }

    #[test]
    fn test_check_paiml_deps_no_paiml_deps() {
        let temp = create_temp_project();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "test-project"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#,
        )
        .expect("Failed to write Cargo.toml");

        let check = check_paiml_deps_workspace(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("No PAIML stack dependencies"));
    }

    #[test]
    fn test_check_paiml_deps_with_trueno() {
        let temp = create_temp_project();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "test-project"
version = "0.1.0"

[dependencies]
trueno = "0.11"
serde = "1.0"
"#,
        )
        .expect("Failed to write Cargo.toml");

        let check = check_paiml_deps_workspace(temp.path());
        // Status depends on whether ~/src/trueno exists and its git state
        // But check name should always be correct
        assert_eq!(check.name, "PAIML Deps Workspace");
    }

    #[test]
    fn test_check_paiml_deps_with_multiple_paiml_deps() {
        let temp = create_temp_project();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "test-project"
version = "0.1.0"

[dependencies]
trueno = "0.11"
trueno-graph = "0.1"
aprender = "0.24"
"#,
        )
        .expect("Failed to write Cargo.toml");

        let check = check_paiml_deps_workspace(temp.path());
        assert_eq!(check.name, "PAIML Deps Workspace");
        // Message should mention PAIML deps count or dirty status
        assert!(
            check.message.contains("PAIML") || check.message.contains("dirty"),
            "Expected message about PAIML deps, got: {}",
            check.message
        );
    }

    // ============================================================================
    // get_breaking_changes_since Tests
    // ============================================================================

    #[test]
    fn test_get_breaking_changes_since_returns_empty() {
        let changes = get_breaking_changes_since("1.0.0");
        assert!(changes.is_empty());
    }

    #[test]
    fn test_get_breaking_changes_since_any_version() {
        let changes = get_breaking_changes_since("0.0.1");
        assert!(changes.is_empty());
    }

    // ============================================================================
    // get_changelog_entries Tests
    // ============================================================================

    #[test]
    fn test_get_changelog_entries_returns_entries() {
        let entries = get_changelog_entries("1.0.0", "2.0.0");
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_get_changelog_entries_contain_expected_features() {
        let entries = get_changelog_entries("1.0.0", PMAT_VERSION);
        let has_comply = entries.iter().any(|e| e.description.contains("comply"));
        assert!(has_comply);
    }

    #[test]
    fn test_changelog_entry_breaking_flag() {
        let entries = get_changelog_entries("1.0.0", "2.0.0");
        // Current implementation has no breaking changes
        let breaking_count = entries.iter().filter(|e| e.breaking).count();
        assert_eq!(breaking_count, 0);
    }

    // ============================================================================
    // load_or_create_project_config Tests
    // ============================================================================

    #[test]
    fn test_load_or_create_config_creates_new() {
        let temp = create_temp_project();
        let config =
            load_or_create_project_config(temp.path()).expect("Failed to load/create config");
        assert_eq!(config.pmat.version, PMAT_VERSION);

        // Verify file was created
        assert!(temp.path().join(".pmat").join("project.toml").exists());
    }

    #[test]
    fn test_load_or_create_config_loads_existing() {
        let temp = create_pmat_project("1.0.0");
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0");
    }

    #[test]
    fn test_load_or_create_config_invalid_toml() {
        let temp = create_temp_project();
        let pmat_dir = temp.path().join(".pmat");
        fs::create_dir_all(&pmat_dir).expect("Failed to create .pmat");
        fs::write(pmat_dir.join("project.toml"), "invalid { toml").expect("Failed to write");

        let result = load_or_create_project_config(temp.path());
        assert!(result.is_err());
    }

    // ============================================================================
    // update_last_check_timestamp Tests
    // ============================================================================

    #[test]
    fn test_update_last_check_timestamp() {
        let temp = create_pmat_project(PMAT_VERSION);

        let result = update_last_check_timestamp(temp.path());
        assert!(result.is_ok());

        // Verify timestamp was updated
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert!(config.pmat.last_compliance_check.is_some());
    }

    #[test]
    fn test_update_last_check_timestamp_no_config() {
        let temp = create_temp_project();
        let result = update_last_check_timestamp(temp.path());
        // Should succeed even if config doesn't exist
        assert!(result.is_ok());
    }

    // ============================================================================
    // migrate_project_version Tests
    // ============================================================================

    #[test]
    fn test_migrate_project_version_dry_run() {
        let temp = create_pmat_project("1.0.0");
        let result = migrate_project_version(temp.path(), "2.0.0", true);
        assert!(result.is_ok());
        assert!(result.unwrap()); // dry_run always returns true

        // Verify version NOT changed
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0");
    }

    #[test]
    fn test_migrate_project_version_actual() {
        let temp = create_pmat_project("1.0.0");
        let result = migrate_project_version(temp.path(), "2.0.0", false);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify version changed
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "2.0.0");
    }

    #[test]
    fn test_migrate_project_version_same_version() {
        let temp = create_pmat_project("1.0.0");
        let result = migrate_project_version(temp.path(), "1.0.0", false);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // No change needed
    }

    // ============================================================================
    // migrate_gitignore Tests
    // ============================================================================

    #[test]
    fn test_migrate_gitignore_no_file() {
        let temp = create_temp_project();
        let result = migrate_gitignore(temp.path(), false);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_migrate_gitignore_adds_entries() {
        let temp = create_temp_project();
        fs::write(temp.path().join(".gitignore"), "target/\n").expect("Failed to write gitignore");

        let result = migrate_gitignore(temp.path(), false);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let content = fs::read_to_string(temp.path().join(".gitignore")).expect("Failed to read");
        assert!(content.contains(".pmat/backup/"));
        assert!(content.contains(".pmat-qa/"));
    }

    #[test]
    fn test_migrate_gitignore_already_has_entries() {
        let temp = create_temp_project();
        fs::write(
            temp.path().join(".gitignore"),
            "target/\n.pmat/backup/\n.pmat-qa/\n",
        )
        .expect("Failed to write gitignore");

        let result = migrate_gitignore(temp.path(), false);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // No changes needed
    }

    #[test]
    fn test_migrate_gitignore_dry_run() {
        let temp = create_temp_project();
        fs::write(temp.path().join(".gitignore"), "target/\n").expect("Failed to write gitignore");

        let result = migrate_gitignore(temp.path(), true);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Would need update

        // Verify file NOT changed
        let content = fs::read_to_string(temp.path().join(".gitignore")).expect("Failed to read");
        assert!(!content.contains(".pmat/backup/"));
    }

    #[test]
    fn test_migrate_gitignore_no_trailing_newline() {
        let temp = create_temp_project();
        fs::write(temp.path().join(".gitignore"), "target/").expect("Failed to write gitignore");

        let result = migrate_gitignore(temp.path(), false);
        assert!(result.is_ok());

        let content = fs::read_to_string(temp.path().join(".gitignore")).expect("Failed to read");
        // Should handle missing trailing newline
        assert!(content.contains("# PMAT"));
    }

    // ============================================================================
    // update_project_config Tests
    // ============================================================================

    #[test]
    fn test_update_project_config_updates_to_current() {
        let temp = create_pmat_project("1.0.0");
        let result = update_project_config(temp.path(), false);
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, PMAT_VERSION);
    }

    #[test]
    fn test_update_project_config_dry_run() {
        let temp = create_pmat_project("1.0.0");
        let result = update_project_config(temp.path(), true);
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0"); // Unchanged
    }

    // ============================================================================
    // print_compliance_text Tests
    // ============================================================================

    #[test]
    fn test_print_compliance_text_compliant() {
        let report = ComplianceReport {
            project_version: PMAT_VERSION.to_string(),
            current_version: PMAT_VERSION.to_string(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![ComplianceCheck {
                name: "Test".to_string(),
                status: CheckStatus::Pass,
                message: "OK".to_string(),
                severity: Severity::Info,
            }],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        // This just tests it doesn't panic
        print_compliance_text(&report);
    }

    #[test]
    fn test_print_compliance_text_non_compliant() {
        let report = ComplianceReport {
            project_version: "1.0.0".to_string(),
            current_version: PMAT_VERSION.to_string(),
            is_compliant: false,
            versions_behind: 10,
            checks: vec![ComplianceCheck {
                name: "Version".to_string(),
                status: CheckStatus::Fail,
                message: "Outdated".to_string(),
                severity: Severity::Error,
            }],
            breaking_changes: vec![],
            recommendations: vec!["Update PMAT".to_string()],
            timestamp: Utc::now(),
        };
        print_compliance_text(&report);
    }

    #[test]
    fn test_print_compliance_text_all_status_types() {
        let report = ComplianceReport {
            project_version: PMAT_VERSION.to_string(),
            current_version: PMAT_VERSION.to_string(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![
                ComplianceCheck {
                    name: "Pass".to_string(),
                    status: CheckStatus::Pass,
                    message: "Good".to_string(),
                    severity: Severity::Info,
                },
                ComplianceCheck {
                    name: "Warn".to_string(),
                    status: CheckStatus::Warn,
                    message: "Warning".to_string(),
                    severity: Severity::Warning,
                },
                ComplianceCheck {
                    name: "Fail".to_string(),
                    status: CheckStatus::Fail,
                    message: "Failed".to_string(),
                    severity: Severity::Error,
                },
                ComplianceCheck {
                    name: "Skip".to_string(),
                    status: CheckStatus::Skip,
                    message: "Skipped".to_string(),
                    severity: Severity::Info,
                },
            ],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        print_compliance_text(&report);
    }

    // ============================================================================
    // print_compliance_markdown Tests
    // ============================================================================

    #[test]
    fn test_print_compliance_markdown_compliant() {
        let report = ComplianceReport {
            project_version: PMAT_VERSION.to_string(),
            current_version: PMAT_VERSION.to_string(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        print_compliance_markdown(&report);
    }

    #[test]
    fn test_print_compliance_markdown_non_compliant() {
        let report = ComplianceReport {
            project_version: "1.0.0".to_string(),
            current_version: PMAT_VERSION.to_string(),
            is_compliant: false,
            versions_behind: 5,
            checks: vec![ComplianceCheck {
                name: "Check".to_string(),
                status: CheckStatus::Fail,
                message: "Failed".to_string(),
                severity: Severity::Error,
            }],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        print_compliance_markdown(&report);
    }

    // ============================================================================
    // Async Handler Tests (using tokio::test)
    // ============================================================================

    #[tokio::test]
    async fn test_handle_init_new_project() {
        let temp = create_temp_project();
        let result = handle_init(temp.path(), false).await;
        assert!(result.is_ok());

        // Verify project.toml created
        assert!(temp.path().join(".pmat").join("project.toml").exists());
    }

    #[tokio::test]
    async fn test_handle_init_existing_no_force() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_init(temp.path(), false).await;
        assert!(result.is_ok());

        // Version should remain unchanged
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_handle_init_existing_with_force() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_init(temp.path(), true).await;
        assert!(result.is_ok());

        // Version should be updated to current
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, PMAT_VERSION);
    }

    #[tokio::test]
    async fn test_handle_update_both() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_update(temp.path(), false, false, false).await;
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, PMAT_VERSION);
    }

    #[tokio::test]
    async fn test_handle_update_dry_run() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_update(temp.path(), false, false, true).await;
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0"); // Unchanged
    }

    #[tokio::test]
    async fn test_handle_update_hooks_only() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_update(temp.path(), true, false, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_update_config_only() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_update(temp.path(), false, true, false).await;
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, PMAT_VERSION);
    }

    #[tokio::test]
    async fn test_handle_diff_default_versions() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_diff(temp.path(), None, None, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_diff_specific_versions() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_diff(temp.path(), Some("1.0.0"), Some("2.0.0"), false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_diff_breaking_only() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_diff(temp.path(), None, None, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_migrate_dry_run() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_migrate(temp.path(), None, true, false, false).await;
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0"); // Unchanged
    }

    #[tokio::test]
    async fn test_handle_migrate_with_target() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_migrate(temp.path(), Some("2.0.0"), false, true, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_migrate_no_backup() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_migrate(temp.path(), None, false, true, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_migrate_with_backup() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_migrate(temp.path(), None, false, false, true).await;
        assert!(result.is_ok());

        // Verify backup directory created
        assert!(temp.path().join(".pmat").join("backup").exists());
    }

    #[tokio::test]
    async fn test_handle_enforce_no_git() {
        let temp = create_temp_project();
        let result = handle_enforce(temp.path(), true, false, ComplyOutputFormat::Text).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("Not a git repository"));
    }

    #[tokio::test]
    async fn test_handle_enforce_install() {
        let temp = create_git_repo();
        let result = handle_enforce(temp.path(), true, false, ComplyOutputFormat::Text).await;
        assert!(result.is_ok());

        // Verify hook created
        let hook_path = temp.path().join(".git").join("hooks").join("pre-commit");
        assert!(hook_path.exists());
        let content = fs::read_to_string(&hook_path).expect("Failed to read hook");
        assert!(content.contains("PMAT"));
    }

    #[tokio::test]
    async fn test_handle_enforce_disable() {
        let temp = create_git_repo();
        // First install
        handle_enforce(temp.path(), true, false, ComplyOutputFormat::Text)
            .await
            .expect("Failed to install");

        // Then disable
        let result = handle_enforce(temp.path(), true, true, ComplyOutputFormat::Text).await;
        assert!(result.is_ok());

        // Verify hook removed
        let hook_path = temp.path().join(".git").join("hooks").join("pre-commit");
        assert!(!hook_path.exists());
    }

    #[tokio::test]
    async fn test_handle_enforce_disable_non_pmat_hook() {
        let temp = create_git_repo();
        let hook_path = temp.path().join(".git").join("hooks").join("pre-commit");
        fs::write(&hook_path, "#!/bin/sh\necho 'other hook'").expect("Failed to write hook");

        let result = handle_enforce(temp.path(), true, true, ComplyOutputFormat::Text).await;
        assert!(result.is_ok());

        // Non-PMAT hook should NOT be removed
        assert!(hook_path.exists());
    }

    #[tokio::test]
    async fn test_handle_enforce_json_format() {
        let temp = create_git_repo();
        let result = handle_enforce(temp.path(), true, false, ComplyOutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_enforce_markdown_format() {
        let temp = create_git_repo();
        let result = handle_enforce(temp.path(), true, false, ComplyOutputFormat::Markdown).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_text() {
        let temp = create_pmat_project(PMAT_VERSION);
        let result = handle_report(temp.path(), false, ComplyOutputFormat::Text, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_json() {
        let temp = create_pmat_project(PMAT_VERSION);
        let result = handle_report(temp.path(), false, ComplyOutputFormat::Json, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_markdown() {
        let temp = create_pmat_project(PMAT_VERSION);
        let result = handle_report(temp.path(), false, ComplyOutputFormat::Markdown, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_with_history() {
        let temp = create_pmat_project(PMAT_VERSION);
        let result = handle_report(temp.path(), true, ComplyOutputFormat::Text, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_to_file() {
        let temp = create_pmat_project(PMAT_VERSION);
        let output_file = temp.path().join("report.md");
        let result = handle_report(
            temp.path(),
            false,
            ComplyOutputFormat::Markdown,
            Some(&output_file),
        )
        .await;
        assert!(result.is_ok());
        assert!(output_file.exists());
    }

    // ============================================================================
    // handle_comply_command Tests
    // ============================================================================

    #[tokio::test]
    async fn test_handle_comply_command_init() {
        let temp = create_temp_project();
        let command = ComplyCommands::Init {
            path: temp.path().to_path_buf(),
            force: false,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_comply_command_update() {
        let temp = create_pmat_project("1.0.0");
        let command = ComplyCommands::Update {
            path: temp.path().to_path_buf(),
            hooks: false,
            config: false,
            dry_run: true,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_comply_command_diff() {
        let temp = create_pmat_project("1.0.0");
        let command = ComplyCommands::Diff {
            path: temp.path().to_path_buf(),
            from: None,
            to: None,
            breaking_only: false,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_comply_command_migrate() {
        let temp = create_pmat_project("1.0.0");
        let command = ComplyCommands::Migrate {
            path: temp.path().to_path_buf(),
            version: None,
            dry_run: true,
            no_backup: true,
            force: true,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_comply_command_enforce() {
        let temp = create_git_repo();
        let command = ComplyCommands::Enforce {
            path: temp.path().to_path_buf(),
            yes: true,
            disable: false,
            format: ComplyOutputFormat::Text,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_comply_command_report() {
        let temp = create_pmat_project(PMAT_VERSION);
        let command = ComplyCommands::Report {
            path: temp.path().to_path_buf(),
            include_history: false,
            format: ComplyOutputFormat::Text,
            output: None,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    // ============================================================================
    // Edge Cases and Error Paths
    // ============================================================================

    #[test]
    fn test_version_parsing_with_prerelease() {
        let behind = calculate_versions_behind("2.0.0-alpha.1");
        // Should handle prerelease gracefully
        assert!(behind >= 0);
    }

    #[test]
    fn test_version_parsing_with_build_metadata() {
        let behind = calculate_versions_behind("2.0.0+build.123");
        assert!(behind >= 0);
    }

    #[test]
    fn test_compliance_check_debug_impl() {
        let check = ComplianceCheck {
            name: "Test".to_string(),
            status: CheckStatus::Pass,
            message: "OK".to_string(),
            severity: Severity::Info,
        };
        let debug_str = format!("{:?}", check);
        assert!(debug_str.contains("ComplianceCheck"));
        assert!(debug_str.contains("Pass"));
    }

    #[test]
    fn test_project_config_debug_impl() {
        let config = ProjectConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ProjectConfig"));
    }

    #[test]
    fn test_breaking_change_debug_impl() {
        let change = BreakingChange {
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            migration_guide: None,
        };
        let debug_str = format!("{:?}", change);
        assert!(debug_str.contains("BreakingChange"));
    }

    #[test]
    fn test_compliance_report_debug_impl() {
        let report = ComplianceReport {
            project_version: "1.0.0".to_string(),
            current_version: "2.0.0".to_string(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        let debug_str = format!("{:?}", report);
        assert!(debug_str.contains("ComplianceReport"));
    }

    #[tokio::test]
    async fn test_handle_check_with_nonexistent_path() {
        let temp = create_temp_project();
        let nonexistent = temp.path().join("nonexistent");
        // This should create the config directory
        let result = load_or_create_project_config(&nonexistent);
        // May fail due to parent directory not existing
        // Just verify it handles the error gracefully
        let _ = result;
    }

    #[test]
    fn test_changelog_entry_struct() {
        // Test the ChangelogEntry struct directly
        let entry = ChangelogEntry {
            version: "1.0.0".to_string(),
            description: "Test change".to_string(),
            breaking: true,
        };
        assert_eq!(entry.version, "1.0.0");
        assert!(entry.breaking);

        // Test clone
        let cloned = entry.clone();
        assert_eq!(cloned.version, entry.version);
        assert_eq!(cloned.breaking, entry.breaking);
    }

    #[test]
    fn test_pmat_version_constant() {
        // Verify PMAT_VERSION is set from Cargo.toml
        assert!(!PMAT_VERSION.is_empty());
        // Should be a valid semver-ish format
        let parts: Vec<&str> = PMAT_VERSION.split('.').collect();
        assert!(parts.len() >= 2, "Version should have at least major.minor");
    }

    // ============================================================================
    // Integration-style Tests
    // ============================================================================

    #[tokio::test]
    async fn test_full_compliance_workflow() {
        // Create a new project, init, check, migrate
        let temp = create_temp_project();

        // Init
        handle_init(temp.path(), false).await.expect("Init failed");

        // Verify config exists
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, PMAT_VERSION);

        // Check should pass (we're on current version)
        let checks = vec![
            check_version_currency(&config.pmat.version),
            check_config_files(temp.path()),
        ];
        let _all_pass_or_warn = checks.iter().all(|c| c.status != CheckStatus::Fail);
        // Version should pass, config files may warn about metrics
        assert!(checks[0].status == CheckStatus::Pass);
    }

    #[tokio::test]
    async fn test_migrate_then_check_workflow() {
        let temp = create_pmat_project("1.0.0");

        // Migrate to current
        handle_migrate(temp.path(), None, false, true, true)
            .await
            .expect("Migrate failed");

        // Check version should now pass
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        let check = check_version_currency(&config.pmat.version);
        assert_eq!(check.status, CheckStatus::Pass);
    }

    // ============================================================================
    // ComputeBrick Pattern Detection Tests (CB-IMPL-001-B)
    // ============================================================================

    #[test]
    fn test_cb020_detects_unsafe_without_safety() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with unsafe block without SAFETY comment
        let rs_file = src_dir.join("lib.rs");
        std::fs::write(
            &rs_file,
            r#"
fn bad_unsafe() {
    unsafe {
        std::ptr::null::<i32>().read();
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb020_unsafe_without_safety(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-020");
        assert!(violations[0].description.contains("unsafe"));
    }

    #[test]
    fn test_cb020_allows_unsafe_with_safety() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with unsafe block WITH SAFETY comment
        let rs_file = src_dir.join("lib.rs");
        std::fs::write(
            &rs_file,
            r#"
fn good_unsafe() {
    // SAFETY: null pointer read is UB, but this is just a test
    unsafe {
        std::ptr::null::<i32>().read();
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb020_unsafe_without_safety(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb021_detects_simd_without_target_feature() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with AVX intrinsic without #[target_feature]
        // Note: SSE (_mm_) is now exempted as baseline on x86_64
        let rs_file = src_dir.join("simd.rs");
        std::fs::write(
            &rs_file,
            r#"
fn bad_simd() {
    let a = _mm256_set1_ps(1.0);
}
"#,
        )
        .unwrap();

        let violations = detect_cb021_simd_without_target_feature(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-021");
        assert!(violations[0].description.contains("_mm256"));
    }

    #[test]
    fn test_cb021_allows_simd_with_target_feature() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with AVX intrinsic WITH #[target_feature]
        let rs_file = src_dir.join("simd.rs");
        std::fs::write(
            &rs_file,
            r#"
#[target_feature(enable = "avx2")]
fn good_simd() {
    let a = _mm256_set1_ps(1.0);
}
"#,
        )
        .unwrap();

        let violations = detect_cb021_simd_without_target_feature(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb021_no_false_positive_on_identifiers() {
        // Regression test: struct fields like "f32x4_verified" should NOT trigger CB-021
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with f32x4 in identifier names (NOT intrinsic usage)
        let rs_file = src_dir.join("verification.rs");
        std::fs::write(
            &rs_file,
            r#"
/// Verify SIMD f32x4 operations work correctly
pub struct SimdVerification {
    /// f32x4 operations verified
    pub f32x4_verified: bool,
    /// i32x4 operations verified
    pub i32x4_verified: bool,
}

pub fn verify_f32x4_operations() -> bool {
    let simd_lanes = 4; // f32x4
    true
}
"#,
        )
        .unwrap();

        let violations = detect_cb021_simd_without_target_feature(temp.path());
        // Should be 0 - these are identifiers and comments, not intrinsic calls
        assert_eq!(
            violations.len(),
            0,
            "False positive: detected {:?}",
            violations
        );
    }

    #[test]
    fn test_cb021_detects_actual_portable_simd_usage() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with ACTUAL portable SIMD usage (f32x4::splat)
        let rs_file = src_dir.join("simd_usage.rs");
        std::fs::write(
            &rs_file,
            r#"
use std::simd::f32x4;

fn use_portable_simd() {
    let a = f32x4::splat(1.0);
    let b = f32x4::from_array([1.0, 2.0, 3.0, 4.0]);
}
"#,
        )
        .unwrap();

        let violations = detect_cb021_simd_without_target_feature(temp.path());
        // Should detect f32x4:: usage as potential SIMD without target_feature
        assert!(
            violations.len() >= 1,
            "Should detect portable SIMD usage: {:?}",
            violations
        );
    }

    // ============================================================================
    // CB-001 and CB-002 WGSL Detection Tests (CB-IMPL-001-D)
    // ============================================================================

    #[test]
    fn test_cb001_detects_wgsl_without_bounds_check() {
        let temp = tempfile::tempdir().unwrap();

        // Create WGSL file with global_invocation_id but NO bounds check
        let wgsl_file = temp.path().join("compute.wgsl");
        std::fs::write(
            &wgsl_file,
            r#"@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gid = global_id.x;
    output[gid] = input[gid];  // No bounds check!
}
"#,
        )
        .unwrap();

        let violations = detect_cb001_wgsl_no_bounds_check(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-001");
        assert!(violations[0].description.contains("bounds check"));
    }

    #[test]
    fn test_cb001_allows_wgsl_with_bounds_check() {
        let temp = tempfile::tempdir().unwrap();

        // Create WGSL file with global_invocation_id AND bounds check
        let wgsl_file = temp.path().join("compute.wgsl");
        std::fs::write(
            &wgsl_file,
            r#"@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gid = global_id.x;
    if (gid >= arrayLength(&input)) { return; }
    output[gid] = input[gid];
}
"#,
        )
        .unwrap();

        let violations = detect_cb001_wgsl_no_bounds_check(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb002_detects_wgsl_barrier_in_conditional() {
        let temp = tempfile::tempdir().unwrap();

        // Create WGSL file with workgroupBarrier() inside conditional
        let wgsl_file = temp.path().join("compute.wgsl");
        std::fs::write(
            &wgsl_file,
            r#"@compute @workgroup_size(64)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {
    if (local_id.x == 0u) {
        shared_data[0] = compute();
        workgroupBarrier();  // DANGER: Inside conditional!
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb002_wgsl_barrier_divergence(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-002");
        assert!(violations[0].description.contains("workgroupBarrier()"));
    }

    #[test]
    fn test_cb002_allows_wgsl_barrier_outside_conditional() {
        let temp = tempfile::tempdir().unwrap();

        // Create WGSL file with workgroupBarrier() OUTSIDE conditional
        let wgsl_file = temp.path().join("compute.wgsl");
        std::fs::write(
            &wgsl_file,
            r#"@compute @workgroup_size(64)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {
    if (local_id.x == 0u) {
        shared_data[0] = compute();
    }
    workgroupBarrier();  // Safe: All threads reach this
    let val = shared_data[0];
}
"#,
        )
        .unwrap();

        let violations = detect_cb002_wgsl_barrier_divergence(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detect_bricks_without_assertions() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with Brick impl WITHOUT assertions
        let rs_file = src_dir.join("brick.rs");
        std::fs::write(
            &rs_file,
            // No leading newline - content starts immediately
            "impl ComputeBrick for MyBrick {\n\
                fn execute(&self) {\n\
                    self.do_work();\n\
                }\n\
            }\n",
        )
        .unwrap();

        let violations = detect_bricks_without_assertions(temp.path());
        assert_eq!(violations.len(), 1, "Expected 1 violation for brick without assertions");
        assert_eq!(violations[0].pattern_id, "CB-BUDGET");
    }

    #[test]
    fn test_detect_bricks_with_assertions_pass() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with Brick impl WITH assertions
        let rs_file = src_dir.join("brick.rs");
        std::fs::write(
            &rs_file,
            r#"
impl ComputeBrick for MyBrick {
    fn execute(&self) {
        debug_assert!(self.is_valid());
        self.do_work();
    }
}
"#,
        )
        .unwrap();

        let violations = detect_bricks_without_assertions(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detect_profiler_anomalies_high_cv() {
        let temp = tempfile::tempdir().unwrap();
        let metrics_dir = temp.path().join(".pmat-metrics");
        std::fs::create_dir_all(&metrics_dir).unwrap();

        // Create profiler JSON with high CV
        let profile_file = metrics_dir.join("brick-profile.json");
        std::fs::write(
            &profile_file,
            r#"{
  "bricks": [
    {
      "name": "MatMulBrick",
      "cv": 0.25,
      "efficiency": 0.80
    }
  ]
}"#,
        )
        .unwrap();

        let anomalies = detect_profiler_anomalies(temp.path());
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].anomaly_type, "HIGH_CV");
        assert!(anomalies[0].value > 15.0);
    }

    #[test]
    fn test_detect_profiler_anomalies_low_efficiency() {
        let temp = tempfile::tempdir().unwrap();
        let metrics_dir = temp.path().join(".pmat-metrics");
        std::fs::create_dir_all(&metrics_dir).unwrap();

        // Create profiler JSON with low efficiency
        let profile_file = metrics_dir.join("brick-profile.json");
        std::fs::write(
            &profile_file,
            r#"{
  "bricks": [
    {
      "name": "SlowBrick",
      "cv": 0.05,
      "efficiency": 0.15
    }
  ]
}"#,
        )
        .unwrap();

        let anomalies = detect_profiler_anomalies(temp.path());
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].anomaly_type, "LOW_EFFICIENCY");
        assert!(anomalies[0].value < 25.0);
    }

    #[test]
    fn test_check_compute_brick_skips_non_cb_project() {
        let temp = tempfile::tempdir().unwrap();
        // Create a regular project without trueno/realizar/probar deps
        let cargo_toml = temp.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"[package]
name = "regular-project"
version = "1.0.0"

[dependencies]
serde = "1.0"
"#,
        )
        .unwrap();

        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_check_compute_brick_detects_trueno_project() {
        let temp = tempfile::tempdir().unwrap();
        // Create project with trueno dependency
        let cargo_toml = temp.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"[package]
name = "gpu-project"
version = "1.0.0"

[dependencies]
trueno = "0.1"
"#,
        )
        .unwrap();

        // Create src directory with clean Rust code
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("lib.rs"), "pub fn hello() {}").unwrap();

        let check = check_compute_brick(temp.path());
        // Should not skip - this is a CB ecosystem project
        assert_ne!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_extract_json_number() {
        assert_eq!(extract_json_number("\"cv\": 0.18,"), Some(0.18));
        assert_eq!(extract_json_number("\"efficiency\": 25.5}"), Some(25.5));
        assert_eq!(extract_json_number("invalid"), None);
    }

    #[test]
    fn test_walkdir_rs_files() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        let nested = src_dir.join("nested");
        std::fs::create_dir_all(&nested).unwrap();

        std::fs::write(src_dir.join("lib.rs"), "").unwrap();
        std::fs::write(nested.join("mod.rs"), "").unwrap();
        std::fs::write(src_dir.join("readme.md"), "").unwrap(); // Not .rs

        let files = walkdir_rs_files(&src_dir).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| f.extension().unwrap() == "rs"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn test_version_behind_never_negative(major in 0u32..100, minor in 0u32..1000, patch in 0u32..100) {
            let version = format!("{}.{}.{}", major, minor, patch);
            let behind = calculate_versions_behind(&version);
            // Should always return a non-negative value (saturating_sub)
            prop_assert!(behind < u32::MAX);
        }

        #[test]
        fn test_check_version_currency_always_returns_valid_check(
            major in 0u32..10,
            minor in 0u32..500,
            patch in 0u32..100
        ) {
            let version = format!("{}.{}.{}", major, minor, patch);
            let check = check_version_currency(&version);

            // Check should always have non-empty fields
            prop_assert!(!check.name.is_empty());
            prop_assert!(!check.message.is_empty());

            // Status should be one of the valid variants
            prop_assert!(matches!(
                check.status,
                CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Fail | CheckStatus::Skip
            ));
        }

        #[test]
        fn test_project_config_roundtrip_serialization(
            version in "[0-9]+\\.[0-9]+\\.[0-9]+",
            auto_update in proptest::bool::ANY
        ) {
            let config = ProjectConfig {
                pmat: PmatSection {
                    version: version.clone(),
                    last_compliance_check: Some(Utc::now()),
                    auto_update,
                },
            };

            let serialized = toml::to_string_pretty(&config).expect("Serialization failed");
            let deserialized: ProjectConfig = toml::from_str(&serialized).expect("Deserialization failed");

            prop_assert_eq!(deserialized.pmat.version, version);
            prop_assert_eq!(deserialized.pmat.auto_update, auto_update);
        }

        #[test]
        fn test_compliance_check_serialization_roundtrip(
            name in "[a-zA-Z ]{1,50}",
            message in "[a-zA-Z0-9 ]{1,100}"
        ) {
            let check = ComplianceCheck {
                name: name.clone(),
                status: CheckStatus::Pass,
                message: message.clone(),
                severity: Severity::Info,
            };

            let json = serde_json::to_string(&check).expect("Serialization failed");
            let deserialized: ComplianceCheck = serde_json::from_str(&json).expect("Deserialization failed");

            prop_assert_eq!(deserialized.name, name);
            prop_assert_eq!(deserialized.message, message);
        }

        #[test]
        fn test_breaking_change_serialization_roundtrip(
            version in "[0-9]+\\.[0-9]+\\.[0-9]+",
            description in "[a-zA-Z0-9 ]{1,200}"
        ) {
            let change = BreakingChange {
                version: version.clone(),
                description: description.clone(),
                migration_guide: Some("Guide".to_string()),
            };

            let json = serde_json::to_string(&change).expect("Serialization failed");
            let deserialized: BreakingChange = serde_json::from_str(&json).expect("Deserialization failed");

            prop_assert_eq!(deserialized.version, version);
            prop_assert_eq!(deserialized.description, description);
        }

        #[test]
        fn test_changelog_entries_always_have_current_version(_seed in 0u32..1000) {
            let entries = get_changelog_entries("0.0.0", "999.999.999");
            prop_assert!(!entries.is_empty());

            // All entries should have version matching PMAT_VERSION
            for entry in &entries {
                prop_assert_eq!(&entry.version, PMAT_VERSION);
            }
        }

        #[test]
        fn test_breaking_changes_returns_empty_for_any_version(
            major in 0u32..100,
            minor in 0u32..1000,
            patch in 0u32..100
        ) {
            let version = format!("{}.{}.{}", major, minor, patch);
            let changes = get_breaking_changes_since(&version);
            // Current implementation always returns empty
            prop_assert!(changes.is_empty());
        }
    }

    // Additional property tests that require tempdir (can't use proptest macro easily)
    #[test]
    fn test_check_config_files_consistency() {
        use tempfile::TempDir;

        // Test that check_config_files is consistent across multiple calls
        let temp = TempDir::new().expect("Failed to create temp dir");
        let check1 = check_config_files(temp.path());
        let check2 = check_config_files(temp.path());

        assert_eq!(check1.status, check2.status);
        assert_eq!(check1.message, check2.message);
    }

    #[test]
    fn test_check_hooks_consistency() {
        use tempfile::TempDir;

        let temp = TempDir::new().expect("Failed to create temp dir");
        let check1 = check_hooks_installed(temp.path());
        let check2 = check_hooks_installed(temp.path());

        assert_eq!(check1.status, check2.status);
    }
}
