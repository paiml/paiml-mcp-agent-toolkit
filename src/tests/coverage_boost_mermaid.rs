//! Coverage boost tests for services/mermaid_generator.rs
//! Tests pure functions: sanitize_id, escape_mermaid_label, get_edge_arrow, get_complexity_color

use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
use crate::services::mermaid_generator::{MermaidGenerator, MermaidOptions};
use rustc_hash::FxHashMap;

fn make_generator() -> MermaidGenerator {
    MermaidGenerator::new(MermaidOptions::default())
}

fn make_generator_with_options(show_complexity: bool, group_by_module: bool) -> MermaidGenerator {
    MermaidGenerator::new(MermaidOptions {
        max_depth: Some(5),
        filter_external: true,
        group_by_module,
        show_complexity,
    })
}

// --- MermaidOptions tests ---

#[test]
fn test_mermaid_options_default() {
    let opts = MermaidOptions::default();
    assert!(opts.max_depth.is_none());
    assert!(!opts.filter_external);
    assert!(!opts.group_by_module);
    assert!(!opts.show_complexity);
}

#[test]
fn test_mermaid_generator_default() {
    let gen = MermaidGenerator::default();
    // Should produce valid output for empty graph
    let graph = DependencyGraph::new();
    let output = gen.generate(&graph);
    assert!(output.contains("graph"));
}

// --- sanitize_id tests ---

#[test]
fn test_sanitize_id_simple() {
    let gen = make_generator();
    assert_eq!(gen.sanitize_id("hello"), "hello");
    assert_eq!(gen.sanitize_id("abc123"), "abc123");
}

#[test]
fn test_sanitize_id_with_colons() {
    let gen = make_generator();
    assert_eq!(gen.sanitize_id("std::io::Read"), "std_io_Read");
}

#[test]
fn test_sanitize_id_with_slashes() {
    let gen = make_generator();
    assert_eq!(gen.sanitize_id("src/lib.rs"), "src_lib_rs");
}

#[test]
fn test_sanitize_id_with_dashes() {
    let gen = make_generator();
    assert_eq!(gen.sanitize_id("my-module"), "my_module");
}

#[test]
fn test_sanitize_id_with_spaces() {
    let gen = make_generator();
    assert_eq!(gen.sanitize_id("hello world"), "hello_world");
}

#[test]
fn test_sanitize_id_numeric_prefix() {
    let gen = make_generator();
    let result = gen.sanitize_id("123abc");
    assert!(result.starts_with('_'));
}

#[test]
fn test_sanitize_id_empty() {
    let gen = make_generator();
    assert_eq!(gen.sanitize_id(""), "_empty");
}

#[test]
fn test_sanitize_id_special_chars() {
    let gen = make_generator();
    let result = gen.sanitize_id("fn@#$%");
    assert!(result.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
}

#[test]
fn test_sanitize_id_dots() {
    let gen = make_generator();
    assert_eq!(gen.sanitize_id("a.b.c"), "a_b_c");
}

// --- escape_mermaid_label tests ---

#[test]
fn test_escape_mermaid_label_plain() {
    let gen = make_generator();
    assert_eq!(gen.escape_mermaid_label("hello"), "hello");
}

#[test]
fn test_escape_mermaid_label_ampersand() {
    let gen = make_generator();
    assert_eq!(gen.escape_mermaid_label("a & b"), "a  and  b");
}

#[test]
fn test_escape_mermaid_label_angle_brackets() {
    let gen = make_generator();
    let result = gen.escape_mermaid_label("Vec<String>");
    assert!(!result.contains('<'));
    assert!(!result.contains('>'));
    assert!(result.contains("(String)"));
}

#[test]
fn test_escape_mermaid_label_pipe() {
    let gen = make_generator();
    let result = gen.escape_mermaid_label("a|b");
    assert!(!result.contains('|'));
}

#[test]
fn test_escape_mermaid_label_brackets() {
    let gen = make_generator();
    let result = gen.escape_mermaid_label("[array]");
    assert!(!result.contains('['));
    assert!(!result.contains(']'));
}

#[test]
fn test_escape_mermaid_label_braces() {
    let gen = make_generator();
    let result = gen.escape_mermaid_label("{map}");
    assert!(!result.contains('{'));
    assert!(!result.contains('}'));
}

#[test]
fn test_escape_mermaid_label_quotes() {
    let gen = make_generator();
    let result = gen.escape_mermaid_label("say \"hello\"");
    assert!(!result.contains('"'));
}

#[test]
fn test_escape_mermaid_label_newlines() {
    let gen = make_generator();
    let result = gen.escape_mermaid_label("line1\nline2");
    assert!(!result.contains('\n'));
}

// --- get_edge_arrow tests ---

#[test]
fn test_get_edge_arrow_calls() {
    let gen = make_generator();
    assert_eq!(gen.get_edge_arrow(&EdgeType::Calls), "-->");
}

#[test]
fn test_get_edge_arrow_imports() {
    let gen = make_generator();
    assert_eq!(gen.get_edge_arrow(&EdgeType::Imports), "-.->");
}

#[test]
fn test_get_edge_arrow_inherits() {
    let gen = make_generator();
    let result = gen.get_edge_arrow(&EdgeType::Inherits);
    assert!(result.contains("inherits"));
}

#[test]
fn test_get_edge_arrow_implements() {
    let gen = make_generator();
    let result = gen.get_edge_arrow(&EdgeType::Implements);
    assert!(result.contains("implements"));
}

#[test]
fn test_get_edge_arrow_uses() {
    let gen = make_generator();
    assert_eq!(gen.get_edge_arrow(&EdgeType::Uses), "---");
}

// --- get_complexity_color tests ---

#[test]
fn test_get_complexity_color_low() {
    let gen = make_generator();
    assert_eq!(gen.get_complexity_color(1), "#90EE90");
    assert_eq!(gen.get_complexity_color(3), "#90EE90");
}

#[test]
fn test_get_complexity_color_medium() {
    let gen = make_generator();
    assert_eq!(gen.get_complexity_color(4), "#FFD700");
    assert_eq!(gen.get_complexity_color(7), "#FFD700");
}

#[test]
fn test_get_complexity_color_high() {
    let gen = make_generator();
    assert_eq!(gen.get_complexity_color(8), "#FFA500");
    assert_eq!(gen.get_complexity_color(12), "#FFA500");
}

#[test]
fn test_get_complexity_color_very_high() {
    let gen = make_generator();
    assert_eq!(gen.get_complexity_color(13), "#FF6347");
    assert_eq!(gen.get_complexity_color(50), "#FF6347");
}

#[test]
fn test_get_complexity_color_zero() {
    let gen = make_generator();
    // 0 falls through to the catch-all
    assert_eq!(gen.get_complexity_color(0), "#FF6347");
}

// --- generate tests ---

#[test]
fn test_generate_empty_graph() {
    let gen = make_generator();
    let graph = DependencyGraph::new();
    let output = gen.generate(&graph);
    assert!(output.contains("graph"));
}

#[test]
fn test_generate_simple_graph() {
    let gen = make_generator();
    let mut graph = DependencyGraph::new();
    graph.nodes.insert(
        "main".to_string(),
        NodeInfo {
            id: "main".to_string(),
            label: "main".to_string(),
            node_type: NodeType::Function,
            complexity: 5,
            file_path: "src/main.rs".to_string(),
            line_number: 1,
            metadata: FxHashMap::default(),
        },
    );
    graph.nodes.insert(
        "helper".to_string(),
        NodeInfo {
            id: "helper".to_string(),
            label: "helper".to_string(),
            node_type: NodeType::Function,
            complexity: 2,
            file_path: "src/lib.rs".to_string(),
            line_number: 1,
            metadata: FxHashMap::default(),
        },
    );
    graph.edges.push(Edge {
        from: "main".to_string(),
        to: "helper".to_string(),
        edge_type: EdgeType::Calls,
        weight: 1,
    });

    let output = gen.generate(&graph);
    assert!(output.contains("graph"));
}

#[test]
fn test_generate_with_complexity() {
    let gen = make_generator_with_options(true, false);
    let mut graph = DependencyGraph::new();
    graph.nodes.insert(
        "complex_fn".to_string(),
        NodeInfo {
            id: "complex_fn".to_string(),
            label: "complex_fn".to_string(),
            node_type: NodeType::Function,
            complexity: 25,
            file_path: "src/complex.rs".to_string(),
            line_number: 1,
            metadata: FxHashMap::default(),
        },
    );

    let output = gen.generate(&graph);
    assert!(!output.is_empty());
}

#[test]
fn test_generate_with_module_grouping() {
    let gen = make_generator_with_options(false, true);
    let mut graph = DependencyGraph::new();
    graph.nodes.insert(
        "mod_a::fn1".to_string(),
        NodeInfo {
            id: "mod_a::fn1".to_string(),
            label: "fn1".to_string(),
            node_type: NodeType::Function,
            complexity: 3,
            file_path: "src/mod_a.rs".to_string(),
            line_number: 1,
            metadata: FxHashMap::default(),
        },
    );
    graph.nodes.insert(
        "mod_b::fn2".to_string(),
        NodeInfo {
            id: "mod_b::fn2".to_string(),
            label: "fn2".to_string(),
            node_type: NodeType::Module,
            complexity: 1,
            file_path: "src/mod_b.rs".to_string(),
            line_number: 1,
            metadata: FxHashMap::default(),
        },
    );

    let output = gen.generate(&graph);
    assert!(!output.is_empty());
}

#[test]
fn test_generate_all_node_types() {
    let gen = make_generator();
    let mut graph = DependencyGraph::new();

    let node_types = vec![
        ("func", NodeType::Function),
        ("module", NodeType::Module),
        ("class", NodeType::Class),
        ("trait_node", NodeType::Trait),
        ("iface", NodeType::Interface),
    ];

    for (name, node_type) in node_types {
        graph.nodes.insert(
            name.to_string(),
            NodeInfo {
                id: name.to_string(),
                label: name.to_string(),
                node_type,
                complexity: 5,
                file_path: String::new(),
                line_number: 0,
                metadata: FxHashMap::default(),
            },
        );
    }

    let output = gen.generate(&graph);
    assert!(!output.is_empty());
}

#[test]
fn test_generate_all_edge_types() {
    let gen = make_generator();
    let mut graph = DependencyGraph::new();

    for i in 0..6 {
        graph.nodes.insert(
            format!("n{i}"),
            NodeInfo {
                id: format!("n{i}"),
                label: format!("node{i}"),
                node_type: NodeType::Function,
                complexity: 1,
                file_path: String::new(),
                line_number: 0,
                metadata: FxHashMap::default(),
            },
        );
    }

    let edge_types = vec![
        EdgeType::Calls,
        EdgeType::Imports,
        EdgeType::Inherits,
        EdgeType::Implements,
        EdgeType::Uses,
    ];

    for (i, edge_type) in edge_types.into_iter().enumerate() {
        graph.edges.push(Edge {
            from: format!("n{i}"),
            to: format!("n{}", i + 1),
            edge_type,
            weight: 1,
        });
    }

    let output = gen.generate(&graph);
    assert!(!output.is_empty());
}
