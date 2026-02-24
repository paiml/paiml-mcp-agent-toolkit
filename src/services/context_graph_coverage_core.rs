    // ============================================================
    // ProjectContextGraph Creation Tests
    // ============================================================

    #[test]
    fn test_new_graph_is_empty() {
        let graph = ProjectContextGraph::new();
        assert_eq!(graph.num_nodes(), 0);
        assert_eq!(graph.num_edges(), 0);
        assert!(graph.hot_symbols().is_empty());
    }

    #[test]
    fn test_default_creates_empty_graph() {
        let graph = ProjectContextGraph::default();
        assert_eq!(graph.num_nodes(), 0);
        assert_eq!(graph.num_edges(), 0);
    }

    #[test]
    fn test_new_equals_default() {
        let new = ProjectContextGraph::new();
        let default = ProjectContextGraph::default();
        assert_eq!(new.num_nodes(), default.num_nodes());
        assert_eq!(new.num_edges(), default.num_edges());
    }

    // ============================================================
    // add_item Tests
    // ============================================================

    #[test]
    fn test_add_single_function() {
        let mut graph = ProjectContextGraph::new();
        let result = graph.add_item("test_fn".to_string(), create_function("test_fn", 1));

        assert!(result.is_ok());
        assert_eq!(graph.num_nodes(), 1);
    }

    #[test]
    fn test_add_multiple_items() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("fn1".to_string(), create_function("fn1", 1))
            .unwrap();
        graph
            .add_item("fn2".to_string(), create_function("fn2", 10))
            .unwrap();
        graph
            .add_item("fn3".to_string(), create_function("fn3", 20))
            .unwrap();

        assert_eq!(graph.num_nodes(), 3);
    }

    #[test]
    fn test_add_duplicate_fails() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("dup".to_string(), create_function("dup", 1))
            .unwrap();

        let result = graph.add_item("dup".to_string(), create_function("dup", 2));

        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("Duplicate"));
        assert!(err_msg.contains("dup"));
    }

    #[test]
    fn test_add_different_item_types() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("my_fn".to_string(), create_function("my_fn", 1))
            .unwrap();
        graph
            .add_item("my_struct".to_string(), create_struct("my_struct", 10))
            .unwrap();
        graph
            .add_item(
                "async_fn".to_string(),
                create_async_function("async_fn", 20),
            )
            .unwrap();
        graph
            .add_item(
                "private_fn".to_string(),
                create_private_function("private_fn", 30),
            )
            .unwrap();

        assert_eq!(graph.num_nodes(), 4);
    }

    // ============================================================
    // get_item Tests
    // ============================================================

    #[test]
    fn test_get_item_exists() {
        let mut graph = ProjectContextGraph::new();
        let item = create_function("test", 1);
        graph.add_item("test".to_string(), item.clone()).unwrap();

        let retrieved = graph.get_item("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &item);
    }

    #[test]
    fn test_get_item_not_found() {
        let graph = ProjectContextGraph::new();
        assert!(graph.get_item("nonexistent").is_none());
    }

    #[test]
    fn test_get_item_after_adding_multiple() {
        let mut graph = ProjectContextGraph::new();

        for i in 0..10 {
            let name = format!("func_{}", i);
            graph
                .add_item(name.clone(), create_function(&name, i * 10))
                .unwrap();
        }

        // Should find all items
        for i in 0..10 {
            let name = format!("func_{}", i);
            assert!(graph.get_item(&name).is_some());
        }

        // Should not find non-existent
        assert!(graph.get_item("func_99").is_none());
    }

    // ============================================================
    // add_edge Tests
    // ============================================================

    #[test]
    fn test_add_edge_between_existing_nodes() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("caller".to_string(), create_function("caller", 1))
            .unwrap();
        graph
            .add_item("callee".to_string(), create_function("callee", 10))
            .unwrap();

        let result = graph.add_edge("caller", "callee");
        assert!(result.is_ok());
        assert_eq!(graph.num_edges(), 1);
    }

    #[test]
    fn test_add_edge_from_nonexistent_node() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("callee".to_string(), create_function("callee", 10))
            .unwrap();

        // Should succeed (silently ignored if from doesn't exist)
        let result = graph.add_edge("nonexistent", "callee");
        assert!(result.is_ok());
        assert_eq!(graph.num_edges(), 0); // Edge not added
    }

    #[test]
    fn test_add_edge_to_nonexistent_node() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("caller".to_string(), create_function("caller", 1))
            .unwrap();

        // Should succeed (silently ignored if to doesn't exist)
        let result = graph.add_edge("caller", "nonexistent");
        assert!(result.is_ok());
        assert_eq!(graph.num_edges(), 0); // Edge not added
    }

    #[test]
    fn test_add_multiple_edges() {
        let graph = build_simple_call_graph();
        assert_eq!(graph.num_edges(), 3);
    }

    #[test]
    fn test_add_self_loop_edge() {
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("recursive".to_string(), create_function("recursive", 1))
            .unwrap();

        let result = graph.add_edge("recursive", "recursive");
        assert!(result.is_ok());
        // Self-loop should be added
        assert_eq!(graph.num_edges(), 1);
    }

    // ============================================================
    // update_hotness / PageRank Tests
    // ============================================================

    #[test]
    fn test_update_hotness_empty_graph() {
        let mut graph = ProjectContextGraph::new();
        let result = graph.update_hotness();
        assert!(result.is_ok());
        assert!(graph.hot_symbols().is_empty());
    }

    #[test]
    fn test_update_hotness_single_node_no_edges() {
        let mut graph = ProjectContextGraph::new();
        graph
            .add_item("lonely".to_string(), create_function("lonely", 1))
            .unwrap();

        let result = graph.update_hotness();
        assert!(result.is_ok());
        // Single node with no edges - may or may not appear in hot_symbols
        // depending on PageRank implementation
    }

    #[test]
    fn test_update_hotness_simple_graph() {
        let mut graph = build_simple_call_graph();

        let result = graph.update_hotness();
        assert!(result.is_ok());

        let hot = graph.hot_symbols();
        assert!(!hot.is_empty());

        // utility should be hottest (most in-edges)
        assert_eq!(hot[0].0, "utility");
    }

    #[test]
    fn test_update_hotness_complex_graph() {
        let mut graph = build_complex_graph();

        let result = graph.update_hotness();
        assert!(result.is_ok());

        let hot = graph.hot_symbols();
        assert!(!hot.is_empty());

        // util_shared should be hottest (called by multiple functions)
        assert_eq!(hot[0].0, "util_shared");
    }

    #[test]
    fn test_update_hotness_updates_cache() {
        let mut graph = build_simple_call_graph();

        // First update
        graph.update_hotness().unwrap();
        let hot1 = graph.hot_symbols();

        // Second update (should produce same results)
        graph.update_hotness().unwrap();
        let hot2 = graph.hot_symbols();

        assert_eq!(hot1.len(), hot2.len());
        for (a, b) in hot1.iter().zip(hot2.iter()) {
            assert_eq!(a.0, b.0);
        }
    }

    // ============================================================
    // hot_symbols Tests
    // ============================================================

    #[test]
    fn test_hot_symbols_empty_before_update() {
        let graph = build_simple_call_graph();
        assert!(graph.hot_symbols().is_empty());
    }

    #[test]
    fn test_hot_symbols_sorted_descending() {
        let mut graph = build_simple_call_graph();
        graph.update_hotness().unwrap();

        let hot = graph.hot_symbols();

        // Verify sorted by score descending
        for i in 1..hot.len() {
            assert!(
                hot[i - 1].1 >= hot[i].1,
                "hot_symbols should be sorted by score descending"
            );
        }
    }

    #[test]
    fn test_hot_symbols_contains_all_nodes_with_edges() {
        let mut graph = build_simple_call_graph();
        graph.update_hotness().unwrap();

        let hot = graph.hot_symbols();
        let names: Vec<&str> = hot.iter().map(|(n, _)| n.as_str()).collect();

        // All nodes in the graph should be in hot_symbols
        assert!(names.contains(&"main"));
        assert!(names.contains(&"helper"));
        assert!(names.contains(&"utility"));
    }
