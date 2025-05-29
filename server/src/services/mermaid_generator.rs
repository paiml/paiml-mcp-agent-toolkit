use crate::models::dag::{DependencyGraph, EdgeType, NodeType};
use std::fmt::Write;

pub struct MermaidGenerator {
    options: MermaidOptions,
}

#[derive(Default)]
pub struct MermaidOptions {
    pub max_depth: Option<usize>,
    pub filter_external: bool,
    pub group_by_module: bool,
    pub show_complexity: bool,
}

impl MermaidGenerator {
    pub fn new(options: MermaidOptions) -> Self {
        Self { options }
    }

    pub fn generate(&self, graph: &DependencyGraph) -> String {
        let mut output = String::from("graph TD\n");

        // Generate nodes
        for (id, node) in &graph.nodes {
            let label = if self.options.show_complexity && node.complexity > 1 {
                format!("{}<br/>⚡{}", node.label, node.complexity)
            } else {
                node.label.clone()
            };

            // Format node with appropriate shape based on type
            // Use brackets for all types and differentiate with CSS classes
            let node_def = match node.node_type {
                NodeType::Class => format!("[{}]", label),
                NodeType::Function => format!("[{}]", label), // Will use CSS to make it look rounded
                NodeType::Module => format!("{{{{{}}}}}", label),
                NodeType::Trait => format!("[{}]", label), // Will use CSS for trapezoid
                NodeType::Interface => format!("[{}]", label), // Will use CSS for inverted trapezoid
            };

            writeln!(&mut output, "    {} {}", self.sanitize_id(id), node_def).unwrap();
        }

        // Add blank line between nodes and edges
        output.push('\n');

        // Generate edges
        for edge in &graph.edges {
            let arrow = match edge.edge_type {
                EdgeType::Calls => "-->",
                EdgeType::Imports => "-.->",
                EdgeType::Inherits => "==>",
                EdgeType::Implements => "-.->>",
                EdgeType::Uses => "-->",
            };

            // Only add edge if both nodes exist in the graph
            if graph.nodes.contains_key(&edge.from) && graph.nodes.contains_key(&edge.to) {
                writeln!(
                    &mut output,
                    "    {} {} {}",
                    self.sanitize_id(&edge.from),
                    arrow,
                    self.sanitize_id(&edge.to)
                )
                .unwrap();
            }
        }

        // Add styling based on complexity if enabled
        if self.options.show_complexity {
            output.push('\n');
            for (id, node) in &graph.nodes {
                let color = match node.complexity {
                    1..=3 => "#90EE90",  // Light green for low complexity
                    4..=7 => "#FFD700",  // Gold for medium complexity
                    8..=12 => "#FFA500", // Orange for high complexity
                    _ => "#FF6347",      // Tomato for very high complexity
                };
                
                // Add node type-specific styling
                let (stroke_style, stroke_width) = match node.node_type {
                    NodeType::Function => (",stroke:#333,stroke-dasharray: 5 5", 2), // Dashed border for functions
                    NodeType::Trait => (",stroke:#663399", 3), // Purple border for traits
                    NodeType::Interface => (",stroke:#4169E1", 3), // Blue border for interfaces
                    _ => ("", 2), // Default for others
                };
                
                writeln!(
                    &mut output,
                    "    style {} fill:{}{},stroke-width:{}px",
                    self.sanitize_id(id),
                    color,
                    stroke_style,
                    stroke_width
                )
                .unwrap();
            }
        }

        output
    }

    fn sanitize_id(&self, id: &str) -> String {
        // Replace invalid Mermaid characters and ensure valid identifier
        let sanitized = id.replace("::", "_").replace(['/', '.', '-', ' '], "_");

        // Ensure it starts with a letter or underscore
        if sanitized.chars().next().is_some_and(|c| c.is_numeric()) {
            format!("_{}", sanitized)
        } else {
            sanitized
        }
    }
}

impl Default for MermaidGenerator {
    fn default() -> Self {
        Self::new(MermaidOptions::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::dag::{Edge, EdgeType, NodeInfo, NodeType};

    #[test]
    fn test_mermaid_generation() {
        let mut graph = DependencyGraph::new();

        graph.add_node(NodeInfo {
            id: "main.rs::main".to_string(),
            label: "main".to_string(),
            node_type: NodeType::Function,
            file_path: "main.rs".to_string(),
            line_number: 1,
            complexity: 2,
        });

        graph.add_node(NodeInfo {
            id: "lib.rs::process".to_string(),
            label: "process".to_string(),
            node_type: NodeType::Function,
            file_path: "lib.rs".to_string(),
            line_number: 10,
            complexity: 5,
        });

        graph.add_edge(Edge {
            from: "main.rs::main".to_string(),
            to: "lib.rs::process".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1,
        });

        let generator = MermaidGenerator::new(MermaidOptions {
            show_complexity: true,
            ..Default::default()
        });

        let output = generator.generate(&graph);

        assert!(output.contains("graph TD"));
        assert!(output.contains("main_rs_main"));
        assert!(output.contains("lib_rs_process"));
        assert!(output.contains("-->")); // Calls arrow
        assert!(output.contains("⚡")); // Complexity indicator
    }

    #[test]
    fn test_all_node_types() {
        let mut graph = DependencyGraph::new();

        // Add one of each node type
        graph.add_node(NodeInfo {
            id: "mod.rs::MyModule".to_string(),
            label: "MyModule".to_string(),
            node_type: NodeType::Module,
            file_path: "mod.rs".to_string(),
            line_number: 1,
            complexity: 1,
        });

        graph.add_node(NodeInfo {
            id: "lib.rs::MyClass".to_string(),
            label: "MyClass".to_string(),
            node_type: NodeType::Class,
            file_path: "lib.rs".to_string(),
            line_number: 10,
            complexity: 5,
        });

        graph.add_node(NodeInfo {
            id: "main.rs::my_function".to_string(),
            label: "my_function".to_string(),
            node_type: NodeType::Function,
            file_path: "main.rs".to_string(),
            line_number: 20,
            complexity: 3,
        });

        graph.add_node(NodeInfo {
            id: "traits.rs::MyTrait".to_string(),
            label: "MyTrait".to_string(),
            node_type: NodeType::Trait,
            file_path: "traits.rs".to_string(),
            line_number: 30,
            complexity: 2,
        });

        graph.add_node(NodeInfo {
            id: "interfaces.rs::MyInterface".to_string(),
            label: "MyInterface".to_string(),
            node_type: NodeType::Interface,
            file_path: "interfaces.rs".to_string(),
            line_number: 40,
            complexity: 4,
        });

        let generator = MermaidGenerator::new(MermaidOptions {
            show_complexity: true,
            ..Default::default()
        });

        let output = generator.generate(&graph);

        // Check that all nodes are present
        assert!(output.contains("mod_rs_MyModule"));
        assert!(output.contains("lib_rs_MyClass"));
        assert!(output.contains("main_rs_my_function"));
        assert!(output.contains("traits_rs_MyTrait"));
        assert!(output.contains("interfaces_rs_MyInterface"));

        // Check node definitions
        assert!(output.contains("{{MyModule}}")); // Module uses double braces
        assert!(output.contains("[MyClass<br/>⚡5]")); // Class uses brackets
        assert!(output.contains("[my_function<br/>⚡3]")); // Function uses brackets (simplified)
        assert!(output.contains("[MyTrait<br/>⚡2]")); // Trait uses brackets
        assert!(output.contains("[MyInterface<br/>⚡4]")); // Interface uses brackets

        // Check styling
        assert!(output.contains("stroke-dasharray: 5 5")); // Functions have dashed border
        assert!(output.contains("stroke:#663399")); // Traits have purple border
        assert!(output.contains("stroke:#4169E1")); // Interfaces have blue border
    }

    #[test]
    fn test_complex_labels() {
        let mut graph = DependencyGraph::new();

        // Test with special characters in labels
        graph.add_node(NodeInfo {
            id: "test.rs::handle_request".to_string(),
            label: "handle_request<T>".to_string(),
            node_type: NodeType::Function,
            file_path: "test.rs".to_string(),
            line_number: 1,
            complexity: 10,
        });

        graph.add_node(NodeInfo {
            id: "test.rs::process_data".to_string(),
            label: "process_data(input: &str)".to_string(),
            node_type: NodeType::Function,
            file_path: "test.rs".to_string(),
            line_number: 10,
            complexity: 15,
        });

        let generator = MermaidGenerator::new(MermaidOptions {
            show_complexity: true,
            ..Default::default()
        });

        let output = generator.generate(&graph);

        // Check that labels are properly included
        assert!(output.contains("handle_request<T><br/>⚡10"));
        assert!(output.contains("process_data(input: &str)<br/>⚡15"));
        
        // Check color coding
        assert!(output.contains("#FFA500")); // Orange for high complexity (10)
        assert!(output.contains("#FF6347")); // Tomato for very high complexity (15)
    }

    #[test]
    fn test_sanitize_id() {
        let generator = MermaidGenerator::new(MermaidOptions::default());
        
        // Test various ID sanitization cases
        assert_eq!(generator.sanitize_id("foo::bar"), "foo_bar");
        assert_eq!(generator.sanitize_id("foo/bar.rs"), "foo_bar_rs");
        assert_eq!(generator.sanitize_id("foo-bar"), "foo_bar");
        assert_eq!(generator.sanitize_id("foo bar"), "foo_bar");
        assert_eq!(generator.sanitize_id("123foo"), "_123foo");
        assert_eq!(generator.sanitize_id("_foo"), "_foo");
    }

    #[test]
    fn test_all_edge_types() {
        let mut graph = DependencyGraph::new();

        // Add nodes
        graph.add_node(NodeInfo {
            id: "a".to_string(),
            label: "A".to_string(),
            node_type: NodeType::Class,
            file_path: "a.rs".to_string(),
            line_number: 1,
            complexity: 1,
        });

        graph.add_node(NodeInfo {
            id: "b".to_string(),
            label: "B".to_string(),
            node_type: NodeType::Class,
            file_path: "b.rs".to_string(),
            line_number: 1,
            complexity: 1,
        });

        // Add different edge types
        let edge_types = vec![
            (EdgeType::Calls, "-->"),
            (EdgeType::Imports, "-.->"),
            (EdgeType::Inherits, "==>"),
            (EdgeType::Implements, "-.->>"),
            (EdgeType::Uses, "-->"),
        ];

        for (_i, (edge_type, expected_arrow)) in edge_types.iter().enumerate() {
            let mut test_graph = graph.clone();
            test_graph.add_edge(Edge {
                from: "a".to_string(),
                to: "b".to_string(),
                edge_type: edge_type.clone(),
                weight: 1,
            });

            let generator = MermaidGenerator::new(MermaidOptions::default());
            let output = generator.generate(&test_graph);

            assert!(
                output.contains(expected_arrow),
                "Edge type {:?} should produce arrow {}",
                edge_type,
                expected_arrow
            );
        }
    }

    #[test]
    fn test_empty_graph() {
        let graph = DependencyGraph::new();
        let generator = MermaidGenerator::new(MermaidOptions::default());
        let output = generator.generate(&graph);

        assert_eq!(output.trim(), "graph TD");
    }

    #[test]
    fn test_no_complexity_display() {
        let mut graph = DependencyGraph::new();

        graph.add_node(NodeInfo {
            id: "test.rs::simple".to_string(),
            label: "simple".to_string(),
            node_type: NodeType::Function,
            file_path: "test.rs".to_string(),
            line_number: 1,
            complexity: 1,
        });

        let generator = MermaidGenerator::new(MermaidOptions {
            show_complexity: true,
            ..Default::default()
        });

        let output = generator.generate(&graph);

        // Complexity 1 should not show the indicator
        assert!(!output.contains("⚡1"));
        assert!(output.contains("[simple]"));
    }
}
