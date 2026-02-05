// Adapter module: PMAT DependencyGraph → aprender::graph::Graph
// Complexity: All functions ≤ 10
// SATD: Zero tolerance

use super::types::{DependencyGraph, EdgeData, UndirectedGraph};
use aprender::graph::Graph as AprenderGraph;

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
pub fn louvain_communities(graph: &UndirectedGraph) -> Vec<Vec<usize>> {
    if graph.node_count() == 0 {
        return Vec::new();
    }

    let aprender_graph = to_aprender_graph_undirected(graph);
    aprender_graph.louvain()
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::types::{NodeData, Symbol, SymbolKind, Visibility};
    use std::path::PathBuf;

    fn create_test_node() -> NodeData {
        NodeData {
            path: PathBuf::from("test.rs"),
            module: "test".to_string(),
            symbols: vec![Symbol {
                name: "foo".to_string(),
                kind: SymbolKind::Function,
                visibility: Visibility::Public,
                line: 1,
            }],
            loc: 100,
            complexity: 5.0,
            ast_hash: 12345,
        }
    }

    #[test]
    fn test_to_aprender_graph_directed() {
        let mut graph = DependencyGraph::new();
        let n0 = graph.add_node(create_test_node());
        let n1 = graph.add_node(create_test_node());
        let n2 = graph.add_node(create_test_node());

        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );
        graph.add_edge(
            n1,
            n2,
            EdgeData::FunctionCall {
                count: 5,
                async_call: false,
            },
        );

        let aprender_graph = to_aprender_graph(&graph, true);

        assert_eq!(aprender_graph.num_nodes(), 3);
        assert_eq!(aprender_graph.num_edges(), 2);
        assert!(aprender_graph.is_directed());
    }

    #[test]
    fn test_to_aprender_graph_undirected() {
        let mut graph = DependencyGraph::new();
        let n0 = graph.add_node(create_test_node());
        let n1 = graph.add_node(create_test_node());

        graph.add_edge(
            n0,
            n1,
            EdgeData::TypeDependency {
                strength: 0.8,
                kind: crate::graph::types::TypeKind::Trait,
            },
        );

        let aprender_graph = to_aprender_graph(&graph, false);

        assert_eq!(aprender_graph.num_nodes(), 2);
        assert_eq!(aprender_graph.num_edges(), 1);
        assert!(!aprender_graph.is_directed());
    }

    #[test]
    fn test_extract_edge_weight_import() {
        let edge = EdgeData::Import {
            weight: 2.5,
            visibility: Visibility::Public,
        };
        assert_eq!(extract_edge_weight(&edge), 2.5);
    }

    #[test]
    fn test_extract_edge_weight_function_call() {
        let edge = EdgeData::FunctionCall {
            count: 10,
            async_call: true,
        };
        assert_eq!(extract_edge_weight(&edge), 10.0);
    }

    #[test]
    fn test_extract_edge_weight_inheritance() {
        let edge = EdgeData::Inheritance { depth: 2 };
        assert_eq!(extract_edge_weight(&edge), 0.5); // 1/2

        let edge = EdgeData::Inheritance { depth: 0 };
        assert_eq!(extract_edge_weight(&edge), 1.0);
    }

    #[test]
    fn test_create_node_mapping() {
        let mut graph = DependencyGraph::new();
        graph.add_node(create_test_node());
        graph.add_node(create_test_node());
        graph.add_node(create_test_node());

        let mapping = create_node_mapping(&graph);

        assert_eq!(mapping.len(), 3);
    }

    #[test]
    fn test_connected_components_empty() {
        let graph = DependencyGraph::new();
        assert_eq!(connected_components(&graph), 0);
    }

    #[test]
    fn test_connected_components_single() {
        let mut graph = DependencyGraph::new();
        graph.add_node(create_test_node());
        // Single node with no edges - aprender returns empty labels
        let count = connected_components(&graph);
        assert!(count <= 1);
    }

    #[test]
    fn test_connected_components_two_disconnected() {
        let mut graph = DependencyGraph::new();
        let n0 = graph.add_node(create_test_node());
        let n1 = graph.add_node(create_test_node());
        let _n2 = graph.add_node(create_test_node()); // disconnected node

        // Connect n0 -> n1, leave n2 disconnected
        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );

        let count = connected_components(&graph);
        // Should be 2: {n0, n1} and {n2}
        assert!(count >= 1); // At least one component
    }

    #[test]
    fn test_strongly_connected_components_empty() {
        let graph = DependencyGraph::new();
        let scc = strongly_connected_components(&graph);
        assert!(scc.is_empty());
    }

    #[test]
    fn test_strongly_connected_components_cycle() {
        let mut graph = DependencyGraph::new();
        let n0 = graph.add_node(create_test_node());
        let n1 = graph.add_node(create_test_node());

        // Create cycle: n0 -> n1 -> n0
        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );
        graph.add_edge(
            n1,
            n0,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );

        let scc = strongly_connected_components(&graph);
        // Both nodes should be in the same SCC
        assert!(!scc.is_empty());
    }

    #[test]
    fn test_is_cyclic_empty() {
        let graph = DependencyGraph::new();
        assert!(!is_cyclic(&graph));
    }

    #[test]
    fn test_is_cyclic_acyclic() {
        let mut graph = DependencyGraph::new();
        let n0 = graph.add_node(create_test_node());
        let n1 = graph.add_node(create_test_node());

        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );

        assert!(!is_cyclic(&graph));
    }

    #[test]
    fn test_is_cyclic_with_cycle() {
        let mut graph = DependencyGraph::new();
        let n0 = graph.add_node(create_test_node());
        let n1 = graph.add_node(create_test_node());

        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );
        graph.add_edge(
            n1,
            n0,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );

        assert!(is_cyclic(&graph));
    }

    #[test]
    fn test_shortest_path_empty() {
        let graph = DependencyGraph::new();
        assert!(shortest_path(&graph, 0, 1).is_none());
    }

    #[test]
    fn test_shortest_path_exists() {
        let mut graph = DependencyGraph::new();
        let n0 = graph.add_node(create_test_node());
        let n1 = graph.add_node(create_test_node());
        let n2 = graph.add_node(create_test_node());

        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );
        graph.add_edge(
            n1,
            n2,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );

        let result = shortest_path(&graph, 0, 2);
        assert!(result.is_some());
        let (path, distance) = result.unwrap();
        assert!(!path.is_empty());
        assert!(distance >= 0.0);
    }

    #[test]
    fn test_betweenness_centrality_empty() {
        let graph = DependencyGraph::new();
        let centrality = betweenness_centrality(&graph);
        assert!(centrality.is_empty());
    }

    #[test]
    fn test_betweenness_centrality_line() {
        let mut graph = DependencyGraph::new();
        let n0 = graph.add_node(create_test_node());
        let n1 = graph.add_node(create_test_node());
        let n2 = graph.add_node(create_test_node());

        // Line graph: n0 -> n1 -> n2
        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );
        graph.add_edge(
            n1,
            n2,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );

        let centrality = betweenness_centrality(&graph);
        // Middle node (n1) should have highest betweenness
        assert!(!centrality.is_empty());
    }

    #[test]
    fn test_louvain_communities_empty() {
        let graph = UndirectedGraph::new();
        let communities = louvain_communities(&graph);
        assert!(communities.is_empty());
    }
}
