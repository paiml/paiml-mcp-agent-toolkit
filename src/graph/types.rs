#![cfg_attr(coverage_nightly, coverage(off))]
// Graph type system for PMAT - using trueno-graph (replaces petgraph)
// Complexity: All functions <= 10
// SATD: Zero tolerance
// Sovereign AI Stack: GraphMatrices uses simple Vec-based sparse representation
// (nalgebra-sparse removed in favor of batuta stack principles)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use trueno_graph::{CsrGraph, NodeId as TruenoNodeId};

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
/// Type kind.
pub enum TypeKind {
    Generic,
    Trait,
    Struct,
    Enum,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Direction classification for flow.
pub enum FlowDirection {
    Forward,
    Backward,
    Bidirectional,
}

/// Type alias for node IDs (uses trueno-graph's NodeId)
pub type NodeId = TruenoNodeId;

/// Primary directed graph for dependency analysis
/// Wraps trueno-graph CsrGraph with separate storage for node/edge data
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Underlying trueno-graph for structure and algorithms
    graph: CsrGraph,
    /// Node data storage (NodeId -> NodeData)
    node_data: HashMap<NodeId, NodeData>,
    /// Edge data storage ((from, to) -> EdgeData)
    edge_data: HashMap<(NodeId, NodeId), EdgeData>,
    /// Next node ID to assign
    next_id: u32,
}

/// Edge reference for iteration
#[derive(Debug, Clone, Copy)]
pub struct EdgeRef<'a> {
    source: NodeId,
    target: NodeId,
    weight: &'a EdgeData,
}

/// Undirected graph for community detection
/// Uses same pattern as DependencyGraph but treats edges as bidirectional
#[derive(Debug, Clone)]
pub struct UndirectedGraph {
    graph: CsrGraph,
    node_data: HashMap<NodeId, NodeData>,
    edge_weights: HashMap<(NodeId, NodeId), f64>,
    next_id: u32,
}

#[derive(Debug, Clone, Copy)]
/// Undirected edge ref.
pub struct UndirectedEdgeRef<'a> {
    source: NodeId,
    target: NodeId,
    weight: f64,

    _phantom: std::marker::PhantomData<&'a ()>,
}

/// Simple sparse matrix using triplet (COO) format
/// Sovereign AI Stack: Replaces nalgebra-sparse with simple Vec-based representation
#[derive(Debug, Clone, Default)]
pub struct SimpleSparseMatrix {
    /// Number of rows
    pub nrows: usize,
    /// Number of columns
    pub ncols: usize,
    /// Triplets (row, col, value)
    pub triplets: Vec<(usize, usize, f64)>,
}

/// Unified matrix representations for different algorithms
/// Sovereign AI Stack: Uses simple Vec-based sparse matrices (no external deps)
#[derive(Debug, Clone)]
pub struct GraphMatrices {
    /// Standard adjacency matrix (triplet format)
    pub adjacency: SimpleSparseMatrix,
    /// Column-stochastic for PageRank (triplet format)
    pub transition: SimpleSparseMatrix,
    /// For spectral clustering (triplet format)
    pub laplacian: SimpleSparseMatrix,
    /// Out-degree vector
    pub out_degrees: Vec<f64>,
    /// Number of nodes in the graph
    pub node_count: usize,
    /// Edge list for efficient iteration (from, to, weight)
    pub edges: Vec<(usize, usize, f64)>,
}

// Implementation files (share this module's scope)
include!("types_dependency_graph.rs");
include!("types_undirected_graph.rs");
include!("types_matrices.rs");
include!("types_tests.rs");
