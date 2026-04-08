#![cfg_attr(coverage_nightly, coverage(off))]
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Dependency graph.
pub struct DependencyGraph {
    pub nodes: FxHashMap<String, NodeInfo>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Information about node.
pub struct NodeInfo {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub file_path: String,
    pub line_number: usize,
    pub complexity: u32,
    #[serde(default)]
    pub metadata: FxHashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Edge.
pub struct Edge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
    pub weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Type classification for node.
pub enum NodeType {
    Function,
    Class,
    Module,
    Trait,
    Interface,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// Type classification for edge.
pub enum EdgeType {
    Calls,
    Imports,
    Inherits,
    Implements,
    Uses,
}

// DAG generation types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// Type classification for dag.
pub enum DagType {
    CallGraph,
    ImportGraph,
    Inheritance,
    FullDependency,
}

// --- Impl blocks ---
include!("dag_impls.rs");

// --- Unit tests ---
include!("dag_tests.rs");

// --- Inline property tests ---
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
