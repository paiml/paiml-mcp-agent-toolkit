// Centrality metrics computation using aprender v0.5.0
// Implements 6 centrality algorithms (Phase 4: Graph Migration)

use super::aprender_adapter::to_aprender_graph;
use super::*;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct CentralityMetrics {
    pub degree: Vec<f64>,
    pub betweenness: Vec<f64>,
    pub closeness: Vec<f64>,
    pub eigenvector: Vec<f64>,
    pub katz: Vec<f64>,
    pub harmonic: Vec<f64>,
}

pub struct CentralityComputer {
    pub normalize: bool,
    pub weighted: bool,
}

impl CentralityComputer {
    pub fn new(normalize: bool, weighted: bool) -> Self {
        CentralityComputer {
            normalize,
            weighted,
        }
    }

    /// Compute all centrality metrics using aprender v0.5.0.
    ///
    /// # Arguments
    /// * `graph` - PMAT dependency graph
    ///
    /// # Returns
    /// CentralityMetrics with 6 centrality algorithms
    ///
    /// # Performance
    /// - Degree: O(n + m)
    /// - Betweenness: O(nm) using Brandes' algorithm
    /// - Closeness: O(n(n+m)) using BFS
    /// - Eigenvector: O(k·m) using power iteration
    /// - Katz: O(k·m) using power iteration
    /// - Harmonic: O(n(n+m)) using BFS
    pub fn compute_all(&self, graph: &DependencyGraph) -> CentralityMetrics {
        // Convert to aprender graph (directed by default for dependency graphs)
        let aprender_graph = to_aprender_graph(graph, true);

        // Compute degree centrality
        let degree_map = aprender_graph.degree_centrality();
        let degree = map_to_vec(&degree_map, aprender_graph.num_nodes());

        // Compute betweenness centrality (using parallel Brandes' algorithm)
        let betweenness = aprender_graph.betweenness_centrality();

        // Compute closeness centrality
        let closeness = aprender_graph.closeness_centrality();

        // Compute eigenvector centrality
        let eigenvector = aprender_graph
            .eigenvector_centrality(100, 1e-6)
            .unwrap_or_else(|_| vec![0.0; aprender_graph.num_nodes()]);

        // Compute Katz centrality (alpha = 0.1 is safe for most graphs)
        let katz = aprender_graph
            .katz_centrality(0.1, 100, 1e-6)
            .unwrap_or_else(|_| vec![0.0; aprender_graph.num_nodes()]);

        // Compute harmonic centrality (more robust for disconnected graphs)
        let harmonic = aprender_graph.harmonic_centrality();

        CentralityMetrics {
            degree,
            betweenness,
            closeness,
            eigenvector,
            katz,
            harmonic,
        }
    }
}

/// Convert HashMap<NodeId, f64> to Vec<f64> for consistent ordering.
fn map_to_vec(map: &HashMap<usize, f64>, size: usize) -> Vec<f64> {
    let mut vec = vec![0.0; size];
    for (&node_id, &value) in map {
        if node_id < size {
            vec[node_id] = value;
        }
    }
    vec
}
