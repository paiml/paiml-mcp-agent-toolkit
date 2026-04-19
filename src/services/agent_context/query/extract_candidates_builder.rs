// Extraction group builder functions for extract_candidates module
// Included by extract_candidates.rs — no `use` imports allowed here.

/// Build extraction groups from prefix and cluster groupings.
///
/// Merges both grouping signals, respects `max_module_lines`, and produces
/// sorted output (largest groups first).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn build_extraction_groups(
    results: &[QueryResult],
    prefix_groups: &HashMap<String, Vec<usize>>,
    cluster_groups: &HashMap<String, Vec<usize>>,
    max_module_lines: usize,
) -> Vec<ExtractionGroup> {
    let mut groups = Vec::new();

    // Process prefix groups
    for (prefix, indices) in prefix_groups {
        let group = build_group(results, indices, prefix, "prefix", max_module_lines);
        if let Some(g) = group {
            groups.push(g);
        }
    }

    // Process cluster groups (skip if already covered by prefix)
    let prefix_indices: std::collections::HashSet<usize> = prefix_groups
        .values()
        .flat_map(|v| v.iter().copied())
        .collect();

    for (cluster_name, indices) in cluster_groups {
        // Skip if most members are already in a prefix group
        let overlap = indices
            .iter()
            .filter(|i| prefix_indices.contains(i))
            .count();
        if overlap > indices.len() / 2 {
            continue;
        }

        let module_name = cluster_name
            .rsplit("::")
            .next()
            .unwrap_or(cluster_name)
            .to_string();
        let group = build_group(
            results,
            indices,
            &module_name,
            "call_cluster",
            max_module_lines,
        );
        if let Some(g) = group {
            groups.push(g);
        }
    }

    // Sort by total LOC descending (biggest extraction targets first)
    groups.sort_by_key(|b| std::cmp::Reverse(b.total_loc));
    groups
}

fn build_group(
    results: &[QueryResult],
    indices: &[usize],
    name: &str,
    signal: &str,
    max_module_lines: usize,
) -> Option<ExtractionGroup> {
    let mut candidates: Vec<ExtractionCandidate> = indices
        .iter()
        .filter_map(|&i| results.get(i))
        .map(|r| ExtractionCandidate {
            function_name: r.function_name.clone(),
            file_path: r.file_path.clone(),
            start_line: r.start_line,
            loc: r.loc,
            io_classification: r.io_classification.clone(),
            io_patterns: r.io_patterns.clone(),
            complexity: r.complexity,
            tdg_grade: r.tdg_grade.clone(),
        })
        .collect();

    let total_loc: u32 = candidates.iter().map(|c| c.loc).sum();
    if total_loc as usize > max_module_lines {
        // Trim to fit within max_module_lines, keeping highest-LOC functions
        candidates.sort_by_key(|b| std::cmp::Reverse(b.loc));
        let mut running = 0u32;
        candidates.retain(|c| {
            running += c.loc;
            (running as usize) <= max_module_lines
        });
    }

    if candidates.len() < 3 {
        return None;
    }

    let pure_count = candidates
        .iter()
        .filter(|c| c.io_classification == "PURE")
        .count();
    let io_count = candidates.len() - pure_count;
    let total_loc: u32 = candidates.iter().map(|c| c.loc).sum();

    let source_file = candidates
        .first()
        .map(|c| c.file_path.clone())
        .unwrap_or_default();

    // Sort candidates by start_line for readable output
    candidates.sort_by_key(|c| c.start_line);

    Some(ExtractionGroup {
        module_name: name.to_string(),
        source_file,
        functions: candidates,
        total_loc,
        pure_count,
        io_count,
        grouping_signal: signal.to_string(),
    })
}
