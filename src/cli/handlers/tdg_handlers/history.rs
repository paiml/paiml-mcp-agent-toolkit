#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG history commands: query, filter by git since/range
//!
//! Sprint 65 Phase 3: TDG History Commands for tracking quality over time.

use super::display::format_history_output;
use super::TdgCommandConfig;
use crate::cli::TdgOutputFormat;
use crate::tdg::TdgAnalyzer;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Handle TDG history subcommand (Sprint 65 Phase 3)
pub(super) async fn handle_history_command(
    analyzer: &TdgAnalyzer,
    commit: Option<String>,
    since: Option<String>,
    range: Option<String>,
    path_filter: Option<PathBuf>,
    format: TdgOutputFormat,
    config: &TdgCommandConfig,
) -> Result<()> {
    let storage = analyzer
        .storage()
        .ok_or_else(|| anyhow!("TDG storage not initialized. Run with --with-git-context flag."))?;

    let mut records = query_history_records(storage, commit, since, range, &config.path).await?;

    if let Some(target_path) = path_filter {
        records.retain(|r| r.identity.path == target_path);
    }

    if records.is_empty() {
        println!("No TDG history found matching criteria.");
        return Ok(());
    }

    let output_str = format_history_output(&records, format)?;
    match &config.output {
        Some(output_path) => std::fs::write(output_path, output_str)?,
        None => println!("{output_str}"),
    }

    Ok(())
}

/// Query TDG history records based on command flags
async fn query_history_records(
    storage: &crate::tdg::TieredStore,
    commit: Option<String>,
    since: Option<String>,
    range: Option<String>,
    repo_path: &Path,
) -> Result<Vec<crate::tdg::FullTdgRecord>> {
    if let Some(commit_ref) = commit {
        // `--commit` takes a ref, but the raw string used to go straight to
        // storage, which only string-matches the SHAs and tags recorded at
        // capture time. So `--commit HEAD`, a branch name, or a tag created
        // after the capture matched nothing — and the error blamed a missing
        // `--with-git-context` run that had just succeeded. Ask git what the
        // ref means first, exactly as the --since/--range paths already do.
        let resolved = resolve_commit_ref(&commit_ref, repo_path);
        let lookup = resolved.as_deref().unwrap_or(commit_ref.as_str());
        let mut found: Vec<crate::tdg::FullTdgRecord> = storage.get_by_commit(lookup).await?;
        if found.is_empty() && lookup != commit_ref.as_str() {
            // Records captured before the SHA existed may still be keyed by the
            // literal the user typed (e.g. a tag name).
            found = storage.get_by_commit(&commit_ref).await?;
        }
        if found.is_empty() {
            return Err(match resolved {
                Some(sha) => anyhow!(
                    "No TDG record for commit '{commit_ref}' (resolved to {sha}). \
That commit was never captured — run `pmat tdg --with-git-context` there."
                ),
                None => anyhow!(
                    "Could not resolve '{commit_ref}' to a commit in {}. \
Pass a SHA, branch, tag or ref that exists in this repository.",
                    repo_path.display()
                ),
            });
        }
        return Ok(found);
    }
    let all_records = storage.get_all_with_git_context().await?;
    if let Some(since_ref) = since {
        return filter_by_git_since(&since_ref, all_records, repo_path);
    }
    if let Some(range_ref) = range {
        return filter_by_git_range(&range_ref, all_records, repo_path);
    }
    Ok(all_records)
}

/// Resolve a git ref (`HEAD`, a branch, a tag, a short SHA) to its full commit
/// SHA in `repo_path`. Returns `None` when the ref does not name a commit here,
/// which is what distinguishes "you typed a ref I cannot resolve" from "that
/// commit exists but was never captured".
fn resolve_commit_ref(commit_ref: &str, repo_path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args([
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{commit_ref}^{{commit}}"),
        ])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if sha.is_empty() {
        None
    } else {
        Some(sha)
    }
}

/// Filter records by git "since" reference
fn filter_by_git_since(
    since_ref: &str,
    mut records: Vec<crate::tdg::storage::FullTdgRecord>,
    repo_path: &Path,
) -> Result<Vec<crate::tdg::storage::FullTdgRecord>> {
    // Get timestamp of the "since" commit using shell git
    let output = Command::new("git")
        .args(["log", "-1", "--format=%ct", since_ref])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to resolve git ref: {since_ref}"));
    }

    let since_time: i64 = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .map_err(|_| anyhow!("Invalid timestamp from git log"))?;

    // Filter records to commits after since_time
    records.retain(|r| {
        if let Some(git_ctx) = &r.git_context {
            let record_time = git_ctx.commit_timestamp.timestamp();
            record_time > since_time
        } else {
            false
        }
    });

    Ok(records)
}

/// Filter records by git commit range
fn filter_by_git_range(
    range_ref: &str,
    mut records: Vec<crate::tdg::storage::FullTdgRecord>,
    repo_path: &Path,
) -> Result<Vec<crate::tdg::storage::FullTdgRecord>> {
    // Parse range (e.g., "HEAD~10..HEAD" or "v2.177.0..v2.178.0")
    let parts: Vec<&str> = range_ref.split("..").collect();
    if parts.len() != 2 {
        return Err(anyhow!(
            "Invalid range format. Expected 'start..end' (e.g., HEAD~10..HEAD)"
        ));
    }

    // Get timestamps using shell git
    let get_timestamp = |git_ref: &str| -> Result<i64> {
        let output = Command::new("git")
            .args(["log", "-1", "--format=%ct", git_ref])
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to resolve git ref: {git_ref}"));
        }

        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .map_err(|_| anyhow!("Invalid timestamp from git log"))
    };

    let start_time = get_timestamp(parts[0])?;
    let end_time = get_timestamp(parts[1])?;

    // Filter records within time range
    records.retain(|r| {
        if let Some(git_ctx) = &r.git_context {
            let record_time = git_ctx.commit_timestamp.timestamp();
            record_time >= start_time && record_time <= end_time
        } else {
            false
        }
    });

    Ok(records)
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod commit_ref_resolution_tests {
    use super::*;
    use crate::models::git_context::GitContext;
    use crate::tdg::storage::{
        AnalysisMetadata, ComponentScores, FileIdentity, FullTdgRecord, SemanticSignature,
    };
    use crate::tdg::TieredStore;
    use chrono::Utc;
    use std::time::SystemTime;

    fn git(repo: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(repo)
            .output()
            .expect("git runs");
        assert!(
            status.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&status.stderr)
        );
    }

    fn one_commit_repo(repo: &Path) -> String {
        git(repo, &["init", "-q", "-b", "main"]);
        git(repo, &["config", "user.email", "t@example.com"]);
        git(repo, &["config", "user.name", "T"]);
        // The developer's globally configured hooks must not run in a fixture.
        git(repo, &["config", "core.hooksPath", "/dev/null"]);
        std::fs::write(repo.join("a.txt"), "hello").unwrap();
        git(repo, &["add", "."]);
        git(repo, &["commit", "-q", "--no-verify", "-m", "first"]);
        let out = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo)
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn record_at(sha: &str) -> FullTdgRecord {
        FullTdgRecord {
            identity: FileIdentity {
                path: std::path::PathBuf::from("a.rs"),
                content_hash: blake3::hash(b"a"),
                size_bytes: 1,
                modified_time: SystemTime::now(),
            },
            score: Default::default(),
            components: ComponentScores::default(),
            semantic_sig: SemanticSignature {
                ast_structure_hash: 1,
                identifier_pattern: String::new(),
                control_flow_pattern: String::new(),
                import_dependencies: Vec::new(),
            },
            metadata: AnalysisMetadata {
                analyzer_version: "test".to_string(),
                analysis_duration_ms: 1,
                language_confidence: 1.0,
                analysis_timestamp: SystemTime::now(),
                cache_hit: false,
            },
            git_context: Some(GitContext {
                commit_sha: sha.to_string(),
                commit_sha_short: sha[..7].to_string(),
                branch: "main".to_string(),
                author_name: "T".to_string(),
                author_email: "t@example.com".to_string(),
                commit_timestamp: Utc::now(),
                commit_message: "first".to_string(),
                tags: vec![],
                parent_commits: vec![],
                remote_url: None,
                is_clean: true,
                uncommitted_files: 0,
            }),
        }
    }

    /// `--commit HEAD` used to fail with "No TDG data found for commit 'HEAD'.
    /// Ensure TDG was run with --with-git-context" even when the run that stored
    /// the record had just succeeded, because the literal string was matched
    /// against the stored SHA instead of being resolved by git.
    #[tokio::test]
    async fn commit_ref_head_resolves_to_the_stored_sha() {
        let dir = tempfile::tempdir().unwrap();
        let sha = one_commit_repo(dir.path());

        let storage = TieredStore::in_memory();
        storage.store(record_at(&sha)).await.unwrap();

        let found =
            query_history_records(&storage, Some("HEAD".to_string()), None, None, dir.path())
                .await
                .expect("HEAD must resolve to the captured commit");
        assert_eq!(found.len(), 1);

        // A branch name must work for the same reason.
        let found =
            query_history_records(&storage, Some("main".to_string()), None, None, dir.path())
                .await
                .expect("a branch name must resolve too");
        assert_eq!(found.len(), 1);
    }

    /// A tag created after the capture names the same commit, so it must find
    /// the same record — it used to fail because only tags recorded at capture
    /// time were string-matched.
    #[tokio::test]
    async fn tag_created_after_capture_still_resolves() {
        let dir = tempfile::tempdir().unwrap();
        let sha = one_commit_repo(dir.path());

        let storage = TieredStore::in_memory();
        storage.store(record_at(&sha)).await.unwrap();

        git(dir.path(), &["tag", "v2.0.0"]);

        let found =
            query_history_records(&storage, Some("v2.0.0".to_string()), None, None, dir.path())
                .await
                .expect("a tag added after capture points at a captured commit");
        assert_eq!(found.len(), 1);
    }

    /// An unresolvable ref must say so, not blame a missing capture run.
    #[tokio::test]
    async fn unresolvable_ref_is_reported_as_such() {
        let dir = tempfile::tempdir().unwrap();
        one_commit_repo(dir.path());
        let storage = TieredStore::in_memory();

        let err = query_history_records(
            &storage,
            Some("no-such-ref".to_string()),
            None,
            None,
            dir.path(),
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(
            err.contains("Could not resolve"),
            "expected a resolution error, got: {err}"
        );
    }
}
