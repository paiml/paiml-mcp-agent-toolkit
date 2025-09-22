// PageRank algorithm tests - TDD approach
// All tests written FIRST before implementation

#[cfg(test)]
mod tests {
    use super::super::super::*;
    use petgraph::graph::DiGraph;

    /// Create a simple 3-node cyclic graph for testing
    fn create_cycle_graph() -> DependencyGraph {
        let mut graph = DiGraph::new();

        let n0 = graph.add_node(NodeData::test_node(0));
        let n1 = graph.add_node(NodeData::test_node(1));
        let n2 = graph.add_node(NodeData::test_node(2));

        // Create cycle: 0 -> 1 -> 2 -> 0
        graph.add_edge(n0, n1, EdgeData::test_edge(1.0));
        graph.add_edge(n1, n2, EdgeData::test_edge(1.0));
        graph.add_edge(n2, n0, EdgeData::test_edge(1.0));

        graph
    }

    /// Create star graph (hub and spokes)
    fn create_star_graph() -> DependencyGraph {
        let mut graph = DiGraph::new();

        let hub = graph.add_node(NodeData::test_node(0));

        // Create 4 spokes
        for i in 1..5 {
            let spoke = graph.add_node(NodeData::test_node(i));
            // Bidirectional edges
            graph.add_edge(hub, spoke, EdgeData::test_edge(1.0));
            graph.add_edge(spoke, hub, EdgeData::test_edge(1.0));
        }

        graph
    }

    #[test]
    fn test_pagerank_sum_preservation() {
        let graph = create_cycle_graph();
        let matrices = GraphMatrices::from(&graph);
        let pr = PageRankComputer::default();
        let ranks = pr.compute(&matrices);

        // PageRank values must sum to 1.0
        let sum: f64 = ranks.iter().sum();
        assert!(
            (sum - 1.0).abs() < 1e-6,
            "PageRank sum = {}, expected 1.0",
            sum
        );
    }

    #[test]
    fn test_pagerank_convergence() {
        let graph = create_star_graph();
        let matrices = GraphMatrices::from(&graph);

        let mut pr = PageRankComputer::default();
        pr.tolerance = 1e-8; // Strict tolerance
        pr.max_iterations = 1000;

        let ranks = pr.compute(&matrices);

        // Should converge before max iterations
        // Check by running one more iteration
        let ranks2 = pr.compute(&matrices);

        for i in 0..ranks.len() {
            assert!(
                (ranks[i] - ranks2[i]).abs() < 1e-7,
                "PageRank not converged at node {}",
                i
            );
        }
    }

    #[test]
    fn test_pagerank_star_graph() {
        let graph = create_star_graph();
        let matrices = GraphMatrices::from(&graph);
        let pr = PageRankComputer::default();
        let ranks = pr.compute(&matrices);

        // Hub (node 0) should have highest PageRank
        for i in 1..ranks.len() {
            assert!(
                ranks[0] > ranks[i],
                "Hub PageRank {} should be > spoke PageRank {}",
                ranks[0],
                ranks[i]
            );
        }

        // All spokes should have equal PageRank
        for i in 2..ranks.len() {
            assert!(
                (ranks[1] - ranks[i]).abs() < 1e-9,
                "Spoke PageRanks should be equal: {} vs {}",
                ranks[1],
                ranks[i]
            );
        }
    }

    #[test]
    fn test_pagerank_complete_graph() {
        // In a complete graph, all nodes should have equal PageRank
        let mut graph = DiGraph::new();

        // Create 4 nodes
        let nodes: Vec<_> = (0..4)
            .map(|i| graph.add_node(NodeData::test_node(i)))
            .collect();

        // Connect every node to every other node
        for i in 0..nodes.len() {
            for j in 0..nodes.len() {
                if i != j {
                    graph.add_edge(nodes[i], nodes[j], EdgeData::test_edge(1.0));
                }
            }
        }

        let matrices = GraphMatrices::from(&graph);
        let pr = PageRankComputer::default();
        let ranks = pr.compute(&matrices);

        // All should be equal (1/n)
        let expected = 1.0 / ranks.len() as f64;
        for rank in ranks {
            assert!(
                (rank - expected).abs() < 1e-6,
                "Expected {}, got {}",
                expected,
                rank
            );
        }
    }

    #[test]
    fn test_pagerank_dangling_nodes() {
        // Test handling of dangling nodes (no outgoing edges)
        let mut graph = DiGraph::new();

        let n0 = graph.add_node(NodeData::test_node(0));
        let n1 = graph.add_node(NodeData::test_node(1));
        let n2 = graph.add_node(NodeData::test_node(2)); // Dangling

        graph.add_edge(n0, n1, EdgeData::test_edge(1.0));
        graph.add_edge(n1, n2, EdgeData::test_edge(1.0));
        // n2 has no outgoing edges

        let matrices = GraphMatrices::from(&graph);
        let pr = PageRankComputer::default();
        let ranks = pr.compute(&matrices);

        // Should still sum to 1
        let sum: f64 = ranks.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);

        // With damping, dangling node distributes its rank randomly
        // This test just ensures the algorithm handles dangling nodes correctly
        // by verifying that sum is preserved and no NaN values
        for rank in &ranks {
            assert!(!rank.is_nan(), "PageRank should not contain NaN");
            assert!(*rank >= 0.0, "PageRank should be non-negative");
        }
    }

    #[test]
    fn test_pagerank_damping_factor() {
        // Use star graph for more pronounced damping effect
        let graph = create_star_graph();
        let matrices = GraphMatrices::from(&graph);

        // Test with different damping factors
        let pr_low = PageRankComputer::new().with_damping(0.5);
        let ranks_low = pr_low.compute(&matrices);

        let pr_high = PageRankComputer::new().with_damping(0.95);
        let ranks_high = pr_high.compute(&matrices);

        // Different damping should give different results (at least for hub)
        let hub_diff = (ranks_low[0] - ranks_high[0]).abs();
        assert!(
            hub_diff > 1e-4,
            "Damping factor should affect hub PageRank: diff={}",
            hub_diff
        );

        // Both should still sum to 1
        assert!((ranks_low.iter().sum::<f64>() - 1.0).abs() < 1e-6);
        assert!((ranks_high.iter().sum::<f64>() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_pagerank_empty_graph() {
        let graph = DiGraph::new();
        let matrices = GraphMatrices::from(&graph);
        let pr = PageRankComputer::default();
        let ranks = pr.compute(&matrices);

        assert_eq!(ranks.len(), 0);
    }

    #[test]
    fn test_pagerank_single_node() {
        let mut graph = DiGraph::new();
        graph.add_node(NodeData::test_node(0));

        let matrices = GraphMatrices::from(&graph);
        let pr = PageRankComputer::default();
        let ranks = pr.compute(&matrices);

        assert_eq!(ranks.len(), 1);
        assert!((ranks[0] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_pagerank_weighted_edges() {
        let mut graph = DiGraph::new();

        let n0 = graph.add_node(NodeData::test_node(0));
        let n1 = graph.add_node(NodeData::test_node(1));
        let n2 = graph.add_node(NodeData::test_node(2));

        // n0 has strong connection to n1, weak to n2
        graph.add_edge(
            n0,
            n1,
            EdgeData::Import {
                weight: 5.0,
                visibility: Visibility::Public,
            },
        );
        graph.add_edge(
            n0,
            n2,
            EdgeData::Import {
                weight: 0.5,
                visibility: Visibility::Public,
            },
        );

        // n1 and n2 point back to n0
        graph.add_edge(n1, n0, EdgeData::test_edge(1.0));
        graph.add_edge(n2, n0, EdgeData::test_edge(1.0));

        let matrices = GraphMatrices::from(&graph);
        let pr = PageRankComputer::default();
        let ranks = pr.compute(&matrices);

        // n1 should have higher rank than n2 due to stronger incoming edge
        assert!(
            ranks[1] > ranks[2],
            "Node with stronger incoming edge should have higher PageRank"
        );
    }
}
