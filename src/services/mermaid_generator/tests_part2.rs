#![cfg_attr(coverage_nightly, coverage(off))]
//! Additional tests: validation tests and string/styling tests

#[cfg(test)]
mod tests {
    use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
    use crate::services::mermaid_generator::{MermaidGenerator, MermaidOptions};
    use rustc_hash::FxHashMap;

    /// Validation tests for real-world Mermaid parser compatibility
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[cfg(test)]
    mod validation_tests {
        use super::*;
        use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};

        /// Test characters that caused the IntelliJ parse error:
        /// "Parse error on line 2: ...cache_rs_Cache_K_V_ [Interface: Cache(K,V)  -  ..."
        #[test]
        fn test_angle_brackets_and_pipes_compatibility() {
            let mut graph = DependencyGraph::new();

            graph.add_node(NodeInfo {
                id: "cache.rs::Cache<K,V>".to_string(),
                label: "Cache<K,V>".to_string(),
                node_type: NodeType::Interface,
                file_path: "cache.rs".to_string(),
                line_number: 1,
                complexity: 6,
                metadata: FxHashMap::default(),
            });

            let generator = MermaidGenerator::new(MermaidOptions {
                show_complexity: true,
                ..Default::default()
            });

            let output = generator.generate(&graph);

            assert!(!output.contains("<")); // No raw angle brackets
            assert!(!output.contains(">")); // No raw angle brackets
            assert!(!output.contains("&#")); // No HTML entities

            assert!(output.contains("cache_rs_Cache_K_V_")); // Node ID is sanitized and present
        }

        /// Test that all edge types produce valid arrow syntax
        #[test]
        fn test_edge_arrow_syntax() {
            let generator = MermaidGenerator::default();

            let edge_types = vec![
                EdgeType::Calls,
                EdgeType::Imports,
                EdgeType::Inherits,
                EdgeType::Implements,
                EdgeType::Uses,
            ];

            for edge_type in edge_types {
                let arrow = generator.get_edge_arrow(&edge_type);

                assert!(
                    arrow.contains("-"),
                    "Arrow '{arrow}' for {edge_type:?} must contain dash"
                );

                match &edge_type {
                    EdgeType::Inherits | EdgeType::Implements => {
                        assert!(
                            arrow.contains("|"),
                            "Labeled edge '{arrow}' should contain pipe"
                        );
                    }
                    _ => {
                        assert!(
                            !arrow.contains(" "),
                            "Arrow '{arrow}' should not contain spaces"
                        );
                        assert!(
                            !arrow.contains("|"),
                            "Arrow '{arrow}' should not contain pipes"
                        );
                    }
                }
            }
        }

        /// Integration test that creates a realistic dependency graph and validates output
        #[test]
        fn test_realistic_dependency_graph() {
            let mut graph = DependencyGraph::new();

            let nodes = vec![
                ("main.rs::main", "main", NodeType::Function, 3),
                ("lib.rs::Config", "Config", NodeType::Class, 5),
                ("error.rs::AppError", "AppError", NodeType::Class, 7),
                ("traits.rs::Processor", "Processor", NodeType::Trait, 4),
                ("utils.rs", "utils", NodeType::Module, 2),
                ("api.rs::Handler<T>", "Handler<T>", NodeType::Interface, 8),
            ];

            for (id, label, node_type, complexity) in nodes {
                graph.add_node(NodeInfo {
                    id: id.to_string(),
                    label: label.to_string(),
                    node_type,
                    file_path: format!("{}.rs", id.split("::").next().unwrap().replace(".rs", "")),
                    line_number: 1,
                    complexity,
                    metadata: FxHashMap::default(),
                });
            }

            let edges = vec![
                ("main.rs::main", "lib.rs::Config", EdgeType::Uses),
                ("main.rs::main", "api.rs::Handler<T>", EdgeType::Calls),
                (
                    "api.rs::Handler<T>",
                    "traits.rs::Processor",
                    EdgeType::Implements,
                ),
                ("lib.rs::Config", "error.rs::AppError", EdgeType::Uses),
            ];

            for (from, to, edge_type) in edges {
                graph.add_edge(Edge {
                    from: from.to_string(),
                    to: to.to_string(),
                    edge_type,
                    weight: 1,
                });
            }

            let generator = MermaidGenerator::new(MermaidOptions {
                show_complexity: true,
                ..Default::default()
            });

            let output = generator.generate(&graph);

            assert!(output.starts_with("graph TD\n"));
            assert!(output.contains("style "));
            assert!(!output.contains("<")); // No raw angle brackets
            assert!(!output.contains("&")); // No raw ampersands
            assert!(
                !output.contains("|")
                    || output.contains("implements")
                    || output.contains("inherits")
            );

            assert!(output.contains("main_rs_main"));
            assert!(output.contains("lib_rs_Config"));
            assert!(output.contains("api_rs_Handler_T_"));

            assert!(output.contains("#FFD700")); // Gold for medium complexity
            assert!(output.contains("#FFA500")); // Orange for high complexity
        }
    }

    /// Test that writeln! to String never fails (validates expect() at lines 130, 146, 164, 219, 251, 293)
    #[test]
    fn test_string_write_never_fails() {
        let generator = MermaidGenerator::new(MermaidOptions {
            show_complexity: true,
            ..Default::default()
        });

        let mut nodes = FxHashMap::default();
        nodes.insert(
            "test::module::function".to_string(),
            NodeInfo {
                id: "test::module::function".to_string(),
                label: "test::module::function".to_string(),
                node_type: NodeType::Function,
                file_path: "test.rs".to_string(),
                line_number: 1,
                complexity: 10,
                metadata: FxHashMap::default(),
            },
        );

        let graph = DependencyGraph {
            nodes,
            edges: vec![],
        };

        let output = generator.generate(&graph);

        assert!(output.contains("graph TD"));
        assert!(output.contains("test_module_function"));
        assert!(output.contains("style ")); // Complexity styling enabled
    }

    /// Test sanitize_id with empty input and numeric prefixes (validates expect() at line 354)
    #[test]
    fn test_sanitize_id_edge_cases() {
        let generator = MermaidGenerator::default();

        assert_eq!(generator.sanitize_id(""), "_empty");
        assert_eq!(generator.sanitize_id("123module"), "_123module");
        assert_eq!(generator.sanitize_id("0start"), "_0start");
        assert_eq!(generator.sanitize_id("validModule"), "validModule");
        assert_eq!(generator.sanitize_id("_underscore"), "_underscore");
        assert_eq!(generator.sanitize_id("my::module"), "my_module");
        assert_eq!(generator.sanitize_id("path/to/file"), "path_to_file");
    }

    /// Test Mermaid generation with complex module names (exercises all unwrap fixes)
    #[test]
    fn test_complex_module_names() {
        let generator = MermaidGenerator::new(MermaidOptions {
            show_complexity: true,
            ..Default::default()
        });

        let mut nodes = FxHashMap::default();

        nodes.insert(
            "std::collections::HashMap".to_string(),
            NodeInfo {
                id: "std::collections::HashMap".to_string(),
                label: "std::collections::HashMap".to_string(),
                node_type: NodeType::Class,
                file_path: "std.rs".to_string(),
                line_number: 1,
                complexity: 15,
                metadata: FxHashMap::default(),
            },
        );

        nodes.insert(
            "123_numeric_start".to_string(),
            NodeInfo {
                id: "123_numeric_start".to_string(),
                label: "123_numeric_start".to_string(),
                node_type: NodeType::Function,
                file_path: "test.rs".to_string(),
                line_number: 5,
                complexity: 5,
                metadata: FxHashMap::default(),
            },
        );

        nodes.insert(
            "".to_string(),
            NodeInfo {
                id: "".to_string(),
                label: "".to_string(),
                node_type: NodeType::Module,
                file_path: "empty.rs".to_string(),
                line_number: 1,
                complexity: 1,
                metadata: FxHashMap::default(),
            },
        );

        let graph = DependencyGraph {
            nodes,
            edges: vec![
                Edge {
                    from: "std::collections::HashMap".to_string(),
                    to: "123_numeric_start".to_string(),
                    edge_type: EdgeType::Calls,
                    weight: 1,
                },
                Edge {
                    from: "123_numeric_start".to_string(),
                    to: "".to_string(),
                    edge_type: EdgeType::Uses,
                    weight: 1,
                },
            ],
        };

        let output = generator.generate(&graph);

        assert!(output.contains("std_collections_HashMap")); // :: -> _
        assert!(output.contains("_123_numeric_start")); // Numeric prefix gets _
        assert!(output.contains("_empty")); // Empty -> "_empty"

        assert!(output.contains("style "));
        assert!(output.contains("-->") || output.contains("---"));
    }

    /// Test edge generation with various edge types (validates writeln! at lines 146, 251)
    #[test]
    fn test_edge_generation() {
        let generator = MermaidGenerator::default();

        let mut nodes = FxHashMap::default();
        nodes.insert(
            "node1".to_string(),
            NodeInfo {
                id: "node1".to_string(),
                label: "Node 1".to_string(),
                node_type: NodeType::Function,
                file_path: "test.rs".to_string(),
                line_number: 1,
                complexity: 5,
                metadata: FxHashMap::default(),
            },
        );
        nodes.insert(
            "node2".to_string(),
            NodeInfo {
                id: "node2".to_string(),
                label: "Node 2".to_string(),
                node_type: NodeType::Function,
                file_path: "test.rs".to_string(),
                line_number: 10,
                complexity: 5,
                metadata: FxHashMap::default(),
            },
        );

        let graph = DependencyGraph {
            nodes,
            edges: vec![
                Edge {
                    from: "node1".to_string(),
                    to: "node2".to_string(),
                    edge_type: EdgeType::Calls,
                    weight: 1,
                },
                Edge {
                    from: "node2".to_string(),
                    to: "node1".to_string(),
                    edge_type: EdgeType::Imports,
                    weight: 1,
                },
            ],
        };

        let output = generator.generate(&graph);

        assert!(output.contains("-->")); // Calls arrow
        assert!(output.contains("-.->") || output.contains("Imports")); // Imports arrow
    }

    /// Test styling generation with complexity-based colors (validates writeln! at lines 164, 293)
    #[test]
    fn test_styling_generation() {
        let generator = MermaidGenerator::new(MermaidOptions {
            show_complexity: true,
            ..Default::default()
        });

        let mut nodes = FxHashMap::default();

        nodes.insert(
            "low".to_string(),
            NodeInfo {
                id: "low".to_string(),
                label: "Low Complexity".to_string(),
                node_type: NodeType::Function,
                file_path: "test.rs".to_string(),
                line_number: 1,
                complexity: 5,
                metadata: FxHashMap::default(),
            },
        );

        nodes.insert(
            "high".to_string(),
            NodeInfo {
                id: "high".to_string(),
                label: "High Complexity".to_string(),
                node_type: NodeType::Function,
                file_path: "test.rs".to_string(),
                line_number: 20,
                complexity: 25,
                metadata: FxHashMap::default(),
            },
        );

        let graph = DependencyGraph {
            nodes,
            edges: vec![],
        };

        let output = generator.generate(&graph);

        assert!(output.contains("style low"));
        assert!(output.contains("style high"));
        assert!(output.contains("#90EE90") || output.contains("#FFD700")); // Green or gold
        assert!(output.contains("#FF6347") || output.contains("#FFA500")); // Red or orange
    }
}
