#![cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
    use crate::services::mermaid_generator::{MermaidGenerator, MermaidOptions};
    use rustc_hash::FxHashMap;
    use std::fs;

    const REFERENCE_STANDARD_PATH: &str = "../artifacts/mermaid/fixtures/reference_standard.mmd";
    const COMPLEX_STYLED_STANDARD_PATH: &str =
        "../artifacts/mermaid/fixtures/complex_styled_standard.mmd";
    const INVALID_EXAMPLE_PATH: &str = "../artifacts/mermaid/fixtures/INVALID_example_diagram.mmd";

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

    fn validate_mermaid_syntax(content: &str) {
        validate_mermaid_directive(content);
        validate_content_not_empty(content);
        validate_no_raw_angle_brackets(content);
        validate_node_definitions(content);
    }

    fn validate_mermaid_directive(content: &str) {
        assert!(
            content.trim_start().starts_with("flowchart")
                || content.trim_start().starts_with("graph")
        );
    }

    fn validate_content_not_empty(content: &str) {
        assert!(content.lines().any(|line| !line.trim().is_empty()));
    }

    fn validate_no_raw_angle_brackets(content: &str) {
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
        let has_invalid_node_definitions = content.lines().any(|line| {
            let line = line.trim();
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

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let first_part = parts[0];
                let rest = &parts[1..];

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

        assert!(reference.contains("flowchart TD"));
        assert!(reference.contains("-->"));

        assert!(complex.contains("classDef"));
        assert!(complex.contains("class"));
    }

    #[test]
    #[ignore = "Artifact files removed"]
    fn test_invalid_example_is_correctly_identified() {
        let invalid_content = load_invalid_example();

        let result = std::panic::catch_unwind(|| {
            validate_mermaid_syntax(&invalid_content);
        });

        assert!(
            result.is_err(),
            "Invalid example should fail validation but was accepted as valid"
        );

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

        validate_mermaid_syntax(&output);

        let _reference = load_reference_standard();
        assert!(
            output.contains("-->"),
            "Should contain arrow connections like reference"
        );

        assert!(output.contains("main_rs_main"));
        assert!(output.contains("lib_rs_process"));
    }

    #[test]
    fn test_all_node_types() {
        let mut graph = DependencyGraph::new();

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

        assert!(output.contains("mod_rs_MyModule"));
        assert!(output.contains("lib_rs_MyClass"));
        assert!(output.contains("main_rs_my_function"));
        assert!(output.contains("traits_rs_MyTrait"));
        assert!(output.contains("interfaces_rs_MyInterface"));

        assert!(output.contains("stroke-dasharray: 5 5")); // Functions have dashed border
        assert!(output.contains("stroke:#663399")); // Traits have purple border
        assert!(output.contains("stroke:#4169E1")); // Interfaces have blue border
    }

    #[test]
    fn test_complex_labels() {
        let mut graph = DependencyGraph::new();

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

        assert!(output.contains("test_rs_handle_request"));
        assert!(output.contains("test_rs_process_data"));

        assert!(output.contains("#FFA500")); // Orange for high complexity (10)
        assert!(output.contains("#FF6347")); // Tomato for very high complexity (15)
    }

    #[test]
    fn test_sanitize_id() {
        let generator = MermaidGenerator::new(MermaidOptions::default());

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

        assert!(!output.contains("Complexity: 1"));
        assert!(output.contains("test_rs_simple"));
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

        assert!(!output.contains("Complexity:"));
        assert!(output.contains("test_rs_complex"));
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

        graph.add_edge(Edge {
            from: "a".to_string(),
            to: "b".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1,
        });

        let generator = MermaidGenerator::new(MermaidOptions::default());
        let output = generator.generate(&graph);

        assert!(output.contains("a"));
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

        assert!(generator.options.show_complexity);
        assert!(generator.options.filter_external);
        assert!(generator.options.group_by_module);
        assert_eq!(generator.options.max_depth, Some(5));
    }
}
