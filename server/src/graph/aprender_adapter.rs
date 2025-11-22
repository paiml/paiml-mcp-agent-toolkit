// Adapter module: PMAT DependencyGraph → aprender::graph::Graph
// Complexity: All functions ≤ 10
// SATD: Zero tolerance

use super::types::{DependencyGraph, EdgeData, UndirectedGraph};
use aprender::graph::Graph as AprenderGraph;
use petgraph::visit::{EdgeRef, IntoNodeReferences};

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
    // Build edge list from petgraph
    let mut edges = Vec::new();

    for edge in graph.edge_references() {
        let source = edge.source().index();
        let target = edge.target().index();

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
    // Build edge list from petgraph
    let mut edges = Vec::new();

    for edge in graph.edge_references() {
        let source = edge.source().index();
        let target = edge.target().index();

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
/// PMAT's petgraph nodes may have gaps, so we need a mapping.
pub fn create_node_mapping(graph: &DependencyGraph) -> Vec<usize> {
    graph
        .node_references()
        .map(|(idx, _)| idx.index())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::types::{NodeData, Symbol, SymbolKind, Visibility};
    use petgraph::Graph;
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
        let mut graph = Graph::new();
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
        let mut graph = Graph::new();
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
        let mut graph = Graph::new();
        graph.add_node(create_test_node());
        graph.add_node(create_test_node());
        graph.add_node(create_test_node());

        let mapping = create_node_mapping(&graph);

        assert_eq!(mapping.len(), 3);
        assert_eq!(mapping, vec![0, 1, 2]);
    }
}
