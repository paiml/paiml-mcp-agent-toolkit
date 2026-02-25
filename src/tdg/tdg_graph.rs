//! O(1) TDG Graph - trueno-graph integration for fast dependency tracking
//!
//! This module provides CSR-backed graph storage for TDG (Test-Driven Grade) analysis,
//! enabling O(1) function dependency lookups and PageRank-based critical function identification.
//!
//! # Architecture
//!
//! **Graph Schema**:
//! - Nodes: Function names
//! - Edges: Function calls (caller → callee)
//! - PageRank: Identifies critical functions (high in-degree = many callers)
//!
//! # Example
//!
//! ```rust,ignore
//! use pmat::tdg::tdg_graph::TdgGraph;
//!
//! let mut graph = TdgGraph::new();
//!
//! // Add functions
//! graph.add_function("main".to_string())?;
//! graph.add_function("helper".to_string())?;
//!
//! // Add edge: main calls helper
//! graph.add_edge("main", "helper")?;
//!
//! // O(1) lookup
//! let exists = graph.has_function("main");
//! assert!(exists);
//!
//! // PageRank identifies critical functions
//! graph.update_criticality()?;
//! let critical = graph.critical_functions();
//! println!("Most critical: {:?}", critical[0]);
//! ```

use anyhow::{Context as _, Result};
use std::collections::HashMap;
use trueno_graph::{pagerank, CsrGraph, NodeId};

/// CSR-backed TDG dependency graph for O(1) function lookups
///
/// Uses trueno-graph for fast access and PageRank-based criticality scoring.
#[derive(Debug, Clone)]
pub struct TdgGraph {
    /// CSR graph for function dependencies
    /// Nodes: Function names
    /// Edges: (caller → callee)
    graph: CsrGraph,

    /// Node ID mapping (function_name → NodeId)
    node_map: HashMap<String, NodeId>,

    /// Reverse mapping (NodeId → function_name)
    reverse_node_map: HashMap<NodeId, String>,

    /// PageRank scores (function_name → criticality score)
    /// Higher score = more critical (many incoming edges)
    criticality_scores: HashMap<String, f32>,

    /// Next node ID counter
    next_node_id: u32,
}

impl Default for TdgGraph {
    fn default() -> Self {
        Self::new()
    }
}

// Core operations: new, add_function, add_edge, has_function,
// update_criticality, critical_functions, num_nodes, num_edges
include!("tdg_graph_operations.rs");

// Terminal graph visualization (trueno-viz integration)
include!("tdg_graph_viz.rs");

// Unit tests
include!("tdg_graph_tests.rs");
