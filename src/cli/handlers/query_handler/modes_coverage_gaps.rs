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
    debug_assert!(!results.is_empty(), "results must not be empty");
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
    debug_assert!(!results.is_empty(), "results must not be empty");
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

