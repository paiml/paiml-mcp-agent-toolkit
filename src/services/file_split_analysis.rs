// ── Analysis & Execution ───────────────────────────────────────────────────

/// Compute cohesion: ratio of internal edges to max possible edges.
fn compute_cohesion(
    local_indices: &[usize],
    index: &AgentContextIndex,
    func_indices: &[usize],
    global_to_local: &HashMap<usize, usize>,
) -> f64 {
    if local_indices.len() < 2 {
        return 1.0;
    }

    let local_set: HashSet<usize> = local_indices.iter().copied().collect();
    let mut internal_edges = 0usize;

    for &li in local_indices {
        let global_idx = func_indices[li];
        if let Some(callees) = index.calls.get(&global_idx) {
            for &callee_global in callees {
                if let Some(&callee_local) = global_to_local.get(&callee_global) {
                    if local_set.contains(&callee_local) && callee_local != li {
                        internal_edges += 1;
                    }
                }
            }
        }
    }

    let n = local_indices.len() as f64;
    let max_edges = n * (n - 1.0);
    if max_edges == 0.0 {
        return 1.0;
    }
    internal_edges as f64 / max_edges
}

/// Compute split impact: which files import this module, and which of them are
/// already in a mutual dependency with it.
///
/// A *circular risk* is a file that both calls into this file **and** is called
/// by it. The cycle already exists at file level; splitting this file spreads its
/// two halves of that cycle across separate modules, where the same mutual
/// dependency becomes a module-level import cycle that the compiler and the
/// reader both have to reckon with. Those files are the ones to look at before
/// executing the plan, so they are reported rather than left for the user to find.
fn compute_impact(index: &AgentContextIndex, file_path: &str) -> SplitImpact {
    let mut importing_files = Vec::new();

    // Scan for files that reference functions in this file
    let target_indices: HashSet<usize> = index
        .file_index
        .get(file_path)
        .map(|v| v.iter().copied().collect())
        .unwrap_or_default();

    // Files this file calls into (outgoing edges), for cycle detection below.
    let outgoing_files = outgoing_dependency_files(index, file_path, &target_indices);
    let mut circular_risks = Vec::new();

    for (other_file, other_indices) in &index.file_index {
        if other_file == file_path {
            continue;
        }
        // Check if any function in other_file calls a function in our file
        let has_dependency = other_indices.iter().any(|&gi| {
            index
                .calls
                .get(&gi)
                .map(|callees| callees.iter().any(|c| target_indices.contains(c)))
                .unwrap_or(false)
        });
        if has_dependency {
            // Incoming edge. If we also call into it, the two files form a cycle.
            if outgoing_files.contains(other_file.as_str()) {
                circular_risks.push(other_file.clone());
            }
            importing_files.push(other_file.clone());
        }
    }

    importing_files.sort();
    circular_risks.sort();

    SplitImpact {
        importing_files,
        circular_risks,
    }
}

/// Files (other than `file_path`) containing a function called by `target_indices`.
fn outgoing_dependency_files<'a>(
    index: &'a AgentContextIndex,
    file_path: &str,
    target_indices: &HashSet<usize>,
) -> HashSet<&'a str> {
    // Reverse the file → functions map once; callees are global function indices.
    let mut file_of: HashMap<usize, &str> = HashMap::new();
    for (other_file, indices) in &index.file_index {
        for &gi in indices {
            file_of.insert(gi, other_file.as_str());
        }
    }

    target_indices
        .iter()
        .filter_map(|gi| index.calls.get(gi))
        .flatten()
        .filter_map(|callee| file_of.get(callee).copied())
        .filter(|f| *f != file_path)
        .collect()
}

// ── Execute Split ──────────────────────────────────────────────────────────

/// Execute a split plan by creating new files with `include!()` pattern.
///
/// For each cluster, creates `{base}_{cluster_name}.rs` and replaces the
/// original file with `include!()` directives.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn execute_split(plan: &SplitPlan, project_root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    use std::fs;

    let source_path = project_root.join(&plan.source_file);
    let source_content = fs::read_to_string(&source_path)?;
    let source_lines: Vec<&str> = source_content.lines().collect();

    let parent_dir = source_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("No parent directory for {}", plan.source_file))?;
    let stem = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid file stem for {}", plan.source_file))?;

    let mut created_files = Vec::new();
    let mut include_directives = Vec::new();

    for cluster in &plan.clusters {
        let cluster_filename = format!("{}_{}.rs", stem, cluster.suggested_name);
        let cluster_path = parent_dir.join(&cluster_filename);

        // Extract lines for this cluster
        let mut cluster_lines = Vec::new();
        let mut ranges: Vec<(usize, usize)> = cluster
            .items
            .iter()
            .map(|item| (item.line_range.0, item.line_range.1))
            .collect();
        ranges.sort_by_key(|r| r.0);

        for (start, end) in &ranges {
            // Lines are 1-indexed in FunctionEntry
            let start_idx = start.saturating_sub(1);
            let total_lines = source_lines.len();
            let end_idx = (*end).min(total_lines);
            for line in &source_lines[start_idx..end_idx] {
                cluster_lines.push(*line);
            }
            cluster_lines.push(""); // blank line separator
        }

        let cluster_content = cluster_lines.join("\n");
        fs::write(&cluster_path, &cluster_content)?;
        created_files.push(cluster_path);

        include_directives.push(format!("include!(\"{}\");", cluster_filename));
    }

    // Note: We don't rewrite the original file automatically — that requires
    // careful handling of use statements, module-level attributes, etc.
    // The user should review the generated files and update the source manually.

    Ok(created_files)
}
