use pmat::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
use pmat::services::mermaid_generator::{MermaidGenerator, MermaidOptions};
use rustc_hash::FxHashMap;

#[test]
fn generate_test_mermaid() {
    let mut graph = DependencyGraph::new();

    // Add comprehensive example nodes
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
        id: "config.rs::ConfigManager".to_string(),
        label: "ConfigManager".to_string(),
        node_type: NodeType::Module,
        file_path: "config.rs".to_string(),
        line_number: 1,
        complexity: 5,
        metadata: FxHashMap::default(),
    });

    graph.add_node(NodeInfo {
        id: "server.rs::HttpServer".to_string(),
        label: "HttpServer".to_string(),
        node_type: NodeType::Class,
        file_path: "server.rs".to_string(),
        line_number: 10,
        complexity: 15,
        metadata: FxHashMap::default(),
    });

    graph.add_node(NodeInfo {
        id: "auth.rs::Authenticator".to_string(),
        label: "Authenticator".to_string(),
        node_type: NodeType::Trait,
        file_path: "auth.rs".to_string(),
        line_number: 5,
        complexity: 4,
        metadata: FxHashMap::default(),
    });

    graph.add_node(NodeInfo {
        id: "cache.rs::Cache<K,V>".to_string(),
        label: "Cache<K,V>".to_string(),
        node_type: NodeType::Interface,
        file_path: "cache.rs".to_string(),
        line_number: 1,
        complexity: 6,
        metadata: FxHashMap::default(),
    });

    // Add edges with different types
    graph.add_edge(Edge {
        from: "main.rs::main".to_string(),
        to: "server.rs::HttpServer".to_string(),
        edge_type: EdgeType::Calls,
        weight: 1,
    });

    graph.add_edge(Edge {
        from: "main.rs::main".to_string(),
        to: "config.rs::ConfigManager".to_string(),
        edge_type: EdgeType::Imports,
        weight: 1,
    });

    graph.add_edge(Edge {
        from: "server.rs::HttpServer".to_string(),
        to: "auth.rs::Authenticator".to_string(),
        edge_type: EdgeType::Implements,
        weight: 1,
    });

    graph.add_edge(Edge {
        from: "cache.rs::Cache<K,V>".to_string(),
        to: "auth.rs::Authenticator".to_string(),
        edge_type: EdgeType::Inherits,
        weight: 1,
    });

    graph.add_edge(Edge {
        from: "config.rs::ConfigManager".to_string(),
        to: "cache.rs::Cache<K,V>".to_string(),
        edge_type: EdgeType::Uses,
        weight: 1,
    });

    let generator = MermaidGenerator::new(MermaidOptions {
        show_complexity: true,
        ..Default::default()
    });

    let output = generator.generate(&graph);

    // Write to artifacts directory
    let artifacts_dir = std::path::Path::new("../../artifacts/mermaid/test_output");
    std::fs::create_dir_all(artifacts_dir).ok();
    let output_path = artifacts_dir.join("intellij_test_updated.mmd");
    std::fs::write(&output_path, &output).unwrap();

    println!("Generated updated test Mermaid file: {output_path:?}");
    println!("{output}");
}
