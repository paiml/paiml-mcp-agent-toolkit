// Community detection algorithms using Louvain method (aprender v0.5.0)
// Following Newman-Girvan modularity optimization
// Implements Task 4.3 (Phase 4: Graph Migration)

use super::aprender_adapter::to_aprender_graph_undirected;
use super::*;
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
            let source_idx = edge.source().0 as usize;
            let target_idx = edge.target().0 as usize;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node_data() -> NodeData {
        NodeData {
            path: std::path::PathBuf::from("test.rs"),
            module: "test".to_string(),
            symbols: vec![],
            loc: 10,
            complexity: 1.0,
            ast_hash: 1,
        }
    }

    #[test]
    fn test_louvain_detector_default() {
        let detector = LouvainDetector::default();
        assert_eq!(detector.resolution, 1.0);
        assert_eq!(detector.max_iterations, 100);
    }

    #[test]
    fn test_louvain_detector_new() {
        let detector = LouvainDetector::new();
        assert_eq!(detector.resolution, 1.0);
    }

    #[test]
    fn test_louvain_detector_with_resolution() {
        let detector = LouvainDetector::new().with_resolution(0.5);
        assert_eq!(detector.resolution, 0.5);
    }

    #[test]
    fn test_detect_communities_empty_graph() {
        let mut detector = LouvainDetector::new();
        let graph = UndirectedGraph::default();
        let communities = detector.detect_communities(&graph);
        assert!(communities.is_empty());
    }

    #[test]
    fn test_detect_communities_single_node() {
        let mut detector = LouvainDetector::new();
        let mut graph = UndirectedGraph::default();
        graph.add_node(make_node_data());
        let communities = detector.detect_communities(&graph);
        assert_eq!(communities.len(), 1);
    }

    #[test]
    fn test_detect_communities_two_disconnected_nodes() {
        let mut detector = LouvainDetector::new();
        let mut graph = UndirectedGraph::default();
        graph.add_node(make_node_data());
        graph.add_node(make_node_data());
        let communities = detector.detect_communities(&graph);
        assert_eq!(communities.len(), 2);
    }

    #[test]
    fn test_detect_communities_two_connected_nodes() {
        let mut detector = LouvainDetector::new();
        let mut graph = UndirectedGraph::default();
        let n1 = graph.add_node(make_node_data());
        let n2 = graph.add_node(make_node_data());
        graph.add_edge(n1, n2, 1.0);
        let communities = detector.detect_communities(&graph);
        assert_eq!(communities.len(), 2);
        // Connected nodes should be in the same community
        assert_eq!(communities[0], communities[1]);
    }

    #[test]
    fn test_calculate_modularity_empty_graph() {
        let detector = LouvainDetector::new();
        let graph = UndirectedGraph::default();
        let communities: Vec<usize> = vec![];
        let modularity = detector.calculate_modularity(&graph, &communities);
        assert_eq!(modularity, 0.0);
    }

    #[test]
    fn test_calculate_modularity_single_community() {
        let detector = LouvainDetector::new();
        let mut graph = UndirectedGraph::default();
        let n1 = graph.add_node(make_node_data());
        let n2 = graph.add_node(make_node_data());
        graph.add_edge(n1, n2, 1.0);

        // All nodes in same community
        let communities = vec![0, 0];
        let modularity = detector.calculate_modularity(&graph, &communities);
        // With single community, modularity should be 0 or slightly positive
        assert!(modularity >= -1.0 && modularity <= 1.0);
    }

    #[test]
    fn test_calculate_modularity_two_communities() {
        let detector = LouvainDetector::new();
        let mut graph = UndirectedGraph::default();
        let n1 = graph.add_node(make_node_data());
        let n2 = graph.add_node(make_node_data());
        let n3 = graph.add_node(make_node_data());
        let n4 = graph.add_node(make_node_data());

        // Two connected pairs
        graph.add_edge(n1, n2, 1.0);
        graph.add_edge(n3, n4, 1.0);

        // Two separate communities
        let communities = vec![0, 0, 1, 1];
        let modularity = detector.calculate_modularity(&graph, &communities);
        // With two perfect communities, modularity should be positive
        assert!(modularity > 0.0);
    }

    #[test]
    fn test_calculate_modularity_no_edges() {
        let detector = LouvainDetector::new();
        let mut graph = UndirectedGraph::default();
        graph.add_node(make_node_data());
        graph.add_node(make_node_data());

        let communities = vec![0, 1];
        let modularity = detector.calculate_modularity(&graph, &communities);
        assert_eq!(modularity, 0.0);
    }

    #[test]
    fn test_louvain_detector_clone() {
        let detector = LouvainDetector::new().with_resolution(0.75);
        let cloned = detector.clone();
        assert_eq!(cloned.resolution, 0.75);
        assert_eq!(cloned.max_iterations, 100);
    }

    #[test]
    fn test_louvain_detector_debug() {
        let detector = LouvainDetector::new();
        let debug_str = format!("{:?}", detector);
        assert!(debug_str.contains("LouvainDetector"));
        assert!(debug_str.contains("resolution"));
    }
}
