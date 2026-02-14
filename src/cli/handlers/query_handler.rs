//! Query Handler - Semantic code search for agents (PMAT-470)
//!
//! Provides RAG-powered code search with quality annotations.
//! Designed as a grep replacement for AI agents.

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::cli::QueryOutputFormat;
use crate::services::agent_context::{
    build_coverage_map, enrich_results_with_churn, enrich_results_with_coverage,
    enrich_results_with_duplicates, enrich_results_with_entropy, enrich_results_with_faults,
    enrich_with_coverage_diff, format_coverage_summary, format_json, format_markdown, format_text,
    format_text_with_code, is_within_indexed_function, raw_search, AgentContextIndex,
    CaseSensitivity, QueryOptions, RankBy, QueryResult, RawSearchOptions, RawSearchOutput,
    RawSearchResult, SearchMode,
};
use crate::services::git_history::{
    ChangeType, CommitInfo, FileChange, GitHistoryIndex, GitHistorySearchEngine, GitSearchOptions,
    GitSearchResult,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

// ── ANSI color constants ────────────────────────────────────────────────────

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const UNDERLINE: &str = "\x1b[4m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const WHITE: &str = "\x1b[1;37m";
const BRIGHT_GREEN: &str = "\x1b[1;32m";
const BRIGHT_RED: &str = "\x1b[1;31m";
const DIM_CYAN: &str = "\x1b[2;36m";

// ── Data structures ─────────────────────────────────────────────────────────

/// Query performance profile — Toyota Way Andon cord instrumentation.
///
/// All query phases are timed. If any phase exceeds its threshold,
/// a warning is printed (Andon cord: make problems visible).
struct QueryProfile {
    phases: Vec<(&'static str, std::time::Duration)>,
    start: Instant,
}

/// Andon cord threshold: any single phase exceeding this triggers a warning.
const ANDON_THRESHOLD_MS: u128 = 500;

impl QueryProfile {
    fn new() -> Self {
        Self { phases: Vec::new(), start: Instant::now() }
    }

    fn phase(&mut self, name: &'static str) {
        self.phases.push((name, self.start.elapsed()));
    }

    fn emit(&self, quiet: bool) {
        if quiet { return; }
        let total = self.start.elapsed();
        let mut prev = std::time::Duration::ZERO;
        let mut violations = Vec::new();
        for (name, cumulative) in &self.phases {
            let delta = *cumulative - prev;
            let delta_ms = delta.as_millis();
            if delta_ms > ANDON_THRESHOLD_MS {
                violations.push((*name, delta_ms));
            }
            prev = *cumulative;
        }
        if !violations.is_empty() {
            eprintln!("{DIM}query profile: {:.0}ms total{RESET}", total.as_secs_f64() * 1000.0);
            for (name, cumulative) in &self.phases {
                let delta = if self.phases.first().map(|f| f.0) == Some(*name) {
                    *cumulative
                } else {
                    let idx = self.phases.iter().position(|p| p.0 == *name).expect("phase must exist");
                    *cumulative - self.phases[idx - 1].1
                };
                let delta_ms = delta.as_millis();
                let marker = if delta_ms > ANDON_THRESHOLD_MS { &format!(" {BRIGHT_RED}ANDON{RESET}") } else { "" };
                eprintln!("  {DIM}{name}: {delta_ms}ms{marker}{RESET}");
            }
        }
    }
}

/// Timing breakdown for git history search phases
struct GitHistoryProfile {
    git_log_ms: u128,
    parse_ms: u128,
    index_ms: u128,
    search_ms: u128,
    annotate_ms: u128,
    total_ms: u128,
    commit_count: usize,
}

/// Quality annotations for a file referenced in git history
#[derive(Default, Clone)]
struct FileAnnotation {
    tdg_grade: Option<String>,
    avg_complexity: Option<f32>,
    max_pagerank: Option<f32>,
    function_count: usize,
    dead_code_count: usize,
    dead_code_pct: f32,
    fault_count: usize,
}

/// Aggregated hotspot info for a file across all commits
#[derive(Default, Clone)]
struct FileHotspot {
    commit_count: usize,
    fix_count: usize,
    feat_count: usize,
    lines_added: u64,
    lines_deleted: u64,
    authors: HashMap<String, usize>,
    annotation: FileAnnotation,
}

/// Co-change pair
struct CoChangePair {
    file_a: String,
    file_b: String,
    count: usize,
    jaccard: f32,
}

/// Per-commit enrichment (reserved for JSON output format)
#[allow(dead_code)]
struct CommitAnnotation {
    work_ticket: Option<WorkTicketInfo>,
    commit_quality: Option<CommitQualityMeta>,
    decay_score: f32,
    impact_risk: f32,
}

/// Work ticket cross-reference
struct WorkTicketInfo {
    ticket_id: String,
    claims_passed: usize,
    claims_total: usize,
    #[allow(dead_code)]
    baseline_tdg: f64,
}

/// Quality metadata from .pmat-metrics/commit-*-meta.json
#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct CommitQualityMeta {
    #[serde(default)]
    work_item_id: String,
    #[serde(default)]
    tdg_score: f64,
    #[serde(default)]
    repo_score: f64,
    #[serde(default)]
    rust_project_score: Option<f64>,
}

/// Dead code cache entry
#[derive(serde::Deserialize)]
struct DeadCodeCache {
    #[serde(default)]
    report: DeadCodeReport,
}

#[derive(serde::Deserialize, Default)]
struct DeadCodeReport {
    #[serde(default)]
    files_with_dead_code: Vec<DeadCodeFile>,
}

#[derive(serde::Deserialize)]
struct DeadCodeFile {
    file_path: String,
    #[serde(default)]
    dead_items: Vec<serde_json::Value>,
    #[serde(default)]
    file_dead_percentage: f32,
}

/// Bug hunter cache entry
#[derive(serde::Deserialize)]
struct BugHunterCache {
    #[serde(default)]
    findings: Vec<BugHunterFinding>,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct BugHunterFinding {
    #[serde(default)]
    file: String,
    #[serde(default)]
    severity: String,
    #[serde(default)]
    suspiciousness: f32,
}

/// Handle the `pmat query` command
///
/// # Arguments
/// * `query` - Natural language query
/// * `limit` - Maximum number of results
/// * `min_grade` - Minimum TDG grade filter
/// * `max_complexity` - Maximum complexity filter
/// * `language` - Language filter
/// * `path_pattern` - File path pattern filter
/// * `project_path` - Project root to search
/// * `format` - Output format
/// * `include_source` - Include full source code
/// * `rebuild_index` - Force rebuild index
/// * `rank_by` - Ranking strategy (relevance, pagerank, centrality, indegree)
/// * `min_pagerank` - Minimum PageRank score filter
/// * `include_project` - Additional project paths to include in search
/// * `churn` - Enrich results with git churn data (commit count, volatility)
/// * `duplicates` - Enrich results with duplicate code detection
/// * `entropy` - Enrich results with entropy/pattern diversity metrics
/// * `faults` - Enrich results with batuta fault pattern annotations
/// * `definition_type` - Filter by definition type (fn, struct, enum, trait, type)
/// * `code` - Show source code inline (default: true, use --summary to disable)
/// * `git_history` - Include git commit history in search via RRF fusion
#[allow(clippy::too_many_arguments)]
pub async fn handle_query(
    query: String,
    limit: usize,
    min_grade: Option<String>,
    max_complexity: Option<u32>,
    language: Option<String>,
    path_pattern: Option<String>,
    project_path: PathBuf,
    format: QueryOutputFormat,
    include_source: bool,
    rebuild_index: bool,
    exclude_tests: bool,
    rank_by: Option<String>,
    min_pagerank: Option<f32>,
    include_project: Vec<PathBuf>,
    churn: bool,
    duplicates: bool,
    entropy: bool,
    faults: bool,
    coverage: bool,
    uncovered_only: bool,
    coverage_diff: Option<PathBuf>,
    coverage_file: Option<PathBuf>,
    coverage_gaps: bool,
    include_excluded: bool,
    definition_type: Option<String>,
    code: bool,
    git_history: bool,
    regex: bool,
    literal: bool,
    raw: bool,
    case_sensitive: bool,
    ignore_case: bool,
    exclude: Option<String>,
    exclude_file: Option<String>,
    files_with_matches: bool,
    count: bool,
    after_context: Option<usize>,
    before_context: Option<usize>,
    context_lines: Option<usize>,
    ptx_flow: bool,
    ptx_diagnostics: bool,
) -> anyhow::Result<()> {
    let quiet = matches!(format, QueryOutputFormat::Json);
    let mut profile = QueryProfile::new();

    // ── Raw search mode: skip index entirely ──────
    if raw {
        return handle_raw_search_mode(
            &query, limit, &format, quiet, literal, ignore_case,
            &language, &exclude_file, &exclude, files_with_matches,
            count, context_lines, after_context, before_context, &project_path,
            exclude_tests,
        );
    }

    // ── Load index ──────
    let mut index = load_query_index(&project_path, rebuild_index, &include_project, quiet)?;
    profile.phase("load_index");

    let is_regex_or_literal = regex || literal;
    let is_ptx = ptx_flow || ptx_diagnostics;
    prepare_index_for_mode(&mut index, is_regex_or_literal, is_ptx, &rank_by);
    profile.phase("source_load");

    emit_index_stats(&index, quiet);

    // ── Coverage-gaps mode ──────
    if coverage_gaps {
        let siblings = collect_siblings(&project_path, &include_project);
        return handle_coverage_gaps_mode(
            &index, &project_path, &format, &coverage_file,
            &language, &path_pattern, exclude_tests, limit, quiet,
            include_excluded, files_with_matches, count, &siblings,
        ).await;
    }

    // ── PTX modes (flow / diagnostics) ──────
    if let Some(output) = handle_ptx_modes(ptx_flow, ptx_diagnostics, &index, &format) {
        print!("{output}");
        return Ok(());
    }

    // ── Execute semantic query + enrich + output ──────
    let effective_include_source = include_source || code || is_regex_or_literal;
    let merge_language = if is_regex_or_literal { language.clone() } else { None };
    let merge_exclude_file = if is_regex_or_literal { exclude_file.clone() } else { None };
    let merge_exclude = if is_regex_or_literal { exclude.clone() } else { None };

    let options = build_query_options(
        limit, min_grade, max_complexity, language, path_pattern,
        effective_include_source, &rank_by, min_pagerank,
        regex, literal, case_sensitive, ignore_case, exclude, exclude_file,
    );
    let mut results = index
        .query(&query, options)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    profile.phase("query");

    apply_result_filters(&mut results, exclude_tests, &definition_type);
    apply_all_enrichments(
        &mut results, &project_path, quiet,
        churn, duplicates, entropy, faults,
        coverage, uncovered_only, &coverage_file, &coverage_diff,
    ).await;
    profile.phase("enrich");
    apply_post_enrichment_sort(&mut results, &rank_by);

    let git_data = fetch_git_data(git_history, &project_path, &query, limit, &index, quiet)?;
    profile.phase("git_history");

    if !is_regex_or_literal {
        backfill_results_source(&mut results, &index);
    }

    let merge_ctx = MergeContext {
        query: &query, literal, ignore_case,
        language: &merge_language, exclude_file: &merge_exclude_file,
        exclude: &merge_exclude, project_path: &project_path,
        is_regex_or_literal,
    };
    let raw_results = merge_raw_results(
        is_regex_or_literal, quiet, &query, limit, &merge_ctx,
        context_lines, after_context, before_context, &results,
    );

    emit_query_output(
        &results, &raw_results, &git_data, &query,
        &format, effective_include_source, coverage,
        files_with_matches, count, context_lines, after_context, before_context,
        &merge_ctx, &project_path, &index,
    )?;
    profile.phase("output");
    profile.emit(quiet);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_query_options(
    limit: usize, min_grade: Option<String>, max_complexity: Option<u32>,
    language: Option<String>, path_pattern: Option<String>,
    include_source: bool, rank_by: &Option<String>, min_pagerank: Option<f32>,
    regex: bool, literal: bool, case_sensitive: bool, ignore_case: bool,
    exclude: Option<String>, exclude_file: Option<String>,
) -> QueryOptions {
    let rank_by_enum = rank_by.as_ref().map(|s| s.parse::<RankBy>().unwrap_or_default()).unwrap_or_default();
    let search_mode = if regex { SearchMode::Regex } else if literal { SearchMode::Literal } else { SearchMode::Semantic };
    let case_sensitivity = if case_sensitive { CaseSensitivity::Sensitive } else if ignore_case { CaseSensitivity::Insensitive } else { CaseSensitivity::Smart };
    QueryOptions {
        limit, min_grade, max_complexity, max_loc: None, language, path_pattern,
        include_source, rank_by: rank_by_enum, min_pagerank,
        search_mode, case_sensitivity,
        exclude_pattern: exclude, exclude_file_pattern: exclude_file,
    }
}

// ── handle_query extracted helpers ──────────────────────────────────────────

/// Print a single raw search match with surrounding context lines
/// Pre-load source and call graph into the index based on the query mode.
fn prepare_index_for_mode(
    index: &mut AgentContextIndex, is_regex_or_literal: bool, is_ptx: bool,
    rank_by: &Option<String>,
) {
    if is_regex_or_literal || is_ptx {
        index.load_all_source();
    }
    let needs_call_graph = is_ptx
        || rank_by.as_deref() == Some("cross-project")
        || rank_by.as_deref() == Some("crossproject")
        || rank_by.as_deref() == Some("xproject");
    if needs_call_graph {
        index.ensure_call_graph();
    }
}

/// Print index stats to stderr (unless in quiet/JSON mode).
fn emit_index_stats(index: &AgentContextIndex, quiet: bool) {
    if !quiet {
        let manifest = index.manifest();
        eprintln!(
            "Index: {} functions in {} files (avg TDG: {:.1})",
            manifest.function_count, manifest.file_count, manifest.avg_tdg_score
        );
    }
}

/// Collect sibling project indexes for workspace coverage merging.
fn collect_siblings(project_path: &std::path::Path, include_project: &[PathBuf]) -> Vec<(PathBuf, String)> {
    let mut siblings = AgentContextIndex::discover_sibling_indexes(project_path);
    for project in include_project {
        let idx_path = project.join(".pmat/context.idx");
        let name = project.file_name().map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| project.display().to_string());
        if !siblings.iter().any(|(_, n)| n == &name) {
            siblings.push((idx_path, name));
        }
    }
    siblings
}

fn print_raw_match_context(
    file_path: &str, line_number: usize, line_content: &str,
    context_before: &[String], context_after: &[String],
) {
    if !context_before.is_empty() {
        let start_line = line_number - context_before.len();
        for (i, line) in context_before.iter().enumerate() {
            println!("{DIM}{}{RESET}:{DIM}{}{RESET}-{}", file_path, start_line + i, line);
        }
    }
    println!("{BOLD}{CYAN}{}{RESET}:{YELLOW}{}{RESET}:{}", file_path, line_number, line_content);
    if !context_after.is_empty() {
        for (i, line) in context_after.iter().enumerate() {
            println!("{DIM}{}{RESET}:{DIM}{}{RESET}-{}", file_path, line_number + 1 + i, line);
        }
    }
}

/// Handle `--raw` mode: pure file-level search without the function index
#[allow(clippy::too_many_arguments)]
fn handle_raw_search_mode(
    query: &str, limit: usize, format: &QueryOutputFormat, quiet: bool,
    literal: bool, ignore_case: bool, language: &Option<String>,
    exclude_file: &Option<String>, exclude: &Option<String>,
    files_with_matches: bool, count: bool,
    context_lines: Option<usize>, after_context: Option<usize>,
    before_context: Option<usize>, project_path: &std::path::Path,
    exclude_tests: bool,
) -> anyhow::Result<()> {
    let ctx_after = context_lines.or(after_context).unwrap_or(0);
    let ctx_before = context_lines.or(before_context).unwrap_or(0);
    // When --exclude-tests is set in raw mode, filter test file paths
    let effective_exclude_file = if exclude_tests && exclude_file.is_none() {
        Some("test".to_string())
    } else {
        None
    };
    let excl_file_ref = effective_exclude_file.as_deref().or(exclude_file.as_deref());
    let raw_opts = RawSearchOptions {
        pattern: query, literal, case_insensitive: ignore_case,
        before_context: ctx_before, after_context: ctx_after, limit,
        language_filter: language.as_deref(),
        exclude_file_pattern: excl_file_ref,
        exclude_pattern: exclude.as_deref(),
        files_with_matches, count_mode: count,
    };
    let output = raw_search(project_path, &raw_opts).map_err(|e| anyhow::anyhow!("{}", e))?;
    match output {
        RawSearchOutput::Files(files) => { for f in &files { println!("{CYAN}{}{RESET}", f); } }
        RawSearchOutput::Counts(counts) => { for c in &counts { println!("{CYAN}{}{RESET}:{YELLOW}{}{RESET}", c.file_path, c.count); } }
        RawSearchOutput::Lines(lines) => {
            if matches!(format, QueryOutputFormat::Json) {
                let json = serde_json::to_string_pretty(&lines).map_err(|e| anyhow::anyhow!("{}", e))?;
                println!("{}", json);
            } else {
                for r in &lines {
                    print_raw_match_context(&r.file_path, r.line_number, &r.line_content, &r.context_before, &r.context_after);
                }
            }
            if !quiet { eprintln!("{} matches", lines.len()); }
        }
    }
    Ok(())
}

/// Run raw search and return non-overlapping results for merge with index results.
/// Used when `--regex` or `--literal` is active (without `--raw`).
#[allow(clippy::too_many_arguments)]
fn run_raw_search_for_merge(
    query: &str, limit: usize, literal: bool, ignore_case: bool,
    language: &Option<String>, exclude_file: &Option<String>,
    exclude: &Option<String>, context_lines: Option<usize>,
    after_context: Option<usize>, before_context: Option<usize>,
    project_path: &std::path::Path, indexed_results: &[QueryResult],
) -> Vec<RawSearchResult> {
    let remaining = limit.saturating_sub(indexed_results.len());
    if remaining == 0 {
        return Vec::new();
    }

    let ctx_after = context_lines.or(after_context).unwrap_or(0);
    let ctx_before = context_lines.or(before_context).unwrap_or(0);
    let raw_opts = RawSearchOptions {
        pattern: query, literal, case_insensitive: ignore_case,
        before_context: ctx_before, after_context: ctx_after,
        limit: remaining + indexed_results.len(), // over-fetch to account for dedup
        language_filter: language.as_deref(),
        exclude_file_pattern: exclude_file.as_deref(),
        exclude_pattern: exclude.as_deref(),
        files_with_matches: false, count_mode: false,
    };

    let output = match raw_search(project_path, &raw_opts) {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let lines = match output {
        RawSearchOutput::Lines(l) => l,
        _ => return Vec::new(),
    };

    // Filter out matches that overlap with indexed function results
    lines.into_iter()
        .filter(|r| !is_within_indexed_function(&r.file_path, r.line_number, indexed_results))
        .take(remaining)
        .collect()
}

/// Run raw search and return file paths for merge with --files-with-matches mode.
#[allow(clippy::too_many_arguments)]
fn run_raw_files_for_merge(
    query: &str, literal: bool, ignore_case: bool,
    language: &Option<String>, exclude_file: &Option<String>,
    exclude: &Option<String>, project_path: &std::path::Path,
) -> Vec<String> {
    let raw_opts = RawSearchOptions {
        pattern: query, literal, case_insensitive: ignore_case,
        before_context: 0, after_context: 0, limit: 0,
        language_filter: language.as_deref(),
        exclude_file_pattern: exclude_file.as_deref(),
        exclude_pattern: exclude.as_deref(),
        files_with_matches: true, count_mode: false,
    };
    match raw_search(project_path, &raw_opts) {
        Ok(RawSearchOutput::Files(f)) => f,
        _ => Vec::new(),
    }
}

/// Run raw search and return per-file counts for merge with --count mode.
#[allow(clippy::too_many_arguments)]
fn run_raw_counts_for_merge(
    query: &str, literal: bool, ignore_case: bool,
    language: &Option<String>, exclude_file: &Option<String>,
    exclude: &Option<String>, project_path: &std::path::Path,
) -> Vec<crate::services::agent_context::FileMatchCount> {
    let raw_opts = RawSearchOptions {
        pattern: query, literal, case_insensitive: ignore_case,
        before_context: 0, after_context: 0, limit: 0,
        language_filter: language.as_deref(),
        exclude_file_pattern: exclude_file.as_deref(),
        exclude_pattern: exclude.as_deref(),
        files_with_matches: false, count_mode: true,
    };
    match raw_search(project_path, &raw_opts) {
        Ok(RawSearchOutput::Counts(c)) => c,
        _ => Vec::new(),
    }
}

/// Load the function index with workspace support
fn load_query_index(
    project_path: &PathBuf, rebuild_index: bool, include_project: &[PathBuf], quiet: bool,
) -> anyhow::Result<AgentContextIndex> {
    let index_path = project_path.join(".pmat/context.idx");
    let workspace_idx = project_path.join(".pmat/workspace.idx");
    let mut siblings = AgentContextIndex::discover_sibling_indexes(project_path);

    for project in include_project {
        let idx_path = project.join(".pmat/context.idx");
        if idx_path.exists() {
            let name = project.file_name().map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| project.display().to_string());
            if !siblings.iter().any(|(_, n)| n == &name) {
                siblings.push((idx_path, name));
            }
        } else if !quiet {
            eprintln!("Warning: No index at {:?}, run 'pmat query --rebuild-index' in that project first", idx_path);
        }
    }

    if !siblings.is_empty() && !rebuild_index && is_workspace_cache_fresh(&workspace_idx, &siblings, &index_path) {
        if !quiet { eprintln!("Loading cached workspace index..."); }
        if let Ok(cached) = AgentContextIndex::load(&workspace_idx) {
            return Ok(cached);
        }
    }

    load_and_merge_index(project_path, &index_path, &workspace_idx, &siblings, rebuild_index, quiet)
}

/// Backfill source code for query results from SQLite.
///
/// In deferred-source mode, `QueryResult.source` is `Some("")` (empty) for
/// semantic queries. This fetches source on-demand for the top N results
/// that need it for display (--include-source, --code, context lines).
fn backfill_results_source(results: &mut [QueryResult], index: &AgentContextIndex) {
    if index.db_path().is_none() {
        return; // Blob-loaded index already has source
    }
    for r in results.iter_mut() {
        // Skip results that already have non-empty source
        if r.source.as_ref().is_some_and(|s| !s.is_empty()) {
            continue;
        }
        // Only backfill if source was requested (Some(""))
        if r.source.is_none() {
            continue;
        }
        let src = index.load_source_for(&r.file_path, r.start_line);
        if !src.is_empty() {
            r.source = Some(src);
        }
    }
}

/// Check if a result looks like a test function
fn is_test_function(r: &QueryResult) -> bool {
    r.function_name.starts_with("test_")
        || r.file_path.starts_with("tests/")
        || r.file_path.contains("/tests/")
        || r.file_path.contains("_tests.")
        || r.file_path.contains("_test.")
}

/// Normalize a definition type filter string to the canonical form
fn normalize_definition_type(def_type: &str) -> String {
    match def_type.to_lowercase().as_str() {
        "fn" | "func" | "function" => "function".to_string(),
        "struct" | "structs" => "struct".to_string(),
        "enum" | "enums" => "enum".to_string(),
        "trait" | "traits" => "trait".to_string(),
        "type" | "types" | "typealias" => "typealias".to_string(),
        other => other.to_string(),
    }
}

/// Apply result filters: exclude-tests and definition-type
fn apply_result_filters(results: &mut Vec<QueryResult>, exclude_tests: bool, definition_type: &Option<String>) {
    if exclude_tests { results.retain(|r| !is_test_function(r)); }
    if let Some(ref def_type) = definition_type {
        let filter_type = normalize_definition_type(def_type);
        results.retain(|r| r.definition_type == filter_type);
    }
}

/// Apply filters for coverage-gaps mode (language, path, exclude-tests)
fn apply_result_filters_coverage(
    results: &mut Vec<QueryResult>, language: &Option<String>,
    path_pattern: &Option<String>, exclude_tests: bool,
) {
    if let Some(ref lang) = language {
        let lang_lower = lang.to_lowercase();
        results.retain(|r| r.language.to_lowercase() == lang_lower);
    }
    if let Some(ref pattern) = path_pattern {
        results.retain(|r| r.file_path.contains(pattern));
    }
    if exclude_tests { results.retain(|r| !is_test_function(r)); }
}

/// Format and print coverage gap results in text mode (testable gaps only)
fn print_coverage_gaps_text(results: &[QueryResult]) {
    println!("{BOLD}{UNDERLINE}Coverage Gaps{RESET} ({} testable functions with uncovered code)\n", results.len());
    for (i, r) in results.iter().enumerate() {
        let pct_color = if r.line_coverage_pct < 50.0 { BRIGHT_RED } else if r.line_coverage_pct < 80.0 { YELLOW } else { GREEN };
        let impact_str = if r.impact_score > 1.0 { format!(" {YELLOW}impact:{:.1}{RESET}", r.impact_score) } else { String::new() };
        println!(
            "  {DIM}{:>3}.{RESET} {BRIGHT_RED}{:>4} uncov{RESET} | {pct_color}{:>5.1}% cov{RESET} | {CYAN}{}{RESET}:{YELLOW}{}{RESET} {WHITE}{}{RESET} {DIM}[{}]{RESET}{impact_str}",
            i + 1, r.missed_lines, r.line_coverage_pct, r.file_path, r.start_line, r.function_name, r.tdg_grade,
        );
    }
    println!();
}

/// Print the excluded summary footer
fn print_exclusion_summary(summary: &crate::services::agent_context::ExclusionSummary) {
    println!("{DIM}Excluded from coverage (not shown):{RESET}");
    if summary.coverage_off_count > 0 {
        println!("  {DIM}coverage(off): {} functions across {} files{RESET}", summary.coverage_off_count, summary.coverage_off_files);
    }
    if summary.dead_code_count > 0 {
        println!("  {DIM}dead code: {} functions across {} files{RESET}", summary.dead_code_count, summary.dead_code_files);
    }
    if summary.makefile_count > 0 {
        println!("  {DIM}Makefile COVERAGE_EXCLUDE: {} functions across {} files{RESET}", summary.makefile_count, summary.makefile_files);
    }
    println!("  {DIM}(use --include-excluded to see these){RESET}");
    println!();
}

/// Print excluded results grouped by category
fn print_excluded_results(excluded: &[&QueryResult]) {
    use crate::services::agent_context::CoverageExclusion;

    let groups: &[(CoverageExclusion, &str)] = &[
        (CoverageExclusion::CoverageOff, "coverage(off)"),
        (CoverageExclusion::DeadCode, "dead code"),
        (CoverageExclusion::MakefileExcluded, "Makefile pattern"),
    ];

    for (kind, label) in groups {
        let in_group: Vec<&&QueryResult> = excluded.iter()
            .filter(|r| r.coverage_exclusion == *kind)
            .collect();
        if in_group.is_empty() { continue; }

        println!("  {DIM}[EXCLUDED: {label}]{RESET} ({} functions)", in_group.len());
        for (i, r) in in_group.iter().enumerate().take(10) {
            println!(
                "    {DIM}{:>3}.{RESET} {DIM}{:>4} uncov{RESET} | {DIM}{:>5.1}% cov{RESET} | {DIM}{}{RESET}:{DIM}{}{RESET} {DIM}{}{RESET} {DIM}[{}]{RESET}",
                i + 1, r.missed_lines, r.line_coverage_pct, r.file_path, r.start_line, r.function_name, r.tdg_grade,
            );
        }
        if in_group.len() > 10 {
            println!("    {DIM}(+{} more){RESET}", in_group.len() - 10);
        }
    }
    println!();
}

/// Output coverage gaps aggregated by file (for --files-with-matches / --count).
///
/// `--files-with-matches`: prints file paths sorted by total uncovered lines desc.
/// `--count`: prints `file_path: N uncovered lines (M functions)` sorted desc.
fn output_coverage_gaps_by_file(results: &[QueryResult], files_only: bool) -> anyhow::Result<()> {
    use std::collections::BTreeMap;
    let mut by_file: BTreeMap<&str, (usize, usize)> = BTreeMap::new(); // (uncov_lines, func_count)
    for r in results {
        let entry = by_file.entry(&r.file_path).or_insert((0, 0));
        entry.0 += r.missed_lines as usize;
        entry.1 += 1;
    }
    let mut sorted: Vec<_> = by_file.into_iter().collect();
    sorted.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
    for (file, (uncov, funcs)) in &sorted {
        if files_only {
            println!("{file}");
        } else {
            println!("{file}: {uncov} uncovered lines ({funcs} functions)");
        }
    }
    Ok(())
}

/// Output coverage gap results in the requested format
fn output_coverage_gaps(
    format: &QueryOutputFormat, testable: Vec<QueryResult>, excluded: Vec<QueryResult>,
    include_excluded: bool,
) -> anyhow::Result<()> {
    let excluded_refs: Vec<&QueryResult> = excluded.iter().collect();
    let excl_summary = crate::services::agent_context::ExclusionSummary::from_results(&excluded_refs);

    match format {
        QueryOutputFormat::Json | QueryOutputFormat::Markdown => {
            let mut all = testable;
            if include_excluded { all.extend(excluded); }
            if matches!(format, QueryOutputFormat::Json) {
                println!("{}", format_json(&all).map_err(|e| anyhow::anyhow!("{}", e))?);
            } else {
                println!("{}", format_markdown(&all));
            }
        }
        _ => {
            print_coverage_gaps_text_with_exclusions(&testable, &excluded_refs, &excl_summary, include_excluded);
            if let Some(summary) = format_coverage_summary(&testable) { eprintln!("{DIM}{}{RESET}", summary); }
        }
    }
    Ok(())
}

/// Print text-mode coverage gaps with exclusion handling
fn print_coverage_gaps_text_with_exclusions(
    testable: &[QueryResult], excluded: &[&QueryResult],
    summary: &crate::services::agent_context::ExclusionSummary, include_excluded: bool,
) {
    if include_excluded && !excluded.is_empty() {
        println!("{BOLD}{UNDERLINE}Coverage Gaps{RESET} ({} testable + {} excluded)\n",
            testable.len(), summary.total());
        if !testable.is_empty() {
            println!("  {BOLD}[TESTABLE]{RESET}");
            print_coverage_gaps_text(testable);
        }
        print_excluded_results(excluded);
    } else {
        print_coverage_gaps_text(testable);
        if !summary.is_empty() { print_exclusion_summary(summary); }
    }
}

/// Handle `--coverage-gaps` mode: rank all functions by uncovered lines,
/// classifying exclusions to filter out coverage(off), dead code, and Makefile patterns.
#[allow(clippy::too_many_arguments)]
async fn handle_coverage_gaps_mode(
    index: &AgentContextIndex, project_path: &std::path::Path,
    format: &QueryOutputFormat, coverage_file: &Option<PathBuf>,
    language: &Option<String>, path_pattern: &Option<String>,
    exclude_tests: bool, limit: usize, quiet: bool,
    include_excluded: bool, files_with_matches: bool, count_mode: bool,
    siblings: &[(PathBuf, String)],
) -> anyhow::Result<()> {
    let mut profile = QueryProfile::new();

    // Lightweight: graph metrics only, skip call graph (not displayed in coverage-gaps)
    let mut results: Vec<QueryResult> = index.functions.iter().enumerate()
        .map(|(i, entry)| QueryResult::from_entry_with_metrics(entry, i, &index.graph_metrics, 0.0))
        .collect();
    profile.phase("build_results");

    apply_result_filters_coverage(&mut results, language, path_pattern, exclude_tests);
    profile.phase("filter");

    // Use cached coverage_off_files from index for O(1) lookup (no file I/O).
    // When db_path is Some, the field was populated from SQLite (even if empty = no files have coverage(off)).
    // When db_path is None (legacy blob), only trust the field if non-empty.
    let cached_cov_off = if index.db_path.is_some() || !index.coverage_off_files.is_empty() {
        Some(&index.coverage_off_files)
    } else {
        None
    };
    if !quiet { eprintln!("Classifying coverage exclusions ({} results)...", results.len()); }
    crate::services::agent_context::classify_exclusions(&mut results, project_path, cached_cov_off);
    profile.phase("classify_exclusions");

    if !quiet { eprintln!("Loading coverage data..."); }
    let cov_path = coverage_file.as_deref();
    if let Err(e) = enrich_results_with_coverage(&mut results, project_path, cov_path).await {
        eprintln!("Error: {}", e);
        return Ok(());
    }

    // Merge sibling coverage caches for workspace-level coverage gaps
    if !siblings.is_empty() {
        let workspace_cov = crate::services::agent_context::load_workspace_coverage(siblings);
        if !workspace_cov.is_empty() {
            if !quiet {
                eprintln!("Merging coverage from {} sibling(s) ({} files)", siblings.len(), workspace_cov.len());
            }
            crate::services::agent_context::enrich_with_coverage(&mut results, &workspace_cov);
        }
    }
    profile.phase("enrich_coverage");

    results.retain(|r| r.lines_total > 0 && r.line_coverage_pct < 100.0);

    let (mut testable, excluded): (Vec<QueryResult>, Vec<QueryResult>) =
        results.into_iter().partition(|r| !r.coverage_excluded);

    testable.sort_by(|a, b| b.missed_lines.cmp(&a.missed_lines)
        .then_with(|| a.line_coverage_pct.partial_cmp(&b.line_coverage_pct).unwrap_or(std::cmp::Ordering::Equal)));
    testable.truncate(limit);

    if testable.is_empty() && excluded.is_empty() {
        eprintln!("No coverage gaps found (100% coverage or no data).");
        return Ok(());
    }

    profile.phase("sort_partition");

    // ── File-level aggregation modes ──────
    if files_with_matches || count_mode {
        let r = output_coverage_gaps_by_file(&testable, files_with_matches);
        profile.phase("output");
        profile.emit(quiet);
        return r;
    }

    let r = output_coverage_gaps(format, testable, excluded, include_excluded);
    profile.phase("output");
    profile.emit(quiet);
    r
}

/// Apply all enrichments (churn, duplicates, entropy, faults, coverage, coverage-diff)
#[allow(clippy::too_many_arguments)]
macro_rules! try_enrich {
    ($results:expr, $quiet:expr, $label:expr, $call:expr) => {
        if !$results.is_empty() {
            if !$quiet { eprintln!($label); }
            if let Err(e) = $call {
                if !$quiet { eprintln!("Warning: {e}"); }
            }
        }
    };
}

async fn apply_churn(results: &mut Vec<QueryResult>, project_path: &std::path::Path, quiet: bool) {
    try_enrich!(results, quiet, "Computing git churn metrics...",
        enrich_results_with_churn(results, project_path, 90).await);
}

async fn apply_duplicates(results: &mut Vec<QueryResult>, project_path: &std::path::Path, quiet: bool) {
    try_enrich!(results, quiet, "Detecting code duplicates...",
        enrich_results_with_duplicates(results, project_path).await);
}

async fn apply_entropy(results: &mut Vec<QueryResult>, project_path: &std::path::Path, quiet: bool) {
    try_enrich!(results, quiet, "Computing pattern diversity...",
        enrich_results_with_entropy(results, project_path).await);
}

async fn apply_faults(results: &mut Vec<QueryResult>, project_path: &std::path::Path, quiet: bool) {
    try_enrich!(results, quiet, "Detecting fault patterns (batuta)...",
        enrich_results_with_faults(results, project_path).await);
}

async fn apply_coverage_enrichment(
    results: &mut Vec<QueryResult>, project_path: &std::path::Path, quiet: bool,
    coverage_file: &Option<PathBuf>, uncovered_only: bool,
) {
    let cov_path = coverage_file.as_deref();
    try_enrich!(results, quiet, "Loading coverage data...",
        enrich_results_with_coverage(results, project_path, cov_path).await);
    if uncovered_only { results.retain(|r| r.lines_total > 0 && r.line_coverage_pct < 100.0); }
}

#[allow(clippy::too_many_arguments)]
async fn apply_all_enrichments(
    results: &mut Vec<QueryResult>, project_path: &std::path::Path, quiet: bool,
    churn: bool, duplicates: bool, entropy: bool, faults: bool,
    coverage: bool, uncovered_only: bool,
    coverage_file: &Option<PathBuf>, coverage_diff: &Option<PathBuf>,
) {
    if churn { apply_churn(results, project_path, quiet).await; }
    if duplicates { apply_duplicates(results, project_path, quiet).await; }
    if entropy { apply_entropy(results, project_path, quiet).await; }
    if faults { apply_faults(results, project_path, quiet).await; }
    if coverage { apply_coverage_enrichment(results, project_path, quiet, coverage_file, uncovered_only).await; }
    if let Some(ref diff_path) = coverage_diff {
        if coverage && !results.is_empty() {
            apply_coverage_diff(results, project_path, diff_path, quiet);
        }
    }
}

/// Apply coverage diff enrichment from a baseline file
fn apply_coverage_diff(results: &mut [QueryResult], project_path: &std::path::Path, diff_path: &std::path::Path, quiet: bool) {
    match std::fs::read_to_string(diff_path) {
        Ok(json) => match build_coverage_map(&json, project_path) {
            Ok(baseline) => { enrich_with_coverage_diff(results, &baseline); }
            Err(e) => { if !quiet { eprintln!("Warning: Could not parse coverage baseline: {}", e); } }
        },
        Err(e) => { if !quiet { eprintln!("Warning: Could not read coverage baseline {}: {}", diff_path.display(), e); } }
    }
}

/// Fetch git history search results if requested
fn fetch_git_history_results(
    project_path: &std::path::Path, query: &str, limit: usize,
    index: &AgentContextIndex, quiet: bool,
) -> anyhow::Result<Option<(Vec<GitSearchResult>, Vec<CommitInfo>)>> {
    if !quiet { eprintln!("Searching git history..."); }
    match search_git_history_profiled(project_path, query, limit, index, quiet) {
        Ok((git_hits, profile, all_commits)) => {
            if !quiet {
                eprintln!(
                    "Git history: {} commits in {}ms (log: {}ms, parse: {}ms, index: {}ms, search: {}ms, annotate: {}ms)",
                    profile.commit_count, profile.total_ms, profile.git_log_ms, profile.parse_ms,
                    profile.index_ms, profile.search_ms, profile.annotate_ms,
                );
                if !git_hits.is_empty() { eprintln!("Found {} relevant commits", git_hits.len()); }
            }
            Ok(Some((git_hits, all_commits)))
        }
        Err(e) => {
            if !quiet { eprintln!("Warning: Git history search failed: {}", e); }
            Ok(None)
        }
    }
}

/// Apply post-enrichment re-sort for Impact ranking
fn apply_post_enrichment_sort(results: &mut [QueryResult], rank_by: &Option<String>) {
    if let Some(ref rank_str) = rank_by {
        let r = rank_str.to_lowercase();
        if r == "impact" || r == "roi" || r == "coverage" {
            results.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap_or(std::cmp::Ordering::Equal));
        } else if r == "cross-project" || r == "crossproject" || r == "xproject" {
            // Secondary sort: boost by cross_project_callers (already set by engine)
            results.sort_by(|a, b| {
                let score_a = a.pagerank * (1.0 + 0.5 * a.cross_project_callers as f32);
                let score_b = b.pagerank * (1.0 + 0.5 * b.cross_project_callers as f32);
                score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }
}

/// Handle special output modes with merged raw results for regex/literal modes.
/// Context for raw+indexed merge operations.
struct MergeContext<'a> {
    query: &'a str,
    literal: bool,
    ignore_case: bool,
    language: &'a Option<String>,
    exclude_file: &'a Option<String>,
    exclude: &'a Option<String>,
    project_path: &'a std::path::Path,
    is_regex_or_literal: bool,
}

type GitData = Option<(Vec<GitSearchResult>, Vec<CommitInfo>)>;

fn fetch_git_data(
    git_history: bool, project_path: &std::path::Path,
    query: &str, limit: usize,
    index: &AgentContextIndex, quiet: bool,
) -> anyhow::Result<GitData> {
    if git_history {
        fetch_git_history_results(project_path, query, limit, index, quiet)
    } else {
        Ok(None)
    }
}

#[allow(clippy::too_many_arguments)]
fn merge_raw_results(
    is_regex_literal: bool, quiet: bool, query: &str, limit: usize,
    ctx: &MergeContext, context_lines: Option<usize>,
    after_context: Option<usize>, before_context: Option<usize>,
    results: &[QueryResult],
) -> Vec<RawSearchResult> {
    if !is_regex_literal { return Vec::new(); }
    if !quiet { eprintln!("Searching raw files for non-indexed matches..."); }
    run_raw_search_for_merge(
        query, limit, ctx.literal, ctx.ignore_case,
        ctx.language, ctx.exclude_file, ctx.exclude,
        context_lines, after_context, before_context,
        ctx.project_path, results,
    )
}

#[allow(clippy::too_many_arguments)]
fn emit_query_output(
    results: &[QueryResult], raw_results: &[RawSearchResult],
    git_data: &GitData, query: &str,
    format: &QueryOutputFormat, include_source: bool, coverage: bool,
    files_with_matches: bool, count: bool,
    context_lines: Option<usize>, after_context: Option<usize>, before_context: Option<usize>,
    merge_ctx: &MergeContext,
    project_path: &std::path::Path,
    index: &AgentContextIndex,
) -> anyhow::Result<()> {
    if results.is_empty() && raw_results.is_empty()
        && git_data.as_ref().map_or(true, |(hits, _)| hits.is_empty())
    {
        eprintln!("No matching functions found for: {}", query);
        return Ok(());
    }

    if try_special_output_modes_merged(
        results, raw_results, files_with_matches, count,
        context_lines, after_context, before_context, merge_ctx,
    )? {
        return Ok(());
    }

    let highlight = if merge_ctx.is_regex_or_literal { Some((query, merge_ctx.literal)) } else { None };
    print_query_output(results, format, include_source, coverage, git_data, project_path, index, highlight);
    print_raw_results(raw_results, format);
    Ok(())
}

/// Print raw file matches (non-indexed).
fn print_raw_results(raw_results: &[RawSearchResult], format: &QueryOutputFormat) {
    if raw_results.is_empty() { return; }
    if matches!(format, QueryOutputFormat::Json) {
        let json = serde_json::to_string_pretty(&raw_results).unwrap_or_default();
        eprintln!("\n{{\"raw_matches\": {}}}", json);
    } else {
        eprintln!("\n{DIM}── Raw file matches ({} non-indexed) ──{RESET}", raw_results.len());
        for r in raw_results {
            print_raw_match_context(&r.file_path, r.line_number, &r.line_content,
                &r.context_before, &r.context_after);
        }
    }
}

/// Returns Ok(true) if handled, Ok(false) for standard output.
#[allow(clippy::too_many_arguments)]
fn try_special_output_modes_merged(
    results: &[QueryResult], raw_results: &[RawSearchResult],
    files_with_matches: bool, count: bool,
    context_lines: Option<usize>, after_context: Option<usize>, before_context: Option<usize>,
    ctx: &MergeContext,
) -> anyhow::Result<bool> {
    if files_with_matches {
        return handle_files_with_matches(results, raw_results, ctx);
    }
    if count {
        return handle_count_mode(results, ctx);
    }
    let ctx_after = context_lines.or(after_context).unwrap_or(0);
    let ctx_before = context_lines.or(before_context).unwrap_or(0);
    if ctx_after > 0 || ctx_before > 0 {
        print_context_lines(results, ctx.project_path, ctx_before, ctx_after);
        print_raw_results(raw_results, &QueryOutputFormat::Text);
        return Ok(true);
    }
    Ok(false)
}

fn handle_files_with_matches(
    results: &[QueryResult], raw_results: &[RawSearchResult], ctx: &MergeContext,
) -> anyhow::Result<bool> {
    let mut seen = std::collections::HashSet::new();
    for r in results { seen.insert(r.file_path.clone()); }
    for r in raw_results { seen.insert(r.file_path.clone()); }
    if ctx.is_regex_or_literal {
        let raw_files = run_raw_files_for_merge(
            ctx.query, ctx.literal, ctx.ignore_case,
            ctx.language, ctx.exclude_file, ctx.exclude, ctx.project_path,
        );
        for f in raw_files { seen.insert(f); }
    }
    let mut sorted: Vec<String> = seen.into_iter().collect();
    sorted.sort();
    for f in &sorted { println!("{CYAN}{}{RESET}", f); }
    Ok(true)
}

fn handle_count_mode(
    results: &[QueryResult], ctx: &MergeContext,
) -> anyhow::Result<bool> {
    let mut file_counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for r in results { *file_counts.entry(r.file_path.clone()).or_insert(0) += 1; }
    if ctx.is_regex_or_literal {
        let raw_counts = run_raw_counts_for_merge(
            ctx.query, ctx.literal, ctx.ignore_case,
            ctx.language, ctx.exclude_file, ctx.exclude, ctx.project_path,
        );
        for c in raw_counts {
            let entry = file_counts.entry(c.file_path).or_insert(0);
            *entry = (*entry).max(c.count);
        }
    }
    for (file, cnt) in &file_counts { println!("{CYAN}{}{RESET}:{YELLOW}{}{RESET}", file, cnt); }
    Ok(true)
}

fn print_context_for_result(r: &QueryResult, project_path: &std::path::Path, ctx_before: usize, ctx_after: usize) {
    let start = r.start_line.saturating_sub(ctx_before).max(1);
    let file_path = project_path.join(&r.file_path);
    let content = match std::fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(_) => {
            // Workspace paths (e.g. "trueno/src/...") are siblings, try parent dir
            let parent_path = project_path.join("..").join(&r.file_path);
            match std::fs::read_to_string(&parent_path) {
                Ok(c) => c,
                Err(_) => return,
            }
        }
    };
    let lines: Vec<&str> = content.lines().collect();
    let end = (r.end_line + ctx_after).min(lines.len());
    println!("{BOLD}{CYAN}{}{RESET}:{YELLOW}{}{RESET}-{YELLOW}{}{RESET}  {WHITE}{}{RESET}  TDG:{GREEN}{}{RESET}",
        r.file_path, start, end, r.function_name, r.tdg_grade);
    for (line_idx, line) in lines.iter().enumerate().skip(start.saturating_sub(1)).take(end - start + 1) {
        let line_num = line_idx + 1;
        if line_num >= r.start_line && line_num <= r.end_line {
            println!("{GREEN}{:>4}{RESET} {}", line_num, line);
        } else {
            println!("{DIM}{:>4} {}{RESET}", line_num, line);
        }
    }
    println!();
}

fn print_context_lines(results: &[QueryResult], project_path: &std::path::Path, ctx_before: usize, ctx_after: usize) {
    for r in results {
        print_context_for_result(r, project_path, ctx_before, ctx_after);
    }
}

/// Print standard query output (text/json/markdown + coverage footer + git history)
#[allow(clippy::too_many_arguments)]
fn print_query_output(
    results: &[QueryResult], format: &QueryOutputFormat, code: bool, coverage: bool,
    git_data: &Option<(Vec<GitSearchResult>, Vec<CommitInfo>)>,
    project_path: &std::path::Path, index: &AgentContextIndex,
    highlight: Option<(&str, bool)>,
) {
    let output = match format {
        QueryOutputFormat::Text => {
            if code { format_text_with_code(results, highlight) } else { format_text(results) }
        }
        QueryOutputFormat::Json => format_json(results).unwrap_or_else(|e| format!("Error: {}", e)),
        QueryOutputFormat::Markdown => format_markdown(results),
    };
    println!("{}", output);

    if coverage && !matches!(format, QueryOutputFormat::Json) {
        if let Some(summary) = format_coverage_summary(results) {
            eprintln!("\x1b[2m{}\x1b[0m", summary);
        }
    }

    if let Some((ref git_hits, ref all_commits)) = git_data {
        if !git_hits.is_empty() {
            let git_output = format_git_history_colorized(git_hits, project_path, index, all_commits);
            println!("{}", git_output);
        }
    }
}

// ── Git history search with profiling ───────────────────────────────────────

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
    let commits = parse_git_log(&log_text);
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

    // Phase 5: annotate — deferred to formatting phase (no pre-warm needed)
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

// ── Git history: annotation builders, formatters, and log parsing ────────────
include!("query_handler_git_format.rs");

// ── Index management (unchanged) ────────────────────────────────────────────

/// Load local index, do incremental update if needed, and merge siblings.
fn try_incremental_update(
    project_path: &PathBuf, index_path: &PathBuf, existing: AgentContextIndex, quiet: bool,
) -> AgentContextIndex {
    if existing.manifest().file_checksums.is_empty() {
        return existing;
    }
    if !quiet { eprintln!("Checking for incremental updates..."); }
    match AgentContextIndex::build_incremental(project_path, &existing) {
        Ok(updated) => {
            if updated.manifest().last_incremental_changes > 0 {
                let _ = updated.save(index_path);
            }
            updated
        }
        Err(_) => existing,
    }
}

fn load_or_build_index(
    project_path: &PathBuf, index_path: &PathBuf, rebuild_index: bool, quiet: bool,
) -> anyhow::Result<AgentContextIndex> {
    // Check for either SQLite (.db) or blob (.idx/) index
    let db_path = index_path.with_extension("db");

    // Fail-fast: detect partial/corrupt index (manifest exists but data missing)
    let manifest_exists = index_path.join("manifest.json").exists();
    let blob_exists = index_path.join("functions.lz4").exists();
    if !db_path.exists() && manifest_exists && !blob_exists {
        eprintln!("Detected partial index (manifest without data), rebuilding...");
        let _ = std::fs::remove_dir_all(index_path);
        return build_and_save_index(project_path, index_path);
    }

    if (!index_path.exists() && !db_path.exists()) || rebuild_index {
        if !quiet { eprintln!("Building index for {:?}...", project_path); }
        return build_and_save_index(project_path, index_path);
    }
    if !quiet { eprintln!("Loading index from {:?}...", index_path); }
    match AgentContextIndex::load(index_path) {
        Ok(existing) => Ok(try_incremental_update(project_path, index_path, existing, quiet)),
        Err(e) => {
            eprintln!("Failed to load index ({}), rebuilding...", e);
            eprintln!("  Hint: for large repos, run 'pmat index' explicitly for faster rebuilds");
            build_and_save_index(project_path, index_path)
        }
    }
}

fn load_and_merge_index(
    project_path: &PathBuf, index_path: &PathBuf,
    workspace_idx: &std::path::Path, siblings: &[(PathBuf, String)],
    rebuild_index: bool, quiet: bool,
) -> anyhow::Result<AgentContextIndex> {
    let mut index = load_or_build_index(project_path, index_path, rebuild_index, quiet)?;
    if !siblings.is_empty() {
        merge_and_cache_workspace(&mut index, siblings, workspace_idx, quiet);
    }
    Ok(index)
}

/// Check if the cached workspace index is newer than all sibling indexes and local index.
fn is_workspace_cache_fresh(
    workspace_idx: &std::path::Path,
    siblings: &[(PathBuf, String)],
    local_idx: &std::path::Path,
) -> bool {
    // Prefer workspace.db mtime, fall back to workspace.idx/manifest.json
    let cache_mtime = newest_index_mtime(workspace_idx);
    let cache_mtime = match cache_mtime {
        Some(t) => t,
        None => return false, // No cache
    };

    // Check local index is not newer than cache
    if let Some(local_mtime) = newest_index_mtime(local_idx) {
        if local_mtime > cache_mtime {
            return false; // Local index updated since cache
        }
    }

    // Cache is fresh if it's newer than every sibling's index
    siblings.iter().all(|(idx_path, _)| {
        match newest_index_mtime(idx_path) {
            Some(sibling_mtime) => cache_mtime > sibling_mtime,
            None => true, // Sibling gone, cache still valid for others
        }
    })
}

/// Get the newest mtime for an index (checks context.db and context.idx/manifest.json).
fn newest_index_mtime(idx_path: &std::path::Path) -> Option<std::time::SystemTime> {
    let db_path = idx_path.with_extension("db");
    let manifest_path = idx_path.join("manifest.json");

    let db_mtime = std::fs::metadata(&db_path).and_then(|m| m.modified()).ok();
    let manifest_mtime = std::fs::metadata(&manifest_path).and_then(|m| m.modified()).ok();

    match (db_mtime, manifest_mtime) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Merge siblings into index and save the combined result as workspace cache.
fn merge_and_cache_workspace(
    index: &mut AgentContextIndex,
    siblings: &[(PathBuf, String)],
    workspace_idx: &std::path::Path,
    quiet: bool,
) {
    if !quiet {
        eprintln!("Merging {} sibling project(s):", siblings.len());
    }
    index.merge_siblings(siblings);

    // Cache the merged index for next time
    match index.save(workspace_idx) {
        Ok(()) => {
            if !quiet {
                eprintln!("Workspace index cached.");
            }
        }
        Err(e) => {
            if !quiet {
                eprintln!("Failed to cache workspace index: {}", e);
            }
        }
    }
}

/// Build index and save to disk.
///
/// Save failures are non-fatal: the in-memory index is returned so the query
/// can still proceed. This prevents "database is locked" errors from killing
/// the entire query (#161).
fn build_and_save_index(
    project_path: &PathBuf,
    index_path: &PathBuf,
) -> anyhow::Result<AgentContextIndex> {
    let start = std::time::Instant::now();
    let index = AgentContextIndex::build(project_path)
        .map_err(|e| anyhow::anyhow!("Failed to build index: {}", e))?;
    eprintln!(
        "  Index built: {} functions in {:.1}s",
        index.all_functions().len(),
        start.elapsed().as_secs_f32()
    );

    // Create .pmat directory if needed
    if let Some(parent) = index_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Save index — non-fatal on failure (index is still usable in memory)
    match index.save(index_path) {
        Ok(()) => eprintln!("Index saved to {:?}", index_path),
        Err(e) => eprintln!("Warning: Failed to save index ({}), using in-memory index", e),
    }

    Ok(index)
}

/// Handle PTX-specific modes (--ptx-flow, --ptx-diagnostics).
/// Returns Some(output) if a PTX mode was active, None otherwise.
fn handle_ptx_modes(
    ptx_flow: bool,
    ptx_diagnostics: bool,
    index: &crate::services::agent_context::AgentContextIndex,
    format: &QueryOutputFormat,
) -> Option<String> {
    if ptx_flow {
        let result = crate::services::agent_context::trace_ptx_dataflow(index);
        return Some(if matches!(format, QueryOutputFormat::Json) {
            crate::services::agent_context::format_ptx_flow_json(&result)
        } else {
            crate::services::agent_context::format_ptx_flow_text(&result)
        });
    }
    if ptx_diagnostics {
        let result = crate::services::agent_context::run_ptx_diagnostics(index);
        return Some(if matches!(format, QueryOutputFormat::Json) {
            crate::services::agent_context::format_ptx_diagnostics_json(&result)
        } else {
            crate::services::agent_context::format_ptx_diagnostics_text(&result)
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_handle_query_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create empty project
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(project_path.join("src/main.rs"), "").unwrap();

        let result = handle_query(
            "test".to_string(),
            10,
            None,
            None,
            None,
            None,
            project_path,
            QueryOutputFormat::Text,
            false,
            false,
            false,
            None,  // rank_by
            None,  // min_pagerank
            vec![], // include_project
            false, // churn
            false, // duplicates
            false, // entropy
            false, // faults
            false, // coverage
            false, // uncovered_only
            None,  // coverage_diff
            None,  // coverage_file
            false, // coverage_gaps
            false, // include_excluded
            None,  // definition_type
            false, // code
            false, // git_history
            false, // regex
            false, // literal
            false, // raw
            false, // case_sensitive
            false, // ignore_case
            None,  // exclude
            None,  // exclude_file
            false, // files_with_matches
            false, // count
            None,  // after_context
            None,  // before_context
            None,  // context_lines
            false, // ptx_flow
            false, // ptx_diagnostics
        )
        .await;

        // Should not error, just find nothing
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_query_with_functions() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create project with a function
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(
            project_path.join("src/main.rs"),
            r#"
/// Handle errors in the API layer
fn handle_api_error(err: String) -> String {
    format!("Error: {}", err)
}

fn main() {
    println!("Hello");
}
"#,
        )
        .unwrap();

        let result = handle_query(
            "error handling".to_string(),
            10,
            None,
            None,
            None,
            None,
            project_path,
            QueryOutputFormat::Json,
            false,
            true, // Force rebuild
            false,
            None,  // rank_by
            None,  // min_pagerank
            vec![], // include_project
            false, // churn
            false, // duplicates
            false, // entropy
            false, // faults
            false, // coverage
            false, // uncovered_only
            None,  // coverage_diff
            None,  // coverage_file
            false, // coverage_gaps
            false, // include_excluded
            None,  // definition_type
            false, // code
            false, // git_history
            false, // regex
            false, // literal
            false, // raw
            false, // case_sensitive
            false, // ignore_case
            None,  // exclude
            None,  // exclude_file
            false, // files_with_matches
            false, // count
            None,  // after_context
            None,  // before_context
            None,  // context_lines
            false, // ptx_flow
            false, // ptx_diagnostics
        )
        .await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_classify_commit_type() {
        assert_eq!(classify_commit_type("fix: null pointer").1, "[fix]");
        assert_eq!(classify_commit_type("feat: add auth").1, "[feat]");
        assert_eq!(classify_commit_type("refactor: simplify parser").1, "[refactor]");
        assert_eq!(classify_commit_type("docs: update README").1, "[docs]");
        assert_eq!(classify_commit_type("chore: bump deps").1, "[chore]");
        assert_eq!(classify_commit_type("random commit").1, "");
        assert_eq!(classify_commit_type("Merge branch main").1, "[merge]");
    }

    #[test]
    fn test_format_timestamp() {
        // 2024-01-01 00:00:00 UTC = 1704067200
        let ts = 1704067200_i64;
        let formatted = format_timestamp(ts);
        // Should produce something reasonable (approximate date)
        assert!(formatted.starts_with("2024"));
    }

    #[test]
    fn test_compute_decay_score() {
        let mut hotspot = FileHotspot::default();
        hotspot.commit_count = 10;
        hotspot.fix_count = 5;
        hotspot.annotation.tdg_grade = Some("D".to_string());
        hotspot.annotation.dead_code_pct = 10.0;

        let decay = compute_decay_score(&hotspot, 100);
        assert!(decay > 0.0);
        assert!(decay <= 1.0);

        // Grade A with no fixes should have low decay
        let mut healthy = FileHotspot::default();
        healthy.commit_count = 5;
        healthy.fix_count = 0;
        healthy.annotation.tdg_grade = Some("A".to_string());
        let healthy_decay = compute_decay_score(&healthy, 100);
        assert!(healthy_decay < decay, "Healthy file should have lower decay");
    }

    #[test]
    fn test_compute_impact_risk() {
        let mut hotspot = FileHotspot::default();
        hotspot.commit_count = 50;
        hotspot.annotation.max_pagerank = Some(0.01);
        hotspot.annotation.fault_count = 3;

        let risk = compute_impact_risk(&hotspot, 100);
        assert!(risk > 0.0);

        // Zero pagerank = zero risk
        let mut low_risk = FileHotspot::default();
        low_risk.commit_count = 50;
        low_risk.annotation.max_pagerank = Some(0.0);
        assert_eq!(compute_impact_risk(&low_risk, 100), 0.0);
    }

    #[test]
    fn test_parse_git_log_with_issue_refs() {
        let log = "PMAT_START\nH:abc1234567890123456789012345678901234567\nS:feat: add auth (PMAT-472)\nN:noah\nE:noah@test.com\nT:1704067200\nPMAT_FILES\nM\tsrc/main.rs";
        let commits = parse_git_log(log);
        assert_eq!(commits.len(), 1);
        assert!(commits[0].issue_refs.contains(&"PMAT-472".to_string()) || commits[0].issue_refs.contains(&"(PMAT-472)".to_string()));
        assert!(commits[0].is_feat);
    }

    #[test]
    fn test_file_annotation_default() {
        let annot = FileAnnotation::default();
        assert_eq!(annot.tdg_grade, None);
        assert_eq!(annot.function_count, 0);
        assert_eq!(annot.dead_code_count, 0);
        assert_eq!(annot.fault_count, 0);
    }
}
