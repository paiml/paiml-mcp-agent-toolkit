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
        let found: Vec<crate::tdg::FullTdgRecord> = storage.get_by_commit(&commit_ref).await?;
        if found.is_empty() {
            return Err(anyhow!(
                "No TDG data found for commit '{}'. Ensure TDG was run with --with-git-context.",
                commit_ref
            ));
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
