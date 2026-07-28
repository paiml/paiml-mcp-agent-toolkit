// Tests for deterministic mermaid engine
// Included by deterministic_mermaid_engine.rs - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::unified_ast_engine::ModuleMetrics;
    use std::path::PathBuf;

    #[test]
    fn test_pagerank_determinism() {
        let engine = DeterministicMermaidEngine::new();
        let mut graph = SimpleStableGraph::new();

        // Create a simple 3-node graph
        let node1 = graph.add_node(ModuleNode {
            name: "node1".to_string(),
            path: PathBuf::from("node1.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics::default(),
        });

        let node2 = graph.add_node(ModuleNode {
            name: "node2".to_string(),
            path: PathBuf::from("node2.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics::default(),
        });

        let node3 = graph.add_node(ModuleNode {
            name: "node3".to_string(),
            path: PathBuf::from("node3.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics::default(),
        });

        graph.add_edge(node1, node2, EdgeType::Calls);
        graph.add_edge(node2, node3, EdgeType::Calls);
        graph.add_edge(node3, node1, EdgeType::Calls);

        // Compute PageRank multiple times
        let scores1 = engine.compute_pagerank(&graph, 0.85, 100);
        let scores2 = engine.compute_pagerank(&graph, 0.85, 100);

        // Results should be identical
        assert_eq!(
            scores1, scores2,
            "PageRank computation must be deterministic"
        );

        // All scores should sum to approximately 1.0
        let sum: f32 = scores1.values().sum();
        assert!(
            (sum - 1.0).abs() < 0.001,
            "PageRank scores should sum to 1.0, got {sum}"
        );
    }

    #[test]
    fn test_mermaid_output_determinism() {
        let engine = DeterministicMermaidEngine::new();
        let mut graph = SimpleStableGraph::new();

        // Add nodes in non-alphabetical order to test sorting
        let node_z = graph.add_node(ModuleNode {
            name: "z_module".to_string(),
            path: PathBuf::from("z.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics {
                complexity: 5,
                ..ModuleMetrics::default()
            },
        });

        let node_a = graph.add_node(ModuleNode {
            name: "a_module".to_string(),
            path: PathBuf::from("a.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics {
                complexity: 10,
                ..ModuleMetrics::default()
            },
        });

        graph.add_edge(node_z, node_a, EdgeType::Imports);

        // Generate diagram multiple times
        let mermaid1 = engine.generate_codebase_modules_mmd(&graph);
        let mermaid2 = engine.generate_codebase_modules_mmd(&graph);

        assert_eq!(mermaid1, mermaid2, "Mermaid output must be deterministic");

        // Check that output is well-formed
        assert!(mermaid1.starts_with("graph TD\n"));
        assert!(mermaid1.contains("a_module"));
        assert!(mermaid1.contains("z_module"));
        assert!(mermaid1.contains("-.->"));
    }

    #[test]
    fn test_sanitize_id() {
        let engine = DeterministicMermaidEngine::new();

        assert_eq!(engine.sanitize_id("foo::bar"), "foo_bar");
        assert_eq!(engine.sanitize_id("foo/bar.rs"), "foo_bar_rs");
        assert_eq!(engine.sanitize_id("foo-bar"), "foo_bar");
        assert_eq!(engine.sanitize_id("foo bar"), "foo_bar");
        assert_eq!(engine.sanitize_id("123foo"), "_123foo");
        assert_eq!(engine.sanitize_id("_foo"), "_foo");
        assert_eq!(engine.sanitize_id(""), "_empty");
    }

    #[test]
    fn test_escape_mermaid_label() {
        let engine = DeterministicMermaidEngine::new();

        assert_eq!(engine.escape_mermaid_label("simple"), "simple");
        assert_eq!(engine.escape_mermaid_label("with|pipe"), "with - pipe");
        assert_eq!(
            engine.escape_mermaid_label("with\"quotes\""),
            "with'quotes'"
        );
        assert_eq!(
            engine.escape_mermaid_label("with[brackets]"),
            "with(brackets)"
        );
        assert_eq!(engine.escape_mermaid_label("with{braces}"), "with(braces)");
        assert_eq!(engine.escape_mermaid_label("with<angle>"), "with(angle)");
        assert_eq!(
            engine.escape_mermaid_label("with&ampersand"),
            "with and ampersand"
        );
        assert_eq!(engine.escape_mermaid_label("line\nbreak"), "line break");
    }

    #[test]
    fn test_is_service_module() {
        let engine = DeterministicMermaidEngine::new();

        assert!(engine.is_service_module("user_service"));
        assert!(engine.is_service_module("api_handler"));
        assert!(engine.is_service_module("payment_controller"));
        assert!(engine.is_service_module("template_engine"));
        assert!(!engine.is_service_module("utils"));
        assert!(!engine.is_service_module("config"));
        assert!(!engine.is_service_module("models"));
    }

    #[test]
    fn test_complexity_styling() {
        let engine = DeterministicMermaidEngine::new();
        let mut graph = SimpleStableGraph::new();

        // Add service modules with different complexities
        let _low_complexity = graph.add_node(ModuleNode {
            name: "simple_service".to_string(),
            path: PathBuf::from("simple.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics {
                complexity: 5,
                ..ModuleMetrics::default()
            },
        });

        let _high_complexity = graph.add_node(ModuleNode {
            name: "complex_service".to_string(),
            path: PathBuf::from("complex.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics {
                complexity: 25,
                ..ModuleMetrics::default()
            },
        });

        let metrics = ProjectMetrics {
            total_complexity: 150,
            total_lines: 1000,
            file_count: 2,
            function_count: 10,
            avg_complexity: 15.0,
            max_complexity: 25,
        };

        let mermaid = engine.generate_service_interactions_mmd(&graph, &metrics);

        // Should contain complexity-based styling
        assert!(mermaid.contains("style simple_service fill:#90EE90")); // Low complexity - green
        assert!(mermaid.contains("style complex_service fill:#FF6347")); // High complexity - red
    }

    #[test]
    fn test_empty_graph() {
        let engine = DeterministicMermaidEngine::new();
        let graph = SimpleStableGraph::new();

        let mermaid = engine.generate_codebase_modules_mmd(&graph);
        assert_eq!(mermaid.trim(), "graph TD");

        let scores = engine.compute_pagerank(&graph, 0.85, 100);
        assert!(scores.is_empty());
    }

    /// Test that writeln! to String never fails (validates expect() at lines 68, 92, 134, 164, 181)
    #[test]
    fn test_string_write_never_fails() {
        let engine = DeterministicMermaidEngine::new();
        let mut graph = SimpleStableGraph::new();

        // Create a node with special characters that might challenge string writing
        let node = graph.add_node(ModuleNode {
            name: "test::module::with::colons".to_string(),
            path: PathBuf::from("test.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics::default(),
        });

        graph.add_edge(node, node, EdgeType::Calls);

        // These operations all use writeln! which should never fail when writing to String
        let mermaid = engine.generate_codebase_modules_mmd(&graph);

        // Verify the string was created successfully
        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("test_module_with_colons"));
    }

    /// Test sanitize_id with empty input and numeric prefixes (validates expect() at line 330)
    #[test]
    fn test_sanitize_id_edge_cases() {
        let engine = DeterministicMermaidEngine::new();

        // Empty input should return "_empty"
        assert_eq!(engine.sanitize_id(""), "_empty");

        // Numeric prefix should be prefixed with underscore
        assert_eq!(engine.sanitize_id("123module"), "_123module");
        assert_eq!(engine.sanitize_id("0start"), "_0start");

        // Valid identifier should pass through
        assert_eq!(engine.sanitize_id("validModule"), "validModule");
        assert_eq!(engine.sanitize_id("_underscore"), "_underscore");

        // Special characters should be replaced (:: becomes single _ via line 310)
        assert_eq!(engine.sanitize_id("my::module"), "my_module"); // "::" -> "_" (single underscore via replace())
        assert_eq!(engine.sanitize_id("path/to/file"), "path_to_file");
    }

    /// Test PageRank adjacency map invariants (validates expect() at lines 219, 223)
    #[test]
    fn test_pagerank_map_synchronization() {
        let engine = DeterministicMermaidEngine::new();
        let mut graph = SimpleStableGraph::new();

        // Create nodes
        let n1 = graph.add_node(ModuleNode {
            name: "n1".to_string(),
            path: PathBuf::from("n1.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics::default(),
        });

        let n2 = graph.add_node(ModuleNode {
            name: "n2".to_string(),
            path: PathBuf::from("n2.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics::default(),
        });

        let n3 = graph.add_node(ModuleNode {
            name: "n3".to_string(),
            path: PathBuf::from("n3.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics::default(),
        });

        // Add edges - all nodes should exist in outgoing/incoming maps
        graph.add_edge(n1, n2, EdgeType::Calls);
        graph.add_edge(n2, n3, EdgeType::Calls);
        graph.add_edge(n3, n1, EdgeType::Calls);

        // Compute PageRank - this exercises the expect() calls at lines 219 and 223
        let scores = engine.compute_pagerank(&graph, 0.85, 100);

        // All nodes should have scores
        assert_eq!(scores.len(), 3);
        assert!(scores.contains_key(&n1));
        assert!(scores.contains_key(&n2));
        assert!(scores.contains_key(&n3));
    }

    /// Test generate_service_interactions_mmd styling output (validates expect() at line 181)
    #[test]
    fn test_service_interactions_styling() {
        let engine = DeterministicMermaidEngine::new();
        let mut graph = SimpleStableGraph::new();

        // Add a service node with low complexity
        let low_service = graph.add_node(ModuleNode {
            name: "user_service".to_string(),
            path: PathBuf::from("user_service.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics {
                complexity: 5,
                ..ModuleMetrics::default()
            },
        });

        // Add a service node with high complexity
        let high_service = graph.add_node(ModuleNode {
            name: "payment_service".to_string(),
            path: PathBuf::from("payment_service.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics {
                complexity: 25,
                ..ModuleMetrics::default()
            },
        });

        graph.add_edge(low_service, high_service, EdgeType::Calls);

        let metrics = ProjectMetrics {
            file_count: 2,
            function_count: 10,
            avg_complexity: 15.0,
            max_complexity: 25,
            total_complexity: 30,
            total_lines: 500,
        };

        let mermaid = engine.generate_service_interactions_mmd(&graph, &metrics);

        // Verify styling was applied successfully
        assert!(mermaid.contains("style user_service fill:#90EE90")); // Low complexity - green
        assert!(mermaid.contains("style payment_service fill:#FF6347")); // High complexity - red
    }

    /// Test that all string writes work with complex module names
    #[test]
    fn test_complex_module_name_handling() {
        let engine = DeterministicMermaidEngine::new();
        let mut graph = SimpleStableGraph::new();

        // Create nodes with challenging names
        let names = vec![
            "std::collections::HashMap",
            "my_crate::utils::helpers",
            "core::ops::Fn",
            "alloc::vec::Vec",
        ];

        let mut nodes = Vec::new();
        for name in &names {
            nodes.push(graph.add_node(ModuleNode {
                name: name.to_string(),
                path: PathBuf::from(format!("{name}.rs")),
                visibility: "public".to_string(),
                metrics: ModuleMetrics::default(),
            }));
        }

        // Add edges between all nodes
        for i in 0..nodes.len() {
            for j in 0..nodes.len() {
                if i != j {
                    graph.add_edge(nodes[i], nodes[j], EdgeType::Calls);
                }
            }
        }

        // This exercises all writeln! operations
        let mermaid = engine.generate_codebase_modules_mmd(&graph);

        // Verify all nodes are present in the output
        assert!(mermaid.contains("std_collections_HashMap"));
        assert!(mermaid.contains("my_crate_utils_helpers"));
        assert!(mermaid.contains("core_ops_Fn"));
        assert!(mermaid.contains("alloc_vec_Vec"));

        // Verify edges are present
        assert!(mermaid.contains("-->"));
    }
}

