// TICKET-001: Test-driven development for graph types
// Writing tests FIRST before implementation

use super::super::*;
use std::path::PathBuf;
use serde_json;

#[test]
fn test_node_data_creation() {
    let node = NodeData {
        path: PathBuf::from("src/main.rs"),
        module: "main".to_string(),
        symbols: vec![
            Symbol {
                name: "main".to_string(),
                kind: SymbolKind::Function,
                visibility: Visibility::Public,
                line: 10,
            }
        ],
        loc: 100,
        complexity: 5.0,
        ast_hash: 0x1234567890ABCDEF,
    };

    assert_eq!(node.path, PathBuf::from("src/main.rs"));
    assert_eq!(node.module, "main");
    assert_eq!(node.symbols.len(), 1);
    assert_eq!(node.loc, 100);
    assert_eq!(node.complexity, 5.0);
    assert_eq!(node.ast_hash, 0x1234567890ABCDEF);
}

#[test]
fn test_node_data_serialization() {
    let node = NodeData {
        path: PathBuf::from("test.rs"),
        module: "test".to_string(),
        symbols: vec![],
        loc: 50,
        complexity: 2.5,
        ast_hash: 0xDEADBEEF,
    };

    let json = serde_json::to_string(&node).unwrap();
    let deserialized: NodeData = serde_json::from_str(&json).unwrap();

    assert_eq!(node.path, deserialized.path);
    assert_eq!(node.module, deserialized.module);
    assert_eq!(node.ast_hash, deserialized.ast_hash);
}

#[test]
fn test_edge_data_import_variant() {
    let edge = EdgeData::Import {
        weight: 2.0,
        visibility: Visibility::Public,
    };

    match edge {
        EdgeData::Import { weight, visibility } => {
            assert_eq!(weight, 2.0);
            assert_eq!(visibility, Visibility::Public);
        }
        _ => panic!("Wrong edge variant"),
    }
}

#[test]
fn test_edge_data_function_call_variant() {
    let edge = EdgeData::FunctionCall {
        count: 5,
        async_call: true,
    };

    match edge {
        EdgeData::FunctionCall { count, async_call } => {
            assert_eq!(count, 5);
            assert!(async_call);
        }
        _ => panic!("Wrong edge variant"),
    }
}

#[test]
fn test_edge_data_type_dependency_variant() {
    let edge = EdgeData::TypeDependency {
        strength: 0.8,
        kind: TypeKind::Trait,
    };

    match edge {
        EdgeData::TypeDependency { strength, kind } => {
            assert_eq!(strength, 0.8);
            assert!(matches!(kind, TypeKind::Trait));
        }
        _ => panic!("Wrong edge variant"),
    }
}

#[test]
fn test_edge_data_weight_conversion() {
    // Import edges weighted highest (x2)
    let import = EdgeData::Import {
        weight: 1.0,
        visibility: Visibility::Private,
    };
    assert_eq!(import.to_numeric_weight(), 2.0);

    // Function calls use count directly
    let func_call = EdgeData::FunctionCall {
        count: 3,
        async_call: false,
    };
    assert_eq!(func_call.to_numeric_weight(), 3.0);

    // Type dependencies weighted x1.5
    let type_dep = EdgeData::TypeDependency {
        strength: 0.5,
        kind: TypeKind::Struct,
    };
    assert_eq!(type_dep.to_numeric_weight(), 0.75);

    // Data flow uses confidence directly
    let data_flow = EdgeData::DataFlow {
        confidence: 0.9,
        direction: FlowDirection::Forward,
    };
    assert_eq!(data_flow.to_numeric_weight(), 0.9);

    // Inheritance inversely weighted by depth
    let inheritance = EdgeData::Inheritance { depth: 2 };
    assert_eq!(inheritance.to_numeric_weight(), 1.0); // 3.0 / (2 + 1)
}

#[test]
fn test_symbol_creation() {
    let symbol = Symbol {
        name: "calculate".to_string(),
        kind: SymbolKind::Function,
        visibility: Visibility::Private,
        line: 42,
    };

    assert_eq!(symbol.name, "calculate");
    assert!(matches!(symbol.kind, SymbolKind::Function));
    assert!(matches!(symbol.visibility, Visibility::Private));
    assert_eq!(symbol.line, 42);
}

#[test]
fn test_visibility_ordering() {
    // Public should be "more visible" than Private
    assert!(Visibility::Public > Visibility::Private);
    assert!(Visibility::Public > Visibility::Protected);
    assert!(Visibility::Protected > Visibility::Private);
}

#[test]
fn test_graph_matrices_from_graph() {
    use petgraph::graph::DiGraph;

    // Create a simple 3-node graph
    let mut graph = DiGraph::<NodeData, EdgeData>::new();

    let n0 = graph.add_node(NodeData::test_node(0));
    let n1 = graph.add_node(NodeData::test_node(1));
    let n2 = graph.add_node(NodeData::test_node(2));

    // Add edges: 0->1, 1->2, 2->0 (cycle)
    graph.add_edge(n0, n1, EdgeData::test_edge(1.0));
    graph.add_edge(n1, n2, EdgeData::test_edge(2.0));
    graph.add_edge(n2, n0, EdgeData::test_edge(3.0));

    let matrices = GraphMatrices::from(&graph);

    // Check dimensions
    assert_eq!(matrices.adjacency.nrows(), 3);
    assert_eq!(matrices.adjacency.ncols(), 3);

    // Check out-degrees
    assert_eq!(matrices.out_degrees.len(), 3);
    assert_eq!(matrices.out_degrees[0], 1.0);
    assert_eq!(matrices.out_degrees[1], 2.0);
    assert_eq!(matrices.out_degrees[2], 3.0);

    // Transition matrix should be column-stochastic
    // (columns sum to 1 for PageRank)
    assert!(matrices.transition.nrows() == 3);
}

#[test]
fn test_type_kind_variants() {
    let generic = TypeKind::Generic;
    let _trait_kind = TypeKind::Trait;
    let _struct_kind = TypeKind::Struct;
    let _enum_kind = TypeKind::Enum;

    // Just ensure all variants exist and can be matched
    match generic {
        TypeKind::Generic => {}
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_flow_direction() {
    let forward = FlowDirection::Forward;
    let backward = FlowDirection::Backward;
    let bidirectional = FlowDirection::Bidirectional;

    assert!(matches!(forward, FlowDirection::Forward));
    assert!(matches!(backward, FlowDirection::Backward));
    assert!(matches!(bidirectional, FlowDirection::Bidirectional));
}

// Test helper implementations
impl NodeData {
    pub fn test_node(id: usize) -> Self {
        NodeData {
            path: PathBuf::from(format!("file_{}.rs", id)),
            module: format!("mod_{}", id),
            symbols: vec![],
            loc: 100,
            complexity: 1.0,
            ast_hash: id as u64,
        }
    }
}

impl EdgeData {
    pub fn test_edge(weight: f64) -> Self {
        EdgeData::Import {
            weight: weight / 2.0, // Divided by 2 since Import multiplies by 2
            visibility: Visibility::Public,
        }
    }
}