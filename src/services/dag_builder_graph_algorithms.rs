// Graph algorithms: filtering, PageRank scoring, and graph pruning

/// Filters dependency graph to only include call edges
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::dag_builder::filter_call_edges;
/// use pmat::models::dag::{DependencyGraph, EdgeType};
///
/// let graph = DependencyGraph::new();
/// let filtered = filter_call_edges(graph);
/// // All edges in filtered graph will be EdgeType::Calls
/// ```
#[must_use]
pub fn filter_call_edges(graph: DependencyGraph) -> DependencyGraph {
    graph.filter_by_edge_type(EdgeType::Calls)
}

/// Filters dependency graph to only include import edges
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::dag_builder::filter_import_edges;
/// use pmat::models::dag::{DependencyGraph, EdgeType};
///
/// let graph = DependencyGraph::new();
/// let filtered = filter_import_edges(graph);
/// // All edges in filtered graph will be EdgeType::Imports
/// ```
#[must_use]
pub fn filter_import_edges(graph: DependencyGraph) -> DependencyGraph {
    graph.filter_by_edge_type(EdgeType::Imports)
}

/// Filters dependency graph to only include inheritance edges
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::dag_builder::filter_inheritance_edges;
/// use pmat::models::dag::{DependencyGraph, EdgeType};
///
/// let graph = DependencyGraph::new();
/// let filtered = filter_inheritance_edges(graph);
/// // All edges in filtered graph will be EdgeType::Inherits
/// ```
#[must_use]
pub fn filter_inheritance_edges(graph: DependencyGraph) -> DependencyGraph {
    graph.filter_by_edge_type(EdgeType::Inherits)
}

/// Add `PageRank` scores to all nodes in the graph
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::dag_builder::add_pagerank_scores;
/// use pmat::models::dag::DependencyGraph;
///
/// let graph = DependencyGraph::new();
/// let scored_graph = add_pagerank_scores(&graph);
/// // Graph nodes now have PageRank scores in metadata
/// ```
#[must_use]
pub fn add_pagerank_scores(graph: &DependencyGraph) -> DependencyGraph {
    if graph.nodes.is_empty() {
        return graph.clone();
    }

    let scores = compute_pagerank_scores(graph);

    // Create a new graph with centrality scores in metadata
    let node_ids: Vec<&String> = graph.nodes.keys().collect();
    let mut new_graph = graph.clone();
    for (id, node) in &mut new_graph.nodes {
        if let Some(idx) = node_ids.iter().position(|&nid| nid == id) {
            node.metadata
                .insert("centrality".to_string(), scores[idx].to_string());
        }
    }

    new_graph
}

/// Prune graph using `PageRank` algorithm to keep only the most important nodes
#[must_use]
pub fn prune_graph_pagerank(graph: &DependencyGraph, max_nodes: usize) -> DependencyGraph {
    if graph.nodes.len() <= max_nodes {
        return graph.clone();
    }

    let scores = compute_pagerank_scores(graph);
    let node_ids: Vec<&String> = graph.nodes.keys().collect();

    // Select top-k nodes
    let mut ranked: Vec<_> = scores.iter().enumerate().collect();
    ranked.sort_by(|a, b| b.1.total_cmp(a.1));
    let keep: FxHashSet<&String> = ranked
        .iter()
        .take(max_nodes)
        .map(|(i, _)| node_ids[*i])
        .collect();

    // Create nodes with centrality scores in metadata
    let mut nodes_with_centrality = FxHashMap::default();
    for (id, node) in &graph.nodes {
        if keep.contains(id) {
            let mut node_copy = node.clone();
            if let Some(idx) = node_ids.iter().position(|&nid| nid == id) {
                node_copy
                    .metadata
                    .insert("centrality".to_string(), scores[idx].to_string());
            }
            nodes_with_centrality.insert(id.clone(), node_copy);
        }
    }

    DependencyGraph {
        nodes: nodes_with_centrality,
        edges: graph
            .edges
            .iter()
            .filter(|e| keep.contains(&e.from) && keep.contains(&e.to))
            .cloned()
            .collect(),
    }
}

/// Shared PageRank computation used by both `add_pagerank_scores` and `prune_graph_pagerank`
fn compute_pagerank_scores(graph: &DependencyGraph) -> Vec<f32> {
    let node_ids: Vec<&String> = graph.nodes.keys().collect();
    let mut scores = vec![1.0f32; node_ids.len()];
    let node_idx: FxHashMap<&String, usize> = node_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    // Precompute out-degrees for O(1) lookup
    let mut out_degrees = vec![0u32; node_ids.len()];
    for edge in &graph.edges {
        if let Some(&from) = node_idx.get(&edge.from) {
            out_degrees[from] += 1;
        }
    }

    // 10 iterations sufficient for ranking
    for _ in 0..10 {
        let mut new_scores = vec![0.15f32; scores.len()];
        for edge in &graph.edges {
            if let (Some(&from), Some(&to)) = (node_idx.get(&edge.from), node_idx.get(&edge.to)) {
                let out_degree = out_degrees[from] as f32;
                if out_degree > 0.0 {
                    new_scores[to] += 0.85 * scores[from] / out_degree;
                }
            }
        }
        scores = new_scores;
    }

    scores
}
