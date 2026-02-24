    #[test]
    fn test_project_context_graph_creation() {
        let graph = ProjectContextGraph::new();
        assert_eq!(graph.num_nodes(), 0);
        assert_eq!(graph.num_edges(), 0);
    }

    #[test]
    fn test_add_item_o1_lookup() {
        let mut graph = ProjectContextGraph::new();

        let func = AstItem::Function {
            name: "test_func".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };

        graph
            .add_item("test_func".to_string(), func.clone())
            .unwrap();

        // O(1) lookup
        let retrieved = graph.get_item("test_func");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &func);

        // Non-existent lookup
        assert!(graph.get_item("nonexistent").is_none());
    }

    #[test]
    fn test_add_duplicate_fails() {
        let mut graph = ProjectContextGraph::new();

        let func = AstItem::Function {
            name: "dup".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };

        graph.add_item("dup".to_string(), func.clone()).unwrap();

        // Duplicate should fail
        let result = graph.add_item("dup".to_string(), func);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }

    #[test]
    fn test_add_edge_relationships() {
        let mut graph = ProjectContextGraph::new();

        // Add two functions
        graph
            .add_item(
                "main".to_string(),
                AstItem::Function {
                    name: "main".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 1,
                },
            )
            .unwrap();

        graph
            .add_item(
                "helper".to_string(),
                AstItem::Function {
                    name: "helper".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 10,
                },
            )
            .unwrap();

        // Add edge: main calls helper
        graph.add_edge("main", "helper").unwrap();

        assert_eq!(graph.num_nodes(), 2);
        assert_eq!(graph.num_edges(), 1);
    }

    #[test]
    fn test_pagerank_hotness() {
        let mut graph = ProjectContextGraph::new();

        // Create a simple call graph:
        // main → helper1
        // main → helper2
        // helper1 → helper2
        // (helper2 should have highest PageRank)

        graph
            .add_item(
                "main".to_string(),
                AstItem::Function {
                    name: "main".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 1,
                },
            )
            .unwrap();

        graph
            .add_item(
                "helper1".to_string(),
                AstItem::Function {
                    name: "helper1".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 10,
                },
            )
            .unwrap();

        graph
            .add_item(
                "helper2".to_string(),
                AstItem::Function {
                    name: "helper2".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 20,
                },
            )
            .unwrap();

        graph.add_edge("main", "helper1").unwrap();
        graph.add_edge("main", "helper2").unwrap();
        graph.add_edge("helper1", "helper2").unwrap();

        // Update PageRank
        graph.update_hotness().unwrap();

        // Get hot symbols
        let hot = graph.hot_symbols();
        assert_eq!(hot.len(), 3);

        // helper2 should be hottest (highest in-degree)
        assert_eq!(hot[0].0, "helper2");
        assert!(hot[0].1 > hot[1].1); // helper2 score > helper1 score
        assert!(hot[0].1 > hot[2].1); // helper2 score > main score
    }

    #[test]
    fn test_hot_symbols_ranking() {
        let mut graph = ProjectContextGraph::new();

        // Add 5 functions with varying in-degrees
        for i in 0..5 {
            graph
                .add_item(
                    format!("func{}", i),
                    AstItem::Function {
                        name: format!("func{}", i),
                        visibility: "pub".to_string(),
                        is_async: false,
                        line: i * 10,
                    },
                )
                .unwrap();
        }

        // func4 called by everyone (hottest)
        for i in 0..4 {
            graph.add_edge(&format!("func{}", i), "func4").unwrap();
        }

        graph.update_hotness().unwrap();
        let hot = graph.hot_symbols();

        // func4 should be #1
        assert_eq!(hot[0].0, "func4");
    }

    #[test]
    fn test_empty_graph_pagerank() {
        let mut graph = ProjectContextGraph::new();
        // Empty graph should not panic
        graph.update_hotness().unwrap();
        assert_eq!(graph.hot_symbols().len(), 0);
    }
