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
    /// Debt tickets found under `.pmat-tickets/`, or `None` when
    /// `--include-history` was not passed.
    ///
    /// `Some(vec![])` is a result — "the flag ran and the store is empty" — and
    /// is rendered as such rather than as silence. Skipped when absent so the
    /// default JSON document is unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<TicketHistoryEntry>>,
}

/// One debt ticket, as written by `pmat comply upgrade` into `.pmat-tickets/`.
///
/// Every field is read from the file; nothing is inferred. `None` means the
/// ticket did not carry that key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TicketHistoryEntry {
    /// File the entry was read from, relative to the project root.
    pub file: String,
    pub ticket_id: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub created_at: Option<String>,
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

/// A `(clause_id, check_fn)` pair for parallel comply execution.
pub(crate) type ClauseCheck = (&'static str, fn(&Path) -> ComplianceCheck);

/// Run each `(clause_id, check_fn)` concurrently, applying config-based
/// filtering to every result. Used by the heaviest comply builders
/// (cot-proof, work-ladder, falsification, binding-scope) whose per-check work
/// — walking `.pmat-work/`, parsing contract YAML, git *read* SHA checks —
/// dominates wall-time and is side-effect-free, so the checks are safe to run
/// in parallel. Result order matches the input `checks` order (rayon's
/// indexed `collect` is order-preserving), keeping report output deterministic.
pub(crate) fn run_checks_parallel(
    project_path: &Path,
    config: &ComplyConfig,
    checks: Vec<ClauseCheck>,
) -> Vec<ComplianceCheck> {
    use rayon::prelude::*;
    checks
        .into_par_iter()
        .map(|(id, f)| filter_check_by_config(f(project_path), id, config))
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BreakingChange {
    pub version: String,
    pub description: String,
    pub migration_guide: Option<String>,
}

/// Format a list of violations for display (indented, one per line).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_violation_list(issues: &[String]) -> String {
    issues
        .iter()
        .map(|i| format!("    - {}", i))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Helper: Create skip check result
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn skip_check(name: &str, message: &str) -> ComplianceCheck {
    ComplianceCheck {
        name: name.to_string(),
        status: CheckStatus::Skip,
        message: message.to_string(),
        severity: Severity::Info,
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub(crate) fn calculate_versions_behind(project_version: &str) -> u32 {
    debug_assert!(
        !project_version.is_empty(),
        "project_version must not be empty"
    );
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

/// PMAT's own changelog, embedded at build time.
///
/// `comply diff` reports what changed *in PMAT* between the version a project
/// is pinned to and the version installed, so the only honest source is PMAT's
/// CHANGELOG.md — not a list written into the source.
const EMBEDDED_CHANGELOG: &str = include_str!("../../../../../CHANGELOG.md");

/// One released version parsed out of CHANGELOG.md.
struct ChangelogSection {
    version: semver::Version,
    description: String,
    breaking: bool,
}

/// Parse `## [x.y.z] - date` sections out of a Keep-a-Changelog document.
///
/// `[Unreleased]` and any heading whose bracketed token is not a semver
/// version are skipped: they name no release, so no range contains them.
fn parse_changelog(text: &str) -> Vec<ChangelogSection> {
    let mut sections: Vec<ChangelogSection> = Vec::new();
    let mut current: Option<(semver::Version, Vec<&str>)> = None;

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## [") {
            if let Some((version, body)) = current.take() {
                sections.push(finish_section(version, &body));
            }
            let token = rest.split(']').next().unwrap_or("");
            if let Ok(version) = semver::Version::parse(token) {
                current = Some((version, Vec::new()));
            }
        } else if let Some((_, body)) = current.as_mut() {
            body.push(line);
        }
    }
    if let Some((version, body)) = current.take() {
        sections.push(finish_section(version, &body));
    }

    sections
}

fn finish_section(version: semver::Version, body: &[&str]) -> ChangelogSection {
    ChangelogSection {
        version,
        description: summarize_section(body),
        breaking: body
            .iter()
            .any(|l| l.contains("BREAKING") || l.trim_start().starts_with("### Removed")),
    }
}

/// First substantive line of a section, used as its one-line description.
fn summarize_section(body: &[&str]) -> String {
    let raw = body
        .iter()
        .map(|l| l.trim())
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .unwrap_or("");
    let cleaned = raw
        .trim_start_matches(['-', '*'])
        .trim()
        .replace("**", "")
        .replace('`', "");
    let cleaned = cleaned.trim();
    if cleaned.chars().count() > 160 {
        let truncated: String = cleaned.chars().take(157).collect();
        format!("{truncated}...")
    } else {
        cleaned.to_string()
    }
}

/// Sections in the half-open range `(from, to]`.
///
/// An unparsable bound yields nothing: a range we cannot evaluate has no
/// entries we can honestly claim fall inside it.
fn sections_in_range(from: &str, to: &str) -> Vec<ChangelogSection> {
    let (Ok(from), Ok(to)) = (
        semver::Version::parse(from.trim_start_matches('v')),
        semver::Version::parse(to.trim_start_matches('v')),
    ) else {
        return Vec::new();
    };

    parse_changelog(EMBEDDED_CHANGELOG)
        .into_iter()
        .filter(|s| s.version > from && s.version <= to)
        .collect()
}

/// Breaking changes released after `from_version`.
///
/// This used to be `vec![]` for every input, so `comply check` could never
/// report a breaking change no matter how far behind a project was.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn get_breaking_changes_since(from_version: &str) -> Vec<BreakingChange> {
    sections_in_range(from_version, PMAT_VERSION)
        .into_iter()
        .filter(|s| s.breaking)
        .map(|s| BreakingChange {
            version: s.version.to_string(),
            description: s.description,
            migration_guide: None,
        })
        .collect()
}

#[derive(Debug, Clone)]
pub(crate) struct ChangelogEntry {
    pub version: String,
    pub description: String,
    pub breaking: bool,
}

/// Changelog entries released in `(from, to]`.
///
/// Both parameters used to be discarded (`_from`, `_to`) in favour of three
/// hardcoded entries stamped with the *current* version, so every range —
/// including v3.29.0 → v3.29.0 — rendered the same three lines, one of them
/// advertising a `cleanup-resources` command that does not exist in the binary.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn get_changelog_entries(from: &str, to: &str) -> Vec<ChangelogEntry> {
    sections_in_range(from, to)
        .into_iter()
        .map(|s| ChangelogEntry {
            version: s.version.to_string(),
            description: s.description,
            breaking: s.breaking,
        })
        .collect()
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn update_last_check_timestamp(project_path: &Path) -> anyhow::Result<()> {
    let config_path = project_path.join(".pmat").join("project.toml");
    if let Ok(mut config) = load_or_create_project_config(project_path) {
        config.pmat.last_compliance_check = Some(Utc::now());
        let content = toml::to_string_pretty(&config)?;
        std::fs::write(&config_path, &content)?;
    }
    Ok(())
}

/// The COMPLIANT / NON-COMPLIANT word, coloured only when colour is enabled.
pub(crate) fn compliance_status_text(is_compliant: bool) -> String {
    if is_compliant {
        c::colored(c::GREEN, "COMPLIANT")
    } else {
        c::colored(c::RED, "NON-COMPLIANT")
    }
}

/// The per-check status glyph, coloured only when colour is enabled.
pub(crate) fn check_status_icon(status: CheckStatus) -> String {
    match status {
        CheckStatus::Pass => c::colored(c::GREEN, "\u{2713}"),
        CheckStatus::Warn => c::colored(c::YELLOW, "\u{26a0}"),
        CheckStatus::Fail => c::colored(c::RED, "\u{2717}"),
        CheckStatus::Skip => c::colored(c::DIM, "-"),
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    // GH #684: the status word and every check icon used to interpolate the raw
    // `pub const` sequences. Those are `const`, so they cannot consult
    // `colors_enabled`, and `pmat comply check --color never > out.txt` wrote
    // **155** escape-bearing lines — byte-identical to `--color auto`. NO_COLOR=1
    // was ignored for the same reason. `c::colored` keeps the colour SELECTION
    // here while honouring the rule.
    println!(
        "Status:          {}\n",
        compliance_status_text(report.is_compliant)
    );
    println!("{}:", c::label("Checks"));
    for check in &report.checks {
        println!(
            "  {} {}: {}",
            check_status_icon(check.status),
            check.name,
            check.message
        );
    }
    if !report.recommendations.is_empty() {
        println!("\n{}:", c::label("Recommendations"));
        for rec in &report.recommendations {
            println!("  \u{2022} {}", rec);
        }
    }
    println!("\n{}", c::rule());
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn update_project_config(project_path: &Path, dry_run: bool) -> anyhow::Result<bool> {
    migrate_project_version(project_path, PMAT_VERSION, dry_run)
}

/// Update project hooks to latest templates
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod changelog_range_tests {
    use super::*;

    const SAMPLE: &str = "\
# Changelog

## [Unreleased]

## [3.0.0] - 2026-01-01

### Removed

- Dropped the old API.

## [2.1.0] - 2025-06-01

- Added a widget.

## [1.5.0] - 2024-01-01

- First widget.
";

    /// A range must contain only the versions inside it. Both parameters used to
    /// be discarded, so every range printed the same three canned entries.
    #[test]
    fn range_filters_by_version() {
        let sections = parse_changelog(SAMPLE);
        let versions: Vec<String> = sections.iter().map(|s| s.version.to_string()).collect();
        assert_eq!(versions, vec!["3.0.0", "2.1.0", "1.5.0"]);

        let in_range: Vec<String> = sections
            .iter()
            .filter(|s| {
                s.version > semver::Version::parse("1.5.0").unwrap()
                    && s.version <= semver::Version::parse("2.1.0").unwrap()
            })
            .map(|s| s.version.to_string())
            .collect();
        assert_eq!(in_range, vec!["2.1.0"]);
    }

    /// `### Removed` and `BREAKING` mark a release as breaking.
    #[test]
    fn breaking_sections_are_detected() {
        let sections = parse_changelog(SAMPLE);
        assert!(sections[0].breaking, "3.0.0 removes an API");
        assert!(!sections[1].breaking);
    }

    /// An empty range must be empty. `--from X --to X` used to print three
    /// entries stamped with the current version.
    #[test]
    fn identical_from_and_to_yields_nothing() {
        assert!(get_changelog_entries(PMAT_VERSION, PMAT_VERSION).is_empty());
    }

    /// A range that ends before this project's first release must not report
    /// entries from the current version.
    #[test]
    fn ancient_range_does_not_report_current_version() {
        let entries = get_changelog_entries("1.0.0", "2.0.0");
        assert!(
            entries.iter().all(|e| e.version != PMAT_VERSION),
            "entries outside the requested range leaked in: {:?}",
            entries
        );
        assert!(
            !entries
                .iter()
                .any(|e| e.description.contains("cleanup-resources")),
            "the canned entry advertising a nonexistent command is still emitted"
        );
    }

    /// Entries must come from the shipped changelog, so a wide range finds the
    /// releases that are actually in it.
    #[test]
    fn wide_range_reads_the_embedded_changelog() {
        let entries = get_changelog_entries("2.0.0", PMAT_VERSION);
        assert!(
            entries.len() > 3,
            "expected the real changelog, got {} entries",
            entries.len()
        );
        assert!(
            entries.iter().any(|e| e.version == PMAT_VERSION),
            "the current release must be inside (2.0.0, {PMAT_VERSION}]"
        );
    }
}

// ── GH #684 (round 4): --color never / NO_COLOR must reach comply check ──

#[cfg(test)]
mod colour_contract_tests {
    use super::{check_status_icon, compliance_status_text, CheckStatus};

    /// `pmat comply check --color never > out.txt` wrote **155**
    /// escape-bearing lines — byte-identical to `--color auto` — because the
    /// status word and every check icon interpolated the raw `pub const`
    /// sequences, which are `const` and so cannot consult `colors_enabled`.
    #[test]
    fn status_text_and_icons_are_plain_when_colour_is_disabled() {
        assert!(
            !crate::cli::colors::colors_enabled(),
            "cargo test captures stdout, so colour must resolve to off here"
        );
        assert_eq!(compliance_status_text(true), "COMPLIANT");
        assert_eq!(compliance_status_text(false), "NON-COMPLIANT");
        for status in [
            CheckStatus::Pass,
            CheckStatus::Warn,
            CheckStatus::Fail,
            CheckStatus::Skip,
        ] {
            let icon = check_status_icon(status);
            assert!(
                !icon.contains('\u{1b}'),
                "expected a plain glyph with colour off, got {icon:?}"
            );
            assert!(!icon.is_empty(), "the glyph itself must survive");
        }
    }
}
