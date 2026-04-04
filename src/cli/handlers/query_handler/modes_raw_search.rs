
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
    // When --exclude-tests is set in raw mode, filter test file paths
    let mut excl_files: Vec<&str> = exclude_file.iter().map(|s| s.as_str()).collect();
    if exclude_tests && excl_files.is_empty() {
        excl_files.push("test");
    }
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
    print_raw_search_output(&output, format, quiet)
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

