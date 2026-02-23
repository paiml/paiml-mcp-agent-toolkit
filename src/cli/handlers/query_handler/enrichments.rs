//! Enrichment logic: churn, duplicates, entropy, faults, coverage, git-history.

use crate::services::agent_context::{
    enrich_results_with_churn, enrich_results_with_coverage, enrich_results_with_duplicates,
    enrich_results_with_entropy, enrich_results_with_faults, AgentContextIndex, QueryResult,
};
use crate::services::git_history::{
    CommitInfo, GitHistoryIndex, GitHistorySearchEngine, GitSearchOptions, GitSearchResult,
};
use std::path::PathBuf;
use std::time::Instant;

use super::git_history::GitHistoryProfile;
use super::options::GitData;

// ── Enrichment macros and functions ─────────────────────────────────────────

/// Apply all enrichments (churn, duplicates, entropy, faults, coverage, coverage-diff)
#[allow(clippy::too_many_arguments)]
macro_rules! try_enrich {
    ($results:expr, $quiet:expr, $label:expr, $call:expr) => {
        if !$results.is_empty() {
            if !$quiet {
                eprintln!($label);
            }
            if let Err(e) = $call {
                if !$quiet {
                    eprintln!("Warning: {e}");
                }
            }
        }
    };
}

async fn apply_churn(results: &mut Vec<QueryResult>, project_path: &std::path::Path, quiet: bool) {
    try_enrich!(
        results,
        quiet,
        "Computing git churn metrics...",
        enrich_results_with_churn(results, project_path, 90).await
    );
}

async fn apply_duplicates(
    results: &mut Vec<QueryResult>,
    project_path: &std::path::Path,
    quiet: bool,
) {
    try_enrich!(
        results,
        quiet,
        "Detecting code duplicates...",
        enrich_results_with_duplicates(results, project_path).await
    );
}

async fn apply_entropy(
    results: &mut Vec<QueryResult>,
    project_path: &std::path::Path,
    quiet: bool,
) {
    try_enrich!(
        results,
        quiet,
        "Computing pattern diversity...",
        enrich_results_with_entropy(results, project_path).await
    );
}

async fn apply_faults(results: &mut Vec<QueryResult>, project_path: &std::path::Path, quiet: bool) {
    try_enrich!(
        results,
        quiet,
        "Detecting fault patterns (batuta)...",
        enrich_results_with_faults(results, project_path).await
    );
}

async fn apply_coverage_enrichment(
    results: &mut Vec<QueryResult>,
    project_path: &std::path::Path,
    quiet: bool,
    coverage_file: &Option<PathBuf>,
    uncovered_only: bool,
) {
    let cov_path = coverage_file.as_deref();
    try_enrich!(
        results,
        quiet,
        "Loading coverage data...",
        enrich_results_with_coverage(results, project_path, cov_path).await
    );
    if uncovered_only {
        results.retain(|r| r.lines_total > 0 && r.line_coverage_pct < 100.0);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn apply_all_enrichments(
    results: &mut Vec<QueryResult>,
    project_path: &std::path::Path,
    quiet: bool,
    churn: bool,
    duplicates: bool,
    entropy: bool,
    faults: bool,
    coverage: bool,
    uncovered_only: bool,
    coverage_file: &Option<PathBuf>,
    coverage_diff: &Option<PathBuf>,
) {
    if churn {
        apply_churn(results, project_path, quiet).await;
    }
    if duplicates {
        apply_duplicates(results, project_path, quiet).await;
    }
    if entropy {
        apply_entropy(results, project_path, quiet).await;
    }
    if faults {
        apply_faults(results, project_path, quiet).await;
    }
    if coverage {
        apply_coverage_enrichment(results, project_path, quiet, coverage_file, uncovered_only)
            .await;
    }
    if let Some(ref diff_path) = coverage_diff {
        if coverage && !results.is_empty() {
            super::modes::apply_coverage_diff(results, project_path, diff_path, quiet);
        }
    }
}

// ── Git history search ──────────────────────────────────────────────────────

pub(super) fn fetch_git_data(
    git_history: bool,
    project_path: &std::path::Path,
    query: &str,
    limit: usize,
    index: &AgentContextIndex,
    quiet: bool,
) -> anyhow::Result<GitData> {
    if git_history {
        fetch_git_history_results(project_path, query, limit, index, quiet)
    } else {
        Ok(None)
    }
}

/// Fetch git history search results if requested
fn fetch_git_history_results(
    project_path: &std::path::Path,
    query: &str,
    limit: usize,
    index: &AgentContextIndex,
    quiet: bool,
) -> anyhow::Result<Option<(Vec<GitSearchResult>, Vec<CommitInfo>)>> {
    if !quiet {
        eprintln!("Searching git history...");
    }
    match search_git_history_profiled(project_path, query, limit, index, quiet) {
        Ok((git_hits, profile, all_commits)) => {
            if !quiet {
                eprintln!(
                    "Git history: {} commits in {}ms (log: {}ms, parse: {}ms, index: {}ms, search: {}ms, annotate: {}ms)",
                    profile.commit_count, profile.total_ms, profile.git_log_ms, profile.parse_ms,
                    profile.index_ms, profile.search_ms, profile.annotate_ms,
                );
                if !git_hits.is_empty() {
                    eprintln!("Found {} relevant commits", git_hits.len());
                }
            }
            Ok(Some((git_hits, all_commits)))
        }
        Err(e) => {
            if !quiet {
                eprintln!("Warning: Git history search failed: {}", e);
            }
            Ok(None)
        }
    }
}

/// Search git history with timing profile and O(1) annotations
/// Returns (search_results, profile, all_parsed_commits)
fn search_git_history_profiled(
    project_path: &std::path::Path,
    query: &str,
    limit: usize,
    _index: &AgentContextIndex,
    _quiet: bool,
) -> anyhow::Result<(Vec<GitSearchResult>, GitHistoryProfile, Vec<CommitInfo>)> {
    let total_start = Instant::now();

    // Phase 1: git log
    let git_start = Instant::now();
    let output = std::process::Command::new("git")
        .args([
            "log",
            "--format=PMAT_START%nH:%H%nS:%s%nN:%an%nE:%ae%nT:%at%nPMAT_FILES",
            "--name-status",
            "-500",
        ])
        .current_dir(project_path)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run git log: {}", e))?;
    let git_log_ms = git_start.elapsed().as_millis();

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git log failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Phase 2: parse
    let parse_start = Instant::now();
    let log_text = String::from_utf8_lossy(&output.stdout);
    let commits = super::git_history::parse_git_log(&log_text);
    let commit_count = commits.len();
    let parse_ms = parse_start.elapsed().as_millis();

    if commits.is_empty() {
        return Ok((
            vec![],
            GitHistoryProfile {
                git_log_ms,
                parse_ms,
                index_ms: 0,
                search_ms: 0,
                annotate_ms: 0,
                total_ms: total_start.elapsed().as_millis(),
                commit_count: 0,
            },
            vec![],
        ));
    }

    // Phase 3: index build
    let index_start = Instant::now();
    let mut git_index = GitHistoryIndex::in_memory()
        .map_err(|e| anyhow::anyhow!("Failed to create git history index: {}", e))?;
    git_index
        .insert_commits(&commits)
        .map_err(|e| anyhow::anyhow!("Failed to index commits: {}", e))?;
    let index_ms = index_start.elapsed().as_millis();

    // Phase 4: search
    let search_start = Instant::now();
    let mut engine = GitHistorySearchEngine::new(&git_index);
    let options = GitSearchOptions {
        limit,
        ..Default::default()
    };
    let results = engine
        .search(query, options)
        .map_err(|e| anyhow::anyhow!("Git history search failed: {}", e))?;
    let search_ms = search_start.elapsed().as_millis();

    // Phase 5: annotate -- deferred to formatting phase (no pre-warm needed)
    let annotate_ms = 0u128;

    let profile = GitHistoryProfile {
        git_log_ms,
        parse_ms,
        index_ms,
        search_ms,
        annotate_ms,
        total_ms: total_start.elapsed().as_millis(),
        commit_count,
    };

    Ok((results, profile, commits))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn merge_raw_results(
    is_regex_literal: bool,
    quiet: bool,
    query: &str,
    limit: usize,
    ctx: &super::options::MergeContext,
    context_lines: Option<usize>,
    after_context: Option<usize>,
    before_context: Option<usize>,
    results: &[QueryResult],
) -> Vec<crate::services::agent_context::RawSearchResult> {
    if !is_regex_literal {
        return Vec::new();
    }
    if !quiet {
        eprintln!("Searching raw files for non-indexed matches...");
    }
    super::modes::run_raw_search_for_merge(
        query,
        limit,
        ctx.literal,
        ctx.ignore_case,
        ctx.language,
        ctx.exclude_file,
        ctx.exclude,
        context_lines,
        after_context,
        before_context,
        ctx.project_path,
        results,
    )
}
