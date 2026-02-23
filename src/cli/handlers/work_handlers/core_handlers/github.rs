#![cfg_attr(coverage_nightly, coverage(off))]
// GitHub integration helpers for work command handlers

use crate::models::roadmap::RoadmapItem;
#[cfg(feature = "github-api")]
use crate::services::github_client::GitHubClient;
use anyhow::{Context, Result};
use std::path::PathBuf;

use super::types::GitHubIssueInfo;

/// Fetch GitHub issue details using octocrab (requires github-api feature)
#[cfg(feature = "github-api")]
pub(super) async fn fetch_github_issue(repo: &str, issue_num: u64) -> Result<GitHubIssueInfo> {
    // Try authenticated client first, fall back to unauthenticated
    let client = match GitHubClient::new(repo) {
        Ok(c) => c,
        Err(_) => {
            // GITHUB_TOKEN not set, try unauthenticated
            GitHubClient::new_unauthenticated(repo)?
        }
    };

    let issue = client.fetch_issue(issue_num).await?;
    Ok(GitHubIssueInfo {
        number: issue.number,
        title: issue.title,
        body: issue.body,
        labels: issue.labels.iter().map(|l| l.name.clone()).collect(),
    })
}

/// Fetch GitHub issue details using gh CLI (no octocrab dependency)
#[cfg(not(feature = "github-api"))]
pub(super) async fn fetch_github_issue(repo: &str, issue_num: u64) -> Result<GitHubIssueInfo> {
    use std::process::Command;

    let output = Command::new("gh")
        .args([
            "issue",
            "view",
            &issue_num.to_string(),
            "--repo",
            repo,
            "--json",
            "number,title,body,labels",
        ])
        .output()
        .context("Failed to run gh CLI. Is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh issue view failed: {}", stderr);
    }

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("Failed to parse gh output")?;

    Ok(GitHubIssueInfo {
        number: json["number"].as_u64().unwrap_or(issue_num),
        title: json["title"].as_str().unwrap_or("").to_string(),
        body: json["body"].as_str().map(|s| s.to_string()),
        labels: json["labels"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|l| l["name"].as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
    })
}

/// Create GitHub issue from roadmap item using octocrab (requires github-api feature)
#[cfg(feature = "github-api")]
pub(super) async fn create_github_issue_from_item(
    repo: &str,
    item: &RoadmapItem,
) -> Result<GitHubIssueInfo> {
    // Requires authentication
    let client = GitHubClient::new(repo)?;

    // Build issue body from acceptance criteria
    let body = if !item.acceptance_criteria.is_empty() {
        let criteria_md: Vec<String> = item
            .acceptance_criteria
            .iter()
            .map(|c| format!("- [ ] {}", c))
            .collect();

        format!(
            "## Acceptance Criteria\n\n{}\n\n---\n\n*Created via `pmat work start --create-github`*",
            criteria_md.join("\n")
        )
    } else {
        "*Created via `pmat work start --create-github`*".to_string()
    };

    let labels = if item.labels.is_empty() {
        None
    } else {
        Some(item.labels.clone())
    };

    let issue = client.create_issue(&item.title, &body, labels).await?;
    Ok(GitHubIssueInfo {
        number: issue.number,
        title: issue.title,
        body: issue.body,
        labels: issue.labels.iter().map(|l| l.name.clone()).collect(),
    })
}

/// Create GitHub issue from roadmap item using gh CLI (no octocrab dependency)
#[cfg(not(feature = "github-api"))]
pub(super) async fn create_github_issue_from_item(
    repo: &str,
    item: &RoadmapItem,
) -> Result<GitHubIssueInfo> {
    use std::process::Command;

    // Build issue body from acceptance criteria
    let body = if !item.acceptance_criteria.is_empty() {
        let criteria_md: Vec<String> = item
            .acceptance_criteria
            .iter()
            .map(|c| format!("- [ ] {}", c))
            .collect();

        format!(
            "## Acceptance Criteria\n\n{}\n\n---\n\n*Created via `pmat work start --create-github`*",
            criteria_md.join("\n")
        )
    } else {
        "*Created via `pmat work start --create-github`*".to_string()
    };

    let mut args = vec![
        "issue".to_string(),
        "create".to_string(),
        "--repo".to_string(),
        repo.to_string(),
        "--title".to_string(),
        item.title.clone(),
        "--body".to_string(),
        body.clone(),
    ];

    // Add labels if present
    for label in &item.labels {
        args.push("--label".to_string());
        args.push(label.clone());
    }

    let output = Command::new("gh")
        .args(&args)
        .output()
        .context("Failed to run gh CLI. Is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh issue create failed: {}", stderr);
    }

    // gh issue create outputs the URL, parse the issue number from it
    let stdout = String::from_utf8_lossy(&output.stdout);
    let issue_num: u64 = stdout
        .trim()
        .rsplit('/')
        .next()
        .and_then(|s| s.parse().ok())
        .context("Failed to parse issue number from gh output")?;

    Ok(GitHubIssueInfo {
        number: issue_num,
        title: item.title.clone(),
        body: Some(body),
        labels: item.labels.clone(),
    })
}

/// Detect GitHub repository from git remote
pub(super) fn detect_github_repo(project_path: &PathBuf) -> Result<Option<String>> {
    use std::process::Command;

    let output = Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .current_dir(project_path)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let url = String::from_utf8_lossy(&output.stdout);
            let url = url.trim();

            // Parse GitHub URL
            // https://github.com/owner/repo.git or git@github.com:owner/repo.git
            if let Some(repo) = parse_github_url(url) {
                return Ok(Some(repo));
            }
        }
    }

    Ok(None)
}

/// Parse GitHub URL to extract owner/repo
pub(super) fn parse_github_url(url: &str) -> Option<String> {
    // HTTPS: https://github.com/owner/repo.git
    if let Some(start) = url.find("github.com/") {
        let rest = url.get(start + 11..).unwrap_or_default();
        let repo = rest.trim_end_matches(".git");
        return Some(repo.to_string());
    }

    // SSH: git@github.com:owner/repo.git
    if let Some(start) = url.find("github.com:") {
        let rest = url.get(start + 11..).unwrap_or_default();
        let repo = rest.trim_end_matches(".git");
        return Some(repo.to_string());
    }

    None
}
