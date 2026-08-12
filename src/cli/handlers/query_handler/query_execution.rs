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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
    search_mode: Option<String>,
    raw: bool,
    case_sensitive: bool,
    ignore_case: bool,
    exclude: Vec<String>,
    exclude_file: Vec<String>,
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
    debug_assert!(!query.is_empty() || coverage_gaps || extract_candidates,
        "query string required unless --coverage-gaps or --extract-candidates");

    let quiet = matches!(format, QueryOutputFormat::Json);
    let mut profile = QueryProfile::new();

    // -- Issue #562: --search-mode {semantic,lexical,hybrid} --
    // `lexical` and `hybrid` desugar onto the existing engine paths so the
    // RRF blend (`hybrid`) and the no-embedding path (`lexical`) are both
    // teachable from one CLI without the `pmat semantic` config gate.
    let (regex, literal, is_search_mode_hybrid, _is_search_mode_lexical) =
        resolve_search_mode(search_mode.as_deref(), regex, literal);

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

    if let Some(res) = dispatch_special_modes(
        coverage_gaps, extract_candidates, suggest_rename, ptx_flow, ptx_diagnostics,
        &mut index, &project_path, &include_project, &format, &coverage_file,
        &language, &path_pattern, exclude_tests, limit, quiet, include_excluded,
        files_with_matches, count, max_module_lines, apply
    ).await {
        return res;
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
        Vec::new()
    };
    let merge_exclude = if is_regex_or_literal {
        exclude.clone()
    } else {
        Vec::new()
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
    let mut results = if is_search_mode_hybrid {
        // Hybrid mode (issue #562): run both lexical and semantic, RRF-fuse
        // the two ranked lists at k=60 (matches `pmat semantic search
        // --search-mode hybrid`). Lexical pass uses the options built above
        // with literal=true; semantic pass uses a clone with the search mode
        // forced to Semantic.
        let lexical_options = options.clone();
        let mut semantic_options = options;
        semantic_options.search_mode =
            crate::services::agent_context::SearchMode::Semantic;
        let lexical_results = index
            .query(&query, lexical_options)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let semantic_results = index
            .query(&query, semantic_options)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        rrf_fuse(lexical_results, semantic_results, limit)
    } else {
        index
            .query(&query, options)
            .map_err(|e| anyhow::anyhow!("{}", e))?
    };
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
        exclude_tests,
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
    //
    // `--files-with-matches` and `--count` are whole-output contracts, not
    // section styles: `rg -l` prints one path per line and `rg -c` prints
    // `path:n`, and nothing else. The code printer honoured them while the
    // document printer kept dumping numbered markdown snippets after the
    // paths, so `pmat query h --files-with-matches | while read f` fed 1.6KB of
    // prose to a loop expecting filenames. Suppressed here, at the one place
    // the section is emitted, rather than taught to each printer.
    if should_emit_docs_section(docs, is_regex_or_literal, files_with_matches, count) {
        emit_docs_section(&query, limit, &project_path, &format, quiet)?;
        profile.phase("docs");
    }

    profile.emit(quiet);
    Ok(())
}

/// RRF (Reciprocal Rank Fusion) of two ranked QueryResult lists at k=60.
///
/// Issue #562: implements `--search-mode hybrid`. Each result is identified
/// by `(file_path, function_name)`; ranks are 1-indexed; the fused score is
/// the sum of `1 / (k + rank)` across the two lists. Returns the top `limit`
/// fused results, with `relevance_score` overwritten to the fused score so
/// downstream sorts surface the merged ranking. The original
/// `QueryResult` payloads (source, signature, quality annotations, etc.)
/// are preserved from whichever list saw the function first — typically the
/// lexical pass since both lists hit the same index.
fn rrf_fuse(
    lexical: Vec<crate::services::agent_context::QueryResult>,
    semantic: Vec<crate::services::agent_context::QueryResult>,
    limit: usize,
) -> Vec<crate::services::agent_context::QueryResult> {
    use std::collections::HashMap;
    const K: f32 = 60.0;

    // Key on (file_path, function_name) — same pair the rest of the
    // pipeline uses to dedupe.
    let mut fused: HashMap<(String, String), (f32, crate::services::agent_context::QueryResult)> =
        HashMap::new();

    for (rank, result) in lexical.into_iter().enumerate() {
        let key = (result.file_path.clone(), result.function_name.clone());
        let rrf = 1.0_f32 / (K + (rank as f32 + 1.0));
        fused
            .entry(key)
            .and_modify(|(score, _)| *score += rrf)
            .or_insert((rrf, result));
    }
    for (rank, result) in semantic.into_iter().enumerate() {
        let key = (result.file_path.clone(), result.function_name.clone());
        let rrf = 1.0_f32 / (K + (rank as f32 + 1.0));
        fused
            .entry(key)
            .and_modify(|(score, _)| *score += rrf)
            .or_insert((rrf, result));
    }

    let mut merged: Vec<(f32, crate::services::agent_context::QueryResult)> =
        fused.into_values().collect();
    merged.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    merged
        .into_iter()
        .take(limit)
        .map(|(score, mut r)| {
            r.relevance_score = score;
            r
        })
        .collect()
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn resolve_search_mode(
    search_mode: Option<&str>,
    regex: bool,
    literal: bool,
) -> (bool, bool, bool, bool) {
    let search_mode_normalized = search_mode.map(|s| s.to_lowercase());
    let is_search_mode_hybrid = matches!(search_mode_normalized.as_deref(), Some("hybrid"));
    let is_search_mode_lexical = matches!(search_mode_normalized.as_deref(), Some("lexical"));
    let (regex, literal) = if is_search_mode_lexical || is_search_mode_hybrid {
        (false, true)
    } else {
        (regex, literal)
    };
    (regex, literal, is_search_mode_hybrid, is_search_mode_lexical)
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
async fn dispatch_special_modes(
    coverage_gaps: bool,
    extract_candidates: bool,
    suggest_rename: bool,
    ptx_flow: bool,
    ptx_diagnostics: bool,
    index: &mut crate::services::agent_context::AgentContextIndex,
    project_path: &PathBuf,
    include_project: &[PathBuf],
    format: &QueryOutputFormat,
    coverage_file: &Option<PathBuf>,
    language: &Option<String>,
    path_pattern: &Option<String>,
    exclude_tests: bool,
    limit: usize,
    quiet: bool,
    include_excluded: bool,
    files_with_matches: bool,
    count: bool,
    max_module_lines: usize,
    apply: bool,
) -> Option<anyhow::Result<()>> {
    if coverage_gaps {
        let siblings = collect_siblings(project_path, include_project);
        return Some(
            handle_coverage_gaps_mode(
                index, project_path, format, coverage_file, language, path_pattern, exclude_tests,
                limit, quiet, include_excluded, files_with_matches, count, &siblings,
            )
            .await,
        );
    }
    if extract_candidates {
        return Some(
            handle_extract_candidates_mode(
                index, project_path, format, language, path_pattern, exclude_tests, limit, quiet,
                max_module_lines,
            )
            .await,
        );
    }
    if suggest_rename {
        return Some(handle_suggest_rename_mode(
            index, project_path, format, path_pattern, limit, quiet, apply,
        ));
    }
    if let Some(output) = handle_ptx_modes(ptx_flow, ptx_diagnostics, index, format) {
        print!("{output}");
        return Some(Ok(()));
    }
    None
}

/// May the supplementary `-- Document Results --` section be printed?
///
/// `--files-with-matches` and `--count` are contracts over the WHOLE of
/// stdout, matching `rg -l` / `rg -c`: one path per line, or one `path:count`
/// per line, and nothing else. The code printer honoured them while this
/// section printed unconditionally, so `pmat query h --files-with-matches`
/// emitted two paths followed by 1.6KB of numbered markdown snippets. THE one
/// place that decision is made — a new supplementary section consults this
/// rather than re-deriving it.
fn should_emit_docs_section(
    docs: bool,
    is_regex_or_literal: bool,
    files_with_matches: bool,
    count: bool,
) -> bool {
    docs && !is_regex_or_literal && !files_with_matches && !count
}
