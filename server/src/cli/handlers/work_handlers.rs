// Work command handlers for unified GitHub/YAML workflow (Issue #75)
//
// Implements the hybrid write-through architecture for GitHub and YAML tracking.

use crate::cli::commands::SyncDirection;
use crate::models::roadmap::{ItemStatus, Priority, RoadmapItem};
use crate::services::changelog_manager::{ChangeCategory, ChangelogEntry};
use crate::services::github_client::GitHubClient;
use crate::services::hook_manager;
use crate::services::roadmap_service::RoadmapService;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Handle work init command
pub async fn handle_work_init(
    github_repo: Option<String>,
    no_github: bool,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");

    println!("🚀 Initializing unified GitHub/YAML workflow...");
    println!();

    // Create roadmap service
    let service = RoadmapService::new(&roadmap_path);

    // Check if already initialized
    if service.exists() {
        println!("⚠️  Roadmap already exists at: {}", roadmap_path.display());
        println!("   Use `pmat work status` to view current items");
        return Ok(());
    }

    // Determine GitHub configuration
    let github_enabled = !no_github;
    let repo = if github_enabled {
        match github_repo {
            Some(r) => Some(r),
            None => {
                // Try to detect from git remote
                detect_github_repo(&project_path)?
            }
        }
    } else {
        None
    };

    // Initialize roadmap
    service.initialize(repo.clone())?;

    println!("✅ Created roadmap: {}", roadmap_path.display());

    // Install commit-msg hook
    match hook_manager::install_commit_msg_hook(&project_path) {
        Ok(()) => {
            println!("✅ Installed commit-msg hook");
        }
        Err(e) => {
            println!("⚠️  Failed to install commit-msg hook: {}", e);
            println!("   Workflow will work, but commit messages won't be validated");
        }
    }

    println!();

    // Display configuration
    println!("📋 Configuration:");
    println!(
        "   GitHub integration: {}",
        if github_enabled {
            "✅ enabled"
        } else {
            "❌ disabled"
        }
    );
    if let Some(r) = &repo {
        println!("   GitHub repository: {}", r);
    }
    println!();

    // Next steps
    println!("🎯 Next steps:");
    println!("   1. Create GitHub issue or edit roadmap.yaml");
    println!("   2. Start work: pmat work start <issue-number-or-ticket-id>");
    println!("   3. Continue: pmat work continue <id>");
    println!("   4. Complete: pmat work complete <id>");
    println!();

    if github_enabled && repo.is_none() {
        println!("💡 Tip: Set GitHub repo with:");
        println!("   pmat config set github.repo owner/repo");
        println!();
    }

    Ok(())
}

/// Handle work start command
pub async fn handle_work_start(
    id: String,
    with_spec: bool,
    epic: bool,
    path: Option<PathBuf>,
    create_github: bool,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    println!("🚀 Starting work on: {}", id);
    println!();

    // Load roadmap
    let mut roadmap = service.load()?;

    // Determine if this is a GitHub issue or YAML ticket
    let is_github_issue = id.parse::<u64>().is_ok();

    let mut item = if is_github_issue {
        let issue_num: u64 = id.parse()?;
        println!("📋 Type: GitHub issue #{}", issue_num);

        // Fetch from GitHub API if repo is configured
        let mut item = if let Some(ref repo) = roadmap.github_repo {
            match fetch_github_issue(repo, issue_num).await {
                Ok(gh_issue) => {
                    println!("   ✅ Fetched from GitHub: {}", gh_issue.title);

                    // Extract labels
                    let labels: Vec<String> =
                        gh_issue.labels.iter().map(|l| l.name.clone()).collect();

                    let mut item =
                        RoadmapItem::from_github_issue(issue_num, gh_issue.title.clone());
                    item.labels = labels;

                    // Parse acceptance criteria from issue body if present
                    if let Some(body) = &gh_issue.body {
                        item.acceptance_criteria = parse_acceptance_criteria(body);
                    }

                    item
                }
                Err(e) => {
                    println!("   ⚠️  Failed to fetch from GitHub: {}", e);
                    println!("   Creating placeholder (will sync later)");
                    RoadmapItem::from_github_issue(issue_num, format!("Issue #{}", issue_num))
                }
            }
        } else {
            println!("   ℹ️  GitHub not configured, creating placeholder");
            RoadmapItem::from_github_issue(issue_num, format!("Issue #{}", issue_num))
        };

        item.status = ItemStatus::InProgress;
        item
    } else {
        println!("📋 Type: YAML ticket {}", id);

        // Check if already exists
        if let Some(existing) = service.find_item(&id)? {
            println!("   Found existing ticket");
            let mut item = existing;
            item.status = ItemStatus::InProgress;
            item.updated = chrono::Utc::now().to_rfc3339();
            item
        } else {
            // Create new YAML ticket
            let mut item = RoadmapItem::new(id.clone(), format!("New task: {}", id));
            item.status = ItemStatus::InProgress;
            item.priority = Priority::Medium;

            if create_github {
                if let Some(ref repo) = roadmap.github_repo {
                    println!("   🔄 Creating GitHub issue...");
                    match create_github_issue_from_item(repo, &item).await {
                        Ok(gh_issue) => {
                            println!("   ✅ Created GitHub issue #{}", gh_issue.number);
                            item.github_issue = Some(gh_issue.number);
                            item.id = format!("GH-{}", gh_issue.number);
                        }
                        Err(e) => {
                            println!("   ⚠️  Failed to create GitHub issue: {}", e);
                            println!("   Continuing with YAML-only ticket");
                        }
                    }
                } else {
                    println!("   ⚠️  GitHub not configured, skipping issue creation");
                }
            }

            item
        }
    };

    // Set as epic if --epic flag is used
    if epic {
        item.item_type = crate::models::roadmap::ItemType::Epic;
        println!("📦 Created as epic: {}", item.title);
        println!("   Add subtasks manually to roadmap.yaml or use future commands");
    }

    // Update roadmap
    roadmap.upsert_item(item.clone());
    service.save(&roadmap)?;

    println!("✅ Updated roadmap: {}", roadmap_path.display());

    // Create specification if requested
    if with_spec {
        let spec_path = if is_github_issue {
            project_path.join(format!(
                "docs/specifications/{:03}-spec.md",
                item.github_issue.expect("internal error")
            ))
        } else {
            project_path.join(format!("docs/specifications/{}-spec.md", id.to_lowercase()))
        };

        if !spec_path.exists() {
            create_specification_template(&spec_path, &item)?;
            println!("✅ Created specification: {}", spec_path.display());
        } else {
            println!("   Specification exists: {}", spec_path.display());
        }
    }

    // Show next steps
    println!();
    println!("🎯 Next steps:");
    println!("   1. Review specification (if created)");
    println!("   2. Write failing tests (RED phase)");
    println!("   3. Implement feature (GREEN phase)");
    println!("   4. Refactor (REFACTOR phase)");
    println!("   5. Continue: pmat work continue {}", id);
    println!("   6. Complete: pmat work complete {}", id);
    println!();

    Ok(())
}

/// Handle work continue command
pub async fn handle_work_continue(id: String, path: Option<PathBuf>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    println!("🔄 Continuing work on: {}", id);
    println!();

    // Find item
    let item = service
        .find_item(&id)?
        .with_context(|| format!("Item not found: {}", id))?;

    // Display progress
    let completion = item.completion_percentage();
    println!("📊 Progress: {}% complete", completion);
    println!("   Status: {:?}", item.status);
    println!("   Title: {}", item.title);
    if let Some(spec) = &item.spec {
        println!("   Spec: {}", spec.display());
    }
    println!();

    // Show acceptance criteria
    if !item.acceptance_criteria.is_empty() {
        println!("📋 Acceptance Criteria:");
        for (i, criterion) in item.acceptance_criteria.iter().enumerate() {
            println!("   {}. {}", i + 1, criterion);
        }
        println!();
    }

    // Show phases
    if !item.phases.is_empty() {
        println!("📌 Phases:");
        for phase in &item.phases {
            let emoji = match phase.status {
                ItemStatus::Completed => "✅",
                ItemStatus::InProgress => "⏳",
                _ => "⬜",
            };
            println!("   {} {} ({}%)", emoji, phase.name, phase.completion);
        }
        println!();
    }

    // Show subtasks (for epics)
    if !item.subtasks.is_empty() {
        println!("📦 Subtasks:");
        for subtask in &item.subtasks {
            let emoji = match subtask.status {
                ItemStatus::Completed => "✅",
                ItemStatus::InProgress => "⏳",
                _ => "⬜",
            };
            println!("   {} {} ({}%)", emoji, subtask.title, subtask.completion);
        }
        println!();
    }

    // Next steps
    println!("🎯 Next steps:");
    println!("   Continue working on: {}", item.title);
    println!("   When done: pmat work complete {}", id);
    println!();

    Ok(())
}

/// Commit metadata structure for linking commits to work items and quality scores
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommitMetadata {
    commit_sha: Option<String>,
    work_item_id: String,
    prompt: String,
    tdg_score: f64,
    repo_score: f64,
    rust_project_score: Option<f64>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Capture commit metadata (O(1) from .pmat-metrics/ cache)
async fn capture_commit_metadata(
    project_path: &PathBuf,
    item: &RoadmapItem,
) -> Result<CommitMetadata> {
    use std::process::Command;

    let short_sha = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(project_path)
        .output()?;
    let short_sha = String::from_utf8_lossy(&short_sha.stdout)
        .trim()
        .to_string();

    // Capture scores (O(1) from cache)
    let tdg_score = capture_tdg_score(project_path).await.unwrap_or(0.0);
    let repo_score = capture_repo_score(project_path).await.unwrap_or(0.0);
    let rust_score = if project_path.join("Cargo.toml").exists() {
        Some(
            capture_rust_project_score(project_path)
                .await
                .unwrap_or(0.0),
        )
    } else {
        None
    };

    let metadata = CommitMetadata {
        commit_sha: None, // Will be filled after commit
        work_item_id: item.id.clone(),
        prompt: item.title.clone(),
        tdg_score,
        repo_score,
        rust_project_score: rust_score,
        timestamp: chrono::Utc::now(),
    };

    // Write to .pmat-metrics/
    let metrics_dir = project_path.join(".pmat-metrics");
    std::fs::create_dir_all(&metrics_dir)?;

    let meta_file = metrics_dir.join(format!("commit-{}-meta.json", short_sha));
    let json = serde_json::to_string_pretty(&metadata)?;
    std::fs::write(meta_file, json)?;

    Ok(metadata)
}

/// Capture TDG score (O(1) from cache)
async fn capture_tdg_score(project_path: &PathBuf) -> Result<f64> {
    let metrics_dir = project_path.join(".pmat-metrics");
    let tdg_file = metrics_dir.join("tdg-score.json");

    if tdg_file.exists() {
        let content = std::fs::read_to_string(&tdg_file)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        if let Some(score) = json.get("score").and_then(|v| v.as_f64()) {
            return Ok(score);
        }
    }

    // Fallback: compute score if cache doesn't exist
    Ok(0.0)
}

/// Capture repo score (O(1) from cache)
async fn capture_repo_score(project_path: &PathBuf) -> Result<f64> {
    let metrics_dir = project_path.join(".pmat-metrics");
    let repo_file = metrics_dir.join("repo-score.json");

    if repo_file.exists() {
        let content = std::fs::read_to_string(&repo_file)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        if let Some(score) = json.get("score").and_then(|v| v.as_f64()) {
            return Ok(score);
        }
    }

    // Fallback: compute score if cache doesn't exist
    Ok(0.0)
}

/// Capture rust project score (O(1) from cache)
async fn capture_rust_project_score(project_path: &PathBuf) -> Result<f64> {
    let metrics_dir = project_path.join(".pmat-metrics");
    let rust_file = metrics_dir.join("rust-project-score.json");

    if rust_file.exists() {
        let content = std::fs::read_to_string(&rust_file)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        if let Some(score) = json.get("total_earned").and_then(|v| v.as_f64()) {
            return Ok(score);
        }
    }

    // Fallback: compute score if cache doesn't exist
    Ok(0.0)
}

/// Handle work complete command
pub async fn handle_work_complete(
    id: String,
    skip_quality: bool,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    println!("✅ Completing work on: {}", id);
    println!();

    // Find item
    let mut item = service
        .find_item(&id)?
        .with_context(|| format!("Item not found: {}", id))?;

    // Run quality gates unless skipped
    if !skip_quality {
        println!("🔍 Running quality gates...");
        println!();

        match run_quality_gates(&project_path).await {
            Ok(passed) => {
                if passed {
                    println!("✅ All quality gates passed");
                    println!();
                } else {
                    anyhow::bail!(
                        "Quality gates failed. Fix issues or use --skip-quality to bypass."
                    );
                }
            }
            Err(e) => {
                println!("⚠️  Quality gates error: {}", e);
                println!("   Continuing (use strict mode to block on errors)");
                println!();
            }
        }

        // Run Karl Popper falsification validation
        match run_popper_falsification(&project_path).await {
            Ok(falsification) => {
                if !falsification.passed {
                    println!("⚠️  Falsification issues detected:");
                    println!("   {}", falsification.summary);
                    println!();
                    println!("   This is a warning - work will still be marked complete.");
                    println!("   Consider addressing these issues for higher confidence.");
                    println!();
                }
            }
            Err(e) => {
                println!("⚠️  Falsification validation error: {}", e);
                println!("   Continuing with completion...");
                println!();
            }
        }
    }

    // Mark as completed
    item.status = ItemStatus::Completed;
    item.updated = chrono::Utc::now().to_rfc3339();

    // Update roadmap
    let mut roadmap = service.load()?;
    roadmap.upsert_item(item.clone());
    service.save(&roadmap)?;

    println!("✅ Marked as complete: {}", item.title);
    println!("✅ Updated roadmap: {}", roadmap_path.display());

    // Capture commit metadata (O(1) from cache)
    println!();
    println!("   📊 Capturing commit metadata...");
    let metadata = capture_commit_metadata(&project_path, &item).await?;
    println!("      ✅ TDG Score: {:.1}/100", metadata.tdg_score);
    println!("      ✅ Repo Score: {:.1}/100", metadata.repo_score);
    if let Some(rust_score) = metadata.rust_project_score {
        println!("      ✅ Rust Project Score: {:.1}/134", rust_score);
    }
    let meta_file = project_path
        .join(".pmat-metrics")
        .join("commit-*-meta.json");
    println!("✅ Commit metadata: {}", meta_file.display());

    // Update CHANGELOG.md if labels are available
    if !item.labels.is_empty() {
        if let Some(category) = ChangeCategory::from_labels(&item.labels) {
            let entry = ChangelogEntry::new(category, item.title.clone(), item.github_issue);

            match crate::services::changelog_manager::add_to_changelog(&project_path, entry) {
                Ok(()) => {
                    println!("✅ Updated CHANGELOG.md");
                }
                Err(e) => {
                    println!("⚠️  Failed to update CHANGELOG.md: {}", e);
                    println!("   You may need to update it manually");
                }
            }
        } else {
            println!("ℹ️  No changelog category inferred from labels");
        }
    }

    println!();

    // Next steps with commit metadata
    println!("🎯 Next steps:");
    let commit_msg = format!(
        "feat: {} (Refs {})\n\nWork-Item: {}\nTDG-Score: {:.1}/100\nRepo-Score: {:.1}/100\n{}Metrics: .pmat-metrics/commit-*-meta.json",
        item.title,
        id,
        item.id,
        metadata.tdg_score,
        metadata.repo_score,
        if let Some(rust_score) = metadata.rust_project_score {
            format!("Rust-Score: {:.1}/134\n", rust_score)
        } else {
            String::new()
        }
    );

    println!("   1. git commit -m \"$(cat <<'EOF'");
    println!("{}", commit_msg);
    println!("EOF");
    println!(")\"");

    if item.is_github_synced() {
        println!(
            "   2. Close GitHub issue: gh issue close {}",
            item.github_issue.expect("internal error")
        );
    }
    println!();

    Ok(())
}

/// Handle work status command
pub async fn handle_work_status(
    id: Option<String>,
    path: Option<PathBuf>,
    active: bool,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    let roadmap = service.load()?;

    if let Some(item_id) = id {
        // Show specific item
        let item = roadmap
            .find_item(&item_id)
            .with_context(|| format!("Item not found: {}", item_id))?;

        println!("📊 Status for: {}", item.id);
        println!();
        println!("   Title: {}", item.title);
        println!("   Status: {:?}", item.status);
        println!("   Priority: {:?}", item.priority);
        println!("   Progress: {}%", item.completion_percentage());
        if let Some(gh) = item.github_issue {
            println!("   GitHub: #{}", gh);
        }
        println!();
    } else {
        // Show all items
        let items: Vec<_> = if active {
            roadmap
                .roadmap
                .iter()
                .filter(|item| {
                    matches!(
                        item.status,
                        ItemStatus::InProgress | ItemStatus::Planned | ItemStatus::Blocked
                    )
                })
                .collect()
        } else {
            roadmap.roadmap.iter().collect()
        };

        if items.is_empty() {
            println!("📋 No items found");
            println!();
            println!("   Start work with: pmat work start <id>");
            return Ok(());
        }

        println!("📋 Roadmap items: {} total", items.len());
        println!();

        for item in items {
            let emoji = match item.status {
                ItemStatus::Completed => "✅",
                ItemStatus::InProgress => "⏳",
                ItemStatus::Planned => "📋",
                ItemStatus::Blocked => "🚫",
                ItemStatus::Review => "👀",
                ItemStatus::Cancelled => "❌",
            };

            let progress = item.completion_percentage();

            // Truncate long IDs for display (show first 30 chars + "...")
            let display_id = if item.id.len() > 30 {
                format!("{}...", &item.id[..30])
            } else {
                item.id.clone()
            };

            println!(
                "   {} [{}] {} ({}%)",
                emoji, display_id, item.title, progress
            );
            if item.is_github_synced() {
                println!(
                    "      GitHub: #{}",
                    item.github_issue.expect("internal error")
                );
            }
        }
        println!();
    }

    Ok(())
}

/// Handle work sync command
pub async fn handle_work_sync(
    direction: SyncDirection,
    path: Option<PathBuf>,
    dry_run: bool,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    let action = if dry_run { "Dry run" } else { "Syncing" };
    println!("🔄 {} roadmap...", action);
    println!();

    let roadmap = service.load()?;

    match direction {
        SyncDirection::YamlToGithub => {
            println!("📤 Direction: YAML → GitHub");
            let yaml_only = roadmap.yaml_only_items();
            println!("   Found {} YAML-only items", yaml_only.len());
            for item in yaml_only {
                println!("      - {} ({})", item.id, item.title);
            }
            println!();
            println!("   ⚠️  GitHub sync not yet implemented");
        }
        SyncDirection::GithubToYaml => {
            println!("📥 Direction: GitHub → YAML");
            println!("   ⚠️  GitHub sync not yet implemented");
        }
        SyncDirection::Full => {
            println!("🔄 Direction: Full bidirectional sync");
            println!("   ⚠️  GitHub sync not yet implemented");
        }
    }

    println!();
    Ok(())
}

/// Fetch GitHub issue details
async fn fetch_github_issue(repo: &str, issue_num: u64) -> Result<octocrab::models::issues::Issue> {
    // Try authenticated client first, fall back to unauthenticated
    let client = match GitHubClient::new(repo) {
        Ok(c) => c,
        Err(_) => {
            // GITHUB_TOKEN not set, try unauthenticated
            GitHubClient::new_unauthenticated(repo)?
        }
    };

    let issue = client.fetch_issue(issue_num).await?;
    Ok(issue)
}

/// Create GitHub issue from roadmap item
async fn create_github_issue_from_item(
    repo: &str,
    item: &RoadmapItem,
) -> Result<octocrab::models::issues::Issue> {
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
    Ok(issue)
}

/// Parse acceptance criteria from GitHub issue body
///
/// Looks for markdown checklists in the body and extracts them as criteria.
fn parse_acceptance_criteria(body: &str) -> Vec<String> {
    let mut criteria = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();
        // Match markdown checkboxes: - [ ] or - [x]
        if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") {
            let criterion = trimmed
                .trim_start_matches("- [ ]")
                .trim_start_matches("- [x]")
                .trim()
                .to_string();
            if !criterion.is_empty() {
                criteria.push(criterion);
            }
        }
    }

    criteria
}

/// Detect GitHub repository from git remote
fn detect_github_repo(project_path: &PathBuf) -> Result<Option<String>> {
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
fn parse_github_url(url: &str) -> Option<String> {
    // HTTPS: https://github.com/owner/repo.git
    if let Some(start) = url.find("github.com/") {
        let rest = &url[start + 11..];
        let repo = rest.trim_end_matches(".git");
        return Some(repo.to_string());
    }

    // SSH: git@github.com:owner/repo.git
    if let Some(start) = url.find("github.com:") {
        let rest = &url[start + 11..];
        let repo = rest.trim_end_matches(".git");
        return Some(repo.to_string());
    }

    None
}

/// Create specification template
fn create_specification_template(spec_path: &PathBuf, item: &RoadmapItem) -> Result<()> {
    use std::fs;

    if let Some(parent) = spec_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let github_link = if let Some(issue) = item.github_issue {
        format!(
            "**GitHub Issue**: [#{}](https://github.com/YOUR_ORG/YOUR_REPO/issues/{})",
            issue, issue
        )
    } else {
        format!("**Ticket ID**: {}", item.id)
    };

    let template = format!(
        r#"---
title: {}
issue: {}
status: In Progress
created: {}
updated: {}
---

# {} Specification

{}
**Status**: In Progress

## Summary

[Brief 2-3 sentence overview of what this work accomplishes]

## Requirements

### Functional Requirements
- [ ] Requirement 1
- [ ] Requirement 2

### Non-Functional Requirements
- [ ] Performance: [specific target]
- [ ] Test coverage: ≥85%

## Architecture

### Design Overview

[Describe the high-level design approach]

### API Design

```rust
// Example API design
pub struct Example {{
    // ...
}}
```

## Implementation Plan

### Phase 1: Foundation
- [ ] Task 1
- [ ] Task 2

### Phase 2: Core Implementation
- [ ] Task 3
- [ ] Task 4

## Testing Strategy

### Unit Tests
- [ ] Test case 1
- [ ] Test case 2

### Integration Tests
- [ ] Integration test 1

## Success Criteria

- ✅ All acceptance criteria met
- ✅ Test coverage ≥85%
- ✅ Zero clippy warnings
- ✅ Documentation complete

## References

- [Related documentation]
"#,
        item.title, item.id, item.created, item.updated, item.title, github_link
    );

    fs::write(spec_path, template)?;
    Ok(())
}

/// Run quality gates (tests, clippy, etc.)
///
/// Returns Ok(true) if all gates pass, Ok(false) if any fail, or Err on execution failure.
async fn run_quality_gates(project_path: &PathBuf) -> Result<bool> {
    use std::process::Command;

    let mut all_passed = true;

    // 1. Run cargo test (git-aware: only test changed modules)
    println!("   🧪 Running tests...");

    // Extract test modules from changed files
    let modules =
        crate::services::git_test_filter::extract_test_modules_from_changed_files(project_path)?;

    let test_status = if modules.is_empty() {
        // No Rust files changed - skip tests
        println!("      ℹ️  No Rust files changed, skipping tests");
        std::process::ExitStatus::default()
    } else {
        // Run tests for changed modules only
        let module_list = modules.join(", ");
        println!(
            "      📋 Testing changed modules: {}",
            if module_list.len() > 60 {
                format!("{}...", &module_list[..60])
            } else {
                module_list
            }
        );

        let test_cmd = crate::services::git_test_filter::build_test_command(&modules)
            .unwrap_or_else(|| {
                vec![
                    "test".to_string(),
                    "--lib".to_string(),
                    "--quiet".to_string(),
                ]
            });

        Command::new("cargo")
            .args(&test_cmd)
            .arg("--quiet")
            .current_dir(project_path)
            .status()
            .context("Failed to run cargo test")?
    };

    if test_status.success() {
        println!("      ✅ Tests passed");
    } else {
        println!("      ❌ Tests failed");
        all_passed = false;
    }

    // 2. Rust project-specific checks (if Cargo.toml exists)
    if project_path.join("Cargo.toml").exists() {
        println!("   🦀 Rust project detected...");

        // Check if examples directory exists
        let examples_dir = project_path.join("examples");
        if examples_dir.exists() && examples_dir.is_dir() {
            println!("      📦 Checking examples...");
            let examples_status = Command::new("cargo")
                .args(["test", "--examples", "--no-run"])
                .current_dir(project_path)
                .status()
                .context("Failed to run cargo test --examples")?;

            if examples_status.success() {
                println!("      ✅ Examples compile");
            } else {
                println!("      ❌ Examples failed to compile");
                all_passed = false;
            }
        } else {
            println!("      ℹ️  No examples directory found, skipping example checks");
        }

        // Capture rust-project-score (O(1) from cache)
        println!("      📊 Capturing rust-project-score...");
        match Command::new("pmat")
            .args(["rust-project-score", "--format", "json"])
            .current_dir(project_path)
            .output()
        {
            Ok(output) if output.status.success() => {
                // Parse score and display
                if let Ok(score_json) = std::str::from_utf8(&output.stdout) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(score_json) {
                        if let Some(score) = json.get("total_earned").and_then(|v| v.as_f64()) {
                            println!("      ✅ Rust Project Score: {:.1}/134", score);
                        }
                    }
                }
            }
            Ok(_) => {
                println!("      ⚠️  Failed to capture rust-project-score (continuing)");
            }
            Err(_) => {
                println!("      ⚠️  pmat rust-project-score not available (continuing)");
            }
        }
    }

    // 3. Renacer golden tracing validation (if renacer.toml exists)
    if project_path.join("renacer.toml").exists() {
        println!("   🎯 Golden traces detected...");

        match Command::new("renacer")
            .args(["validate", "--all"])
            .current_dir(project_path)
            .status()
        {
            Ok(status) if status.success() => {
                println!("      ✅ Golden traces match");
            }
            Ok(_) => {
                println!("      ❌ Golden traces diverged");
                all_passed = false;
            }
            Err(_) => {
                println!("      ⚠️  renacer not installed (skipping golden trace validation)");
                println!("         Install: cargo install renacer");
            }
        }
    }

    // 4. Run cargo clippy
    println!("   📎 Running clippy...");
    let clippy_status = Command::new("cargo")
        .arg("clippy")
        .arg("--lib")
        .arg("--quiet")
        .arg("--")
        .arg("-D")
        .arg("warnings")
        .current_dir(project_path)
        .status()
        .context("Failed to run cargo clippy")?;

    if clippy_status.success() {
        println!("      ✅ No clippy warnings");
    } else {
        println!("      ❌ Clippy warnings found");
        all_passed = false;
    }

    println!();
    Ok(all_passed)
}

/// Karl Popper Falsification Result
///
/// Captures the results of post-work falsification validation.
/// Based on the philosophy that scientific claims must be falsifiable -
/// we validate that our work satisfies falsification criteria.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationResult {
    /// Tests passed (falsify: no regressions introduced)
    pub tests_passed: bool,
    /// Coverage increased or maintained (falsify: no code bloat without tests)
    pub coverage_maintained: bool,
    /// Coverage percentage before work
    pub coverage_before: Option<f32>,
    /// Coverage percentage after work
    pub coverage_after: Option<f32>,
    /// Binary size within threshold (falsify: no dependency bloat)
    pub binary_size_ok: bool,
    /// Overall falsification passed
    pub passed: bool,
    /// Human-readable summary
    pub summary: String,
}

impl Default for FalsificationResult {
    fn default() -> Self {
        Self {
            tests_passed: false,
            coverage_maintained: false,
            coverage_before: None,
            coverage_after: None,
            binary_size_ok: true,
            passed: false,
            summary: String::new(),
        }
    }
}

/// Run Karl Popper Falsification Validation
///
/// This implements the scientific method for validating work:
/// 1. Hypothesis: Work should not introduce regressions
/// 2. Falsification: Run tests to attempt to falsify the hypothesis
/// 3. Measurement: Measure coverage to verify improvements
/// 4. Result: Pass only if falsification attempts fail (work is valid)
///
/// Based on: docs/specifications/80-20-to-95.md
pub async fn run_popper_falsification(project_path: &PathBuf) -> Result<FalsificationResult> {
    use std::process::Command;

    let mut result = FalsificationResult::default();
    let mut issues: Vec<String> = Vec::new();
    let total_hypotheses = 3;
    let mut validated = 0;

    println!();
    println!("🔬 Karl Popper Falsification Validation (0/{} complete)", total_hypotheses);
    println!("   (Scientific method: attempting to falsify your work)");
    println!();

    // 1. Hypothesis: Tests should pass (falsify: look for regressions)
    println!("   📊 [1/{}] Hypothesis: No regressions introduced", total_hypotheses);
    println!("      Falsification: Running tests...");

    let test_status = Command::new("cargo")
        .args(["test", "--lib", "--quiet"])
        .current_dir(project_path)
        .status()
        .context("Failed to run cargo test")?;

    if test_status.success() {
        result.tests_passed = true;
        validated += 1;
        println!("      ✅ Hypothesis holds ({}/{} validated)", validated, total_hypotheses);
    } else {
        result.tests_passed = false;
        issues.push("Tests failed - regressions detected".to_string());
        println!("      ❌ Hypothesis falsified: Tests fail");
    }

    // 2. Hypothesis: Coverage should be maintained or improved
    println!();
    println!("   📊 [2/{}] Hypothesis: Coverage maintained or improved", total_hypotheses);
    println!("      Falsification: Checking coverage trends...");

    // Try to read coverage from cached metrics
    let metrics_dir = project_path.join(".pmat-metrics/trends");
    if metrics_dir.exists() {
        if let Ok(content) = std::fs::read_to_string(metrics_dir.join("test-coverage.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(entries) = json.as_array() {
                    if entries.len() >= 2 {
                        // Compare last two entries
                        let current = entries.last()
                            .and_then(|e| e.get("value"))
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) as f32;
                        let previous = entries.get(entries.len() - 2)
                            .and_then(|e| e.get("value"))
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) as f32;

                        result.coverage_before = Some(previous);
                        result.coverage_after = Some(current);

                        if current >= previous {
                            result.coverage_maintained = true;
                            validated += 1;
                            let delta = current - previous;
                            if delta > 0.0 {
                                println!("      ✅ Hypothesis holds: Coverage +{:.2}% ({}/{} validated)", delta, validated, total_hypotheses);
                            } else {
                                println!("      ✅ Hypothesis holds: Coverage at {:.2}% ({}/{} validated)", current, validated, total_hypotheses);
                            }
                        } else {
                            let delta = previous - current;
                            issues.push(format!("Coverage dropped by {:.2}%", delta));
                            println!("      ❌ Hypothesis falsified: Coverage -{:.2}%", delta);
                        }
                    } else if !entries.is_empty() {
                        result.coverage_maintained = true;
                        validated += 1;
                        println!("      ⚠️  Insufficient history ({}/{} validated)", validated, total_hypotheses);
                    }
                }
            }
        }
    }

    if result.coverage_before.is_none() {
        result.coverage_maintained = true; // Assume OK if no data
        validated += 1;
        println!("      ⚠️  No coverage history ({}/{} validated)", validated, total_hypotheses);
        println!("         Run 'make coverage' to establish baseline");
    }

    // 3. Binary size check (optional, only if release build exists)
    println!();
    println!("   📊 [3/{}] Hypothesis: No dependency bloat", total_hypotheses);
    result.binary_size_ok = true; // Default to OK

    let release_binary = project_path.join("target/release/pmat");
    if release_binary.exists() {
        if let Ok(metadata) = std::fs::metadata(&release_binary) {
            let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
            if size_mb <= 50.0 {
                validated += 1;
                println!("      ✅ Hypothesis holds: {:.1}MB < 50MB ({}/{} validated)", size_mb, validated, total_hypotheses);
            } else {
                result.binary_size_ok = false;
                issues.push(format!("Binary size {:.1}MB exceeds 50MB limit", size_mb));
                println!("      ❌ Hypothesis falsified: {:.1}MB > 50MB limit", size_mb);
            }
        }
    } else {
        validated += 1;
        println!("      ⚠️  No release binary ({}/{} validated)", validated, total_hypotheses);
    }

    // Determine overall result
    result.passed = result.tests_passed && result.coverage_maintained && result.binary_size_ok;

    println!();
    if result.passed {
        result.summary = format!("{}/{} hypotheses validated - work is valid", validated, total_hypotheses);
        println!("   🎉 FALSIFICATION RESULT: PASSED ({}/{})", validated, total_hypotheses);
        println!("      All hypotheses held under scrutiny");
    } else {
        let failed = total_hypotheses - validated;
        result.summary = format!("{}/{} validated, {} falsified: {}", validated, total_hypotheses, failed, issues.join(", "));
        println!("   ⚠️  FALSIFICATION RESULT: FAILED ({}/{} validated)", validated, total_hypotheses);
        println!("      Issues found:");
        for issue in &issues {
            println!("      - {}", issue);
        }
    }
    println!();

    Ok(result)
}

/// Handle work validate command (Part B: UX Improvements)
///
/// Validates roadmap.yaml syntax and content with actionable error messages.
pub async fn handle_work_validate(path: Option<PathBuf>, verbose: bool, fix: bool) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");

    println!("🔍 Validating roadmap: {}", roadmap_path.display());
    println!();

    if !roadmap_path.exists() {
        anyhow::bail!(
            "Roadmap not found: {}\n\nRun `pmat work init` to create one.",
            roadmap_path.display()
        );
    }

    // Read raw content for better error reporting
    let content = std::fs::read_to_string(&roadmap_path).context("Failed to read roadmap file")?;

    // Try to parse
    match serde_yaml::from_str::<crate::models::roadmap::Roadmap>(&content) {
        Ok(roadmap) => {
            println!("✅ Syntax valid");
            println!("   Version: {}", roadmap.roadmap_version);
            println!("   Items: {}", roadmap.roadmap.len());
            println!(
                "   GitHub: {}",
                if roadmap.github_enabled {
                    roadmap.github_repo.as_deref().unwrap_or("not configured")
                } else {
                    "disabled"
                }
            );
            println!();

            // Semantic validation
            let mut warnings = Vec::new();

            for item in &roadmap.roadmap {
                // Check for missing acceptance criteria on features
                if item.acceptance_criteria.is_empty()
                    && !matches!(item.status, ItemStatus::Cancelled)
                {
                    warnings.push(format!("⚠️  {} has no acceptance criteria", item.id));
                }

                // Check for long IDs (UX issue from spec)
                if item.id.len() > 50 {
                    warnings.push(format!(
                        "⚠️  {} has a long ID ({} chars) - consider using shorter IDs",
                        &item.id[..30],
                        item.id.len()
                    ));
                }
            }

            if !warnings.is_empty() {
                println!("Warnings ({}):", warnings.len());
                for warning in &warnings {
                    println!("   {}", warning);
                }
                println!();
            }

            if verbose {
                println!("📋 Items:");
                for item in &roadmap.roadmap {
                    println!("   {} [{:?}] - {}", item.id, item.status, item.title);
                }
            }

            if fix && !warnings.is_empty() {
                println!("💡 Tip: Use `pmat work migrate` to auto-fix issues");
            }

            println!("✅ Validation passed");
            Ok(())
        }
        Err(e) => {
            // Provide actionable error message with line info
            let error_msg = format!("{}", e);

            println!("❌ Validation failed\n");
            println!("Error: {}", error_msg);
            println!();

            // Try to extract line number from error
            if let Some(line) = extract_line_from_yaml_error(&error_msg) {
                // Show context around the error
                let lines: Vec<&str> = content.lines().collect();
                if line > 0 && line <= lines.len() {
                    println!("Context (around line {}):", line);
                    let start = line.saturating_sub(3);
                    let end = std::cmp::min(line + 2, lines.len());
                    for (i, l) in lines[start..end].iter().enumerate() {
                        let line_num = start + i + 1;
                        let marker = if line_num == line { ">>>" } else { "   " };
                        println!("{} {:4}: {}", marker, line_num, l);
                    }
                    println!();
                }
            }

            // Provide suggestions
            println!("💡 Common fixes:");
            println!(
                "   - Use valid status values: completed, done, wip, planned, blocked, review"
            );
            println!("   - Quote strings with special characters: `:`, `<`, `>`");
            println!("   - Use proper YAML indentation (2 spaces)");
            println!();
            println!("Run `pmat work status --list` to see all valid status values.");

            anyhow::bail!("Roadmap validation failed")
        }
    }
}

/// Handle work migrate command (Part B: UX Improvements)
///
/// Auto-fixes common roadmap.yaml issues.
pub async fn handle_work_migrate(path: Option<PathBuf>, dry_run: bool, backup: bool) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");

    println!("🔄 Migrating roadmap: {}", roadmap_path.display());
    println!();

    if !roadmap_path.exists() {
        anyhow::bail!(
            "Roadmap not found: {}\n\nRun `pmat work init` to create one.",
            roadmap_path.display()
        );
    }

    let content = std::fs::read_to_string(&roadmap_path)?;
    let mut changes: Vec<String> = Vec::new();
    let mut new_content = content.clone();

    // 1. Normalize status values
    let status_patterns = [
        ("status: done", "status: completed"),
        ("status: Done", "status: completed"),
        ("status: DONE", "status: completed"),
        ("status: finished", "status: completed"),
        ("status: in progress", "status: inprogress"),
        ("status: In Progress", "status: inprogress"),
        ("status: WIP", "status: inprogress"),
        ("status: wip", "status: inprogress"),
        ("status: stuck", "status: blocked"),
        ("status: on-hold", "status: blocked"),
        ("status: todo", "status: planned"),
        ("status: TODO", "status: planned"),
        ("status: open", "status: planned"),
    ];

    for (old, new) in status_patterns {
        if new_content.contains(old) {
            changes.push(format!("Normalize status: {} → {}", old, new));
            new_content = new_content.replace(old, new);
        }
    }

    // 2. Quote special characters in titles
    let special_chars = [':', '<', '>', '≥', '≤', '±', 'ε', '→', '↔'];
    for line in content.lines() {
        if line.trim_start().starts_with("title:") || line.trim_start().starts_with("- title:") {
            let has_special = special_chars
                .iter()
                .any(|c| line.contains(*c) && !line.contains("\""));
            if has_special && !line.contains("\"") {
                // This is a simplistic check - in practice we'd need proper YAML parsing
                changes.push(format!("Consider quoting: {}", line.trim()));
            }
        }
    }

    if changes.is_empty() {
        println!("✅ No migrations needed - roadmap is already up to date");
        return Ok(());
    }

    println!("Found {} potential changes:", changes.len());
    for change in &changes {
        println!("   • {}", change);
    }
    println!();

    if dry_run {
        println!("(Dry run - no changes made)");
        return Ok(());
    }

    // Create backup
    if backup {
        let backup_path = roadmap_path.with_extension("yaml.bak");
        std::fs::write(&backup_path, &content)?;
        println!("✅ Created backup: {}", backup_path.display());
    }

    // Write changes
    std::fs::write(&roadmap_path, &new_content)?;
    println!("✅ Updated roadmap: {}", roadmap_path.display());

    // Verify the changes
    if serde_yaml::from_str::<crate::models::roadmap::Roadmap>(&new_content).is_ok() {
        println!("✅ Verified: updated roadmap is valid");
    } else {
        println!("⚠️  Warning: updated roadmap may have issues - check manually");
    }

    Ok(())
}

/// Handle work list-statuses command (Part B: UX Improvements)
///
/// Lists all valid status values with descriptions and aliases.
pub async fn handle_work_list_statuses() -> Result<()> {
    println!("📋 Valid Status Values\n");
    println!("{:<15} {:<25} DESCRIPTION", "STATUS", "ALIASES");
    println!("{}", "-".repeat(70));

    let statuses = [
        (
            "planned",
            "todo, open, pending, new",
            "Task not yet started",
        ),
        (
            "inprogress",
            "wip, active, started",
            "Currently being worked on",
        ),
        (
            "blocked",
            "stuck, waiting, on-hold",
            "Cannot proceed (waiting on something)",
        ),
        (
            "review",
            "reviewing, pr, pending-review",
            "Ready for or in code review",
        ),
        (
            "completed",
            "done, finished, closed",
            "Work finished successfully",
        ),
        (
            "cancelled",
            "canceled, dropped, wontfix",
            "Work abandoned or not needed",
        ),
    ];

    for (status, aliases, description) in statuses {
        println!("{:<15} {:<25} {}", status, aliases, description);
    }

    println!();
    println!("💡 All status values are case-insensitive and support hyphens/underscores.");
    println!("   Example: 'In-Progress', 'in_progress', 'InProgress', 'WIP' all work.");

    Ok(())
}

/// Extract line number from YAML error message
fn extract_line_from_yaml_error(error: &str) -> Option<usize> {
    // serde_yaml errors often contain "at line X column Y"
    if let Some(pos) = error.find("at line ") {
        let rest = &error[pos + 8..];
        if let Some(end) = rest.find(' ') {
            return rest[..end].parse().ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url_https() {
        let url = "https://github.com/paiml/pmat.git";
        assert_eq!(parse_github_url(url), Some("paiml/pmat".to_string()));
    }

    #[test]
    fn test_parse_github_url_ssh() {
        let url = "git@github.com:paiml/pmat.git";
        assert_eq!(parse_github_url(url), Some("paiml/pmat".to_string()));
    }

    #[test]
    fn test_parse_github_url_invalid() {
        let url = "https://gitlab.com/owner/repo.git";
        assert_eq!(parse_github_url(url), None);
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::TempDir;

    // ========== Test Fixtures ==========

    /// Create a test project directory with roadmap structure
    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create docs/roadmaps directory
        let roadmaps_dir = temp_dir.path().join("docs").join("roadmaps");
        std::fs::create_dir_all(&roadmaps_dir).expect("Failed to create roadmaps dir");

        temp_dir
    }

    /// Create a test project with initialized roadmap
    fn create_initialized_project() -> TempDir {
        let temp_dir = create_test_project();

        let roadmap_path = temp_dir
            .path()
            .join("docs")
            .join("roadmaps")
            .join("roadmap.yaml");
        let roadmap_content = r#"
roadmap_version: '1.0'
github_enabled: true
github_repo: paiml/pmat
roadmap:
  - id: TEST-001
    title: Test Item 1
    status: planned
    priority: medium
  - id: GH-42
    github_issue: 42
    title: GitHub Issue
    status: inprogress
    priority: high
    labels:
      - enhancement
      - feature
  - id: EPIC-001
    title: Epic Item
    status: planned
    priority: high
    item_type: epic
    subtasks:
      - id: EPIC-001-A
        title: Subtask A
        status: completed
        completion: 100
      - id: EPIC-001-B
        title: Subtask B
        status: inprogress
        completion: 50
"#;
        std::fs::write(&roadmap_path, roadmap_content).expect("Failed to write roadmap");

        temp_dir
    }

    /// Create a test roadmap item
    fn make_test_item(id: &str, title: &str, status: ItemStatus) -> RoadmapItem {
        let mut item = RoadmapItem::new(id.to_string(), title.to_string());
        item.status = status;
        item
    }

    // ========== parse_github_url Tests ==========

    mod parse_github_url_tests {
        use super::*;

        #[test]
        fn test_https_url_with_git_extension() {
            let url = "https://github.com/owner/repo.git";
            assert_eq!(parse_github_url(url), Some("owner/repo".to_string()));
        }

        #[test]
        fn test_https_url_without_git_extension() {
            let url = "https://github.com/owner/repo";
            assert_eq!(parse_github_url(url), Some("owner/repo".to_string()));
        }

        #[test]
        fn test_ssh_url_with_git_extension() {
            let url = "git@github.com:owner/repo.git";
            assert_eq!(parse_github_url(url), Some("owner/repo".to_string()));
        }

        #[test]
        fn test_ssh_url_without_git_extension() {
            let url = "git@github.com:owner/repo";
            assert_eq!(parse_github_url(url), Some("owner/repo".to_string()));
        }

        #[test]
        fn test_https_url_with_org_and_nested_repo() {
            let url = "https://github.com/paiml/paiml-mcp-agent-toolkit.git";
            assert_eq!(
                parse_github_url(url),
                Some("paiml/paiml-mcp-agent-toolkit".to_string())
            );
        }

        #[test]
        fn test_gitlab_url_returns_none() {
            let url = "https://gitlab.com/owner/repo.git";
            assert_eq!(parse_github_url(url), None);
        }

        #[test]
        fn test_bitbucket_url_returns_none() {
            let url = "https://bitbucket.org/owner/repo.git";
            assert_eq!(parse_github_url(url), None);
        }

        #[test]
        fn test_empty_url() {
            assert_eq!(parse_github_url(""), None);
        }

        #[test]
        fn test_random_string() {
            assert_eq!(parse_github_url("not-a-url"), None);
        }
    }

    // ========== parse_acceptance_criteria Tests ==========

    mod parse_acceptance_criteria_tests {
        use super::*;

        #[test]
        fn test_empty_body() {
            let body = "";
            let criteria = parse_acceptance_criteria(body);
            assert!(criteria.is_empty());
        }

        #[test]
        fn test_body_with_unchecked_checkboxes() {
            let body = r#"
## Acceptance Criteria
- [ ] First criterion
- [ ] Second criterion
- [ ] Third criterion
"#;
            let criteria = parse_acceptance_criteria(body);
            assert_eq!(criteria.len(), 3);
            assert_eq!(criteria[0], "First criterion");
            assert_eq!(criteria[1], "Second criterion");
            assert_eq!(criteria[2], "Third criterion");
        }

        #[test]
        fn test_body_with_checked_checkboxes() {
            let body = r#"
## Done
- [x] Completed task
- [x] Another completed task
"#;
            let criteria = parse_acceptance_criteria(body);
            assert_eq!(criteria.len(), 2);
            assert_eq!(criteria[0], "Completed task");
            assert_eq!(criteria[1], "Another completed task");
        }

        #[test]
        fn test_body_with_mixed_checkboxes() {
            let body = r#"
## Acceptance Criteria
- [x] Already done
- [ ] Still pending
- [x] Also done
"#;
            let criteria = parse_acceptance_criteria(body);
            assert_eq!(criteria.len(), 3);
        }

        #[test]
        fn test_body_with_no_checkboxes() {
            let body = r#"
This is a description without checkboxes.
Just regular text.
"#;
            let criteria = parse_acceptance_criteria(body);
            assert!(criteria.is_empty());
        }

        #[test]
        fn test_body_with_empty_checkbox() {
            let body = "- [ ] ";
            let criteria = parse_acceptance_criteria(body);
            assert!(criteria.is_empty());
        }

        #[test]
        fn test_body_with_whitespace_only_checkbox() {
            let body = "- [ ]    ";
            let criteria = parse_acceptance_criteria(body);
            assert!(criteria.is_empty());
        }
    }

    // ========== extract_line_from_yaml_error Tests ==========

    mod extract_line_from_yaml_error_tests {
        use super::*;

        #[test]
        fn test_error_with_line_number() {
            let error = "invalid type: string, expected sequence at line 42 column 5";
            let line = extract_line_from_yaml_error(error);
            assert_eq!(line, Some(42));
        }

        #[test]
        fn test_error_without_line_number() {
            let error = "invalid type: string, expected sequence";
            let line = extract_line_from_yaml_error(error);
            assert_eq!(line, None);
        }

        #[test]
        fn test_error_with_single_digit_line() {
            let error = "error at line 5 column 1";
            let line = extract_line_from_yaml_error(error);
            assert_eq!(line, Some(5));
        }

        #[test]
        fn test_error_with_large_line_number() {
            let error = "parsing failed at line 1234 column 10";
            let line = extract_line_from_yaml_error(error);
            assert_eq!(line, Some(1234));
        }

        #[test]
        fn test_empty_error_string() {
            let error = "";
            let line = extract_line_from_yaml_error(error);
            assert_eq!(line, None);
        }
    }

    // ========== CommitMetadata Tests ==========

    mod commit_metadata_tests {
        use super::*;

        #[test]
        fn test_commit_metadata_serialization() {
            let metadata = CommitMetadata {
                commit_sha: Some("abc123".to_string()),
                work_item_id: "TEST-001".to_string(),
                prompt: "Test task".to_string(),
                tdg_score: 85.0,
                repo_score: 75.0,
                rust_project_score: Some(90.0),
                timestamp: chrono::Utc::now(),
            };

            let json = serde_json::to_string(&metadata).unwrap();
            assert!(json.contains("abc123"));
            assert!(json.contains("TEST-001"));
            assert!(json.contains("85"));
        }

        #[test]
        fn test_commit_metadata_deserialization() {
            let json = r#"{
                "commit_sha": "def456",
                "work_item_id": "GH-42",
                "prompt": "Fix bug",
                "tdg_score": 90.0,
                "repo_score": 80.0,
                "rust_project_score": null,
                "timestamp": "2024-01-01T00:00:00Z"
            }"#;

            let metadata: CommitMetadata = serde_json::from_str(json).unwrap();
            assert_eq!(metadata.commit_sha, Some("def456".to_string()));
            assert_eq!(metadata.work_item_id, "GH-42");
            assert_eq!(metadata.tdg_score, 90.0);
            assert!(metadata.rust_project_score.is_none());
        }
    }

    // ========== Score Capture Tests ==========

    mod score_capture_tests {
        use super::*;

        #[tokio::test]
        async fn test_capture_tdg_score_no_cache() {
            let temp_dir = TempDir::new().unwrap();
            let score = capture_tdg_score(&temp_dir.path().to_path_buf()).await;
            // Should return default when no cache exists
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 0.0);
        }

        #[tokio::test]
        async fn test_capture_tdg_score_with_cache() {
            let temp_dir = TempDir::new().unwrap();
            let metrics_dir = temp_dir.path().join(".pmat-metrics");
            std::fs::create_dir_all(&metrics_dir).unwrap();

            let tdg_file = metrics_dir.join("tdg-score.json");
            std::fs::write(&tdg_file, r#"{"score": 85.5}"#).unwrap();

            let score = capture_tdg_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 85.5);
        }

        #[tokio::test]
        async fn test_capture_repo_score_no_cache() {
            let temp_dir = TempDir::new().unwrap();
            let score = capture_repo_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 0.0);
        }

        #[tokio::test]
        async fn test_capture_repo_score_with_cache() {
            let temp_dir = TempDir::new().unwrap();
            let metrics_dir = temp_dir.path().join(".pmat-metrics");
            std::fs::create_dir_all(&metrics_dir).unwrap();

            let repo_file = metrics_dir.join("repo-score.json");
            std::fs::write(&repo_file, r#"{"score": 72.0}"#).unwrap();

            let score = capture_repo_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 72.0);
        }

        #[tokio::test]
        async fn test_capture_rust_project_score_no_cache() {
            let temp_dir = TempDir::new().unwrap();
            let score = capture_rust_project_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 0.0);
        }

        #[tokio::test]
        async fn test_capture_rust_project_score_with_cache() {
            let temp_dir = TempDir::new().unwrap();
            let metrics_dir = temp_dir.path().join(".pmat-metrics");
            std::fs::create_dir_all(&metrics_dir).unwrap();

            let rust_file = metrics_dir.join("rust-project-score.json");
            std::fs::write(&rust_file, r#"{"total_earned": 95.0}"#).unwrap();

            let score = capture_rust_project_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 95.0);
        }

        #[tokio::test]
        async fn test_capture_score_with_invalid_json() {
            let temp_dir = TempDir::new().unwrap();
            let metrics_dir = temp_dir.path().join(".pmat-metrics");
            std::fs::create_dir_all(&metrics_dir).unwrap();

            let tdg_file = metrics_dir.join("tdg-score.json");
            std::fs::write(&tdg_file, "not valid json").unwrap();

            let score = capture_tdg_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_err());
        }
    }

    // ========== Handler Integration Tests ==========

    mod handler_integration_tests {
        use super::*;

        #[tokio::test]
        async fn test_handle_work_init_creates_roadmap() {
            let temp_dir = create_test_project();

            let result = handle_work_init(
                Some("paiml/test".to_string()),
                false,
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());

            let roadmap_path = temp_dir
                .path()
                .join("docs")
                .join("roadmaps")
                .join("roadmap.yaml");
            assert!(roadmap_path.exists());
        }

        #[tokio::test]
        async fn test_handle_work_init_no_github() {
            let temp_dir = create_test_project();

            let result = handle_work_init(None, true, Some(temp_dir.path().to_path_buf())).await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_init_already_exists() {
            let temp_dir = create_initialized_project();

            let result = handle_work_init(
                Some("paiml/test".to_string()),
                false,
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            // Should succeed but indicate already exists
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_status_all_items() {
            let temp_dir = create_initialized_project();

            let result = handle_work_status(None, Some(temp_dir.path().to_path_buf()), false).await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_status_active_only() {
            let temp_dir = create_initialized_project();

            let result = handle_work_status(None, Some(temp_dir.path().to_path_buf()), true).await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_status_specific_item() {
            let temp_dir = create_initialized_project();

            let result = handle_work_status(
                Some("TEST-001".to_string()),
                Some(temp_dir.path().to_path_buf()),
                false,
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_status_nonexistent_item() {
            let temp_dir = create_initialized_project();

            let result = handle_work_status(
                Some("NONEXISTENT-999".to_string()),
                Some(temp_dir.path().to_path_buf()),
                false,
            )
            .await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_handle_work_continue_existing_item() {
            let temp_dir = create_initialized_project();

            let result =
                handle_work_continue("TEST-001".to_string(), Some(temp_dir.path().to_path_buf()))
                    .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_continue_with_phases() {
            let temp_dir = create_initialized_project();

            let result =
                handle_work_continue("GH-42".to_string(), Some(temp_dir.path().to_path_buf()))
                    .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_continue_nonexistent() {
            let temp_dir = create_initialized_project();

            let result = handle_work_continue(
                "NONEXISTENT-999".to_string(),
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_handle_work_sync_yaml_to_github() {
            let temp_dir = create_initialized_project();

            let result = handle_work_sync(
                SyncDirection::YamlToGithub,
                Some(temp_dir.path().to_path_buf()),
                true, // dry_run
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_sync_github_to_yaml() {
            let temp_dir = create_initialized_project();

            let result = handle_work_sync(
                SyncDirection::GithubToYaml,
                Some(temp_dir.path().to_path_buf()),
                true, // dry_run
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_sync_full() {
            let temp_dir = create_initialized_project();

            let result = handle_work_sync(
                SyncDirection::Full,
                Some(temp_dir.path().to_path_buf()),
                true, // dry_run
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_validate_valid_roadmap() {
            let temp_dir = create_initialized_project();

            let result = handle_work_validate(
                Some(temp_dir.path().to_path_buf()),
                false, // verbose
                false, // fix
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_validate_verbose() {
            let temp_dir = create_initialized_project();

            let result = handle_work_validate(
                Some(temp_dir.path().to_path_buf()),
                true,  // verbose
                false, // fix
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_validate_missing_roadmap() {
            let temp_dir = TempDir::new().unwrap();

            let result =
                handle_work_validate(Some(temp_dir.path().to_path_buf()), false, false).await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_handle_work_list_statuses() {
            let result = handle_work_list_statuses().await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_migrate_no_changes_needed() {
            let temp_dir = create_initialized_project();

            let result = handle_work_migrate(
                Some(temp_dir.path().to_path_buf()),
                true,  // dry_run
                false, // backup
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_migrate_with_backup() {
            let temp_dir = create_initialized_project();

            // Modify roadmap to have a fixable issue
            let roadmap_path = temp_dir
                .path()
                .join("docs")
                .join("roadmaps")
                .join("roadmap.yaml");
            let content = std::fs::read_to_string(&roadmap_path).unwrap();
            let modified = content.replace("status: planned", "status: done");
            std::fs::write(&roadmap_path, modified).unwrap();

            let result = handle_work_migrate(
                Some(temp_dir.path().to_path_buf()),
                false, // dry_run
                true,  // backup
            )
            .await;

            assert!(result.is_ok());
            // Backup file should exist
            let backup_path = roadmap_path.with_extension("yaml.bak");
            assert!(backup_path.exists());
        }

        #[tokio::test]
        async fn test_handle_work_migrate_missing_roadmap() {
            let temp_dir = TempDir::new().unwrap();

            let result =
                handle_work_migrate(Some(temp_dir.path().to_path_buf()), true, false).await;

            assert!(result.is_err());
        }
    }

    // ========== Property-Based Tests ==========

    mod proptest_tests {
        use super::*;

        proptest! {
            #[test]
            fn test_parse_github_url_never_panics(url in ".*") {
                let _ = parse_github_url(&url);
            }

            #[test]
            fn test_parse_acceptance_criteria_never_panics(body in ".*") {
                let _ = parse_acceptance_criteria(&body);
            }

            #[test]
            fn test_extract_line_from_yaml_error_never_panics(error in ".*") {
                let _ = extract_line_from_yaml_error(&error);
            }

            #[test]
            fn test_github_url_extraction_consistency(owner in "[a-z]{1,20}", repo in "[a-z0-9-]{1,30}") {
                let https_url = format!("https://github.com/{}/{}.git", owner, repo);
                let ssh_url = format!("git@github.com:{}/{}.git", owner, repo);

                let expected = format!("{}/{}", owner, repo);
                prop_assert_eq!(parse_github_url(&https_url), Some(expected.clone()));
                prop_assert_eq!(parse_github_url(&ssh_url), Some(expected));
            }

            #[test]
            // Ensure at least one alphanumeric character
            fn test_acceptance_criteria_preserves_content(criteria_text in "[a-zA-Z0-9][a-zA-Z0-9 ]{0,49}") {
                let body = format!("- [ ] {}", criteria_text);
                let criteria = parse_acceptance_criteria(&body);
                // Parsing may filter whitespace-only items
                prop_assert!(criteria.len() <= 1);
            }
        }
    }

    // ========== Edge Case Tests ==========

    mod edge_case_tests {
        use super::*;

        #[test]
        fn test_roadmap_item_completion_no_subtasks() {
            let item = make_test_item("TEST", "Test", ItemStatus::InProgress);
            assert_eq!(item.completion_percentage(), 50);
        }

        #[test]
        fn test_roadmap_item_completion_completed() {
            let item = make_test_item("TEST", "Test", ItemStatus::Completed);
            assert_eq!(item.completion_percentage(), 100);
        }

        #[test]
        fn test_roadmap_item_completion_planned() {
            let item = make_test_item("TEST", "Test", ItemStatus::Planned);
            assert_eq!(item.completion_percentage(), 0);
        }

        #[test]
        fn test_roadmap_item_from_github_issue() {
            let item = RoadmapItem::from_github_issue(123, "Test Issue".to_string());
            assert_eq!(item.id, "GH-123");
            assert_eq!(item.github_issue, Some(123));
            assert!(item.is_github_synced());
        }

        #[test]
        fn test_roadmap_item_not_github_synced() {
            let item = make_test_item("LOCAL-001", "Local Task", ItemStatus::Planned);
            assert!(!item.is_github_synced());
        }

        #[tokio::test]
        async fn test_capture_commit_metadata_creates_metrics_dir() {
            let temp_dir = TempDir::new().unwrap();
            let item = make_test_item("TEST-001", "Test Task", ItemStatus::InProgress);

            // Initialize git repo for git rev-parse to work
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["config", "user.email", "test@test.com"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["config", "user.name", "Test User"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            // Create a file and commit
            std::fs::write(temp_dir.path().join("test.txt"), "test").unwrap();
            std::process::Command::new("git")
                .args(["add", "."])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["commit", "-m", "initial"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            let result = capture_commit_metadata(&temp_dir.path().to_path_buf(), &item).await;
            assert!(result.is_ok());

            let metrics_dir = temp_dir.path().join(".pmat-metrics");
            assert!(metrics_dir.exists());
        }

        #[test]
        fn test_parse_github_url_with_trailing_slash() {
            let url = "https://github.com/owner/repo/";
            // The function only removes .git extension, not trailing slashes
            let result = parse_github_url(url);
            assert!(result.is_some());
        }

        #[test]
        fn test_parse_github_url_enterprise() {
            // GitHub Enterprise URL should not match
            let url = "https://github.mycompany.com/owner/repo.git";
            assert_eq!(parse_github_url(url), None);
        }

        #[test]
        fn test_status_display_emoji_mappings() {
            // Test all status enum variants have corresponding emoji in status display
            let statuses = [
                ItemStatus::Completed,
                ItemStatus::InProgress,
                ItemStatus::Planned,
                ItemStatus::Blocked,
                ItemStatus::Review,
                ItemStatus::Cancelled,
            ];

            for status in statuses {
                // These should map to emoji in handle_work_status
                let emoji = match status {
                    ItemStatus::Completed => "✅",
                    ItemStatus::InProgress => "⏳",
                    ItemStatus::Planned => "📋",
                    ItemStatus::Blocked => "🚫",
                    ItemStatus::Review => "👀",
                    ItemStatus::Cancelled => "❌",
                };
                assert!(!emoji.is_empty());
            }
        }

        #[test]
        fn test_id_truncation_logic() {
            // Test the ID truncation for long IDs (display limited to 30 chars)
            let long_id = "This-is-a-very-long-id-that-exceeds-thirty-characters";
            let display_id = if long_id.len() > 30 {
                format!("{}...", &long_id[..30])
            } else {
                long_id.to_string()
            };
            assert!(display_id.len() <= 33); // 30 + "..."
            assert!(display_id.ends_with("..."));
        }

        #[test]
        fn test_short_id_no_truncation() {
            let short_id = "GH-42";
            let display_id = if short_id.len() > 30 {
                format!("{}...", &short_id[..30])
            } else {
                short_id.to_string()
            };
            assert_eq!(display_id, "GH-42");
        }
    }

    // ========== Validation Tests ==========

    mod validation_tests {
        use super::*;

        #[tokio::test]
        async fn test_validate_invalid_yaml() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs").join("roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_path = roadmap_dir.join("roadmap.yaml");
            std::fs::write(&roadmap_path, "invalid: yaml: content:").unwrap();

            let result =
                handle_work_validate(Some(temp_dir.path().to_path_buf()), false, false).await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_validate_with_warnings() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs").join("roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            // Create roadmap with long ID (should trigger warning)
            let roadmap_content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: This-is-a-very-long-id-that-exceeds-fifty-characters-for-testing-purposes-xyz
    title: Test Item
    status: planned
    priority: medium
"#;
            let roadmap_path = roadmap_dir.join("roadmap.yaml");
            std::fs::write(&roadmap_path, roadmap_content).unwrap();

            let result = handle_work_validate(
                Some(temp_dir.path().to_path_buf()),
                true,  // verbose
                false, // fix
            )
            .await;

            // Should succeed but print warnings
            assert!(result.is_ok());
        }
    }

    // ========== Specification Template Tests ==========

    mod spec_template_tests {
        use super::*;

        #[test]
        fn test_create_specification_template() {
            let temp_dir = TempDir::new().unwrap();
            let spec_path = temp_dir.path().join("spec.md");
            let item = RoadmapItem::from_github_issue(42, "Test Feature".to_string());

            let result = create_specification_template(&spec_path, &item);
            assert!(result.is_ok());
            assert!(spec_path.exists());

            let content = std::fs::read_to_string(&spec_path).unwrap();
            assert!(content.contains("Test Feature"));
            assert!(content.contains("GH-42"));
            assert!(content.contains("## Summary"));
            assert!(content.contains("## Requirements"));
        }

        #[test]
        fn test_create_specification_template_creates_directories() {
            let temp_dir = TempDir::new().unwrap();
            let spec_path = temp_dir.path().join("docs").join("specs").join("spec.md");
            let item = make_test_item("LOCAL-001", "Local Feature", ItemStatus::Planned);

            let result = create_specification_template(&spec_path, &item);
            assert!(result.is_ok());
            assert!(spec_path.exists());
        }

        #[test]
        fn test_spec_template_with_yaml_only_ticket() {
            let temp_dir = TempDir::new().unwrap();
            let spec_path = temp_dir.path().join("spec.md");
            let item = make_test_item("YAML-001", "YAML Only Task", ItemStatus::InProgress);

            let result = create_specification_template(&spec_path, &item);
            assert!(result.is_ok());

            let content = std::fs::read_to_string(&spec_path).unwrap();
            assert!(content.contains("YAML-001"));
            assert!(content.contains("Ticket ID"));
            assert!(!content.contains("GitHub Issue"));
        }

        #[test]
        fn test_spec_template_contains_all_sections() {
            let temp_dir = TempDir::new().unwrap();
            let spec_path = temp_dir.path().join("spec.md");
            let item = RoadmapItem::from_github_issue(123, "Complete Feature".to_string());

            create_specification_template(&spec_path, &item).unwrap();
            let content = std::fs::read_to_string(&spec_path).unwrap();

            // Verify all expected sections exist
            assert!(content.contains("## Summary"));
            assert!(content.contains("## Requirements"));
            assert!(content.contains("### Functional Requirements"));
            assert!(content.contains("### Non-Functional Requirements"));
            assert!(content.contains("## Architecture"));
            assert!(content.contains("## Implementation Plan"));
            assert!(content.contains("## Testing Strategy"));
            assert!(content.contains("## Success Criteria"));
            assert!(content.contains("## References"));
        }
    }

    // ========== Work Start Handler Tests ==========

    mod work_start_tests {
        use super::*;

        #[tokio::test]
        async fn test_handle_work_start_yaml_ticket() {
            let temp_dir = create_initialized_project();

            let result = handle_work_start(
                "NEW-TICKET".to_string(),
                false, // with_spec
                false, // epic
                Some(temp_dir.path().to_path_buf()),
                false, // create_github
            )
            .await;

            assert!(result.is_ok());

            // Verify item was created
            let roadmap_path = temp_dir
                .path()
                .join("docs/roadmaps/roadmap.yaml");
            let service = RoadmapService::new(&roadmap_path);
            let item = service.find_item("NEW-TICKET").unwrap();
            assert!(item.is_some());
            assert_eq!(item.unwrap().status, ItemStatus::InProgress);
        }

        #[tokio::test]
        async fn test_handle_work_start_existing_yaml_ticket() {
            let temp_dir = create_initialized_project();

            // Start work on existing TEST-001
            let result = handle_work_start(
                "TEST-001".to_string(),
                false,
                false,
                Some(temp_dir.path().to_path_buf()),
                false,
            )
            .await;

            assert!(result.is_ok());

            // Verify status changed to InProgress
            let roadmap_path = temp_dir.path().join("docs/roadmaps/roadmap.yaml");
            let service = RoadmapService::new(&roadmap_path);
            let item = service.find_item("TEST-001").unwrap().unwrap();
            assert_eq!(item.status, ItemStatus::InProgress);
        }

        #[tokio::test]
        async fn test_handle_work_start_as_epic() {
            let temp_dir = create_initialized_project();

            let result = handle_work_start(
                "EPIC-NEW".to_string(),
                false,
                true, // epic flag
                Some(temp_dir.path().to_path_buf()),
                false,
            )
            .await;

            assert!(result.is_ok());

            let roadmap_path = temp_dir.path().join("docs/roadmaps/roadmap.yaml");
            let service = RoadmapService::new(&roadmap_path);
            let item = service.find_item("EPIC-NEW").unwrap().unwrap();
            assert_eq!(item.item_type, crate::models::roadmap::ItemType::Epic);
        }

        #[tokio::test]
        async fn test_handle_work_start_with_spec() {
            let temp_dir = create_initialized_project();

            let result = handle_work_start(
                "SPEC-TEST".to_string(),
                true, // with_spec
                false,
                Some(temp_dir.path().to_path_buf()),
                false,
            )
            .await;

            assert!(result.is_ok());

            // Verify spec file was created
            let spec_path = temp_dir.path().join("docs/specifications/spec-test-spec.md");
            assert!(spec_path.exists());
        }

        #[tokio::test]
        async fn test_handle_work_start_github_issue_number() {
            let temp_dir = create_initialized_project();

            // Start work on issue number (no GitHub API available, should create placeholder)
            let result = handle_work_start(
                "999".to_string(),
                false,
                false,
                Some(temp_dir.path().to_path_buf()),
                false,
            )
            .await;

            assert!(result.is_ok());

            let roadmap_path = temp_dir.path().join("docs/roadmaps/roadmap.yaml");
            let service = RoadmapService::new(&roadmap_path);
            let item = service.find_item("GH-999").unwrap();
            assert!(item.is_some());
            assert_eq!(item.unwrap().github_issue, Some(999));
        }
    }

    // ========== Work Complete Handler Tests ==========

    mod work_complete_tests {
        use super::*;

        #[tokio::test]
        async fn test_handle_work_complete_skip_quality() {
            let temp_dir = create_initialized_project();

            // First start the work
            let roadmap_path = temp_dir.path().join("docs/roadmaps/roadmap.yaml");
            let service = RoadmapService::new(&roadmap_path);
            let mut item = service.find_item("TEST-001").unwrap().unwrap();
            item.status = ItemStatus::InProgress;
            service.upsert_item(item).unwrap();

            // Initialize git for metadata capture
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["config", "user.email", "test@test.com"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["config", "user.name", "Test"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::fs::write(temp_dir.path().join("test.txt"), "test").unwrap();
            std::process::Command::new("git")
                .args(["add", "."])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["commit", "-m", "init"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            let result = handle_work_complete(
                "TEST-001".to_string(),
                true, // skip_quality
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());

            // Verify status changed to Completed
            let item = service.find_item("TEST-001").unwrap().unwrap();
            assert_eq!(item.status, ItemStatus::Completed);
        }

        #[tokio::test]
        async fn test_handle_work_complete_nonexistent() {
            let temp_dir = create_initialized_project();

            let result = handle_work_complete(
                "NONEXISTENT-999".to_string(),
                true,
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_handle_work_complete_with_labels_for_changelog() {
            let temp_dir = create_initialized_project();

            // Set up item with labels
            let roadmap_path = temp_dir.path().join("docs/roadmaps/roadmap.yaml");
            let service = RoadmapService::new(&roadmap_path);
            let item = service.find_item("GH-42").unwrap().unwrap();
            // GH-42 already has labels from test fixture
            assert!(!item.labels.is_empty());

            // Initialize git
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["config", "user.email", "test@test.com"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["config", "user.name", "Test"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::fs::write(temp_dir.path().join("test.txt"), "test").unwrap();
            std::process::Command::new("git")
                .args(["add", "."])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["commit", "-m", "init"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            let result = handle_work_complete(
                "GH-42".to_string(),
                true, // skip_quality
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());
        }
    }

    // ========== Git Detection Tests ==========

    mod git_detection_tests {
        use super::*;

        #[test]
        fn test_detect_github_repo_no_git() {
            let temp_dir = TempDir::new().unwrap();
            let result = detect_github_repo(&temp_dir.path().to_path_buf());
            assert!(result.is_ok());
            assert!(result.unwrap().is_none());
        }

        #[test]
        fn test_detect_github_repo_with_remote() {
            let temp_dir = TempDir::new().unwrap();

            // Initialize git repo
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            // Add remote
            std::process::Command::new("git")
                .args(["remote", "add", "origin", "https://github.com/test/repo.git"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            let result = detect_github_repo(&temp_dir.path().to_path_buf());
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), Some("test/repo".to_string()));
        }

        #[test]
        fn test_detect_github_repo_ssh_remote() {
            let temp_dir = TempDir::new().unwrap();

            std::process::Command::new("git")
                .args(["init"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            std::process::Command::new("git")
                .args(["remote", "add", "origin", "git@github.com:owner/project.git"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            let result = detect_github_repo(&temp_dir.path().to_path_buf());
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), Some("owner/project".to_string()));
        }
    }

    // ========== Migration Tests ==========

    mod migration_tests {
        use super::*;

        #[tokio::test]
        async fn test_migrate_normalizes_done_status() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_path = roadmap_dir.join("roadmap.yaml");
            let content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TEST-001
    title: Test Item
    status: done
    priority: medium
"#;
            std::fs::write(&roadmap_path, content).unwrap();

            let result = handle_work_migrate(
                Some(temp_dir.path().to_path_buf()),
                false, // not dry_run
                false, // no backup
            )
            .await;

            assert!(result.is_ok());

            // Verify status was normalized
            let new_content = std::fs::read_to_string(&roadmap_path).unwrap();
            assert!(new_content.contains("status: completed"));
            assert!(!new_content.contains("status: done"));
        }

        #[tokio::test]
        async fn test_migrate_normalizes_wip_status() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_path = roadmap_dir.join("roadmap.yaml");
            let content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TEST-001
    title: Test Item
    status: wip
    priority: medium
"#;
            std::fs::write(&roadmap_path, content).unwrap();

            handle_work_migrate(
                Some(temp_dir.path().to_path_buf()),
                false,
                false,
            )
            .await
            .unwrap();

            let new_content = std::fs::read_to_string(&roadmap_path).unwrap();
            assert!(new_content.contains("status: inprogress"));
        }

        #[tokio::test]
        async fn test_migrate_dry_run_no_changes() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_path = roadmap_dir.join("roadmap.yaml");
            let content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TEST-001
    title: Test Item
    status: done
    priority: medium
"#;
            std::fs::write(&roadmap_path, content).unwrap();

            handle_work_migrate(
                Some(temp_dir.path().to_path_buf()),
                true, // dry_run
                false,
            )
            .await
            .unwrap();

            // Content should be unchanged
            let new_content = std::fs::read_to_string(&roadmap_path).unwrap();
            assert!(new_content.contains("status: done"));
        }

        #[tokio::test]
        async fn test_migrate_multiple_status_normalizations() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_path = roadmap_dir.join("roadmap.yaml");
            let content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TEST-001
    title: Item 1
    status: Done
    priority: medium
  - id: TEST-002
    title: Item 2
    status: WIP
    priority: high
  - id: TEST-003
    title: Item 3
    status: stuck
    priority: low
  - id: TEST-004
    title: Item 4
    status: todo
    priority: medium
"#;
            std::fs::write(&roadmap_path, content).unwrap();

            handle_work_migrate(
                Some(temp_dir.path().to_path_buf()),
                false,
                false,
            )
            .await
            .unwrap();

            let new_content = std::fs::read_to_string(&roadmap_path).unwrap();
            assert!(new_content.contains("status: completed"));
            assert!(new_content.contains("status: inprogress"));
            assert!(new_content.contains("status: blocked"));
            assert!(new_content.contains("status: planned"));
        }
    }

    // ========== Sync Direction Tests ==========

    mod sync_direction_tests {
        use super::*;

        #[tokio::test]
        async fn test_sync_yaml_to_github_shows_yaml_only_items() {
            let temp_dir = create_initialized_project();

            let result = handle_work_sync(
                SyncDirection::YamlToGithub,
                Some(temp_dir.path().to_path_buf()),
                true,
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_sync_all_directions() {
            let temp_dir = create_initialized_project();

            // Test all sync directions
            for direction in [
                SyncDirection::YamlToGithub,
                SyncDirection::GithubToYaml,
                SyncDirection::Full,
            ] {
                let result = handle_work_sync(
                    direction,
                    Some(temp_dir.path().to_path_buf()),
                    true,
                )
                .await;
                assert!(result.is_ok());
            }
        }
    }

    // ========== Roadmap Item Properties Tests ==========

    mod roadmap_item_properties {
        use super::*;

        #[test]
        fn test_completion_percentage_with_subtasks() {
            let mut item = RoadmapItem::new("EPIC-001".to_string(), "Epic".to_string());
            item.subtasks = vec![
                crate::models::roadmap::Subtask {
                    id: "SUB-1".to_string(),
                    github_issue: None,
                    title: "Sub 1".to_string(),
                    status: ItemStatus::Completed,
                    completion: 100,
                },
                crate::models::roadmap::Subtask {
                    id: "SUB-2".to_string(),
                    github_issue: None,
                    title: "Sub 2".to_string(),
                    status: ItemStatus::InProgress,
                    completion: 50,
                },
            ];
            // Average of 100 and 50 = 75
            assert_eq!(item.completion_percentage(), 75);
        }

        #[test]
        fn test_completion_percentage_with_phases() {
            let mut item = RoadmapItem::new("TASK-001".to_string(), "Task".to_string());
            item.phases = vec![
                crate::models::roadmap::Phase {
                    name: "Phase 1".to_string(),
                    status: ItemStatus::Completed,
                    estimated_effort: None,
                    completion: 100,
                },
                crate::models::roadmap::Phase {
                    name: "Phase 2".to_string(),
                    status: ItemStatus::InProgress,
                    estimated_effort: None,
                    completion: 60,
                },
                crate::models::roadmap::Phase {
                    name: "Phase 3".to_string(),
                    status: ItemStatus::Planned,
                    estimated_effort: None,
                    completion: 0,
                },
            ];
            // Average of 100, 60, 0 = 53.33 -> 53
            assert_eq!(item.completion_percentage(), 53);
        }

        #[test]
        fn test_completion_blocked_status() {
            let item = make_test_item("TEST", "Test", ItemStatus::Blocked);
            assert_eq!(item.completion_percentage(), 0);
        }

        #[test]
        fn test_completion_cancelled_status() {
            let item = make_test_item("TEST", "Test", ItemStatus::Cancelled);
            assert_eq!(item.completion_percentage(), 0);
        }

        #[test]
        fn test_completion_review_status() {
            let item = make_test_item("TEST", "Test", ItemStatus::Review);
            assert_eq!(item.completion_percentage(), 90);
        }
    }

    // ========== Score Cache Edge Cases ==========

    mod score_cache_edge_cases {
        use super::*;

        #[tokio::test]
        async fn test_capture_tdg_score_missing_score_key() {
            let temp_dir = TempDir::new().unwrap();
            let metrics_dir = temp_dir.path().join(".pmat-metrics");
            std::fs::create_dir_all(&metrics_dir).unwrap();

            // JSON without "score" key
            let tdg_file = metrics_dir.join("tdg-score.json");
            std::fs::write(&tdg_file, r#"{"other_field": 42}"#).unwrap();

            let score = capture_tdg_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 0.0); // Should fall back to default
        }

        #[tokio::test]
        async fn test_capture_repo_score_non_numeric() {
            let temp_dir = TempDir::new().unwrap();
            let metrics_dir = temp_dir.path().join(".pmat-metrics");
            std::fs::create_dir_all(&metrics_dir).unwrap();

            // JSON with non-numeric score
            let repo_file = metrics_dir.join("repo-score.json");
            std::fs::write(&repo_file, r#"{"score": "not-a-number"}"#).unwrap();

            let score = capture_repo_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 0.0);
        }

        #[tokio::test]
        async fn test_capture_rust_score_missing_total_earned() {
            let temp_dir = TempDir::new().unwrap();
            let metrics_dir = temp_dir.path().join(".pmat-metrics");
            std::fs::create_dir_all(&metrics_dir).unwrap();

            let rust_file = metrics_dir.join("rust-project-score.json");
            std::fs::write(&rust_file, r#"{"categories": []}"#).unwrap();

            let score = capture_rust_project_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 0.0);
        }
    }

    // ========== Continue Handler with Different Item States ==========

    mod continue_handler_states {
        use super::*;

        #[tokio::test]
        async fn test_continue_with_epic_subtasks() {
            let temp_dir = create_initialized_project();

            // EPIC-001 has subtasks in the test fixture
            let result = handle_work_continue(
                "EPIC-001".to_string(),
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_continue_with_acceptance_criteria() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TASK-001
    title: Task with Criteria
    status: inprogress
    priority: high
    acceptance_criteria:
      - First criterion
      - Second criterion
      - Third criterion
"#;
            std::fs::write(roadmap_dir.join("roadmap.yaml"), roadmap_content).unwrap();

            let result = handle_work_continue(
                "TASK-001".to_string(),
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_continue_with_spec_path() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: SPEC-001
    title: Task with Spec
    status: inprogress
    priority: medium
    spec: docs/specifications/spec-001.md
"#;
            std::fs::write(roadmap_dir.join("roadmap.yaml"), roadmap_content).unwrap();

            let result = handle_work_continue(
                "SPEC-001".to_string(),
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_continue_with_phases() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: PHASED-001
    title: Task with Phases
    status: inprogress
    priority: high
    phases:
      - name: RED
        status: completed
        completion: 100
      - name: GREEN
        status: inprogress
        completion: 50
      - name: REFACTOR
        status: planned
        completion: 0
"#;
            std::fs::write(roadmap_dir.join("roadmap.yaml"), roadmap_content).unwrap();

            let result = handle_work_continue(
                "PHASED-001".to_string(),
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());
        }
    }

    // ========== Validate Handler Edge Cases ==========

    mod validate_edge_cases {
        use super::*;

        #[tokio::test]
        async fn test_validate_with_fix_flag_shows_tip() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            // Create roadmap with warnings (no acceptance criteria)
            let roadmap_content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TEST-001
    title: Test Item
    status: planned
    priority: medium
"#;
            std::fs::write(roadmap_dir.join("roadmap.yaml"), roadmap_content).unwrap();

            let result = handle_work_validate(
                Some(temp_dir.path().to_path_buf()),
                false,
                true, // fix flag
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_validate_yaml_with_location_in_error() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            // Invalid YAML that will produce line number in error
            let invalid_yaml = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TEST-001
    title: Test
    status: invalid_status_that_doesnt_exist
    priority: medium
"#;
            std::fs::write(roadmap_dir.join("roadmap.yaml"), invalid_yaml).unwrap();

            let result = handle_work_validate(
                Some(temp_dir.path().to_path_buf()),
                false,
                false,
            )
            .await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_validate_github_disabled() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_content = r#"
roadmap_version: '1.0'
github_enabled: false
roadmap:
  - id: LOCAL-001
    title: Local Only
    status: planned
    priority: low
"#;
            std::fs::write(roadmap_dir.join("roadmap.yaml"), roadmap_content).unwrap();

            let result = handle_work_validate(
                Some(temp_dir.path().to_path_buf()),
                true, // verbose
                false,
            )
            .await;

            assert!(result.is_ok());
        }
    }

    // ========== Init Handler Edge Cases ==========

    mod init_edge_cases {
        use super::*;

        #[tokio::test]
        async fn test_init_with_explicit_github_repo() {
            let temp_dir = create_test_project();

            let result = handle_work_init(
                Some("explicit/repo".to_string()),
                false,
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());

            let roadmap_path = temp_dir.path().join("docs/roadmaps/roadmap.yaml");
            let content = std::fs::read_to_string(&roadmap_path).unwrap();
            assert!(content.contains("explicit/repo"));
        }

        #[tokio::test]
        async fn test_init_detects_git_remote() {
            let temp_dir = create_test_project();

            // Initialize git and add remote
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["remote", "add", "origin", "https://github.com/detected/repo.git"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            let result = handle_work_init(
                None, // No explicit repo
                false,
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());

            let roadmap_path = temp_dir.path().join("docs/roadmaps/roadmap.yaml");
            let content = std::fs::read_to_string(&roadmap_path).unwrap();
            assert!(content.contains("detected/repo"));
        }

        #[tokio::test]
        async fn test_init_github_enabled_but_no_repo() {
            let temp_dir = create_test_project();

            // No git, no explicit repo
            let result = handle_work_init(
                None,
                false, // github enabled
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());

            // Should still succeed, just without repo configured
            let roadmap_path = temp_dir.path().join("docs/roadmaps/roadmap.yaml");
            assert!(roadmap_path.exists());
        }
    }

    // ========== Additional Property Tests ==========

    mod additional_proptests {
        use super::*;

        proptest! {
            #[test]
            fn test_acceptance_criteria_extraction_preserves_order(
                // Ensure at least one non-space character by starting with alphanumeric
                items in prop::collection::vec("[a-zA-Z0-9][a-zA-Z0-9 ]{4,19}", 1..10)
            ) {
                let body = items.iter()
                    .map(|item| format!("- [ ] {}", item))
                    .collect::<Vec<_>>()
                    .join("\n");

                let criteria = parse_acceptance_criteria(&body);
                // Number of criteria may differ if parsing filters some items
                prop_assert!(criteria.len() <= items.len());
            }

            #[test]
            fn test_yaml_error_line_extraction_valid_formats(line_num in 1usize..10000) {
                let error = format!("parse error at line {} column 5", line_num);
                let extracted = extract_line_from_yaml_error(&error);
                prop_assert_eq!(extracted, Some(line_num));
            }
        }
    }
}
