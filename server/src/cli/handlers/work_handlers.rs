// Work command handlers for unified GitHub/YAML workflow (Issue #75)
//
// Implements the hybrid write-through architecture for GitHub and YAML tracking.

use crate::cli::commands::SyncDirection;
use crate::models::roadmap::{ItemStatus, Priority, RoadmapItem};
use crate::services::github_client::GitHubClient;
use crate::services::hook_manager;
use crate::services::roadmap_service::RoadmapService;
use anyhow::{Context, Result};
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
    println!("   GitHub integration: {}", if github_enabled { "✅ enabled" } else { "❌ disabled" });
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
    _epic: bool,
    path: Option<PathBuf>,
    create_github: bool,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    println!("🚀 Starting work on: {}", id);
    println!();

    // Load roadmap
    let mut roadmap = service
        .load()
        .context("Failed to load roadmap. Run `pmat work init` first.")?;

    // Determine if this is a GitHub issue or YAML ticket
    let is_github_issue = id.parse::<u64>().is_ok();

    let item = if is_github_issue {
        let issue_num: u64 = id.parse()?;
        println!("📋 Type: GitHub issue #{}", issue_num);

        // Fetch from GitHub API if repo is configured
        let mut item = if let Some(ref repo) = roadmap.github_repo {
            match fetch_github_issue(repo, issue_num).await {
                Ok(gh_issue) => {
                    println!("   ✅ Fetched from GitHub: {}", gh_issue.title);

                    // Extract labels
                    let labels: Vec<String> = gh_issue
                        .labels
                        .iter()
                        .map(|l| l.name.clone())
                        .collect();

                    let mut item = RoadmapItem::from_github_issue(issue_num, gh_issue.title.clone());
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

    // Update roadmap
    roadmap.upsert_item(item.clone());
    service.save(&roadmap)?;

    println!("✅ Updated roadmap: {}", roadmap_path.display());

    // Create specification if requested
    if with_spec {
        let spec_path = if is_github_issue {
            project_path.join(format!("docs/specifications/{:03}-spec.md", item.github_issue.unwrap()))
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
                    anyhow::bail!("Quality gates failed. Fix issues or use --skip-quality to bypass.");
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
    println!();

    // Next steps
    println!("🎯 Next steps:");
    println!("   1. Create commit: git commit -m \"feat: {} (Refs {})\"", item.title, id);
    if item.is_github_synced() {
        println!("   2. Close GitHub issue: gh issue close {}", item.github_issue.unwrap());
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

    let roadmap = service
        .load()
        .context("Failed to load roadmap. Run `pmat work init` first.")?;

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
            println!(
                "   {} {} - {} ({}%)",
                emoji, item.id, item.title, progress
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

    let roadmap = service
        .load()
        .context("Failed to load roadmap. Run `pmat work init` first.")?;

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
        format!("*Created via `pmat work start --create-github`*")
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
        format!("**GitHub Issue**: [#{}](https://github.com/YOUR_ORG/YOUR_REPO/issues/{})", issue, issue)
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
        item.title,
        item.id,
        item.created,
        item.updated,
        item.title,
        github_link
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

    // 1. Run cargo test
    println!("   🧪 Running tests...");
    let test_status = Command::new("cargo")
        .arg("test")
        .arg("--lib")
        .arg("--quiet")
        .current_dir(project_path)
        .status()
        .context("Failed to run cargo test")?;

    if test_status.success() {
        println!("      ✅ Tests passed");
    } else {
        println!("      ❌ Tests failed");
        all_passed = false;
    }

    // 2. Run cargo clippy
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
