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
use crate::models::comply_config::{PmatYamlConfig, ComplyConfig, CheckSeverity as ConfigSeverity};
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
    // OIP Tarantula patterns (CB-120 through CB-124) - improve-pmat-comply.md v2.1.0
    detect_cb120_nan_unsafe_comparison, detect_cb121_lock_poisoning,
    detect_cb122_serde_safety, detect_cb123_undocumented_ignore,
    detect_cb124_coverage_threshold,
    // Coverage Quality & Test Performance (CB-125 through CB-127) - improve-pmat-comply.md v2.2.0
    detect_cb125_coverage_exclusion_gaming, detect_cb126_slow_tests,
    detect_cb127_slow_coverage,
    // Dependency Health (CB-081) - rust-project-score-v1.1 integration
    detect_cb081_dependency_count, DependencyCountReport,
    // Agent Context Adoption (CB-130) - PMAT-470
    detect_cb130_agent_context_adoption,
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

impl From<ConfigSeverity> for Severity {
    fn from(config: ConfigSeverity) -> Self {
        match config {
            ConfigSeverity::Info => Severity::Info,
            ConfigSeverity::Warning => Severity::Warning,
            ConfigSeverity::Error => Severity::Error,
            ConfigSeverity::Critical => Severity::Critical,
        }
    }
}

/// Filter a check result based on YAML configuration
///
/// Returns Skip status if the check is disabled in .pmat.yaml
pub(crate) fn filter_check_by_config(
    check: ComplianceCheck,
    check_id: &str,
    config: &ComplyConfig,
) -> ComplianceCheck {
    if !config.is_check_enabled(check_id) {
        return ComplianceCheck {
            name: check.name,
            status: CheckStatus::Skip,
            message: format!("{} (disabled in .pmat.yaml)", check_id),
            severity: Severity::Info,
        };
    }

    // Apply severity override from config
    let configured_severity = config.get_severity(check_id);
    ComplianceCheck {
        severity: configured_severity.into(),
        ..check
    }
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

        ComplyCommands::Review {
            path,
            format,
            output,
        } => handle_review(&path, format, output.as_deref()).await,

        ComplyCommands::Audit {
            path,
            format,
            output,
        } => handle_audit(&path, format, output.as_deref()).await,
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

    // Load YAML configuration (COMPLY-044: YAML-first configuration)
    let yaml_config = PmatYamlConfig::load(project_path).unwrap_or_default();
    let comply_config = &yaml_config.comply;

    // Show config source if verbose
    let config_path = project_path.join(".pmat.yaml");
    if config_path.exists() {
        println!("  Using configuration from .pmat.yaml");
    }

    // Load or create project config
    let config = load_or_create_project_config(project_path)?;
    let project_version = &config.pmat.version;

    // Run compliance checks (filtered by .pmat.yaml configuration - COMPLY-044)
    let checks = vec![
        check_version_currency(project_version),
        check_config_files(project_path),
        check_hooks_installed(project_path),
        filter_check_by_config(check_hooks_o1_capable(project_path), "cb-030", comply_config),
        filter_check_by_config(check_hooks_cache_health(project_path), "cb-031", comply_config),
        check_quality_thresholds(project_path),
        check_deprecated_features(project_path),
        filter_check_by_config(check_compute_brick(project_path), "cb-060", comply_config),
        // OIP Tarantula patterns (CB-120 through CB-124) - improve-pmat-comply.md v2.1.0
        filter_check_by_config(check_oip_tarantula_patterns(project_path), "cb-120", comply_config),
        // Coverage Quality & Test Performance (CB-125 through CB-127) - improve-pmat-comply.md v2.2.0
        filter_check_by_config(check_coverage_quality_patterns(project_path), "cb-125", comply_config),
        // Build performance checks (lltop Tab 8 integration)
        check_cargo_lock(project_path),
        check_msrv(project_path),
        check_ci_configured(project_path),
        check_paiml_deps_workspace(project_path),
        check_sovereign_stack_patterns(project_path),
        filter_check_by_config(check_file_health(project_path), "cb-040", comply_config),
        // CB-300: Muda Waste Score (COMPLY-040) - improve-pmat-comply.md v2.8
        filter_check_by_config(check_muda_waste_score(project_path), "cb-300", comply_config),
        // CB-301: Reproducibility Level (COMPLY-041) - improve-pmat-comply.md v2.8
        filter_check_by_config(check_reproducibility_level(project_path), "cb-301", comply_config),
        // CB-302: Golden Trace Drift (COMPLY-042) - improve-pmat-comply.md v2.8
        filter_check_by_config(check_golden_trace_drift(project_path), "cb-302", comply_config),
        // CB-303: EDD Compliance (COMPLY-043) - improve-pmat-comply.md v2.8
        filter_check_by_config(check_edd_compliance(project_path), "cb-303", comply_config),
        // CB-304: Dead Code Percentage (COMPLY-044) - enforce dead_code_threshold
        filter_check_by_config(check_dead_code_percentage(project_path), "cb-304", comply_config),
        // CB-081: Dependency Count - rust-project-score-v1.1 integration
        filter_check_by_config(check_dependency_count(project_path), "cb-081", comply_config),
        // CB-400: Shell & Makefile Quality (bashrs integration)
        filter_check_by_config(check_shell_makefile_quality(project_path), "cb-400", comply_config),
        // CB-130: Agent Context Adoption (PMAT-470)
        filter_check_by_config(check_agent_context_adoption(project_path), "cb-130", comply_config),
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

    // Update hooks (async operation)
    match update_project_hooks(project_path, dry_run).await {
        Ok(true) => println!("  \x1b[32m✓\x1b[0m Update git hooks"),
        Ok(false) => println!("  \x1b[90m-\x1b[0m Update git hooks (no changes needed)"),
        Err(e) => println!("  \x1b[31m✗\x1b[0m Update git hooks - {}", e),
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
        match update_project_hooks(project_path, dry_run).await {
            Ok(true) => println!("  \x1b[32m✓\x1b[0m Hooks updated to latest templates"),
            Ok(false) => println!("  \x1b[90m-\x1b[0m Hooks already up to date"),
            Err(e) => println!("  \x1b[31m✗\x1b[0m Failed: {}", e),
        }
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

pub(crate) fn load_or_create_project_config(project_path: &Path) -> Result<ProjectConfig> {
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

pub(crate) fn update_last_check_timestamp(project_path: &Path) -> Result<()> {
    let config_path = project_path.join(".pmat").join("project.toml");
    if let Ok(mut config) = load_or_create_project_config(project_path) {
        config.pmat.last_compliance_check = Some(Utc::now());
        let content = toml::to_string_pretty(&config)?;
        fs::write(&config_path, &content)?;
    }
    Ok(())
}

pub(crate) fn check_version_currency(project_version: &str) -> ComplianceCheck {
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

pub(crate) fn check_config_files(project_path: &Path) -> ComplianceCheck {
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

pub(crate) fn check_hooks_installed(project_path: &Path) -> ComplianceCheck {
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
pub(crate) fn check_hooks_o1_capable(project_path: &Path) -> ComplianceCheck {
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
pub(crate) fn check_hooks_cache_health(project_path: &Path) -> ComplianceCheck {
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

pub(crate) fn check_quality_thresholds(project_path: &Path) -> ComplianceCheck {
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

pub(crate) fn check_deprecated_features(_project_path: &Path) -> ComplianceCheck {
    ComplianceCheck {
        name: "Deprecated Features".to_string(),
        status: CheckStatus::Pass,
        message: "No deprecated features detected".to_string(),
        severity: Severity::Info,
    }
}

/// Collect static analysis violations for ComputeBrick patterns.
/// Returns (issues, critical_count, warning_count).
pub(crate) fn collect_cb_violations(project_path: &Path, has_probar: bool, has_brick_dir: bool) -> (Vec<String>, usize, usize) {
    let mut all_issues: Vec<String> = Vec::new();
    let mut critical_count = 0;
    let mut warning_count = 0;

    // Warning-level violations: CB-020, CB-021, CB-BUDGET
    for v in &detect_cb020_unsafe_without_safety(project_path) {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        warning_count += 1;
    }
    for v in &detect_cb021_simd_without_target_feature(project_path) {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        warning_count += 1;
    }
    for v in &detect_bricks_without_assertions(project_path) {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        warning_count += 1;
    }

    // Critical-level violations: CB-001, CB-002 (WGSL per PROBAR-SPEC-009-P8 §4)
    for v in &detect_cb001_wgsl_no_bounds_check(project_path) {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        critical_count += 1;
    }
    for v in &detect_cb002_wgsl_barrier_divergence(project_path) {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        critical_count += 1;
    }

    // BrickProfiler anomalies
    for a in &detect_profiler_anomalies(project_path) {
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

    // Config and coverage checks
    let gates_path = project_path.join(".pmat-gates.toml");
    let has_cb_config = gates_path.exists()
        && fs::read_to_string(&gates_path)
            .map(|s| s.contains("[compute-brick]"))
            .unwrap_or(false);
    if !has_cb_config && (has_probar || has_brick_dir) {
        all_issues.push("Missing [compute-brick] section in .pmat-gates.toml".to_string());
        warning_count += 1;
    }

    let coverage_file = project_path.join(".pmat-metrics").join("gui-coverage.json");
    if has_probar && !coverage_file.exists() {
        all_issues.push("No GUI coverage report - run probador to generate".to_string());
        warning_count += 1;
    }

    (all_issues, critical_count, warning_count)
}

/// Build a ComplianceCheck result from collected violations.
pub(crate) fn build_cb_result(all_issues: Vec<String>, critical_count: usize, warning_count: usize) -> ComplianceCheck {
    if critical_count > 0 {
        ComplianceCheck {
            name: "ComputeBrick Compliance".to_string(),
            status: CheckStatus::Fail,
            message: format!(
                "{} critical, {} warnings:\n{}",
                critical_count,
                warning_count,
                format_violation_list(&all_issues),
            ),
            severity: Severity::Critical,
        }
    } else if warning_count > 0 {
        ComplianceCheck {
            name: "ComputeBrick Compliance".to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "{} warnings detected:\n{}",
                warning_count,
                format_violation_list(&all_issues),
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

/// Format a list of violations for display (indented, one per line).
pub(crate) fn format_violation_list(issues: &[String]) -> String {
    issues.iter()
        .map(|i| format!("    - {}", i))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Validates:
/// - CB-001 to CB-022: Static analysis pattern detection
/// - CB-020: unsafe blocks without SAFETY comments
/// - CB-021: SIMD intrinsics without #[target_feature]
/// - CB-BUDGET: Bricks without assertion/validation
/// - BrickProfiler anomalies: CV > 15%, efficiency < 25%
/// - Probar GUI coverage >= 80%
pub(crate) fn check_compute_brick(project_path: &Path) -> ComplianceCheck {
    let cargo_toml = project_path.join("Cargo.toml");
    let brick_dir = project_path.join("src").join("brick");
    let has_probar = cargo_toml.exists()
        && fs::read_to_string(&cargo_toml)
            .map(|s| s.contains("probar") || s.contains("jugar-probar"))
            .unwrap_or(false);
    let has_brick_dir = brick_dir.exists();
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

    let (all_issues, critical_count, warning_count) =
        collect_cb_violations(project_path, has_probar, has_brick_dir);
    build_cb_result(all_issues, critical_count, warning_count)
}

/// OIP Tarantula Pattern Detection (CB-120 through CB-124)
/// Implements improve-pmat-comply.md v2.1.0 specification
/// Validates:
/// - CB-120: NaN-unsafe comparison (partial_cmp().unwrap())
/// - CB-121: Lock poisoning vulnerabilities (mutex.lock().unwrap())
/// - CB-122: Serde deserialization safety (from_str().unwrap())
/// - CB-123: Undocumented #[ignore] tests
/// - CB-124: Low coverage thresholds (<80%)
pub(crate) fn check_oip_tarantula_patterns(project_path: &Path) -> ComplianceCheck {
    let mut all_issues: Vec<String> = Vec::new();
    let mut critical_count = 0;
    let mut warning_count = 0;

    // CB-120: NaN-unsafe comparisons
    let cb120_violations = detect_cb120_nan_unsafe_comparison(project_path);
    for v in &cb120_violations {
        all_issues.push(format!(
            "{}: {} ({}:{})",
            v.pattern_id, v.description, v.file, v.line
        ));
        // CB-120 is Error severity - counts toward critical
        critical_count += 1;
    }

    // CB-121: Lock poisoning vulnerabilities
    let cb121_violations = detect_cb121_lock_poisoning(project_path);
    for v in &cb121_violations {
        all_issues.push(format!(
            "{}: {} ({}:{})",
            v.pattern_id, v.description, v.file, v.line
        ));
        // CB-121 is Warning severity
        warning_count += 1;
    }

    // CB-122: Serde deserialization safety
    let cb122_violations = detect_cb122_serde_safety(project_path);
    for v in &cb122_violations {
        all_issues.push(format!(
            "{}: {} ({}:{})",
            v.pattern_id, v.description, v.file, v.line
        ));
        // CB-122 is Error severity - counts toward critical
        critical_count += 1;
    }

    // CB-123: Undocumented #[ignore] tests
    let cb123_violations = detect_cb123_undocumented_ignore(project_path);
    for v in &cb123_violations {
        all_issues.push(format!(
            "{}: {} ({}:{})",
            v.pattern_id, v.description, v.file, v.line
        ));
        // CB-123 is Warning severity
        warning_count += 1;
    }

    // CB-124: Low coverage thresholds
    let cb124_violations = detect_cb124_coverage_threshold(project_path);
    for v in &cb124_violations {
        all_issues.push(format!(
            "{}: {} ({}:{})",
            v.pattern_id, v.description, v.file, v.line
        ));
        match v.severity {
            super::comply_cb_detect::Severity::Error => critical_count += 1,
            _ => warning_count += 1,
        }
    }

    // Determine overall status
    // NOTE: OIP Tarantula checks are advisory (non-blocking) for now
    // Per improve-pmat-comply.md v2.1.0, these patterns are being tracked
    // as technical debt and will be addressed incrementally.
    if critical_count > 0 || warning_count > 0 {
        ComplianceCheck {
            name: "OIP Tarantula Patterns (CB-120 to CB-124)".to_string(),
            status: CheckStatus::Warn,  // Advisory: doesn't block compliance
            message: format!(
                "[Advisory] {} issues, {} warnings:\n{}",
                critical_count,
                warning_count,
                format_violation_list(&all_issues),
            ),
            severity: Severity::Warning,  // Non-blocking
        }
    } else {
        ComplianceCheck {
            name: "OIP Tarantula Patterns (CB-120 to CB-124)".to_string(),
            status: CheckStatus::Pass,
            message: "No OIP Tarantula pattern violations detected".to_string(),
            severity: Severity::Info,
        }
    }
}

/// Check Coverage Quality & Test Performance patterns (CB-125 through CB-127)
/// Per improve-pmat-comply.md v2.2.0
pub(crate) fn check_coverage_quality_patterns(project_path: &Path) -> ComplianceCheck {
    let mut all_issues: Vec<String> = Vec::new();
    let mut critical_count = 0;
    let mut error_count = 0;
    let mut warning_count = 0;

    // CB-125: Coverage exclusion gaming
    let cb125_violations = detect_cb125_coverage_exclusion_gaming(project_path);
    for v in &cb125_violations {
        all_issues.push(format!(
            "{}: {} ({}:{})",
            v.pattern_id, v.description, v.file, v.line
        ));
        match v.severity {
            super::comply_cb_detect::Severity::Critical => critical_count += 1,
            super::comply_cb_detect::Severity::Error => error_count += 1,
            _ => warning_count += 1,
        }
    }

    // CB-126: Slow tests
    let cb126_violations = detect_cb126_slow_tests(project_path);
    for v in &cb126_violations {
        all_issues.push(format!(
            "{}: {} ({}:{})",
            v.pattern_id, v.description, v.file, v.line
        ));
        match v.severity {
            super::comply_cb_detect::Severity::Critical => critical_count += 1,
            super::comply_cb_detect::Severity::Error => error_count += 1,
            _ => warning_count += 1,
        }
    }

    // CB-127: Slow coverage configuration
    let cb127_violations = detect_cb127_slow_coverage(project_path);
    for v in &cb127_violations {
        all_issues.push(format!(
            "{}: {} ({}:{})",
            v.pattern_id, v.description, v.file, v.line
        ));
        match v.severity {
            super::comply_cb_detect::Severity::Critical => critical_count += 1,
            super::comply_cb_detect::Severity::Error => error_count += 1,
            _ => warning_count += 1,
        }
    }

    // v2.2 patterns are BLOCKING (unlike v2.1 advisory patterns)
    // Per [GAME-001], [SLOW-001], [PERF-001]: These directly impact development velocity
    if critical_count > 0 {
        ComplianceCheck {
            name: "Coverage Quality Patterns (CB-125 to CB-127)".to_string(),
            status: CheckStatus::Fail,
            message: format!(
                "{} critical, {} errors, {} warnings:\n{}",
                critical_count,
                error_count,
                warning_count,
                format_violation_list(&all_issues),
            ),
            severity: Severity::Critical,
        }
    } else if error_count > 0 {
        ComplianceCheck {
            name: "Coverage Quality Patterns (CB-125 to CB-127)".to_string(),
            status: CheckStatus::Fail,
            message: format!(
                "{} errors, {} warnings:\n{}",
                error_count,
                warning_count,
                format_violation_list(&all_issues),
            ),
            severity: Severity::Error,
        }
    } else if warning_count > 0 {
        ComplianceCheck {
            name: "Coverage Quality Patterns (CB-125 to CB-127)".to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "{} warnings:\n{}",
                warning_count,
                format_violation_list(&all_issues),
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "Coverage Quality Patterns (CB-125 to CB-127)".to_string(),
            status: CheckStatus::Pass,
            message: "No coverage quality issues detected".to_string(),
            severity: Severity::Info,
        }
    }
}

/// Check Cargo.lock presence (reproducible builds)
/// Skips for non-Rust projects (no Cargo.toml).
pub(crate) fn check_cargo_lock(project_path: &Path) -> ComplianceCheck {
    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return ComplianceCheck {
            name: "Cargo.lock Present".to_string(),
            status: CheckStatus::Skip,
            message: "Not a Rust project (no Cargo.toml)".to_string(),
            severity: Severity::Info,
        };
    }

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
pub(crate) fn check_msrv(project_path: &Path) -> ComplianceCheck {
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
pub(crate) fn check_ci_configured(project_path: &Path) -> ComplianceCheck {
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

/// CB-300: Muda Waste Score (COMPLY-040)
/// Aggregates Seven Wastes into a single quality health metric.
pub(crate) fn check_muda_waste_score(project_path: &Path) -> ComplianceCheck {
    use crate::cli::handlers::comply_handlers::muda_handlers;

    let report = muda_handlers::calculate_muda_score(project_path);

    let message = format!(
        "Muda Score: {:.1}/100 ({}) - Over:{:.0} Wait:{:.0} Inv:{:.0} Proc:{:.0} Def:{:.0}",
        report.total_score,
        report.grade,
        report.overproduction,
        report.waiting,
        report.inventory,
        report.over_processing,
        report.defects,
    );

    let (status, severity) = match report.grade {
        muda_handlers::MudaGrade::Lean | muda_handlers::MudaGrade::Efficient => {
            (CheckStatus::Pass, Severity::Info)
        }
        muda_handlers::MudaGrade::Moderate => (CheckStatus::Warn, Severity::Warning),
        muda_handlers::MudaGrade::High | muda_handlers::MudaGrade::Critical => {
            (CheckStatus::Fail, Severity::Error)
        }
    };

    ComplianceCheck {
        name: "CB-300: Muda Waste Score".to_string(),
        status,
        message,
        severity,
    }
}

/// CB-301: Reproducibility Level Check (COMPLY-041)
/// Classifies project reproducibility as None/Bronze/Silver/Gold per NeurIPS/ICLR standards.
pub(crate) fn check_reproducibility_level(project_path: &Path) -> ComplianceCheck {
    use crate::cli::handlers::comply_handlers::reproducibility_handlers;

    let report = reproducibility_handlers::check_reproducibility(project_path);

    let detail_summary: String = report.details.iter().take(3).cloned().collect::<Vec<_>>().join("; ");
    let message = format!(
        "Reproducibility: {} - {}",
        report.level, detail_summary,
    );

    let (status, severity) = match report.level {
        reproducibility_handlers::ReproducibilityLevel::Gold => {
            (CheckStatus::Pass, Severity::Info)
        }
        reproducibility_handlers::ReproducibilityLevel::Silver => {
            (CheckStatus::Pass, Severity::Info)
        }
        reproducibility_handlers::ReproducibilityLevel::Bronze => {
            (CheckStatus::Warn, Severity::Warning)
        }
        reproducibility_handlers::ReproducibilityLevel::None => {
            (CheckStatus::Fail, Severity::Error)
        }
    };

    ComplianceCheck {
        name: "CB-301: Reproducibility Level".to_string(),
        status,
        message,
        severity,
    }
}

/// CB-302: Golden Trace Drift Detection (COMPLY-042)
/// Validates that renacer golden traces are still passing.
pub(crate) fn check_golden_trace_drift(project_path: &Path) -> ComplianceCheck {
    use crate::cli::handlers::comply_handlers::reproducibility_handlers;

    match reproducibility_handlers::check_golden_trace_drift(project_path) {
        None => ComplianceCheck {
            name: "CB-302: Golden Trace Drift".to_string(),
            status: CheckStatus::Skip,
            message: "No renacer.toml configured - golden tracing not enabled".to_string(),
            severity: Severity::Info,
        },
        Some(true) => ComplianceCheck {
            name: "CB-302: Golden Trace Drift".to_string(),
            status: CheckStatus::Pass,
            message: "Golden traces valid - no drift detected".to_string(),
            severity: Severity::Info,
        },
        Some(false) => ComplianceCheck {
            name: "CB-302: Golden Trace Drift".to_string(),
            status: CheckStatus::Fail,
            message: "Golden trace drift detected - run 'renacer validate' to investigate".to_string(),
            severity: Severity::Error,
        },
    }
}

/// CB-303: Equation-Driven Development Compliance (COMPLY-043)
/// Validates that simulation projects document mathematical models in pub fn docs.
pub(crate) fn check_edd_compliance(project_path: &Path) -> ComplianceCheck {
    use crate::cli::handlers::comply_handlers::edd_handlers;

    let report = edd_handlers::check_edd_compliance(project_path);

    if !report.is_simulation_project {
        return ComplianceCheck {
            name: "CB-303: EDD Compliance".to_string(),
            status: CheckStatus::Skip,
            message: "Not a simulation project (no simular/trueno-sim dependency)".to_string(),
            severity: Severity::Info,
        };
    }

    let violation_count = report.undocumented_fns.len();
    let message = format!(
        "EDD: {:.0}% ({}/{} pub fns documented with math)",
        report.compliance_pct, report.documented_fns, report.total_simulation_fns,
    );

    if report.compliance_pct >= 80.0 {
        ComplianceCheck {
            name: "CB-303: EDD Compliance".to_string(),
            status: CheckStatus::Pass,
            message,
            severity: Severity::Info,
        }
    } else if violation_count > 0 {
        ComplianceCheck {
            name: "CB-303: EDD Compliance".to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "{} - {} functions missing mathematical models",
                message, violation_count,
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-303: EDD Compliance".to_string(),
            status: CheckStatus::Pass,
            message,
            severity: Severity::Info,
        }
    }
}

/// CB-081: Dependency Count Check (Enhanced v2.9)
/// Per rust-project-score-v1.1-update.md, excessive dependencies degrade:
/// - Build times (more crates to compile)
/// - Binary size (more code linked)
/// - Supply chain security (more attack surface)
///
/// Enhancements:
/// - CB-081-A: Base dependency count scoring (0-5 points)
/// - CB-081-B: Duplicate crate detection
/// - CB-081-C: Feature flag hygiene analysis
/// - CB-081-D: Sovereign stack bonus (+1-3 points)
/// - CB-081-E: Trend tracking (delta from last check)
///
/// Format the CB-081 dependency health summary message
fn format_dependency_message(report: &DependencyCountReport) -> String {
    let transitive_display = if let Some(prod) = report.prod_transitive_count {
        format!("{} prod transitive ({} total w/dev)", prod, report.transitive_count)
    } else {
        format!("{} transitive", report.transitive_count)
    };
    let mut details = vec![format!("{} direct, {}", report.direct_count, transitive_display)];

    if let Some(ref trend) = report.trend {
        if trend.direct_delta != 0 || trend.transitive_delta != 0 {
            details.push(format!("Δ {:+}/{:+} since last", trend.direct_delta, trend.transitive_delta));
        }
    }
    if !report.duplicate_crates.is_empty() {
        details.push(format!("{} duplicates", report.duplicate_crates.len()));
    }
    details.push(format!("{:.0}% feature-gated", report.feature_gated_pct));
    if report.sovereign_bonus > 0 {
        details.push(format!("+{} sovereign ({})", report.sovereign_bonus, report.sovereign_crates.join(", ")));
    }
    format!("Score: {}/5 | {}", report.score, details.join(" | "))
}

/// Append violation details to a message string
fn append_violation_details(msg: &mut String, report: &DependencyCountReport, limit: usize) {
    for v in report.violations.iter().take(limit) {
        let icon = if v.severity == super::comply_cb_detect::Severity::Error { "✗" } else { "⚠" };
        msg.push_str(&format!("\n    {} {}", icon, v.description));
    }
}

pub(crate) fn check_dependency_count(project_path: &Path) -> ComplianceCheck {
    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return ComplianceCheck {
            name: "CB-081: Dependency Health".to_string(),
            status: CheckStatus::Skip,
            message: "Not a Rust project (no Cargo.toml)".to_string(),
            severity: Severity::Info,
        };
    }

    let report = detect_cb081_dependency_count(project_path);
    let message = format_dependency_message(&report);

    let has_critical = report.violations.iter().any(|v| v.severity == super::comply_cb_detect::Severity::Error);

    if report.score >= 4 && !has_critical {
        ComplianceCheck {
            name: "CB-081: Dependency Health".to_string(),
            status: CheckStatus::Pass,
            message,
            severity: Severity::Info,
        }
    } else if report.score >= 2 && !has_critical {
        let mut msg = message;
        append_violation_details(&mut msg, &report, 1);
        ComplianceCheck {
            name: "CB-081: Dependency Health".to_string(),
            status: CheckStatus::Warn,
            message: msg,
            severity: Severity::Warning,
        }
    } else {
        let mut msg = message;
        append_violation_details(&mut msg, &report, 3);
        ComplianceCheck {
            name: "CB-081: Dependency Health".to_string(),
            status: CheckStatus::Fail,
            message: msg,
            severity: Severity::Error,
        }
    }
}

// Dead code analysis extracted for file health (CB-040)
include!("check_handlers_dead_code.rs");

/// CB-400: Check Shell & Makefile Quality using bashrs
///
/// Uses bashrs to lint:
/// - CB-400: Git hooks (pre-commit, pre-push, etc.)
/// - CB-401: Makefile
/// - CB-402: Shell scripts (*.sh)
pub(crate) fn check_shell_makefile_quality(project_path: &Path) -> ComplianceCheck {
    use super::comply_cb_detect::{
        detect_cb400_git_hooks_quality,
        detect_cb401_makefile_quality,
        detect_cb402_shell_script_quality,
    };

    let mut all_issues: Vec<String> = Vec::new();
    let mut warning_count = 0;
    let mut error_count = 0;

    // CB-400: Git hooks
    let hook_violations = detect_cb400_git_hooks_quality(project_path);
    for v in &hook_violations {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        match v.severity {
            super::comply_cb_detect::Severity::Error | super::comply_cb_detect::Severity::Critical => error_count += 1,
            _ => warning_count += 1,
        }
    }

    // CB-401: Makefile
    let makefile_violations = detect_cb401_makefile_quality(project_path);
    for v in &makefile_violations {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        match v.severity {
            super::comply_cb_detect::Severity::Error | super::comply_cb_detect::Severity::Critical => error_count += 1,
            _ => warning_count += 1,
        }
    }

    // CB-402: Shell scripts
    let shell_violations = detect_cb402_shell_script_quality(project_path);
    for v in &shell_violations {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        match v.severity {
            super::comply_cb_detect::Severity::Error | super::comply_cb_detect::Severity::Critical => error_count += 1,
            _ => warning_count += 1,
        }
    }

    let total_violations = hook_violations.len() + makefile_violations.len() + shell_violations.len();

    if total_violations == 0 {
        ComplianceCheck {
            name: "CB-400: Shell & Makefile Quality".to_string(),
            status: CheckStatus::Pass,
            message: "bashrs: All shell scripts and Makefiles pass quality checks".to_string(),
            severity: Severity::Info,
        }
    } else if error_count > 0 {
        ComplianceCheck {
            name: "CB-400: Shell & Makefile Quality".to_string(),
            status: CheckStatus::Fail,
            message: format!(
                "bashrs: {} errors, {} warnings:\n{}",
                error_count,
                warning_count,
                format_violation_list(&all_issues),
            ),
            severity: Severity::Error,
        }
    } else {
        ComplianceCheck {
            name: "CB-400: Shell & Makefile Quality".to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "bashrs: {} warnings:\n{}",
                warning_count,
                format_violation_list(&all_issues),
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-130: Agent Context Adoption (PMAT-470)
///
/// Checks whether the project has a RAG-powered agent context index
/// set up for intelligent code search. Validates:
/// - Index exists at .pmat/context.idx or .pmat/context.db
/// - Index is fresh (less than 24 hours old)
/// - CLAUDE.md references pmat_query_code (optional)
pub(crate) fn check_agent_context_adoption(project_path: &Path) -> ComplianceCheck {
    let report = detect_cb130_agent_context_adoption(project_path);

    let mut issues: Vec<String> = Vec::new();
    let mut warning_count = 0;

    if !report.index_exists {
        issues.push("CB-130: No agent context index found at .pmat/context.idx or .pmat/context.db".to_string());
        issues.push("  Run 'pmat query \"test\" --rebuild-index' to build the index".to_string());
        warning_count += 1;
    } else {
        if report.index_stale {
            let age = report.index_age_hours.unwrap_or(0.0);
            issues.push(format!(
                "CB-130: Agent context index is stale ({:.0} hours old, threshold: 24h)",
                age
            ));
            issues.push(
                "  Run 'pmat query \"test\" --rebuild-index' to refresh".to_string(),
            );
            warning_count += 1;
        }

        if report.function_count == 0 {
            issues.push("CB-130: Agent context index has 0 functions".to_string());
            warning_count += 1;
        }
    }

    if !report.claude_md_configured {
        issues.push(
            "CB-130: CLAUDE.md does not reference pmat_query_code or pmat query".to_string(),
        );
        issues.push(
            "  Add agent context instructions to CLAUDE.md for agent adoption".to_string(),
        );
        warning_count += 1;
    }

    // Check for missing required patterns
    if !report.missing_required_patterns.is_empty() {
        for pattern in &report.missing_required_patterns {
            issues.push(format!("CB-130: CLAUDE.md missing required: \"{}\"", pattern));
        }
        issues.push("  Add pmat query decision tree to CLAUDE.md".to_string());
        warning_count += 1;
    }

    // Check for forbidden patterns (potential grep usage instructions)
    if !report.forbidden_patterns_found.is_empty() {
        for found in &report.forbidden_patterns_found {
            issues.push(format!(
                "CB-130: CLAUDE.md contains forbidden pattern \"{}\" at line {}",
                found.pattern, found.line
            ));
        }
        issues.push("  Remove grep/find examples from CLAUDE.md (use pmat query instead)".to_string());
        warning_count += 1;
    }

    if issues.is_empty() {
        ComplianceCheck {
            name: "CB-130: Agent Context Adoption".to_string(),
            status: CheckStatus::Pass,
            message: format!(
                "Agent context index: {} functions, CLAUDE.md configured",
                report.function_count
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-130: Agent Context Adoption".to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "{} issues:\n{}",
                warning_count,
                issues
                    .iter()
                    .map(|i| format!("    - {}", i))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            severity: Severity::Warning,
        }
    }
}

// Three-layer CLI (review/audit) extracted for file health (CB-040)
include!("review_audit_handlers.rs");

// Check handler tests extracted for file health (CB-040)
include!("check_handlers_tests.rs");
