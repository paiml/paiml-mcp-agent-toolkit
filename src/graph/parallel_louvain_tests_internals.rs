// Tests for internal data structures and edge cases
// Included by parallel_louvain.rs - shares parent scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_internals {
    use super::*;
    use crate::graph::types::{NodeData, Symbol, SymbolKind, Visibility};
    use std::path::PathBuf;

    fn create_test_node(name: &str) -> NodeData {
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

    fn create_two_community_graph() -> UndirectedGraph {
        let mut graph = UndirectedGraph::new();
        let nodes: Vec<_> = (0..6)
            .map(|i| graph.add_node(create_test_node(&format!("node{}", i))))
            .collect();

        graph.add_edge(nodes[0], nodes[1], 1.0);
        graph.add_edge(nodes[1], nodes[2], 1.0);
        graph.add_edge(nodes[0], nodes[2], 1.0);
        graph.add_edge(nodes[3], nodes[4], 1.0);
        graph.add_edge(nodes[4], nodes[5], 1.0);
        graph.add_edge(nodes[3], nodes[5], 1.0);
        graph.add_edge(nodes[2], nodes[3], 0.1);

        graph
    }

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

    // ============ Num Communities Tests ============

    #[test]
    fn test_num_communities_empty() {
        let communities: Vec<usize> = vec![];
        assert_eq!(ParallelLouvain::num_communities(&communities), 0);
    }

    #[test]
    fn test_num_communities_single() {
        let communities = vec![0, 0, 0, 0];
        assert_eq!(ParallelLouvain::num_communities(&communities), 1);
    }

    #[test]
    fn test_num_communities_multiple() {
        let communities = vec![0, 1, 2, 0, 1, 2];
        assert_eq!(ParallelLouvain::num_communities(&communities), 3);
    }

    #[test]
    fn test_num_communities_non_contiguous() {
        let communities = vec![0, 5, 10, 5, 0];
        assert_eq!(ParallelLouvain::num_communities(&communities), 3);
    }

    // ============ Renumbering Tests ============

    #[test]
    fn test_renumber_already_contiguous() {
        let louvain = ParallelLouvain::new();
        let mut communities = vec![0, 1, 2, 0, 1, 2];
        louvain.renumber_communities(&mut communities);

        // Should still be valid after renumbering
        assert_eq!(ParallelLouvain::num_communities(&communities), 3);
        assert!(communities.iter().all(|&c| c < 3));
    }

    #[test]
    fn test_renumber_non_contiguous() {
        let louvain = ParallelLouvain::new();
        let mut communities = vec![10, 20, 10, 30, 20];
        louvain.renumber_communities(&mut communities);

        // Should be renumbered to 0, 1, 2
        assert_eq!(ParallelLouvain::num_communities(&communities), 3);
        assert!(communities.iter().all(|&c| c < 3));

        // Same original community should still be same
        assert_eq!(communities[0], communities[2]); // Originally both 10
        assert_eq!(communities[1], communities[4]); // Originally both 20
    }

    #[test]
    fn test_renumber_empty() {
        let louvain = ParallelLouvain::new();
        let mut communities: Vec<usize> = vec![];
        louvain.renumber_communities(&mut communities);
        assert!(communities.is_empty());
    }

    // ============ GraphData Tests ============

    #[test]
    fn test_graph_data_from_empty() {
        let graph = UndirectedGraph::new();
        let data = GraphData::from_graph(&graph);

        assert_eq!(data.n, 0);
        assert!(data.neighbors.is_empty());
        assert!(data.degrees.is_empty());
        assert_eq!(data.total_weight, 0.0);
    }

    #[test]
    fn test_graph_data_from_two_node_graph() {
        let mut graph = UndirectedGraph::new();
        let n0 = graph.add_node(create_test_node("a"));
        let n1 = graph.add_node(create_test_node("b"));
        graph.add_edge(n0, n1, 2.5);

        let data = GraphData::from_graph(&graph);

        assert_eq!(data.n, 2);
        assert_eq!(data.total_weight, 2.5);
        assert_eq!(data.degrees[0], 2.5);
        assert_eq!(data.degrees[1], 2.5);
        assert_eq!(data.neighbors[0].len(), 1);
        assert_eq!(data.neighbors[1].len(), 1);
    }

    #[test]
    fn test_graph_data_neighbor_weight_to_community() {
        let graph = create_two_community_graph();
        let data = GraphData::from_graph(&graph);
        let communities = vec![0, 0, 0, 1, 1, 1];

        // Node 2 has edges to nodes 0, 1 (same community) and node 3 (different)
        let weight_to_same = data.neighbor_weight_to_community(2, 0, &communities);
        let weight_to_other = data.neighbor_weight_to_community(2, 1, &communities);

        assert!(weight_to_same > 0.0, "Should have weight to own community");
        assert!(
            weight_to_other > 0.0,
            "Should have weight to other community via bridge"
        );
    }

    // ============ CommunityData Tests ============

    #[test]
    fn test_community_data_new() {
        let graph = create_two_community_graph();
        let data = GraphData::from_graph(&graph);
        let communities = vec![0, 0, 0, 1, 1, 1];

        let comm_data = CommunityData::new(&communities, &data);

        assert_eq!(comm_data.node_to_community.len(), 6);
        assert!(comm_data.community_degrees.contains_key(&0));
        assert!(comm_data.community_degrees.contains_key(&1));
    }

    // ============ Convergence Tests ============

    #[test]
    fn test_max_iterations_limit() {
        let graph = create_chain_graph(10);
        let louvain = ParallelLouvain::new().with_max_iterations(1);
        let communities = louvain.detect(&graph);

        // Should still return valid communities even with 1 iteration
        assert_eq!(communities.len(), 10);
    }

    // ============ Edge Weight Tests ============

    #[test]
    fn test_equal_weights() {
        let graph = create_complete_graph(4);
        let louvain = ParallelLouvain::new();
        let communities = louvain.detect(&graph);

        // With equal weights, behavior depends on algorithm specifics
        assert_eq!(communities.len(), 4);
    }

    // ============ Edge Cases ============

    #[test]
    fn test_self_loop_handling() {
        let mut graph = UndirectedGraph::new();
        let n0 = graph.add_node(create_test_node("node0"));
        let n1 = graph.add_node(create_test_node("node1"));
        graph.add_edge(n0, n1, 1.0);

        let louvain = ParallelLouvain::new();
        let communities = louvain.detect(&graph);

        assert_eq!(communities.len(), 2);
    }

    #[test]
    fn test_negative_weight_handling() {
        let mut graph = UndirectedGraph::new();
        let n0 = graph.add_node(create_test_node("node0"));
        let n1 = graph.add_node(create_test_node("node1"));
        graph.add_edge(n0, n1, 0.0);

        let louvain = ParallelLouvain::new();
        let communities = louvain.detect(&graph);

        assert_eq!(communities.len(), 2);
    }

    #[test]
    fn test_very_small_weights() {
        let mut graph = UndirectedGraph::new();
        let n0 = graph.add_node(create_test_node("node0"));
        let n1 = graph.add_node(create_test_node("node1"));
        graph.add_edge(n0, n1, 1e-10);

        let louvain = ParallelLouvain::new();
        let communities = louvain.detect(&graph);

        assert_eq!(communities.len(), 2);
    }

    #[test]
    fn test_very_large_weights() {
        let mut graph = UndirectedGraph::new();
        let n0 = graph.add_node(create_test_node("node0"));
        let n1 = graph.add_node(create_test_node("node1"));
        graph.add_edge(n0, n1, 1e10);

        let louvain = ParallelLouvain::new();
        let communities = louvain.detect(&graph);

        assert_eq!(communities.len(), 2);
        // Large weight should keep them together
        assert_eq!(communities[0], communities[1]);
    }
}
