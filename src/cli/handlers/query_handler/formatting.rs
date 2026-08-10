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
        && git_data.as_ref().is_none_or(|(hits, _)| hits.is_empty())
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
        format,
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

/// Colour code, or nothing when stdout is not a terminal.
///
/// `--files-with-matches`, `--count` and `-A/-B/-C` wrote raw CYAN/YELLOW
/// escapes unconditionally, so redirecting them to a file produced a file full
/// of `\e[36m`. Everything these three modes print goes through here.
fn tint(code: &'static str) -> &'static str {
    use std::io::IsTerminal;
    if std::io::stdout().is_terminal() {
        code
    } else {
        ""
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
    format: &QueryOutputFormat,
) -> anyhow::Result<bool> {
    if files_with_matches {
        return handle_files_with_matches(results, raw_results, ctx, format);
    }
    if count {
        return handle_count_mode(results, ctx, format);
    }
    let ctx_after = context_lines.or(after_context).unwrap_or(0);
    let ctx_before = context_lines.or(before_context).unwrap_or(0);
    if ctx_after > 0 || ctx_before > 0 {
        // `--format json` used to be discarded on this path twice over: the
        // context blocks were printed as ANSI text, and the raw matches were
        // rendered with a hardcoded `QueryOutputFormat::Text`.
        print_context_lines(results, ctx.project_path, ctx_before, ctx_after, format);
        print_raw_results(raw_results, format);
        return Ok(true);
    }
    Ok(false)
}

fn handle_files_with_matches(
    results: &[QueryResult],
    raw_results: &[RawSearchResult],
    ctx: &MergeContext,
    format: &QueryOutputFormat,
) -> anyhow::Result<bool> {
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
    println!("{}", render_files_with_matches(&sorted, format)?);
    Ok(true)
}

/// `--files-with-matches` in the declared format.
fn render_files_with_matches(
    files: &[String],
    format: &QueryOutputFormat,
) -> anyhow::Result<String> {
    if matches!(format, QueryOutputFormat::Json) {
        return Ok(serde_json::to_string_pretty(files)?);
    }
    Ok(files
        .iter()
        .map(|f| format!("{}{}{}", tint(CYAN), f, tint(RESET)))
        .collect::<Vec<_>>()
        .join("\n"))
}

fn handle_count_mode(
    results: &[QueryResult],
    ctx: &MergeContext,
    format: &QueryOutputFormat,
) -> anyhow::Result<bool> {
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
    println!("{}", render_counts(&file_counts, format)?);
    Ok(true)
}

/// `--count` in the declared format. `--format json` used to emit
/// `\e[36m<path>\e[0m:\e[33m1\e[0m`, which is not JSON by any reading.
fn render_counts(
    file_counts: &std::collections::BTreeMap<String, usize>,
    format: &QueryOutputFormat,
) -> anyhow::Result<String> {
    if matches!(format, QueryOutputFormat::Json) {
        let rows: Vec<serde_json::Value> = file_counts
            .iter()
            .map(|(file, count)| serde_json::json!({ "file": file, "count": count }))
            .collect();
        return Ok(serde_json::to_string_pretty(&rows)?);
    }
    Ok(file_counts
        .iter()
        .map(|(file, cnt)| {
            format!(
                "{}{}{}:{}{}{}",
                tint(CYAN),
                file,
                tint(RESET),
                tint(YELLOW),
                cnt,
                tint(RESET)
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

/// The lines a context block covers, or `None` when the file could not be read.
fn context_block(
    r: &QueryResult,
    project_path: &std::path::Path,
    ctx_before: usize,
    ctx_after: usize,
) -> Option<(usize, usize, Vec<String>)> {
    let start = r.start_line.saturating_sub(ctx_before).max(1);
    let file_path = project_path.join(&r.file_path);
    let content = match std::fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(_) => {
            // Workspace paths (e.g. "trueno/src/...") are siblings, try parent dir
            let parent_path = project_path.join("..").join(&r.file_path);
            std::fs::read_to_string(&parent_path).ok()?
        }
    };
    let lines: Vec<&str> = content.lines().collect();
    let end = (r.end_line + ctx_after).min(lines.len());
    let body = lines
        .iter()
        .enumerate()
        .skip(start.saturating_sub(1))
        .take((end + 1).saturating_sub(start))
        .map(|(_, line)| (*line).to_string())
        .collect();
    Some((start, end, body))
}

fn print_context_for_result(
    r: &QueryResult,
    project_path: &std::path::Path,
    ctx_before: usize,
    ctx_after: usize,
) {
    let Some((start, end, body)) = context_block(r, project_path, ctx_before, ctx_after) else {
        return;
    };
    let pv_display = r
        .contract_level
        .as_deref()
        .map(|l| format!("  PV:{}{l}{}", tint(GREEN), tint(RESET)))
        .unwrap_or_default();
    println!(
        "{}{}{}{}:{}{}{}-{}{}{}  {}{}{}  TDG:{}{}{}{pv_display}",
        tint(BOLD),
        tint(CYAN),
        r.file_path,
        tint(RESET),
        tint(YELLOW),
        start,
        tint(RESET),
        tint(YELLOW),
        end,
        tint(RESET),
        tint(WHITE),
        r.function_name,
        tint(RESET),
        tint(GREEN),
        r.tdg_grade,
        tint(RESET)
    );
    for (offset, line) in body.iter().enumerate() {
        let line_num = start + offset;
        if line_num >= r.start_line && line_num <= r.end_line {
            println!("{}{:>4}{} {}", tint(GREEN), line_num, tint(RESET), line);
        } else {
            println!("{}{:>4} {}{}", tint(DIM), line_num, line, tint(RESET));
        }
    }
    println!();
}

/// Context blocks as JSON: one object per match, with the covered lines.
fn context_blocks_json(
    results: &[QueryResult],
    project_path: &std::path::Path,
    ctx_before: usize,
    ctx_after: usize,
) -> serde_json::Value {
    let blocks: Vec<serde_json::Value> = results
        .iter()
        .filter_map(|r| {
            let (start, end, body) = context_block(r, project_path, ctx_before, ctx_after)?;
            let lines: Vec<serde_json::Value> = body
                .iter()
                .enumerate()
                .map(|(offset, line)| {
                    let line_num = start + offset;
                    serde_json::json!({
                        "line_number": line_num,
                        "text": line,
                        "is_match": line_num >= r.start_line && line_num <= r.end_line,
                    })
                })
                .collect();
            Some(serde_json::json!({
                "file": r.file_path,
                "function": r.function_name,
                "tdg_grade": r.tdg_grade,
                "contract_level": r.contract_level,
                "start_line": start,
                "end_line": end,
                "lines": lines,
            }))
        })
        .collect();
    serde_json::json!({ "context_matches": blocks })
}

fn print_context_lines(
    results: &[QueryResult],
    project_path: &std::path::Path,
    ctx_before: usize,
    ctx_after: usize,
    format: &QueryOutputFormat,
) {
    if matches!(format, QueryOutputFormat::Json) {
        let json = context_blocks_json(results, project_path, ctx_before, ctx_after);
        println!(
            "{}",
            serde_json::to_string_pretty(&json).unwrap_or_default()
        );
        return;
    }
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

#[cfg(test)]
mod declared_format_tests {
    use super::*;

    fn result_for(file: &str, start: usize, end: usize) -> QueryResult {
        serde_json::from_value(serde_json::json!({
            "file_path": file,
            "function_name": "f",
            "signature": "fn f()",
            "doc_comment": null,
            "start_line": start,
            "end_line": end,
            "language": "rust",
            "tdg_score": 90.0,
            "tdg_grade": "A",
            "complexity": 1,
            "big_o": "O(1)",
            "satd_count": 0,
            "loc": 3,
            "relevance_score": 1.0,
            "source": null
        }))
        .expect("QueryResult fixture")
    }

    /// `query --count --format json` emitted
    /// `\e[36m<path>\e[0m:\e[33m1\e[0m` — ANSI text, not JSON, and not even
    /// valid JSON to a `| jq` consumer.
    #[test]
    fn count_mode_honours_format_json() {
        let counts = std::collections::BTreeMap::from([
            ("src/a.rs".to_string(), 3usize),
            ("src/b.rs".to_string(), 1usize),
        ]);
        let rendered = render_counts(&counts, &QueryOutputFormat::Json).expect("render");
        assert!(
            !rendered.contains('\u{1b}'),
            "JSON must not carry ANSI escapes: {rendered:?}"
        );
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("valid json");
        assert_eq!(value[0]["file"], "src/a.rs");
        assert_eq!(value[0]["count"], 3);
        assert_eq!(value[1]["count"], 1);
    }

    /// Same defect on `--files-with-matches`.
    #[test]
    fn files_with_matches_honours_format_json() {
        let files = vec!["src/a.rs".to_string(), "src/b.rs".to_string()];
        let rendered = render_files_with_matches(&files, &QueryOutputFormat::Json).expect("render");
        assert!(!rendered.contains('\u{1b}'), "{rendered:?}");
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("valid json");
        assert_eq!(value, serde_json::json!(["src/a.rs", "src/b.rs"]));
    }

    /// ... and on `-A/-B/-C`, which additionally hardcoded
    /// `print_raw_results(.., &QueryOutputFormat::Text)`.
    #[test]
    fn context_mode_honours_format_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("a.rs"), "one\ntwo\nthree\nfour\n").expect("write");

        let results = vec![result_for("a.rs", 2, 2)];
        let value = context_blocks_json(&results, dir.path(), 1, 1);
        let rendered = serde_json::to_string(&value).expect("serialize");
        assert!(!rendered.contains('\u{1b}'), "{rendered:?}");

        let block = &value["context_matches"][0];
        assert_eq!(block["file"], "a.rs");
        assert_eq!(block["start_line"], 1);
        assert_eq!(block["end_line"], 3);
        assert_eq!(block["lines"][0]["text"], "one");
        assert_eq!(block["lines"][0]["is_match"], false);
        assert_eq!(block["lines"][1]["text"], "two");
        assert_eq!(block["lines"][1]["is_match"], true);
    }

    /// Text mode keeps its shape, minus the escapes when stdout is redirected —
    /// these modes wrote raw CYAN/YELLOW into files.
    #[test]
    fn text_mode_is_plain_when_stdout_is_not_a_terminal() {
        use std::io::IsTerminal;
        let files = vec!["src/a.rs".to_string()];
        let rendered = render_files_with_matches(&files, &QueryOutputFormat::Text).expect("render");
        if std::io::stdout().is_terminal() {
            assert!(rendered.contains("src/a.rs"));
        } else {
            assert_eq!(rendered, "src/a.rs");
        }
    }
}
