
    #[test]
    fn test_calculate_metrics_all_types() {
        let graph = create_simple_graph();
        let result = calculate_metrics(
            &graph,
            vec![
                GraphMetricType::Centrality,
                GraphMetricType::Betweenness,
                GraphMetricType::Closeness,
                GraphMetricType::PageRank,
            ],
            vec![],
            0.85,
            100,
            1e-6,
        )
        .unwrap();

        for node in &result.nodes {
            assert!(node.degree_centrality >= 0.0);
            assert!(node.betweenness_centrality >= 0.0);
            assert!(node.closeness_centrality >= 0.0);
            assert!(node.pagerank >= 0.0);
        }
    }

    #[test]
    fn test_calculate_metrics_empty_graph() {
        let graph = create_empty_graph();
        let result = calculate_metrics(
            &graph,
            vec![GraphMetricType::Centrality],
            vec![],
            0.85,
            100,
            1e-6,
        )
        .unwrap();

        assert_eq!(result.total_nodes, 0);
        assert_eq!(result.total_edges, 0);
        assert!(result.nodes.is_empty());
    }

    #[test]
    fn test_calculate_metrics_single_node() {
        let graph = create_single_node_graph();
        let result = calculate_metrics(
            &graph,
            vec![GraphMetricType::Centrality],
            vec![],
            0.85,
            100,
            1e-6,
        )
        .unwrap();

        assert_eq!(result.total_nodes, 1);
        assert_eq!(result.total_edges, 0);
        assert_eq!(result.density, 0.0);
    }

    #[test]
    fn test_calculate_metrics_disconnected_graph() {
        let graph = create_disconnected_graph();
        let result = calculate_metrics(
            &graph,
            vec![GraphMetricType::Centrality],
            vec![],
            0.85,
            100,
            1e-6,
        )
        .unwrap();

        assert_eq!(result.total_nodes, 4);
        assert_eq!(result.connected_components, 2);
    }

    // calculate_betweenness tests

    #[test]
    fn test_calculate_betweenness_linear_graph() {
        let graph = create_linear_graph();
        let node_indices: Vec<_> = graph.node_indices().collect();

        // Middle node should have higher betweenness
        let betweenness_n2 = calculate_betweenness(&graph, node_indices[1]);
        assert!(betweenness_n2 >= 0.0);
    }

    #[test]
    fn test_calculate_betweenness_star_graph() {
        let graph = create_star_graph();
        let center_idx = graph.node_indices().next().unwrap();

        let betweenness = calculate_betweenness(&graph, center_idx);
        assert!(betweenness >= 0.0);
    }

    #[test]
    fn test_calculate_betweenness_two_node_graph() {
        let mut graph = SimpleGraph::new();
        let a = graph.add_node("A".to_string());
        let b = graph.add_node("B".to_string());
        graph.add_edge(a, b);

        // With only 2 nodes, betweenness should be 0
        let betweenness = calculate_betweenness(&graph, a);
        assert_eq!(betweenness, 0.0);
    }

    // calculate_closeness tests

    #[test]
    fn test_calculate_closeness_simple_graph() {
        let graph = create_simple_graph();
        let node = graph.node_indices().next().unwrap();

        let closeness = calculate_closeness(&graph, node);
        assert!(closeness >= 0.0);
    }

    #[test]
    fn test_calculate_closeness_disconnected_node() {
        let mut graph = SimpleGraph::new();
        graph.add_node("isolated".to_string());

        let node = graph.node_indices().next().unwrap();
        let closeness = calculate_closeness(&graph, node);
        assert_eq!(closeness, 0.0);
    }

    #[test]
    fn test_calculate_closeness_star_center() {
        let graph = create_star_graph();
        let center = graph.node_indices().next().unwrap();

        let closeness = calculate_closeness(&graph, center);
        assert!(closeness > 0.0);
    }

    // calculate_pagerank tests

    #[test]
    fn test_calculate_pagerank_simple() {
        let graph = create_simple_graph();
        let pageranks = calculate_pagerank(&graph, &[], 0.85, 100, 1e-6).unwrap();

        assert_eq!(pageranks.len(), 3);
        let total: f64 = pageranks.iter().sum();
        assert!((total - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_calculate_pagerank_with_seeds() {
        let graph = create_star_graph();
        let pageranks =
            calculate_pagerank(&graph, &["center".to_string()], 0.85, 100, 1e-6).unwrap();

        assert_eq!(pageranks.len(), 5);
    }

    #[test]
    fn test_calculate_pagerank_damping_variations() {
        let graph = create_simple_graph();

        // Test different damping factors
        let pr_high = calculate_pagerank(&graph, &[], 0.99, 100, 1e-6).unwrap();
        let pr_low = calculate_pagerank(&graph, &[], 0.5, 100, 1e-6).unwrap();

        // Both should have valid pageranks
        assert!(!pr_high.is_empty());
        assert!(!pr_low.is_empty());
    }

    #[test]
    fn test_calculate_pagerank_convergence() {
        let graph = create_simple_graph();

        // Test with tight convergence threshold
        let pr = calculate_pagerank(&graph, &[], 0.85, 1000, 1e-10).unwrap();
        assert!(!pr.is_empty());
    }

    #[test]
    fn test_calculate_pagerank_dangling_nodes() {
        // Graph with dangling node (no outgoing edges)
        let mut graph = SimpleGraph::new();
        let a = graph.add_node("A".to_string());
        let b = graph.add_node("B".to_string());
        let _c = graph.add_node("C".to_string()); // dangling

        graph.add_edge(a, b);

        let pageranks = calculate_pagerank(&graph, &[], 0.85, 100, 1e-6).unwrap();
        assert_eq!(pageranks.len(), 3);
    }

    // is_on_shortest_path tests

    #[test]
    fn test_is_on_shortest_path_linear() {
        let graph = create_linear_graph();
        let indices: Vec<_> = graph.node_indices().collect();

        // n2 is on path from n1 to n3
        let on_path = is_on_shortest_path(&graph, indices[0], indices[2], indices[1]);
        assert!(on_path);
    }

    #[test]
    fn test_is_on_shortest_path_not_on_path() {
        let mut graph = SimpleGraph::new();
        let a = graph.add_node("A".to_string());
        let b = graph.add_node("B".to_string());
        let c = graph.add_node("C".to_string());

        graph.add_edge(a, b);
        graph.add_edge(a, c);

        // c is not on path from a to b (direct edge)
        let on_path = is_on_shortest_path(&graph, a, b, c);
        assert!(!on_path);
    }

    #[test]
    fn test_is_on_shortest_path_no_path() {
        let graph = create_disconnected_graph();
        let indices: Vec<_> = graph.node_indices().collect();

        // No path between disconnected components
        let on_path = is_on_shortest_path(&graph, indices[0], indices[2], indices[1]);
        assert!(!on_path);
    }

    // filter_results tests

    fn create_mock_result() -> GraphMetricsResult {
        GraphMetricsResult {
            nodes: vec![
                NodeMetrics {
                    name: "high".to_string(),
                    degree_centrality: 0.9,
                    betweenness_centrality: 0.8,
                    closeness_centrality: 0.7,
                    pagerank: 0.3,
                    in_degree: 5,
                    out_degree: 4,
                },
                NodeMetrics {
                    name: "medium".to_string(),
                    degree_centrality: 0.5,
                    betweenness_centrality: 0.4,
                    closeness_centrality: 0.3,
                    pagerank: 0.2,
                    in_degree: 2,
                    out_degree: 2,
                },
                NodeMetrics {
                    name: "low".to_string(),
                    degree_centrality: 0.1,
                    betweenness_centrality: 0.05,
                    closeness_centrality: 0.08,
                    pagerank: 0.1,
                    in_degree: 1,
                    out_degree: 0,
                },
            ],
            total_nodes: 3,
            total_edges: 5,
            density: 0.5,
            average_degree: 3.33,
            max_degree: 9,
            connected_components: 1,
        }
    }

    #[test]
    fn test_filter_results_top_k() {
        let result = create_mock_result();
        let filtered = filter_results(result, 2, 0.0);

        assert_eq!(filtered.nodes.len(), 2);
        assert_eq!(filtered.nodes[0].name, "high");
    }

    #[test]
    fn test_filter_results_min_centrality() {
        let result = create_mock_result();
        let filtered = filter_results(result, 10, 0.2);

        // Only nodes with centrality >= 0.2 should remain
        assert!(filtered.nodes.iter().all(|n| n.degree_centrality >= 0.2
            || n.betweenness_centrality >= 0.2
            || n.closeness_centrality >= 0.2));
    }

    #[test]
    fn test_filter_results_combined() {
        let result = create_mock_result();
        let filtered = filter_results(result, 1, 0.0);

        assert_eq!(filtered.nodes.len(), 1);
        assert_eq!(filtered.nodes[0].name, "high");
    }

    #[test]
    fn test_filter_results_large_top_k() {
        let result = create_mock_result();
        let filtered = filter_results(result, 100, 0.0);

        assert_eq!(filtered.nodes.len(), 3);
    }

    // GraphML export tests

    #[test]
    fn test_write_graphml_header() {
        let mut output = String::new();
        write_graphml_header(&mut output).unwrap();

        assert!(output.contains("<?xml version"));
        assert!(output.contains("graphml"));
        assert!(output.contains("graph id=\"G\""));
    }

    #[test]
    fn test_write_graphml_nodes() {
        let nodes = vec![
            NodeMetrics {
                name: "node1".to_string(),
                degree_centrality: 0.5,
                betweenness_centrality: 0.0,
                closeness_centrality: 0.0,
                pagerank: 0.0,
                in_degree: 1,
                out_degree: 1,
            },
            NodeMetrics {
                name: "node2".to_string(),
                degree_centrality: 0.3,
                betweenness_centrality: 0.0,
                closeness_centrality: 0.0,
                pagerank: 0.0,
                in_degree: 0,
                out_degree: 1,
            },
        ];

        let mut output = String::new();
        write_graphml_nodes(&mut output, &nodes).unwrap();

        assert!(output.contains("node1"));
        assert!(output.contains("node2"));
        assert!(output.contains("<node id="));
    }

    #[test]
    fn test_write_graphml_edges() {
        let graph = create_simple_graph();
        let mut output = String::new();
        write_graphml_edges(&mut output, &graph).unwrap();

        assert!(output.contains("<edge source="));
        assert!(output.contains("target="));
    }

    #[test]
    fn test_write_graphml_footer() {
        let mut output = String::new();
        write_graphml_footer(&mut output).unwrap();

        assert!(output.contains("</graph>"));
        assert!(output.contains("</graphml>"));
    }

    #[test]
    fn test_write_graphml_file_with_path() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.graphml");
        let graphml_content = "<graphml>test</graphml>";

        write_graphml_file(graphml_content, &Some(file_path.clone())).unwrap();

        let content = std::fs::read_to_string(file_path.with_extension("graphml")).unwrap();
        assert!(content.contains("test"));
    }

    #[test]
    fn test_write_graphml_file_without_path() {
        let result = write_graphml_file("<graphml/>", &None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_to_graphml_integration() {
        use tempfile::tempdir;

        let graph = create_simple_graph();
        let result = GraphMetricsResult {
            nodes: vec![
                NodeMetrics {
                    name: "A".to_string(),
                    degree_centrality: 0.5,
                    betweenness_centrality: 0.0,
                    closeness_centrality: 0.0,
                    pagerank: 0.33,
                    in_degree: 1,
                    out_degree: 2,
                },
                NodeMetrics {
                    name: "B".to_string(),
                    degree_centrality: 0.5,
                    betweenness_centrality: 0.0,
                    closeness_centrality: 0.0,
                    pagerank: 0.33,
                    in_degree: 2,
                    out_degree: 1,
                },
            ],
            total_nodes: 3,
            total_edges: 3,
            density: 0.5,
            average_degree: 2.0,
            max_degree: 3,
            connected_components: 1,
        };

        let dir = tempdir().unwrap();
        let path = dir.path().join("output.graphml");

        let result_export = export_to_graphml(&graph, &result, &Some(path.clone()));
        assert!(result_export.is_ok());
    }
