// Tests for ParallelLouvain community detection (builder, detection, modularity)
// Included by parallel_louvain.rs - shares parent scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::types::{NodeData, Symbol, SymbolKind, Visibility};
    use std::path::PathBuf;

    /// Create a test node for graph construction.
    pub(super) fn create_test_node(name: &str) -> NodeData {
        NodeData {
            path: PathBuf::from(format!("{}.rs", name)),
            module: name.to_string(),
            symbols: vec![Symbol {
                name: name.to_string(),
                kind: SymbolKind::Function,
                visibility: Visibility::Public,
                line: 1,
            }],
            loc: 100,
            complexity: 5.0,
            ast_hash: 12345,
        }
    }

    /// Create a simple graph with two clear communities.
    /// Community 1: nodes 0, 1, 2 (fully connected)
    /// Community 2: nodes 3, 4, 5 (fully connected)
    /// Single bridge edge between communities
    fn create_two_community_graph() -> UndirectedGraph {
        let mut graph = UndirectedGraph::new();

        // Add 6 nodes
        let nodes: Vec<_> = (0..6)
            .map(|i| graph.add_node(create_test_node(&format!("node{}", i))))
            .collect();

        // Community 1: fully connected (0, 1, 2)
        graph.add_edge(nodes[0], nodes[1], 1.0);
        graph.add_edge(nodes[1], nodes[2], 1.0);
        graph.add_edge(nodes[0], nodes[2], 1.0);

        // Community 2: fully connected (3, 4, 5)
        graph.add_edge(nodes[3], nodes[4], 1.0);
        graph.add_edge(nodes[4], nodes[5], 1.0);
        graph.add_edge(nodes[3], nodes[5], 1.0);

        // Bridge edge (weak connection)
        graph.add_edge(nodes[2], nodes[3], 0.1);

        graph
    }

    /// Create a simple chain graph.
    fn create_chain_graph(n: usize) -> UndirectedGraph {
        let mut graph = UndirectedGraph::new();
        let nodes: Vec<_> = (0..n)
            .map(|i| graph.add_node(create_test_node(&format!("node{}", i))))
            .collect();

        for i in 0..n - 1 {
            graph.add_edge(nodes[i], nodes[i + 1], 1.0);
        }

        graph
    }

    /// Create a complete graph (all nodes connected to all others).
    fn create_complete_graph(n: usize) -> UndirectedGraph {
        let mut graph = UndirectedGraph::new();
        let nodes: Vec<_> = (0..n)
            .map(|i| graph.add_node(create_test_node(&format!("node{}", i))))
            .collect();

        for i in 0..n {
            for j in i + 1..n {
                graph.add_edge(nodes[i], nodes[j], 1.0);
            }
        }

        graph
    }

    /// Create a star graph with one central node.
    fn create_star_graph(n: usize) -> UndirectedGraph {
        let mut graph = UndirectedGraph::new();
        let nodes: Vec<_> = (0..n)
            .map(|i| graph.add_node(create_test_node(&format!("node{}", i))))
            .collect();

        // Node 0 is the center
        for i in 1..n {
            graph.add_edge(nodes[0], nodes[i], 1.0);
        }

        graph
    }

    // ============ Default and Builder Tests ============

    #[test]
    fn test_default_parameters() {
        let louvain = ParallelLouvain::default();
        assert_eq!(louvain.resolution, 1.0);
        assert_eq!(louvain.max_iterations, 100);
        assert!((louvain.min_improvement - 1e-6).abs() < 1e-10);
        assert_eq!(louvain.num_threads, 0);
    }

    #[test]
    fn test_new_equals_default() {
        let louvain1 = ParallelLouvain::new();
        let louvain2 = ParallelLouvain::default();
        assert_eq!(louvain1.resolution, louvain2.resolution);
        assert_eq!(louvain1.max_iterations, louvain2.max_iterations);
    }

    #[test]
    fn test_builder_with_resolution() {
        let louvain = ParallelLouvain::new().with_resolution(0.5);
        assert_eq!(louvain.resolution, 0.5);
    }

    #[test]
    fn test_builder_with_max_iterations() {
        let louvain = ParallelLouvain::new().with_max_iterations(50);
        assert_eq!(louvain.max_iterations, 50);
    }

    #[test]
    fn test_builder_with_min_improvement() {
        let louvain = ParallelLouvain::new().with_min_improvement(1e-8);
        assert!((louvain.min_improvement - 1e-8).abs() < 1e-12);
    }

    #[test]
    fn test_builder_with_num_threads() {
        let louvain = ParallelLouvain::new().with_num_threads(4);
        assert_eq!(louvain.num_threads, 4);
    }

    #[test]
    fn test_builder_chaining() {
        let louvain = ParallelLouvain::new()
            .with_resolution(0.8)
            .with_max_iterations(200)
            .with_min_improvement(1e-4)
            .with_num_threads(8);

        assert_eq!(louvain.resolution, 0.8);
        assert_eq!(louvain.max_iterations, 200);
        assert!((louvain.min_improvement - 1e-4).abs() < 1e-10);
        assert_eq!(louvain.num_threads, 8);
    }

    // ============ Empty and Single Node Tests ============

    #[test]
    fn test_detect_empty_graph() {
        let graph = UndirectedGraph::new();
        let louvain = ParallelLouvain::new();
        let communities = louvain.detect(&graph);
        assert!(communities.is_empty());
    }

    #[test]
    fn test_detect_single_node() {
        let mut graph = UndirectedGraph::new();
        graph.add_node(create_test_node("single"));

        let louvain = ParallelLouvain::new();
        let communities = louvain.detect(&graph);

        assert_eq!(communities.len(), 1);
        assert_eq!(communities[0], 0);
    }

    #[test]
    fn test_detect_two_disconnected_nodes() {
        let mut graph = UndirectedGraph::new();
        graph.add_node(create_test_node("node0"));
        graph.add_node(create_test_node("node1"));
        // No edges

        let louvain = ParallelLouvain::new();
        let communities = louvain.detect(&graph);

        assert_eq!(communities.len(), 2);
        // Disconnected nodes may end up in different communities
    }

    #[test]
    fn test_detect_two_connected_nodes() {
        let mut graph = UndirectedGraph::new();
        let n0 = graph.add_node(create_test_node("node0"));
        let n1 = graph.add_node(create_test_node("node1"));
        graph.add_edge(n0, n1, 1.0);

        let louvain = ParallelLouvain::new();
        let communities = louvain.detect(&graph);

        assert_eq!(communities.len(), 2);
        // Strongly connected nodes should be in the same community
        assert_eq!(communities[0], communities[1]);
    }

    // ============ Community Detection Quality Tests ============

    #[test]
    fn test_detect_complete_graph() {
        let graph = create_complete_graph(5);
        let louvain = ParallelLouvain::new();
        let communities = louvain.detect(&graph);

        assert_eq!(communities.len(), 5);

        // Complete graph should ideally be one community
        let num_communities = ParallelLouvain::num_communities(&communities);
        assert!(
            num_communities <= 2,
            "Complete graph should have few communities"
        );
    }

    #[test]
    fn test_detect_chain_graph() {
        let graph = create_chain_graph(6);
        let louvain = ParallelLouvain::new();
        let communities = louvain.detect(&graph);

        assert_eq!(communities.len(), 6);
        // Chain graph may split into multiple communities or be one
    }

    #[test]
    fn test_detect_star_graph() {
        let graph = create_star_graph(5);
        let louvain = ParallelLouvain::new();
        let communities = louvain.detect(&graph);

        assert_eq!(communities.len(), 5);
        // Star graph should typically be one community
    }

    // ============ Resolution Parameter Tests ============

    #[test]
    fn test_high_resolution_more_communities() {
        let graph = create_two_community_graph();

        let low_res = ParallelLouvain::new().with_resolution(0.5);
        let high_res = ParallelLouvain::new().with_resolution(2.0);

        let communities_low = low_res.detect(&graph);
        let communities_high = high_res.detect(&graph);

        let num_low = ParallelLouvain::num_communities(&communities_low);
        let num_high = ParallelLouvain::num_communities(&communities_high);

        // Higher resolution typically leads to more (smaller) communities
        assert!(
            num_high >= num_low,
            "High resolution should not reduce communities"
        );
    }

    #[test]
    fn test_zero_resolution() {
        let graph = create_two_community_graph();
        let louvain = ParallelLouvain::new().with_resolution(0.0);
        let communities = louvain.detect(&graph);

        // Zero resolution should put everything in one community
        let num_communities = ParallelLouvain::num_communities(&communities);
        assert_eq!(num_communities, 1);
    }

    // ============ Modularity Calculation Tests ============

    #[test]
    fn test_modularity_empty_graph() {
        let graph = UndirectedGraph::new();
        let louvain = ParallelLouvain::new();
        let communities: Vec<usize> = vec![];
        let modularity = louvain.calculate_modularity(&graph, &communities);
        assert_eq!(modularity, 0.0);
    }

    #[test]
    fn test_modularity_two_communities() {
        let graph = create_two_community_graph();
        let louvain = ParallelLouvain::new();

        // Perfect community assignment
        let perfect = vec![0, 0, 0, 1, 1, 1];
        let modularity_perfect = louvain.calculate_modularity(&graph, &perfect);

        // Random/bad assignment
        let bad = vec![0, 1, 0, 1, 0, 1];
        let modularity_bad = louvain.calculate_modularity(&graph, &bad);

        // Perfect assignment should have higher modularity
        assert!(
            modularity_perfect > modularity_bad,
            "Perfect assignment should have higher modularity: {} vs {}",
            modularity_perfect,
            modularity_bad
        );
    }

    #[test]
    fn test_modularity_is_bounded() {
        let graph = create_two_community_graph();
        let louvain = ParallelLouvain::new();
        let communities = louvain.detect(&graph);
        let modularity = louvain.calculate_modularity(&graph, &communities);

        // Modularity should be in range [-0.5, 1]
        assert!(
            modularity >= -0.5 && modularity <= 1.0,
            "Modularity {} should be in [-0.5, 1]",
            modularity
        );
    }
}
