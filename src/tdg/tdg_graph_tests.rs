#[cfg_attr(coverage_nightly, coverage(off))]
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
    fn test_tdg_graph_default() {
        let graph = TdgGraph::default();
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
    fn test_add_edge_nonexistent_node_returns_ok() {
        let mut graph = TdgGraph::new();
        graph.add_function("existing".to_string()).unwrap();

        // Edge with non-existent "to" node silently succeeds
        let result = graph.add_edge("existing", "nonexistent");
        assert!(result.is_ok());
        assert_eq!(graph.num_edges(), 0);

        // Edge with non-existent "from" node silently succeeds
        let result = graph.add_edge("nonexistent", "existing");
        assert!(result.is_ok());
        assert_eq!(graph.num_edges(), 0);

        // Edge with both non-existent nodes silently succeeds
        let result = graph.add_edge("no_a", "no_b");
        assert!(result.is_ok());
        assert_eq!(graph.num_edges(), 0);
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
        fn test_tdg_graph_vis_edge_count() {
            let mut graph = TdgGraph::new();
            graph.add_function("a".to_string()).unwrap();
            graph.add_function("b".to_string()).unwrap();
            graph.add_function("c".to_string()).unwrap();
            graph.add_edge("a", "b").unwrap();
            graph.add_edge("b", "c").unwrap();
            graph.add_edge("a", "c").unwrap();
            graph.update_criticality().unwrap();

            let vis = graph.to_vis_graph();
            assert_eq!(vis.node_count(), 3);
            // Verify edges were copied from the CSR graph
            assert_eq!(vis.edges.len(), 3);
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
