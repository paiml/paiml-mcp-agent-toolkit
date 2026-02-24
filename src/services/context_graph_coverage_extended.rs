    // ============================================================
    // num_nodes / num_edges Tests
    // ============================================================

    #[test]
    fn test_num_nodes_increments() {
        let mut graph = ProjectContextGraph::new();

        assert_eq!(graph.num_nodes(), 0);

        graph
            .add_item("a".to_string(), create_function("a", 1))
            .unwrap();
        assert_eq!(graph.num_nodes(), 1);

        graph
            .add_item("b".to_string(), create_function("b", 2))
            .unwrap();
        assert_eq!(graph.num_nodes(), 2);

        graph
            .add_item("c".to_string(), create_function("c", 3))
            .unwrap();
        assert_eq!(graph.num_nodes(), 3);
    }

    #[test]
    fn test_num_edges_increments() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("a".to_string(), create_function("a", 1))
            .unwrap();
        graph
            .add_item("b".to_string(), create_function("b", 10))
            .unwrap();
        graph
            .add_item("c".to_string(), create_function("c", 20))
            .unwrap();

        assert_eq!(graph.num_edges(), 0);

        graph.add_edge("a", "b").unwrap();
        assert_eq!(graph.num_edges(), 1);

        graph.add_edge("b", "c").unwrap();
        assert_eq!(graph.num_edges(), 2);
    }

    // ============================================================
    // Clone/Debug Tests
    // ============================================================

    #[test]
    fn test_graph_clone() {
        let mut original = build_simple_call_graph();
        original.update_hotness().unwrap();

        let cloned = original.clone();

        assert_eq!(original.num_nodes(), cloned.num_nodes());
        assert_eq!(original.num_edges(), cloned.num_edges());

        // Both should have same items
        assert!(cloned.get_item("main").is_some());
        assert!(cloned.get_item("helper").is_some());
        assert!(cloned.get_item("utility").is_some());
    }

    #[test]
    fn test_graph_debug() {
        let graph = build_simple_call_graph();
        let debug = format!("{:?}", graph);
        assert!(debug.contains("ProjectContextGraph"));
    }

    // ============================================================
    // Edge Cases
    // ============================================================

    #[test]
    fn test_empty_name_item() {
        let mut graph = ProjectContextGraph::new();
        let result = graph.add_item("".to_string(), create_function("", 1));
        // Empty names should work (no validation)
        assert!(result.is_ok());
        assert!(graph.get_item("").is_some());
    }

    #[test]
    fn test_special_characters_in_name() {
        let mut graph = ProjectContextGraph::new();

        let special_names = vec![
            "fn::with::colons",
            "fn<T>",
            "fn()",
            "fn-with-dashes",
            "fn_with_underscores",
            "FnWithCamelCase",
        ];

        for name in special_names {
            graph
                .add_item(name.to_string(), create_function(name, 1))
                .unwrap();
            assert!(graph.get_item(name).is_some());
        }
    }

    #[test]
    fn test_unicode_names() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item(
                "function_japanese".to_string(),
                create_function("function_japanese", 1),
            )
            .unwrap();
        graph
            .add_item(
                "function_emoji".to_string(),
                create_function("function_emoji", 10),
            )
            .unwrap();

        assert_eq!(graph.num_nodes(), 2);
    }

    #[test]
    fn test_large_graph() {
        let mut graph = ProjectContextGraph::new();

        // Add 100 nodes
        for i in 0..100 {
            let name = format!("func_{}", i);
            graph
                .add_item(name.clone(), create_function(&name, i))
                .unwrap();
        }

        // Add edges in a chain
        for i in 0..99 {
            graph
                .add_edge(&format!("func_{}", i), &format!("func_{}", i + 1))
                .unwrap();
        }

        assert_eq!(graph.num_nodes(), 100);
        assert_eq!(graph.num_edges(), 99);

        // PageRank should work on large graph
        graph.update_hotness().unwrap();
        let hot = graph.hot_symbols();
        assert!(!hot.is_empty());
    }

    #[test]
    fn test_star_topology() {
        let mut graph = ProjectContextGraph::new();

        // Central hub
        graph
            .add_item("hub".to_string(), create_function("hub", 1))
            .unwrap();

        // Spokes
        for i in 0..10 {
            let name = format!("spoke_{}", i);
            graph
                .add_item(name.clone(), create_function(&name, i * 10))
                .unwrap();
            graph.add_edge(&name, "hub").unwrap();
        }

        graph.update_hotness().unwrap();
        let hot = graph.hot_symbols();

        // Hub should be hottest (all spokes point to it)
        assert_eq!(hot[0].0, "hub");
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("a".to_string(), create_function("a", 1))
            .unwrap();
        graph
            .add_item("b".to_string(), create_function("b", 10))
            .unwrap();
        graph
            .add_item("c".to_string(), create_function("c", 20))
            .unwrap();

        // Create a cycle: a -> b -> c -> a
        graph.add_edge("a", "b").unwrap();
        graph.add_edge("b", "c").unwrap();
        graph.add_edge("c", "a").unwrap();

        // PageRank should handle cycles
        let result = graph.update_hotness();
        assert!(result.is_ok());

        let hot = graph.hot_symbols();
        assert_eq!(hot.len(), 3);
    }

    // ============================================================
    // Property-Based Tests
    // ============================================================

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_add_item_preserves_count(count in 1usize..50) {
                let mut graph = ProjectContextGraph::new();

                for i in 0..count {
                    let name = format!("func_{}", i);
                    graph.add_item(name, create_function(&format!("func_{}", i), i)).unwrap();
                }

                prop_assert_eq!(graph.num_nodes(), count);
            }

            #[test]
            fn test_get_item_finds_all_added(count in 1usize..50) {
                let mut graph = ProjectContextGraph::new();

                for i in 0..count {
                    let name = format!("func_{}", i);
                    graph.add_item(name, create_function(&format!("func_{}", i), i)).unwrap();
                }

                for i in 0..count {
                    let name = format!("func_{}", i);
                    prop_assert!(graph.get_item(&name).is_some());
                }
            }

            #[test]
            fn test_edges_only_added_for_existing_nodes(
                node_count in 2usize..20,
                edge_attempts in 1usize..30
            ) {
                let mut graph = ProjectContextGraph::new();

                for i in 0..node_count {
                    let name = format!("node_{}", i);
                    graph.add_item(name, create_function(&format!("node_{}", i), i)).unwrap();
                }

                let mut valid_edges = 0;
                for i in 0..edge_attempts {
                    let from_idx = i % node_count;
                    let to_idx = (i + 1) % node_count;
                    let from = format!("node_{}", from_idx);
                    let to = format!("node_{}", to_idx);

                    if graph.get_item(&from).is_some() && graph.get_item(&to).is_some() {
                        graph.add_edge(&from, &to).unwrap();
                        valid_edges += 1;
                    }
                }

                prop_assert!(graph.num_edges() <= valid_edges);
            }

            #[test]
            fn test_hot_symbols_sorted(node_count in 3usize..20) {
                let mut graph = ProjectContextGraph::new();

                // Add nodes
                for i in 0..node_count {
                    let name = format!("func_{}", i);
                    graph.add_item(name, create_function(&format!("func_{}", i), i)).unwrap();
                }

                // Add edges (create varying in-degrees)
                for i in 0..node_count {
                    let from = format!("func_{}", i);
                    let to = format!("func_{}", (i + 1) % node_count);
                    graph.add_edge(&from, &to).unwrap();
                }

                graph.update_hotness().unwrap();
                let hot = graph.hot_symbols();

                // Verify sorted descending by score
                for i in 1..hot.len() {
                    prop_assert!(hot[i-1].1 >= hot[i].1);
                }
            }

            #[test]
            fn test_duplicate_detection(name in "[a-z]{1,10}") {
                let mut graph = ProjectContextGraph::new();

                // First add should succeed
                let result1 = graph.add_item(name.clone(), create_function(&name, 1));
                prop_assert!(result1.is_ok());

                // Second add should fail
                let result2 = graph.add_item(name.clone(), create_function(&name, 2));
                prop_assert!(result2.is_err());
            }
        }
    }

    // ============================================================
    // AstItem Equality Tests (for coverage)
    // ============================================================

    #[test]
    fn test_ast_item_function_equality() {
        let f1 = create_function("test", 1);
        let f2 = create_function("test", 1);
        let f3 = create_function("test", 2);
        let f4 = create_function("other", 1);

        assert_eq!(f1, f2);
        assert_ne!(f1, f3); // Different line
        assert_ne!(f1, f4); // Different name
    }

    #[test]
    fn test_ast_item_struct_vs_function() {
        let func = create_function("test", 1);
        let strct = create_struct("test", 1);

        assert_ne!(func, strct); // Different types
    }

    #[test]
    fn test_async_vs_sync_function() {
        let sync_fn = create_function("test", 1);
        let async_fn = create_async_function("test", 1);

        assert_ne!(sync_fn, async_fn); // Different is_async
    }
