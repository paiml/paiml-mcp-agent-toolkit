/// Convert PMAT's DependencyGraph to aprender's Graph format.
///
/// # Arguments
/// * `graph` - PMAT dependency graph (petgraph::DiGraph)
/// * `is_directed` - Whether to treat as directed graph
///
/// # Returns
/// aprender Graph with edge list representation
///
/// # Example
/// ```ignore
/// let aprender_graph = to_aprender_graph(&dependency_graph, true);
/// let pagerank = aprender_graph.pagerank(0.85, 100, 1e-6)?;
/// ```
pub fn to_aprender_graph(graph: &DependencyGraph, is_directed: bool) -> AprenderGraph {
    // Build edge list from trueno-graph
    let mut edges = Vec::new();

    for edge in graph.edge_references() {
        let source = edge.source().0 as usize;
        let target = edge.target().0 as usize;

        edges.push((source, target));
    }

    // Create aprender graph from edge list
    AprenderGraph::from_edges(&edges, is_directed)
}

/// Convert PMAT's UndirectedGraph to aprender's Graph format.
///
/// # Arguments
/// * `graph` - PMAT undirected graph (petgraph::UnGraph)
///
/// # Returns
/// aprender Graph (undirected)
///
/// # Example
/// ```ignore
/// let aprender_graph = to_aprender_graph_undirected(&undirected_graph);
/// let communities = aprender_graph.louvain();
/// ```
pub fn to_aprender_graph_undirected(graph: &UndirectedGraph) -> AprenderGraph {
    // Build edge list from trueno-graph
    let mut edges = Vec::new();

    for edge in graph.edge_references() {
        let source = edge.source().0 as usize;
        let target = edge.target().0 as usize;

        edges.push((source, target));
    }

    // Create aprender graph from edge list (undirected)
    AprenderGraph::from_edges(&edges, false)
}

/// Extract edge weight from EdgeData enum.
///
/// Different edge types have different weight semantics:
/// - Import: direct weight field
/// - FunctionCall: count as weight
/// - TypeDependency: strength as weight
/// - DataFlow: confidence as weight
/// - Inheritance: inverse of depth (closer = higher weight)
pub fn extract_edge_weight(edge_data: &EdgeData) -> f64 {
    match edge_data {
        EdgeData::Import { weight, .. } => *weight,
        EdgeData::FunctionCall { count, .. } => *count as f64,
        EdgeData::TypeDependency { strength, .. } => *strength,
        EdgeData::DataFlow { confidence, .. } => *confidence,
        EdgeData::Inheritance { depth } => {
            if *depth == 0 {
                1.0
            } else {
                1.0 / (*depth as f64)
            }
        }
    }
}

/// Create node ID mapping for aprender graph.
///
/// aprender uses contiguous node IDs starting from 0.
/// PMAT's trueno-graph nodes may have gaps, so we need a mapping.
pub fn create_node_mapping(graph: &DependencyGraph) -> Vec<usize> {
    graph
        .node_references()
        .map(|(idx, _)| idx.0 as usize)
        .collect()
}
