#![cfg_attr(coverage_nightly, coverage(off))]
//! Kaizen Handler - Autonomous continuous improvement (Toyota Way)
//!
//! Orchestrates deterministic fixes (clippy --fix, cargo fmt) and optional
//! AI sub-agent delegation for complex issues. One-shot by default:
//! scan → fix → report → exit.

use crate::cli::commands::KaizenOutputFormat;
use anyhow::{Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Configuration for a kaizen run
pub struct KaizenConfig {
    pub path: PathBuf,
    pub dry_run: bool,
    pub commit: bool,
    pub push: bool,
    pub auto_agent: bool,
    pub max_agents: usize,
    pub format: KaizenOutputFormat,
    pub output: Option<PathBuf>,
    pub skip_clippy: bool,
    pub skip_fmt: bool,
    pub skip_comply: bool,
    pub skip_github: bool,
}

/// Source of a kaizen finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSource {
    Clippy,
    Rustfmt,
    Comply,
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
}

/// Report from a kaizen run
#[derive(Debug, Clone, Serialize)]
pub struct KaizenReport {
    pub findings: Vec<KaizenFinding>,
    pub auto_fixed_count: usize,
    pub agent_fixed_count: usize,
    pub remaining_count: usize,
    pub commit_hash: Option<String>,
    pub pushed: bool,
}

/// Main kaizen handler entry point
pub async fn handle_kaizen(config: KaizenConfig) -> Result<()> {
    let path = config.path.canonicalize().unwrap_or(config.path.clone());
    eprintln!("Kaizen: scanning {} ...", path.display());

    // Phase 1: Scan for findings
    let mut findings = Vec::new();

    if !config.skip_clippy {
        let clippy_findings = scan_clippy(&path)?;
        findings.extend(clippy_findings);
    }

    if !config.skip_fmt {
        let fmt_findings = scan_rustfmt(&path)?;
        findings.extend(fmt_findings);
    }

    if !config.skip_comply {
        let comply_findings = scan_comply(&path)?;
        findings.extend(comply_findings);
    }

    if !config.skip_github {
        let gh_findings = scan_github_issues(&path)?;
        findings.extend(gh_findings);
    }

    // Phase 2: Sort by severity descending
    findings.sort_by(|a, b| b.severity.cmp(&a.severity));

    let total_found = findings.len();
    eprintln!("Kaizen: found {} issues", total_found);

    // Phase 3: Apply deterministic fixes (unless dry_run)
    let mut auto_fixed = 0usize;
    if !config.dry_run {
        auto_fixed = apply_safe_fixes(&path, &mut findings)?;
        if auto_fixed > 0 {
            eprintln!("Kaizen: auto-fixed {} issues", auto_fixed);
        }
    }

    // Phase 4: Commit deterministic fixes
    let mut commit_hash = None;
    if !config.dry_run && config.commit && auto_fixed > 0 {
        commit_hash = commit_changes(&path, &format!("kaizen: auto-fix {} issues", auto_fixed))?;
    }

    // Phase 5: Agent delegation (if enabled)
    let mut agent_fixed = 0usize;
    if !config.dry_run && config.auto_agent {
        let agent_findings: Vec<&KaizenFinding> = findings
            .iter()
            .filter(|f| !f.fix_applied && f.agent_fixable)
            .collect();

        if !agent_findings.is_empty() {
            eprintln!(
                "Kaizen: delegating {} issues to AI agents (max {})",
                agent_findings.len(),
                config.max_agents
            );
            agent_fixed = spawn_agents(&path, &mut findings, config.max_agents, config.commit)?;
        }
    }

    // Phase 6: Push if requested
    let mut pushed = false;
    if !config.dry_run && config.push && (auto_fixed > 0 || agent_fixed > 0) {
        pushed = push_changes(&path)?;
    }

    // Build report
    let remaining = findings.iter().filter(|f| !f.fix_applied).count();
    let report = KaizenReport {
        findings,
        auto_fixed_count: auto_fixed,
        agent_fixed_count: agent_fixed,
        remaining_count: remaining,
        commit_hash,
        pushed,
    };

    // Output report
    let output_text = format_report(&report, config.format);
    if let Some(output_path) = &config.output {
        std::fs::write(output_path, &output_text)
            .with_context(|| format!("Failed to write output to {}", output_path.display()))?;
        eprintln!("Kaizen: report written to {}", output_path.display());
    } else {
        println!("{output_text}");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Scanners
// ---------------------------------------------------------------------------

/// Scan for clippy warnings using JSON output
fn scan_clippy(path: &Path) -> Result<Vec<KaizenFinding>> {
    let output = Command::new("cargo")
        .args(["clippy", "--message-format=json", "--quiet"])
        .current_dir(path)
        .output()
        .context("Failed to run cargo clippy")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut findings = Vec::new();

    for line in stdout.lines() {
        let json: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if json.get("reason").and_then(|r| r.as_str()) != Some("compiler-message") {
            continue;
        }
        let Some(message) = json.get("message") else {
            continue;
        };
        let level = message.get("level").and_then(|l| l.as_str()).unwrap_or("");
        if level != "warning" {
            continue;
        }

        let code = message
            .get("code")
            .and_then(|c| c.get("code"))
            .and_then(|c| c.as_str())
            .unwrap_or("unknown");

        let rendered = message
            .get("rendered")
            .and_then(|r| r.as_str())
            .unwrap_or("");

        let file = extract_file_from_message(message);

        findings.push(KaizenFinding {
            source: FindingSource::Clippy,
            severity: FindingSeverity::Medium,
            category: format!("clippy::{code}"),
            message: first_line(rendered),
            file,
            auto_fixable: true,
            agent_fixable: false,
            fix_applied: false,
            agent_prompt: None,
        });
    }

    Ok(findings)
}

/// Scan for unformatted files
fn scan_rustfmt(path: &Path) -> Result<Vec<KaizenFinding>> {
    let output = Command::new("cargo")
        .args(["fmt", "--", "--check", "-l"])
        .current_dir(path)
        .output()
        .context("Failed to run cargo fmt --check")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut findings = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        findings.push(KaizenFinding {
            source: FindingSource::Rustfmt,
            severity: FindingSeverity::Low,
            category: "rustfmt::unformatted".to_string(),
            message: format!("File needs formatting: {line}"),
            file: Some(line.to_string()),
            auto_fixable: true,
            agent_fixable: false,
            fix_applied: false,
            agent_prompt: None,
        });
    }

    Ok(findings)
}

/// Scan for PMAT compliance violations
fn scan_comply(path: &Path) -> Result<Vec<KaizenFinding>> {
    let output = Command::new("pmat")
        .args(["comply", "check", "-p", &path.to_string_lossy(), "-f", "json"])
        .current_dir(path)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Ok(Vec::new()), // pmat not available
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = match serde_json::from_str(&stdout) {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };

    let mut findings = Vec::new();

    if let Some(checks) = json.get("checks").and_then(|c| c.as_array()) {
        for check in checks {
            let status = check.get("status").and_then(|s| s.as_str()).unwrap_or("");
            if status == "pass" || status == "skip" {
                continue;
            }

            let id = check
                .get("id")
                .and_then(|s| s.as_str())
                .unwrap_or("CB-???");
            let msg = check
                .get("message")
                .and_then(|s| s.as_str())
                .unwrap_or("Compliance violation");

            findings.push(KaizenFinding {
                source: FindingSource::Comply,
                severity: FindingSeverity::High,
                category: format!("comply::{id}"),
                message: msg.to_string(),
                file: None,
                auto_fixable: false,
                agent_fixable: true,
                fix_applied: false,
                agent_prompt: Some(format!(
                    "Fix PMAT compliance violation {id}: {msg}. \
                     Run `pmat comply check` after fixing to verify."
                )),
            });
        }
    }

    Ok(findings)
}

/// Scan for open GitHub issues
fn scan_github_issues(path: &Path) -> Result<Vec<KaizenFinding>> {
    let output = Command::new("gh")
        .args([
            "issue",
            "list",
            "--json",
            "number,title,labels",
            "--state",
            "open",
            "--limit",
            "20",
        ])
        .current_dir(path)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Ok(Vec::new()), // gh not available
    };

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let issues: Vec<serde_json::Value> = match serde_json::from_str(&stdout) {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };

    let mut findings = Vec::new();

    for issue in &issues {
        let number = issue.get("number").and_then(|n| n.as_u64()).unwrap_or(0);
        let title = issue
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Untitled");

        let is_bug = issue
            .get("labels")
            .and_then(|l| l.as_array())
            .map(|labels| {
                labels.iter().any(|l| {
                    l.get("name")
                        .and_then(|n| n.as_str())
                        .map(|n| n.to_lowercase().contains("bug"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        let severity = if is_bug {
            FindingSeverity::High
        } else {
            FindingSeverity::Medium
        };

        findings.push(KaizenFinding {
            source: FindingSource::GitHubIssue,
            severity,
            category: format!("github::issue#{number}"),
            message: format!("#{number}: {title}"),
            file: None,
            auto_fixable: false,
            agent_fixable: true,
            fix_applied: false,
            agent_prompt: Some(format!(
                "Fix GitHub issue #{number}: {title}. \
                 Read the issue with `gh issue view {number}` first. \
                 Run tests after fixing."
            )),
        });
    }

    Ok(findings)
}

// ---------------------------------------------------------------------------
// Auto-fix
// ---------------------------------------------------------------------------

/// Apply safe deterministic fixes (clippy --fix + cargo fmt), verify with cargo check.
/// Returns the number of fixes applied.
fn apply_safe_fixes(path: &Path, findings: &mut [KaizenFinding]) -> Result<usize> {
    let has_clippy = findings
        .iter()
        .any(|f| f.source == FindingSource::Clippy && f.auto_fixable);
    let has_fmt = findings
        .iter()
        .any(|f| f.source == FindingSource::Rustfmt && f.auto_fixable);

    if !has_clippy && !has_fmt {
        return Ok(0);
    }

    // Run clippy --fix
    if has_clippy {
        let status = Command::new("cargo")
            .args([
                "clippy",
                "--fix",
                "--allow-dirty",
                "--allow-staged",
                "--quiet",
            ])
            .current_dir(path)
            .status()
            .context("Failed to run cargo clippy --fix")?;

        if !status.success() {
            eprintln!("Kaizen: clippy --fix returned non-zero, reverting");
            let _ = Command::new("git")
                .args(["checkout", "--", "."])
                .current_dir(path)
                .status();
            return Ok(0);
        }
    }

    // Run cargo fmt
    if has_fmt {
        let status = Command::new("cargo")
            .args(["fmt"])
            .current_dir(path)
            .status()
            .context("Failed to run cargo fmt")?;

        if !status.success() {
            eprintln!("Kaizen: cargo fmt returned non-zero");
        }
    }

    // Verify with cargo check
    let check = Command::new("cargo")
        .args(["check", "--quiet"])
        .current_dir(path)
        .status()
        .context("Failed to run cargo check")?;

    if !check.success() {
        eprintln!("Kaizen: cargo check failed after fixes, reverting");
        let _ = Command::new("git")
            .args(["checkout", "--", "."])
            .current_dir(path)
            .status();
        return Ok(0);
    }

    // Mark applied
    let mut count = 0usize;
    for f in findings.iter_mut() {
        if f.auto_fixable && (f.source == FindingSource::Clippy || f.source == FindingSource::Rustfmt)
        {
            f.fix_applied = true;
            count += 1;
        }
    }

    Ok(count)
}

// ---------------------------------------------------------------------------
// Agent delegation
// ---------------------------------------------------------------------------

/// Spawn AI sub-agents for complex findings. Runs sequentially in v1.
fn spawn_agents(
    path: &Path,
    findings: &mut [KaizenFinding],
    max_agents: usize,
    commit: bool,
) -> Result<usize> {
    let mut fixed = 0usize;
    let mut attempted = 0usize;

    for finding in findings.iter_mut() {
        if attempted >= max_agents {
            break;
        }
        if finding.fix_applied || !finding.agent_fixable {
            continue;
        }
        let Some(prompt) = &finding.agent_prompt else {
            continue;
        };

        attempted += 1;
        eprintln!(
            "Kaizen: agent [{}/{}] {}",
            attempted, max_agents, finding.category
        );

        let status = Command::new("claude")
            .args([
                "-p",
                prompt,
                "--allowedTools",
                "Bash,Read,Edit,Write,Grep,Glob",
            ])
            .current_dir(path)
            .status();

        match status {
            Ok(s) if s.success() => {
                // Verify with cargo check
                let check = Command::new("cargo")
                    .args(["check", "--quiet"])
                    .current_dir(path)
                    .status();

                if check.map(|s| s.success()).unwrap_or(false) {
                    finding.fix_applied = true;
                    fixed += 1;

                    if commit {
                        let msg = format!("kaizen(agent): fix {}", finding.category);
                        let _ = commit_changes(path, &msg);
                    }
                } else {
                    eprintln!("Kaizen: agent fix broke cargo check, reverting");
                    let _ = Command::new("git")
                        .args(["checkout", "--", "."])
                        .current_dir(path)
                        .status();
                }
            }
            Ok(_) => {
                eprintln!("Kaizen: agent returned non-zero for {}", finding.category);
            }
            Err(e) => {
                eprintln!("Kaizen: failed to spawn claude: {e}");
                break; // claude CLI not available, stop trying
            }
        }
    }

    Ok(fixed)
}

// ---------------------------------------------------------------------------
// Git helpers
// ---------------------------------------------------------------------------

fn commit_changes(path: &Path, message: &str) -> Result<Option<String>> {
    let add = Command::new("git")
        .args(["add", "-A"])
        .current_dir(path)
        .status()
        .context("Failed to run git add")?;

    if !add.success() {
        return Ok(None);
    }

    let output = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(path)
        .output()
        .context("Failed to run git commit")?;

    if !output.status.success() {
        return Ok(None);
    }

    // Get the commit hash
    let hash_output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(path)
        .output()?;

    let hash = String::from_utf8_lossy(&hash_output.stdout).trim().to_string();
    Ok(Some(hash))
}

fn push_changes(path: &Path) -> Result<bool> {
    let status = Command::new("git")
        .args(["push", "origin", "master"])
        .current_dir(path)
        .status()
        .context("Failed to run git push")?;

    Ok(status.success())
}

// ---------------------------------------------------------------------------
// Report formatting
// ---------------------------------------------------------------------------

fn format_report(report: &KaizenReport, format: KaizenOutputFormat) -> String {
    match format {
        KaizenOutputFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        KaizenOutputFormat::Markdown => format_report_markdown(report),
        KaizenOutputFormat::Text => format_report_text(report),
    }
}

fn format_report_text(report: &KaizenReport) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "Kaizen Report: {} findings | {} auto-fixed | {} agent-fixed | {} remaining\n",
        report.findings.len(),
        report.auto_fixed_count,
        report.agent_fixed_count,
        report.remaining_count,
    ));

    if let Some(hash) = &report.commit_hash {
        out.push_str(&format!("Commit: {hash}\n"));
    }
    if report.pushed {
        out.push_str("Pushed: yes\n");
    }

    out.push('\n');

    for finding in &report.findings {
        let status = if finding.fix_applied {
            "FIXED"
        } else if finding.agent_fixable {
            "AGENT"
        } else {
            "TODO "
        };

        let severity = match finding.severity {
            FindingSeverity::Critical => "CRIT",
            FindingSeverity::High => "HIGH",
            FindingSeverity::Medium => "MED ",
            FindingSeverity::Low => "LOW ",
        };

        let file = finding.file.as_deref().unwrap_or("");
        out.push_str(&format!(
            "  [{status}] [{severity}] {category} {file}\n         {msg}\n",
            category = finding.category,
            msg = finding.message,
        ));
    }

    out
}

fn format_report_markdown(report: &KaizenReport) -> String {
    let mut out = String::new();

    out.push_str("# Kaizen Report\n\n");
    out.push_str(&format!(
        "| Metric | Count |\n|--------|-------|\n\
         | Findings | {} |\n| Auto-fixed | {} |\n| Agent-fixed | {} |\n| Remaining | {} |\n\n",
        report.findings.len(),
        report.auto_fixed_count,
        report.agent_fixed_count,
        report.remaining_count,
    ));

    if let Some(hash) = &report.commit_hash {
        out.push_str(&format!("**Commit**: `{hash}`\n\n"));
    }

    out.push_str("## Findings\n\n");
    out.push_str("| Status | Severity | Category | File | Message |\n");
    out.push_str("|--------|----------|----------|------|---------|\n");

    for finding in &report.findings {
        let status = if finding.fix_applied {
            "Fixed"
        } else if finding.agent_fixable {
            "Agent"
        } else {
            "Todo"
        };

        let severity = format!("{:?}", finding.severity);
        let file = finding.file.as_deref().unwrap_or("-");
        let msg = finding.message.replace('|', "\\|");

        out.push_str(&format!(
            "| {status} | {severity} | `{}` | `{file}` | {msg} |\n",
            finding.category,
        ));
    }

    out
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn extract_file_from_message(message: &serde_json::Value) -> Option<String> {
    message
        .get("spans")
        .and_then(|s| s.as_array())
        .and_then(|spans| spans.first())
        .and_then(|span| span.get("file_name"))
        .and_then(|f| f.as_str())
        .map(String::from)
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or(s).to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_severity_ordering() {
        assert!(FindingSeverity::Critical > FindingSeverity::High);
        assert!(FindingSeverity::High > FindingSeverity::Medium);
        assert!(FindingSeverity::Medium > FindingSeverity::Low);
    }

    #[test]
    fn test_first_line() {
        assert_eq!(first_line("hello\nworld"), "hello");
        assert_eq!(first_line("single"), "single");
        assert_eq!(first_line(""), "");
    }

    #[test]
    fn test_extract_file_from_message() {
        let msg = serde_json::json!({
            "spans": [{"file_name": "src/main.rs", "line_start": 10}]
        });
        assert_eq!(
            extract_file_from_message(&msg),
            Some("src/main.rs".to_string())
        );
    }

    #[test]
    fn test_extract_file_from_message_empty() {
        let msg = serde_json::json!({"spans": []});
        assert_eq!(extract_file_from_message(&msg), None);
    }

    #[test]
    fn test_format_report_text() {
        let report = KaizenReport {
            findings: vec![KaizenFinding {
                source: FindingSource::Clippy,
                severity: FindingSeverity::Medium,
                category: "clippy::needless_return".to_string(),
                message: "unnecessary return".to_string(),
                file: Some("src/lib.rs".to_string()),
                auto_fixable: true,
                agent_fixable: false,
                fix_applied: true,
                agent_prompt: None,
            }],
            auto_fixed_count: 1,
            agent_fixed_count: 0,
            remaining_count: 0,
            commit_hash: Some("abc1234".to_string()),
            pushed: false,
        };
        let text = format_report_text(&report);
        assert!(text.contains("1 findings"));
        assert!(text.contains("1 auto-fixed"));
        assert!(text.contains("FIXED"));
        assert!(text.contains("clippy::needless_return"));
    }

    #[test]
    fn test_format_report_json() {
        let report = KaizenReport {
            findings: vec![],
            auto_fixed_count: 0,
            agent_fixed_count: 0,
            remaining_count: 0,
            commit_hash: None,
            pushed: false,
        };
        let json = format_report(&report, KaizenOutputFormat::Json);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["auto_fixed_count"], 0);
    }

    #[test]
    fn test_format_report_markdown() {
        let report = KaizenReport {
            findings: vec![KaizenFinding {
                source: FindingSource::GitHubIssue,
                severity: FindingSeverity::High,
                category: "github::issue#42".to_string(),
                message: "Fix the widget".to_string(),
                file: None,
                auto_fixable: false,
                agent_fixable: true,
                fix_applied: false,
                agent_prompt: Some("Fix it".to_string()),
            }],
            auto_fixed_count: 0,
            agent_fixed_count: 0,
            remaining_count: 1,
            commit_hash: None,
            pushed: false,
        };
        let md = format_report_markdown(&report);
        assert!(md.contains("# Kaizen Report"));
        assert!(md.contains("github::issue#42"));
        assert!(md.contains("Agent"));
    }
}
