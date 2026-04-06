//! Kaizen git operations: commit, push, branch detection, GitHub issue creation.

use super::{FindingSeverity, FindingSource, GithubIssueRef, KaizenFinding};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub(crate) fn commit_changes(path: &Path, message: &str) -> Result<Option<String>> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
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

pub(crate) fn push_changes(path: &Path) -> Result<bool> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let branch = detect_default_branch(path);
    let status = Command::new("git")
        .args(["push", "origin", &branch])
        .current_dir(path)
        .status()
        .context("Failed to run git push")?;

    Ok(status.success())
}

/// Detect the current branch name for pushing.
/// Falls back to "master" if detection fails.
pub(crate) fn detect_default_branch(path: &Path) -> String {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // Try to get the current branch name
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(path)
        .output();

    if let Ok(o) = output {
        if o.status.success() {
            let branch = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !branch.is_empty() && branch != "HEAD" {
                return branch;
            }
        }
    }

    "master".to_string()
}

// ---------------------------------------------------------------------------
// GitHub issue creation
// ---------------------------------------------------------------------------

/// Create GitHub issues for unfixed findings via `gh issue create`.
pub(crate) fn create_github_issues(
    path: &Path,
    findings: &[&KaizenFinding],
) -> Vec<GithubIssueRef> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
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
                eprintln!(
                    "Kaizen: gh issue create failed: {}",
                    super::scanning::first_line(&stderr)
                );
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

pub(crate) fn truncate(s: &str, max: usize) -> String {
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

pub(crate) fn severity_to_labels(severity: FindingSeverity, source: FindingSource) -> Vec<String> {
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

pub(crate) fn extract_issue_number(url: &str) -> u64 {
    debug_assert!(!url.is_empty(), "url must not be empty");
    // URL format: https://github.com/org/repo/issues/123
    url.rsplit('/')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_default_branch() {
        // Should return a non-empty string (either current branch or "master" fallback)
        let branch = detect_default_branch(std::path::Path::new("."));
        assert!(!branch.is_empty());
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
            crate_name: None,
        };
        let body = format_issue_body(&finding);
        assert!(body.contains("**Severity**: High"));
        assert!(body.contains("**File**: `src/lib.rs`"));
        assert!(body.contains("TDG grade below A"));
        assert!(body.contains("Improve function quality"));
        assert!(body.contains("pmat kaizen"));
    }
}
