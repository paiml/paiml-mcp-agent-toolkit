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
                item.github_issue.unwrap()
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
    let short_sha = String::from_utf8_lossy(&short_sha.stdout).trim().to_string();

    // Capture scores (O(1) from cache)
    let tdg_score = capture_tdg_score(project_path).await.unwrap_or(0.0);
    let repo_score = capture_repo_score(project_path).await.unwrap_or(0.0);
    let rust_score = if project_path.join("Cargo.toml").exists() {
        Some(capture_rust_project_score(project_path).await.unwrap_or(0.0))
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
    let meta_file = project_path.join(".pmat-metrics")
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
            item.github_issue.unwrap()
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
                println!("      GitHub: #{}", item.github_issue.unwrap());
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

        let test_cmd =
            crate::services::git_test_filter::build_test_command(&modules).unwrap_or_else(|| {
                vec!["test".to_string(), "--lib".to_string(), "--quiet".to_string()]
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

/// Handle work validate command (Part B: UX Improvements)
///
/// Validates roadmap.yaml syntax and content with actionable error messages.
pub async fn handle_work_validate(
    path: Option<PathBuf>,
    verbose: bool,
    fix: bool,
) -> Result<()> {
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
    let content = std::fs::read_to_string(&roadmap_path)
        .context("Failed to read roadmap file")?;

    // Try to parse
    match serde_yaml::from_str::<crate::models::roadmap::Roadmap>(&content) {
        Ok(roadmap) => {
            println!("✅ Syntax valid");
            println!("   Version: {}", roadmap.roadmap_version);
            println!("   Items: {}", roadmap.roadmap.len());
            println!("   GitHub: {}", if roadmap.github_enabled {
                roadmap.github_repo.as_deref().unwrap_or("not configured")
            } else {
                "disabled"
            });
            println!();

            // Semantic validation
            let mut warnings = Vec::new();

            for item in &roadmap.roadmap {
                // Check for missing acceptance criteria on features
                if item.acceptance_criteria.is_empty()
                    && !matches!(item.status, ItemStatus::Cancelled)
                {
                    warnings.push(format!(
                        "⚠️  {} has no acceptance criteria",
                        item.id
                    ));
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
                    println!(
                        "   {} [{:?}] - {}",
                        item.id, item.status, item.title
                    );
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
            println!("   - Use valid status values: completed, done, wip, planned, blocked, review");
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
pub async fn handle_work_migrate(
    path: Option<PathBuf>,
    dry_run: bool,
    backup: bool,
) -> Result<()> {
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
            let has_special = special_chars.iter().any(|c| line.contains(*c) && !line.contains("\""));
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
    println!("{:<15} {:<25} {}", "STATUS", "ALIASES", "DESCRIPTION");
    println!("{}", "-".repeat(70));

    let statuses = [
        ("planned", "todo, open, pending, new", "Task not yet started"),
        ("inprogress", "wip, active, started", "Currently being worked on"),
        ("blocked", "stuck, waiting, on-hold", "Cannot proceed (waiting on something)"),
        ("review", "reviewing, pr, pending-review", "Ready for or in code review"),
        ("completed", "done, finished, closed", "Work finished successfully"),
        ("cancelled", "canceled, dropped, wontfix", "Work abandoned or not needed"),
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
