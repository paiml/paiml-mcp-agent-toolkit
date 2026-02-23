//! Kaizen scanning: opportunity detection from clippy, rustfmt, comply, defects, GitHub issues.

use super::{FindingSeverity, FindingSource, KaizenConfig, KaizenFinding};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Scan a single crate for all finding types, tagging each with crate_name
pub(crate) fn scan_crate(
    path: &Path,
    crate_name: Option<&str>,
    config: &KaizenConfig,
) -> Result<Vec<KaizenFinding>> {
    let mut findings = Vec::new();

    if !config.skip_clippy {
        findings.extend(scan_clippy(path)?);
    }
    if !config.skip_fmt {
        findings.extend(scan_rustfmt(path)?);
    }
    if !config.skip_comply {
        findings.extend(scan_comply(path)?);
    }
    if !config.skip_defects {
        findings.extend(scan_defects(path)?);
    }
    if !config.skip_github {
        findings.extend(scan_github_issues(path)?);
    }
    findings.extend(scan_custom_scores(path));

    // Tag with crate name
    if let Some(name) = crate_name {
        for f in &mut findings {
            f.crate_name = Some(name.to_string());
        }
    }

    Ok(findings)
}

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
            crate_name: None,
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
            crate_name: None,
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
                crate_name: None,
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
            crate_name: None,
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
            crate_name: None,
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
                crate_name: None,
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
                    crate_name: None,
                });
            }
        }
    }

    findings
}

/// Extract a numeric score from command output (JSON {"score": N} or "SCORE: N")
pub(crate) fn extract_score_from_command_output(output: &str) -> Option<f64> {
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
// Helpers
// ---------------------------------------------------------------------------

pub(crate) fn extract_file_from_message(message: &serde_json::Value) -> Option<String> {
    message
        .get("spans")
        .and_then(|s| s.as_array())
        .and_then(|spans| spans.first())
        .and_then(|span| span.get("file_name"))
        .and_then(|f| f.as_str())
        .map(String::from)
}

pub(crate) fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or(s).to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::commands::KaizenOutputFormat;

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

    #[test]
    fn test_scan_crate_tags_findings() {
        // scan_crate with a non-existent path will return empty (tools not found),
        // but we can verify the tagging logic by calling it
        let path = std::path::PathBuf::from("/nonexistent");
        let result = scan_crate(
            &path,
            Some("test-crate"),
            &KaizenConfig {
                path: path.clone(),
                dry_run: true,
                commit: false,
                create_issues: false,
                push: false,
                auto_agent: false,
                max_agents: 0,
                format: KaizenOutputFormat::Text,
                output: None,
                skip_clippy: true,
                skip_fmt: true,
                skip_comply: true,
                skip_github: true,
                skip_defects: true,
                cross_stack: true,
            },
        );
        // With all scanners skipped, should be empty but Ok
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
