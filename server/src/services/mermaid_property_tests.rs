#[cfg(test)]
mod tests {
    use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
    use crate::services::mermaid_generator::{MermaidGenerator, MermaidOptions};
    use proptest::prelude::*;

    // Strategy for generating valid node IDs
    fn node_id_strategy() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-zA-Z][a-zA-Z0-9_]{0,50}").unwrap()
    }

    // Strategy for generating labels with special characters
    fn label_strategy() -> impl Strategy<Value = String> {
        prop::string::string_regex(".{0,100}").unwrap()
    }

    // Strategy for generating node types
    fn node_type_strategy() -> impl Strategy<Value = NodeType> {
        prop_oneof![
            Just(NodeType::Function),
            Just(NodeType::Class),
            Just(NodeType::Module),
            Just(NodeType::Trait),
            Just(NodeType::Interface),
        ]
    }

    // Strategy for generating edge types
    fn edge_type_strategy() -> impl Strategy<Value = EdgeType> {
        prop_oneof![
            Just(EdgeType::Calls),
            Just(EdgeType::Imports),
            Just(EdgeType::Inherits),
            Just(EdgeType::Implements),
            Just(EdgeType::Uses),
        ]
    }

    // Strategy for generating nodes
    fn node_strategy() -> impl Strategy<Value = NodeInfo> {
        (
            node_id_strategy(),
            label_strategy(),
            node_type_strategy(),
            prop::string::string_regex("[a-zA-Z0-9_/\\.]{1,50}\\.rs").unwrap(),
            0usize..10000,
            0u32..100,
        )
            .prop_map(
                |(id, label, node_type, file_path, line_number, complexity)| NodeInfo {
                    id,
                    label,
                    node_type,
                    file_path,
                    line_number,
                    complexity,
                },
            )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn mermaid_generation_never_panics(
            nodes in prop::collection::vec(node_strategy(), 0..100)
        ) {
            let mut graph = DependencyGraph::new();
            let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();

            // Add nodes
            for node in nodes {
                graph.add_node(node);
            }

            // Add some edges
            if node_ids.len() >= 2 {
                let edge_count = node_ids.len().min(20);
                for i in 0..edge_count {
                    let from_idx = i % node_ids.len();
                    let to_idx = (i * 3 + 1) % node_ids.len();
                    if from_idx != to_idx {
                        graph.add_edge(Edge {
                            from: node_ids[from_idx].clone(),
                            to: node_ids[to_idx].clone(),
                            edge_type: EdgeType::Calls,
                            weight: 1,
                        });
                    }
                }
            }

            let generator = MermaidGenerator::new(MermaidOptions::default());

            // Should not panic
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                generator.generate(&graph)
            }));

            prop_assert!(result.is_ok());

            if let Ok(output) = result {
                // Basic validation
                prop_assert!(output.starts_with("graph TD\n"));
            }
        }

        #[test]
        fn escaped_labels_are_valid(
            label in label_strategy()
        ) {
            let mut graph = DependencyGraph::new();

            graph.add_node(NodeInfo {
                id: "test_node".to_string(),
                label: label.clone(),
                node_type: NodeType::Function,
                file_path: "test.rs".to_string(),
                line_number: 1,
                complexity: 5,
            });

            let generator = MermaidGenerator::new(MermaidOptions {
                show_complexity: true,
                ..Default::default()
            });

            let output = generator.generate(&graph);

            // Should contain the node
            prop_assert!(output.contains("test_node"));

            // Should not have unescaped special characters in the output
            if output.contains("[") {
                // Find the label content
                let lines: Vec<&str> = output.lines().collect();
                for line in lines {
                    if line.contains("test_node") && line.contains("[") {
                        let start = line.find("[").unwrap() + 1;
                        let end = line.rfind("]").unwrap();
                        let content = &line[start..end];

                        // Basic validation that special chars were properly handled
                        if content.contains(" - ") {
                            prop_assert!(!content.contains("|"));
                        }
                        if content.contains(" and ") {
                            prop_assert!(!content.contains("&"));
                        }
                    }
                }
            }
        }

        #[test]
        fn sanitized_ids_are_valid(
            id in prop::string::string_regex(".{0,100}").unwrap()
        ) {
            let generator = MermaidGenerator::default();
            let sanitized = generator.sanitize_id(&id);

            // Should only contain valid characters
            prop_assert!(sanitized.chars().all(|c| c.is_alphanumeric() || c == '_'));

            // Should start with letter or underscore
            if !sanitized.is_empty() {
                prop_assert!(sanitized.chars().next().unwrap().is_alphabetic() ||
                           sanitized.starts_with('_'));
            }
        }

        #[test]
        fn all_nodes_appear_in_output(
            nodes in prop::collection::vec(node_strategy(), 1..20)
        ) {
            let mut graph = DependencyGraph::new();

            for node in &nodes {
                graph.add_node(node.clone());
            }

            let generator = MermaidGenerator::new(MermaidOptions::default());
            let output = generator.generate(&graph);

            // All nodes should appear in the output
            for node in &nodes {
                let sanitized_id = generator.sanitize_id(&node.id);
                prop_assert!(output.contains(&sanitized_id));
            }
        }

        #[test]
        fn complexity_colors_are_consistent(
            complexity in 0u32..100
        ) {
            let generator = MermaidGenerator::default();
            let color = generator.get_complexity_color(complexity);

            // Should return a valid color
            prop_assert!(color.starts_with('#'));
            prop_assert_eq!(color.len(), 7); // #RRGGBB format

            // Colors should be consistent for ranges
            match complexity {
                1..=3 => prop_assert_eq!(color, "#90EE90"),
                4..=7 => prop_assert_eq!(color, "#FFD700"),
                8..=12 => prop_assert_eq!(color, "#FFA500"),
                _ => prop_assert_eq!(color, "#FF6347"),
            }
        }

        #[test]
        fn edge_arrows_are_correct(
            edge_type in edge_type_strategy()
        ) {
            let generator = MermaidGenerator::default();
            let arrow = generator.get_edge_arrow(&edge_type);

            match edge_type {
                EdgeType::Calls => prop_assert_eq!(arrow, "-->"),
                EdgeType::Imports => prop_assert_eq!(arrow, "-.->"),
                EdgeType::Inherits => prop_assert_eq!(arrow, "-->|inherits|"),
                EdgeType::Implements => prop_assert_eq!(arrow, "-->|implements|"),
                EdgeType::Uses => prop_assert_eq!(arrow, "---"),
            }
        }
    }
}
