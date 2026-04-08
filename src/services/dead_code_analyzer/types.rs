#![allow(unused)]
//! Types, enums, and structs for dead code analysis.

use crate::models::unified_ast::NodeKey;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Hierarchical bitset for efficient reachability tracking
pub struct HierarchicalBitSet {
    levels: Vec<roaring::RoaringBitmap>,

    total_nodes: usize,
}

impl HierarchicalBitSet {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(capacity: usize) -> Self {
        Self {
            levels: vec![roaring::RoaringBitmap::new()],
            total_nodes: capacity,
        }
    }

    /// Sets the bit at the given index
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::dead_code_analyzer::HierarchicalBitSet;
    ///
    /// let mut bitset = HierarchicalBitSet::new(100);
    /// bitset.set(42);
    /// assert!(bitset.is_set(42));
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn set(&mut self, index: u32) {
        self.levels[0].insert(index);
    }

    /// Checks if the bit at the given index is set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::dead_code_analyzer::HierarchicalBitSet;
    ///
    /// let mut bitset = HierarchicalBitSet::new(100);
    /// assert!(!bitset.is_set(10));
    /// bitset.set(10);
    /// assert!(bitset.is_set(10));
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_set(&self, index: u32) -> bool {
        self.levels[0].contains(index)
    }

    /// As mut slice.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // For now, we'll use a simple approach without SIMD
        // This method isn't actually used in the current implementation
        // but we need it to compile
        &mut []
    }

    /// Returns the count of set bits
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::dead_code_analyzer::HierarchicalBitSet;
    ///
    /// let mut bitset = HierarchicalBitSet::new(100);
    /// assert_eq!(bitset.count_set(), 0);
    /// bitset.set(10);
    /// bitset.set(20);
    /// bitset.set(30);
    /// assert_eq!(bitset.count_set(), 3);
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn count_set(&self) -> usize {
        self.levels[0].len() as usize
    }
}

/// Cross-language reference graph for tracking dependencies across language boundaries
///
/// Maintains a directed graph of references between code elements across different
/// programming languages. Supports direct calls, imports, inheritance relationships,
/// and dynamic dispatch scenarios.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::dead_code_analyzer::{CrossLangReferenceGraph, ReferenceEdge, ReferenceNode};
/// use pmat::models::unified_ast::NodeKey;
/// use std::collections::HashMap;
///
/// // Note: edge_index is private, so we create an empty graph conceptually
/// let edges: Vec<ReferenceEdge> = vec![];
/// let nodes: HashMap<NodeKey, ReferenceNode> = HashMap::new();
///
/// // Graph starts empty and nodes/edges are added during AST analysis
/// assert!(edges.is_empty());
/// assert!(nodes.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct CrossLangReferenceGraph {
    /// Directed edges representing references between code elements
    pub edges: Vec<ReferenceEdge>,
    /// Nodes in the reference graph, indexed by their unique key
    pub nodes: HashMap<NodeKey, ReferenceNode>,
    /// Fast lookup index mapping nodes to their outgoing edges
    pub(crate) edge_index: HashMap<NodeKey, Vec<usize>>,
}

impl CrossLangReferenceGraph {
    /// Create a new empty reference graph
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn new() -> Self {
        Self {
            edges: Vec::new(),
            nodes: HashMap::new(),
            edge_index: HashMap::new(),
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Edges for chunk.
    pub fn edges_for_chunk(&self, _chunk: &[u8]) -> Vec<ReferenceEdge> {
        // TRACKED: Implement efficient edge lookup for chunks
        Vec::new()
    }
}

#[derive(Debug, Clone)]
/// Reference edge.
pub struct ReferenceEdge {
    pub from: NodeKey,
    pub to: NodeKey,
    pub reference_type: ReferenceType,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
/// Reference node.
pub struct ReferenceNode {
    pub key: NodeKey,
    pub name: String,
    pub language: crate::models::unified_ast::Language,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Type classification for reference.
pub enum ReferenceType {
    DirectCall,
    IndirectCall,
    Import,
    Inheritance,
    TypeReference,
    DynamicDispatch,
}

/// Virtual table resolver for dynamic dispatch analysis
///
/// Resolves virtual method calls and interface implementations to determine
/// all possible targets of dynamic dispatch. Critical for accurate dead code
/// detection in object-oriented languages like Java, C#, C++, and JavaScript.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::dead_code_analyzer::VTableResolver;
///
/// let resolver = VTableResolver::new();
///
/// // Resolve all possible targets for an interface method call
/// let targets = resolver.resolve_dynamic_call("Drawable", "draw");
///
/// // Initially empty - populated during AST analysis
/// assert!(targets.is_empty());
/// ```
pub struct VTableResolver {
    /// Virtual method tables for each class/interface
    vtables: HashMap<String, VTable>,
    /// Mapping from interfaces to their implementing types
    interface_impls: HashMap<String, Vec<String>>,
}

#[derive(Clone)]
struct VTable {
    base_type: String,
    methods: HashMap<String, NodeKey>,
}

impl Default for VTableResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl VTableResolver {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            vtables: HashMap::new(),
            interface_impls: HashMap::new(),
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Resolve dynamic call.
    pub fn resolve_dynamic_call(&self, interface: &str, method: &str) -> Vec<NodeKey> {
        let mut targets = Vec::new();

        if let Some(impls) = self.interface_impls.get(interface) {
            for impl_type in impls {
                if let Some(vtable) = self.vtables.get(impl_type) {
                    if let Some(&node_key) = vtable.methods.get(method) {
                        targets.push(node_key);
                    }
                }
            }
        }

        targets
    }
}

/// Coverage data integration from external tools (llvm-cov, grcov, etc.)
///
/// Integrates test coverage data to improve dead code detection accuracy by
/// identifying code that is syntactically reachable but never executed in practice.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::dead_code_analyzer::CoverageData;
/// use std::collections::{HashMap, HashSet};
///
/// let mut covered_lines = HashMap::new();
/// let mut main_rs_lines = HashSet::new();
/// main_rs_lines.insert(10);  // Line 10 is covered
/// main_rs_lines.insert(15);  // Line 15 is covered
/// covered_lines.insert("src/main.rs".to_string(), main_rs_lines);
///
/// let mut execution_counts = HashMap::new();
/// let mut main_rs_counts = HashMap::new();
/// main_rs_counts.insert(10, 5);  // Line 10 executed 5 times
/// main_rs_counts.insert(15, 1);  // Line 15 executed 1 time
/// execution_counts.insert("src/main.rs".to_string(), main_rs_counts);
///
/// let coverage_data = CoverageData {
///     covered_lines,
///     execution_counts,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageData {
    /// Lines that were covered during test execution, organized by file path
    pub covered_lines: HashMap<String, HashSet<u32>>,
    /// Number of times each line was executed during tests
    pub execution_counts: HashMap<String, HashMap<u32, u64>>,
}

/// Dead code analysis report containing all detected dead code segments
///
/// Provides a comprehensive report of unreachable code, unused functions, variables,
/// and classes discovered during analysis. Includes confidence scores and detailed
/// reasoning for each finding.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::dead_code_analyzer::{DeadCodeReport, DeadCodeSummary, DeadCodeItem};
/// use std::collections::HashMap;
///
/// // Create a sample report showing no dead code found
/// let report = DeadCodeReport {
///     dead_functions: vec![],
///     dead_classes: vec![],
///     dead_variables: vec![],
///     unreachable_code: vec![],
///     summary: DeadCodeSummary {
///         total_dead_code_lines: 0,
///         percentage_dead: 0.0,
///         dead_by_type: HashMap::new(),
///         confidence_level: 0.95,
///     },
/// };
///
/// assert_eq!(report.summary.total_dead_code_lines, 0);
/// assert!(report.dead_functions.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeReport {
    /// Functions that are defined but never called
    pub dead_functions: Vec<DeadCodeItem>,
    /// Classes that are defined but never instantiated or referenced
    pub dead_classes: Vec<DeadCodeItem>,
    /// Variables that are declared but never used
    pub dead_variables: Vec<DeadCodeItem>,
    /// Code blocks that are syntactically valid but unreachable
    pub unreachable_code: Vec<UnreachableBlock>,
    /// Statistical summary of the analysis
    pub summary: DeadCodeSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Dead code item.
pub struct DeadCodeItem {
    pub node_key: NodeKey,
    pub name: String,
    pub file_path: String,
    pub line_number: u32,
    pub dead_type: DeadCodeType,
    pub confidence: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Type classification for dead code.
pub enum DeadCodeType {
    UnusedFunction,
    UnusedClass,
    UnusedVariable,
    UnreachableCode,
    DeadBranch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Unreachable block.
pub struct UnreachableBlock {
    pub start_line: u32,
    pub end_line: u32,
    pub file_path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Summary of dead code analysis.
pub struct DeadCodeSummary {
    pub total_dead_code_lines: usize,
    pub percentage_dead: f32,
    pub dead_by_type: HashMap<String, usize>,
    pub confidence_level: f32,
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchical_bitset() {
        let mut bitset = HierarchicalBitSet::new(1000);

        bitset.set(10);
        bitset.set(100);

        assert!(bitset.is_set(10));
        assert!(bitset.is_set(100));
        assert!(!bitset.is_set(50));
        assert_eq!(bitset.count_set(), 2);
    }

    #[test]
    fn test_vtable_resolver() {
        let resolver = VTableResolver::new();

        let targets = resolver.resolve_dynamic_call("IRenderer", "render");
        assert_eq!(targets.len(), 0);
    }

    #[test]
    fn test_reference_edge_creation() {
        let edge = ReferenceEdge {
            from: 1,
            to: 2,
            reference_type: ReferenceType::DirectCall,
            confidence: 0.95,
        };

        assert_eq!(edge.from, 1);
        assert_eq!(edge.to, 2);
        assert_eq!(edge.reference_type, ReferenceType::DirectCall);
        assert_eq!(edge.confidence, 0.95);
    }

    #[test]
    fn test_reference_node_creation() {
        let node = ReferenceNode {
            key: 42,
            name: "test_function".to_string(),
            language: crate::models::unified_ast::Language::Rust,
        };

        assert_eq!(node.key, 42);
        assert_eq!(node.name, "test_function");
        assert_eq!(node.language, crate::models::unified_ast::Language::Rust);
    }

    #[test]
    fn test_coverage_data_creation() {
        use std::collections::{HashMap, HashSet};

        let mut covered_lines = HashMap::new();
        let mut line_set = HashSet::new();
        line_set.insert(10);
        line_set.insert(20);
        covered_lines.insert("test.rs".to_string(), line_set);

        let mut execution_counts = HashMap::new();
        let mut counts = HashMap::new();
        counts.insert(10, 5);
        counts.insert(20, 3);
        execution_counts.insert("test.rs".to_string(), counts);

        let coverage = CoverageData {
            covered_lines,
            execution_counts,
        };

        assert!(coverage.covered_lines.contains_key("test.rs"));
        assert!(coverage.execution_counts.contains_key("test.rs"));
    }

    #[test]
    fn test_cross_lang_reference_graph() {
        let mut graph = CrossLangReferenceGraph::new();

        let node = ReferenceNode {
            key: 1,
            name: "test".to_string(),
            language: crate::models::unified_ast::Language::Rust,
        };

        graph.nodes.insert(1, node);

        let edge = ReferenceEdge {
            from: 1,
            to: 2,
            reference_type: ReferenceType::DirectCall,
            confidence: 0.9,
        };

        graph.edges.push(edge);

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 1);

        let chunk = &[0u8; 8];
        let edges = graph.edges_for_chunk(chunk);
        assert_eq!(edges.len(), 0); // Implementation returns empty vec
    }
}
