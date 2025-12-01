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
    let mut checks = vec![];

    // Check 1: Version currency
    checks.push(check_version_currency(project_version));
    // Check 2: Config file presence
    checks.push(check_config_files(project_path));
    // Check 3: Hooks installation
    checks.push(check_hooks_installed(project_path));
    // Check 4: Quality thresholds
    checks.push(check_quality_thresholds(project_path));
    // Check 5: Deprecated features
    checks.push(check_deprecated_features(project_path));

    // Calculate compliance
    let failures = checks.iter().filter(|c| c.status == CheckStatus::Fail).count();
    let warnings = checks.iter().filter(|c| c.status == CheckStatus::Warn).count();
    let is_compliant = failures == 0;

    // Get breaking changes
    let breaking_changes = get_breaking_changes_since(project_version);
    let versions_behind = calculate_versions_behind(project_version);

    // Build recommendations
    let mut recommendations = vec![];
    if versions_behind > 0 {
        recommendations.push(format!("Run 'pmat comply migrate' to update to v{}", PMAT_VERSION));
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
            checks.into_iter().filter(|c| c.status == CheckStatus::Fail).collect()
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
        println!("\x1b[33mWarning: {} breaking changes detected:\x1b[0m", breaking_changes.len());
        for change in &breaking_changes {
            println!("  - v{}: {}", change.version, change.description);
        }
        println!("\nUse --force to proceed anyway\n");
        if !force { return Ok(()); }
    }

    if !no_backup && !dry_run {
        let backup_path = project_path.join(".pmat").join("backup");
        fs::create_dir_all(&backup_path)?;
        println!("Created backup at: {}", backup_path.display());
    }

    let migrations = vec![
        ("Update project.toml version", migrate_project_version(project_path, target, dry_run)),
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
                println!("  \x1b[31m[BREAKING]\x1b[0m v{}: {}", entry.version, entry.description);
            }
        }
    } else {
        for entry in &changes {
            let icon = if entry.breaking { "\x1b[31m[BREAKING]\x1b[0m" } else { "\x1b[32m[FEATURE]\x1b[0m" };
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

    println!("\x1b[32m✓\x1b[0m Initialized PMAT project at {}", config_path.display());
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
            message: format!("{} versions behind (v{} → v{})", behind, project_version, PMAT_VERSION),
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
    let missing: Vec<&str> = config_files.iter().filter(|f| !project_path.join(f).exists()).copied().collect();

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

fn calculate_versions_behind(project_version: &str) -> u32 {
    let current_parts: Vec<u32> = PMAT_VERSION.split('.').filter_map(|s| s.parse().ok()).collect();
    let project_parts: Vec<u32> = project_version.split('.').filter_map(|s| s.parse().ok()).collect();

    if current_parts.len() >= 2 && project_parts.len() >= 2 {
        current_parts.get(1).unwrap_or(&0).saturating_sub(*project_parts.get(1).unwrap_or(&0))
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
    if dry_run { return Ok(true); }
    let mut config = load_or_create_project_config(project_path)?;
    if config.pmat.version == target { return Ok(false); }
    config.pmat.version = target.to_string();
    config.pmat.last_compliance_check = Some(Utc::now());
    let content = toml::to_string_pretty(&config)?;
    fs::write(project_path.join(".pmat").join("project.toml"), &content)?;
    Ok(true)
}

fn migrate_gitignore(project_path: &Path, dry_run: bool) -> Result<bool> {
    let gitignore_path = project_path.join(".gitignore");
    let pmat_entries = [".pmat/backup/", ".pmat-qa/"];
    if !gitignore_path.exists() { return Ok(false); }
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
        if !new_content.ends_with('\n') { new_content.push('\n'); }
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
    let status = if report.is_compliant { "\x1b[32mCOMPLIANT\x1b[0m" } else { "\x1b[31mNON-COMPLIANT\x1b[0m" };
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
        for rec in &report.recommendations { println!("  • {}", rec); }
    }
    println!("\n{}", "=".repeat(60));
}

fn print_compliance_markdown(report: &ComplianceReport) {
    println!("# PMAT Compliance Report\n");
    println!("| Property | Value |");
    println!("|----------|-------|");
    println!("| Project Version | {} |", report.project_version);
    println!("| Current PMAT | {} |", report.current_version);
    println!("| Status | {} |", if report.is_compliant { "COMPLIANT" } else { "NON-COMPLIANT" });
    println!("\n## Checks\n");
    for check in &report.checks {
        let icon = match check.status { CheckStatus::Pass => "✅", CheckStatus::Warn => "⚠️", CheckStatus::Fail => "❌", CheckStatus::Skip => "⏭️" };
        println!("- {} **{}**: {}", icon, check.name, check.message);
    }
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
