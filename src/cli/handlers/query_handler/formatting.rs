//! Output formatting, display, context printing.

use super::modes::{print_raw_match_context, run_raw_counts_for_merge, run_raw_files_for_merge};
use super::options::*;
use crate::cli::QueryOutputFormat;
use crate::services::agent_context::{
    format_coverage_summary, format_json, format_markdown, format_text, format_text_with_code,
    AgentContextIndex, QueryResult, RawSearchResult,
};
use crate::services::git_history::{CommitInfo, GitSearchResult};

// ── Main query output ───────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub(super) fn emit_query_output(
    results: &[QueryResult],
    raw_results: &[RawSearchResult],
    git_data: &GitData,
    query: &str,
    format: &QueryOutputFormat,
    include_source: bool,
    coverage: bool,
    files_with_matches: bool,
    count: bool,
    context_lines: Option<usize>,
    after_context: Option<usize>,
    before_context: Option<usize>,
    merge_ctx: &MergeContext,
    project_path: &std::path::Path,
    index: &AgentContextIndex,
) -> anyhow::Result<()> {
    if results.is_empty()
        && raw_results.is_empty()
        && git_data.as_ref().map_or(true, |(hits, _)| hits.is_empty())
    {
        eprintln!("No matching functions found for: {}", query);
        return Ok(());
    }

    if try_special_output_modes_merged(
        results,
        raw_results,
        files_with_matches,
        count,
        context_lines,
        after_context,
        before_context,
        merge_ctx,
    )? {
        return Ok(());
    }

    let highlight = if merge_ctx.is_regex_or_literal {
        Some((query, merge_ctx.literal))
    } else {
        None
    };
    print_query_output(
        results,
        format,
        include_source,
        coverage,
        git_data,
        project_path,
        index,
        highlight,
    );
    print_raw_results(raw_results, format);
    Ok(())
}

/// Print raw file matches (non-indexed).
fn print_raw_results(raw_results: &[RawSearchResult], format: &QueryOutputFormat) {
    debug_assert!(!raw_results.is_empty(), "raw_results must not be empty");
    if raw_results.is_empty() {
        return;
    }
    if matches!(format, QueryOutputFormat::Json) {
        let json = serde_json::to_string_pretty(&raw_results).unwrap_or_default();
        eprintln!("\n{{\"raw_matches\": {}}}", json);
    } else {
        eprintln!(
            "\n{DIM}-- Raw file matches ({} non-indexed) --{RESET}",
            raw_results.len()
        );
        for r in raw_results {
            print_raw_match_context(
                &r.file_path,
                r.line_number,
                &r.line_content,
                &r.context_before,
                &r.context_after,
            );
        }
    }
}

/// Returns Ok(true) if handled, Ok(false) for standard output.
#[allow(clippy::too_many_arguments)]
fn try_special_output_modes_merged(
    results: &[QueryResult],
    raw_results: &[RawSearchResult],
    files_with_matches: bool,
    count: bool,
    context_lines: Option<usize>,
    after_context: Option<usize>,
    before_context: Option<usize>,
    ctx: &MergeContext,
) -> anyhow::Result<bool> {
    debug_assert!(!results.is_empty(), "results must not be empty");
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
    results: &[QueryResult],
    raw_results: &[RawSearchResult],
    ctx: &MergeContext,
) -> anyhow::Result<bool> {
    debug_assert!(!results.is_empty(), "results must not be empty");
    let mut seen = std::collections::HashSet::new();
    for r in results {
        seen.insert(r.file_path.clone());
    }
    for r in raw_results {
        seen.insert(r.file_path.clone());
    }
    if ctx.is_regex_or_literal {
        let raw_files = run_raw_files_for_merge(
            ctx.query,
            ctx.literal,
            ctx.ignore_case,
            ctx.language,
            ctx.exclude_file,
            ctx.exclude,
            ctx.project_path,
        );
        for f in raw_files {
            seen.insert(f);
        }
    }
    let mut sorted: Vec<String> = seen.into_iter().collect();
    sorted.sort();
    for f in &sorted {
        println!("{CYAN}{}{RESET}", f);
    }
    Ok(true)
}

fn handle_count_mode(results: &[QueryResult], ctx: &MergeContext) -> anyhow::Result<bool> {
    debug_assert!(!results.is_empty(), "results must not be empty");
    let mut file_counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for r in results {
        *file_counts.entry(r.file_path.clone()).or_insert(0) += 1;
    }
    if ctx.is_regex_or_literal {
        let raw_counts = run_raw_counts_for_merge(
            ctx.query,
            ctx.literal,
            ctx.ignore_case,
            ctx.language,
            ctx.exclude_file,
            ctx.exclude,
            ctx.project_path,
        );
        for c in raw_counts {
            let entry = file_counts.entry(c.file_path).or_insert(0);
            *entry = (*entry).max(c.count);
        }
    }
    for (file, cnt) in &file_counts {
        println!("{CYAN}{}{RESET}:{YELLOW}{}{RESET}", file, cnt);
    }
    Ok(true)
}

fn print_context_for_result(
    r: &QueryResult,
    project_path: &std::path::Path,
    ctx_before: usize,
    ctx_after: usize,
) {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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
    for (line_idx, line) in lines
        .iter()
        .enumerate()
        .skip(start.saturating_sub(1))
        .take(end - start + 1)
    {
        let line_num = line_idx + 1;
        if line_num >= r.start_line && line_num <= r.end_line {
            println!("{GREEN}{:>4}{RESET} {}", line_num, line);
        } else {
            println!("{DIM}{:>4} {}{RESET}", line_num, line);
        }
    }
    println!();
}

fn print_context_lines(
    results: &[QueryResult],
    project_path: &std::path::Path,
    ctx_before: usize,
    ctx_after: usize,
) {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    debug_assert!(!results.is_empty(), "results must not be empty");
    for r in results {
        print_context_for_result(r, project_path, ctx_before, ctx_after);
    }
}

/// Print standard query output (text/json/markdown + coverage footer + git history)
#[allow(clippy::too_many_arguments)]
fn print_query_output(
    results: &[QueryResult],
    format: &QueryOutputFormat,
    code: bool,
    coverage: bool,
    git_data: &Option<(Vec<GitSearchResult>, Vec<CommitInfo>)>,
    project_path: &std::path::Path,
    index: &AgentContextIndex,
    highlight: Option<(&str, bool)>,
) {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    debug_assert!(!results.is_empty(), "results must not be empty");
    let output = match format {
        QueryOutputFormat::Text => {
            if code {
                format_text_with_code(results, highlight)
            } else {
                format_text(results)
            }
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
            let git_output = super::git_history::format_git_history_colorized(
                git_hits,
                project_path,
                index,
                all_commits,
            );
            println!("{}", git_output);
        }
    }
}
