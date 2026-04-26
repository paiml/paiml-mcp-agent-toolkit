#![cfg_attr(coverage_nightly, coverage(off))]

use std::collections::{HashMap, HashSet};

use trueno_graph::{pagerank, CsrGraph, NodeId};

use super::types::{DepAnalysis, DepEdge, GraphAnalysis};

/// Build and analyze dependency graph using trueno-graph
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

#[cfg(test)]
mod graph_tests {
    //! Covers analyze_dependency_graph + count_transitive_deps + apply_graph_analysis
    //! in deps_audit_handlers/graph.rs (2 uncov on broad, 0% cov).
    use super::*;
    use crate::cli::handlers::deps_audit_handlers::types::DepCategory;

    fn dep(name: &str) -> DepAnalysis {
        DepAnalysis {
            name: name.to_string(),
            version: "1.0".to_string(),
            category: DepCategory::Core,
            replacement: None,
            reason: String::new(),
            transitive_count: 0,
            estimated_size_kb: 0,
            pagerank_score: 0.0,
            in_degree: 0,
            out_degree: 0,
            is_bridge: false,
            is_orphan: false,
        }
    }

    fn edge(from: &str, to: &str) -> DepEdge {
        DepEdge {
            from: from.to_string(),
            to: to.to_string(),
        }
    }

    // ── analyze_dependency_graph ──

    #[test]
    fn test_analyze_dependency_graph_empty_inputs() {
        let analysis = analyze_dependency_graph(&[], &[], &[]);
        assert!(analysis.pagerank_scores.is_empty());
        assert!(analysis.in_degrees.is_empty());
        assert!(analysis.out_degrees.is_empty());
    }

    #[test]
    fn test_analyze_dependency_graph_single_node_no_edges() {
        let direct = vec!["foo".to_string()];
        let all = vec!["foo".to_string()];
        let analysis = analyze_dependency_graph(&direct, &all, &[]);
        // Single isolated node → orphan.
        assert!(analysis.orphans.contains("foo"));
    }

    #[test]
    fn test_analyze_dependency_graph_with_edges_populates_degrees() {
        let direct = vec!["a".to_string()];
        let all = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let edges = vec![edge("a", "b"), edge("b", "c")];
        let analysis = analyze_dependency_graph(&direct, &all, &edges);
        // a → b → c: out_degrees(a)=1, out_degrees(b)=1, in_degrees(b)=1, in_degrees(c)=1.
        assert_eq!(*analysis.out_degrees.get("a").unwrap_or(&0), 1);
        assert_eq!(*analysis.out_degrees.get("b").unwrap_or(&0), 1);
        assert_eq!(*analysis.in_degrees.get("b").unwrap_or(&0), 1);
        assert_eq!(*analysis.in_degrees.get("c").unwrap_or(&0), 1);
    }

    #[test]
    fn test_analyze_dependency_graph_unknown_edge_endpoints_dropped() {
        // Edge points to a name not in all_packages → silently dropped.
        let direct = vec!["a".to_string()];
        let all = vec!["a".to_string()];
        let edges = vec![edge("a", "missing")];
        let analysis = analyze_dependency_graph(&direct, &all, &edges);
        // No degrees recorded since the edge couldn't be inserted.
        assert_eq!(*analysis.out_degrees.get("a").unwrap_or(&0), 0);
    }

    // ── count_transitive_deps ──

    #[test]
    fn test_count_transitive_deps_unknown_start_returns_zero() {
        let name_to_id = HashMap::new();
        let id_to_name = HashMap::new();
        let n = count_transitive_deps("missing", &[], &name_to_id, &id_to_name);
        assert_eq!(n, 0);
    }

    #[test]
    fn test_count_transitive_deps_isolated_node_returns_zero() {
        let mut name_to_id = HashMap::new();
        name_to_id.insert("a".to_string(), NodeId(0));
        let mut id_to_name = HashMap::new();
        id_to_name.insert(NodeId(0), "a".to_string());
        let n = count_transitive_deps("a", &[], &name_to_id, &id_to_name);
        // No edges → only "a" is reachable, but we don't count self.
        assert_eq!(n, 0);
    }

    #[test]
    fn test_count_transitive_deps_chain_three_nodes() {
        let mut name_to_id = HashMap::new();
        name_to_id.insert("a".to_string(), NodeId(0));
        name_to_id.insert("b".to_string(), NodeId(1));
        name_to_id.insert("c".to_string(), NodeId(2));
        let mut id_to_name = HashMap::new();
        id_to_name.insert(NodeId(0), "a".to_string());
        id_to_name.insert(NodeId(1), "b".to_string());
        id_to_name.insert(NodeId(2), "c".to_string());
        // a → b → c. Transitive count from a = 2 (b + c).
        let edges = vec![(NodeId(0), NodeId(1), 1.0), (NodeId(1), NodeId(2), 1.0)];
        let n = count_transitive_deps("a", &edges, &name_to_id, &id_to_name);
        assert_eq!(n, 2);
    }

    #[test]
    fn test_count_transitive_deps_diamond_no_double_count() {
        // a → b, a → c, b → d, c → d. Transitive from a = 3 (b, c, d).
        let mut name_to_id = HashMap::new();
        for (i, n) in ["a", "b", "c", "d"].iter().enumerate() {
            name_to_id.insert((*n).to_string(), NodeId(i as u32));
        }
        let mut id_to_name = HashMap::new();
        for (i, n) in ["a", "b", "c", "d"].iter().enumerate() {
            id_to_name.insert(NodeId(i as u32), (*n).to_string());
        }
        let edges = vec![
            (NodeId(0), NodeId(1), 1.0), // a → b
            (NodeId(0), NodeId(2), 1.0), // a → c
            (NodeId(1), NodeId(3), 1.0), // b → d
            (NodeId(2), NodeId(3), 1.0), // c → d
        ];
        let n = count_transitive_deps("a", &edges, &name_to_id, &id_to_name);
        assert_eq!(n, 3, "diamond shouldn't double-count d");
    }

    // ── apply_graph_analysis ──

    #[test]
    fn test_apply_graph_analysis_populates_metrics_from_analysis() {
        let mut deps = vec![dep("foo"), dep("bar")];

        let mut pagerank_scores = HashMap::new();
        pagerank_scores.insert("foo".to_string(), 0.42);
        let mut in_degrees = HashMap::new();
        in_degrees.insert("foo".to_string(), 3);
        let mut out_degrees = HashMap::new();
        out_degrees.insert("foo".to_string(), 5);
        let mut bridges = HashSet::new();
        bridges.insert("foo".to_string());
        let mut orphans = HashSet::new();
        orphans.insert("bar".to_string());
        let mut transitive_counts = HashMap::new();
        transitive_counts.insert("foo".to_string(), 17);

        let analysis = GraphAnalysis {
            pagerank_scores,
            in_degrees,
            out_degrees,
            bridges,
            orphans,
            transitive_counts,
        };

        apply_graph_analysis(&mut deps, &analysis);

        assert!((deps[0].pagerank_score - 0.42).abs() < 1e-6);
        assert_eq!(deps[0].in_degree, 3);
        assert_eq!(deps[0].out_degree, 5);
        assert!(deps[0].is_bridge);
        assert!(!deps[0].is_orphan);
        assert_eq!(deps[0].transitive_count, 17);

        // bar has no entries in any map except orphans.
        assert!(deps[1].is_orphan);
        assert_eq!(deps[1].pagerank_score, 0.0);
        assert_eq!(deps[1].in_degree, 0);
    }

    #[test]
    fn test_apply_graph_analysis_empty_deps_no_panic() {
        let mut deps: Vec<DepAnalysis> = vec![];
        let analysis = GraphAnalysis {
            pagerank_scores: HashMap::new(),
            in_degrees: HashMap::new(),
            out_degrees: HashMap::new(),
            bridges: HashSet::new(),
            orphans: HashSet::new(),
            transitive_counts: HashMap::new(),
        };
        apply_graph_analysis(&mut deps, &analysis);
    }
}
