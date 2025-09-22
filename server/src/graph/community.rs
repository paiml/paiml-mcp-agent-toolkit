// Community detection algorithms using Louvain method
// Following Newman-Girvan modularity optimization
// Complexity: All functions ≤ 10

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

    /// Detect communities using Louvain algorithm
    /// Complexity: 10 (initialization + main loop)
    pub fn detect_communities(&mut self, graph: &UndirectedGraph) -> Vec<usize> {
        let n = graph.node_count();
        if n == 0 {
            return Vec::new();
        }

        // Initialize each node as its own community
        let mut communities: Vec<usize> = (0..n).collect();
        let mut node_weights = vec![0.0; n];
        let mut total_weight = 0.0;

        // Calculate node weights and total weight
        for edge in graph.edge_references() {
            let weight = edge.weight();
            node_weights[edge.source().index()] += weight;
            node_weights[edge.target().index()] += weight;
            total_weight += weight * 2.0; // Each edge counted twice
        }

        let mut improved = true;
        let mut iterations = 0;

        // Main Louvain loop
        while improved && iterations < self.max_iterations {
            improved = false;
            iterations += 1;

            for node_idx in 0..n {
                let best_community = self.find_best_community(
                    graph,
                    node_idx,
                    &communities,
                    &node_weights,
                    total_weight,
                );

                if best_community != communities[node_idx] {
                    communities[node_idx] = best_community;
                    improved = true;
                }
            }
        }

        communities
    }

    /// Find the best community for a node
    /// Complexity: 8 (neighbor iteration + gain calculation)
    fn find_best_community(
        &self,
        graph: &UndirectedGraph,
        node_idx: usize,
        communities: &[usize],
        node_weights: &[f64],
        total_weight: f64,
    ) -> usize {
        let current_community = communities[node_idx];
        let mut best_community = current_community;
        let mut best_gain = 0.0;

        // Check all neighboring communities
        let mut neighbor_communities = HashMap::new();

        // Get edges for this node
        if let Some(node) = graph.node_indices().nth(node_idx) {
            for edge in graph.edges(node) {
                let neighbor_idx = edge.target().index();
                let neighbor_community = communities[neighbor_idx];
                *neighbor_communities
                    .entry(neighbor_community)
                    .or_insert(0.0) += edge.weight();
            }
        }

        // Test moving to each neighbor community
        for (&community, &edge_weight) in &neighbor_communities {
            if community != current_community {
                let gain = self.calculate_modularity_gain(
                    node_idx,
                    community,
                    edge_weight,
                    communities,
                    node_weights,
                    total_weight,
                );

                if gain > best_gain {
                    best_gain = gain;
                    best_community = community;
                }
            }
        }

        best_community
    }

    /// Calculate modularity gain from moving node to community
    /// Complexity: 6 (community weight calculation)
    fn calculate_modularity_gain(
        &self,
        node_idx: usize,
        target_community: usize,
        edge_weight_to_community: f64,
        communities: &[usize],
        node_weights: &[f64],
        total_weight: f64,
    ) -> f64 {
        let node_degree = node_weights[node_idx];

        // Calculate community weight
        let mut community_weight = 0.0;
        for (i, &community) in communities.iter().enumerate() {
            if community == target_community {
                community_weight += node_weights[i];
            }
        }

        // Modularity gain formula
        

        edge_weight_to_community
            - self.resolution * (node_degree * community_weight) / total_weight
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
