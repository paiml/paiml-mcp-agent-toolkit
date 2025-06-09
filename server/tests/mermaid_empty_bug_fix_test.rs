use paiml_mcp_agent_toolkit::models::dag::{DependencyGraph, NodeInfo, NodeType};
use paiml_mcp_agent_toolkit::services::mermaid_generator::{MermaidGenerator, MermaidOptions};
use rustc_hash::FxHashMap;

#[test]
fn test_regression_empty_nodes_bug() {
    // This specific test ensures we never regress to empty nodes
    let mut graph = DependencyGraph::new();

    // Add problematic node labels from real-world cases
    let test_cases = vec![
        ("fn_process", "fn|process", "Function with pipe"),
        ("struct_T", "struct <T>", "Generic struct"),
        ("impl_Display", "impl Display for &'a str", "Complex impl"),
        ("async_fn", "async fn handle_request()", "Async function"),
        (
            "mod_tests",
            "mod tests { #[test] }",
            "Module with attributes",
        ),
        (
            "trait_Iterator",
            "trait Iterator<Item=T>",
            "Associated type",
        ),
        ("use_io", "use std::io::{Read, Write}", "Multiple imports"),
    ];

    for (id, label, _description) in &test_cases {
        graph.add_node(NodeInfo {
            id: id.to_string(),
            label: label.to_string(),
            node_type: NodeType::Function,
            file_path: String::new(),
            line_number: 0,
            complexity: 5,
            metadata: FxHashMap::default(),
        });
    }

    let generator = MermaidGenerator::new(MermaidOptions::default());
    let output = generator.generate(&graph);

    println!("Generated Mermaid diagram:\n{output}");

    // Verify each label appears in output (escaped)
    for (_id, label, description) in &test_cases {
        let escaped = generator.escape_mermaid_label(label);
        assert!(
            output.contains(&escaped),
            "Label '{label}' ({description}) not found in output. Escaped label: '{escaped}'"
        );
    }

    // Verify no bare IDs without labels
    for (id, _, _) in &test_cases {
        let sanitized_id = generator.sanitize_id(id);
        // The ID should appear with a bracket or parenthesis, not alone
        let bare_id_pattern = format!("    {sanitized_id}\n");
        assert!(
            !output.contains(&bare_id_pattern),
            "Found bare ID '{sanitized_id}' without label in output"
        );
    }

    // Verify proper node formatting
    assert!(output.contains("["), "No node labels with brackets found");
    assert!(output.contains("graph TD"), "Missing graph declaration");
}

#[test]
fn test_mermaid_label_escaping() {
    let generator = MermaidGenerator::new(MermaidOptions::default());

    // Test various problematic characters
    let test_cases = vec![
        ("simple", "simple"),
        ("with|pipe", "with - pipe"),
        ("with\"quote", "with'quote"),
        ("with<angle>", "with(angle)"),
        ("with[bracket]", "with(bracket)"),
        ("with{brace}", "with(brace)"),
        ("with\nnewline", "with newline"),
        ("with&ampersand", "with and ampersand"),
    ];

    for (input, expected) in test_cases {
        let escaped = generator.escape_mermaid_label(input);
        assert_eq!(
            escaped, expected,
            "Escaping '{input}' failed. Expected: '{expected}', Got: '{escaped}'"
        );
    }
}

#[test]
fn test_node_types_have_labels() {
    let mut graph = DependencyGraph::new();

    // Test each node type
    let node_types = vec![
        ("module", "TestModule", NodeType::Module),
        ("function", "test_function", NodeType::Function),
        ("class", "TestClass", NodeType::Class),
        ("trait", "TestTrait", NodeType::Trait),
        ("interface", "TestInterface", NodeType::Interface),
    ];

    for (id, label, node_type) in &node_types {
        graph.add_node(NodeInfo {
            id: id.to_string(),
            label: label.to_string(),
            node_type: node_type.clone(),
            file_path: String::new(),
            line_number: 0,
            complexity: 1,
            metadata: FxHashMap::default(),
        });
    }

    let generator = MermaidGenerator::new(MermaidOptions::default());
    let output = generator.generate(&graph);

    // Each label should appear in the output
    for (_id, label, _node_type) in &node_types {
        assert!(
            output.contains(label),
            "Node label '{label}' not found in output"
        );
    }
}

#[test]
fn test_complexity_styled_diagram_has_labels() {
    let mut graph = DependencyGraph::new();

    graph.add_node(NodeInfo {
        id: "complex_fn".to_string(),
        label: "process_data".to_string(),
        node_type: NodeType::Function,
        file_path: String::new(),
        line_number: 0,
        complexity: 15,
        metadata: FxHashMap::default(),
    });

    graph.add_node(NodeInfo {
        id: "simple_fn".to_string(),
        label: "get_value".to_string(),
        node_type: NodeType::Function,
        file_path: String::new(),
        line_number: 0,
        complexity: 2,
        metadata: FxHashMap::default(),
    });

    let generator = MermaidGenerator::new(MermaidOptions {
        show_complexity: true,
        filter_external: false,
        max_depth: None,
        group_by_module: false,
    });

    let output = generator.generate(&graph);

    // Verify labels exist
    assert!(
        output.contains("process_data"),
        "Complex function label missing"
    );
    assert!(
        output.contains("get_value"),
        "Simple function label missing"
    );

    // Verify styling exists
    assert!(output.contains("style"), "No style declarations found");
    assert!(output.contains("fill:"), "No fill colors found");
}

#[test]
fn test_empty_graph_doesnt_crash() {
    let graph = DependencyGraph::new();
    let generator = MermaidGenerator::new(MermaidOptions::default());
    let output = generator.generate(&graph);

    assert!(
        output.contains("graph TD"),
        "Empty graph should still have declaration"
    );
    assert!(!output.contains("["), "Empty graph should have no nodes");
}

#[test]
fn test_special_characters_in_node_ids() {
    let mut graph = DependencyGraph::new();

    // Test IDs that need sanitization
    let test_ids = vec![
        ("src/lib.rs::main", "main function"),
        ("module::sub-module", "nested module"),
        ("file.name.with.dots", "dotted name"),
        ("123_starts_with_number", "numeric start"),
        ("", "empty id"),
    ];

    for (id, label) in &test_ids {
        graph.add_node(NodeInfo {
            id: id.to_string(),
            label: label.to_string(),
            node_type: NodeType::Function,
            file_path: String::new(),
            line_number: 0,
            complexity: 1,
            metadata: FxHashMap::default(),
        });
    }

    let generator = MermaidGenerator::new(MermaidOptions::default());
    let output = generator.generate(&graph);

    // Verify all labels appear
    for (_id, label) in &test_ids {
        if !label.is_empty() {
            assert!(
                output.contains(label),
                "Label '{label}' not found in output"
            );
        }
    }

    // Verify no invalid characters in IDs
    assert!(!output.contains("::"), "Unescaped :: found in output");
    assert!(!output.contains("src/"), "Unescaped path separator found");
}
