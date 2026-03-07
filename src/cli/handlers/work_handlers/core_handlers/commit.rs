#![cfg_attr(coverage_nightly, coverage(off))]
// Commit metadata capture and changelog update helpers

use crate::models::roadmap::RoadmapItem;
use anyhow::Result;
use std::path::{Path, PathBuf};

use super::types::CommitMetadata;

/// Capture TDG score (O(1) from cache)
pub(super) async fn capture_tdg_score(project_path: &PathBuf) -> Result<f64> {
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
pub(super) async fn capture_repo_score(project_path: &PathBuf) -> Result<f64> {
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
pub(super) async fn capture_rust_project_score(project_path: &PathBuf) -> Result<f64> {
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

/// Capture commit metadata (O(1) from .pmat-metrics/ cache)
pub(super) async fn capture_commit_metadata(
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

/// Update changelog from item labels (helper for handle_work_complete)
pub(super) fn update_changelog(project_path: &PathBuf, item: &RoadmapItem) {
    use crate::services::changelog_manager::{ChangeCategory, ChangelogEntry};

    if item.labels.is_empty() {
        return;
    }

    if let Some(category) = ChangeCategory::from_labels(&item.labels) {
        let entry = ChangelogEntry::new(category, item.title.clone(), item.github_issue);
        match crate::services::changelog_manager::add_to_changelog(project_path, entry) {
            Ok(()) => println!("✅ Updated CHANGELOG.md"),
            Err(e) => {
                println!("⚠️  Failed to update CHANGELOG.md: {}", e);
                println!("   You may need to update it manually");
            }
        }
    } else {
        println!("ℹ️  No changelog category inferred from labels");
    }
}

/// Print completion next steps with commit metadata (helper for handle_work_complete)
pub(super) fn print_complete_next_steps(item: &RoadmapItem, id: &str, metadata: &CommitMetadata) {
    println!("🎯 Next steps:");
    let rust_score_line = metadata
        .rust_project_score
        .map(|s| format!("Rust-Score: {:.1}/134\n", s))
        .unwrap_or_default();
    let commit_msg = format!(
        "feat: {} (Refs {})\n\nWork-Item: {}\nTDG-Score: {:.1}/100\nRepo-Score: {:.1}/100\n{}Metrics: .pmat-metrics/commit-*-meta.json",
        item.title, id, item.id, metadata.tdg_score, metadata.repo_score, rust_score_line
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
}

/// Auto-commit tracked files modified by `pmat work complete`.
///
/// Prevents the circular dependency where `pmat work complete` creates
/// dirty files that the user must manually commit before pushing.
/// Files committed: docs/roadmaps/roadmap.yaml, CHANGELOG.md (if modified).
pub(super) fn auto_commit_work_files(
    project_path: &Path,
    item: &RoadmapItem,
    id: &str,
    metadata: &CommitMetadata,
) {
    use std::process::Command;

    // Stage files that pmat work complete may have modified
    let roadmap_path = "docs/roadmaps/roadmap.yaml";
    let changelog_path = "CHANGELOG.md";

    let mut files_to_add = vec![roadmap_path];
    if project_path.join(changelog_path).exists() {
        // Only stage CHANGELOG.md if it has changes
        let status = Command::new("git")
            .args(["diff", "--quiet", "--", changelog_path])
            .current_dir(project_path)
            .status();
        if matches!(status, Ok(s) if !s.success()) {
            files_to_add.push(changelog_path);
        }
    }

    // git add the modified files
    let add_status = Command::new("git")
        .arg("add")
        .args(&files_to_add)
        .current_dir(project_path)
        .status();

    if !matches!(add_status, Ok(s) if s.success()) {
        println!("⚠️  Auto-commit: failed to stage files");
        println!();
        print_complete_next_steps(item, id, metadata);
        return;
    }

    // Build commit message
    let rust_score_line = metadata
        .rust_project_score
        .map(|s| format!("Rust-Score: {:.1}/134\n", s))
        .unwrap_or_default();
    let commit_msg = format!(
        "feat: {} (Refs {})\n\nWork-Item: {}\nTDG-Score: {:.1}/100\nRepo-Score: {:.1}/100\n{}Metrics: .pmat-metrics/commit-*-meta.json",
        item.title, id, item.id, metadata.tdg_score, metadata.repo_score, rust_score_line
    );

    let commit_status = Command::new("git")
        .args(["commit", "-m", &commit_msg, "--no-verify"])
        .current_dir(project_path)
        .status();

    match commit_status {
        Ok(s) if s.success() => {
            println!();
            println!("✅ Auto-committed work completion files");
            if item.is_github_synced() {
                println!(
                    "🎯 Next: gh issue close {}",
                    item.github_issue.expect("internal error")
                );
            }
            println!("🎯 Next: git push origin master");
        }
        _ => {
            println!("⚠️  Auto-commit failed (nothing to commit or hook error)");
            println!();
            print_complete_next_steps(item, id, metadata);
        }
    }
}
