// Graph type system for PMAT - using trueno-graph (replaces petgraph)
// Complexity: All functions ≤ 10
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

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            graph: CsrGraph::new(),
            node_data: HashMap::new(),
            edge_data: HashMap::new(),
            next_id: 0,
        }
    }

    /// Add a node to the graph and return its ID
    pub fn add_node(&mut self, data: NodeData) -> NodeId {
        let id = TruenoNodeId(self.next_id);
        self.next_id += 1;
        self.node_data.insert(id, data);
        id
    }

    /// Add an edge between two nodes
    pub fn add_edge(&mut self, from: NodeId, to: NodeId, data: EdgeData) {
        // Store edge data
        self.edge_data.insert((from, to), data);
        // Add to trueno-graph with weight 1.0
        let _ = self.graph.add_edge(from, to, 1.0);
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.node_data.len()
    }

    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.edge_data.len()
    }

    /// Get node data by ID
    pub fn node_weight(&self, id: NodeId) -> Option<&NodeData> {
        self.node_data.get(&id)
    }

    /// Get mutable node data by ID
    pub fn node_weight_mut(&mut self, id: NodeId) -> Option<&mut NodeData> {
        self.node_data.get_mut(&id)
    }

    /// Get edge data between two nodes
    pub fn edge_weight(&self, from: NodeId, to: NodeId) -> Option<&EdgeData> {
        self.edge_data.get(&(from, to))
    }

    /// Check if an edge exists
    pub fn contains_edge(&self, from: NodeId, to: NodeId) -> bool {
        self.edge_data.contains_key(&(from, to))
    }

    /// Get all node IDs
    pub fn node_indices(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.node_data.keys().copied()
    }

    /// Iterate over all edges with their data
    pub fn edge_references(&self) -> impl Iterator<Item = EdgeRef<'_>> + '_ {
        self.edge_data.iter().map(|((from, to), data)| EdgeRef {
            source: *from,
            target: *to,
            weight: data,
        })
    }

    /// Get outgoing neighbors of a node
    pub fn neighbors(&self, node: NodeId) -> Vec<NodeId> {
        self.graph
            .outgoing_neighbors(node)
            .map(|neighbors| neighbors.iter().map(|&n| TruenoNodeId(n)).collect())
            .unwrap_or_default()
    }

    /// Get outgoing neighbors (alias for neighbors)
    pub fn neighbors_directed_outgoing(&self, node: NodeId) -> Vec<NodeId> {
        self.neighbors(node)
    }

    /// Get incoming neighbors of a node
    pub fn neighbors_directed_incoming(&self, node: NodeId) -> Vec<NodeId> {
        self.graph
            .incoming_neighbors(node)
            .map(|neighbors| neighbors.iter().map(|&n| TruenoNodeId(n)).collect())
            .unwrap_or_default()
    }

    /// Get edges from a node (outgoing)
    pub fn edges(&self, node: NodeId) -> impl Iterator<Item = EdgeRef<'_>> + '_ {
        self.edge_data
            .iter()
            .filter(move |((from, _), _)| *from == node)
            .map(|((from, to), data)| EdgeRef {
                source: *from,
                target: *to,
                weight: data,
            })
    }

    /// Get underlying trueno-graph (for algorithms)
    pub fn inner(&self) -> &CsrGraph {
        &self.graph
    }

    /// Iterate over nodes with their data
    pub fn node_references(&self) -> impl Iterator<Item = (NodeId, &NodeData)> + '_ {
        self.node_data.iter().map(|(id, data)| (*id, data))
    }
}

/// Edge reference for iteration
#[derive(Debug, Clone, Copy)]
pub struct EdgeRef<'a> {
    source: NodeId,
    target: NodeId,
    weight: &'a EdgeData,
}

impl<'a> EdgeRef<'a> {
    pub fn source(&self) -> NodeId {
        self.source
    }

    pub fn target(&self) -> NodeId {
        self.target
    }

    pub fn weight(&self) -> &'a EdgeData {
        self.weight
    }
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

impl Default for UndirectedGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl UndirectedGraph {
    pub fn new() -> Self {
        Self {
            graph: CsrGraph::new(),
            node_data: HashMap::new(),
            edge_weights: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_node(&mut self, data: NodeData) -> NodeId {
        let id = TruenoNodeId(self.next_id);
        self.next_id += 1;
        self.node_data.insert(id, data);
        id
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId, weight: f64) {
        // Store both directions for undirected access
        self.edge_weights.insert((from, to), weight);
        self.edge_weights.insert((to, from), weight);
        // Add to trueno-graph
        let _ = self.graph.add_edge(from, to, weight as f32);
        let _ = self.graph.add_edge(to, from, weight as f32);
    }

    pub fn node_count(&self) -> usize {
        self.node_data.len()
    }

    pub fn edge_count(&self) -> usize {
        // Divide by 2 since we store both directions
        self.edge_weights.len() / 2
    }

    pub fn node_weight(&self, id: NodeId) -> Option<&NodeData> {
        self.node_data.get(&id)
    }

    pub fn edge_weight(&self, from: NodeId, to: NodeId) -> Option<f64> {
        self.edge_weights.get(&(from, to)).copied()
    }

    pub fn node_indices(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.node_data.keys().copied()
    }

    pub fn neighbors(&self, node: NodeId) -> Vec<NodeId> {
        self.graph
            .outgoing_neighbors(node)
            .map(|neighbors| neighbors.iter().map(|&n| TruenoNodeId(n)).collect())
            .unwrap_or_default()
    }

    pub fn edge_references(&self) -> impl Iterator<Item = UndirectedEdgeRef<'_>> + '_ {
        // Only return each edge once (from < to)
        self.edge_weights
            .iter()
            .filter(|((from, to), _)| from.0 < to.0)
            .map(|((from, to), weight)| UndirectedEdgeRef {
                source: *from,
                target: *to,
                weight: *weight,
                _phantom: std::marker::PhantomData,
            })
    }

    pub fn inner(&self) -> &CsrGraph {
        &self.graph
    }

    pub fn node_references(&self) -> impl Iterator<Item = (NodeId, &NodeData)> + '_ {
        self.node_data.iter().map(|(id, data)| (*id, data))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UndirectedEdgeRef<'a> {
    source: NodeId,
    target: NodeId,
    weight: f64,
    #[allow(dead_code)]
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl UndirectedEdgeRef<'_> {
    pub fn source(&self) -> NodeId {
        self.source
    }

    pub fn target(&self) -> NodeId {
        self.target
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }
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

impl SimpleSparseMatrix {
    /// Create a new sparse matrix with given dimensions
    pub fn new(nrows: usize, ncols: usize) -> Self {
        Self {
            nrows,
            ncols,
            triplets: Vec::new(),
        }
    }

    /// Add a value at (row, col)
    pub fn push(&mut self, row: usize, col: usize, value: f64) {
        self.triplets.push((row, col, value));
    }

    /// Get row values as iterator
    pub fn row_values(&self, row: usize) -> impl Iterator<Item = (usize, f64)> + '_ {
        self.triplets
            .iter()
            .filter(move |(r, _, _)| *r == row)
            .map(|(_, c, v)| (*c, *v))
    }

    /// Get all values in a row as Vec
    pub fn row_as_vec(&self, row: usize) -> Vec<(usize, f64)> {
        self.row_values(row).collect()
    }
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

/// Conversion from DependencyGraph to matrix representations
/// Complexity: 8 (loop with matrix operations)
/// Sovereign AI Stack: Uses simple Vec-based sparse matrices
impl From<&DependencyGraph> for GraphMatrices {
    fn from(graph: &DependencyGraph) -> Self {
        let n = graph.node_count();
        let mut adjacency = SimpleSparseMatrix::new(n, n);
        let mut out_degrees = vec![0.0; n];
        let mut edges = Vec::new();

        // Build adjacency matrix with edge weights
        for edge in graph.edge_references() {
            let weight = edge.weight().to_numeric_weight();
            let source = edge.source().0 as usize;
            let target = edge.target().0 as usize;

            if source < n && target < n {
                edges.push((source, target, weight));
                adjacency.push(source, target, weight);
                out_degrees[source] += weight;
            }
        }

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
    fn normalize_columns(
        adjacency: &SimpleSparseMatrix,
        out_degrees: &[f64],
    ) -> SimpleSparseMatrix {
        let n = adjacency.nrows;
        let mut result = SimpleSparseMatrix::new(n, n);

        for i in 0..n {
            if out_degrees[i] > 0.0 {
                for (col, value) in adjacency.row_values(i) {
                    result.push(i, col, value / out_degrees[i]);
                }
            }
        }

        result
    }

    /// Compute graph Laplacian
    /// Complexity: 6 (simplified matrix operations)
    fn compute_laplacian(adjacency: &SimpleSparseMatrix) -> SimpleSparseMatrix {
        let n = adjacency.nrows;
        let mut result = SimpleSparseMatrix::new(n, n);

        // Compute degree matrix D and build Laplacian L = D - A
        for i in 0..n {
            let row_vals: Vec<_> = adjacency.row_values(i).collect();
            let degree: f64 = row_vals.iter().map(|(_, v)| v).sum();

            // Add diagonal degree
            result.push(i, i, degree);

            // Subtract adjacency values
            for (col, value) in row_vals {
                result.push(i, col, -value);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    // DependencyGraph Tests
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
    fn test_dependency_graph_new() {
        let graph = DependencyGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_dependency_graph_add_node() {
        let mut graph = DependencyGraph::new();
        let id = graph.add_node(create_test_node(0));
        assert_eq!(id.0, 0);
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn test_dependency_graph_add_edge() {
        let mut graph = DependencyGraph::new();
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

        assert_eq!(graph.edge_count(), 1);
        assert!(graph.contains_edge(n0, n1));
        assert!(!graph.contains_edge(n1, n0));
    }

    #[test]
    fn test_dependency_graph_node_weight() {
        let mut graph = DependencyGraph::new();
        let id = graph.add_node(create_test_node(42));

        let data = graph.node_weight(id).unwrap();
        assert_eq!(data.ast_hash, 42);
    }

    #[test]
    fn test_dependency_graph_edge_weight() {
        let mut graph = DependencyGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));

        graph.add_edge(
            n0,
            n1,
            EdgeData::FunctionCall {
                count: 5,
                async_call: true,
            },
        );

        let edge = graph.edge_weight(n0, n1).unwrap();
        if let EdgeData::FunctionCall { count, async_call } = edge {
            assert_eq!(*count, 5);
            assert!(*async_call);
        } else {
            panic!("Wrong edge type");
        }
    }

    #[test]
    fn test_dependency_graph_neighbors() {
        let mut graph = DependencyGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));
        let n2 = graph.add_node(create_test_node(2));

        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );
        graph.add_edge(
            n0,
            n2,
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );

        let neighbors = graph.neighbors(n0);
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&n1));
        assert!(neighbors.contains(&n2));
    }

    #[test]
    fn test_dependency_graph_edge_references() {
        let mut graph = DependencyGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));

        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 2.0,
                visibility: Visibility::Public,
            },
        );

        let edges: Vec<_> = graph.edge_references().collect();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].source(), n0);
        assert_eq!(edges[0].target(), n1);
    }

    // ============================================================
    // UndirectedGraph Tests
    // ============================================================

    #[test]
    fn test_undirected_graph_new() {
        let graph = UndirectedGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_undirected_graph_add_edge() {
        let mut graph = UndirectedGraph::new();
        let n0 = graph.add_node(create_test_node(0));
        let n1 = graph.add_node(create_test_node(1));

        graph.add_edge(n0, n1, 1.5);

        assert_eq!(graph.edge_count(), 1);
        // Both directions should have weight
        assert_eq!(graph.edge_weight(n0, n1), Some(1.5));
        assert_eq!(graph.edge_weight(n1, n0), Some(1.5));
    }

    // ============================================================
    // EdgeData Tests
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
    fn test_to_numeric_weight_function_call() {
        let edge = EdgeData::FunctionCall {
            count: 7,
            async_call: false,
        };
        assert_eq!(edge.to_numeric_weight(), 7.0);
    }

    #[test]
    fn test_to_numeric_weight_inheritance() {
        let edge = EdgeData::Inheritance { depth: 0 };
        assert_eq!(edge.to_numeric_weight(), 3.0); // 3.0 / (0 + 1)

        let edge = EdgeData::Inheritance { depth: 2 };
        assert_eq!(edge.to_numeric_weight(), 1.0); // 3.0 / (2 + 1)
    }

    // ============================================================
    // GraphMatrices Tests
    // ============================================================

    #[test]
    fn test_graph_matrices_empty_graph() {
        let graph = DependencyGraph::new();
        let matrices = GraphMatrices::from(&graph);

        assert_eq!(matrices.node_count, 0);
        assert_eq!(matrices.out_degrees.len(), 0);
        assert_eq!(matrices.edges.len(), 0);
    }

    #[test]
    fn test_graph_matrices_single_edge() {
        let mut graph = DependencyGraph::new();
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
        assert_eq!(matrices.edges.len(), 1);
    }
}
