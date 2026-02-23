//! Search modes: raw, coverage-gaps, extract-candidates, suggest-rename, PTX, docs.

use super::options::*;
use crate::cli::QueryOutputFormat;
use crate::services::agent_context::{
    build_coverage_map, enrich_results_with_coverage, enrich_with_coverage_diff,
    format_coverage_summary, format_json, format_markdown, is_within_indexed_function, raw_search,
    suggest_renames, AgentContextIndex, QueryResult, RawSearchOptions, RawSearchOutput,
    RawSearchResult, RenameSignal, RenameSuggestion,
};
use std::path::PathBuf;

// ── Raw search mode ─────────────────────────────────────────────────────────

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
    exclude_file: &Option<String>,
    exclude: &Option<String>,
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
    let effective_exclude_file = if exclude_tests && exclude_file.is_none() {
        Some("test".to_string())
    } else {
        None
    };
    let excl_file_ref = effective_exclude_file
        .as_deref()
        .or(exclude_file.as_deref());
    let raw_opts = RawSearchOptions {
        pattern: query,
        literal,
        case_insensitive: ignore_case,
        before_context: ctx_before,
        after_context: ctx_after,
        limit,
        language_filter: language.as_deref(),
        exclude_file_pattern: excl_file_ref,
        exclude_pattern: exclude.as_deref(),
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
    exclude_file: &Option<String>,
    exclude: &Option<String>,
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
    let raw_opts = RawSearchOptions {
        pattern: query,
        literal,
        case_insensitive: ignore_case,
        before_context: ctx_before,
        after_context: ctx_after,
        limit: remaining + indexed_results.len(), // over-fetch to account for dedup
        language_filter: language.as_deref(),
        exclude_file_pattern: exclude_file.as_deref(),
        exclude_pattern: exclude.as_deref(),
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
    exclude_file: &Option<String>,
    exclude: &Option<String>,
    project_path: &std::path::Path,
) -> Vec<String> {
    let raw_opts = RawSearchOptions {
        pattern: query,
        literal,
        case_insensitive: ignore_case,
        before_context: 0,
        after_context: 0,
        limit: 0,
        language_filter: language.as_deref(),
        exclude_file_pattern: exclude_file.as_deref(),
        exclude_pattern: exclude.as_deref(),
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
    exclude_file: &Option<String>,
    exclude: &Option<String>,
    project_path: &std::path::Path,
) -> Vec<crate::services::agent_context::FileMatchCount> {
    let raw_opts = RawSearchOptions {
        pattern: query,
        literal,
        case_insensitive: ignore_case,
        before_context: 0,
        after_context: 0,
        limit: 0,
        language_filter: language.as_deref(),
        exclude_file_pattern: exclude_file.as_deref(),
        exclude_pattern: exclude.as_deref(),
        files_with_matches: false,
        count_mode: true,
    };
    match raw_search(project_path, &raw_opts) {
        Ok(RawSearchOutput::Counts(c)) => c,
        _ => Vec::new(),
    }
}

// ── Coverage-gaps mode ──────────────────────────────────────────────────────

/// Handle `--coverage-gaps` mode: rank all functions by uncovered lines,
/// classifying exclusions to filter out coverage(off), dead code, and Makefile patterns.
#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_coverage_gaps_mode(
    index: &AgentContextIndex,
    project_path: &std::path::Path,
    format: &QueryOutputFormat,
    coverage_file: &Option<PathBuf>,
    language: &Option<String>,
    path_pattern: &Option<String>,
    exclude_tests: bool,
    limit: usize,
    quiet: bool,
    include_excluded: bool,
    files_with_matches: bool,
    count_mode: bool,
    siblings: &[(PathBuf, String)],
) -> anyhow::Result<()> {
    let mut profile = QueryProfile::new();

    // Lightweight: graph metrics only, skip call graph (not displayed in coverage-gaps)
    let mut results: Vec<QueryResult> = index
        .functions
        .iter()
        .enumerate()
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
    if !quiet {
        eprintln!(
            "Classifying coverage exclusions ({} results)...",
            results.len()
        );
    }
    crate::services::agent_context::classify_exclusions(&mut results, project_path, cached_cov_off);
    profile.phase("classify_exclusions");

    if !quiet {
        eprintln!("Loading coverage data...");
    }
    let cov_path = coverage_file.as_deref();
    let coverage_loaded =
        match enrich_results_with_coverage(&mut results, project_path, cov_path).await {
            Ok(()) => true,
            Err(e) => {
                eprintln!("{YELLOW}Warning:{RESET} {}", e);
                eprintln!("{DIM}Showing functions without coverage enrichment.{RESET}");
                false
            }
        };

    // Merge sibling coverage caches for workspace-level coverage gaps
    if coverage_loaded && !siblings.is_empty() {
        let workspace_cov = crate::services::agent_context::load_workspace_coverage(siblings);
        if !workspace_cov.is_empty() {
            if !quiet {
                eprintln!(
                    "Merging coverage from {} sibling(s) ({} files)",
                    siblings.len(),
                    workspace_cov.len()
                );
            }
            crate::services::agent_context::enrich_with_coverage(&mut results, &workspace_cov);
        }
    }
    profile.phase("enrich_coverage");

    // Only filter by coverage data if coverage was successfully loaded
    if coverage_loaded {
        results.retain(|r| r.lines_total > 0 && r.line_coverage_pct < 100.0);
    }

    let (mut testable, excluded): (Vec<QueryResult>, Vec<QueryResult>) =
        results.into_iter().partition(|r| !r.coverage_excluded);

    testable.sort_by(|a, b| {
        b.missed_lines.cmp(&a.missed_lines).then_with(|| {
            a.line_coverage_pct
                .partial_cmp(&b.line_coverage_pct)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    testable.truncate(limit);

    if testable.is_empty() && excluded.is_empty() {
        eprintln!("No coverage gaps found (100% coverage or no data).");
        return Ok(());
    }

    profile.phase("sort_partition");

    // -- File-level aggregation modes --
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

/// Format and print coverage gap results in text mode (testable gaps only)
fn print_coverage_gaps_text(results: &[QueryResult]) {
    println!(
        "{BOLD}{UNDERLINE}Coverage Gaps{RESET} ({} testable functions with uncovered code)\n",
        results.len()
    );
    for (i, r) in results.iter().enumerate() {
        let pct_color = if r.line_coverage_pct < 50.0 {
            BRIGHT_RED
        } else if r.line_coverage_pct < 80.0 {
            YELLOW
        } else {
            GREEN
        };
        let impact_str = if r.impact_score > 1.0 {
            format!(" {YELLOW}impact:{:.1}{RESET}", r.impact_score)
        } else {
            String::new()
        };
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
        println!(
            "  {DIM}coverage(off): {} functions across {} files{RESET}",
            summary.coverage_off_count, summary.coverage_off_files
        );
    }
    if summary.dead_code_count > 0 {
        println!(
            "  {DIM}dead code: {} functions across {} files{RESET}",
            summary.dead_code_count, summary.dead_code_files
        );
    }
    if summary.makefile_count > 0 {
        println!(
            "  {DIM}Makefile COVERAGE_EXCLUDE: {} functions across {} files{RESET}",
            summary.makefile_count, summary.makefile_files
        );
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
        let in_group: Vec<&&QueryResult> = excluded
            .iter()
            .filter(|r| r.coverage_exclusion == *kind)
            .collect();
        if in_group.is_empty() {
            continue;
        }

        println!(
            "  {DIM}[EXCLUDED: {label}]{RESET} ({} functions)",
            in_group.len()
        );
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
    format: &QueryOutputFormat,
    testable: Vec<QueryResult>,
    excluded: Vec<QueryResult>,
    include_excluded: bool,
) -> anyhow::Result<()> {
    let excluded_refs: Vec<&QueryResult> = excluded.iter().collect();
    let excl_summary =
        crate::services::agent_context::ExclusionSummary::from_results(&excluded_refs);

    match format {
        QueryOutputFormat::Json | QueryOutputFormat::Markdown => {
            let mut all = testable;
            if include_excluded {
                all.extend(excluded);
            }
            if matches!(format, QueryOutputFormat::Json) {
                println!(
                    "{}",
                    format_json(&all).map_err(|e| anyhow::anyhow!("{}", e))?
                );
            } else {
                println!("{}", format_markdown(&all));
            }
        }
        _ => {
            print_coverage_gaps_text_with_exclusions(
                &testable,
                &excluded_refs,
                &excl_summary,
                include_excluded,
            );
            if let Some(summary) = format_coverage_summary(&testable) {
                eprintln!("{DIM}{}{RESET}", summary);
            }
        }
    }
    Ok(())
}

/// Print text-mode coverage gaps with exclusion handling
fn print_coverage_gaps_text_with_exclusions(
    testable: &[QueryResult],
    excluded: &[&QueryResult],
    summary: &crate::services::agent_context::ExclusionSummary,
    include_excluded: bool,
) {
    if include_excluded && !excluded.is_empty() {
        println!(
            "{BOLD}{UNDERLINE}Coverage Gaps{RESET} ({} testable + {} excluded)\n",
            testable.len(),
            summary.total()
        );
        if !testable.is_empty() {
            println!("  {BOLD}[TESTABLE]{RESET}");
            print_coverage_gaps_text(testable);
        }
        print_excluded_results(excluded);
    } else {
        print_coverage_gaps_text(testable);
        if !summary.is_empty() {
            print_exclusion_summary(summary);
        }
    }
}

// ── PTX modes ───────────────────────────────────────────────────────────────

/// Handle PTX-specific modes (--ptx-flow, --ptx-diagnostics).
/// Returns Some(output) if a PTX mode was active, None otherwise.
pub(super) fn handle_ptx_modes(
    ptx_flow: bool,
    ptx_diagnostics: bool,
    index: &AgentContextIndex,
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

// ── Suggest-rename mode ─────────────────────────────────────────────────────

pub(super) fn handle_suggest_rename_mode(
    index: &AgentContextIndex,
    project_path: &std::path::Path,
    format: &QueryOutputFormat,
    path_pattern: &Option<String>,
    limit: usize,
    quiet: bool,
    apply: bool,
) -> anyhow::Result<()> {
    let path_filter = path_pattern.as_deref();
    let mut suggestions = suggest_renames(index, path_filter);

    // Filter out NoSignal entries (no useful suggestion)
    suggestions.retain(|s| s.signal != RenameSignal::NoSignal);

    if suggestions.len() > limit {
        suggestions.truncate(limit);
    }

    if !quiet {
        eprintln!(
            "Found {} _part_ files with rename suggestions",
            suggestions.len()
        );
    }

    match format {
        QueryOutputFormat::Json => {
            print_suggest_rename_json(&suggestions);
        }
        QueryOutputFormat::Markdown => {
            print_suggest_rename_markdown(&suggestions);
        }
        _ => {
            print_suggest_rename_text(&suggestions);
        }
    }

    if apply {
        execute_renames(&suggestions, project_path)?;
    }

    Ok(())
}

fn print_suggest_rename_text(suggestions: &[RenameSuggestion]) {
    println!(
        "\n{BOLD}Rename Suggestions{RESET} ({} _part_ files found)\n",
        suggestions.len()
    );

    for (i, s) in suggestions.iter().enumerate() {
        let conf_color = if s.confidence >= 0.90 {
            BRIGHT_GREEN
        } else if s.confidence >= 0.70 {
            GREEN
        } else if s.confidence >= 0.50 {
            YELLOW
        } else {
            DIM
        };

        println!(
            "  {BOLD}{:>3}.{RESET} {conf_color}[{:.2}]{RESET} {DIM_CYAN}{}{RESET} -> {CYAN}{}{RESET}",
            i + 1,
            s.confidence,
            s.current_path,
            s.suggested_name
        );
        println!(
            "       {DIM}Signal: {signal:?} ({reasoning}) | {count} definitions{RESET}",
            signal = s.signal,
            reasoning = s.reasoning,
            count = s.definition_count
        );
        if let Some(ref parent) = s.parent_file {
            println!(
                "       {DIM}Parent: {parent} ({pattern}){RESET}",
                pattern = s.inclusion_pattern.as_deref().unwrap_or("unknown")
            );
        }
        println!();
    }
}

fn print_suggest_rename_json(suggestions: &[RenameSuggestion]) {
    match serde_json::to_string_pretty(suggestions) {
        Ok(json) => println!("{json}"),
        Err(e) => eprintln!("JSON serialization error: {e}"),
    }
}

fn print_suggest_rename_markdown(suggestions: &[RenameSuggestion]) {
    println!("# Rename Suggestions\n");
    println!("| # | Confidence | Current Path | Suggested Name | Signal | Definitions |");
    println!("|---|-----------|-------------|----------------|--------|-------------|");
    for (i, s) in suggestions.iter().enumerate() {
        println!(
            "| {} | {:.2} | `{}` | `{}` | {:?} | {} |",
            i + 1,
            s.confidence,
            s.current_path,
            s.suggested_name,
            s.signal,
            s.definition_count
        );
    }
}

fn execute_renames(
    suggestions: &[RenameSuggestion],
    project_path: &std::path::Path,
) -> anyhow::Result<()> {
    let applicable: Vec<&RenameSuggestion> = suggestions
        .iter()
        .filter(|s| s.confidence >= 0.70 && s.signal != RenameSignal::NoSignal)
        .collect();

    if applicable.is_empty() {
        eprintln!("{YELLOW}No suggestions with confidence >= 0.70 to apply{RESET}");
        return Ok(());
    }

    eprintln!("\n{BOLD}Applying {} renames...{RESET}\n", applicable.len());

    let mut success_count = 0;
    for s in &applicable {
        let old_path = project_path.join(&s.current_path);
        let new_path = project_path.join(&s.suggested_path);

        // Update parent file if detected
        if let Some(ref parent) = s.parent_file {
            let parent_path = project_path.join(parent);
            if parent_path.exists() {
                let old_filename = std::path::Path::new(&s.current_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if let Ok(content) = std::fs::read_to_string(&parent_path) {
                    let updated = content.replace(old_filename, &s.suggested_name);
                    if updated != content {
                        if let Err(e) = std::fs::write(&parent_path, &updated) {
                            eprintln!("  {RED}Failed to update parent {parent}: {e}{RESET}");
                            continue;
                        }
                        eprintln!("  {GREEN}Updated{RESET} {parent}");
                    }
                }
            }
        }

        // Perform git mv
        let status = std::process::Command::new("git")
            .args([
                "mv",
                &old_path.to_string_lossy(),
                &new_path.to_string_lossy(),
            ])
            .current_dir(project_path)
            .status();

        match status {
            Ok(exit) if exit.success() => {
                eprintln!(
                    "  {GREEN}Renamed{RESET} {} -> {}",
                    s.current_path, s.suggested_name
                );
                success_count += 1;
            }
            Ok(_) => {
                eprintln!("  {RED}git mv failed{RESET} for {}", s.current_path);
            }
            Err(e) => {
                eprintln!("  {RED}Error{RESET}: {e}");
            }
        }
    }

    eprintln!(
        "\n{BOLD}Done:{RESET} {success_count}/{} renames applied",
        applicable.len()
    );

    if success_count > 0 {
        eprintln!("{DIM}Run `cargo check` to verify the renames compile correctly{RESET}");
    }

    Ok(())
}

// ── Extract candidates mode ─────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_extract_candidates_mode(
    index: &mut AgentContextIndex,
    _project_path: &std::path::Path,
    format: &QueryOutputFormat,
    language: &Option<String>,
    path_pattern: &Option<String>,
    exclude_tests: bool,
    limit: usize,
    quiet: bool,
    max_module_lines: usize,
) -> anyhow::Result<()> {
    use crate::services::agent_context::query::extract_candidates::{
        build_extraction_groups, classify_all_results, group_by_call_cluster, group_by_prefix,
    };

    let mut profile = QueryProfile::new();

    // Need source for I/O pattern scanning + call graph for clustering
    index.load_all_source();
    index.ensure_call_graph();
    profile.phase("source_load");

    // Build results with graph metrics and call graph
    let mut results: Vec<QueryResult> = index
        .functions
        .iter()
        .enumerate()
        .map(|(i, entry)| QueryResult::from_entry_with_context(entry, i, index, 0.0, true))
        .collect();
    profile.phase("build_results");

    // Filter to functions only
    results.retain(|r| r.definition_type == "function");
    apply_result_filters_coverage(&mut results, language, path_pattern, exclude_tests);
    profile.phase("filter");

    // Classify I/O patterns
    classify_all_results(&mut results);
    profile.phase("classify_io");

    // Group by prefix and call clusters
    let prefix_groups = group_by_prefix(&results);
    let cluster_groups = group_by_call_cluster(&results);
    let groups =
        build_extraction_groups(&results, &prefix_groups, &cluster_groups, max_module_lines);
    profile.phase("grouping");

    if !quiet {
        eprintln!(
            "Found {} extraction groups from {} functions",
            groups.len(),
            results.len()
        );
    }

    let display_groups = if limit > 0 && groups.len() > limit {
        &groups[..limit]
    } else {
        &groups
    };

    match format {
        QueryOutputFormat::Json => {
            let json = serde_json::to_string_pretty(display_groups)
                .map_err(|e| anyhow::anyhow!("JSON serialize: {e}"))?;
            println!("{json}");
        }
        QueryOutputFormat::Markdown => {
            print_extract_candidates_markdown(display_groups);
        }
        _ => {
            print_extract_candidates_text(display_groups);
        }
    }

    profile.phase("output");
    profile.emit(quiet);
    Ok(())
}

fn print_extract_candidates_text(
    groups: &[crate::services::agent_context::query::extract_candidates::ExtractionGroup],
) {
    println!(
        "\n{BOLD}{UNDERLINE}Extract Candidates{RESET} ({} groups)\n",
        groups.len()
    );

    for (i, g) in groups.iter().enumerate() {
        let pure_pct = if g.functions.is_empty() {
            0
        } else {
            g.pure_count * 100 / g.functions.len()
        };
        let purity_color = if pure_pct >= 80 {
            BRIGHT_GREEN
        } else if pure_pct >= 50 {
            YELLOW
        } else {
            RED
        };

        println!(
            "{BOLD}{:>3}. {CYAN}{}{RESET} ({} fns, {} LOC, {purity_color}{}% pure{RESET}) [{DIM}{}{RESET}]",
            i + 1,
            g.module_name,
            g.functions.len(),
            g.total_loc,
            pure_pct,
            g.grouping_signal,
        );
        println!("     {DIM}from: {}{RESET}", g.source_file);

        for c in &g.functions {
            let io_badge = if c.io_classification == "PURE" {
                format!("{GREEN}[PURE]{RESET}")
            } else {
                format!("{YELLOW}[IO: {}]{RESET}", c.io_patterns.join(","))
            };
            println!(
                "     {DIM}{:>6}:{RESET} {WHITE}{}{RESET} {io_badge} {DIM}({} LOC, [{}]){RESET}",
                c.start_line, c.function_name, c.loc, c.tdg_grade,
            );
        }
        println!();
    }
}

fn print_extract_candidates_markdown(
    groups: &[crate::services::agent_context::query::extract_candidates::ExtractionGroup],
) {
    println!("# Extract Candidates\n");
    for (i, g) in groups.iter().enumerate() {
        let pure_pct = if g.functions.is_empty() {
            0
        } else {
            g.pure_count * 100 / g.functions.len()
        };
        println!(
            "## {}. `{}` ({} fns, {} LOC, {}% pure) [{}]\n",
            i + 1,
            g.module_name,
            g.functions.len(),
            g.total_loc,
            pure_pct,
            g.grouping_signal,
        );
        println!("Source: `{}`\n", g.source_file);
        println!("| Line | Function | I/O | LOC | Grade |");
        println!("|------|----------|-----|-----|-------|");
        for c in &g.functions {
            let io_label = if c.io_classification == "PURE" {
                "PURE".to_string()
            } else {
                format!("IO: {}", c.io_patterns.join(", "))
            };
            println!(
                "| {} | `{}` | {} | {} | {} |",
                c.start_line, c.function_name, io_label, c.loc, c.tdg_grade,
            );
        }
        println!();
    }
}

// ── Document search helpers ─────────────────────────────────────────────────

/// Handle `--docs-only` mode: search only documents, skip code index.
pub(super) fn handle_docs_search(
    query: &str,
    limit: usize,
    project_path: &PathBuf,
    format: &QueryOutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    let doc_results = run_document_query(query, limit, project_path, quiet)?;

    match format {
        QueryOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&doc_results)
                .map_err(|e| anyhow::anyhow!("JSON serialize: {e}"))?;
            println!("{json}");
        }
        _ => {
            print_document_results(&doc_results, false);
        }
    }
    Ok(())
}

/// Emit a document results section appended after code results (for `--docs`).
pub(super) fn emit_docs_section(
    query: &str,
    limit: usize,
    project_path: &PathBuf,
    format: &QueryOutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    let doc_results = run_document_query(query, limit, project_path, quiet)?;

    if doc_results.is_empty() {
        return Ok(());
    }

    match format {
        QueryOutputFormat::Json => {
            // For JSON, print a separate documents array
            let json = serde_json::json!({ "documents": doc_results });
            println!(
                "{}",
                serde_json::to_string_pretty(&json)
                    .map_err(|e| anyhow::anyhow!("JSON serialize: {e}"))?
            );
        }
        _ => {
            print_document_results(&doc_results, true);
        }
    }
    Ok(())
}

/// Execute the document query: build index if needed, then FTS5 search.
fn run_document_query(
    query: &str,
    limit: usize,
    project_path: &PathBuf,
    quiet: bool,
) -> anyhow::Result<Vec<crate::services::agent_context::DocumentResult>> {
    use crate::services::agent_context::document_index::{build_document_index, query_documents};
    use crate::services::agent_context::function_index::sqlite_backend::open_db;

    let db_path = project_path.join(".pmat").join("context.db");
    if !db_path.exists() {
        // Need to create DB with schema first
        std::fs::create_dir_all(project_path.join(".pmat"))
            .map_err(|e| anyhow::anyhow!("Failed to create .pmat dir: {e}"))?;
    }

    let conn = open_db(&db_path).map_err(|e| anyhow::anyhow!("{e}"))?;

    // Ensure documents schema exists (may be missing on pre-existing DBs)
    crate::services::agent_context::document_index::create_documents_schema(&conn)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Lazy-build document index
    if !quiet {
        eprint!("{DIM}Building document index...{RESET}");
    }
    let build_result =
        build_document_index(&conn, project_path).map_err(|e| anyhow::anyhow!("{e}"))?;
    if !quiet {
        eprintln!(
            "\r{DIM}Documents: {} scanned, {} indexed, {} cached{RESET}",
            build_result.files_scanned, build_result.files_indexed, build_result.files_skipped
        );
        for err in &build_result.errors {
            eprintln!("{DIM}{YELLOW}  warn: {err}{RESET}");
        }
    }

    let results = query_documents(&conn, query, limit).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(results)
}

/// Print document results to terminal with colors.
pub(super) fn print_document_results(
    results: &[crate::services::agent_context::DocumentResult],
    show_separator: bool,
) {
    if results.is_empty() {
        eprintln!("{DIM}No document matches found.{RESET}");
        return;
    }

    if show_separator {
        println!("\n{BOLD}-- Document Results --{RESET}\n");
    }

    for (i, r) in results.iter().enumerate() {
        let doc_type_badge = match r.doc_type.as_str() {
            "pdf" => format!("{RED}PDF{RESET}"),
            "svg" => format!("{GREEN}SVG{RESET}"),
            "image" => format!("{YELLOW}IMG{RESET}"),
            "markdown" => format!("{CYAN}MD{RESET}"),
            "plaintext" => format!("{DIM}TXT{RESET}"),
            other => other.to_string(),
        };

        let location = if let Some(page) = r.page_number {
            format!(" p.{page}")
        } else if let Some(ref heading) = r.section_heading {
            format!(" \u{00a7} {heading}")
        } else {
            String::new()
        };

        let quality_bar = if r.extraction_quality >= 0.8 {
            format!("{GREEN}\u{25cf}{RESET}")
        } else if r.extraction_quality >= 0.5 {
            format!("{YELLOW}\u{25cf}{RESET}")
        } else {
            format!("{RED}\u{25cb}{RESET}")
        };

        println!(
            "{DIM}{:>3}.{RESET} [{doc_type_badge}] {quality_bar} {BOLD}{}{RESET}{DIM}{location}{RESET}",
            i + 1,
            r.file_path,
        );

        // Print snippet (first 200 chars)
        let snippet = if r.snippet.len() > 200 {
            format!("{}...", &r.snippet[..200])
        } else {
            r.snippet.clone()
        };
        println!("     {DIM}{snippet}{RESET}");
    }

    println!(
        "\n{DIM}Found {} document match{}{RESET}",
        results.len(),
        if results.len() == 1 { "" } else { "es" }
    );
}

/// Apply coverage diff enrichment from a baseline file
pub(super) fn apply_coverage_diff(
    results: &mut [QueryResult],
    project_path: &std::path::Path,
    diff_path: &std::path::Path,
    quiet: bool,
) {
    match std::fs::read_to_string(diff_path) {
        Ok(json) => match build_coverage_map(&json, project_path) {
            Ok(baseline) => {
                enrich_with_coverage_diff(results, &baseline);
            }
            Err(e) => {
                if !quiet {
                    eprintln!("Warning: Could not parse coverage baseline: {}", e);
                }
            }
        },
        Err(e) => {
            if !quiet {
                eprintln!(
                    "Warning: Could not read coverage baseline {}: {}",
                    diff_path.display(),
                    e
                );
            }
        }
    }
}
