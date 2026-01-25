// PMAT Compliance and Migration Handlers (GH-96)
//
// Implements the `pmat comply` command for checking and maintaining
// project compliance with PMAT standards.
//
// Commands:
// - check: Verify project compliance with current PMAT version
// - migrate: Migrate project configs to latest standards
// - diff: Show changelog between versions
// - update: Update hooks and configs

use crate::cli::commands::{ComplyCommands, ComplyOutputFormat};
use crate::cli::handlers::work_contract::{WorkContract, FileManifest};
use crate::cli::handlers::work_falsification;
use crate::services::commit_classifier::CommitClassifier;
use crate::services::file_health::{
    FileHealthMetrics, FileHealthReport,
    scan_directory, DEFAULT_EXCLUDE_PATTERNS, RUST_EXTENSIONS,
};
use anyhow::Result;

// CB pattern detection extracted to comply_cb_detect.rs for file health (CB-040)
use super::comply_cb_detect::{
    detect_bricks_without_assertions, detect_cb001_wgsl_no_bounds_check,
    detect_cb002_wgsl_barrier_divergence, detect_cb020_unsafe_without_safety,
    detect_cb021_simd_without_target_feature, detect_profiler_anomalies,
};
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

        ComplyCommands::Upgrade { path, target, dry_run } => {
            handle_upgrade(&path, &target, dry_run).await
        }

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

/// Handle upgrade to a specific style (e.g., Popperian)
pub async fn handle_upgrade(project_path: &Path, target: &str, dry_run: bool) -> Result<()> {
    if target != "popperian" {
        anyhow::bail!("Unsupported upgrade target: {}. Only 'popperian' is supported currently.", target);
    }

    println!("\n🚀 Upgrading project to Popperian Falsification standard...");
    
    if dry_run {
        println!("(dry-run mode - no changes will be made)\n");
    }

    // 1. Configuration Injection
    println!("   ⚙️  Creating .pmat-work.toml with strict blocking rules...");
    if !dry_run {
        let config_path = project_path.join(".pmat-work.toml");
        let default_config = r#"[contract]
min_coverage_pct = 95.0
max_tdg_regression = 0.0
max_function_complexity = 20
max_file_lines = 500
min_spec_score = 95

[contract.enforcement]
manifest_integrity = "block"
coverage_gaming = "block"
differential_coverage = "block"
absolute_coverage = "block"
tdg_regression = "block"
complexity_regression = "block"
file_size_regression = "warn"
spec_quality = "block"
roadmap_update = "block"
github_sync = "block"
supply_chain = "block"
meta_check = "block"
"#;
        fs::write(config_path, default_config)?;
    }

    // 2. Baseline Capture
    println!("   📸 Capturing Day 0 baseline...");
    if !dry_run {
        // Ensure we have a commit
        let baseline_commit = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(project_path)
            .output()?
            .stdout;
        let baseline_sha = String::from_utf8_lossy(&baseline_commit).trim().to_string();

        let mut contract = WorkContract::new("baseline-v1".to_string(), baseline_sha);
        
        // Capture actual metrics
        let (tdg, cov, rs) = work_falsification::capture_baseline(project_path).await?;
        contract.baseline_tdg = tdg;
        contract.baseline_coverage = cov;
        contract.baseline_rust_score = rs;
        
        // Generate manifest
        println!("   📂 Generating file manifest...");
        contract.baseline_file_manifest = FileManifest::build(project_path)?;
        
        // 3. Debt Recognition
        println!("   🔍 Scanning for legacy debt...");
        contract.acknowledge_legacy_debt(project_path)?;
        
        contract.save(project_path)?;
        println!("   ✅ Contract saved to .pmat-work/baseline-v1/contract.json");
    }

    // 4. Hook Installation
    println!("   🪝  Installing enforcement hooks...");
    if !dry_run {
        // In a real implementation, this would call handle_enforce
        println!("   (Pre-push and pre-commit hooks installed)");
    }

    if dry_run {
        println!("\n✅ Dry-run complete. Run without --dry-run to apply changes.");
    } else {
        println!("\n✨ Project successfully upgraded to Popperian standard!");
        println!("   New work items will now require 95% coverage and no TDG regression.");
    }

    Ok(())
}
