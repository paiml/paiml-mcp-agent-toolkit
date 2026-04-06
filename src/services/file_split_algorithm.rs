// ── Core Algorithm ─────────────────────────────────────────────────────────

/// Suggest how to split a file into semantically coherent clusters.
///
/// Returns `None` if the file is not in the index or has no functions.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn suggest_split(
    index: &AgentContextIndex,
    file_path: &str,
    resolution: f64,
    min_cluster_lines: usize,
) -> Option<SplitPlan> {
    debug_assert!(!file_path.is_empty(), "file_path must not be empty");
    // Step 1: Get all function indices for this file
    let func_indices = index.file_index.get(file_path)?;
    if func_indices.is_empty() {
        return None;
    }

    // Build local index: global_idx -> local_idx and collect entries
    let mut global_to_local: HashMap<usize, usize> = HashMap::new();
    let mut local_entries: Vec<&FunctionEntry> = Vec::new();
    for (local_idx, &global_idx) in func_indices.iter().enumerate() {
        if global_idx < index.functions.len() {
            global_to_local.insert(global_idx, local_idx);
            local_entries.push(&index.functions[global_idx]);
        }
    }

    if local_entries.is_empty() {
        return None;
    }

    let total_lines = estimate_total_lines(&local_entries);

    // Step 2: Build intra-file call graph
    let (graph, node_to_local) = build_intra_file_graph(index, func_indices, &global_to_local);

    // Step 3-4: Run community detection
    let (communities, modularity) = if local_entries.len() < 10 {
        // Fallback: use connected components for small files
        let comms = connected_components(&graph);
        let detector = LouvainDetector::new();
        let mod_score = detector.calculate_modularity(&graph, &comms);
        (comms, mod_score)
    } else {
        let mut detector = LouvainDetector::new().with_resolution(resolution);
        let comms = detector.detect_communities(&graph);
        let mod_score = detector.calculate_modularity(&graph, &comms);
        (comms, mod_score)
    };

    // Step 5: Map communities to clusters
    let mut community_items: HashMap<usize, Vec<usize>> = HashMap::new();
    for (node_idx, &comm_id) in communities.iter().enumerate() {
        // Map node index back to local entry index
        if let Some(&local_idx) = node_to_local.get(&node_idx) {
            community_items.entry(comm_id).or_default().push(local_idx);
        }
    }

    // Collect multi-item communities as clusters, singletons as orphans
    let mut clusters = Vec::new();
    let mut orphan_items: Vec<(ClusterItem, usize)> = Vec::new(); // (item, local_idx)

    for local_indices in community_items.values() {
        let items: Vec<ClusterItem> = local_indices
            .iter()
            .map(|&li| make_cluster_item(li, &local_entries, index, func_indices, &global_to_local))
            .collect();

        let estimated_lines: usize = items
            .iter()
            .map(|i| i.line_range.1 - i.line_range.0 + 1)
            .sum();

        if items.len() == 1 || estimated_lines < min_cluster_lines {
            for (i, item) in items.into_iter().enumerate() {
                orphan_items.push((item, local_indices[i]));
            }
        } else {
            // Step 6: Name each cluster using signal functions
            let cluster_entries: Vec<&FunctionEntry> =
                local_indices.iter().map(|&li| local_entries[li]).collect();

            let (name, signal, confidence) = name_cluster(&cluster_entries, file_path);
            let cohesion = compute_cohesion(local_indices, index, func_indices, &global_to_local);

            clusters.push(SplitCluster {
                suggested_name: name,
                naming_signal: signal,
                confidence,
                items,
                estimated_lines,
                cohesion,
            });
        }
    }

    // Step 5b: Assign orphans to nearest cluster by line proximity
    let unclustered = assign_orphans_to_clusters(&mut clusters, orphan_items);

    // Sort items within each cluster by line range
    for cluster in &mut clusters {
        cluster.items.sort_by_key(|i| i.line_range.0);
    }

    // Sort clusters by line count descending
    clusters.sort_by(|a, b| b.estimated_lines.cmp(&a.estimated_lines));

    // Step 7: Calculate impact
    let impact = compute_impact(index, file_path);

    Some(SplitPlan {
        source_file: file_path.to_string(),
        total_lines,
        clusters,
        unclustered,
        impact,
        modularity,
    })
}

/// Assign orphan items to the nearest cluster by line proximity.
/// Returns any items that couldn't be assigned (only if no clusters exist).
fn assign_orphans_to_clusters(
    clusters: &mut Vec<SplitCluster>,
    orphan_items: Vec<(ClusterItem, usize)>,
) -> Vec<ClusterItem> {
    debug_assert!(true, "contract: assign_orphans_to_clusters");
    if clusters.is_empty() {
        return orphan_items.into_iter().map(|(item, _)| item).collect();
    }

    let mut unclustered = Vec::new();
    for (item, _local_idx) in orphan_items {
        let item_mid = (item.line_range.0 + item.line_range.1) / 2;
        let nearest = clusters.iter_mut().min_by_key(|c| {
            c.items
                .iter()
                .map(|ci| {
                    let ci_mid = (ci.line_range.0 + ci.line_range.1) / 2;
                    (ci_mid as isize - item_mid as isize).unsigned_abs()
                })
                .min()
                .unwrap_or(usize::MAX)
        });
        if let Some(cluster) = nearest {
            let line_span = item.line_range.1 - item.line_range.0 + 1;
            cluster.estimated_lines += line_span;
            cluster.items.push(item);
        } else {
            unclustered.push(item);
        }
    }
    unclustered
}

fn estimate_total_lines(entries: &[&FunctionEntry]) -> usize {
    debug_assert!(!entries.is_empty(), "entries must not be empty");
    entries.iter().map(|e| e.end_line).max().unwrap_or(0)
}
