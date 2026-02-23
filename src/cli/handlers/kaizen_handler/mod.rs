#![cfg_attr(coverage_nightly, coverage(off))]
//! Kaizen Handler - Autonomous continuous improvement (Toyota Way)
//!
//! Orchestrates deterministic fixes (clippy --fix, cargo fmt) and optional
//! AI sub-agent delegation for complex issues. One-shot by default:
//! scan -> fix -> report -> exit.

pub mod fixing;
pub mod git_ops;
pub mod prioritization;
pub mod reporting;
pub mod scanning;

use crate::cli::commands::KaizenOutputFormat;
use crate::cli::handlers::comply_handlers::cross_crate_handlers::discover_workspace_crates;
use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

// Re-export submodule items used within the crate
pub(crate) use fixing::{apply_safe_fixes, apply_safe_fixes_for_crate, spawn_agents, spawn_agents_for_crate};
pub(crate) use git_ops::{commit_changes, create_github_issues, push_changes};
pub(crate) use prioritization::{enrich_with_tarantula, sort_findings};
pub(crate) use reporting::output_report;
pub(crate) use scanning::scan_crate;

/// Configuration for a kaizen run
pub struct KaizenConfig {
    pub path: PathBuf,
    pub dry_run: bool,
    pub commit: bool,
    pub create_issues: bool,
    pub push: bool,
    pub auto_agent: bool,
    pub max_agents: usize,
    pub format: KaizenOutputFormat,
    pub output: Option<PathBuf>,
    pub skip_clippy: bool,
    pub skip_fmt: bool,
    pub skip_comply: bool,
    pub skip_github: bool,
    pub skip_defects: bool,
    pub cross_stack: bool,
}

/// Source of a kaizen finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSource {
    Clippy,
    Rustfmt,
    Comply,
    Defects,
    CoverageGap,
    GitHubIssue,
}

/// Severity of a kaizen finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// A single finding from a kaizen scan
#[derive(Debug, Clone, Serialize)]
pub struct KaizenFinding {
    pub source: FindingSource,
    pub severity: FindingSeverity,
    pub category: String,
    pub message: String,
    pub file: Option<String>,
    pub auto_fixable: bool,
    pub agent_fixable: bool,
    pub fix_applied: bool,
    pub agent_prompt: Option<String>,
    /// Tarantula suspiciousness score (0.0-1.0), if coverage data available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspiciousness_score: Option<f32>,
    /// Which crate this finding belongs to (set in cross-stack mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crate_name: Option<String>,
}

/// A GitHub issue created by kaizen for an unfixed finding
#[derive(Debug, Clone, Serialize)]
pub struct GithubIssueRef {
    pub number: u64,
    pub url: String,
    pub finding_category: String,
}

/// Report from a kaizen run
#[derive(Debug, Clone, Serialize)]
pub struct KaizenReport {
    pub findings: Vec<KaizenFinding>,
    pub auto_fixed_count: usize,
    pub agent_fixed_count: usize,
    pub remaining_count: usize,
    pub issues_created: Vec<GithubIssueRef>,
    pub commit_hash: Option<String>,
    pub pushed: bool,
    /// Crates scanned (populated in cross-stack mode)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub crates_scanned: Vec<String>,
}

/// Result of applying fix/commit/agent/issues/push phases
struct PhaseResult {
    auto_fixed: usize,
    agent_fixed: usize,
    commit_hash: Option<String>,
    issues_created: Vec<GithubIssueRef>,
    pushed: bool,
}

/// Accumulator for cross-stack phase results
#[derive(Default)]
struct CrossStackAccumulator {
    auto_fixed: usize,
    agent_fixed: usize,
    last_commit: Option<String>,
    issues: Vec<GithubIssueRef>,
    pushed: bool,
}

// ---------------------------------------------------------------------------
// Entry point and orchestration
// ---------------------------------------------------------------------------

/// Main kaizen handler entry point
pub async fn handle_kaizen(config: KaizenConfig) -> Result<()> {
    let path = config.path.canonicalize().unwrap_or(config.path.clone());

    if config.cross_stack {
        handle_kaizen_cross_stack(&path, &config).await
    } else {
        handle_kaizen_single(&path, &config).await
    }
}

/// Single-project kaizen (original behavior)
async fn handle_kaizen_single(path: &Path, config: &KaizenConfig) -> Result<()> {
    eprintln!("Kaizen: scanning {} ...", path.display());

    // Phase 1: Scan for findings
    let mut findings = scan_crate(path, None, config)?;

    // Phase 2: Enrich with tarantula suspiciousness from coverage data
    enrich_with_tarantula(path, &mut findings);

    // Phase 2b: Sort by composite priority (severity * suspiciousness)
    sort_findings(&mut findings);

    let total_found = findings.len();
    eprintln!("Kaizen: found {} issues", total_found);

    // Phase 3-7: Fix, commit, agent, issues, push
    let result = apply_phases(path, &mut findings, config)?;

    // Build report
    let remaining = findings.iter().filter(|f| !f.fix_applied).count();
    let report = KaizenReport {
        findings,
        auto_fixed_count: result.auto_fixed,
        agent_fixed_count: result.agent_fixed,
        remaining_count: remaining,
        issues_created: result.issues_created,
        commit_hash: result.commit_hash,
        pushed: result.pushed,
        crates_scanned: Vec::new(),
    };

    output_report(&report, config)
}

/// Cross-stack kaizen: discover all batuta stack crates, scan each, fix per-crate
async fn handle_kaizen_cross_stack(path: &Path, config: &KaizenConfig) -> Result<()> {
    eprintln!("Kaizen: cross-stack mode, discovering crates ...");

    let crates = discover_workspace_crates(path, None);
    let crate_names: Vec<String> = crates.iter().map(|c| c.name.clone()).collect();

    eprintln!(
        "Kaizen: discovered {} crates: {}",
        crates.len(),
        crate_names.join(", ")
    );

    // Phase 1: Scan all crates
    let mut all_findings = Vec::new();
    for ci in &crates {
        eprintln!("\nKaizen: scanning {} ({}) ...", ci.name, ci.path.display());
        let mut crate_findings = scan_crate(&ci.path, Some(&ci.name), config)?;
        enrich_with_tarantula(&ci.path, &mut crate_findings);
        eprintln!("  {} findings in {}", crate_findings.len(), ci.name);
        all_findings.extend(crate_findings);
    }

    sort_findings(&mut all_findings);
    eprintln!(
        "\nKaizen: found {} total issues across stack",
        all_findings.len()
    );

    // Phase 3-7: Apply fixes, commit, agent, per-crate
    let mut accum = CrossStackAccumulator::default();

    for ci in &crates {
        process_crate_phases(ci, config, &mut all_findings, &mut accum)?;
    }

    // GitHub issues (filed from the primary project)
    if !config.dry_run && config.create_issues {
        let unfixed: Vec<&KaizenFinding> = all_findings.iter().filter(|f| !f.fix_applied).collect();
        if !unfixed.is_empty() {
            eprintln!(
                "Kaizen: filing {} GitHub issues for unfixed findings",
                unfixed.len()
            );
            accum.issues = create_github_issues(path, &unfixed);
        }
    }

    // Build report
    let remaining = all_findings.iter().filter(|f| !f.fix_applied).count();
    let report = KaizenReport {
        findings: all_findings,
        auto_fixed_count: accum.auto_fixed,
        agent_fixed_count: accum.agent_fixed,
        remaining_count: remaining,
        issues_created: accum.issues,
        commit_hash: accum.last_commit,
        pushed: accum.pushed,
        crates_scanned: crate_names,
    };

    output_report(&report, config)
}

/// Check if a crate has any fixable findings
fn crate_has_fixable(findings: &[KaizenFinding], crate_name: &str) -> bool {
    findings
        .iter()
        .any(|f| f.crate_name.as_deref() == Some(crate_name) && f.auto_fixable && !f.fix_applied)
}

/// Check if a crate had any fixes applied
fn crate_had_fixes(findings: &[KaizenFinding], crate_name: &str) -> bool {
    findings
        .iter()
        .any(|f| f.crate_name.as_deref() == Some(crate_name) && f.fix_applied)
}

/// Try to push changes for a crate, logging results
fn try_push_crate(crate_path: &Path, crate_name: &str, accum: &mut CrossStackAccumulator) {
    match push_changes(crate_path) {
        Ok(true) => {
            accum.pushed = true;
            eprintln!("Kaizen: pushed changes for {}", crate_name);
        }
        Ok(false) => eprintln!("Kaizen: push failed for {}", crate_name),
        Err(e) => eprintln!("Kaizen: push error for {}: {}", crate_name, e),
    }
}

/// Process fix/commit/agent/push phases for a single crate in cross-stack mode
fn process_crate_phases(
    ci: &crate::cli::handlers::comply_handlers::cross_crate_handlers::CrateInfo,
    config: &KaizenConfig,
    findings: &mut [KaizenFinding],
    accum: &mut CrossStackAccumulator,
) -> Result<()> {
    let crate_name = &ci.name;
    let crate_path = &ci.path;
    let has_fixable = crate_has_fixable(findings, crate_name);

    if !has_fixable && !config.auto_agent {
        return Ok(());
    }

    if !config.dry_run && has_fixable {
        let fixed = apply_safe_fixes_for_crate(crate_path, crate_name, findings)?;
        if fixed > 0 {
            eprintln!("Kaizen: auto-fixed {} issues in {}", fixed, crate_name);
            accum.auto_fixed += fixed;
            if config.commit {
                accum.last_commit = commit_changes(
                    crate_path,
                    &format!("kaizen: auto-fix {} issues in {}", fixed, crate_name),
                )?;
            }
        }
    }

    if !config.dry_run && config.auto_agent {
        accum.agent_fixed += spawn_agents_for_crate(
            crate_path,
            crate_name,
            findings,
            config.max_agents,
            config.commit,
        )?;
    }

    if !config.dry_run && config.push && crate_had_fixes(findings, crate_name) {
        try_push_crate(crate_path, crate_name, accum);
    }

    Ok(())
}

/// Apply phases 3-7 for a single project (non-cross-stack)
fn apply_phases(
    path: &Path,
    findings: &mut Vec<KaizenFinding>,
    config: &KaizenConfig,
) -> Result<PhaseResult> {
    // Phase 3: Apply deterministic fixes
    let mut auto_fixed = 0usize;
    if !config.dry_run {
        auto_fixed = apply_safe_fixes(path, findings)?;
        if auto_fixed > 0 {
            eprintln!("Kaizen: auto-fixed {} issues", auto_fixed);
        }
    }

    // Phase 4: Commit
    let mut commit_hash = None;
    if !config.dry_run && config.commit && auto_fixed > 0 {
        commit_hash = commit_changes(path, &format!("kaizen: auto-fix {} issues", auto_fixed))?;
    }

    // Phase 5: Agent delegation
    let mut agent_fixed = 0usize;
    if !config.dry_run && config.auto_agent {
        let agent_count = findings
            .iter()
            .filter(|f| !f.fix_applied && f.agent_fixable)
            .count();
        if agent_count > 0 {
            eprintln!(
                "Kaizen: delegating {} issues to AI agents (max {})",
                agent_count, config.max_agents
            );
            agent_fixed = spawn_agents(path, findings, config.max_agents, config.commit)?;
        }
    }

    // Phase 6: GitHub issues
    let mut issues_created = Vec::new();
    if !config.dry_run && config.create_issues {
        let unfixed: Vec<&KaizenFinding> = findings.iter().filter(|f| !f.fix_applied).collect();
        if !unfixed.is_empty() {
            eprintln!(
                "Kaizen: filing {} GitHub issues for unfixed findings",
                unfixed.len()
            );
            issues_created = create_github_issues(path, &unfixed);
        }
    }

    // Phase 7: Push
    let mut pushed = false;
    if !config.dry_run && config.push && (auto_fixed > 0 || agent_fixed > 0) {
        pushed = push_changes(path)?;
    }

    Ok(PhaseResult {
        auto_fixed,
        agent_fixed,
        commit_hash,
        issues_created,
        pushed,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_crate_name_serialization() {
        let finding = KaizenFinding {
            source: FindingSource::Clippy,
            severity: FindingSeverity::Medium,
            category: "test".to_string(),
            message: "test".to_string(),
            file: None,
            auto_fixable: false,
            agent_fixable: false,
            fix_applied: false,
            agent_prompt: None,
            suspiciousness_score: None,
            crate_name: Some("pmat".to_string()),
        };
        let json = serde_json::to_string(&finding).unwrap();
        assert!(json.contains("\"crate_name\":\"pmat\""));

        // None should be omitted
        let finding_no_crate = KaizenFinding {
            crate_name: None,
            ..finding
        };
        let json2 = serde_json::to_string(&finding_no_crate).unwrap();
        assert!(!json2.contains("crate_name"));
    }
}
