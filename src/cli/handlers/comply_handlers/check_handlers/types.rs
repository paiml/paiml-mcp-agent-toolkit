// PMAT Compliance and Migration Handlers - Types, Enums, and Helpers
//
// Contains all shared types used across the check_handlers submodules.

use crate::cli::colors as c;
use crate::models::comply_config::{CheckSeverity as ConfigSeverity, ComplyConfig};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Current PMAT version (from Cargo.toml)
pub(crate) const PMAT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Project compliance information stored in .pmat/project.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProjectConfig {
    pub pmat: PmatSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PmatSection {
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
pub(crate) struct ComplianceReport {
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
pub(crate) struct ComplianceCheck {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub(crate) enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub(crate) enum Severity {
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

/// Filter a check result based on YAML configuration.
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
    let configured_severity = config.get_severity(check_id);
    ComplianceCheck {
        severity: configured_severity.into(),
        ..check
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BreakingChange {
    pub version: String,
    pub description: String,
    pub migration_guide: Option<String>,
}

/// Format a list of violations for display (indented, one per line).
pub(crate) fn format_violation_list(issues: &[String]) -> String {
    issues
        .iter()
        .map(|i| format!("    - {}", i))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Helper: Create skip check result
pub(crate) fn skip_check(name: &str, message: &str) -> ComplianceCheck {
    ComplianceCheck {
        name: name.to_string(),
        status: CheckStatus::Skip,
        message: message.to_string(),
        severity: Severity::Info,
    }
}

pub(crate) fn calculate_versions_behind(project_version: &str) -> u32 {
    let current_parts: Vec<u32> = PMAT_VERSION
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let project_parts: Vec<u32> = project_version
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    if current_parts.len() >= 2 && project_parts.len() >= 2 {
        let cur_major = *current_parts.first().unwrap_or(&0);
        let cur_minor = *current_parts.get(1).unwrap_or(&0);
        let proj_major = *project_parts.first().unwrap_or(&0);
        let proj_minor = *project_parts.get(1).unwrap_or(&0);
        if cur_major > proj_major {
            (cur_major - proj_major) * 10 + cur_minor.saturating_sub(proj_minor)
        } else if cur_major == proj_major {
            cur_minor.saturating_sub(proj_minor)
        } else {
            0
        }
    } else {
        0
    }
}

pub(crate) fn get_breaking_changes_since(_from_version: &str) -> Vec<BreakingChange> {
    vec![]
}

#[derive(Debug, Clone)]
pub(crate) struct ChangelogEntry {
    pub version: String,
    pub description: String,
    pub breaking: bool,
}

pub(crate) fn get_changelog_entries(_from: &str, _to: &str) -> Vec<ChangelogEntry> {
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

pub(crate) fn load_or_create_project_config(project_path: &Path) -> anyhow::Result<ProjectConfig> {
    let config_path = project_path.join(".pmat").join("project.toml");
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        Ok(toml::from_str(&content)?)
    } else {
        let config = ProjectConfig::default();
        let pmat_dir = project_path.join(".pmat");
        if !pmat_dir.exists() {
            std::fs::create_dir_all(&pmat_dir)?;
        }
        let content = toml::to_string_pretty(&config)?;
        std::fs::write(&config_path, &content)?;
        Ok(config)
    }
}

pub(crate) fn update_last_check_timestamp(project_path: &Path) -> anyhow::Result<()> {
    let config_path = project_path.join(".pmat").join("project.toml");
    if let Ok(mut config) = load_or_create_project_config(project_path) {
        config.pmat.last_compliance_check = Some(Utc::now());
        let content = toml::to_string_pretty(&config)?;
        std::fs::write(&config_path, &content)?;
    }
    Ok(())
}

pub(crate) fn print_compliance_text(report: &ComplianceReport) {
    println!("\n{}", c::rule());
    println!("{}", c::header("PMAT Compliance Report"));
    println!("{}", c::rule());
    println!("\nProject Version: {}", c::number(&report.project_version));
    println!("Current PMAT:    {}", c::number(&report.current_version));
    println!(
        "Versions Behind: {}",
        c::number(&report.versions_behind.to_string())
    );
    let status = if report.is_compliant {
        format!("{}COMPLIANT{}", c::GREEN, c::RESET)
    } else {
        format!("{}NON-COMPLIANT{}", c::RED, c::RESET)
    };
    println!("Status:          {}\n", status);
    println!("{}:", c::label("Checks"));
    for check in &report.checks {
        let icon = match check.status {
            CheckStatus::Pass => format!("{}\u{2713}{}", c::GREEN, c::RESET),
            CheckStatus::Warn => format!("{}\u{26a0}{}", c::YELLOW, c::RESET),
            CheckStatus::Fail => format!("{}\u{2717}{}", c::RED, c::RESET),
            CheckStatus::Skip => format!("{}-{}", c::DIM, c::RESET),
        };
        println!("  {} {}: {}", icon, check.name, check.message);
    }
    if !report.recommendations.is_empty() {
        println!("\n{}:", c::label("Recommendations"));
        for rec in &report.recommendations {
            println!("  \u{2022} {}", rec);
        }
    }
    println!("\n{}", c::rule());
}

pub(crate) fn print_compliance_markdown(report: &ComplianceReport) {
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
            CheckStatus::Pass => "\u{2705}",
            CheckStatus::Warn => "\u{26a0}\u{fe0f}",
            CheckStatus::Fail => "\u{274c}",
            CheckStatus::Skip => "\u{23ed}\u{fe0f}",
        };
        println!("- {} **{}**: {}", icon, check.name, check.message);
    }
}

pub(crate) fn migrate_project_version(
    project_path: &Path,
    target: &str,
    dry_run: bool,
) -> anyhow::Result<bool> {
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
    std::fs::write(project_path.join(".pmat").join("project.toml"), &content)?;
    Ok(true)
}

pub(crate) fn migrate_gitignore(project_path: &Path, dry_run: bool) -> anyhow::Result<bool> {
    let gitignore_path = project_path.join(".gitignore");
    let pmat_entries = [
        ".pmat/backup/",
        ".pmat-qa/",
        ".pmat/context.idx/",
        ".pmat/workspace.idx/",
        ".pmat/deps-cache.json",
    ];
    if !gitignore_path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(&gitignore_path)?;
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
        std::fs::write(&gitignore_path, &new_content)?;
    }
    Ok(needs_update)
}

pub(crate) fn update_project_config(project_path: &Path, dry_run: bool) -> anyhow::Result<bool> {
    migrate_project_version(project_path, PMAT_VERSION, dry_run)
}

/// Update project hooks to latest templates
pub(crate) async fn update_project_hooks(
    project_path: &Path,
    dry_run: bool,
) -> anyhow::Result<bool> {
    use crate::cli::handlers::hooks_command_handlers::HooksCommand;
    let hooks_dir = project_path.join(".git/hooks");
    if !hooks_dir.exists() {
        return Ok(false);
    }
    let hooks_cmd = HooksCommand::new(hooks_dir.clone(), project_path.join("pmat.toml"));
    let status = hooks_cmd.status().await?;
    let action = determine_hook_action(&hooks_cmd, &status).await?;
    if action == HookAction::UpToDate {
        return Ok(false);
    }
    if dry_run {
        return Ok(true);
    }
    match action {
        HookAction::Install => {
            hooks_cmd.install(false, true, false).await?;
        }
        HookAction::ForceReplace => {
            hooks_cmd.install(true, true, false).await?;
        }
        HookAction::Refresh => {
            hooks_cmd.refresh().await?;
        }
        HookAction::UpToDate => unreachable!(),
    }
    Ok(true)
}

#[derive(PartialEq)]
pub(crate) enum HookAction {
    Install,
    ForceReplace,
    Refresh,
    UpToDate,
}

pub(crate) async fn determine_hook_action(
    hooks_cmd: &crate::cli::handlers::hooks_command_handlers::HooksCommand,
    status: &crate::cli::handlers::hooks_command_handlers::HookStatus,
) -> anyhow::Result<HookAction> {
    if !status.installed {
        return Ok(HookAction::Install);
    }
    if !status.is_pmat_managed {
        return Ok(HookAction::ForceReplace);
    }
    let verify = hooks_cmd.verify(false).await?;
    if verify.issues.iter().any(|i| i.contains("outdated")) {
        return Ok(HookAction::Refresh);
    }
    Ok(HookAction::UpToDate)
}
