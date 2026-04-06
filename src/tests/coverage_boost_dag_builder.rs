#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for dag_builder module
//! Target: Test pure functions for graph filtering and manipulation

use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
use crate::services::dag_builder::{
    add_pagerank_scores, filter_call_edges, filter_import_edges, filter_inheritance_edges,
    prune_graph_pagerank, DagBuilder,
};

// ============================================================================
// Helper functions for creating test data
// ============================================================================

fn create_test_node(id: &str, node_type: NodeType) -> NodeInfo {
    debug_assert!(!id.is_empty(), "id must not be empty");
    NodeInfo {
        id: id.to_string(),
        label: id.to_string(),
        node_type,
        file_path: format!("{}.rs", id),
        line_number: 1,
        complexity: 5,
        metadata: Default::default(),
    }
}

fn create_test_edge(from: &str, to: &str, edge_type: EdgeType) -> Edge {
    debug_assert!(!from.is_empty(), "from must not be empty");
    debug_assert!(!to.is_empty(), "to must not be empty");
    Edge {
        from: from.to_string(),
        to: to.to_string(),
        edge_type,
        weight: 1,
    }
}

fn create_basic_graph() -> DependencyGraph {
    let mut graph = DependencyGraph::new();

    graph.nodes.insert(
        "main".to_string(),
        create_test_node("main", NodeType::Function),
    );
    graph.nodes.insert(
        "helper".to_string(),
        create_test_node("helper", NodeType::Function),
    );
    graph.nodes.insert(
        "utils".to_string(),
        create_test_node("utils", NodeType::Module),
    );
    graph.nodes.insert(
        "MyClass".to_string(),
        create_test_node("MyClass", NodeType::Class),
    );
    graph.nodes.insert(
        "MyTrait".to_string(),
        create_test_node("MyTrait", NodeType::Trait),
    );

    graph
        .edges
        .push(create_test_edge("main", "helper", EdgeType::Calls));
    graph
        .edges
        .push(create_test_edge("main", "utils", EdgeType::Imports));
    graph
        .edges
        .push(create_test_edge("MyClass", "MyTrait", EdgeType::Implements));

    graph
}

// ============================================================================
// DagBuilder::new tests
// ============================================================================

#[test]
fn test_dag_builder_new_creates_empty_graph() {
    let _builder = DagBuilder::new();
    // Build from empty context
    let graph = DagBuilder::build_from_project(&crate::services::context::ProjectContext {
        project_type: "rust".to_string(),
        files: vec![],
        summary: crate::services::context::ProjectSummary {
            total_files: 0,
            total_functions: 0,
            total_structs: 0,
            total_enums: 0,
            total_traits: 0,
            total_impls: 0,
            dependencies: vec![],
        },
        graph: None,
    });
    assert!(graph.nodes.is_empty());
    assert!(graph.edges.is_empty());
}

#[test]
fn test_dag_builder_default_is_new() {
    let _default_builder = DagBuilder::default();
    let _new_builder = DagBuilder::new();

    // Both should create empty graphs
    let graph1 = DagBuilder::build_from_project(&crate::services::context::ProjectContext {
        project_type: "rust".to_string(),
        files: vec![],
        summary: crate::services::context::ProjectSummary {
            total_files: 0,
            total_functions: 0,
            total_structs: 0,
            total_enums: 0,
            total_traits: 0,
            total_impls: 0,
            dependencies: vec![],
        },
        graph: None,
    });
    assert!(graph1.nodes.is_empty());
}

// ============================================================================
// filter_call_edges tests
// ============================================================================

#[test]
fn test_filter_call_edges_empty_graph() {
    let graph = DependencyGraph::new();
    let filtered = filter_call_edges(graph);
    assert!(filtered.edges.is_empty());
}

#[test]
fn test_filter_call_edges_keeps_only_calls() {
    let graph = create_basic_graph();
    let filtered = filter_call_edges(graph);

    // Should only have call edges
    assert_eq!(filtered.edges.len(), 1);
    assert!(filtered
        .edges
        .iter()
        .all(|e| e.edge_type == EdgeType::Calls));
    assert!(filtered
        .edges
        .iter()
        .any(|e| e.from == "main" && e.to == "helper"));
}

#[test]
fn test_filter_call_edges_keeps_connected_nodes() {
    let graph = create_basic_graph();
    let filtered = filter_call_edges(graph);

    // Only nodes connected by call edges should be preserved
    // main -> helper is the only call edge, so only main and helper should remain
    assert_eq!(filtered.nodes.len(), 2);
    assert!(filtered.nodes.contains_key("main"));
    assert!(filtered.nodes.contains_key("helper"));
}

#[test]
fn test_filter_call_edges_no_call_edges() {
    let mut graph = DependencyGraph::new();
    graph
        .nodes
        .insert("a".to_string(), create_test_node("a", NodeType::Module));
    graph
        .nodes
        .insert("b".to_string(), create_test_node("b", NodeType::Module));
    graph
        .edges
        .push(create_test_edge("a", "b", EdgeType::Imports));

    let filtered = filter_call_edges(graph);
    assert!(filtered.edges.is_empty());
}

// ============================================================================
// filter_import_edges tests
// ============================================================================

#[test]
fn test_filter_import_edges_empty_graph() {
    let graph = DependencyGraph::new();
    let filtered = filter_import_edges(graph);
    assert!(filtered.edges.is_empty());
}

#[test]
fn test_filter_import_edges_keeps_only_imports() {
    let graph = create_basic_graph();
    let filtered = filter_import_edges(graph);

    // Should only have import edges
    assert_eq!(filtered.edges.len(), 1);
    assert!(filtered
        .edges
        .iter()
        .all(|e| e.edge_type == EdgeType::Imports));
    assert!(filtered
        .edges
        .iter()
        .any(|e| e.from == "main" && e.to == "utils"));
}

#[test]
fn test_filter_import_edges_keeps_connected_nodes() {
    let graph = create_basic_graph();
    let filtered = filter_import_edges(graph);

    // Only nodes connected by import edges should be preserved
    // main -> utils is the only import edge
    assert_eq!(filtered.nodes.len(), 2);
    assert!(filtered.nodes.contains_key("main"));
    assert!(filtered.nodes.contains_key("utils"));
}

#[test]
fn test_filter_import_edges_no_import_edges() {
    let mut graph = DependencyGraph::new();
    graph
        .nodes
        .insert("a".to_string(), create_test_node("a", NodeType::Function));
    graph
        .nodes
        .insert("b".to_string(), create_test_node("b", NodeType::Function));
    graph
        .edges
        .push(create_test_edge("a", "b", EdgeType::Calls));

    let filtered = filter_import_edges(graph);
    assert!(filtered.edges.is_empty());
}

// ============================================================================
// filter_inheritance_edges tests
// ============================================================================

#[test]
fn test_filter_inheritance_edges_empty_graph() {
    let graph = DependencyGraph::new();
    let filtered = filter_inheritance_edges(graph);
    assert!(filtered.edges.is_empty());
}

#[test]
fn test_filter_inheritance_edges_keeps_only_inherits() {
    let mut graph = create_basic_graph();
    // Add an Inherits edge
    graph.nodes.insert(
        "ChildClass".to_string(),
        create_test_node("ChildClass", NodeType::Class),
    );
    graph.edges.push(create_test_edge(
        "ChildClass",
        "MyClass",
        EdgeType::Inherits,
    ));

    let filtered = filter_inheritance_edges(graph);

    // filter_inheritance_edges only keeps Inherits edges, not Implements
    assert_eq!(filtered.edges.len(), 1);
    assert!(filtered
        .edges
        .iter()
        .all(|e| e.edge_type == EdgeType::Inherits));
}

#[test]
fn test_filter_inheritance_edges_returns_empty_when_no_inherits() {
    let graph = create_basic_graph();
    // basic_graph has Implements but no Inherits
    let filtered = filter_inheritance_edges(graph);

    // No Inherits edges means empty nodes
    assert!(filtered.nodes.is_empty());
    assert!(filtered.edges.is_empty());
}

// ============================================================================
// add_pagerank_scores tests
// ============================================================================

#[test]
fn test_add_pagerank_scores_empty_graph() {
    let graph = DependencyGraph::new();
    let scored = add_pagerank_scores(graph);
    assert!(scored.nodes.is_empty());
}

#[test]
fn test_add_pagerank_scores_single_node() {
    let mut graph = DependencyGraph::new();
    graph
        .nodes
        .insert("a".to_string(), create_test_node("a", NodeType::Function));

    let scored = add_pagerank_scores(graph);
    assert_eq!(scored.nodes.len(), 1);
}

#[test]
fn test_add_pagerank_scores_preserves_structure() {
    let graph = create_basic_graph();
    let node_count = graph.nodes.len();
    let edge_count = graph.edges.len();

    let scored = add_pagerank_scores(graph);

    assert_eq!(scored.nodes.len(), node_count);
    assert_eq!(scored.edges.len(), edge_count);
}

#[test]
fn test_add_pagerank_scores_with_chain() {
    let mut graph = DependencyGraph::new();
    graph
        .nodes
        .insert("a".to_string(), create_test_node("a", NodeType::Function));
    graph
        .nodes
        .insert("b".to_string(), create_test_node("b", NodeType::Function));
    graph
        .nodes
        .insert("c".to_string(), create_test_node("c", NodeType::Function));

    // Chain: a -> b -> c
    graph
        .edges
        .push(create_test_edge("a", "b", EdgeType::Calls));
    graph
        .edges
        .push(create_test_edge("b", "c", EdgeType::Calls));

    let scored = add_pagerank_scores(graph);
    assert_eq!(scored.nodes.len(), 3);
}

// ============================================================================
// prune_graph_pagerank tests
// ============================================================================

#[test]
fn test_prune_graph_pagerank_empty_graph() {
    let graph = DependencyGraph::new();
    let pruned = prune_graph_pagerank(&graph, 10);
    assert!(pruned.nodes.is_empty());
}

#[test]
fn test_prune_graph_pagerank_no_pruning_needed() {
    let graph = create_basic_graph();
    let pruned = prune_graph_pagerank(&graph, 100); // Much larger than our graph

    // Should keep all nodes
    assert_eq!(pruned.nodes.len(), graph.nodes.len());
}

#[test]
fn test_prune_graph_pagerank_max_zero() {
    let graph = create_basic_graph();
    let pruned = prune_graph_pagerank(&graph, 0);

    // Should return empty graph
    assert!(pruned.nodes.is_empty());
}

#[test]
fn test_prune_graph_pagerank_keeps_max_nodes() {
    let graph = create_basic_graph();
    let pruned = prune_graph_pagerank(&graph, 2);

    // Should keep at most 2 nodes
    assert!(pruned.nodes.len() <= 2);
}

#[test]
fn test_prune_graph_pagerank_removes_orphan_edges() {
    let mut graph = DependencyGraph::new();
    graph
        .nodes
        .insert("a".to_string(), create_test_node("a", NodeType::Function));
    graph
        .nodes
        .insert("b".to_string(), create_test_node("b", NodeType::Function));
    graph
        .nodes
        .insert("c".to_string(), create_test_node("c", NodeType::Function));

    graph
        .edges
        .push(create_test_edge("a", "b", EdgeType::Calls));
    graph
        .edges
        .push(create_test_edge("b", "c", EdgeType::Calls));

    // Prune to 2 nodes - should also remove edges that reference missing nodes
    let pruned = prune_graph_pagerank(&graph, 2);

    // All remaining edges should reference existing nodes
    for edge in &pruned.edges {
        assert!(pruned.nodes.contains_key(&edge.from));
        assert!(pruned.nodes.contains_key(&edge.to));
    }
}

// ============================================================================
// EdgeType tests
// ============================================================================

#[test]
fn test_edge_type_equality() {
    assert_eq!(EdgeType::Calls, EdgeType::Calls);
    assert_eq!(EdgeType::Imports, EdgeType::Imports);
    assert_eq!(EdgeType::Inherits, EdgeType::Inherits);
    assert_eq!(EdgeType::Implements, EdgeType::Implements);
    assert_ne!(EdgeType::Calls, EdgeType::Imports);
}

#[test]
fn test_edge_type_clone() {
    let edge_type = EdgeType::Calls;
    let cloned = edge_type.clone();
    assert_eq!(edge_type, cloned);
}

// ============================================================================
// NodeType tests
// ============================================================================

#[test]
fn test_node_type_equality() {
    assert_eq!(NodeType::Function, NodeType::Function);
    assert_eq!(NodeType::Class, NodeType::Class);
    assert_eq!(NodeType::Module, NodeType::Module);
    assert_eq!(NodeType::Trait, NodeType::Trait);
    assert_ne!(NodeType::Function, NodeType::Class);
}

#[test]
fn test_node_type_clone() {
    let node_type = NodeType::Function;
    let cloned = node_type.clone();
    assert_eq!(node_type, cloned);
}

// ============================================================================
// DependencyGraph tests
// ============================================================================

#[test]
fn test_dependency_graph_new() {
    let graph = DependencyGraph::new();
    assert!(graph.nodes.is_empty());
    assert!(graph.edges.is_empty());
}

#[test]
fn test_dependency_graph_node_insertion() {
    let mut graph = DependencyGraph::new();
    graph.nodes.insert(
        "test".to_string(),
        create_test_node("test", NodeType::Function),
    );

    assert_eq!(graph.nodes.len(), 1);
    assert!(graph.nodes.contains_key("test"));
}

#[test]
fn test_dependency_graph_edge_insertion() {
    let mut graph = DependencyGraph::new();
    graph
        .edges
        .push(create_test_edge("a", "b", EdgeType::Calls));

    assert_eq!(graph.edges.len(), 1);
    assert_eq!(graph.edges[0].from, "a");
    assert_eq!(graph.edges[0].to, "b");
}

// ============================================================================
// NodeInfo tests
// ============================================================================

#[test]
fn test_node_info_creation() {
    let node = create_test_node("test_fn", NodeType::Function);

    assert_eq!(node.id, "test_fn");
    assert_eq!(node.label, "test_fn");
    assert_eq!(node.node_type, NodeType::Function);
    assert_eq!(node.complexity, 5);
}

#[test]
fn test_node_info_clone() {
    let node = create_test_node("test_fn", NodeType::Function);
    let cloned = node.clone();

    assert_eq!(cloned.id, node.id);
    assert_eq!(cloned.label, node.label);
    assert_eq!(cloned.node_type, node.node_type);
}

// ============================================================================
// Edge tests
// ============================================================================

#[test]
fn test_edge_creation() {
    let edge = create_test_edge("from_node", "to_node", EdgeType::Calls);

    assert_eq!(edge.from, "from_node");
    assert_eq!(edge.to, "to_node");
    assert_eq!(edge.edge_type, EdgeType::Calls);
    assert_eq!(edge.weight, 1);
}

#[test]
fn test_edge_clone() {
    let edge = create_test_edge("a", "b", EdgeType::Imports);
    let cloned = edge.clone();

    assert_eq!(cloned.from, edge.from);
    assert_eq!(cloned.to, edge.to);
    assert_eq!(cloned.edge_type, edge.edge_type);
}

// ============================================================================
// Integration tests
// ============================================================================

#[test]
fn test_graph_manipulation_chain() {
    // Create -> Filter -> Prune
    let graph = create_basic_graph();

    let filtered = filter_call_edges(graph);
    assert_eq!(filtered.edges.len(), 1);

    let filtered_len = filtered.nodes.len();
    let scored = add_pagerank_scores(filtered);
    assert_eq!(scored.nodes.len(), filtered_len);

    let pruned = prune_graph_pagerank(&scored, 3);
    assert!(pruned.nodes.len() <= 3);
}

#[test]
fn test_multiple_filters_combined() {
    let graph = create_basic_graph();

    // Each filter produces independent results
    let calls = filter_call_edges(graph.clone());
    let imports = filter_import_edges(graph.clone());
    let inheritance = filter_inheritance_edges(graph);

    // calls: main -> helper (1 edge)
    // imports: main -> utils (1 edge)
    // inheritance: none (Implements is not Inherits)
    let total_filtered = calls.edges.len() + imports.edges.len() + inheritance.edges.len();
    assert_eq!(total_filtered, 2); // 1 call + 1 import + 0 inherits
}
