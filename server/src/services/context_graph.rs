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
    pub fn add_edge(&mut self, from: &str, to: &str) -> Result<()> {
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
    pub fn get_item(&self, name: &str) -> Option<&AstItem> {
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
    pub fn num_nodes(&self) -> usize {
        self.node_map.len()
    }

    /// Get number of edges in graph
    #[must_use]
    pub fn num_edges(&self) -> usize {
        self.graph.num_edges()
    }
}

impl Default for ProjectContextGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_context_graph_creation() {
        let graph = ProjectContextGraph::new();
        assert_eq!(graph.num_nodes(), 0);
        assert_eq!(graph.num_edges(), 0);
    }

    #[test]
    fn test_add_item_o1_lookup() {
        let mut graph = ProjectContextGraph::new();

        let func = AstItem::Function {
            name: "test_func".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };

        graph
            .add_item("test_func".to_string(), func.clone())
            .unwrap();

        // O(1) lookup
        let retrieved = graph.get_item("test_func");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &func);

        // Non-existent lookup
        assert!(graph.get_item("nonexistent").is_none());
    }

    #[test]
    fn test_add_duplicate_fails() {
        let mut graph = ProjectContextGraph::new();

        let func = AstItem::Function {
            name: "dup".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };

        graph.add_item("dup".to_string(), func.clone()).unwrap();

        // Duplicate should fail
        let result = graph.add_item("dup".to_string(), func);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }

    #[test]
    fn test_add_edge_relationships() {
        let mut graph = ProjectContextGraph::new();

        // Add two functions
        graph
            .add_item(
                "main".to_string(),
                AstItem::Function {
                    name: "main".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 1,
                },
            )
            .unwrap();

        graph
            .add_item(
                "helper".to_string(),
                AstItem::Function {
                    name: "helper".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 10,
                },
            )
            .unwrap();

        // Add edge: main calls helper
        graph.add_edge("main", "helper").unwrap();

        assert_eq!(graph.num_nodes(), 2);
        assert_eq!(graph.num_edges(), 1);
    }

    #[test]
    fn test_pagerank_hotness() {
        let mut graph = ProjectContextGraph::new();

        // Create a simple call graph:
        // main → helper1
        // main → helper2
        // helper1 → helper2
        // (helper2 should have highest PageRank)

        graph
            .add_item(
                "main".to_string(),
                AstItem::Function {
                    name: "main".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 1,
                },
            )
            .unwrap();

        graph
            .add_item(
                "helper1".to_string(),
                AstItem::Function {
                    name: "helper1".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 10,
                },
            )
            .unwrap();

        graph
            .add_item(
                "helper2".to_string(),
                AstItem::Function {
                    name: "helper2".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 20,
                },
            )
            .unwrap();

        graph.add_edge("main", "helper1").unwrap();
        graph.add_edge("main", "helper2").unwrap();
        graph.add_edge("helper1", "helper2").unwrap();

        // Update PageRank
        graph.update_hotness().unwrap();

        // Get hot symbols
        let hot = graph.hot_symbols();
        assert_eq!(hot.len(), 3);

        // helper2 should be hottest (highest in-degree)
        assert_eq!(hot[0].0, "helper2");
        assert!(hot[0].1 > hot[1].1); // helper2 score > helper1 score
        assert!(hot[0].1 > hot[2].1); // helper2 score > main score
    }

    #[test]
    fn test_hot_symbols_ranking() {
        let mut graph = ProjectContextGraph::new();

        // Add 5 functions with varying in-degrees
        for i in 0..5 {
            graph
                .add_item(
                    format!("func{}", i),
                    AstItem::Function {
                        name: format!("func{}", i),
                        visibility: "pub".to_string(),
                        is_async: false,
                        line: i * 10,
                    },
                )
                .unwrap();
        }

        // func4 called by everyone (hottest)
        for i in 0..4 {
            graph.add_edge(&format!("func{}", i), "func4").unwrap();
        }

        graph.update_hotness().unwrap();
        let hot = graph.hot_symbols();

        // func4 should be #1
        assert_eq!(hot[0].0, "func4");
    }

    #[test]
    fn test_empty_graph_pagerank() {
        let mut graph = ProjectContextGraph::new();
        // Empty graph should not panic
        graph.update_hotness().unwrap();
        assert_eq!(graph.hot_symbols().len(), 0);
    }
}
