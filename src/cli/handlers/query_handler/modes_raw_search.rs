
/// Print a single raw search match with surrounding context lines
pub(super) fn print_raw_match_context(
    file_path: &str,
    line_number: usize,
    line_content: &str,
    context_before: &[String],
    context_after: &[String],
) {
    if !context_before.is_empty() {
        let start_line = line_number - context_before.len();
        for (i, line) in context_before.iter().enumerate() {
            println!(
                "{DIM}{}{RESET}:{DIM}{}{RESET}-{}",
                file_path,
                start_line + i,
                line
            );
        }
    }
    println!(
        "{BOLD}{CYAN}{}{RESET}:{YELLOW}{}{RESET}:{}",
        file_path, line_number, line_content
    );
    if !context_after.is_empty() {
        for (i, line) in context_after.iter().enumerate() {
            println!(
                "{DIM}{}{RESET}:{DIM}{}{RESET}-{}",
                file_path,
                line_number + 1 + i,
                line
            );
        }
    }
}

/// Handle `--raw` mode: pure file-level search without the function index
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_raw_search_mode(
    query: &str,
    limit: usize,
    format: &QueryOutputFormat,
    quiet: bool,
    literal: bool,
    ignore_case: bool,
    language: &Option<String>,
    exclude_file: &[String],
    exclude: &[String],
    files_with_matches: bool,
    count: bool,
    context_lines: Option<usize>,
    after_context: Option<usize>,
    before_context: Option<usize>,
    project_path: &std::path::Path,
    exclude_tests: bool,
) -> anyhow::Result<()> {
    let ctx_after = context_lines.or(after_context).unwrap_or(0);
    let ctx_before = context_lines.or(before_context).unwrap_or(0);
    let excl_files: Vec<&str> = exclude_file.iter().map(|s| s.as_str()).collect();
    let raw_opts = RawSearchOptions {
        pattern: query,
        literal,
        case_insensitive: ignore_case,
        before_context: ctx_before,
        after_context: ctx_after,
        limit,
        language_filter: language.as_deref(),
        exclude_file_pattern: excl_files,
        exclude_pattern: exclude.iter().map(|s| s.as_str()).collect(),
        files_with_matches,
        count_mode: count,
    };
    let output = raw_search(project_path, &raw_opts).map_err(|e| anyhow::anyhow!("{}", e))?;
    // --exclude-tests in raw mode: filter results from test files by path
    // heuristic. The glob exclude can't reliably express nested test paths
    // (e.g. src/**/foo_tests_basic.rs), so filter the output directly.
    let output = if exclude_tests {
        drop_test_file_results(output)
    } else {
        output
    };
    print_raw_search_output(&output, format, quiet)
}

/// Drop raw-search results originating from test files (for `--exclude-tests`).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn drop_test_file_results(output: RawSearchOutput) -> RawSearchOutput {
    use super::options::is_test_path;
    match output {
        RawSearchOutput::Files(files) => {
            RawSearchOutput::Files(files.into_iter().filter(|f| !is_test_path(f)).collect())
        }
        RawSearchOutput::Counts(counts) => RawSearchOutput::Counts(
            counts
                .into_iter()
                .filter(|c| !is_test_path(&c.file_path))
                .collect(),
        ),
        RawSearchOutput::Lines(lines) => RawSearchOutput::Lines(
            lines
                .into_iter()
                .filter(|l| !is_test_path(&l.file_path))
                .collect(),
        ),
    }
}

fn print_raw_search_output(
    output: &RawSearchOutput,
    format: &QueryOutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    match output {
        RawSearchOutput::Files(files) => {
            for f in files {
                println!("{CYAN}{f}{RESET}");
            }
        }
        RawSearchOutput::Counts(counts) => {
            for c in counts {
                println!("{CYAN}{}{RESET}:{YELLOW}{}{RESET}", c.file_path, c.count);
            }
        }
        RawSearchOutput::Lines(lines) => {
            print_raw_lines(lines, format, quiet)?;
        }
    }
    Ok(())
}

fn print_raw_lines(
    lines: &[RawSearchResult],
    format: &QueryOutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    if matches!(format, QueryOutputFormat::Json) {
        let json = serde_json::to_string_pretty(lines).map_err(|e| anyhow::anyhow!("{}", e))?;
        println!("{}", json);
    } else {
        for r in lines {
            print_raw_match_context(
                &r.file_path,
                r.line_number,
                &r.line_content,
                &r.context_before,
                &r.context_after,
            );
        }
    }
    if !quiet {
        eprintln!("{} matches", lines.len());
    }
    Ok(())
}

/// Run raw search and return non-overlapping results for merge with index results.
/// Used when `--regex` or `--literal` is active (without `--raw`).
#[allow(clippy::too_many_arguments)]
pub(super) fn run_raw_search_for_merge(
    query: &str,
    limit: usize,
    literal: bool,
    ignore_case: bool,
    language: &Option<String>,
    exclude_file: &[String],
    exclude: &[String],
    context_lines: Option<usize>,
    after_context: Option<usize>,
    before_context: Option<usize>,
    project_path: &std::path::Path,
    indexed_results: &[QueryResult],
) -> Vec<RawSearchResult> {
    let remaining = limit.saturating_sub(indexed_results.len());
    if remaining == 0 {
        return Vec::new();
    }

    let ctx_after = context_lines.or(after_context).unwrap_or(0);
    let ctx_before = context_lines.or(before_context).unwrap_or(0);
    let excl_refs: Vec<&str> = exclude_file.iter().map(|s| s.as_str()).collect();
    let raw_opts = RawSearchOptions {
        pattern: query,
        literal,
        case_insensitive: ignore_case,
        before_context: ctx_before,
        after_context: ctx_after,
        limit: remaining + indexed_results.len(), // over-fetch to account for dedup
        language_filter: language.as_deref(),
        exclude_file_pattern: excl_refs,
        exclude_pattern: exclude.iter().map(|s| s.as_str()).collect(),
        files_with_matches: false,
        count_mode: false,
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
    lines
        .into_iter()
        .filter(|r| !is_within_indexed_function(&r.file_path, r.line_number, indexed_results))
        .take(remaining)
        .collect()
}

/// Run raw search and return file paths for merge with --files-with-matches mode.
#[allow(clippy::too_many_arguments)]
pub(super) fn run_raw_files_for_merge(
    query: &str,
    literal: bool,
    ignore_case: bool,
    language: &Option<String>,
    exclude_file: &[String],
    exclude: &[String],
    project_path: &std::path::Path,
) -> Vec<String> {
    let excl_refs: Vec<&str> = exclude_file.iter().map(|s| s.as_str()).collect();
    let raw_opts = RawSearchOptions {
        pattern: query,
        literal,
        case_insensitive: ignore_case,
        before_context: 0,
        after_context: 0,
        limit: 0,
        language_filter: language.as_deref(),
        exclude_file_pattern: excl_refs,
        exclude_pattern: exclude.iter().map(|s| s.as_str()).collect(),
        files_with_matches: true,
        count_mode: false,
    };
    match raw_search(project_path, &raw_opts) {
        Ok(RawSearchOutput::Files(f)) => f,
        _ => Vec::new(),
    }
}

/// Run raw search and return per-file counts for merge with --count mode.
#[allow(clippy::too_many_arguments)]
pub(super) fn run_raw_counts_for_merge(
    query: &str,
    literal: bool,
    ignore_case: bool,
    language: &Option<String>,
    exclude_file: &[String],
    exclude: &[String],
    project_path: &std::path::Path,
) -> Vec<crate::services::agent_context::FileMatchCount> {
    let excl_refs: Vec<&str> = exclude_file.iter().map(|s| s.as_str()).collect();
    let raw_opts = RawSearchOptions {
        pattern: query,
        literal,
        case_insensitive: ignore_case,
        before_context: 0,
        after_context: 0,
        limit: 0,
        language_filter: language.as_deref(),
        exclude_file_pattern: excl_refs,
        exclude_pattern: exclude.iter().map(|s| s.as_str()).collect(),
        files_with_matches: false,
        count_mode: true,
    };
    match raw_search(project_path, &raw_opts) {
        Ok(RawSearchOutput::Counts(c)) => c,
        _ => Vec::new(),
    }
}

// `items_after_test_module` fires because `modes_raw_search.rs` is
// include!()'d into modes.rs ahead of the other `modes_*.rs` siblings —
// so this test module is *textually* followed by production items from
// other include files. Silence the lint since reordering the includes
// would churn the entire handler module for no gain.
#[allow(clippy::items_after_test_module)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod modes_raw_search_tests {
    use super::*;

    /// Drives print_raw_match_context through all four arms:
    /// empty/non-empty context_before × empty/non-empty context_after.
    #[test]
    fn test_print_raw_match_context_all_context_shapes() {
        // No context either side — skips both for/iter blocks.
        print_raw_match_context("a.rs", 10, "let x = 1;", &[], &[]);
        // Before only.
        print_raw_match_context(
            "a.rs",
            10,
            "let x = 1;",
            &["fn foo() {".to_string(), "    // body".to_string()],
            &[],
        );
        // After only.
        print_raw_match_context(
            "a.rs",
            10,
            "let x = 1;",
            &[],
            &["    return x;".to_string(), "}".to_string()],
        );
        // Both.
        print_raw_match_context(
            "a.rs",
            10,
            "let x = 1;",
            &["before".to_string()],
            &["after".to_string()],
        );
    }

    /// print_raw_search_output dispatches on the RawSearchOutput variant.
    /// Each arm only does println!, so just calling it ticks coverage.
    #[test]
    fn test_print_raw_search_output_files_variant() {
        let out = RawSearchOutput::Files(vec!["a.rs".into(), "b.rs".into()]);
        print_raw_search_output(&out, &QueryOutputFormat::Text, true).unwrap();
    }

    #[test]
    fn test_print_raw_search_output_counts_variant() {
        let out = RawSearchOutput::Counts(vec![
            crate::services::agent_context::FileMatchCount {
                file_path: "a.rs".into(),
                count: 3,
            },
            crate::services::agent_context::FileMatchCount {
                file_path: "b.rs".into(),
                count: 1,
            },
        ]);
        print_raw_search_output(&out, &QueryOutputFormat::Text, true).unwrap();
    }

    #[test]
    fn test_print_raw_search_output_lines_variant_text_format() {
        let out = RawSearchOutput::Lines(vec![RawSearchResult {
            file_path: "a.rs".into(),
            line_number: 1,
            line_content: "fn x() {}".into(),
            context_before: vec![],
            context_after: vec![],
        }]);
        // Text format → delegates to print_raw_lines → print_raw_match_context.
        print_raw_search_output(&out, &QueryOutputFormat::Text, true).unwrap();
    }

    #[test]
    fn test_print_raw_search_output_lines_variant_json_format() {
        let out = RawSearchOutput::Lines(vec![RawSearchResult {
            file_path: "a.rs".into(),
            line_number: 1,
            line_content: "fn x() {}".into(),
            context_before: vec!["before".into()],
            context_after: vec!["after".into()],
        }]);
        // JSON format → serializes via serde_json::to_string_pretty.
        print_raw_search_output(&out, &QueryOutputFormat::Json, true).unwrap();
    }

    /// print_raw_lines non-quiet vs quiet: the trailing "N matches" eprintln
    /// only fires when quiet=false.
    #[test]
    fn test_print_raw_lines_quiet_vs_verbose() {
        let lines = vec![RawSearchResult {
            file_path: "a.rs".into(),
            line_number: 1,
            line_content: "x".into(),
            context_before: vec![],
            context_after: vec![],
        }];
        // Verbose: hits the `!quiet` eprintln branch.
        print_raw_lines(&lines, &QueryOutputFormat::Text, false).unwrap();
        // Quiet: skips it.
        print_raw_lines(&lines, &QueryOutputFormat::Text, true).unwrap();
    }

    /// run_raw_search_for_merge should return empty when `remaining = 0`
    /// (the early-exit branch on line 147). limit=0 with empty indexed =>
    /// 0.saturating_sub(0) == 0 => early return.
    #[test]
    fn test_run_raw_search_for_merge_early_exit_on_zero_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let out = run_raw_search_for_merge(
            "fn",
            0, // limit=0 → remaining=0 → early exit
            false,
            false,
            &None,
            &[],
            &[],
            None,
            None,
            None,
            tmp.path(),
            &[],
        );
        assert!(out.is_empty(), "limit=0 → empty merge result");
    }
}
