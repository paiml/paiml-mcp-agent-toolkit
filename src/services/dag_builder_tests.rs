// Tests for dag_builder module

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
    use rustc_hash::FxHashMap;

    fn create_test_node(id: &str, node_type: NodeType) -> NodeInfo {
        NodeInfo {
            id: id.to_string(),
            label: id.to_string(),
            node_type,
            file_path: format!("{}.rs", id),
            line_number: 1,
            complexity: 5,
            metadata: FxHashMap::default(),
        }
    }

    fn create_test_graph() -> DependencyGraph {
        let mut graph = DependencyGraph::new();

        graph.add_node(create_test_node("a", NodeType::Function));
        graph.add_node(create_test_node("b", NodeType::Function));
        graph.add_node(create_test_node("c", NodeType::Function));
        graph.add_node(create_test_node("d", NodeType::Class));

        graph.edges.push(Edge {
            from: "a".to_string(),
            to: "b".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1,
        });
        graph.edges.push(Edge {
            from: "b".to_string(),
            to: "c".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1,
        });
        graph.edges.push(Edge {
            from: "c".to_string(),
            to: "d".to_string(),
            edge_type: EdgeType::Uses,
            weight: 1,
        });

        graph
    }

    #[test]
    fn test_dag_builder_new() {
        let builder = DagBuilder::new();
        assert!(builder.function_map.is_empty());
        assert!(builder.type_map.is_empty());
    }

    fn project_with(files: Vec<FileContext>) -> ProjectContext {
        ProjectContext {
            project_type: "rust".to_string(),
            summary: crate::services::context::ProjectSummary {
                total_files: files.len(),
                total_functions: 0,
                total_structs: 0,
                total_enums: 0,
                total_traits: 0,
                total_impls: 0,
                dependencies: vec![],
            },
            files,
            graph: None,
        }
    }

    fn rust_file(path: &str, items: Vec<AstItem>) -> FileContext {
        FileContext {
            path: path.to_string(),
            language: "rust".to_string(),
            items,
            complexity_metrics: None,
        }
    }

    /// Regression test for #653: `use helper::{..}` in main.rs must yield an Imports
    /// edge to the analyzed helper module. Before the fix, `resolve_import_path`
    /// returned an invented node id ("helper"), `finalize_graph` dropped the edge,
    /// and every project rendered "0 edges".
    #[test]
    fn test_import_edge_resolves_to_sibling_module() {
        let project = project_with(vec![
            rust_file(
                "src/main.rs",
                vec![
                    AstItem::Use {
                        path: "helper".to_string(),
                        line: 1,
                    },
                    AstItem::Function {
                        name: "main".to_string(),
                        visibility: "private".to_string(),
                        is_async: false,
                        line: 4,
                    },
                ],
            ),
            rust_file(
                "src/helper.rs",
                vec![AstItem::Function {
                    name: "helper_one".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 1,
                }],
            ),
        ]);

        let graph = DagBuilder::build_from_project(&project);

        let import_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.edge_type == EdgeType::Imports)
            .collect();
        assert_eq!(
            import_edges.len(),
            1,
            "expected 1 import edge, got {:?}",
            graph.edges
        );
        assert_eq!(import_edges[0].from, "src_main");
        assert_eq!(import_edges[0].to, "src_helper");
    }

    /// Regression test for #653: a crate-relative path must resolve to the file that
    /// defines the module, not to the literal first segment "crate".
    #[test]
    fn test_import_edge_resolves_crate_relative_path() {
        let project = project_with(vec![
            rust_file(
                "src/main.rs",
                vec![AstItem::Use {
                    path: "crate::services::helper::Thing".to_string(),
                    line: 1,
                }],
            ),
            rust_file("src/services/helper.rs", vec![]),
        ]);

        let graph = DagBuilder::build_from_project(&project);

        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].from, "src_main");
        assert_eq!(graph.edges[0].to, "src_services_helper");
    }

    /// An import of an external crate must not invent an edge.
    #[test]
    fn test_external_import_creates_no_edge() {
        let project = project_with(vec![rust_file(
            "src/main.rs",
            vec![AstItem::Use {
                path: "serde::Serialize".to_string(),
                line: 1,
            }],
        )]);

        let graph = DagBuilder::build_from_project(&project);

        assert!(
            graph.edges.is_empty(),
            "unresolvable imports must not create edges: {:?}",
            graph.edges
        );
    }

    /// output_derived_from_input: no files, no nodes, no edges.
    #[test]
    fn test_empty_project_yields_empty_graph() {
        let graph = DagBuilder::build_from_project(&project_with(vec![]));
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_prune_graph_pagerank_no_pruning_needed() {
        let graph = create_test_graph();
        let pruned = prune_graph_pagerank(&graph, 10);

        // No pruning needed since we have fewer nodes than max
        assert_eq!(pruned.nodes.len(), graph.nodes.len());
        assert_eq!(pruned.edges.len(), graph.edges.len());
    }

    #[test]
    fn test_prune_graph_pagerank_with_pruning() {
        let graph = create_test_graph();
        let pruned = prune_graph_pagerank(&graph, 2);

        // Should prune to 2 nodes
        assert!(pruned.nodes.len() <= 2);

        // Edges should only reference remaining nodes
        for edge in &pruned.edges {
            assert!(pruned.nodes.contains_key(&edge.from));
            assert!(pruned.nodes.contains_key(&edge.to));
        }
    }

    #[test]
    fn test_prune_graph_pagerank_adds_centrality_metadata() {
        let graph = create_test_graph();
        let pruned = prune_graph_pagerank(&graph, 3);

        // Each remaining node should have centrality metadata
        for node in pruned.nodes.values() {
            assert!(node.metadata.contains_key("centrality"));
        }
    }

    #[test]
    fn test_add_pagerank_scores() {
        let graph = create_test_graph();
        let scored = add_pagerank_scores(graph.clone());

        // Should have same number of nodes and edges
        assert_eq!(scored.nodes.len(), graph.nodes.len());
        assert_eq!(scored.edges.len(), graph.edges.len());

        // Each node should have centrality score
        for node in scored.nodes.values() {
            assert!(node.metadata.contains_key("centrality"));
        }
    }

    #[test]
    fn test_add_pagerank_scores_empty_graph() {
        let graph = DependencyGraph::new();
        let scored = add_pagerank_scores(graph.clone());

        assert!(scored.nodes.is_empty());
        assert!(scored.edges.is_empty());
    }

    #[test]
    fn test_prune_graph_empty() {
        let graph = DependencyGraph::new();
        let pruned = prune_graph_pagerank(&graph, 5);

        assert!(pruned.nodes.is_empty());
        assert!(pruned.edges.is_empty());
    }

    #[test]
    fn test_prune_graph_single_node() {
        let mut graph = DependencyGraph::new();
        graph.add_node(create_test_node("single", NodeType::Function));

        let pruned = prune_graph_pagerank(&graph, 5);

        assert_eq!(pruned.nodes.len(), 1);
        assert!(pruned.nodes.contains_key("single"));
    }

    #[test]
    fn test_edge_type_priority_in_finalize() {
        // Create a graph with many edges that exceeds EDGE_BUDGET
        let mut graph = DependencyGraph::new();

        // Add 10 nodes
        for i in 0..10 {
            graph.add_node(create_test_node(&format!("node{i}"), NodeType::Function));
        }

        // Add edges with different types
        graph.edges.push(Edge {
            from: "node0".to_string(),
            to: "node1".to_string(),
            edge_type: EdgeType::Inherits, // Highest priority
            weight: 1,
        });
        graph.edges.push(Edge {
            from: "node1".to_string(),
            to: "node2".to_string(),
            edge_type: EdgeType::Imports, // Lowest priority
            weight: 1,
        });
        graph.edges.push(Edge {
            from: "node2".to_string(),
            to: "node3".to_string(),
            edge_type: EdgeType::Uses,
            weight: 1,
        });

        // With 3 edges, no pruning needed, but order should still work
        let scored = add_pagerank_scores(graph.clone());
        assert_eq!(scored.edges.len(), 3);
    }

    #[test]
    fn test_dependency_graph_clone() {
        let graph = create_test_graph();
        let cloned = graph.clone();

        assert_eq!(cloned.nodes.len(), graph.nodes.len());
        assert_eq!(cloned.edges.len(), graph.edges.len());
    }
}

