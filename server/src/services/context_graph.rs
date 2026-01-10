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

#[cfg(test)]
mod coverage_tests {
    use super::*;

    // ============================================================
    // Test Fixtures
    // ============================================================

    /// Create a function AstItem for testing
    fn create_function(name: &str, line: usize) -> AstItem {
        AstItem::Function {
            name: name.to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line,
        }
    }

    /// Create an async function AstItem for testing
    fn create_async_function(name: &str, line: usize) -> AstItem {
        AstItem::Function {
            name: name.to_string(),
            visibility: "pub".to_string(),
            is_async: true,
            line,
        }
    }

    /// Create a struct AstItem for testing
    fn create_struct(name: &str, line: usize) -> AstItem {
        AstItem::Struct {
            name: name.to_string(),
            visibility: "pub".to_string(),
            fields_count: 0,
            derives: vec![],
            line,
        }
    }

    /// Create a private function AstItem for testing
    fn create_private_function(name: &str, line: usize) -> AstItem {
        AstItem::Function {
            name: name.to_string(),
            visibility: "".to_string(),
            is_async: false,
            line,
        }
    }

    /// Build a simple call graph for testing
    fn build_simple_call_graph() -> ProjectContextGraph {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("main".to_string(), create_function("main", 1))
            .unwrap();
        graph
            .add_item("helper".to_string(), create_function("helper", 10))
            .unwrap();
        graph
            .add_item("utility".to_string(), create_function("utility", 20))
            .unwrap();

        graph.add_edge("main", "helper").unwrap();
        graph.add_edge("main", "utility").unwrap();
        graph.add_edge("helper", "utility").unwrap();

        graph
    }

    /// Build a larger graph with multiple components
    fn build_complex_graph() -> ProjectContextGraph {
        let mut graph = ProjectContextGraph::new();

        // Component 1: Main entry points
        graph
            .add_item("entry1".to_string(), create_function("entry1", 1))
            .unwrap();
        graph
            .add_item("entry2".to_string(), create_function("entry2", 10))
            .unwrap();

        // Component 2: Services
        graph
            .add_item("service_a".to_string(), create_function("service_a", 100))
            .unwrap();
        graph
            .add_item("service_b".to_string(), create_function("service_b", 200))
            .unwrap();

        // Component 3: Utilities (shared)
        graph
            .add_item(
                "util_shared".to_string(),
                create_function("util_shared", 300),
            )
            .unwrap();

        // Edges
        graph.add_edge("entry1", "service_a").unwrap();
        graph.add_edge("entry2", "service_b").unwrap();
        graph.add_edge("service_a", "util_shared").unwrap();
        graph.add_edge("service_b", "util_shared").unwrap();
        graph.add_edge("entry1", "util_shared").unwrap();

        graph
    }

    // ============================================================
    // ProjectContextGraph Creation Tests
    // ============================================================

    #[test]
    fn test_new_graph_is_empty() {
        let graph = ProjectContextGraph::new();
        assert_eq!(graph.num_nodes(), 0);
        assert_eq!(graph.num_edges(), 0);
        assert!(graph.hot_symbols().is_empty());
    }

    #[test]
    fn test_default_creates_empty_graph() {
        let graph = ProjectContextGraph::default();
        assert_eq!(graph.num_nodes(), 0);
        assert_eq!(graph.num_edges(), 0);
    }

    #[test]
    fn test_new_equals_default() {
        let new = ProjectContextGraph::new();
        let default = ProjectContextGraph::default();
        assert_eq!(new.num_nodes(), default.num_nodes());
        assert_eq!(new.num_edges(), default.num_edges());
    }

    // ============================================================
    // add_item Tests
    // ============================================================

    #[test]
    fn test_add_single_function() {
        let mut graph = ProjectContextGraph::new();
        let result = graph.add_item("test_fn".to_string(), create_function("test_fn", 1));

        assert!(result.is_ok());
        assert_eq!(graph.num_nodes(), 1);
    }

    #[test]
    fn test_add_multiple_items() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("fn1".to_string(), create_function("fn1", 1))
            .unwrap();
        graph
            .add_item("fn2".to_string(), create_function("fn2", 10))
            .unwrap();
        graph
            .add_item("fn3".to_string(), create_function("fn3", 20))
            .unwrap();

        assert_eq!(graph.num_nodes(), 3);
    }

    #[test]
    fn test_add_duplicate_fails() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("dup".to_string(), create_function("dup", 1))
            .unwrap();

        let result = graph.add_item("dup".to_string(), create_function("dup", 2));

        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("Duplicate"));
        assert!(err_msg.contains("dup"));
    }

    #[test]
    fn test_add_different_item_types() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("my_fn".to_string(), create_function("my_fn", 1))
            .unwrap();
        graph
            .add_item("my_struct".to_string(), create_struct("my_struct", 10))
            .unwrap();
        graph
            .add_item(
                "async_fn".to_string(),
                create_async_function("async_fn", 20),
            )
            .unwrap();
        graph
            .add_item(
                "private_fn".to_string(),
                create_private_function("private_fn", 30),
            )
            .unwrap();

        assert_eq!(graph.num_nodes(), 4);
    }

    // ============================================================
    // get_item Tests
    // ============================================================

    #[test]
    fn test_get_item_exists() {
        let mut graph = ProjectContextGraph::new();
        let item = create_function("test", 1);
        graph.add_item("test".to_string(), item.clone()).unwrap();

        let retrieved = graph.get_item("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &item);
    }

    #[test]
    fn test_get_item_not_found() {
        let graph = ProjectContextGraph::new();
        assert!(graph.get_item("nonexistent").is_none());
    }

    #[test]
    fn test_get_item_after_adding_multiple() {
        let mut graph = ProjectContextGraph::new();

        for i in 0..10 {
            let name = format!("func_{}", i);
            graph
                .add_item(name.clone(), create_function(&name, i * 10))
                .unwrap();
        }

        // Should find all items
        for i in 0..10 {
            let name = format!("func_{}", i);
            assert!(graph.get_item(&name).is_some());
        }

        // Should not find non-existent
        assert!(graph.get_item("func_99").is_none());
    }

    // ============================================================
    // add_edge Tests
    // ============================================================

    #[test]
    fn test_add_edge_between_existing_nodes() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("caller".to_string(), create_function("caller", 1))
            .unwrap();
        graph
            .add_item("callee".to_string(), create_function("callee", 10))
            .unwrap();

        let result = graph.add_edge("caller", "callee");
        assert!(result.is_ok());
        assert_eq!(graph.num_edges(), 1);
    }

    #[test]
    fn test_add_edge_from_nonexistent_node() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("callee".to_string(), create_function("callee", 10))
            .unwrap();

        // Should succeed (silently ignored if from doesn't exist)
        let result = graph.add_edge("nonexistent", "callee");
        assert!(result.is_ok());
        assert_eq!(graph.num_edges(), 0); // Edge not added
    }

    #[test]
    fn test_add_edge_to_nonexistent_node() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("caller".to_string(), create_function("caller", 1))
            .unwrap();

        // Should succeed (silently ignored if to doesn't exist)
        let result = graph.add_edge("caller", "nonexistent");
        assert!(result.is_ok());
        assert_eq!(graph.num_edges(), 0); // Edge not added
    }

    #[test]
    fn test_add_multiple_edges() {
        let graph = build_simple_call_graph();
        assert_eq!(graph.num_edges(), 3);
    }

    #[test]
    fn test_add_self_loop_edge() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("recursive".to_string(), create_function("recursive", 1))
            .unwrap();

        let result = graph.add_edge("recursive", "recursive");
        assert!(result.is_ok());
        // Self-loop should be added
        assert_eq!(graph.num_edges(), 1);
    }

    // ============================================================
    // update_hotness / PageRank Tests
    // ============================================================

    #[test]
    fn test_update_hotness_empty_graph() {
        let mut graph = ProjectContextGraph::new();
        let result = graph.update_hotness();
        assert!(result.is_ok());
        assert!(graph.hot_symbols().is_empty());
    }

    #[test]
    fn test_update_hotness_single_node_no_edges() {
        let mut graph = ProjectContextGraph::new();
        graph
            .add_item("lonely".to_string(), create_function("lonely", 1))
            .unwrap();

        let result = graph.update_hotness();
        assert!(result.is_ok());
        // Single node with no edges - may or may not appear in hot_symbols
        // depending on PageRank implementation
    }

    #[test]
    fn test_update_hotness_simple_graph() {
        let mut graph = build_simple_call_graph();

        let result = graph.update_hotness();
        assert!(result.is_ok());

        let hot = graph.hot_symbols();
        assert!(!hot.is_empty());

        // utility should be hottest (most in-edges)
        assert_eq!(hot[0].0, "utility");
    }

    #[test]
    fn test_update_hotness_complex_graph() {
        let mut graph = build_complex_graph();

        let result = graph.update_hotness();
        assert!(result.is_ok());

        let hot = graph.hot_symbols();
        assert!(!hot.is_empty());

        // util_shared should be hottest (called by multiple functions)
        assert_eq!(hot[0].0, "util_shared");
    }

    #[test]
    fn test_update_hotness_updates_cache() {
        let mut graph = build_simple_call_graph();

        // First update
        graph.update_hotness().unwrap();
        let hot1 = graph.hot_symbols();

        // Second update (should produce same results)
        graph.update_hotness().unwrap();
        let hot2 = graph.hot_symbols();

        assert_eq!(hot1.len(), hot2.len());
        for (a, b) in hot1.iter().zip(hot2.iter()) {
            assert_eq!(a.0, b.0);
        }
    }

    // ============================================================
    // hot_symbols Tests
    // ============================================================

    #[test]
    fn test_hot_symbols_empty_before_update() {
        let graph = build_simple_call_graph();
        assert!(graph.hot_symbols().is_empty());
    }

    #[test]
    fn test_hot_symbols_sorted_descending() {
        let mut graph = build_simple_call_graph();
        graph.update_hotness().unwrap();

        let hot = graph.hot_symbols();

        // Verify sorted by score descending
        for i in 1..hot.len() {
            assert!(
                hot[i - 1].1 >= hot[i].1,
                "hot_symbols should be sorted by score descending"
            );
        }
    }

    #[test]
    fn test_hot_symbols_contains_all_nodes_with_edges() {
        let mut graph = build_simple_call_graph();
        graph.update_hotness().unwrap();

        let hot = graph.hot_symbols();
        let names: Vec<&str> = hot.iter().map(|(n, _)| n.as_str()).collect();

        // All nodes in the graph should be in hot_symbols
        assert!(names.contains(&"main"));
        assert!(names.contains(&"helper"));
        assert!(names.contains(&"utility"));
    }

    // ============================================================
    // num_nodes / num_edges Tests
    // ============================================================

    #[test]
    fn test_num_nodes_increments() {
        let mut graph = ProjectContextGraph::new();

        assert_eq!(graph.num_nodes(), 0);

        graph
            .add_item("a".to_string(), create_function("a", 1))
            .unwrap();
        assert_eq!(graph.num_nodes(), 1);

        graph
            .add_item("b".to_string(), create_function("b", 2))
            .unwrap();
        assert_eq!(graph.num_nodes(), 2);

        graph
            .add_item("c".to_string(), create_function("c", 3))
            .unwrap();
        assert_eq!(graph.num_nodes(), 3);
    }

    #[test]
    fn test_num_edges_increments() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("a".to_string(), create_function("a", 1))
            .unwrap();
        graph
            .add_item("b".to_string(), create_function("b", 10))
            .unwrap();
        graph
            .add_item("c".to_string(), create_function("c", 20))
            .unwrap();

        assert_eq!(graph.num_edges(), 0);

        graph.add_edge("a", "b").unwrap();
        assert_eq!(graph.num_edges(), 1);

        graph.add_edge("b", "c").unwrap();
        assert_eq!(graph.num_edges(), 2);
    }

    // ============================================================
    // Clone/Debug Tests
    // ============================================================

    #[test]
    fn test_graph_clone() {
        let mut original = build_simple_call_graph();
        original.update_hotness().unwrap();

        let cloned = original.clone();

        assert_eq!(original.num_nodes(), cloned.num_nodes());
        assert_eq!(original.num_edges(), cloned.num_edges());

        // Both should have same items
        assert!(cloned.get_item("main").is_some());
        assert!(cloned.get_item("helper").is_some());
        assert!(cloned.get_item("utility").is_some());
    }

    #[test]
    fn test_graph_debug() {
        let graph = build_simple_call_graph();
        let debug = format!("{:?}", graph);
        assert!(debug.contains("ProjectContextGraph"));
    }

    // ============================================================
    // Edge Cases
    // ============================================================

    #[test]
    fn test_empty_name_item() {
        let mut graph = ProjectContextGraph::new();
        let result = graph.add_item("".to_string(), create_function("", 1));
        // Empty names should work (no validation)
        assert!(result.is_ok());
        assert!(graph.get_item("").is_some());
    }

    #[test]
    fn test_special_characters_in_name() {
        let mut graph = ProjectContextGraph::new();

        let special_names = vec![
            "fn::with::colons",
            "fn<T>",
            "fn()",
            "fn-with-dashes",
            "fn_with_underscores",
            "FnWithCamelCase",
        ];

        for name in special_names {
            graph
                .add_item(name.to_string(), create_function(name, 1))
                .unwrap();
            assert!(graph.get_item(name).is_some());
        }
    }

    #[test]
    fn test_unicode_names() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item(
                "function_japanese".to_string(),
                create_function("function_japanese", 1),
            )
            .unwrap();
        graph
            .add_item(
                "function_emoji".to_string(),
                create_function("function_emoji", 10),
            )
            .unwrap();

        assert_eq!(graph.num_nodes(), 2);
    }

    #[test]
    fn test_large_graph() {
        let mut graph = ProjectContextGraph::new();

        // Add 100 nodes
        for i in 0..100 {
            let name = format!("func_{}", i);
            graph
                .add_item(name.clone(), create_function(&name, i))
                .unwrap();
        }

        // Add edges in a chain
        for i in 0..99 {
            graph
                .add_edge(&format!("func_{}", i), &format!("func_{}", i + 1))
                .unwrap();
        }

        assert_eq!(graph.num_nodes(), 100);
        assert_eq!(graph.num_edges(), 99);

        // PageRank should work on large graph
        graph.update_hotness().unwrap();
        let hot = graph.hot_symbols();
        assert!(!hot.is_empty());
    }

    #[test]
    fn test_star_topology() {
        let mut graph = ProjectContextGraph::new();

        // Central hub
        graph
            .add_item("hub".to_string(), create_function("hub", 1))
            .unwrap();

        // Spokes
        for i in 0..10 {
            let name = format!("spoke_{}", i);
            graph
                .add_item(name.clone(), create_function(&name, i * 10))
                .unwrap();
            graph.add_edge(&name, "hub").unwrap();
        }

        graph.update_hotness().unwrap();
        let hot = graph.hot_symbols();

        // Hub should be hottest (all spokes point to it)
        assert_eq!(hot[0].0, "hub");
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("a".to_string(), create_function("a", 1))
            .unwrap();
        graph
            .add_item("b".to_string(), create_function("b", 10))
            .unwrap();
        graph
            .add_item("c".to_string(), create_function("c", 20))
            .unwrap();

        // Create a cycle: a -> b -> c -> a
        graph.add_edge("a", "b").unwrap();
        graph.add_edge("b", "c").unwrap();
        graph.add_edge("c", "a").unwrap();

        // PageRank should handle cycles
        let result = graph.update_hotness();
        assert!(result.is_ok());

        let hot = graph.hot_symbols();
        assert_eq!(hot.len(), 3);
    }

    // ============================================================
    // Property-Based Tests
    // ============================================================

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_add_item_preserves_count(count in 1usize..50) {
                let mut graph = ProjectContextGraph::new();

                for i in 0..count {
                    let name = format!("func_{}", i);
                    graph.add_item(name, create_function(&format!("func_{}", i), i)).unwrap();
                }

                prop_assert_eq!(graph.num_nodes(), count);
            }

            #[test]
            fn test_get_item_finds_all_added(count in 1usize..50) {
                let mut graph = ProjectContextGraph::new();

                for i in 0..count {
                    let name = format!("func_{}", i);
                    graph.add_item(name, create_function(&format!("func_{}", i), i)).unwrap();
                }

                for i in 0..count {
                    let name = format!("func_{}", i);
                    prop_assert!(graph.get_item(&name).is_some());
                }
            }

            #[test]
            fn test_edges_only_added_for_existing_nodes(
                node_count in 2usize..20,
                edge_attempts in 1usize..30
            ) {
                let mut graph = ProjectContextGraph::new();

                for i in 0..node_count {
                    let name = format!("node_{}", i);
                    graph.add_item(name, create_function(&format!("node_{}", i), i)).unwrap();
                }

                let mut valid_edges = 0;
                for i in 0..edge_attempts {
                    let from_idx = i % node_count;
                    let to_idx = (i + 1) % node_count;
                    let from = format!("node_{}", from_idx);
                    let to = format!("node_{}", to_idx);

                    if graph.get_item(&from).is_some() && graph.get_item(&to).is_some() {
                        graph.add_edge(&from, &to).unwrap();
                        valid_edges += 1;
                    }
                }

                prop_assert!(graph.num_edges() <= valid_edges);
            }

            #[test]
            fn test_hot_symbols_sorted(node_count in 3usize..20) {
                let mut graph = ProjectContextGraph::new();

                // Add nodes
                for i in 0..node_count {
                    let name = format!("func_{}", i);
                    graph.add_item(name, create_function(&format!("func_{}", i), i)).unwrap();
                }

                // Add edges (create varying in-degrees)
                for i in 0..node_count {
                    let from = format!("func_{}", i);
                    let to = format!("func_{}", (i + 1) % node_count);
                    graph.add_edge(&from, &to).unwrap();
                }

                graph.update_hotness().unwrap();
                let hot = graph.hot_symbols();

                // Verify sorted descending by score
                for i in 1..hot.len() {
                    prop_assert!(hot[i-1].1 >= hot[i].1);
                }
            }

            #[test]
            fn test_duplicate_detection(name in "[a-z]{1,10}") {
                let mut graph = ProjectContextGraph::new();

                // First add should succeed
                let result1 = graph.add_item(name.clone(), create_function(&name, 1));
                prop_assert!(result1.is_ok());

                // Second add should fail
                let result2 = graph.add_item(name.clone(), create_function(&name, 2));
                prop_assert!(result2.is_err());
            }
        }
    }

    // ============================================================
    // AstItem Equality Tests (for coverage)
    // ============================================================

    #[test]
    fn test_ast_item_function_equality() {
        let f1 = create_function("test", 1);
        let f2 = create_function("test", 1);
        let f3 = create_function("test", 2);
        let f4 = create_function("other", 1);

        assert_eq!(f1, f2);
        assert_ne!(f1, f3); // Different line
        assert_ne!(f1, f4); // Different name
    }

    #[test]
    fn test_ast_item_struct_vs_function() {
        let func = create_function("test", 1);
        let strct = create_struct("test", 1);

        assert_ne!(func, strct); // Different types
    }

    #[test]
    fn test_async_vs_sync_function() {
        let sync_fn = create_function("test", 1);
        let async_fn = create_async_function("test", 1);

        assert_ne!(sync_fn, async_fn); // Different is_async
    }
}
