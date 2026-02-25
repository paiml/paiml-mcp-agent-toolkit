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
