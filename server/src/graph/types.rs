// Graph type system for PMAT
// Complexity: All functions ≤ 10
// SATD: Zero tolerance

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use petgraph::graph::{DiGraph, UnGraph};
use petgraph::visit::EdgeRef;
use nalgebra_sparse::{CsrMatrix, CooMatrix};

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
    Import { weight: f64, visibility: Visibility },
    FunctionCall { count: usize, async_call: bool },
    TypeDependency { strength: f64, kind: TypeKind },
    DataFlow { confidence: f64, direction: FlowDirection },
    Inheritance { depth: usize },
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