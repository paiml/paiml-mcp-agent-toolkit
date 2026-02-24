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

