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

    if !config.skip_defects {
        let defect_findings = scan_defects(&path)?;
        findings.extend(defect_findings);
    }

    if !config.skip_github {
        let gh_findings = scan_github_issues(&path)?;
        findings.extend(gh_findings);
    }

    // Scan custom project scores from .pmat.yaml
    let custom_findings = scan_custom_scores(&path);
    findings.extend(custom_findings);

    // Phase 2: Enrich with tarantula suspiciousness from coverage data
    enrich_with_tarantula(&path, &mut findings);

    // Phase 2b: Sort by composite priority (severity * suspiciousness)
    findings.sort_by(|a, b| {
        composite_priority(b)
            .partial_cmp(&composite_priority(a))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

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

    // Phase 6: Create GitHub issues for unfixed findings
    let mut issues_created = Vec::new();
    if !config.dry_run && config.create_issues {
        let unfixed: Vec<&KaizenFinding> = findings.iter().filter(|f| !f.fix_applied).collect();

        if !unfixed.is_empty() {
            eprintln!(
                "Kaizen: filing {} GitHub issues for unfixed findings",
                unfixed.len()
            );
            issues_created = create_github_issues(&path, &unfixed);
        }
    }

    // Phase 7: Push if requested
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
        issues_created,
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
            suspiciousness_score: None,
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
            suspiciousness_score: None,
        });
    }

    Ok(findings)
}

/// Scan for PMAT compliance violations
fn scan_comply(path: &Path) -> Result<Vec<KaizenFinding>> {
    let output = Command::new("pmat")
        .args([
            "comply",
            "check",
            "-p",
            &path.to_string_lossy(),
            "-f",
            "json",
        ])
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

            let id = check.get("id").and_then(|s| s.as_str()).unwrap_or("CB-???");
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
                suspiciousness_score: None,
            });
        }
    }

    Ok(findings)
}

/// Scan for known defect patterns (batuta bug-hunt: unwrap, panic, unsafe, etc.)
fn scan_defects(path: &Path) -> Result<Vec<KaizenFinding>> {
    let output = Command::new("pmat")
        .args([
            "analyze",
            "defects",
            "-p",
            &path.to_string_lossy(),
            "--format",
            "json",
        ])
        .current_dir(path)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Ok(Vec::new()),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = match serde_json::from_str(&stdout) {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };

    let mut findings = Vec::new();

    let defects = json.get("defects").and_then(|d| d.as_array());
    let Some(defects) = defects else {
        return Ok(findings);
    };

    for defect in defects {
        let id = defect
            .get("id")
            .and_then(|s| s.as_str())
            .unwrap_or("DEFECT-???");
        let name = defect
            .get("name")
            .and_then(|s| s.as_str())
            .unwrap_or("Unknown defect");
        let sev_str = defect
            .get("severity")
            .and_then(|s| s.as_str())
            .unwrap_or("Medium");
        let fix = defect
            .get("fix_recommendation")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        let severity = match sev_str.to_lowercase().as_str() {
            "critical" => FindingSeverity::Critical,
            "high" => FindingSeverity::High,
            "low" => FindingSeverity::Low,
            _ => FindingSeverity::Medium,
        };

        let instances = defect.get("instances").and_then(|i| i.as_array());
        let instance_count = instances.map(|i| i.len()).unwrap_or(0);

        // Create one finding per defect pattern (not per instance) for actionability
        let first_file = instances
            .and_then(|insts| insts.first())
            .and_then(|inst| inst.get("file"))
            .and_then(|f| f.as_str())
            .map(String::from);

        findings.push(KaizenFinding {
            source: FindingSource::Defects,
            severity,
            category: format!("defect::{id}"),
            message: format!("{name} ({instance_count} instances)"),
            file: first_file.clone(),
            auto_fixable: false,
            agent_fixable: true,
            fix_applied: false,
            agent_prompt: Some(format!(
                "Fix defect pattern {id}: {name}. {fix} \
                 There are {instance_count} instances. \
                 Start with file {} and fix all instances. \
                 Run `pmat analyze defects` after fixing to verify.",
                first_file.as_deref().unwrap_or("the project")
            )),
            suspiciousness_score: None,
        });
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
            suspiciousness_score: None,
        });
    }

    Ok(findings)
}

/// Scan custom project scores from .pmat.yaml scoring plugins
fn scan_custom_scores(path: &Path) -> Vec<KaizenFinding> {
    let config = match crate::models::comply_config::PmatYamlConfig::load(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    if config.scoring.custom_scores.is_empty() {
        return Vec::new();
    }

    let mut findings = Vec::new();

    for score_def in &config.scoring.custom_scores {
        let output = Command::new("sh")
            .args(["-c", &score_def.command])
            .current_dir(path)
            .output();

        let output = match output {
            Ok(o) => o,
            Err(_) => continue,
        };

        if !output.status.success() {
            findings.push(KaizenFinding {
                source: FindingSource::Comply,
                severity: FindingSeverity::High,
                category: format!("score::{}", score_def.id),
                message: format!("{}: command failed", score_def.name),
                file: None,
                auto_fixable: false,
                agent_fixable: true,
                fix_applied: false,
                agent_prompt: Some(format!(
                    "Fix failing score check '{}': command `{}` failed. \
                     Investigate and fix the underlying issue.",
                    score_def.name, score_def.command
                )),
                suspiciousness_score: None,
            });
            continue;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Reuse the same score extraction from the comply handler
        let score = extract_score_from_command_output(&stdout);

        if let (Some(actual), Some(min)) = (score, score_def.min_score) {
            if actual < min {
                let severity = match score_def.severity {
                    crate::models::comply_config::CheckSeverity::Critical => {
                        FindingSeverity::Critical
                    }
                    crate::models::comply_config::CheckSeverity::Error => FindingSeverity::High,
                    crate::models::comply_config::CheckSeverity::Warning => FindingSeverity::Medium,
                    crate::models::comply_config::CheckSeverity::Info => FindingSeverity::Low,
                };
                findings.push(KaizenFinding {
                    source: FindingSource::Comply,
                    severity,
                    category: format!("score::{}", score_def.id),
                    message: format!(
                        "{}: score {:.1} below minimum {:.1}",
                        score_def.name, actual, min
                    ),
                    file: None,
                    auto_fixable: false,
                    agent_fixable: true,
                    fix_applied: false,
                    agent_prompt: Some(format!(
                        "Improve '{}' score from {:.1} to at least {:.1}. \
                         The score command is: `{}`",
                        score_def.name, actual, min, score_def.command
                    )),
                    suspiciousness_score: None,
                });
            }
        }
    }

    findings
}

/// Extract a numeric score from command output (JSON {"score": N} or "SCORE: N")
fn extract_score_from_command_output(output: &str) -> Option<f64> {
    for line in output.lines() {
        let line = line.trim();
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(score) = json.get("score").and_then(|s| s.as_f64()) {
                return Some(score);
            }
        }
    }
    for line in output.lines() {
        if let Some(rest) = line.trim().strip_prefix("SCORE:") {
            if let Ok(score) = rest.trim().parse::<f64>() {
                return Some(score);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tarantula suspiciousness enrichment
// ---------------------------------------------------------------------------

/// Compute composite priority: severity_weight * (1.0 + suspiciousness).
/// Critical=4, High=3, Medium=2, Low=1.
fn composite_priority(finding: &KaizenFinding) -> f32 {
    let severity_weight = match finding.severity {
        FindingSeverity::Critical => 4.0f32,
        FindingSeverity::High => 3.0,
        FindingSeverity::Medium => 2.0,
        FindingSeverity::Low => 1.0,
    };
    let suspiciousness = finding.suspiciousness_score.unwrap_or(0.0);
    severity_weight * (1.0 + suspiciousness)
}

/// Enrich findings with tarantula suspiciousness scores from LCOV coverage data.
/// Gracefully does nothing if no coverage data is available.
fn enrich_with_tarantula(path: &Path, findings: &mut [KaizenFinding]) {
    let lcov_candidates = [
        path.join("target/coverage/lcov.info"),
        path.join("target/llvm-cov/lcov.info"),
        path.join("coverage/lcov.info"),
        path.join("lcov.info"),
    ];

    let lcov_path = match lcov_candidates.iter().find(|p| p.exists()) {
        Some(p) => p,
        None => return, // No coverage data, skip enrichment
    };

    let content = match std::fs::read_to_string(lcov_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Parse LCOV into file -> line -> hit_count map
    let line_hits = parse_lcov_line_hits(&content);
    if line_hits.is_empty() {
        return;
    }

    // Build per-file suspiciousness: ratio of uncovered lines
    // Higher ratio = more suspicious (more untested code)
    for finding in findings.iter_mut() {
        if let Some(ref file) = finding.file {
            // Normalize path: strip leading "./" or project prefix
            let normalized = file.trim_start_matches("./");
            if let Some(hits) = line_hits.get(normalized) {
                let total = hits.len() as f32;
                if total > 0.0 {
                    let uncovered = hits.values().filter(|&&h| h == 0).count() as f32;
                    finding.suspiciousness_score = Some(uncovered / total);
                }
            }
        }
    }
}

/// Parse LCOV format into file -> (line -> hit_count) map.
fn parse_lcov_line_hits(
    content: &str,
) -> std::collections::HashMap<String, std::collections::HashMap<usize, u64>> {
    let mut result: std::collections::HashMap<String, std::collections::HashMap<usize, u64>> =
        std::collections::HashMap::new();
    let mut current_file = String::new();

    for line in content.lines() {
        if let Some(sf) = line.strip_prefix("SF:") {
            current_file = sf.trim().to_string();
        } else if let Some(da) = line.strip_prefix("DA:") {
            if current_file.is_empty() {
                continue;
            }
            let parts: Vec<&str> = da.split(',').collect();
            if parts.len() >= 2 {
                if let (Ok(line_no), Ok(hits)) =
                    (parts[0].parse::<usize>(), parts[1].parse::<u64>())
                {
                    result
                        .entry(current_file.clone())
                        .or_default()
                        .insert(line_no, hits);
                }
            }
        } else if line == "end_of_record" {
            current_file.clear();
        }
    }

    result
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
        if f.auto_fixable
            && (f.source == FindingSource::Clippy || f.source == FindingSource::Rustfmt)
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

    let hash = String::from_utf8_lossy(&hash_output.stdout)
        .trim()
        .to_string();
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
// GitHub issue creation
// ---------------------------------------------------------------------------

/// Create GitHub issues for unfixed findings via `gh issue create`.
fn create_github_issues(path: &Path, findings: &[&KaizenFinding]) -> Vec<GithubIssueRef> {
    let mut refs = Vec::new();

    for finding in findings {
        let title = truncate(
            &format!("kaizen: {} - {}", finding.category, finding.message),
            70,
        );
        let body = format_issue_body(finding);
        let labels = severity_to_labels(finding.severity, finding.source);

        let mut args = vec![
            "issue".to_string(),
            "create".to_string(),
            "--title".to_string(),
            title,
            "--body".to_string(),
            body,
        ];
        for label in &labels {
            args.push("--label".to_string());
            args.push(label.clone());
        }

        let output = Command::new("gh").args(&args).current_dir(path).output();

        match output {
            Ok(o) if o.status.success() => {
                let url = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let number = extract_issue_number(&url);
                refs.push(GithubIssueRef {
                    number,
                    url,
                    finding_category: finding.category.clone(),
                });
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                eprintln!("Kaizen: gh issue create failed: {}", first_line(&stderr));
                break; // gh auth or repo issue, stop trying
            }
            Err(e) => {
                eprintln!("Kaizen: gh not available: {e}");
                break;
            }
        }
    }

    refs
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

fn format_issue_body(finding: &KaizenFinding) -> String {
    let mut body = String::new();
    body.push_str(&format!("**Severity**: {:?}\n", finding.severity));
    body.push_str(&format!("**Source**: {:?}\n", finding.source));
    body.push_str(&format!("**Category**: `{}`\n", finding.category));
    if let Some(file) = &finding.file {
        body.push_str(&format!("**File**: `{}`\n", file));
    }
    body.push_str(&format!("\n## Details\n\n{}\n", finding.message));
    if let Some(prompt) = &finding.agent_prompt {
        body.push_str(&format!("\n## Suggested Fix\n\n{}\n", prompt));
    }
    body.push_str("\n---\n*Filed automatically by `pmat kaizen`*\n");
    body
}

fn severity_to_labels(severity: FindingSeverity, source: FindingSource) -> Vec<String> {
    let mut labels = vec!["kaizen".to_string()];
    labels.push(match severity {
        FindingSeverity::Critical => "priority:critical".to_string(),
        FindingSeverity::High => "priority:high".to_string(),
        FindingSeverity::Medium => "priority:medium".to_string(),
        FindingSeverity::Low => "priority:low".to_string(),
    });
    labels.push(match source {
        FindingSource::Clippy => "clippy".to_string(),
        FindingSource::Rustfmt => "rustfmt".to_string(),
        FindingSource::Comply => "comply".to_string(),
        FindingSource::Defects => "defect".to_string(),
        FindingSource::CoverageGap => "coverage".to_string(),
        FindingSource::GitHubIssue => "triage".to_string(),
    });
    labels
}

fn extract_issue_number(url: &str) -> u64 {
    // URL format: https://github.com/org/repo/issues/123
    url.rsplit('/')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
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
        "Kaizen Report: {} findings | {} auto-fixed | {} agent-fixed | {} remaining | {} issues filed\n",
        report.findings.len(),
        report.auto_fixed_count,
        report.agent_fixed_count,
        report.remaining_count,
        report.issues_created.len(),
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

    if !report.issues_created.is_empty() {
        out.push_str("\nIssues Created:\n");
        for issue in &report.issues_created {
            out.push_str(&format!(
                "  #{} {} ({})\n",
                issue.number, issue.url, issue.finding_category
            ));
        }
    }

    out
}

fn format_report_markdown(report: &KaizenReport) -> String {
    let mut out = String::new();

    out.push_str("# Kaizen Report\n\n");
    out.push_str(&format!(
        "| Metric | Count |\n|--------|-------|\n\
         | Findings | {} |\n| Auto-fixed | {} |\n| Agent-fixed | {} |\n| Remaining | {} |\n| Issues Filed | {} |\n\n",
        report.findings.len(),
        report.auto_fixed_count,
        report.agent_fixed_count,
        report.remaining_count,
        report.issues_created.len(),
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

    if !report.issues_created.is_empty() {
        out.push_str("\n## Issues Created\n\n");
        out.push_str("| # | Category | URL |\n");
        out.push_str("|---|----------|-----|\n");
        for issue in &report.issues_created {
            out.push_str(&format!(
                "| {} | `{}` | {} |\n",
                issue.number, issue.finding_category, issue.url
            ));
        }
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
                suspiciousness_score: None,
            }],
            auto_fixed_count: 1,
            agent_fixed_count: 0,
            remaining_count: 0,
            issues_created: vec![],
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
            issues_created: vec![],
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
                suspiciousness_score: None,
            }],
            auto_fixed_count: 0,
            agent_fixed_count: 0,
            remaining_count: 1,
            issues_created: vec![],
            commit_hash: None,
            pushed: false,
        };
        let md = format_report_markdown(&report);
        assert!(md.contains("# Kaizen Report"));
        assert!(md.contains("github::issue#42"));
        assert!(md.contains("Agent"));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("exactly_ten", 11), "exactly_ten");
        assert_eq!(
            truncate("this is a long string that exceeds the limit", 20),
            "this is a long st..."
        );
    }

    #[test]
    fn test_extract_issue_number() {
        assert_eq!(
            extract_issue_number("https://github.com/org/repo/issues/42"),
            42
        );
        assert_eq!(
            extract_issue_number("https://github.com/org/repo/issues/1234"),
            1234
        );
        assert_eq!(extract_issue_number("not-a-url"), 0);
        assert_eq!(extract_issue_number(""), 0);
    }

    #[test]
    fn test_severity_to_labels() {
        let labels = severity_to_labels(FindingSeverity::Critical, FindingSource::Clippy);
        assert!(labels.contains(&"kaizen".to_string()));
        assert!(labels.contains(&"priority:critical".to_string()));
        assert!(labels.contains(&"clippy".to_string()));

        let labels = severity_to_labels(FindingSeverity::Low, FindingSource::Defects);
        assert!(labels.contains(&"priority:low".to_string()));
        assert!(labels.contains(&"defect".to_string()));
    }

    #[test]
    fn test_format_issue_body() {
        let finding = KaizenFinding {
            source: FindingSource::Comply,
            severity: FindingSeverity::High,
            category: "comply::CB-200".to_string(),
            message: "TDG grade below A".to_string(),
            file: Some("src/lib.rs".to_string()),
            auto_fixable: false,
            agent_fixable: true,
            fix_applied: false,
            agent_prompt: Some("Improve function quality".to_string()),
            suspiciousness_score: None,
        };
        let body = format_issue_body(&finding);
        assert!(body.contains("**Severity**: High"));
        assert!(body.contains("**File**: `src/lib.rs`"));
        assert!(body.contains("TDG grade below A"));
        assert!(body.contains("Improve function quality"));
        assert!(body.contains("pmat kaizen"));
    }

    #[test]
    fn test_report_with_issues_created() {
        let report = KaizenReport {
            findings: vec![],
            auto_fixed_count: 0,
            agent_fixed_count: 0,
            remaining_count: 0,
            issues_created: vec![GithubIssueRef {
                number: 42,
                url: "https://github.com/org/repo/issues/42".to_string(),
                finding_category: "comply::CB-200".to_string(),
            }],
            commit_hash: None,
            pushed: false,
        };

        let text = format_report_text(&report);
        assert!(text.contains("1 issues filed"));
        assert!(text.contains("#42"));
        assert!(text.contains("comply::CB-200"));

        let md = format_report_markdown(&report);
        assert!(md.contains("Issues Filed | 1"));
        assert!(md.contains("## Issues Created"));
    }

    #[test]
    fn test_composite_priority_severity_only() {
        let finding = KaizenFinding {
            source: FindingSource::Clippy,
            severity: FindingSeverity::Critical,
            category: "test".to_string(),
            message: "test".to_string(),
            file: None,
            auto_fixable: false,
            agent_fixable: false,
            fix_applied: false,
            agent_prompt: None,
            suspiciousness_score: None,
        };
        // Critical=4.0 * (1.0 + 0.0) = 4.0
        assert!((composite_priority(&finding) - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_composite_priority_with_suspiciousness() {
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
            suspiciousness_score: Some(0.8),
        };
        // Medium=2.0 * (1.0 + 0.8) = 3.6
        assert!((composite_priority(&finding) - 3.6).abs() < 0.001);
    }

    #[test]
    fn test_composite_priority_ordering() {
        // High severity, no suspiciousness: 3.0 * 1.0 = 3.0
        let high_no_sus = KaizenFinding {
            source: FindingSource::Clippy,
            severity: FindingSeverity::High,
            category: "a".to_string(),
            message: "a".to_string(),
            file: None,
            auto_fixable: false,
            agent_fixable: false,
            fix_applied: false,
            agent_prompt: None,
            suspiciousness_score: None,
        };
        // Medium severity, high suspiciousness: 2.0 * (1.0 + 0.9) = 3.8
        let med_high_sus = KaizenFinding {
            source: FindingSource::Clippy,
            severity: FindingSeverity::Medium,
            category: "b".to_string(),
            message: "b".to_string(),
            file: None,
            auto_fixable: false,
            agent_fixable: false,
            fix_applied: false,
            agent_prompt: None,
            suspiciousness_score: Some(0.9),
        };
        // Medium with high suspiciousness should rank higher
        assert!(composite_priority(&med_high_sus) > composite_priority(&high_no_sus));
    }

    #[test]
    fn test_parse_lcov_line_hits() {
        let lcov = "\
SF:src/main.rs\n\
DA:1,5\n\
DA:2,0\n\
DA:3,10\n\
end_of_record\n\
SF:src/lib.rs\n\
DA:10,0\n\
DA:20,3\n\
end_of_record\n";

        let hits = parse_lcov_line_hits(lcov);
        assert_eq!(hits.len(), 2);

        let main_hits = hits.get("src/main.rs").unwrap();
        assert_eq!(main_hits.get(&1), Some(&5));
        assert_eq!(main_hits.get(&2), Some(&0));
        assert_eq!(main_hits.get(&3), Some(&10));

        let lib_hits = hits.get("src/lib.rs").unwrap();
        assert_eq!(lib_hits.get(&10), Some(&0));
        assert_eq!(lib_hits.get(&20), Some(&3));
    }

    #[test]
    fn test_parse_lcov_empty() {
        let hits = parse_lcov_line_hits("");
        assert!(hits.is_empty());
    }

    #[test]
    fn test_extract_score_from_command_output_json() {
        assert_eq!(
            extract_score_from_command_output(r#"{"score": 95.5}"#),
            Some(95.5)
        );
        assert_eq!(
            extract_score_from_command_output("some output\n{\"score\": 42.0}\nmore output"),
            Some(42.0)
        );
    }

    #[test]
    fn test_extract_score_from_command_output_score_prefix() {
        assert_eq!(extract_score_from_command_output("SCORE: 88.3"), Some(88.3));
        assert_eq!(
            extract_score_from_command_output("running tests...\nSCORE: 100"),
            Some(100.0)
        );
    }

    #[test]
    fn test_extract_score_from_command_output_no_score() {
        assert_eq!(extract_score_from_command_output("no score here"), None);
        assert_eq!(extract_score_from_command_output(""), None);
    }
}
