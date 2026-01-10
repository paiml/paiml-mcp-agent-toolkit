// Graph type system for PMAT
// Complexity: All functions ≤ 10
// SATD: Zero tolerance

use nalgebra_sparse::{CooMatrix, CsrMatrix};
use petgraph::graph::{DiGraph, UnGraph};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Core node data structure for dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub path: PathBuf,
    pub module: String,
    pub symbols: Vec<Symbol>,
    pub loc: usize,
    pub complexity: f64,
    pub ast_hash: u64, // For incremental updates
}

/// Symbol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub visibility: Visibility,
    pub line: usize,
}

/// Symbol types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Module,
    Variable,
    Constant,
}

/// Visibility levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum Visibility {
    Private,
    Protected,
    Public,
}

/// Edge types representing different dependency relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeData {
    Import {
        weight: f64,
        visibility: Visibility,
    },
    FunctionCall {
        count: usize,
        async_call: bool,
    },
    TypeDependency {
        strength: f64,
        kind: TypeKind,
    },
    DataFlow {
        confidence: f64,
        direction: FlowDirection,
    },
    Inheritance {
        depth: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TypeKind {
    Generic,
    Trait,
    Struct,
    Enum,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FlowDirection {
    Forward,
    Backward,
    Bidirectional,
}

/// Primary directed graph for dependency analysis
pub type DependencyGraph = DiGraph<NodeData, EdgeData>;

/// Undirected projection for community detection
pub type UndirectedGraph = UnGraph<NodeData, f64>;

/// Type alias for consistent node indexing
pub type NodeId = petgraph::graph::NodeIndex<u32>;

/// Unified matrix representations for different algorithms
pub struct GraphMatrices {
    /// Standard adjacency matrix
    pub adjacency: CsrMatrix<f64>,
    /// Column-stochastic for PageRank
    pub transition: CsrMatrix<f64>,
    /// For spectral clustering
    pub laplacian: CsrMatrix<f64>,
    /// Out-degree vector
    pub out_degrees: Vec<f64>,
    /// Number of nodes in the graph
    pub node_count: usize,
    /// Edge list for efficient iteration (from, to, weight)
    pub edges: Vec<(usize, usize, f64)>,
}

impl EdgeData {
    /// Convert heterogeneous edge types to numeric weights
    /// Complexity: 3 (simple match with arithmetic)
    pub fn to_numeric_weight(&self) -> f64 {
        match self {
            EdgeData::Import { weight, .. } => *weight * 2.0, // Imports weighted higher
            EdgeData::FunctionCall { count, .. } => *count as f64,
            EdgeData::TypeDependency { strength, .. } => *strength * 1.5,
            EdgeData::DataFlow { confidence, .. } => *confidence,
            EdgeData::Inheritance { depth } => 3.0 / (*depth as f64 + 1.0),
        }
    }
}

/// Conversion from petgraph to matrix representations
/// Complexity: 8 (loop with matrix operations)
impl From<&DependencyGraph> for GraphMatrices {
    fn from(graph: &DependencyGraph) -> Self {
        let n = graph.node_count();
        let mut coo = CooMatrix::new(n, n);
        let mut out_degrees = vec![0.0; n];
        let mut edges = Vec::new();

        // Build adjacency matrix with edge weights
        for edge in graph.edge_references() {
            let weight = edge.weight().to_numeric_weight();
            let source = edge.source().index();
            let target = edge.target().index();

            edges.push((source, target, weight));

            coo.push(source, target, weight);
            out_degrees[source] += weight;
        }

        let adjacency = CsrMatrix::from(&coo);

        // Create column-stochastic transition matrix
        let transition = Self::normalize_columns(&adjacency, &out_degrees);

        // Compute Laplacian L = D - A
        let laplacian = Self::compute_laplacian(&adjacency);

        GraphMatrices {
            adjacency,
            transition,
            laplacian,
            out_degrees,
            node_count: n,
            edges,
        }
    }
}

impl GraphMatrices {
    /// Normalize columns for stochastic matrix
    /// Complexity: 6 (nested loop with early exit)
    fn normalize_columns(adjacency: &CsrMatrix<f64>, out_degrees: &[f64]) -> CsrMatrix<f64> {
        let n = adjacency.nrows();
        let mut coo = CooMatrix::new(n, n);

        for i in 0..n {
            let row = adjacency.row(i);
            if out_degrees[i] > 0.0 {
                for (&value, &col) in row.values().iter().zip(row.col_indices()) {
                    coo.push(i, col, value / out_degrees[i]);
                }
            }
        }

        CsrMatrix::from(&coo)
    }

    /// Compute graph Laplacian
    /// Complexity: 6 (simplified matrix operations)
    fn compute_laplacian(adjacency: &CsrMatrix<f64>) -> CsrMatrix<f64> {
        let n = adjacency.nrows();
        let mut coo = CooMatrix::new(n, n);

        // Compute degree matrix D and build Laplacian L = D - A
        for i in 0..n {
            let row = adjacency.row(i);
            let degree: f64 = row.values().iter().sum();

            // Add diagonal degree
            coo.push(i, i, degree);

            // Subtract adjacency values
            for (&value, &col) in row.values().iter().zip(row.col_indices()) {
                coo.push(i, col, -value);
            }
        }

        CsrMatrix::from(&coo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::graph::DiGraph;

    // ============================================================
    // NodeData Tests
    // ============================================================

    #[test]
    fn test_node_data_default_construction() {
        let node = NodeData {
            path: PathBuf::from("test.rs"),
            module: String::new(),
            symbols: Vec::new(),
            loc: 0,
            complexity: 0.0,
            ast_hash: 0,
        };
        assert!(node.path.to_str().unwrap().contains("test.rs"));
        assert!(node.module.is_empty());
        assert!(node.symbols.is_empty());
        assert_eq!(node.loc, 0);
        assert_eq!(node.complexity, 0.0);
        assert_eq!(node.ast_hash, 0);
    }

    #[test]
    fn test_node_data_with_multiple_symbols() {
        let node = NodeData {
            path: PathBuf::from("lib.rs"),
            module: "mylib".to_string(),
            symbols: vec![
                Symbol {
                    name: "foo".to_string(),
                    kind: SymbolKind::Function,
                    visibility: Visibility::Public,
                    line: 1,
                },
                Symbol {
                    name: "Bar".to_string(),
                    kind: SymbolKind::Struct,
                    visibility: Visibility::Private,
                    line: 10,
                },
                Symbol {
                    name: "MyTrait".to_string(),
                    kind: SymbolKind::Trait,
                    visibility: Visibility::Public,
                    line: 50,
                },
            ],
            loc: 200,
            complexity: 15.5,
            ast_hash: 0xCAFEBABE,
        };

        assert_eq!(node.symbols.len(), 3);
        assert_eq!(node.symbols[0].name, "foo");
        assert_eq!(node.symbols[1].name, "Bar");
        assert_eq!(node.symbols[2].name, "MyTrait");
    }

    #[test]
    fn test_node_data_clone() {
        let original = NodeData {
            path: PathBuf::from("original.rs"),
            module: "orig".to_string(),
            symbols: vec![Symbol {
                name: "test".to_string(),
                kind: SymbolKind::Function,
                visibility: Visibility::Public,
                line: 5,
            }],
            loc: 50,
            complexity: 3.0,
            ast_hash: 123,
        };

        let cloned = original.clone();
        assert_eq!(original.path, cloned.path);
        assert_eq!(original.module, cloned.module);
        assert_eq!(original.symbols.len(), cloned.symbols.len());
        assert_eq!(original.loc, cloned.loc);
        assert_eq!(original.complexity, cloned.complexity);
        assert_eq!(original.ast_hash, cloned.ast_hash);
    }

    #[test]
    fn test_node_data_serialization_roundtrip() {
        let node = NodeData {
            path: PathBuf::from("src/lib.rs"),
            module: "lib".to_string(),
            symbols: vec![
                Symbol {
                    name: "init".to_string(),
                    kind: SymbolKind::Function,
                    visibility: Visibility::Public,
                    line: 1,
                },
                Symbol {
                    name: "Config".to_string(),
                    kind: SymbolKind::Struct,
                    visibility: Visibility::Public,
                    line: 20,
                },
            ],
            loc: 150,
            complexity: 8.5,
            ast_hash: 0xDEADC0DE,
        };

        let json = serde_json::to_string(&node).expect("serialization failed");
        let deserialized: NodeData =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(node.path, deserialized.path);
        assert_eq!(node.module, deserialized.module);
        assert_eq!(node.symbols.len(), deserialized.symbols.len());
        assert_eq!(node.loc, deserialized.loc);
        assert_eq!(node.complexity, deserialized.complexity);
        assert_eq!(node.ast_hash, deserialized.ast_hash);
    }

    // ============================================================
    // Symbol Tests
    // ============================================================

    #[test]
    fn test_symbol_all_kinds() {
        let kinds = [
            SymbolKind::Function,
            SymbolKind::Struct,
            SymbolKind::Enum,
            SymbolKind::Trait,
            SymbolKind::Module,
            SymbolKind::Variable,
            SymbolKind::Constant,
        ];

        for (i, kind) in kinds.iter().enumerate() {
            let symbol = Symbol {
                name: format!("symbol_{}", i),
                kind: kind.clone(),
                visibility: Visibility::Public,
                line: i + 1,
            };
            assert_eq!(symbol.line, i + 1);
        }
    }

    #[test]
    fn test_symbol_serialization_roundtrip() {
        let symbol = Symbol {
            name: "my_function".to_string(),
            kind: SymbolKind::Function,
            visibility: Visibility::Protected,
            line: 42,
        };

        let json = serde_json::to_string(&symbol).expect("serialization failed");
        let deserialized: Symbol =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(symbol.name, deserialized.name);
        assert_eq!(symbol.kind, deserialized.kind);
        assert_eq!(symbol.visibility, deserialized.visibility);
        assert_eq!(symbol.line, deserialized.line);
    }

    #[test]
    fn test_symbol_clone() {
        let original = Symbol {
            name: "original_sym".to_string(),
            kind: SymbolKind::Enum,
            visibility: Visibility::Private,
            line: 100,
        };

        let cloned = original.clone();
        assert_eq!(original.name, cloned.name);
        assert_eq!(original.kind, cloned.kind);
        assert_eq!(original.visibility, cloned.visibility);
        assert_eq!(original.line, cloned.line);
    }

    // ============================================================
    // SymbolKind Tests
    // ============================================================

    #[test]
    fn test_symbol_kind_equality() {
        assert_eq!(SymbolKind::Function, SymbolKind::Function);
        assert_eq!(SymbolKind::Struct, SymbolKind::Struct);
        assert_eq!(SymbolKind::Enum, SymbolKind::Enum);
        assert_eq!(SymbolKind::Trait, SymbolKind::Trait);
        assert_eq!(SymbolKind::Module, SymbolKind::Module);
        assert_eq!(SymbolKind::Variable, SymbolKind::Variable);
        assert_eq!(SymbolKind::Constant, SymbolKind::Constant);
    }

    #[test]
    fn test_symbol_kind_inequality() {
        assert_ne!(SymbolKind::Function, SymbolKind::Struct);
        assert_ne!(SymbolKind::Enum, SymbolKind::Trait);
        assert_ne!(SymbolKind::Module, SymbolKind::Variable);
        assert_ne!(SymbolKind::Constant, SymbolKind::Function);
    }

    #[test]
    fn test_symbol_kind_serialization() {
        let kinds = vec![
            SymbolKind::Function,
            SymbolKind::Struct,
            SymbolKind::Enum,
            SymbolKind::Trait,
            SymbolKind::Module,
            SymbolKind::Variable,
            SymbolKind::Constant,
        ];

        for kind in kinds {
            let json = serde_json::to_string(&kind).expect("serialization failed");
            let deserialized: SymbolKind =
                serde_json::from_str(&json).expect("deserialization failed");
            assert_eq!(kind, deserialized);
        }
    }

    // ============================================================
    // Visibility Tests
    // ============================================================

    #[test]
    fn test_visibility_ordering_comprehensive() {
        assert!(Visibility::Public > Visibility::Protected);
        assert!(Visibility::Public > Visibility::Private);
        assert!(Visibility::Protected > Visibility::Private);

        assert!(Visibility::Private < Visibility::Protected);
        assert!(Visibility::Private < Visibility::Public);
        assert!(Visibility::Protected < Visibility::Public);
    }

    #[test]
    fn test_visibility_equality() {
        assert_eq!(Visibility::Public, Visibility::Public);
        assert_eq!(Visibility::Protected, Visibility::Protected);
        assert_eq!(Visibility::Private, Visibility::Private);

        assert_ne!(Visibility::Public, Visibility::Private);
        assert_ne!(Visibility::Protected, Visibility::Private);
        assert_ne!(Visibility::Public, Visibility::Protected);
    }

    #[test]
    fn test_visibility_serialization() {
        let visibilities = vec![
            Visibility::Private,
            Visibility::Protected,
            Visibility::Public,
        ];

        for vis in visibilities {
            let json = serde_json::to_string(&vis).expect("serialization failed");
            let deserialized: Visibility =
                serde_json::from_str(&json).expect("deserialization failed");
            assert_eq!(vis, deserialized);
        }
    }

    // ============================================================
    // TypeKind Tests
    // ============================================================

    #[test]
    fn test_type_kind_equality() {
        assert_eq!(TypeKind::Generic, TypeKind::Generic);
        assert_eq!(TypeKind::Trait, TypeKind::Trait);
        assert_eq!(TypeKind::Struct, TypeKind::Struct);
        assert_eq!(TypeKind::Enum, TypeKind::Enum);
    }

    #[test]
    fn test_type_kind_inequality() {
        assert_ne!(TypeKind::Generic, TypeKind::Trait);
        assert_ne!(TypeKind::Struct, TypeKind::Enum);
        assert_ne!(TypeKind::Generic, TypeKind::Struct);
        assert_ne!(TypeKind::Trait, TypeKind::Enum);
    }

    #[test]
    fn test_type_kind_serialization() {
        let kinds = vec![
            TypeKind::Generic,
            TypeKind::Trait,
            TypeKind::Struct,
            TypeKind::Enum,
        ];

        for kind in kinds {
            let json = serde_json::to_string(&kind).expect("serialization failed");
            let deserialized: TypeKind =
                serde_json::from_str(&json).expect("deserialization failed");
            assert_eq!(kind, deserialized);
        }
    }

    // ============================================================
    // FlowDirection Tests
    // ============================================================

    #[test]
    fn test_flow_direction_equality() {
        assert_eq!(FlowDirection::Forward, FlowDirection::Forward);
        assert_eq!(FlowDirection::Backward, FlowDirection::Backward);
        assert_eq!(FlowDirection::Bidirectional, FlowDirection::Bidirectional);
    }

    #[test]
    fn test_flow_direction_inequality() {
        assert_ne!(FlowDirection::Forward, FlowDirection::Backward);
        assert_ne!(FlowDirection::Forward, FlowDirection::Bidirectional);
        assert_ne!(FlowDirection::Backward, FlowDirection::Bidirectional);
    }

    #[test]
    fn test_flow_direction_serialization() {
        let directions = vec![
            FlowDirection::Forward,
            FlowDirection::Backward,
            FlowDirection::Bidirectional,
        ];

        for dir in directions {
            let json = serde_json::to_string(&dir).expect("serialization failed");
            let deserialized: FlowDirection =
                serde_json::from_str(&json).expect("deserialization failed");
            assert_eq!(dir, deserialized);
        }
    }

    // ============================================================
    // EdgeData Tests
    // ============================================================

    #[test]
    fn test_edge_data_import() {
        let edge = EdgeData::Import {
            weight: 1.5,
            visibility: Visibility::Public,
        };

        if let EdgeData::Import { weight, visibility } = edge {
            assert_eq!(weight, 1.5);
            assert_eq!(visibility, Visibility::Public);
        } else {
            panic!("Expected Import variant");
        }
    }

    #[test]
    fn test_edge_data_function_call() {
        let edge = EdgeData::FunctionCall {
            count: 10,
            async_call: false,
        };

        if let EdgeData::FunctionCall { count, async_call } = edge {
            assert_eq!(count, 10);
            assert!(!async_call);
        } else {
            panic!("Expected FunctionCall variant");
        }
    }

    #[test]
    fn test_edge_data_type_dependency() {
        let edge = EdgeData::TypeDependency {
            strength: 0.95,
            kind: TypeKind::Generic,
        };

        if let EdgeData::TypeDependency { strength, kind } = edge {
            assert_eq!(strength, 0.95);
            assert_eq!(kind, TypeKind::Generic);
        } else {
            panic!("Expected TypeDependency variant");
        }
    }

    #[test]
    fn test_edge_data_data_flow() {
        let edge = EdgeData::DataFlow {
            confidence: 0.85,
            direction: FlowDirection::Bidirectional,
        };

        if let EdgeData::DataFlow {
            confidence,
            direction,
        } = edge
        {
            assert_eq!(confidence, 0.85);
            assert_eq!(direction, FlowDirection::Bidirectional);
        } else {
            panic!("Expected DataFlow variant");
        }
    }

    #[test]
    fn test_edge_data_inheritance() {
        let edge = EdgeData::Inheritance { depth: 3 };

        if let EdgeData::Inheritance { depth } = edge {
            assert_eq!(depth, 3);
        } else {
            panic!("Expected Inheritance variant");
        }
    }

    #[test]
    fn test_edge_data_clone() {
        let original = EdgeData::Import {
            weight: 2.5,
            visibility: Visibility::Protected,
        };

        let cloned = original.clone();
        if let (
            EdgeData::Import {
                weight: w1,
                visibility: v1,
            },
            EdgeData::Import {
                weight: w2,
                visibility: v2,
            },
        ) = (&original, &cloned)
        {
            assert_eq!(*w1, *w2);
            assert_eq!(*v1, *v2);
        } else {
            panic!("Clone failed");
        }
    }

    #[test]
    fn test_edge_data_serialization_all_variants() {
        let edges = vec![
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
            EdgeData::FunctionCall {
                count: 5,
                async_call: true,
            },
            EdgeData::TypeDependency {
                strength: 0.5,
                kind: TypeKind::Trait,
            },
            EdgeData::DataFlow {
                confidence: 0.9,
                direction: FlowDirection::Forward,
            },
            EdgeData::Inheritance { depth: 2 },
        ];

        for edge in edges {
            let json = serde_json::to_string(&edge).expect("serialization failed");
            let deserialized: EdgeData =
                serde_json::from_str(&json).expect("deserialization failed");

            // Verify serialization round-trip works
            let json2 = serde_json::to_string(&deserialized).expect("re-serialization failed");
            assert_eq!(json, json2);
        }
    }

    // ============================================================
    // EdgeData::to_numeric_weight Tests
    // ============================================================

    #[test]
    fn test_to_numeric_weight_import() {
        let edge = EdgeData::Import {
            weight: 1.0,
            visibility: Visibility::Private,
        };
        assert_eq!(edge.to_numeric_weight(), 2.0); // weight * 2.0
    }

    #[test]
    fn test_to_numeric_weight_import_zero() {
        let edge = EdgeData::Import {
            weight: 0.0,
            visibility: Visibility::Public,
        };
        assert_eq!(edge.to_numeric_weight(), 0.0);
    }

    #[test]
    fn test_to_numeric_weight_import_large() {
        let edge = EdgeData::Import {
            weight: 100.0,
            visibility: Visibility::Public,
        };
        assert_eq!(edge.to_numeric_weight(), 200.0);
    }

    #[test]
    fn test_to_numeric_weight_function_call() {
        let edge = EdgeData::FunctionCall {
            count: 7,
            async_call: false,
        };
        assert_eq!(edge.to_numeric_weight(), 7.0);
    }

    #[test]
    fn test_to_numeric_weight_function_call_zero() {
        let edge = EdgeData::FunctionCall {
            count: 0,
            async_call: true,
        };
        assert_eq!(edge.to_numeric_weight(), 0.0);
    }

    #[test]
    fn test_to_numeric_weight_function_call_large() {
        let edge = EdgeData::FunctionCall {
            count: 1000,
            async_call: false,
        };
        assert_eq!(edge.to_numeric_weight(), 1000.0);
    }

    #[test]
    fn test_to_numeric_weight_type_dependency() {
        let edge = EdgeData::TypeDependency {
            strength: 0.5,
            kind: TypeKind::Struct,
        };
        assert_eq!(edge.to_numeric_weight(), 0.75); // strength * 1.5
    }

    #[test]
    fn test_to_numeric_weight_type_dependency_zero() {
        let edge = EdgeData::TypeDependency {
            strength: 0.0,
            kind: TypeKind::Enum,
        };
        assert_eq!(edge.to_numeric_weight(), 0.0);
    }

    #[test]
    fn test_to_numeric_weight_type_dependency_one() {
        let edge = EdgeData::TypeDependency {
            strength: 1.0,
            kind: TypeKind::Trait,
        };
        assert_eq!(edge.to_numeric_weight(), 1.5);
    }

    #[test]
    fn test_to_numeric_weight_data_flow() {
        let edge = EdgeData::DataFlow {
            confidence: 0.8,
            direction: FlowDirection::Forward,
        };
        assert_eq!(edge.to_numeric_weight(), 0.8);
    }

    #[test]
    fn test_to_numeric_weight_data_flow_zero() {
        let edge = EdgeData::DataFlow {
            confidence: 0.0,
            direction: FlowDirection::Backward,
        };
        assert_eq!(edge.to_numeric_weight(), 0.0);
    }

    #[test]
    fn test_to_numeric_weight_data_flow_one() {
        let edge = EdgeData::DataFlow {
            confidence: 1.0,
            direction: FlowDirection::Bidirectional,
        };
        assert_eq!(edge.to_numeric_weight(), 1.0);
    }

    #[test]
    fn test_to_numeric_weight_inheritance_depth_zero() {
        let edge = EdgeData::Inheritance { depth: 0 };
        assert_eq!(edge.to_numeric_weight(), 3.0); // 3.0 / (0 + 1)
    }

    #[test]
    fn test_to_numeric_weight_inheritance_depth_one() {
        let edge = EdgeData::Inheritance { depth: 1 };
        assert_eq!(edge.to_numeric_weight(), 1.5); // 3.0 / (1 + 1)
    }

    #[test]
    fn test_to_numeric_weight_inheritance_depth_two() {
        let edge = EdgeData::Inheritance { depth: 2 };
        assert_eq!(edge.to_numeric_weight(), 1.0); // 3.0 / (2 + 1)
    }

    #[test]
    fn test_to_numeric_weight_inheritance_deep() {
        let edge = EdgeData::Inheritance { depth: 9 };
        assert_eq!(edge.to_numeric_weight(), 0.3); // 3.0 / (9 + 1)
    }

    // ============================================================
    // GraphMatrices Tests
    // ============================================================

    fn create_test_node(id: usize) -> NodeData {
        NodeData {
            path: PathBuf::from(format!("file_{}.rs", id)),
            module: format!("mod_{}", id),
            symbols: vec![],
            loc: 100,
            complexity: 1.0,
            ast_hash: id as u64,
        }
    }

    #[test]
    fn test_graph_matrices_empty_graph() {
        let graph: DependencyGraph = DiGraph::new();
        let matrices = GraphMatrices::from(&graph);

        assert_eq!(matrices.node_count, 0);
        assert_eq!(matrices.out_degrees.len(), 0);
        assert_eq!(matrices.edges.len(), 0);
        assert_eq!(matrices.adjacency.nrows(), 0);
        assert_eq!(matrices.adjacency.ncols(), 0);
    }

    #[test]
    fn test_graph_matrices_single_node_no_edges() {
        let mut graph: DependencyGraph = DiGraph::new();
        graph.add_node(create_test_node(0));

        let matrices = GraphMatrices::from(&graph);

        assert_eq!(matrices.node_count, 1);
        assert_eq!(matrices.out_degrees.len(), 1);
        assert_eq!(matrices.out_degrees[0], 0.0);
        assert_eq!(matrices.edges.len(), 0);
    }

    #[test]
    fn test_graph_matrices_two_nodes_one_edge() {
        let mut graph: DependencyGraph = DiGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));

        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );

        let matrices = GraphMatrices::from(&graph);

        assert_eq!(matrices.node_count, 2);
        assert_eq!(matrices.out_degrees.len(), 2);
        assert_eq!(matrices.out_degrees[0], 2.0); // Import weight * 2
        assert_eq!(matrices.out_degrees[1], 0.0);
        assert_eq!(matrices.edges.len(), 1);
        assert_eq!(matrices.edges[0], (0, 1, 2.0));
    }

    #[test]
    fn test_graph_matrices_cycle() {
        let mut graph: DependencyGraph = DiGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));
        let n2 = graph.add_node(create_test_node(2));

        // Create a cycle: 0 -> 1 -> 2 -> 0
        graph.add_edge(
            n0,
            n1,
            EdgeData::FunctionCall {
                count: 1,
                async_call: false,
            },
        );
        graph.add_edge(
            n1,
            n2,
            EdgeData::FunctionCall {
                count: 2,
                async_call: false,
            },
        );
        graph.add_edge(
            n2,
            n0,
            EdgeData::FunctionCall {
                count: 3,
                async_call: false,
            },
        );

        let matrices = GraphMatrices::from(&graph);

        assert_eq!(matrices.node_count, 3);
        assert_eq!(matrices.out_degrees[0], 1.0);
        assert_eq!(matrices.out_degrees[1], 2.0);
        assert_eq!(matrices.out_degrees[2], 3.0);
        assert_eq!(matrices.edges.len(), 3);
    }

    #[test]
    fn test_graph_matrices_multiple_edges_from_same_node() {
        let mut graph: DependencyGraph = DiGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));
        let n2 = graph.add_node(create_test_node(2));

        // Multiple edges from node 0
        graph.add_edge(
            n0,
            n1,
            EdgeData::FunctionCall {
                count: 5,
                async_call: false,
            },
        );
        graph.add_edge(
            n0,
            n2,
            EdgeData::FunctionCall {
                count: 3,
                async_call: true,
            },
        );

        let matrices = GraphMatrices::from(&graph);

        assert_eq!(matrices.out_degrees[0], 8.0); // 5 + 3
        assert_eq!(matrices.out_degrees[1], 0.0);
        assert_eq!(matrices.out_degrees[2], 0.0);
        assert_eq!(matrices.edges.len(), 2);
    }

    #[test]
    fn test_graph_matrices_different_edge_types() {
        let mut graph: DependencyGraph = DiGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));
        let n2 = graph.add_node(create_test_node(2));
        let n3 = graph.add_node(create_test_node(3));

        // Different edge types
        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );
        graph.add_edge(
            n1,
            n2,
            EdgeData::TypeDependency {
                strength: 1.0,
                kind: TypeKind::Struct,
            },
        );
        graph.add_edge(
            n2,
            n3,
            EdgeData::DataFlow {
                confidence: 1.0,
                direction: FlowDirection::Forward,
            },
        );

        let matrices = GraphMatrices::from(&graph);

        assert_eq!(matrices.out_degrees[0], 2.0); // Import: 1.0 * 2
        assert_eq!(matrices.out_degrees[1], 1.5); // TypeDep: 1.0 * 1.5
        assert_eq!(matrices.out_degrees[2], 1.0); // DataFlow: 1.0
        assert_eq!(matrices.out_degrees[3], 0.0); // No outgoing edges
    }

    #[test]
    fn test_graph_matrices_adjacency_dimensions() {
        let mut graph: DependencyGraph = DiGraph::new();
        for i in 0..5 {
            graph.add_node(create_test_node(i));
        }

        let matrices = GraphMatrices::from(&graph);

        assert_eq!(matrices.adjacency.nrows(), 5);
        assert_eq!(matrices.adjacency.ncols(), 5);
        assert_eq!(matrices.transition.nrows(), 5);
        assert_eq!(matrices.transition.ncols(), 5);
        assert_eq!(matrices.laplacian.nrows(), 5);
        assert_eq!(matrices.laplacian.ncols(), 5);
    }

    #[test]
    fn test_graph_matrices_transition_matrix_stochastic() {
        let mut graph: DependencyGraph = DiGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));
        let n2 = graph.add_node(create_test_node(2));

        // Node 0 has edges to both n1 and n2
        graph.add_edge(
            n0,
            n1,
            EdgeData::FunctionCall {
                count: 2,
                async_call: false,
            },
        );
        graph.add_edge(
            n0,
            n2,
            EdgeData::FunctionCall {
                count: 2,
                async_call: false,
            },
        );

        let matrices = GraphMatrices::from(&graph);

        // Row 0 should sum to 1.0 (stochastic)
        let row = matrices.transition.row(0);
        let row_sum: f64 = row.values().iter().sum();
        assert!((row_sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_graph_matrices_laplacian_diagonal() {
        let mut graph: DependencyGraph = DiGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));

        // Single edge with weight 2.0 (FunctionCall count=2)
        graph.add_edge(
            n0,
            n1,
            EdgeData::FunctionCall {
                count: 2,
                async_call: false,
            },
        );

        let matrices = GraphMatrices::from(&graph);

        // Laplacian row 0: diagonal = degree = 2.0, off-diagonal (0,1) = -2.0
        // Sum of row should be 0 for Laplacian
        let row = matrices.laplacian.row(0);
        let row_sum: f64 = row.values().iter().sum();
        assert!(
            row_sum.abs() < 1e-10,
            "Laplacian row sum should be 0, got {}",
            row_sum
        );
    }

    #[test]
    fn test_graph_matrices_edges_order() {
        let mut graph: DependencyGraph = DiGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));
        let n2 = graph.add_node(create_test_node(2));

        graph.add_edge(
            n0,
            n1,
            EdgeData::FunctionCall {
                count: 1,
                async_call: false,
            },
        );
        graph.add_edge(
            n1,
            n2,
            EdgeData::FunctionCall {
                count: 2,
                async_call: false,
            },
        );

        let matrices = GraphMatrices::from(&graph);

        assert_eq!(matrices.edges.len(), 2);

        // Check edges contain correct weights
        let has_edge_0_1 = matrices.edges.iter().any(|e| e.0 == 0 && e.1 == 1 && e.2 == 1.0);
        let has_edge_1_2 = matrices.edges.iter().any(|e| e.0 == 1 && e.1 == 2 && e.2 == 2.0);

        assert!(has_edge_0_1, "Should have edge 0->1 with weight 1.0");
        assert!(has_edge_1_2, "Should have edge 1->2 with weight 2.0");
    }

    #[test]
    fn test_graph_matrices_self_loop() {
        let mut graph: DependencyGraph = DiGraph::new();
        let n0 = graph.add_node(create_test_node(0));

        // Self-loop
        graph.add_edge(
            n0,
            n0,
            EdgeData::FunctionCall {
                count: 5,
                async_call: false,
            },
        );

        let matrices = GraphMatrices::from(&graph);

        assert_eq!(matrices.node_count, 1);
        assert_eq!(matrices.out_degrees[0], 5.0);
        assert_eq!(matrices.edges.len(), 1);
        assert_eq!(matrices.edges[0], (0, 0, 5.0));
    }

    #[test]
    fn test_graph_matrices_large_graph() {
        let mut graph: DependencyGraph = DiGraph::new();
        let nodes: Vec<_> = (0..100).map(|i| graph.add_node(create_test_node(i))).collect();

        // Create a chain
        for i in 0..99 {
            graph.add_edge(
                nodes[i],
                nodes[i + 1],
                EdgeData::FunctionCall {
                    count: 1,
                    async_call: false,
                },
            );
        }

        let matrices = GraphMatrices::from(&graph);

        assert_eq!(matrices.node_count, 100);
        assert_eq!(matrices.edges.len(), 99);

        // All nodes except the last should have out-degree 1.0
        for i in 0..99 {
            assert_eq!(matrices.out_degrees[i], 1.0);
        }
        assert_eq!(matrices.out_degrees[99], 0.0);
    }

    // ============================================================
    // Type Alias Tests (ensure they work correctly)
    // ============================================================

    #[test]
    fn test_dependency_graph_type_alias() {
        let mut graph: DependencyGraph = DiGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));

        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_undirected_graph_type_alias() {
        let mut graph: UndirectedGraph = UnGraph::new_undirected();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));

        graph.add_edge(n0, n1, 1.5);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_node_id_type_alias() {
        let mut graph: DependencyGraph = DiGraph::new();
        let id: NodeId = graph.add_node(create_test_node(0));

        assert_eq!(id.index(), 0);
    }
}
