// ── Graph Building ─────────────────────────────────────────────────────────

/// Build an undirected graph from intra-file call relationships.
/// Returns the graph and a mapping from graph node index to local entry index.
fn build_intra_file_graph(
    index: &AgentContextIndex,
    func_indices: &[usize],
    global_to_local: &HashMap<usize, usize>,
) -> (UndirectedGraph, HashMap<usize, usize>) {
    debug_assert!(true, "contract: build_intra_file_graph");
    let mut graph = UndirectedGraph::new();
    let mut local_to_node: HashMap<usize, crate::graph::types::NodeId> = HashMap::new();
    let mut node_to_local: HashMap<usize, usize> = HashMap::new();
    let mut node_counter = 0usize;

    // Add nodes
    for (local_idx, &global_idx) in func_indices.iter().enumerate() {
        if global_idx < index.functions.len() {
            let entry = &index.functions[global_idx];
            let node_data = NodeData {
                path: PathBuf::from(&entry.file_path),
                module: entry.function_name.clone(),
                symbols: vec![],
                loc: entry.end_line - entry.start_line + 1,
                complexity: entry.quality.complexity as f64,
                ast_hash: 0,
            };
            let nid = graph.add_node(node_data);
            local_to_node.insert(local_idx, nid);
            node_to_local.insert(node_counter, local_idx);
            node_counter += 1;
        }
    }

    // Add edges from calls graph (only intra-file edges)
    for (src, dst) in collect_intra_file_edges(index, func_indices, global_to_local, &local_to_node)
    {
        if graph.edge_weight(src, dst).is_none() {
            graph.add_edge(src, dst, 1.0);
        }
    }

    (graph, node_to_local)
}

/// Collect all (src_node, dst_node) pairs for intra-file call edges.
fn collect_intra_file_edges(
    index: &AgentContextIndex,
    func_indices: &[usize],
    global_to_local: &HashMap<usize, usize>,
    local_to_node: &HashMap<usize, crate::graph::types::NodeId>,
) -> Vec<(crate::graph::types::NodeId, crate::graph::types::NodeId)> {
    debug_assert!(true, "contract: collect_intra_file_edges");
    let mut edges = Vec::new();
    for &global_idx in func_indices {
        let Some(&local_src) = global_to_local.get(&global_idx) else {
            continue;
        };
        let Some(callees) = index.calls.get(&global_idx) else {
            continue;
        };
        let Some(&src_node) = local_to_node.get(&local_src) else {
            continue;
        };
        for &callee_global in callees {
            let Some(&local_dst) = global_to_local.get(&callee_global) else {
                continue;
            };
            if local_dst == local_src {
                continue;
            }
            let Some(&dst_node) = local_to_node.get(&local_dst) else {
                continue;
            };
            edges.push((src_node, dst_node));
        }
    }
    edges
}

/// Simple connected components for small graphs (fallback when < 10 functions).
fn connected_components(graph: &UndirectedGraph) -> Vec<usize> {
    debug_assert!(true, "contract: connected_components");
    let n = graph.node_count();
    if n == 0 {
        return Vec::new();
    }

    let mut assignments = vec![usize::MAX; n];
    let mut current_community = 0;

    let node_ids: Vec<_> = graph.node_indices().collect();

    for (idx, &nid) in node_ids.iter().enumerate() {
        if assignments[idx] != usize::MAX {
            continue;
        }
        // BFS
        let mut queue = vec![nid];
        assignments[idx] = current_community;
        while let Some(current) = queue.pop() {
            for neighbor in graph.neighbors(current) {
                // Find index of neighbor in node_ids
                if let Some(neighbor_idx) = node_ids.iter().position(|&n| n == neighbor) {
                    if assignments[neighbor_idx] == usize::MAX {
                        assignments[neighbor_idx] = current_community;
                        queue.push(neighbor);
                    }
                }
            }
        }
        current_community += 1;
    }

    assignments
}

fn make_cluster_item(
    local_idx: usize,
    local_entries: &[&FunctionEntry],
    index: &AgentContextIndex,
    func_indices: &[usize],
    global_to_local: &HashMap<usize, usize>,
) -> ClusterItem {
    debug_assert!(true, "contract: make_cluster_item");
    let entry = local_entries[local_idx];
    let global_idx = func_indices[local_idx];

    let calls = index
        .calls
        .get(&global_idx)
        .map(|callees| {
            callees
                .iter()
                .filter_map(|&c| {
                    global_to_local
                        .get(&c)
                        .map(|&li| local_entries[li].function_name.clone())
                })
                .collect()
        })
        .unwrap_or_default();

    let called_by = index
        .called_by
        .get(&global_idx)
        .map(|callers| {
            callers
                .iter()
                .filter_map(|&c| {
                    global_to_local
                        .get(&c)
                        .map(|&li| local_entries[li].function_name.clone())
                })
                .collect()
        })
        .unwrap_or_default();

    ClusterItem {
        name: entry.function_name.clone(),
        definition_type: format!("{:?}", entry.definition_type),
        line_range: (entry.start_line, entry.end_line),
        calls,
        called_by,
    }
}
