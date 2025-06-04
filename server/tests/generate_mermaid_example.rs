use paiml_mcp_agent_toolkit::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
use paiml_mcp_agent_toolkit::services::mermaid_generator::{MermaidGenerator, MermaidOptions};
use std::collections::HashMap;
use std::fs;

#[test]
fn generate_example_mermaid_diagram() {
    // Create a sample dependency graph
    let mut graph = DependencyGraph::new();

    // Add nodes representing different components
    graph.add_node(NodeInfo {
        id: "main.rs::main".to_string(),
        label: "main".to_string(),
        node_type: NodeType::Function,
        file_path: "src/bin/main.rs".to_string(),
        line_number: 10,
        complexity: 5,
        metadata: HashMap::new(),
    });

    graph.add_node(NodeInfo {
        id: "lib.rs::TemplateServer".to_string(),
        label: "TemplateServer".to_string(),
        node_type: NodeType::Class,
        file_path: "src/lib.rs".to_string(),
        line_number: 25,
        complexity: 12,
        metadata: HashMap::new(),
    });

    graph.add_node(NodeInfo {
        id: "mermaid_generator.rs::MermaidGenerator".to_string(),
        label: "MermaidGenerator".to_string(),
        node_type: NodeType::Class,
        file_path: "src/services/mermaid_generator.rs".to_string(),
        line_number: 4,
        complexity: 8,
        metadata: HashMap::new(),
    });

    graph.add_node(NodeInfo {
        id: "dag.rs::DependencyGraph".to_string(),
        label: "DependencyGraph".to_string(),
        node_type: NodeType::Class,
        file_path: "src/models/dag.rs".to_string(),
        line_number: 5,
        complexity: 3,
        metadata: HashMap::new(),
    });

    graph.add_node(NodeInfo {
        id: "template_service.rs::TemplateService".to_string(),
        label: "TemplateService".to_string(),
        node_type: NodeType::Trait,
        file_path: "src/services/template_service.rs".to_string(),
        line_number: 15,
        complexity: 6,
        metadata: HashMap::new(),
    });

    graph.add_node(NodeInfo {
        id: "embedded_templates.rs::get_template".to_string(),
        label: "get_template".to_string(),
        node_type: NodeType::Function,
        file_path: "src/services/embedded_templates.rs".to_string(),
        line_number: 100,
        complexity: 15,
        metadata: HashMap::new(),
    });

    graph.add_node(NodeInfo {
        id: "ast_parser::Parser".to_string(),
        label: "Parser<T>".to_string(),
        node_type: NodeType::Interface,
        file_path: "src/ast/parser.rs".to_string(),
        line_number: 50,
        complexity: 20,
        metadata: HashMap::new(),
    });

    // Add edges representing relationships
    graph.add_edge(Edge {
        from: "main.rs::main".to_string(),
        to: "lib.rs::TemplateServer".to_string(),
        edge_type: EdgeType::Uses,
        weight: 1,
    });

    graph.add_edge(Edge {
        from: "lib.rs::TemplateServer".to_string(),
        to: "template_service.rs::TemplateService".to_string(),
        edge_type: EdgeType::Implements,
        weight: 1,
    });

    graph.add_edge(Edge {
        from: "mermaid_generator.rs::MermaidGenerator".to_string(),
        to: "dag.rs::DependencyGraph".to_string(),
        edge_type: EdgeType::Uses,
        weight: 1,
    });

    graph.add_edge(Edge {
        from: "main.rs::main".to_string(),
        to: "mermaid_generator.rs::MermaidGenerator".to_string(),
        edge_type: EdgeType::Calls,
        weight: 1,
    });

    graph.add_edge(Edge {
        from: "lib.rs::TemplateServer".to_string(),
        to: "embedded_templates.rs::get_template".to_string(),
        edge_type: EdgeType::Calls,
        weight: 1,
    });

    graph.add_edge(Edge {
        from: "ast_parser::Parser".to_string(),
        to: "dag.rs::DependencyGraph".to_string(),
        edge_type: EdgeType::Imports,
        weight: 1,
    });

    graph.add_edge(Edge {
        from: "mermaid_generator.rs::MermaidGenerator".to_string(),
        to: "template_service.rs::TemplateService".to_string(),
        edge_type: EdgeType::Inherits,
        weight: 1,
    });

    // Generate Mermaid diagram with complexity colors
    let generator = MermaidGenerator::new(MermaidOptions {
        show_complexity: true,
        ..Default::default()
    });

    let mermaid_output = generator.generate(&graph);

    // Save to artifacts directory
    let artifacts_dir = std::path::Path::new("../../artifacts/mermaid/test_output");
    fs::create_dir_all(artifacts_dir).ok();
    let output_path = artifacts_dir.join("dependency_graph.mmd");
    fs::write(&output_path, &mermaid_output).expect("Failed to write file");

    println!("\n=== Generated Mermaid Diagram ===\n");
    println!("{}", mermaid_output);
    println!("\n=== Saved to {:?} ===", output_path);
    println!("\nYou can open this file in IntelliJ IDEA with the Mermaid plugin installed.");
    println!("The diagram shows:");
    println!("- Different node types (Function, Class, Trait, Interface) with different shapes");
    println!("- Complexity-based coloring (green=low, yellow=medium, orange=high, red=very high)");
    println!("- Different edge types (solid=calls, dashed=imports, inheritance arrows, etc.)");

    // Verify the output contains expected elements
    assert!(mermaid_output.contains("graph TD"));
    assert!(mermaid_output.contains("main_rs_main"));
    assert!(mermaid_output.contains("lib_rs_TemplateServer"));
    assert!(mermaid_output.contains("-->")); // Calls arrow
    assert!(mermaid_output.contains("-.->")); // Imports arrow
    assert!(mermaid_output.contains("-->|inherits|")); // Inherits arrow
    assert!(mermaid_output.contains("-->|implements|")); // Implements arrow
    assert!(mermaid_output.contains("---")); // Uses arrow
}
