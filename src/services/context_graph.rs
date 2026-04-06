#![cfg_attr(coverage_nightly, coverage(off))]
//! O(1) Context Graph - trueno-graph integration for fast symbol lookups
//!
//! This module provides CSR-backed graph storage for project context,
//! enabling O(1) symbol lookups and PageRank-based "hot path" identification.
//!
//! # Architecture
//!
//! **Graph Schema**:
//! - Nodes: AstItem (functions, structs, enums, traits)
//! - Edges: Relationships (caller → callee, user → used_struct)
//! - PageRank: Identifies "hot" code paths (frequently used symbols)
//!
//! # Example
//!
//! ```rust,ignore
//! use pmat::services::context_graph::ProjectContextGraph;
//! use pmat::services::context::AstItem;
//!
//! let mut graph = ProjectContextGraph::new();
//!
//! // Add functions
//! graph.add_item("main".to_string(), AstItem::Function {
//!     name: "main".to_string(),
//!     visibility: "pub".to_string(),
//!     is_async: false,
//!     line: 1,
//! })?;
//!
//! graph.add_item("helper".to_string(), AstItem::Function {
//!     name: "helper".to_string(),
//!     visibility: "pub".to_string(),
//!     is_async: false,
//!     line: 10,
//! })?;
//!
//! // Add edge: main calls helper
//! graph.add_edge("main", "helper")?;
//!
//! // O(1) lookup
//! let item = graph.get_item("main");
//! assert!(item.is_some());
//!
//! // PageRank identifies hot paths
//! graph.update_hotness()?;
//! let hot = graph.hot_symbols();
//! println!("Hottest symbol: {:?}", hot[0]);
//! ```

use crate::services::context::AstItem;
use anyhow::{Context as _, Result};
use std::collections::HashMap;
use trueno_graph::{pagerank, CsrGraph, NodeId};

/// CSR-backed project context for O(1) symbol lookups
///
/// Uses trueno-graph for fast access and PageRank-based importance scoring.
#[derive(Debug, Clone)]
pub struct ProjectContextGraph {
    /// In-memory cache (symbol_name → AstItem)
    cache: HashMap<String, AstItem>,

    /// CSR graph for relationships
    /// Nodes: AstItem (functions, structs, etc.)
    /// Edges: (caller → callee, user → used_struct)
    graph: CsrGraph,

    /// Node ID mapping (symbol_name → NodeId)
    node_map: HashMap<String, NodeId>,

    /// Reverse mapping (NodeId → symbol_name)
    reverse_node_map: HashMap<NodeId, String>,

    /// PageRank scores (symbol_name → hotness score)
    hotness_cache: HashMap<String, f32>,

    /// Next node ID counter
    next_node_id: u32,
}

impl ProjectContextGraph {
    /// Create new project context graph
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            graph: CsrGraph::new(),
            node_map: HashMap::new(),
            reverse_node_map: HashMap::new(),
            hotness_cache: HashMap::new(),
            next_node_id: 0,
        }
    }

    /// Add AstItem to graph (O(1))
    ///
    /// Creates a new node in the CSR graph and stores mappings for fast lookups.
    /// Also calls `graph.set_node_name()` for consistency with trueno-graph patterns.
    ///
    /// # Arguments
    ///
    /// * `name` - Symbol name (unique identifier)
    /// * `item` - AstItem to store
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Errors
    ///
    /// Returns error if item already exists (duplicate symbol name)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_item(&mut self, name: String, item: AstItem) -> Result<()> {
        // Check for duplicates
        if self.node_map.contains_key(&name) {
            anyhow::bail!("Duplicate symbol name: {}", name);
        }

        // Create node
        let node_id = NodeId(self.next_node_id);
        self.next_node_id += 1;

        // Store mappings
        self.node_map.insert(name.clone(), node_id);
        self.reverse_node_map.insert(node_id, name.clone());
        self.cache.insert(name.clone(), item);

        // Set node name in graph (trueno-graph pattern from examples)
        self.graph.set_node_name(node_id, name);

        Ok(())
    }

    /// Add edge between items (e.g., function calls function)
    ///
    /// Creates a directed edge in the CSR graph representing a relationship.
    ///
    /// # Arguments
    ///
    /// * `from` - Source symbol name (caller)
    /// * `to` - Target symbol name (callee)
    ///
    /// # Returns
    ///
    /// Ok(()) on success, silently ignores if either symbol doesn't exist
    ///
    /// # Errors
    ///
    /// Returns error if CSR graph operation fails
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_edge(&mut self, from: &str, to: &str) -> Result<()> {
        debug_assert!(!from.is_empty(), "from must not be empty");
        debug_assert!(!to.is_empty(), "to must not be empty");
        if let (Some(&from_id), Some(&to_id)) = (self.node_map.get(from), self.node_map.get(to)) {
            self.graph
                .add_edge(from_id, to_id, 1.0)
                .context("Failed to add edge to CSR graph")?;
        }
        Ok(())
    }

    /// Get item by name (O(1))
    ///
    /// Fast hash-based lookup of AstItem by symbol name.
    ///
    /// # Arguments
    ///
    /// * `name` - Symbol name to look up
    ///
    /// # Returns
    ///
    /// Some(&AstItem) if found, None otherwise
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_item(&self, name: &str) -> Option<&AstItem> {
        debug_assert!(!name.is_empty(), "name must not be empty");
        self.cache.get(name)
    }

    /// Update PageRank hotness scores
    ///
    /// Runs PageRank algorithm on the CSR graph to identify "hot" symbols
    /// (frequently used functions, structs, etc.).
    ///
    /// # Errors
    ///
    /// Returns error if PageRank computation fails
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn update_hotness(&mut self) -> Result<()> {
        if self.graph.num_nodes() == 0 {
            return Ok(());
        }

        // Run PageRank (20 iterations, tolerance 1e-6)
        let scores = pagerank(&self.graph, 20, 1e-6).context("PageRank computation failed")?;

        // Aggregate scores by symbol name
        self.hotness_cache.clear();
        for (node_id, score) in scores.iter().enumerate() {
            let node_id = NodeId(node_id as u32);
            if let Some(name) = self.reverse_node_map.get(&node_id) {
                self.hotness_cache.insert(name.clone(), *score);
            }
        }

        Ok(())
    }

    /// Get hot symbols (sorted by PageRank score)
    ///
    /// Returns all symbols ranked by importance (PageRank score).
    ///
    /// # Returns
    ///
    /// Vec<(symbol_name, pagerank_score)> sorted by score (highest first)
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn hot_symbols(&self) -> Vec<(String, f32)> {
        let mut symbols: Vec<_> = self
            .hotness_cache
            .iter()
            .map(|(name, score)| (name.clone(), *score))
            .collect();
        symbols.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        symbols
    }

    /// Get number of nodes in graph
    ///
    /// Returns the count of nodes we've added to the graph (tracked via node_map),
    /// not the CSR graph's node count (which only tracks nodes with edges).
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn num_nodes(&self) -> usize {
        self.node_map.len()
    }

    /// Get number of edges in graph
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn num_edges(&self) -> usize {
        self.graph.num_edges()
    }
}

impl Default for ProjectContextGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    include!("context_graph_tests.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    include!("context_graph_coverage_fixtures.rs");
    include!("context_graph_coverage_core.rs");
    include!("context_graph_coverage_extended.rs");
}

// Design-by-contract specifications (Verus-style)
// #[requires(project_path.is_dir())]
// #[ensures(result.is_ok() ==> ret.len() > 0)]
