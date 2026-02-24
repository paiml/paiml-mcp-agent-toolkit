// Tests for graph type system
// Included by types.rs - shares parent module scope (no `use` imports)

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // NodeData Tests
    // ============================================================

    #[test]
    fn test_node_data_default_construction() {
        let node = NodeData {
            path: PathBuf::from("test.rs"),
            module: String::new(),
            symbols: Vec::new(),
            loc: 0,
            complexity: 0.0,
            ast_hash: 0,
        };
        assert!(node.path.to_str().unwrap().contains("test.rs"));
        assert!(node.module.is_empty());
        assert!(node.symbols.is_empty());
        assert_eq!(node.loc, 0);
        assert_eq!(node.complexity, 0.0);
        assert_eq!(node.ast_hash, 0);
    }

    #[test]
    fn test_node_data_with_multiple_symbols() {
        let node = NodeData {
            path: PathBuf::from("lib.rs"),
            module: "mylib".to_string(),
            symbols: vec![
                Symbol {
                    name: "foo".to_string(),
                    kind: SymbolKind::Function,
                    visibility: Visibility::Public,
                    line: 1,
                },
                Symbol {
                    name: "Bar".to_string(),
                    kind: SymbolKind::Struct,
                    visibility: Visibility::Private,
                    line: 10,
                },
                Symbol {
                    name: "MyTrait".to_string(),
                    kind: SymbolKind::Trait,
                    visibility: Visibility::Public,
                    line: 50,
                },
            ],
            loc: 200,
            complexity: 15.5,
            ast_hash: 0xCAFEBABE,
        };

        assert_eq!(node.symbols.len(), 3);
        assert_eq!(node.symbols[0].name, "foo");
        assert_eq!(node.symbols[1].name, "Bar");
        assert_eq!(node.symbols[2].name, "MyTrait");
    }

    #[test]
    fn test_node_data_clone() {
        let original = NodeData {
            path: PathBuf::from("original.rs"),
            module: "orig".to_string(),
            symbols: vec![Symbol {
                name: "test".to_string(),
                kind: SymbolKind::Function,
                visibility: Visibility::Public,
                line: 5,
            }],
            loc: 50,
            complexity: 3.0,
            ast_hash: 123,
        };

        let cloned = original.clone();
        assert_eq!(original.path, cloned.path);
        assert_eq!(original.module, cloned.module);
        assert_eq!(original.symbols.len(), cloned.symbols.len());
        assert_eq!(original.loc, cloned.loc);
        assert_eq!(original.complexity, cloned.complexity);
        assert_eq!(original.ast_hash, cloned.ast_hash);
    }

    #[test]
    fn test_node_data_serialization_roundtrip() {
        let node = NodeData {
            path: PathBuf::from("src/lib.rs"),
            module: "lib".to_string(),
            symbols: vec![
                Symbol {
                    name: "init".to_string(),
                    kind: SymbolKind::Function,
                    visibility: Visibility::Public,
                    line: 1,
                },
                Symbol {
                    name: "Config".to_string(),
                    kind: SymbolKind::Struct,
                    visibility: Visibility::Public,
                    line: 20,
                },
            ],
            loc: 150,
            complexity: 8.5,
            ast_hash: 0xDEADC0DE,
        };

        let json = serde_json::to_string(&node).expect("serialization failed");
        let deserialized: NodeData = serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(node.path, deserialized.path);
        assert_eq!(node.module, deserialized.module);
        assert_eq!(node.symbols.len(), deserialized.symbols.len());
        assert_eq!(node.loc, deserialized.loc);
        assert_eq!(node.complexity, deserialized.complexity);
        assert_eq!(node.ast_hash, deserialized.ast_hash);
    }

    // ============================================================
    // DependencyGraph Tests
    // ============================================================

    fn create_test_node(id: usize) -> NodeData {
        NodeData {
            path: PathBuf::from(format!("file_{}.rs", id)),
            module: format!("mod_{}", id),
            symbols: vec![],
            loc: 100,
            complexity: 1.0,
            ast_hash: id as u64,
        }
    }

    #[test]
    fn test_dependency_graph_new() {
        let graph = DependencyGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_dependency_graph_add_node() {
        let mut graph = DependencyGraph::new();
        let id = graph.add_node(create_test_node(0));
        assert_eq!(id.0, 0);
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn test_dependency_graph_add_edge() {
        let mut graph = DependencyGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));

        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );

        assert_eq!(graph.edge_count(), 1);
        assert!(graph.contains_edge(n0, n1));
        assert!(!graph.contains_edge(n1, n0));
    }

    #[test]
    fn test_dependency_graph_node_weight() {
        let mut graph = DependencyGraph::new();
        let id = graph.add_node(create_test_node(42));

        let data = graph.node_weight(id).unwrap();
        assert_eq!(data.ast_hash, 42);
    }

    #[test]
    fn test_dependency_graph_edge_weight() {
        let mut graph = DependencyGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));

        graph.add_edge(
            n0,
            n1,
            EdgeData::FunctionCall {
                count: 5,
                async_call: true,
            },
        );

        let edge = graph.edge_weight(n0, n1).unwrap();
        if let EdgeData::FunctionCall { count, async_call } = edge {
            assert_eq!(*count, 5);
            assert!(*async_call);
        } else {
            panic!("Wrong edge type");
        }
    }

    #[test]
    fn test_dependency_graph_neighbors() {
        let mut graph = DependencyGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));
        let n2 = graph.add_node(create_test_node(2));

        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );
        graph.add_edge(
            n0,
            n2,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );

        let neighbors = graph.neighbors(n0);
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&n1));
        assert!(neighbors.contains(&n2));
    }

    #[test]
    fn test_dependency_graph_edge_references() {
        let mut graph = DependencyGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));

        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 2.0,
                visibility: Visibility::Public,
            },
        );

        let edges: Vec<_> = graph.edge_references().collect();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].source(), n0);
        assert_eq!(edges[0].target(), n1);
    }

    // ============================================================
    // UndirectedGraph Tests
    // ============================================================

    #[test]
    fn test_undirected_graph_new() {
        let graph = UndirectedGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_undirected_graph_add_edge() {
        let mut graph = UndirectedGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));

        graph.add_edge(n0, n1, 1.5);

        assert_eq!(graph.edge_count(), 1);
        // Both directions should have weight
        assert_eq!(graph.edge_weight(n0, n1), Some(1.5));
        assert_eq!(graph.edge_weight(n1, n0), Some(1.5));
    }

    // ============================================================
    // EdgeData Tests
    // ============================================================

    #[test]
    fn test_to_numeric_weight_import() {
        let edge = EdgeData::Import {
            weight: 1.0,
            visibility: Visibility::Private,
        };
        assert_eq!(edge.to_numeric_weight(), 2.0); // weight * 2.0
    }

    #[test]
    fn test_to_numeric_weight_function_call() {
        let edge = EdgeData::FunctionCall {
            count: 7,
            async_call: false,
        };
        assert_eq!(edge.to_numeric_weight(), 7.0);
    }

    #[test]
    fn test_to_numeric_weight_inheritance() {
        let edge = EdgeData::Inheritance { depth: 0 };
        assert_eq!(edge.to_numeric_weight(), 3.0); // 3.0 / (0 + 1)

        let edge = EdgeData::Inheritance { depth: 2 };
        assert_eq!(edge.to_numeric_weight(), 1.0); // 3.0 / (2 + 1)
    }

    // ============================================================
    // GraphMatrices Tests
    // ============================================================

    #[test]
    fn test_graph_matrices_empty_graph() {
        let graph = DependencyGraph::new();
        let matrices = GraphMatrices::from(&graph);

        assert_eq!(matrices.node_count, 0);
        assert_eq!(matrices.out_degrees.len(), 0);
        assert_eq!(matrices.edges.len(), 0);
    }

    #[test]
    fn test_graph_matrices_single_edge() {
        let mut graph = DependencyGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));

        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );

        let matrices = GraphMatrices::from(&graph);

        assert_eq!(matrices.node_count, 2);
        assert_eq!(matrices.edges.len(), 1);
    }
}
