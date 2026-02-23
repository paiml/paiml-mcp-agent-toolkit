#![cfg_attr(coverage_nightly, coverage(off))]

use std::collections::{HashMap, HashSet};

use trueno_graph::{pagerank, CsrGraph, NodeId};

use super::types::{DepAnalysis, DepEdge, GraphAnalysis};

/// Build and analyze dependency graph using trueno-graph
pub fn analyze_dependency_graph(
    direct_deps: &[String],
    all_packages: &[String],
    edges: &[DepEdge],
) -> GraphAnalysis {
    let mut name_to_id: HashMap<String, NodeId> = HashMap::new();
    let mut id_to_name: HashMap<NodeId, String> = HashMap::new();

    // Assign node IDs
    for (i, name) in all_packages.iter().enumerate() {
        let node_id = NodeId(i as u32);
        name_to_id.insert(name.clone(), node_id);
        id_to_name.insert(node_id, name.clone());
    }

    // Build edge list for CSR graph (with weights = 1.0)
    let mut edge_list: Vec<(NodeId, NodeId, f32)> = Vec::new();
    let mut in_degrees: HashMap<String, usize> = HashMap::new();
    let mut out_degrees: HashMap<String, usize> = HashMap::new();

    for edge in edges {
        if let (Some(&from_id), Some(&to_id)) =
            (name_to_id.get(&edge.from), name_to_id.get(&edge.to))
        {
            edge_list.push((from_id, to_id, 1.0)); // Default weight = 1.0
            *out_degrees.entry(edge.from.clone()).or_insert(0) += 1;
            *in_degrees.entry(edge.to.clone()).or_insert(0) += 1;
        }
    }

    // Build CSR graph
    let num_nodes = all_packages.len();
    let graph = CsrGraph::from_edge_list(&edge_list).unwrap_or_else(|_| CsrGraph::new());

    // Calculate PageRank
    let pagerank_vec = pagerank(&graph, 20, 1e-6)
        .unwrap_or_else(|_| vec![1.0 / num_nodes.max(1) as f32; num_nodes]);
    let mut pagerank_scores: HashMap<String, f32> = HashMap::new();
    for (i, &score) in pagerank_vec.iter().enumerate() {
        if let Some(name) = id_to_name.get(&NodeId(i as u32)) {
            pagerank_scores.insert(name.clone(), score);
        }
    }

    // Find orphans (direct deps that nothing else depends on)
    let mut orphans = HashSet::new();
    let direct_set: HashSet<_> = direct_deps.iter().cloned().collect();
    for dep in direct_deps {
        if in_degrees.get(dep).copied().unwrap_or(0) == 0 {
            // No other package depends on this
            if direct_set.contains(dep) {
                orphans.insert(dep.clone());
            }
        }
    }

    // Find bridges (deps that connect otherwise unconnected parts)
    // Simple heuristic: deps with high betweenness = both in and out degree > 0
    // and are the only path between their dependents and dependencies
    let mut bridges = HashSet::new();
    for (name, &out_deg) in &out_degrees {
        let in_deg = in_degrees.get(name).copied().unwrap_or(0);
        // High connectivity on both sides suggests bridge
        if in_deg >= 2 && out_deg >= 3 {
            bridges.insert(name.clone());
        }
    }

    // Calculate transitive dependency counts (how many deps does each direct dep bring)
    let mut transitive_counts: HashMap<String, usize> = HashMap::new();
    for dep in direct_deps {
        let count = count_transitive_deps(dep, &edge_list, &name_to_id, &id_to_name);
        transitive_counts.insert(dep.clone(), count);
    }

    GraphAnalysis {
        pagerank_scores,
        in_degrees,
        out_degrees,
        bridges,
        orphans,
        transitive_counts,
    }
}

/// Count transitive dependencies via BFS
pub fn count_transitive_deps(
    start: &str,
    edges: &[(NodeId, NodeId, f32)],
    name_to_id: &HashMap<String, NodeId>,
    _id_to_name: &HashMap<NodeId, String>,
) -> usize {
    let Some(&start_id) = name_to_id.get(start) else {
        return 0;
    };

    // Build adjacency list (ignoring weights for traversal)
    let mut adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for &(from, to, _weight) in edges {
        adj.entry(from).or_default().push(to);
    }

    // BFS to count reachable nodes
    let mut visited = HashSet::new();
    let mut queue = vec![start_id];
    visited.insert(start_id);

    while let Some(node) = queue.pop() {
        if let Some(neighbors) = adj.get(&node) {
            for &neighbor in neighbors {
                if visited.insert(neighbor) {
                    queue.push(neighbor);
                }
            }
        }
    }

    // Don't count self
    visited.len().saturating_sub(1)
}

/// Apply graph analysis to dependency list
pub fn apply_graph_analysis(deps: &mut [DepAnalysis], analysis: &GraphAnalysis) {
    for dep in deps.iter_mut() {
        dep.pagerank_score = *analysis.pagerank_scores.get(&dep.name).unwrap_or(&0.0);
        dep.in_degree = *analysis.in_degrees.get(&dep.name).unwrap_or(&0);
        dep.out_degree = *analysis.out_degrees.get(&dep.name).unwrap_or(&0);
        dep.is_bridge = analysis.bridges.contains(&dep.name);
        dep.is_orphan = analysis.orphans.contains(&dep.name);
        dep.transitive_count = *analysis.transitive_counts.get(&dep.name).unwrap_or(&0);
    }
}
