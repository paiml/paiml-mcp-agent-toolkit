use crate::models::dag::{DependencyGraph, EdgeType, NodeInfo, NodeType};
use crate::services::fixed_graph_builder::{FixedGraphBuilder, GraphConfig};
use crate::services::semantic_naming::SemanticNamer;
use std::fmt::Write;

pub struct MermaidGenerator {
    options: MermaidOptions,
    namer: SemanticNamer,
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
        Self {
            options,
            namer: SemanticNamer::new(),
        }
    }

    pub fn generate(&self, graph: &DependencyGraph) -> String {
        // Use the deterministic fixed graph builder
        let config = GraphConfig {
            max_nodes: 50,  // Default reasonable limit
            max_edges: 400, // Below Mermaid's 500 edge limit
            grouping: crate::services::fixed_graph_builder::GroupingStrategy::Module,
        };
        self.generate_with_config(graph, &config)
    }

    pub fn generate_with_config(&self, graph: &DependencyGraph, config: &GraphConfig) -> String {
        // Build fixed-size graph with PageRank selection
        let builder = FixedGraphBuilder::new(config.clone());
        let fixed_graph = match builder.build(graph) {
            Ok(fixed) => fixed,
            Err(_) => {
                // Fallback to original implementation if builder fails
                return self.generate_legacy(graph);
            }
        };

        let mut output = String::from("graph TD\n");

        // Generate nodes with deterministic ordering
        for node in fixed_graph.nodes.values() {
            let sanitized_id = self.sanitize_id(&node.id);
            let escaped_label = self.escape_mermaid_label(&node.display_name);

            // Generate node with proper shape based on type
            let node_def = match node.node_type {
                NodeType::Module => format!("{sanitized_id}[{escaped_label}]"),
                NodeType::Function => format!("{sanitized_id}[{escaped_label}]"),
                NodeType::Class => format!("{sanitized_id}[{escaped_label}]"),
                NodeType::Trait => format!("{sanitized_id}(({escaped_label}))"),
                NodeType::Interface => format!("{sanitized_id}(({escaped_label}))"),
            };

            writeln!(output, "    {node_def}").unwrap();
        }

        // Add blank line between nodes and edges
        output.push('\n');

        // Generate edges
        for edge in &fixed_graph.edges {
            let arrow = self.get_edge_arrow(&edge.edge_type);
            writeln!(
                output,
                "    {} {} {}",
                self.sanitize_id(&edge.from),
                arrow,
                self.sanitize_id(&edge.to)
            )
            .unwrap();
        }

        // Add styling based on complexity if enabled
        if self.options.show_complexity {
            output.push('\n');
            for node in fixed_graph.nodes.values() {
                let color = self.get_complexity_color(node.complexity as u32);
                let (stroke_style, stroke_width) = self.get_node_stroke_style(&node.node_type);

                writeln!(
                    output,
                    "    style {} fill:{}{},stroke-width:{}px",
                    self.sanitize_id(&node.id),
                    color,
                    stroke_style,
                    stroke_width
                )
                .unwrap();
            }
        }

        output
    }

    // Legacy implementation for fallback
    fn generate_legacy(&self, graph: &DependencyGraph) -> String {
        let mut output = String::from("graph TD\n");

        // Generate nodes
        self.generate_nodes(graph, &mut output);

        // Add blank line between nodes and edges
        output.push('\n');

        // Generate edges
        self.generate_edges(graph, &mut output);

        // Add styling based on complexity if enabled
        if self.options.show_complexity {
            output.push('\n');
            self.generate_styles(graph, &mut output);
        }

        output
    }

    fn generate_nodes(&self, graph: &DependencyGraph, output: &mut String) {
        for (id, node) in &graph.nodes {
            let sanitized_id = self.sanitize_id(id);
            let semantic_name = self.get_semantic_name(id, node);
            let escaped_label = self.escape_mermaid_label(&semantic_name);

            // Generate node with proper shape based on type
            let node_def = match node.node_type {
                NodeType::Module => {
                    format!("{sanitized_id}[{escaped_label}]")
                }
                NodeType::Function => {
                    format!("{sanitized_id}[{escaped_label}]")
                }
                NodeType::Class => {
                    format!("{sanitized_id}[{escaped_label}]")
                }
                NodeType::Trait => {
                    format!("{sanitized_id}(({escaped_label}))")
                }
                NodeType::Interface => {
                    format!("{sanitized_id}(({escaped_label}))")
                }
            };

            writeln!(output, "    {node_def}").unwrap();
        }
    }

    #[inline]
    pub fn escape_mermaid_label(&self, label: &str) -> String {
        // For IntelliJ compatibility, use simple character replacements instead of HTML entities
        label
            .replace('&', " and ")
            .replace('"', "'")
            .replace('<', "(")
            .replace('>', ")")
            .replace('|', " - ")
            .replace('[', "(")
            .replace(']', ")")
            .replace('{', "(")
            .replace('}', ")")
            .replace('\n', " ")
    }

    fn generate_edges(&self, graph: &DependencyGraph, output: &mut String) {
        for edge in &graph.edges {
            if graph.nodes.contains_key(&edge.from) && graph.nodes.contains_key(&edge.to) {
                let arrow = self.get_edge_arrow(&edge.edge_type);
                writeln!(
                    output,
                    "    {} {} {}",
                    self.sanitize_id(&edge.from),
                    arrow,
                    self.sanitize_id(&edge.to)
                )
                .unwrap();
            }
        }
    }

    #[inline]
    pub fn get_edge_arrow(&self, edge_type: &EdgeType) -> &'static str {
        match edge_type {
            EdgeType::Calls => "-->",
            EdgeType::Imports => "-.->",
            EdgeType::Inherits => "-->|inherits|",
            EdgeType::Implements => "-->|implements|",
            EdgeType::Uses => "---",
        }
    }

    fn generate_styles(&self, graph: &DependencyGraph, output: &mut String) {
        for (id, node) in &graph.nodes {
            let color = self.get_complexity_color(node.complexity);
            let (stroke_style, stroke_width) = self.get_node_stroke_style(&node.node_type);

            writeln!(
                output,
                "    style {} fill:{}{},stroke-width:{}px",
                self.sanitize_id(id),
                color,
                stroke_style,
                stroke_width
            )
            .unwrap();
        }
    }

    #[inline]
    pub fn get_complexity_color(&self, complexity: u32) -> &'static str {
        match complexity {
            1..=3 => "#90EE90",  // Light green for low complexity
            4..=7 => "#FFD700",  // Gold for medium complexity
            8..=12 => "#FFA500", // Orange for high complexity
            _ => "#FF6347",      // Tomato for very high complexity
        }
    }

    fn get_node_stroke_style(&self, node_type: &NodeType) -> (&'static str, u32) {
        match node_type {
            NodeType::Function => (",stroke:#333,stroke-dasharray: 5 5", 2), // Dashed border for functions
            NodeType::Trait => (",stroke:#663399", 3), // Purple border for traits
            NodeType::Interface => (",stroke:#4169E1", 3), // Blue border for interfaces
            _ => ("", 2),                              // Default for others
        }
    }

    #[inline]
    pub fn sanitize_id(&self, id: &str) -> String {
        // First replace common multi-character patterns
        let sanitized = id.replace("::", "_").replace(['/', '.', '-', ' '], "_");

        // Then replace any remaining non-alphanumeric characters with underscores
        let sanitized: String = sanitized
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        // Ensure it starts with a letter or underscore
        if sanitized.is_empty() {
            "_empty".to_string()
        } else if sanitized.chars().next().unwrap().is_numeric() {
            format!("_{sanitized}")
        } else {
            sanitized
        }
    }

    fn get_semantic_name(&self, id: &str, node: &NodeInfo) -> String {
        self.namer.get_semantic_name(id, node)
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
    use rustc_hash::FxHashMap;
    use std::fs;

    const REFERENCE_STANDARD_PATH: &str = "../artifacts/mermaid/fixtures/reference_standard.mmd";
    const COMPLEX_STYLED_STANDARD_PATH: &str =
        "../artifacts/mermaid/fixtures/complex_styled_standard.mmd";
    const INVALID_EXAMPLE_PATH: &str = "../artifacts/mermaid/fixtures/INVALID_example_diagram.mmd";

    /// Load and validate that our reference standards are syntactically correct
    fn load_reference_standard() -> String {
        fs::read_to_string(REFERENCE_STANDARD_PATH)
            .expect("Could not read reference standard - run from project root")
    }

    fn load_complex_styled_standard() -> String {
        fs::read_to_string(COMPLEX_STYLED_STANDARD_PATH)
            .expect("Could not read complex styled standard - run from project root")
    }

    fn load_invalid_example() -> String {
        fs::read_to_string(INVALID_EXAMPLE_PATH)
            .expect("Could not read invalid example - run from project root")
    }

    /// Validate basic syntax patterns that must be present in any valid Mermaid
    fn validate_mermaid_syntax(content: &str) {
        validate_mermaid_directive(content);
        validate_content_not_empty(content);
        validate_no_raw_angle_brackets(content);
        validate_node_definitions(content);
    }

    fn validate_mermaid_directive(content: &str) {
        // Must start with flowchart or graph directive
        assert!(
            content.trim_start().starts_with("flowchart")
                || content.trim_start().starts_with("graph")
        );
    }

    fn validate_content_not_empty(content: &str) {
        // Must contain at least one line with content
        assert!(content.lines().any(|line| !line.trim().is_empty()));
    }

    fn validate_no_raw_angle_brackets(content: &str) {
        // Should not contain raw angle brackets or problematic characters outside of valid contexts
        let has_raw_brackets = content.lines().any(|line| {
            let line = line.trim();
            (line.contains('<') || line.contains('>'))
                && !line.contains("-->")
                && !line.contains("<-")
                && !line.contains("[]")
                && !line.contains("()")
                && !line.contains("{}")
        });
        assert!(
            !has_raw_brackets,
            "Found raw angle brackets outside valid contexts"
        );
    }

    fn validate_node_definitions(content: &str) {
        // Check for invalid node definition pattern: raw text after node ID without proper brackets
        // Valid: node_id[Label], node_id{Label}, node_id(Label)
        // Invalid: node_id Label (raw text directly after ID)
        let has_invalid_node_definitions = content.lines().any(|line| {
            let line = line.trim();
            // Skip empty lines, edges (contain -->), style lines, classDef lines, class lines
            if line.is_empty()
                || line.contains("-->")
                || line.contains("-.->")
                || line.contains("---")
                || line.starts_with("style ")
                || line.starts_with("classDef ")
                || line.starts_with("class ")
                || line.starts_with("graph ")
                || line.starts_with("flowchart ")
            {
                return false;
            }

            // Check if line contains a node ID followed by raw text (problematic pattern)
            // Look for pattern: word_characters followed by space and then more characters
            // but NOT followed by valid arrow patterns
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let first_part = parts[0];
                let rest = &parts[1..];

                // If first part looks like a node ID (alphanumeric + underscores)
                // and there's text after it that's not an arrow or valid syntax
                if first_part.chars().all(|c| c.is_alphanumeric() || c == '_')
                    && !rest.is_empty()
                    && !rest[0].starts_with("-->")
                    && !rest[0].starts_with("-.->")
                    && !rest[0].starts_with("---")
                    && !rest[0].starts_with("[")
                    && !rest[0].starts_with("{")
                    && !rest[0].starts_with("(")
                {
                    return true;
                }
            }
            false
        });
        assert!(
            !has_invalid_node_definitions,
            "Found invalid node definitions with raw text after node ID"
        );
    }

    #[test]
    #[ignore = "Artifact files removed"]
    fn test_reference_standards_are_valid() {
        let reference = load_reference_standard();
        let complex = load_complex_styled_standard();

        validate_mermaid_syntax(&reference);
        validate_mermaid_syntax(&complex);

        // Reference standard should be simple
        assert!(reference.contains("flowchart TD"));
        assert!(reference.contains("-->"));

        // Complex standard should have styling
        assert!(complex.contains("classDef"));
        assert!(complex.contains("class"));
    }

    #[test]
    #[ignore = "Artifact files removed"]
    fn test_invalid_example_is_correctly_identified() {
        let invalid_content = load_invalid_example();

        // The invalid example should fail our validation
        // It contains problematic syntax like "cache_rs_Cache_K_V_ Interface: Cache(K,V)  -  Complexity: 6"
        // which is raw text after a node ID without proper brackets
        let result = std::panic::catch_unwind(|| {
            validate_mermaid_syntax(&invalid_content);
        });

        // Should panic because it's invalid syntax
        assert!(
            result.is_err(),
            "Invalid example should fail validation but was accepted as valid"
        );

        // Verify it contains the problematic patterns we expect to catch
        assert!(invalid_content.contains("Interface: Cache(K,V)"));
        assert!(invalid_content.contains("Class: HttpServer"));
        assert!(invalid_content.contains("Module: ConfigManager"));
    }

    #[test]
    #[ignore = "Artifact files removed"]
    fn test_generated_output_matches_reference_syntax() {
        let mut graph = DependencyGraph::new();

        graph.add_node(NodeInfo {
            id: "main.rs::main".to_string(),
            label: "main".to_string(),
            node_type: NodeType::Function,
            file_path: "main.rs".to_string(),
            line_number: 1,
            complexity: 2,
            metadata: FxHashMap::default(),
        });

        graph.add_node(NodeInfo {
            id: "lib.rs::process".to_string(),
            label: "process".to_string(),
            node_type: NodeType::Function,
            file_path: "lib.rs".to_string(),
            line_number: 10,
            complexity: 5,
            metadata: FxHashMap::default(),
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

        // Validate against reference standards
        validate_mermaid_syntax(&output);

        // Should follow basic structural patterns from reference
        let _reference = load_reference_standard();
        assert!(
            output.contains("-->"),
            "Should contain arrow connections like reference"
        );

        // Should contain proper node identifiers
        assert!(output.contains("main_rs_main"));
        assert!(output.contains("lib_rs_process"));
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
            metadata: FxHashMap::default(),
        });

        graph.add_node(NodeInfo {
            id: "lib.rs::MyClass".to_string(),
            label: "MyClass".to_string(),
            node_type: NodeType::Class,
            file_path: "lib.rs".to_string(),
            line_number: 10,
            complexity: 5,
            metadata: FxHashMap::default(),
        });

        graph.add_node(NodeInfo {
            id: "main.rs::my_function".to_string(),
            label: "my_function".to_string(),
            node_type: NodeType::Function,
            file_path: "main.rs".to_string(),
            line_number: 20,
            complexity: 3,
            metadata: FxHashMap::default(),
        });

        graph.add_node(NodeInfo {
            id: "traits.rs::MyTrait".to_string(),
            label: "MyTrait".to_string(),
            node_type: NodeType::Trait,
            file_path: "traits.rs".to_string(),
            line_number: 30,
            complexity: 2,
            metadata: FxHashMap::default(),
        });

        graph.add_node(NodeInfo {
            id: "interfaces.rs::MyInterface".to_string(),
            label: "MyInterface".to_string(),
            node_type: NodeType::Interface,
            file_path: "interfaces.rs".to_string(),
            line_number: 40,
            complexity: 4,
            metadata: FxHashMap::default(),
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

        // Check that nodes are present without special formatting for maximum compatibility
        assert!(output.contains("mod_rs_MyModule")); // Module node ID is present
        assert!(output.contains("lib_rs_MyClass")); // Class node ID is present
        assert!(output.contains("main_rs_my_function")); // Function node ID is present
        assert!(output.contains("traits_rs_MyTrait")); // Trait node ID is present
        assert!(output.contains("interfaces_rs_MyInterface")); // Interface node ID is present

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
            metadata: FxHashMap::default(),
        });

        graph.add_node(NodeInfo {
            id: "test.rs::process_data".to_string(),
            label: "process_data(input: &str)".to_string(),
            node_type: NodeType::Function,
            file_path: "test.rs".to_string(),
            line_number: 10,
            complexity: 15,
            metadata: FxHashMap::default(),
        });

        let generator = MermaidGenerator::new(MermaidOptions {
            show_complexity: true,
            ..Default::default()
        });

        let output = generator.generate(&graph);

        // Check that node IDs are present (labels removed for compatibility)
        assert!(output.contains("test_rs_handle_request"));
        assert!(output.contains("test_rs_process_data"));

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
            metadata: FxHashMap::default(),
        });

        graph.add_node(NodeInfo {
            id: "b".to_string(),
            label: "B".to_string(),
            node_type: NodeType::Class,
            file_path: "b.rs".to_string(),
            line_number: 1,
            complexity: 1,
            metadata: FxHashMap::default(),
        });

        // Add different edge types
        let edge_types = [
            (EdgeType::Calls, "-->"),
            (EdgeType::Imports, "-.->"),
            (EdgeType::Inherits, "-->|inherits|"),
            (EdgeType::Implements, "-->|implements|"),
            (EdgeType::Uses, "---"),
        ];

        for (edge_type, expected_arrow) in edge_types.iter() {
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
                "Edge type {edge_type:?} should produce arrow {expected_arrow}"
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
            metadata: FxHashMap::default(),
        });

        let generator = MermaidGenerator::new(MermaidOptions {
            show_complexity: true,
            ..Default::default()
        });

        let output = generator.generate(&graph);

        // Complexity 1 should not show the indicator
        assert!(!output.contains("Complexity: 1"));
        assert!(output.contains("test_rs_simple")); // Node ID is present
    }

    #[test]
    fn test_without_complexity_display() {
        let mut graph = DependencyGraph::new();

        graph.add_node(NodeInfo {
            id: "test.rs::complex".to_string(),
            label: "complex_function".to_string(),
            node_type: NodeType::Function,
            file_path: "test.rs".to_string(),
            line_number: 1,
            complexity: 10,
            metadata: FxHashMap::default(),
        });

        let generator = MermaidGenerator::new(MermaidOptions {
            show_complexity: false,
            ..Default::default()
        });

        let output = generator.generate(&graph);

        // Should not contain complexity when disabled
        assert!(!output.contains("Complexity:"));
        assert!(output.contains("test_rs_complex")); // Node ID is present
                                                     // Should not have styling section
        assert!(!output.contains("style test_rs_complex"));
    }

    #[test]
    fn test_edge_with_missing_node() {
        let mut graph = DependencyGraph::new();

        graph.add_node(NodeInfo {
            id: "a".to_string(),
            label: "NodeA".to_string(),
            node_type: NodeType::Class,
            file_path: "a.rs".to_string(),
            line_number: 1,
            complexity: 1,
            metadata: FxHashMap::default(),
        });

        // Add edge where 'b' doesn't exist
        graph.add_edge(Edge {
            from: "a".to_string(),
            to: "b".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1,
        });

        let generator = MermaidGenerator::new(MermaidOptions::default());
        let output = generator.generate(&graph);

        // Should contain node a
        assert!(output.contains("a")); // Node ID is present
                                       // Should NOT contain the edge since b doesn't exist
        assert!(!output.contains("a --> b"));
    }

    #[test]
    fn test_default_implementation() {
        let generator = MermaidGenerator::default();
        let graph = DependencyGraph::new();
        let output = generator.generate(&graph);

        assert_eq!(output.trim(), "graph TD");
    }

    #[test]
    fn test_numeric_id_sanitization() {
        let generator = MermaidGenerator::new(MermaidOptions::default());

        // More edge cases for sanitization
        assert_eq!(generator.sanitize_id(""), "_empty");
        assert_eq!(generator.sanitize_id("9abc"), "_9abc");
        assert_eq!(generator.sanitize_id("a-b.c/d::e"), "a_b_c_d_e");
        assert_eq!(generator.sanitize_id("¡Hola!"), "_Hola_");
        assert_eq!(generator.sanitize_id("你好"), "__");
    }

    #[test]
    fn test_options_configuration() {
        let options = MermaidOptions {
            max_depth: Some(5),
            filter_external: true,
            group_by_module: true,
            show_complexity: true,
        };

        let generator = MermaidGenerator::new(options);

        // Test that options are stored correctly
        assert!(generator.options.show_complexity);
        assert!(generator.options.filter_external);
        assert!(generator.options.group_by_module);
        assert_eq!(generator.options.max_depth, Some(5));
    }

    #[test]
    fn test_mermaid_output_format() {
        let mut graph = DependencyGraph::new();

        // Add a simple node
        graph.add_node(NodeInfo {
            id: "test".to_string(),
            label: "Test".to_string(),
            node_type: NodeType::Module,
            file_path: "test.rs".to_string(),
            line_number: 1,
            complexity: 3,
            metadata: FxHashMap::default(),
        });

        let generator = MermaidGenerator::new(MermaidOptions {
            show_complexity: true,
            ..Default::default()
        });

        let output = generator.generate(&graph);

        // Check basic structure
        assert!(output.starts_with("graph TD\n"));
        assert!(output.contains("test")); // Node ID is present

        // Check that styling is present when show_complexity is true
        assert!(output.contains("style test fill:#90EE90"));
    }

    #[test]
    fn test_regression_empty_nodes_bug() {
        // This specific test ensures we never regress to empty nodes
        let mut graph = DependencyGraph::new();

        // Add problematic node labels from real-world cases
        let test_cases = vec![
            ("fn|process", "Function with pipe"),
            ("struct <T>", "Generic struct"),
            ("impl Display for &'a str", "Complex impl"),
            ("async fn handle_request()", "Async function"),
            ("mod tests { #[test] }", "Module with attributes"),
            ("trait Iterator<Item=T>", "Associated type"),
            ("use std::io::{Read, Write}", "Multiple imports"),
        ];

        for (id, label) in test_cases.iter() {
            graph.add_node(NodeInfo {
                id: id.to_string(),
                label: label.to_string(),
                node_type: NodeType::Function,
                file_path: "test.rs".to_string(),
                line_number: 1,
                complexity: 5,
                metadata: FxHashMap::default(),
            });
        }

        let generator = MermaidGenerator::new(MermaidOptions::default());
        let output = generator.generate(&graph);

        // Verify each label appears in output (escaped)
        for (id, _label) in test_cases {
            let sanitized_id = generator.sanitize_id(id);
            // Verify the node has a label (not just a bare ID)
            assert!(
                output.contains(&format!("{sanitized_id}[")),
                "Node '{id}' is missing its label brackets in output"
            );
        }

        // Verify no bare IDs (nodes without labels)
        let lines: Vec<&str> = output.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            // Skip empty lines, the graph directive, edges, and style lines
            if trimmed.is_empty()
                || trimmed.starts_with("graph")
                || trimmed.contains("-->")
                || trimmed.contains("-.->")
                || trimmed.contains("---")
                || trimmed.starts_with("style")
            {
                continue;
            }

            // Node lines should have brackets or parentheses
            if !trimmed.contains("graph") {
                assert!(
                    trimmed.contains('[') || trimmed.contains("(("),
                    "Found bare node ID without label: {trimmed}"
                );
            }
        }
    }

    #[test]
    fn test_escape_mermaid_label() {
        let generator = MermaidGenerator::default();

        // Test special character escaping
        assert_eq!(generator.escape_mermaid_label("simple"), "simple");
        assert_eq!(generator.escape_mermaid_label("with|pipe"), "with - pipe");
        assert_eq!(
            generator.escape_mermaid_label("with\"quotes\""),
            "with'quotes'"
        );
        assert_eq!(
            generator.escape_mermaid_label("with'apostrophe"),
            "with'apostrophe"
        );
        assert_eq!(
            generator.escape_mermaid_label("with[brackets]"),
            "with(brackets)"
        );
        assert_eq!(
            generator.escape_mermaid_label("with{braces}"),
            "with(braces)"
        );
        assert_eq!(generator.escape_mermaid_label("with<angle>"), "with(angle)");
        assert_eq!(
            generator.escape_mermaid_label("with&ampersand"),
            "with and ampersand"
        );
        assert_eq!(generator.escape_mermaid_label("line\nbreak"), "line break");
        assert_eq!(
            generator.escape_mermaid_label("Function: test | Complexity: 5"),
            "Function: test  -  Complexity: 5"
        );
    }

    /// Validation tests for real-world Mermaid parser compatibility
    #[cfg(test)]
    mod validation_tests {
        use super::*;
        use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};

        /// Test characters that caused the IntelliJ parse error:
        /// "Parse error on line 2: ...cache_rs_Cache_K_V_ [Interface: Cache(K,V)  -  ..."
        #[test]
        fn test_angle_brackets_and_pipes_compatibility() {
            let mut graph = DependencyGraph::new();

            // This exact combination caused parse errors in IntelliJ
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

            // Ensure problematic characters are properly escaped
            assert!(!output.contains("<")); // No raw angle brackets
            assert!(!output.contains(">")); // No raw angle brackets
            assert!(!output.contains("&#")); // No HTML entities
                                             // Check that node ID is properly sanitized

            // Should be parseable - node ID is present
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

                // All arrows must contain at least one dash
                assert!(
                    arrow.contains("-"),
                    "Arrow '{arrow}' for {edge_type:?} must contain dash"
                );

                // Basic arrow syntax validation - all arrows should contain dashes
                // and only labeled edges should contain pipes/spaces
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

            // Add typical Rust project nodes
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

            // Add realistic edges
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

            // Should be well-formed Mermaid syntax
            assert!(output.starts_with("graph TD\n"));
            assert!(output.contains("style ")); // Should have styling
            assert!(!output.contains("<")); // No raw angle brackets
            assert!(!output.contains("&")); // No raw ampersands
            assert!(
                !output.contains("|")
                    || output.contains("implements")
                    || output.contains("inherits")
            ); // Pipes only in edge labels

            // All nodes should be present
            assert!(output.contains("main_rs_main"));
            assert!(output.contains("lib_rs_Config"));
            assert!(output.contains("api_rs_Handler_T_"));

            // Complexity styling should be present for complex nodes
            assert!(output.contains("#FFD700")); // Gold for medium complexity
            assert!(output.contains("#FFA500")); // Orange for high complexity
        }
    }
}

// Include property-based tests
#[cfg(test)]
#[path = "mermaid_property_tests.rs"]
mod property_tests;
