// Work command handlers for unified GitHub/YAML workflow (Issue #75)
//
// Implements the hybrid write-through architecture for GitHub and YAML tracking.

use crate::cli::commands::SyncDirection;
use crate::models::roadmap::{ItemStatus, Priority, RoadmapItem};
use crate::services::changelog_manager::{ChangeCategory, ChangelogEntry};
#[cfg(feature = "github-api")]
use crate::services::github_client::GitHubClient;
use crate::services::hook_manager;
use crate::services::roadmap_service::RoadmapService;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// Quality handlers extracted to work_quality_handlers.rs for file health compliance (CB-040)
pub use super::work_quality_handlers::{run_popper_falsification, run_quality_gates, FalsificationResult};

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

                    let mut item =
                        RoadmapItem::from_github_issue(issue_num, gh_issue.title.clone());
                    item.labels = gh_issue.labels.clone();

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

/// Minimal issue info for API-agnostic GitHub operations
/// Works with either octocrab (github-api feature) or gh CLI fallback
#[derive(Debug, Clone)]
pub struct GitHubIssueInfo {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub labels: Vec<String>,
}

/// Fetch GitHub issue details using octocrab (requires github-api feature)
#[cfg(feature = "github-api")]
async fn fetch_github_issue(repo: &str, issue_num: u64) -> Result<GitHubIssueInfo> {
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
async fn fetch_github_issue(repo: &str, issue_num: u64) -> Result<GitHubIssueInfo> {
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
async fn create_github_issue_from_item(repo: &str, item: &RoadmapItem) -> Result<GitHubIssueInfo> {
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
async fn create_github_issue_from_item(repo: &str, item: &RoadmapItem) -> Result<GitHubIssueInfo> {
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

/// Handle work add command (CRUD: Create)
///
/// Creates a new work ticket in roadmap.yaml with optional GitHub issue creation.
pub async fn handle_work_add(
    title: String,
    description: Option<String>,
    priority: crate::cli::commands::WorkPriority,
    tags: Option<String>,
    path: Option<PathBuf>,
    create_github: bool,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    // Validate roadmap exists
    if !service.exists() {
        anyhow::bail!(
            "No roadmap found at {}. Run 'pmat work init' first.",
            roadmap_path.display()
        );
    }

    // Load existing roadmap to find next available ID
    let roadmap = service.load()?;
    let next_id = generate_next_id(&roadmap);

    // Create new item
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let item = crate::models::roadmap::RoadmapItem {
        id: next_id.clone(),
        github_issue: None,
        item_type: crate::models::roadmap::ItemType::Task,
        title: title.clone(),
        status: crate::models::roadmap::ItemStatus::Planned,
        priority: priority.to_roadmap_priority(),
        assigned_to: None,
        created: now.clone(),
        updated: now,
        spec: None,
        acceptance_criteria: description
            .as_ref()
            .map(|d| vec![d.clone()])
            .unwrap_or_default(),
        phases: vec![],
        subtasks: vec![],
        estimated_effort: None,
        labels: tags
            .clone()
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default(),
        notes: None,
    };

    // Save to roadmap
    service.upsert_item(item)?;

    println!("✅ Created ticket: {}", next_id);
    println!("   Title: {}", title);
    println!("   Priority: {:?}", priority);
    if let Some(desc) = description {
        println!("   Description: {}", desc);
    }
    if let Some(t) = tags {
        println!("   Tags: {}", t);
    }

    // Create GitHub issue if requested
    if create_github {
        println!("\n⚠️  GitHub issue creation not yet implemented. Use 'pmat work sync' after creating the ticket.");
    }

    Ok(())
}

/// Handle work list command (CRUD: Read - simple list)
///
/// Lists all work tickets with optional filtering.
pub async fn handle_work_list(
    status: Option<String>,
    priority: Option<crate::cli::commands::WorkPriority>,
    count_only: bool,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    if !service.exists() {
        anyhow::bail!(
            "No roadmap found at {}. Run 'pmat work init' first.",
            roadmap_path.display()
        );
    }

    let roadmap = service.load()?;

    // Filter items
    let items: Vec<_> = roadmap
        .roadmap
        .iter()
        .filter(|item| {
            // Filter by status if specified
            if let Some(ref s) = status {
                let item_status = format!("{:?}", item.status).to_lowercase();
                if !item_status.contains(&s.to_lowercase()) {
                    return false;
                }
            }
            // Filter by priority if specified
            if let Some(ref p) = priority {
                let roadmap_priority = p.to_roadmap_priority();
                if item.priority != roadmap_priority {
                    return false;
                }
            }
            true
        })
        .collect();

    if count_only {
        println!("{}", items.len());
        return Ok(());
    }

    if items.is_empty() {
        println!("No tickets found matching criteria.");
        return Ok(());
    }

    // Print header
    println!("{:<12} {:<12} {:<10} TITLE", "ID", "STATUS", "PRIORITY");
    println!("{}", "-".repeat(70));

    // Print items
    for item in items {
        let status_str = format!("{:?}", item.status).to_lowercase();
        let priority_str = format!("{:?}", item.priority).to_lowercase();
        let title_truncated = if item.title.len() > 40 {
            format!("{}...", &item.title[..37])
        } else {
            item.title.clone()
        };
        println!(
            "{:<12} {:<12} {:<10} {}",
            item.id, status_str, priority_str, title_truncated
        );
    }

    Ok(())
}

/// Handle work edit command (CRUD: Update)
///
/// Edits an existing work ticket.
pub async fn handle_work_edit(
    id: String,
    title: Option<String>,
    description: Option<String>,
    priority: Option<crate::cli::commands::WorkPriority>,
    status: Option<String>,
    tags: Option<String>,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    if !service.exists() {
        anyhow::bail!(
            "No roadmap found at {}. Run 'pmat work init' first.",
            roadmap_path.display()
        );
    }

    // Find the item (with fuzzy matching)
    let item = find_item_fuzzy(&service, &id)?;
    let mut updated_item = item.clone();
    let mut changes = vec![];

    // Apply changes
    if let Some(new_title) = title {
        updated_item.title = new_title.clone();
        changes.push(format!("title: {}", new_title));
    }

    if let Some(desc) = description {
        updated_item.acceptance_criteria = vec![desc.clone()];
        changes.push(format!("description: {}", desc));
    }

    if let Some(p) = priority {
        updated_item.priority = p.to_roadmap_priority();
        changes.push(format!("priority: {:?}", p));
    }

    if let Some(s) = status {
        let new_status = crate::models::roadmap::ItemStatus::from_string(&s)
            .map_err(|e| anyhow::anyhow!("Invalid status '{}': {}", s, e))?;
        updated_item.status = new_status;
        changes.push(format!("status: {}", s));
    }

    if let Some(t) = tags {
        updated_item.labels = t.split(',').map(|s| s.trim().to_string()).collect();
        changes.push(format!("labels: {}", t));
    }

    if changes.is_empty() {
        println!("⚠️  No changes specified. Use --title, --description, --priority, --status, or --tags.");
        return Ok(());
    }

    // Update timestamp
    updated_item.updated = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // Save
    service.upsert_item(updated_item)?;

    println!("✅ Updated ticket: {}", item.id);
    for change in changes {
        println!("   {}", change);
    }

    Ok(())
}

/// Handle work delete command (CRUD: Delete)
///
/// Deletes a work ticket from roadmap.yaml.
pub async fn handle_work_delete(id: String, force: bool, path: Option<PathBuf>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    if !service.exists() {
        anyhow::bail!(
            "No roadmap found at {}. Run 'pmat work init' first.",
            roadmap_path.display()
        );
    }

    // Find the item (with fuzzy matching)
    let item = find_item_fuzzy(&service, &id)?;

    // Confirm deletion unless --force
    if !force {
        println!("About to delete ticket:");
        println!("  ID: {}", item.id);
        println!("  Title: {}", item.title);
        println!("  Status: {:?}", item.status);
        println!();
        println!("⚠️  Use --force to skip this confirmation.");
        return Ok(());
    }

    // Delete
    service.remove_item(&item.id)?;
    println!("🗑️  Deleted ticket: {} - {}", item.id, item.title);

    Ok(())
}

/// Handle work annotate command - show unified quality metrics for a ticket
pub async fn handle_work_annotate(
    id: String,
    path: Option<PathBuf>,
    format: crate::cli::commands::AnnotateOutputFormat,
    with_churn: bool,
    churn_days: u32,
) -> Result<()> {
    use crate::cli::commands::AnnotateOutputFormat;
    use crate::services::spec_parser::SpecParser;

    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    if !service.exists() {
        anyhow::bail!(
            "No roadmap found at {}. Run 'pmat work init' first.",
            roadmap_path.display()
        );
    }

    // Find the ticket
    let item = find_item_fuzzy(&service, &id)?;

    // Collect annotations
    let mut annotations = TicketAnnotations {
        ticket_id: item.id.clone(),
        title: item.title.clone(),
        status: format!("{:?}", item.status),
        priority: format!("{:?}", item.priority),
        spec_path: item.spec.clone(),
        spec_score: None,
        files: vec![],
        avg_tdg: None,
        total_churn: None,
        churn_hotspots: vec![],
        coverage_percent: None,
        repeated_fixes: vec![],
    };

    // Get spec score if spec exists
    if let Some(ref spec_path) = item.spec {
        let full_spec_path = project_path.join(spec_path);
        if full_spec_path.exists() {
            let parser = SpecParser::new();
            if let Ok(spec) = parser.parse_file(&full_spec_path) {
                annotations.spec_score = Some(calculate_spec_score_simple(&spec));
            }
        }
    }

    // Find related files from acceptance criteria or labels
    let related_files = find_related_files(&item, &project_path);
    annotations.files = related_files.clone();

    // Calculate TDG for related files (simplified - just count)
    if !related_files.is_empty() {
        // For now, show file count as proxy for complexity
        annotations.avg_tdg = Some(related_files.len() as f64 * 1.5); // Placeholder
    }

    // Churn analysis if requested
    if with_churn && !related_files.is_empty() {
        let churn_result = analyze_churn_simple(&project_path, &related_files, churn_days);
        annotations.total_churn = Some(churn_result.total_commits);
        annotations.churn_hotspots = churn_result.hotspots;
        annotations.repeated_fixes = churn_result.repeated_fixes;
    }

    // Output based on format
    match format {
        AnnotateOutputFormat::Text => print_annotations_text(&annotations),
        AnnotateOutputFormat::Json => print_annotations_json(&annotations)?,
        AnnotateOutputFormat::Markdown => print_annotations_markdown(&annotations),
    }

    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
struct TicketAnnotations {
    ticket_id: String,
    title: String,
    status: String,
    priority: String,
    spec_path: Option<PathBuf>,
    spec_score: Option<f64>,
    files: Vec<PathBuf>,
    avg_tdg: Option<f64>,
    total_churn: Option<usize>,
    churn_hotspots: Vec<String>,
    coverage_percent: Option<f64>,
    repeated_fixes: Vec<RepeatedFix>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct RepeatedFix {
    file: String,
    line_range: String,
    fix_count: usize,
    description: String,
}

struct ChurnResult {
    total_commits: usize,
    hotspots: Vec<String>,
    repeated_fixes: Vec<RepeatedFix>,
}

fn calculate_spec_score_simple(spec: &crate::services::spec_parser::ParsedSpec) -> f64 {
    let mut score = 0.0;
    if !spec.issue_refs.is_empty() {
        score += 10.0;
    }
    score += (spec.code_examples.len().min(5) * 4) as f64;
    score += (spec.acceptance_criteria.len().min(10) * 3) as f64;
    score += (spec.claims.len().min(20)) as f64;
    if !spec.title.is_empty() {
        score += 5.0;
    }
    score += (spec.test_requirements.len().min(5) * 3) as f64;
    score.min(100.0)
}

fn find_related_files(
    item: &crate::models::roadmap::RoadmapItem,
    project_path: &Path,
) -> Vec<PathBuf> {
    let mut files = Vec::new();

    // Check if spec mentions files
    if let Some(ref spec_path) = item.spec {
        let full_path = project_path.join(spec_path);
        if full_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                // Extract file paths from spec (e.g., `server/src/foo.rs`)
                let re = regex::Regex::new(r"`([\w/._-]+\.(?:rs|ts|py|go|js))`").ok();
                if let Some(re) = re {
                    for cap in re.captures_iter(&content) {
                        if let Some(m) = cap.get(1) {
                            let file_path = project_path.join(m.as_str());
                            if file_path.exists() {
                                files.push(PathBuf::from(m.as_str()));
                            }
                        }
                    }
                }
            }
        }
    }

    // Also check labels for file hints
    for label in &item.labels {
        if label.ends_with(".rs") || label.ends_with(".ts") {
            let file_path = project_path.join(label);
            if file_path.exists() {
                files.push(PathBuf::from(label));
            }
        }
    }

    files.into_iter().take(10).collect() // Limit to 10 files
}

fn analyze_churn_simple(project_path: &Path, files: &[PathBuf], days: u32) -> ChurnResult {
    let mut total_commits = 0;
    let mut hotspots = Vec::new();
    let mut repeated_fixes = Vec::new();

    for file in files {
        // Run git log to count commits
        let output = std::process::Command::new("git")
            .args([
                "log",
                "--oneline",
                &format!("--since={} days ago", days),
                "--",
                &file.to_string_lossy(),
            ])
            .current_dir(project_path)
            .output();

        if let Ok(output) = output {
            let commit_count = String::from_utf8_lossy(&output.stdout)
                .lines()
                .count();
            total_commits += commit_count;

            if commit_count > 5 {
                hotspots.push(format!("{}: {} commits", file.display(), commit_count));
            }

            // Check for repeated fix patterns (same file, similar commit messages)
            let log_output = std::process::Command::new("git")
                .args([
                    "log",
                    "--oneline",
                    &format!("--since={} days ago", days),
                    "--grep=fix",
                    "-i",
                    "--",
                    &file.to_string_lossy(),
                ])
                .current_dir(project_path)
                .output();

            if let Ok(log_output) = log_output {
                let fix_count = String::from_utf8_lossy(&log_output.stdout)
                    .lines()
                    .count();
                if fix_count >= 2 {
                    repeated_fixes.push(RepeatedFix {
                        file: file.to_string_lossy().to_string(),
                        line_range: "various".to_string(),
                        fix_count,
                        description: format!("{} fix commits in {} days (Tarantula alert)", fix_count, days),
                    });
                }
            }
        }
    }

    ChurnResult {
        total_commits,
        hotspots,
        repeated_fixes,
    }
}

fn print_annotations_text(ann: &TicketAnnotations) {
    println!("📊 Quality Annotations for {}\n", ann.ticket_id);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Title:    {}", ann.title);
    println!("Status:   {}", ann.status);
    println!("Priority: {}", ann.priority);
    println!();

    // Spec section
    println!("📋 SPECIFICATION");
    if let Some(ref spec) = ann.spec_path {
        println!("   Path:  {}", spec.display());
        if let Some(score) = ann.spec_score {
            let status = if score >= 95.0 { "✅" } else { "❌" };
            println!("   Score: {:.1}/100 {}", score, status);
        }
    } else {
        println!("   ⚠️  No specification linked");
    }
    println!();

    // Files section
    println!("📁 RELATED FILES ({})", ann.files.len());
    if ann.files.is_empty() {
        println!("   No files detected");
    } else {
        for f in &ann.files {
            println!("   • {}", f.display());
        }
    }
    println!();

    // TDG section
    println!("📈 TDG (Test-Driven Grade)");
    if let Some(tdg) = ann.avg_tdg {
        println!("   Avg Score: {:.1}/10", tdg);
    } else {
        println!("   Not calculated (no files)");
    }
    println!();

    // Churn section
    println!("🔄 CHURN ANALYSIS");
    if let Some(churn) = ann.total_churn {
        println!("   Total Commits: {}", churn);
        if !ann.churn_hotspots.is_empty() {
            println!("   Hotspots:");
            for h in &ann.churn_hotspots {
                println!("     ⚠️  {}", h);
            }
        }
    } else {
        println!("   Run with --with-churn to analyze");
    }
    println!();

    // Tarantula section
    println!("🔴 TARANTULA FAULT DETECTION");
    if ann.repeated_fixes.is_empty() {
        println!("   ✅ No repeated fix patterns detected");
    } else {
        for fix in &ann.repeated_fixes {
            println!("   ⚠️  {}: {}", fix.file, fix.description);
        }
    }
    println!();

    // Coverage section
    println!("📊 COVERAGE");
    if let Some(cov) = ann.coverage_percent {
        let status = if cov >= 95.0 { "✅" } else { "❌" };
        println!("   {:.1}% {}", cov, status);
    } else {
        println!("   Not available (run coverage analysis)");
    }
}

fn print_annotations_json(ann: &TicketAnnotations) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(ann)?);
    Ok(())
}

fn print_annotations_markdown(ann: &TicketAnnotations) {
    println!("# Quality Annotations: {}\n", ann.ticket_id);
    println!("**Title:** {}", ann.title);
    println!("**Status:** {} | **Priority:** {}\n", ann.status, ann.priority);

    println!("## Specification");
    if let Some(ref spec) = ann.spec_path {
        let score_str = ann.spec_score.map(|s| format!("{:.1}/100", s)).unwrap_or_else(|| "N/A".to_string());
        println!("| Metric | Value |");
        println!("|--------|-------|");
        println!("| Path | {} |", spec.display());
        println!("| Score | {} |", score_str);
    } else {
        println!("⚠️ No specification linked\n");
    }

    println!("\n## Metrics Summary");
    println!("| Metric | Value | Status |");
    println!("|--------|-------|--------|");
    println!("| Files | {} | - |", ann.files.len());
    println!("| TDG | {} | {} |",
        ann.avg_tdg.map(|t| format!("{:.1}", t)).unwrap_or_else(|| "N/A".to_string()),
        if ann.avg_tdg.map(|t| t >= 7.0).unwrap_or(false) { "✅" } else { "⚠️" }
    );
    println!("| Churn | {} | {} |",
        ann.total_churn.map(|c| c.to_string()).unwrap_or_else(|| "N/A".to_string()),
        if ann.total_churn.map(|c| c < 10).unwrap_or(true) { "✅" } else { "⚠️" }
    );
    println!("| Repeated Fixes | {} | {} |",
        ann.repeated_fixes.len(),
        if ann.repeated_fixes.is_empty() { "✅" } else { "🔴" }
    );
}

/// Generate the next available ID for a new ticket
fn generate_next_id(roadmap: &crate::models::roadmap::Roadmap) -> String {
    let mut max_num = 0u32;

    for item in &roadmap.roadmap {
        // Try to extract number from IDs like "PMAT-001", "GH-123", etc.
        if let Some(num_str) = item.id.split('-').next_back() {
            if let Ok(num) = num_str.parse::<u32>() {
                max_num = max_num.max(num);
            }
        }
    }

    format!("PMAT-{:03}", max_num + 1)
}

/// Find an item with fuzzy ID matching (case-insensitive, partial match)
fn find_item_fuzzy(
    service: &RoadmapService,
    id: &str,
) -> Result<crate::models::roadmap::RoadmapItem> {
    // First try exact match
    if let Ok(Some(item)) = service.find_item(id) {
        return Ok(item);
    }

    // Load all items for fuzzy matching
    let roadmap = service.load()?;

    // Try case-insensitive exact match
    let id_lower = id.to_lowercase();
    for item in &roadmap.roadmap {
        if item.id.to_lowercase() == id_lower {
            return Ok(item.clone());
        }
    }

    // Try partial match (ID contains the search string)
    let mut matches: Vec<_> = roadmap
        .roadmap
        .iter()
        .filter(|item| item.id.to_lowercase().contains(&id_lower))
        .collect();

    match matches.len() {
        0 => anyhow::bail!(
            "Ticket '{}' not found. Use 'pmat work list' to see available tickets.",
            id
        ),
        1 => Ok(matches.pop().expect("verified 1 element exists").clone()),
        _ => {
            let match_ids: Vec<_> = matches.iter().map(|i| i.id.as_str()).collect();
            anyhow::bail!(
                "Ambiguous ID '{}'. Multiple matches: {}. Please be more specific.",
                id,
                match_ids.join(", ")
            )
        }
    }
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


// Tests extracted to work_handlers_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "work_handlers_tests.rs"]
mod tests;
