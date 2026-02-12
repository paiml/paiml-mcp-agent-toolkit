#![cfg_attr(coverage_nightly, coverage(off))]
// Centrality metrics computation using aprender v0.5.0
// Implements 6 centrality algorithms (Phase 4: Graph Migration)

use super::aprender_adapter::to_aprender_graph;
use super::*;
use aprender::graph::GraphCentrality;
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // CentralityMetrics Tests
    // =========================================================================

    #[test]
    fn test_centrality_metrics_default_fields() {
        let metrics = CentralityMetrics {
            degree: vec![],
            betweenness: vec![],
            closeness: vec![],
            eigenvector: vec![],
            katz: vec![],
            harmonic: vec![],
        };

        assert!(metrics.degree.is_empty());
        assert!(metrics.betweenness.is_empty());
        assert!(metrics.closeness.is_empty());
        assert!(metrics.eigenvector.is_empty());
        assert!(metrics.katz.is_empty());
        assert!(metrics.harmonic.is_empty());
    }

    #[test]
    fn test_centrality_metrics_with_values() {
        let metrics = CentralityMetrics {
            degree: vec![1.0, 2.0, 3.0],
            betweenness: vec![0.1, 0.2, 0.3],
            closeness: vec![0.5, 0.6, 0.7],
            eigenvector: vec![0.9, 0.8, 0.7],
            katz: vec![0.3, 0.4, 0.5],
            harmonic: vec![0.2, 0.3, 0.4],
        };

        assert_eq!(metrics.degree.len(), 3);
        assert_eq!(metrics.betweenness.len(), 3);
    }

    #[test]
    fn test_centrality_metrics_clone() {
        let metrics = CentralityMetrics {
            degree: vec![1.0],
            betweenness: vec![0.1],
            closeness: vec![0.5],
            eigenvector: vec![0.9],
            katz: vec![0.3],
            harmonic: vec![0.2],
        };

        let cloned = metrics.clone();
        assert_eq!(cloned.degree, metrics.degree);
    }

    #[test]
    fn test_centrality_metrics_debug() {
        let metrics = CentralityMetrics {
            degree: vec![1.0],
            betweenness: vec![],
            closeness: vec![],
            eigenvector: vec![],
            katz: vec![],
            harmonic: vec![],
        };

        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("CentralityMetrics"));
    }

    // =========================================================================
    // CentralityComputer Tests
    // =========================================================================

    #[test]
    fn test_centrality_computer_new() {
        let computer = CentralityComputer::new(true, false);
        assert!(computer.normalize);
        assert!(!computer.weighted);
    }

    #[test]
    fn test_centrality_computer_new_weighted() {
        let computer = CentralityComputer::new(false, true);
        assert!(!computer.normalize);
        assert!(computer.weighted);
    }

    #[test]
    fn test_centrality_computer_both_flags() {
        let computer = CentralityComputer::new(true, true);
        assert!(computer.normalize);
        assert!(computer.weighted);
    }

    #[test]
    fn test_centrality_computer_no_flags() {
        let computer = CentralityComputer::new(false, false);
        assert!(!computer.normalize);
        assert!(!computer.weighted);
    }

    // =========================================================================
    // map_to_vec Tests
    // =========================================================================

    #[test]
    fn test_map_to_vec_empty() {
        let map: HashMap<usize, f64> = HashMap::new();
        let vec = map_to_vec(&map, 5);
        assert_eq!(vec, vec![0.0; 5]);
    }

    #[test]
    fn test_map_to_vec_partial() {
        let mut map = HashMap::new();
        map.insert(0, 1.0);
        map.insert(2, 2.0);

        let vec = map_to_vec(&map, 4);
        assert_eq!(vec, vec![1.0, 0.0, 2.0, 0.0]);
    }

    #[test]
    fn test_map_to_vec_full() {
        let mut map = HashMap::new();
        map.insert(0, 1.0);
        map.insert(1, 2.0);
        map.insert(2, 3.0);

        let vec = map_to_vec(&map, 3);
        assert_eq!(vec, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_map_to_vec_out_of_bounds() {
        let mut map = HashMap::new();
        map.insert(5, 1.0); // Out of bounds for size 3

        let vec = map_to_vec(&map, 3);
        assert_eq!(vec, vec![0.0; 3]); // Should be all zeros
    }

    #[test]
    fn test_map_to_vec_zero_size() {
        let map: HashMap<usize, f64> = HashMap::new();
        let vec = map_to_vec(&map, 0);
        assert!(vec.is_empty());
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    #[test]
    fn test_compute_all_empty_graph() {
        let computer = CentralityComputer::new(true, false);
        let graph = DependencyGraph::new();

        let metrics = computer.compute_all(&graph);

        assert!(metrics.degree.is_empty());
        assert!(metrics.betweenness.is_empty());
    }
}
