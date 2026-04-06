/// Count connected components using aprender's SIMD-accelerated algorithm.
///
/// # Arguments
/// * `graph` - PMAT dependency graph (treated as undirected)
///
/// # Returns
/// Number of connected components
///
/// # Performance
/// O(V + E) with SIMD acceleration
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn connected_components(graph: &DependencyGraph) -> usize {
    if graph.node_count() == 0 {
        return 0;
    }

    let aprender_graph = to_aprender_graph(graph, false); // undirected for components
    let labels = aprender_graph.connected_components();

    if labels.is_empty() {
        return 0;
    }

    // Count unique component labels
    let max_label = labels.iter().copied().max().unwrap_or(0);
    max_label + 1
}

/// Compute strongly connected components using aprender's algorithm.
///
/// # Arguments
/// * `graph` - PMAT dependency graph (directed)
///
/// # Returns
/// Vector of component labels (nodes with same label are in same SCC)
///
/// # Note
/// Replaces petgraph::algo::kosaraju_scc with aprender's SIMD version
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn strongly_connected_components(graph: &DependencyGraph) -> Vec<usize> {
    if graph.node_count() == 0 {
        return Vec::new();
    }

    let aprender_graph = to_aprender_graph(graph, true);
    aprender_graph.strongly_connected_components()
}

/// Check if directed graph has cycles using aprender's topological sort.
///
/// # Arguments
/// * `graph` - PMAT dependency graph
///
/// # Returns
/// true if graph has cycles, false if acyclic
///
/// # Algorithm
/// Uses topological sort - if sort fails, graph has cycles
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn is_cyclic(graph: &DependencyGraph) -> bool {
    if graph.node_count() == 0 {
        return false;
    }

    let aprender_graph = to_aprender_graph(graph, true);
    aprender_graph.topological_sort().is_none()
}

/// Compute shortest path using aprender's dijkstra algorithm.
///
/// # Arguments
/// * `graph` - PMAT dependency graph
/// * `source` - Source node index
/// * `target` - Target node index
///
/// # Returns
/// Some((path, distance)) if path exists, None otherwise
///
/// # Performance
/// SIMD-accelerated for cache efficiency
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn shortest_path(
    graph: &DependencyGraph,
    source: usize,
    target: usize,
) -> Option<(Vec<usize>, f64)> {
    if graph.node_count() == 0 {
        return None;
    }

    let aprender_graph = to_aprender_graph(graph, true);
    aprender_graph.dijkstra(source, target)
}

/// Compute betweenness centrality for all nodes.
///
/// # Arguments
/// * `graph` - PMAT dependency graph
///
/// # Returns
/// Betweenness centrality scores for each node
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn betweenness_centrality(graph: &DependencyGraph) -> Vec<f64> {
    if graph.node_count() == 0 {
        return Vec::new();
    }

    let aprender_graph = to_aprender_graph(graph, true);
    aprender_graph.betweenness_centrality()
}

/// Run Louvain community detection.
///
/// # Arguments
/// * `graph` - PMAT undirected graph
///
/// # Returns
/// Vector of communities, each community is a vector of node IDs
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn louvain_communities(graph: &UndirectedGraph) -> Vec<Vec<usize>> {
    if graph.node_count() == 0 {
        return Vec::new();
    }

    let aprender_graph = to_aprender_graph_undirected(graph);
    aprender_graph.louvain()
}
