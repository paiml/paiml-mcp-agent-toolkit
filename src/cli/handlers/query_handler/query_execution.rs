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
