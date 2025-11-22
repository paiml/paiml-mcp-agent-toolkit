// Community detection algorithms using Louvain method (aprender v0.5.0)
// Following Newman-Girvan modularity optimization
// Implements Task 4.3 (Phase 4: Graph Migration)

use super::aprender_adapter::to_aprender_graph_undirected;
use super::*;
use petgraph::visit::EdgeRef;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LouvainDetector {
    pub resolution: f64,
    pub max_iterations: usize,
}

impl Default for LouvainDetector {
    fn default() -> Self {
        LouvainDetector {
            resolution: 1.0,
            max_iterations: 100,
        }
    }
}

impl LouvainDetector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_resolution(mut self, resolution: f64) -> Self {
        self.resolution = resolution;
        self
    }

    /// Detect communities using aprender's Louvain algorithm.
    ///
    /// # Arguments
    /// * `graph` - Undirected PMAT graph
    ///
    /// # Returns
    /// Community assignment vector (node_id -> community_id)
    ///
    /// # Algorithm
    /// - aprender Louvain with Newman-Girvan modularity
    /// - Output converted from Vec<Vec<NodeId>> to Vec<usize>
    pub fn detect_communities(&mut self, graph: &UndirectedGraph) -> Vec<usize> {
        let n = graph.node_count();
        if n == 0 {
            return Vec::new();
        }

        // Convert to aprender graph (undirected for Louvain)
        let aprender_graph = to_aprender_graph_undirected(graph);

        // Run aprender Louvain
        let communities_vec = aprender_graph.louvain();

        // Convert from Vec<Vec<NodeId>> to Vec<usize>
        // communities_vec[i] = list of node IDs in community i
        // We need: assignments[node_id] = community_id
        let mut assignments = vec![0; n];
        for (community_id, community_nodes) in communities_vec.iter().enumerate() {
            for &node_id in community_nodes {
                if node_id < n {
                    assignments[node_id] = community_id;
                }
            }
        }

        assignments
    }

    /// Calculate modularity of community assignment
    /// Complexity: 9 (edge iteration + community mapping)
    pub fn calculate_modularity(&self, graph: &UndirectedGraph, communities: &[usize]) -> f64 {
        if graph.node_count() == 0 {
            return 0.0;
        }

        let mut total_weight = 0.0;
        let mut community_internal_weight = 0.0;
        let mut node_weights = vec![0.0; graph.node_count()];

        // Calculate total weight and node degrees
        for edge in graph.edge_references() {
            let weight = edge.weight();
            let source_idx = edge.source().index();
            let target_idx = edge.target().index();

            node_weights[source_idx] += weight;
            node_weights[target_idx] += weight;
            total_weight += weight * 2.0;

            // Check if edge is within same community
            if communities[source_idx] == communities[target_idx] {
                community_internal_weight += weight * 2.0;
            }
        }

        if total_weight == 0.0 {
            return 0.0;
        }

        // Calculate expected internal weight
        let mut expected_internal = 0.0;
        let mut community_degrees: HashMap<usize, f64> = HashMap::new();

        for (node_idx, &community) in communities.iter().enumerate() {
            *community_degrees.entry(community).or_insert(0.0) += node_weights[node_idx];
        }

        for degree in community_degrees.values() {
            expected_internal += degree * degree / total_weight;
        }

        // Modularity = (actual - expected) / total
        (community_internal_weight - expected_internal) / total_weight
    }
}
