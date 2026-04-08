#![cfg_attr(coverage_nightly, coverage(off))]
//! Fixed-size graph builder with intelligent node selection
//!
//! This module creates optimally-sized dependency graphs that respect rendering
//! limitations while preserving the most important structural information.
//! It uses `PageRank` algorithm to identify critical nodes and ensures generated
//! graphs are both informative and renderable.
//!
//! # Problem Solved
//!
//! Large codebases can have thousands of dependencies, making visualization
//! impossible. This builder:
//! - Limits graph size to renderable limits (e.g., Mermaid's 500 edge limit)
//! - Selects the most important nodes using `PageRank` scores
//! - Preserves critical structural relationships
//! - Groups related nodes for better organization
//!
//! # Algorithm
//!
//! 1. **`PageRank` Calculation**: Identify important nodes by connectivity
//! 2. **Node Selection**: Choose top N nodes by `PageRank` score
//! 3. **Edge Filtering**: Keep only edges between selected nodes
//! 4. **Grouping**: Optionally group by module or directory
//! 5. **Simplification**: Remove redundant edges and merge similar nodes
//!
//! # Example
//!
//! ```no_run
//! use pmat::services::fixed_graph_builder::{FixedGraphBuilder, GraphConfig, GroupingStrategy};
//! use pmat::models::dag::DependencyGraph;
//!
//! let config = GraphConfig {
//!     max_nodes: 50,
//!     max_edges: 400,
//!     grouping: GroupingStrategy::Module,
//! };
//!
//! let builder = FixedGraphBuilder::new(config);
//! let full_graph = DependencyGraph::new(); // Your full dependency graph
//!
//! let fixed_graph = builder.build(&full_graph).unwrap();
//!
//! println!("Reduced from {} to {} nodes",
//!          full_graph.nodes.len(), fixed_graph.nodes.len());
//! println!("Reduced from {} to {} edges",
//!          full_graph.edges.len(), fixed_graph.edges.len());
//!
//! // Graph is now guaranteed to be renderable
//! assert!(fixed_graph.nodes.len() <= 50);
//! assert!(fixed_graph.edges.len() <= 400);
//! ```

use anyhow::Result;
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::models::dag::{DependencyGraph, Edge, NodeInfo, NodeType};
use crate::services::semantic_naming::SemanticNamer;

/// Configuration for graph building
#[derive(Debug, Clone)]
pub struct GraphConfig {
    pub max_nodes: usize,
    pub max_edges: usize,
    pub grouping: GroupingStrategy,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            max_nodes: 20,
            max_edges: 60,
            grouping: GroupingStrategy::Module,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Strategy options for grouping.
pub enum GroupingStrategy {
    Module,
    Directory,
    None,
}

/// A fixed-size graph with semantic names
#[derive(Debug, Clone, PartialEq)]
pub struct FixedGraph {
    pub nodes: BTreeMap<String, FixedNode>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq)]
/// Fixed node.
pub struct FixedNode {
    pub id: String,
    pub display_name: String,
    pub node_type: NodeType,
    pub complexity: u64,
}

/// Builds fixed-size graphs with PageRank-based node selection
pub struct FixedGraphBuilder {
    max_nodes: usize,
    max_edges: usize,
    namer: SemanticNamer,
}

// --- Submodule includes ---

include!("fixed_graph_builder_core.rs");
include!("fixed_graph_builder_tests.rs");
