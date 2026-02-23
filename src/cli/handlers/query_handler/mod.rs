//! Query Handler - Semantic code search for agents (PMAT-470)
//!
//! Provides RAG-powered code search with quality annotations.
//! Designed as a grep replacement for AI agents.

#![cfg_attr(coverage_nightly, coverage(off))]

mod enrichments;
mod formatting;
mod git_history;
mod indexing;
mod modes;
mod options;

use crate::cli::QueryOutputFormat;
use std::path::PathBuf;

use enrichments::{apply_all_enrichments, fetch_git_data, merge_raw_results};
use formatting::emit_query_output;
use indexing::{
    backfill_results_source, collect_siblings, emit_index_stats, load_query_index,
    prepare_index_for_mode,
};
use modes::{
    emit_docs_section, handle_coverage_gaps_mode, handle_docs_search,
    handle_extract_candidates_mode, handle_ptx_modes, handle_raw_search_mode,
    handle_suggest_rename_mode,
};
use options::{
    apply_post_enrichment_sort, apply_result_filters, build_query_options, MergeContext,
    QueryProfile,
};

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
    suggest_rename: bool,
    apply: bool,
    docs: bool,
    docs_only: bool,
    extract_candidates: bool,
    max_module_lines: usize,
) -> anyhow::Result<()> {
    let quiet = matches!(format, QueryOutputFormat::Json);
    let mut profile = QueryProfile::new();

    // -- Raw search mode: skip index entirely --
    if raw {
        return handle_raw_search_mode(
            &query,
            limit,
            &format,
            quiet,
            literal,
            ignore_case,
            &language,
            &exclude_file,
            &exclude,
            files_with_matches,
            count,
            context_lines,
            after_context,
            before_context,
            &project_path,
            exclude_tests,
        );
    }

    // -- Docs-only mode: search documents, skip code index --
    if docs_only {
        return handle_docs_search(&query, limit, &project_path, &format, quiet);
    }

    // -- Load index --
    let mut index = load_query_index(&project_path, rebuild_index, &include_project, quiet)?;
    profile.phase("load_index");

    let is_regex_or_literal = regex || literal;
    let is_ptx = ptx_flow || ptx_diagnostics;
    prepare_index_for_mode(&mut index, is_regex_or_literal, is_ptx, &rank_by);
    profile.phase("source_load");

    emit_index_stats(&index, quiet);

    // -- Coverage-gaps mode --
    if coverage_gaps {
        let siblings = collect_siblings(&project_path, &include_project);
        return handle_coverage_gaps_mode(
            &index,
            &project_path,
            &format,
            &coverage_file,
            &language,
            &path_pattern,
            exclude_tests,
            limit,
            quiet,
            include_excluded,
            files_with_matches,
            count,
            &siblings,
        )
        .await;
    }

    // -- Extract-candidates mode --
    if extract_candidates {
        return handle_extract_candidates_mode(
            &mut index,
            &project_path,
            &format,
            &language,
            &path_pattern,
            exclude_tests,
            limit,
            quiet,
            max_module_lines,
        )
        .await;
    }

    // -- Suggest-rename mode --
    if suggest_rename {
        return handle_suggest_rename_mode(
            &index,
            &project_path,
            &format,
            &path_pattern,
            limit,
            quiet,
            apply,
        );
    }

    // -- PTX modes (flow / diagnostics) --
    if let Some(output) = handle_ptx_modes(ptx_flow, ptx_diagnostics, &index, &format) {
        print!("{output}");
        return Ok(());
    }

    // -- Execute semantic query + enrich + output --
    let effective_include_source = include_source || code || is_regex_or_literal;
    let merge_language = if is_regex_or_literal {
        language.clone()
    } else {
        None
    };
    let merge_exclude_file = if is_regex_or_literal {
        exclude_file.clone()
    } else {
        None
    };
    let merge_exclude = if is_regex_or_literal {
        exclude.clone()
    } else {
        None
    };

    let options = build_query_options(
        limit,
        min_grade,
        max_complexity,
        language,
        path_pattern,
        effective_include_source,
        &rank_by,
        min_pagerank,
        regex,
        literal,
        case_sensitive,
        ignore_case,
        exclude,
        exclude_file,
    );
    let mut results = index
        .query(&query, options)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    profile.phase("query");

    apply_result_filters(&mut results, exclude_tests, &definition_type);
    apply_all_enrichments(
        &mut results,
        &project_path,
        quiet,
        churn,
        duplicates,
        entropy,
        faults,
        coverage,
        uncovered_only,
        &coverage_file,
        &coverage_diff,
    )
    .await;
    profile.phase("enrich");
    apply_post_enrichment_sort(&mut results, &rank_by);

    let git_data = fetch_git_data(git_history, &project_path, &query, limit, &index, quiet)?;
    profile.phase("git_history");

    if !is_regex_or_literal {
        backfill_results_source(&mut results, &index);
    }

    let merge_ctx = MergeContext {
        query: &query,
        literal,
        ignore_case,
        language: &merge_language,
        exclude_file: &merge_exclude_file,
        exclude: &merge_exclude,
        project_path: &project_path,
        is_regex_or_literal,
    };
    let raw_results = merge_raw_results(
        is_regex_or_literal,
        quiet,
        &query,
        limit,
        &merge_ctx,
        context_lines,
        after_context,
        before_context,
        &results,
    );

    emit_query_output(
        &results,
        &raw_results,
        &git_data,
        &query,
        &format,
        effective_include_source,
        coverage,
        files_with_matches,
        count,
        context_lines,
        after_context,
        before_context,
        &merge_ctx,
        &project_path,
        &index,
    )?;
    profile.phase("output");

    // -- Append document results (default on for semantic mode) --
    if docs && !is_regex_or_literal {
        emit_docs_section(&query, limit, &project_path, &format, quiet)?;
        profile.phase("docs");
    }

    profile.emit(quiet);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::git_history::{classify_commit_type, compute_decay_score, compute_impact_risk, format_timestamp, parse_git_log};
    use super::options::{FileAnnotation, FileHotspot};
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
            None,   // rank_by
            None,   // min_pagerank
            vec![], // include_project
            false,  // churn
            false,  // duplicates
            false,  // entropy
            false,  // faults
            false,  // coverage
            false,  // uncovered_only
            None,   // coverage_diff
            None,   // coverage_file
            false,  // coverage_gaps
            false,  // include_excluded
            None,   // definition_type
            false,  // code
            false,  // git_history
            false,  // regex
            false,  // literal
            false,  // raw
            false,  // case_sensitive
            false,  // ignore_case
            None,   // exclude
            None,   // exclude_file
            false,  // files_with_matches
            false,  // count
            None,   // after_context
            None,   // before_context
            None,   // context_lines
            false,  // ptx_flow
            false,  // ptx_diagnostics
            false,  // suggest_rename
            false,  // apply
            false,  // docs
            false,  // docs_only
            false,  // extract_candidates
            500,    // max_module_lines
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
            None,   // rank_by
            None,   // min_pagerank
            vec![], // include_project
            false,  // churn
            false,  // duplicates
            false,  // entropy
            false,  // faults
            false,  // coverage
            false,  // uncovered_only
            None,   // coverage_diff
            None,   // coverage_file
            false,  // coverage_gaps
            false,  // include_excluded
            None,   // definition_type
            false,  // code
            false,  // git_history
            false,  // regex
            false,  // literal
            false,  // raw
            false,  // case_sensitive
            false,  // ignore_case
            None,   // exclude
            None,   // exclude_file
            false,  // files_with_matches
            false,  // count
            None,   // after_context
            None,   // before_context
            None,   // context_lines
            false,  // ptx_flow
            false,  // ptx_diagnostics
            false,  // suggest_rename
            false,  // apply
            false,  // docs
            false,  // docs_only
            false,  // extract_candidates
            500,    // max_module_lines
        )
        .await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_classify_commit_type() {
        assert_eq!(classify_commit_type("fix: null pointer").1, "[fix]");
        assert_eq!(classify_commit_type("feat: add auth").1, "[feat]");
        assert_eq!(
            classify_commit_type("refactor: simplify parser").1,
            "[refactor]"
        );
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
        assert!(
            healthy_decay < decay,
            "Healthy file should have lower decay"
        );
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
        assert!(
            commits[0].issue_refs.contains(&"PMAT-472".to_string())
                || commits[0].issue_refs.contains(&"(PMAT-472)".to_string())
        );
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
