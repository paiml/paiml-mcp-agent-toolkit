#![cfg_attr(coverage_nightly, coverage(off))]
// Commit metadata capture and changelog update helpers

use crate::cli::colors as c;
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

/// The denominator for a `Rust-Score` trailer: the rubric's own total.
///
/// Was the literal `134`, while the numerator is `total_earned` from the
/// **289**-point rubric (`RustProjectScoreOrchestrator::max_points()`, asserted
/// at `orchestrator.rs:690` as 130+26+20+15+10+12+16+20+10+15+15). A recorded
/// run of 236.9 therefore rendered as `Rust-Score: 236.9/134` — 176.8% — into a
/// git commit trailer, where it is permanent and machine-read.
///
/// Taken from the orchestrator rather than re-hardcoded, so adding a scorer
/// cannot reintroduce the drift. Construction allocates the scorer set and
/// touches no I/O.
/// One `Rust-Score:` trailer line, or nothing when the score was not measured.
pub(super) fn rust_score_trailer(score: Option<f64>) -> String {
    score.map_or_else(String::new, |s| {
        format!(
            "Rust-Score: {s:.1}/{:.0}\n",
            crate::services::rust_project_score::rubric_max_points()
        )
    })
}

/// Capture rust project score (O(1) from cache).
///
/// `None` means NOT MEASURED. It used to return `Ok(0.0)` when the cache was
/// absent — and nothing in this repository writes
/// `.pmat-metrics/rust-project-score.json`: `grep -rn` finds two readers (here
/// and `work_falsification/cache.rs`), two test fixtures, and zero writers. So
/// on any real checkout every `pmat work complete` stamped
/// `Rust-Score: 0.0/134` into the commit message — a score nobody computed,
/// formatted as if it had been, and indistinguishable from a genuine zero.
pub(super) async fn capture_rust_project_score(project_path: &PathBuf) -> Result<Option<f64>> {
    let rust_file = project_path
        .join(".pmat-metrics")
        .join("rust-project-score.json");

    if !rust_file.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&rust_file)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    Ok(json.get("total_earned").and_then(serde_json::Value::as_f64))
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
    // A non-Rust project has no score, and a Rust project with no cached score
    // has no score either. Both are `None` — the trailer is then omitted rather
    // than asserting 0.0.
    let rust_score = if project_path.join("Cargo.toml").exists() {
        capture_rust_project_score(project_path)
            .await
            .unwrap_or(None)
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

    // Write to .pmat-metrics/ — created with its own ignore rule (#1070) so the
    // commit metadata record does not dirty the tree it describes.
    let metrics_dir = crate::utils::pmat_cache_dir::ensure_metrics_dir(project_path);

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
            Ok(()) => println!("{}", c::pass("Updated CHANGELOG.md")),
            Err(e) => {
                println!(
                    "{}",
                    c::warn(&format!("Failed to update CHANGELOG.md: {}", e))
                );
                println!("   {}", c::dim("You may need to update it manually"));
            }
        }
    } else {
        println!(
            "ℹ️  {}",
            c::dim("No changelog category inferred from labels")
        );
    }
}

/// Print completion next steps with commit metadata (helper for handle_work_complete)
pub(super) fn print_complete_next_steps(item: &RoadmapItem, id: &str, metadata: &CommitMetadata) {
    println!("{}", c::subheader("🎯 Next steps:"));
    let rust_score_line = metadata.rust_project_score;
    let rust_score_line = rust_score_trailer(rust_score_line);
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
        println!("{}", c::warn("Auto-commit: failed to stage files"));
        println!();
        print_complete_next_steps(item, id, metadata);
        return;
    }

    // Build commit message
    let rust_score_line = metadata.rust_project_score;
    let rust_score_line = rust_score_trailer(rust_score_line);
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
            println!("{}", c::pass("Auto-committed work completion files"));
            if item.is_github_synced() {
                println!(
                    "{} Next: gh issue close {}",
                    c::label("🎯"),
                    item.github_issue.expect("internal error")
                );
            }
            println!("{} Next: git push origin master", c::label("🎯"));
        }
        _ => {
            println!(
                "{}",
                c::warn("Auto-commit failed (nothing to commit or hook error)")
            );
            println!();
            print_complete_next_steps(item, id, metadata);
        }
    }
}

#[cfg(test)]
mod rust_score_trailer_tests {
    //! REGRESSION: the `Rust-Score` trailer carried a hardcoded `/134`
    //! denominator over a numerator drawn from the **289**-point rubric, and
    //! rendered an unmeasured score as `0.0`.
    //!
    //! Both are permanent: `pmat work complete` writes this into a git commit
    //! message, where it is machine-read by the agent instructions
    //! (`docs/agent-instructions/pmat-work-quality-principles.md`).
    //!
    //! The pre-existing tests in `src/tests/coverage_boost_work_core_handlers.rs`
    //! did not catch it because they rebuild the format string locally and
    //! assert against their own copy — they never call this module, so they
    //! would pass whatever the shipped code did.
    use super::*;

    /// The denominator must be the rubric's own total, not a literal.
    #[test]
    fn denominator_is_the_rubric_total_not_134() {
        let line = rust_score_trailer(Some(236.9));
        let max = crate::services::rust_project_score::rubric_max_points();
        assert!(
            (max - 289.0).abs() < f64::EPSILON,
            "rubric total moved to {max}; update this test deliberately, not by reflex"
        );
        assert_eq!(line, "Rust-Score: 236.9/289\n");
        assert!(
            !line.contains("/134"),
            "236.9/134 is 176.8% — a score above its own maximum: {line}"
        );
    }

    /// Not measured must print NOTHING, never `0.0`.
    ///
    /// Nothing in this repository writes `.pmat-metrics/rust-project-score.json`
    /// — two readers, two test fixtures, zero writers — so on a real checkout
    /// this was every commit.
    #[test]
    fn an_unmeasured_score_emits_no_trailer() {
        assert_eq!(
            rust_score_trailer(None),
            "",
            "an unmeasured score must not be rendered as a measurement"
        );
    }

    /// A genuine zero is still reportable, and must not be confused with the
    /// absence above.
    #[test]
    fn a_measured_zero_is_still_reported() {
        assert_eq!(rust_score_trailer(Some(0.0)), "Rust-Score: 0.0/289\n");
    }

    /// The cache reader returns None for a project with no cached score, rather
    /// than a fabricated 0.0.
    #[tokio::test]
    async fn missing_cache_reads_as_not_measured() {
        let dir = tempfile::tempdir().expect("tempdir");
        let got = capture_rust_project_score(&dir.path().to_path_buf())
            .await
            .expect("read");
        assert_eq!(got, None, "a missing cache is not a score of zero");
    }

    /// …and a present cache still reads.
    #[tokio::test]
    async fn present_cache_reads_the_total_earned() {
        let dir = tempfile::tempdir().expect("tempdir");
        let m = dir.path().join(".pmat-metrics");
        std::fs::create_dir_all(&m).expect("mkdir");
        std::fs::write(
            m.join("rust-project-score.json"),
            r#"{"total_earned": 236.9}"#,
        )
        .expect("write");
        let got = capture_rust_project_score(&dir.path().to_path_buf())
            .await
            .expect("read");
        assert_eq!(got, Some(236.9));
    }
}
