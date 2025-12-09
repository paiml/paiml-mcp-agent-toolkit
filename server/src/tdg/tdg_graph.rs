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
use trueno_graph::{CsrGraph, NodeId, pagerank};

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

impl TdgGraph {
    /// Create new TDG graph
    #[must_use]
    pub fn new() -> Self {
        Self {
            graph: CsrGraph::new(),
            node_map: HashMap::new(),
            reverse_node_map: HashMap::new(),
            criticality_scores: HashMap::new(),
            next_node_id: 0,
        }
    }

    /// Add function to graph (O(1))
    ///
    /// # Arguments
    ///
    /// * `name` - Function name (unique identifier)
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Errors
    ///
    /// Returns error if function already exists (duplicate name)
    pub fn add_function(&mut self, name: String) -> Result<()> {
        // Check for duplicates
        if self.node_map.contains_key(&name) {
            anyhow::bail!("Duplicate function name: {}", name);
        }

        // Create node
        let node_id = NodeId(self.next_node_id);
        self.next_node_id += 1;

        // Store mappings
        self.node_map.insert(name.clone(), node_id);
        self.reverse_node_map.insert(node_id, name.clone());

        // Set node name in graph (trueno-graph pattern)
        self.graph.set_node_name(node_id, name);

        Ok(())
    }

    /// Add edge between functions (e.g., function calls function)
    ///
    /// # Arguments
    ///
    /// * `from` - Caller function name
    /// * `to` - Callee function name
    ///
    /// # Returns
    ///
    /// Ok(()) on success, silently ignores if either function doesn't exist
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

    /// Check if function exists in graph (O(1))
    ///
    /// # Arguments
    ///
    /// * `name` - Function name to check
    ///
    /// # Returns
    ///
    /// true if function exists, false otherwise
    #[must_use]
    pub fn has_function(&self, name: &str) -> bool {
        self.node_map.contains_key(name)
    }

    /// Update PageRank criticality scores
    ///
    /// Runs PageRank algorithm on the CSR graph to identify critical functions
    /// (functions with many incoming edges = called by many others).
    ///
    /// # Errors
    ///
    /// Returns error if PageRank computation fails
    pub fn update_criticality(&mut self) -> Result<()> {
        if self.graph.num_nodes() == 0 {
            return Ok(());
        }

        // Run PageRank (20 iterations, tolerance 1e-6)
        let scores = pagerank(&self.graph, 20, 1e-6).context("PageRank computation failed")?;

        // Aggregate scores by function name
        self.criticality_scores.clear();
        for (node_id, score) in scores.iter().enumerate() {
            let node_id = NodeId(node_id as u32);
            if let Some(name) = self.reverse_node_map.get(&node_id) {
                self.criticality_scores.insert(name.clone(), *score);
            }
        }

        Ok(())
    }

    /// Get critical functions (sorted by PageRank score)
    ///
    /// Returns functions ranked by criticality (PageRank score).
    ///
    /// # Returns
    ///
    /// Vec<(function_name, criticality_score)> sorted by score (highest first)
    #[must_use]
    pub fn critical_functions(&self) -> Vec<(String, f32)> {
        let mut functions: Vec<_> = self
            .criticality_scores
            .iter()
            .map(|(name, score)| (name.clone(), *score))
            .collect();
        functions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        functions
    }

    /// Get number of nodes in graph
    ///
    /// Returns the count of functions we've added to the graph.
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

impl Default for TdgGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Terminal Graph Visualization (trueno-viz integration)
// ============================================================================

#[cfg(feature = "viz")]
impl TdgGraph {
    /// Convert to visualization graph for terminal rendering
    ///
    /// Creates a `VisGraph` from the TDG dependency graph with criticality scores.
    ///
    /// # Returns
    ///
    /// `VisGraph` ready for terminal visualization
    #[must_use]
    pub fn to_vis_graph(&self) -> crate::viz::terminal::VisGraph {
        let mut vis = crate::viz::terminal::VisGraph::new();

        // Build index mapping (function name → vis node index)
        let mut name_to_idx: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        // Add all nodes with their criticality scores
        for (name, &_node_id) in &self.node_map {
            let criticality = self.criticality_scores.get(name).copied().unwrap_or(0.0);
            let idx = vis.nodes.len();
            name_to_idx.insert(name.clone(), idx);
            vis.add_node(name.clone(), criticality);
        }

        // Add edges by iterating over adjacency
        for (_node_id, neighbors, _weights) in self.graph.iter_adjacency() {
            let from_name = self.reverse_node_map.get(&_node_id);
            if let Some(from) = from_name {
                if let Some(&from_idx) = name_to_idx.get(from) {
                    for &neighbor_id in neighbors {
                        let to_node_id = NodeId(neighbor_id);
                        if let Some(to_name) = self.reverse_node_map.get(&to_node_id) {
                            if let Some(&to_idx) = name_to_idx.get(to_name) {
                                vis.add_edge(from_idx, to_idx);
                            }
                        }
                    }
                }
            }
        }

        vis
    }
}

#[cfg(feature = "viz")]
impl crate::viz::terminal::Visualizable for TdgGraph {
    fn render_terminal(&self, config: &crate::viz::terminal::RenderConfig) -> Result<String> {
        let vis = self.to_vis_graph();
        vis.render_terminal(config)
    }

    fn node_count(&self) -> usize {
        self.num_nodes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tdg_graph_creation() {
        let graph = TdgGraph::new();
        assert_eq!(graph.num_nodes(), 0);
        assert_eq!(graph.num_edges(), 0);
    }

    #[test]
    fn test_add_function_o1_lookup() {
        let mut graph = TdgGraph::new();

        graph.add_function("test_func".to_string()).unwrap();

        // O(1) lookup
        assert!(graph.has_function("test_func"));
        assert!(!graph.has_function("nonexistent"));
        assert_eq!(graph.num_nodes(), 1);
    }

    #[test]
    fn test_add_duplicate_fails() {
        let mut graph = TdgGraph::new();

        graph.add_function("dup".to_string()).unwrap();

        // Duplicate should fail
        let result = graph.add_function("dup".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }

    #[test]
    fn test_add_edge_dependencies() {
        let mut graph = TdgGraph::new();

        // Add two functions
        graph.add_function("main".to_string()).unwrap();
        graph.add_function("helper".to_string()).unwrap();

        // Add edge: main calls helper
        graph.add_edge("main", "helper").unwrap();

        assert_eq!(graph.num_nodes(), 2);
        assert_eq!(graph.num_edges(), 1);
    }

    #[test]
    fn test_pagerank_criticality() {
        let mut graph = TdgGraph::new();

        // Create a simple call graph:
        // main → helper1
        // main → helper2
        // helper1 → helper2
        // (helper2 should have highest PageRank = most critical)

        graph.add_function("main".to_string()).unwrap();
        graph.add_function("helper1".to_string()).unwrap();
        graph.add_function("helper2".to_string()).unwrap();

        graph.add_edge("main", "helper1").unwrap();
        graph.add_edge("main", "helper2").unwrap();
        graph.add_edge("helper1", "helper2").unwrap();

        // Update PageRank
        graph.update_criticality().unwrap();

        // Get critical functions
        let critical = graph.critical_functions();
        assert_eq!(critical.len(), 3);

        // helper2 should be most critical (highest in-degree)
        assert_eq!(critical[0].0, "helper2");
        assert!(critical[0].1 > critical[1].1); // helper2 score > helper1 score
        assert!(critical[0].1 > critical[2].1); // helper2 score > main score
    }

    #[test]
    fn test_critical_functions_ranking() {
        let mut graph = TdgGraph::new();

        // Add 5 functions with varying in-degrees
        for i in 0..5 {
            graph.add_function(format!("func{}", i)).unwrap();
        }

        // func4 called by everyone (most critical)
        for i in 0..4 {
            graph.add_edge(&format!("func{}", i), "func4").unwrap();
        }

        graph.update_criticality().unwrap();
        let critical = graph.critical_functions();

        // func4 should be #1
        assert_eq!(critical[0].0, "func4");
    }

    #[test]
    fn test_empty_graph_pagerank() {
        let mut graph = TdgGraph::new();
        // Empty graph should not panic
        graph.update_criticality().unwrap();
        assert_eq!(graph.critical_functions().len(), 0);
    }

    // ====================================================================
    // Visualization tests (feature = "viz")
    // ====================================================================

    #[cfg(feature = "viz")]
    mod viz_tests {
        use super::*;
        use crate::viz::terminal::{RenderConfig, Visualizable};

        #[test]
        fn test_tdg_graph_to_vis_graph() {
            let mut graph = TdgGraph::new();

            graph.add_function("main".to_string()).unwrap();
            graph.add_function("helper".to_string()).unwrap();
            graph.add_edge("main", "helper").unwrap();
            graph.update_criticality().unwrap();

            let vis = graph.to_vis_graph();

            assert_eq!(vis.node_count(), 2);
        }

        #[test]
        fn test_tdg_graph_render_terminal() {
            let mut graph = TdgGraph::new();

            graph.add_function("main".to_string()).unwrap();
            graph.add_function("process".to_string()).unwrap();
            graph.add_function("save".to_string()).unwrap();
            graph.add_edge("main", "process").unwrap();
            graph.add_edge("process", "save").unwrap();
            graph.update_criticality().unwrap();

            let config = RenderConfig::default();
            let result = graph.render_terminal(&config);

            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(!output.is_empty());
        }

        #[test]
        fn test_tdg_graph_node_count() {
            let mut graph = TdgGraph::new();

            for i in 0..10 {
                graph.add_function(format!("func_{}", i)).unwrap();
            }

            assert_eq!(graph.node_count(), 10);
        }

        #[test]
        fn test_tdg_graph_vis_with_criticality() {
            let mut graph = TdgGraph::new();

            // Create a hub-and-spoke pattern
            // center is called by all others
            graph.add_function("center".to_string()).unwrap();
            for i in 0..5 {
                let name = format!("spoke_{}", i);
                graph.add_function(name.clone()).unwrap();
                graph.add_edge(&name, "center").unwrap();
            }

            graph.update_criticality().unwrap();
            let vis = graph.to_vis_graph();

            // Find center's criticality
            let center_idx = vis.nodes.iter().position(|n| n == "center").unwrap();
            let center_criticality = vis.criticality[center_idx];

            // Center should have highest criticality
            for (i, &crit) in vis.criticality.iter().enumerate() {
                if i != center_idx {
                    assert!(
                        center_criticality >= crit,
                        "Center should have highest criticality"
                    );
                }
            }
        }
    }
}
