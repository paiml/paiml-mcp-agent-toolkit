
    // format_output tests

    #[test]
    fn test_format_output_json() {
        let result = create_mock_result();
        let output = format_output(result, GraphMetricsOutputFormat::Json).unwrap();

        assert!(output.contains("total_nodes"));
        assert!(output.contains("\"nodes\""));
        serde_json::from_str::<serde_json::Value>(&output).expect("should be valid JSON");
    }

    #[test]
    fn test_format_output_human() {
        let result = create_mock_result();
        let output = format_output(result, GraphMetricsOutputFormat::Human).unwrap();

        assert!(output.contains("Graph Metrics Analysis"));
        assert!(output.contains("Total nodes"));
    }

    #[test]
    fn test_format_output_summary() {
        let result = create_mock_result();
        let output = format_output(result, GraphMetricsOutputFormat::Summary).unwrap();

        assert!(output.contains("Graph Metrics"));
        assert!(output.contains("Total"));
    }

    #[test]
    fn test_format_output_detailed() {
        let result = create_mock_result();
        let output = format_output(result, GraphMetricsOutputFormat::Detailed).unwrap();

        assert!(output.contains("Graph Metrics"));
    }

    #[test]
    fn test_format_output_csv() {
        let result = create_mock_result();
        let output = format_output(result, GraphMetricsOutputFormat::Csv).unwrap();

        assert!(output.contains("name,degree_centrality"));
        assert!(output.contains("high"));
        assert!(output.contains("medium"));
    }

    /// This test used to pin the defect: it asserted that `-f graph-ml` returns
    /// the developer note "GraphML export handled separately", which the command
    /// then wrote out as the whole output file with exit 0.
    #[test]
    fn test_format_output_graphml() {
        let result = create_mock_result();
        let err = format_output(result, GraphMetricsOutputFormat::GraphML)
            .expect_err("graph-ml must not render prose as a document");

        let message = err.to_string();
        assert!(
            !message.contains("handled separately"),
            "the error must tell the user what to run: {message}"
        );
        assert!(message.contains("--export-graphml"), "{message}");
    }

    #[test]
    fn test_format_output_markdown() {
        let result = create_mock_result();
        let output = format_output(result, GraphMetricsOutputFormat::Markdown).unwrap();

        assert!(output.contains("# Graph Metrics Report"));
        assert!(output.contains("| Metric | Value |"));
        assert!(output.contains("| Node |"));
    }

    // format helper function tests

    #[test]
    fn test_format_gm_as_json() {
        let result = create_mock_result();
        let json = format_gm_as_json(result).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("nodes").is_some());
        assert!(parsed.get("total_nodes").is_some());
    }

    #[test]
    fn test_format_gm_as_human() {
        let result = create_mock_result();
        let output = format_gm_as_human(result).unwrap();

        assert!(output.contains("Graph Metrics Analysis"));
        assert!(output.contains("Graph Statistics"));
        assert!(output.contains("Top Nodes"));
    }

    #[test]
    fn test_format_gm_as_csv() {
        let result = create_mock_result();
        let csv = format_gm_as_csv(result).unwrap();

        let lines: Vec<&str> = csv.lines().collect();
        assert!(lines[0].contains("name,degree_centrality"));
        assert!(lines.len() >= 4); // header + 3 data rows
    }

    #[test]
    fn test_format_gm_as_markdown() {
        let result = create_mock_result();
        let md = format_gm_as_markdown(result).unwrap();

        assert!(md.contains("# Graph Metrics Report"));
        assert!(md.contains("## Summary"));
        assert!(md.contains("## Top Nodes"));
        assert!(md.contains("|--------|"));
    }

    // write helper function tests

    #[test]
    fn test_write_gm_human_header() {
        let mut output = String::new();
        write_gm_human_header(&mut output).unwrap();
        let output = strip_ansi(&output);

        assert!(output.contains("Graph Metrics Analysis"));
        assert!(output.contains("Graph Statistics"));
    }

    #[test]
    fn test_write_gm_statistics() {
        let result = create_mock_result();
        let mut output = String::new();
        write_gm_statistics(&mut output, &result).unwrap();
        let output = strip_ansi(&output);

        assert!(output.contains("Total nodes:"));
        assert!(output.contains("3"));
        assert!(output.contains("Total edges:"));
        assert!(output.contains("5"));
        assert!(output.contains("Density:"));
        assert!(output.contains("Average degree:"));
        assert!(output.contains("Max degree:"));
        assert!(output.contains("9"));
        assert!(output.contains("Connected components:"));
        assert!(output.contains("1"));
    }

    #[test]
    fn test_write_gm_top_nodes() {
        let result = create_mock_result();
        let mut output = String::new();
        write_gm_top_nodes(&mut output, &result).unwrap();

        assert!(output.contains("Top Nodes by Centrality"));
        assert!(output.contains("high"));
        assert!(output.contains("medium"));
    }

    #[test]
    fn test_write_gm_node_details() {
        let node = NodeMetrics {
            name: "test_node".to_string(),
            degree_centrality: 0.75,
            betweenness_centrality: 0.5,
            closeness_centrality: 0.6,
            pagerank: 0.25,
            in_degree: 3,
            out_degree: 4,
        };

        let mut output = String::new();
        write_gm_node_details(&mut output, 1, &node).unwrap();
        let output = strip_ansi(&output);

        assert!(output.contains("1."));
        assert!(output.contains("test_node"));
        assert!(output.contains("Degree:"));
        assert!(output.contains("0.750"));
        assert!(output.contains("in: 3"));
        assert!(output.contains("out: 4"));
        assert!(output.contains("Betweenness:"));
        assert!(output.contains("0.500"));
        assert!(output.contains("Closeness:"));
        assert!(output.contains("0.600"));
        assert!(output.contains("PageRank:"));
        assert!(output.contains("0.250"));
    }

    #[test]
    fn test_write_gm_markdown_header() {
        let mut output = String::new();
        write_gm_markdown_header(&mut output).unwrap();

        assert!(output.contains("# Graph Metrics Report"));
        assert!(output.contains("## Summary"));
    }

    #[test]
    fn test_write_gm_markdown_summary() {
        let result = create_mock_result();
        let mut output = String::new();
        write_gm_markdown_summary(&mut output, &result).unwrap();

        assert!(output.contains("| Metric | Value |"));
        assert!(output.contains("|--------|-------|"));
        assert!(output.contains("| Total Nodes | 3 |"));
        assert!(output.contains("| Total Edges | 5 |"));
    }

    #[test]
    fn test_write_gm_markdown_top_nodes() {
        let result = create_mock_result();
        let mut output = String::new();
        write_gm_markdown_top_nodes(&mut output, &result).unwrap();

        assert!(output.contains("## Top Nodes"));
        assert!(output.contains("| Node | Degree | Betweenness | Closeness | PageRank |"));
        assert!(output.contains("high"));
    }

    #[test]
    fn test_write_gm_markdown_top_nodes_limit() {
        // Test that it only shows top 10
        let mut result = create_mock_result();
        for i in 0..15 {
            result.nodes.push(NodeMetrics {
                name: format!("node{}", i),
                degree_centrality: 0.1,
                betweenness_centrality: 0.1,
                closeness_centrality: 0.1,
                pagerank: 0.05,
                in_degree: 1,
                out_degree: 1,
            });
        }

        let mut output = String::new();
        write_gm_markdown_top_nodes(&mut output, &result).unwrap();

        // Count data rows (lines with node names)
        let data_rows: Vec<&str> = output
            .lines()
            .filter(|l| l.starts_with("| ") && !l.contains("Node") && !l.contains("---"))
            .collect();

        assert!(data_rows.len() <= 10);
    }

    // Edge case and error tests

    #[test]
    fn test_calculate_metrics_clustering_variant() {
        let graph = create_simple_graph();
        // Test with Clustering and Components variants (handled by _ => {} match)
        let result = calculate_metrics(
            &graph,
            vec![GraphMetricType::Clustering, GraphMetricType::Components],
            vec![],
            0.85,
            100,
            1e-6,
        )
        .unwrap();

        // These variants don't modify metrics but shouldn't error
        assert_eq!(result.total_nodes, 3);
    }

    #[test]
    fn test_calculate_metrics_all_variant() {
        let graph = create_simple_graph();
        let result =
            calculate_metrics(&graph, vec![GraphMetricType::All], vec![], 0.85, 100, 1e-6).unwrap();

        assert_eq!(result.total_nodes, 3);
    }

    #[test]
    fn test_node_metrics_debug() {
        let metrics = NodeMetrics {
            name: "debug_test".to_string(),
            degree_centrality: 0.5,
            betweenness_centrality: 0.3,
            closeness_centrality: 0.7,
            pagerank: 0.1,
            in_degree: 2,
            out_degree: 3,
        };

        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("debug_test"));
        assert!(debug_str.contains("NodeMetrics"));
    }

    #[test]
    fn test_graph_metrics_result_debug() {
        let result = create_mock_result();
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("GraphMetricsResult"));
    }

    // Pagerank edge cases

    #[test]
    fn test_pagerank_early_convergence() {
        let graph = create_simple_graph();
        // Test with very few iterations but high threshold
        let pr = calculate_pagerank(&graph, &[], 0.85, 2, 10.0).unwrap();
        assert_eq!(pr.len(), 3);
    }

    #[test]
    fn test_pagerank_zero_damping() {
        let graph = create_simple_graph();
        let pr = calculate_pagerank(&graph, &[], 0.0, 100, 1e-6).unwrap();

        // With 0 damping, all nodes should have equal probability
        let expected = 1.0 / 3.0;
        for p in &pr {
            assert!((*p - expected).abs() < 0.1);
        }
    }

    #[test]
    fn test_pagerank_full_damping() {
        let graph = create_simple_graph();
        let pr = calculate_pagerank(&graph, &[], 1.0, 100, 1e-6).unwrap();
        assert!(!pr.is_empty());
    }

    // Graph density edge cases

    #[test]
    fn test_density_single_node() {
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

        // Density should be 0 for single node
        assert_eq!(result.density, 0.0);
    }

    #[test]
    fn test_average_degree_calculation() {
        let graph = create_star_graph();
        let result = calculate_metrics(
            &graph,
            vec![GraphMetricType::Centrality],
            vec![],
            0.85,
            100,
            1e-6,
        )
        .unwrap();

        // 4 edges, 5 nodes, so average = 8/5 = 1.6 (counting both directions)
        assert!(result.average_degree > 0.0);
    }

    #[test]
    fn test_max_degree_calculation() {
        let graph = create_star_graph();
        let result = calculate_metrics(
            &graph,
            vec![GraphMetricType::Centrality],
            vec![],
            0.85,
            100,
            1e-6,
        )
        .unwrap();

        // Center node has degree 4 (all outgoing)
        assert_eq!(result.max_degree, 4);
    }

    // SimpleGraph tests

    #[test]
    fn test_simple_graph_basic_operations() {
        let mut graph = SimpleGraph::new();

        let a = graph.add_node("A".to_string());
        let b = graph.add_node("B".to_string());

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 0);

        graph.add_edge(a, b);

        assert_eq!(graph.edge_count(), 1);
        assert_eq!(graph.out_degree(a), 1);
        assert_eq!(graph.in_degree(b), 1);
    }

    #[test]
    fn test_simple_graph_dijkstra() {
        let mut graph = SimpleGraph::new();
        let a = graph.add_node("A".to_string());
        let b = graph.add_node("B".to_string());
        let c = graph.add_node("C".to_string());

        graph.add_edge(a, b);
        graph.add_edge(b, c);

        let distances = graph.dijkstra(a, None);
        assert_eq!(distances.get(&a), Some(&0));
        assert_eq!(distances.get(&b), Some(&1));
        assert_eq!(distances.get(&c), Some(&2));
    }

    #[test]
    fn test_simple_graph_connected_components() {
        let mut graph = SimpleGraph::new();
        let a = graph.add_node("A".to_string());
        let b = graph.add_node("B".to_string());
        let c = graph.add_node("C".to_string());
        let d = graph.add_node("D".to_string());

        graph.add_edge(a, b);
        graph.add_edge(c, d);

        assert_eq!(graph.connected_components(), 2);
    }

    // ── -f summary / -f human / -f detailed are three renderings (R45) ──────

    /// Regression: `Summary`, `Detailed` and `Human` shared one match arm, so
    /// `analyze graph-metrics --format summary|detailed|human` produced
    /// byte-identical output — 2634 identical bytes on this repo's
    /// `src/cli/analysis`. `--help` documents `summary` as "Summary statistics
    /// only" and `detailed` as "Detailed metrics with rankings"; neither was
    /// true of a renderer that also served `human`.
    #[test]
    fn test_summary_human_detailed_are_not_the_same_document() {
        let summary = strip_ansi(&format_output(
            create_mock_result(),
            crate::cli::GraphMetricsOutputFormat::Summary,
        )
        .unwrap());
        let human = strip_ansi(&format_output(
            create_mock_result(),
            crate::cli::GraphMetricsOutputFormat::Human,
        )
        .unwrap());
        let detailed = strip_ansi(&format_output(
            create_mock_result(),
            crate::cli::GraphMetricsOutputFormat::Detailed,
        )
        .unwrap());

        assert_ne!(summary, human, "summary must not equal human");
        assert_ne!(human, detailed, "human must not equal detailed");
        assert_ne!(summary, detailed, "summary must not equal detailed");

        // summary: statistics only, no per-node listing.
        assert!(summary.contains("Total nodes:"));
        assert!(
            !summary.contains("Top Nodes by Centrality"),
            "summary is statistics only:\n{summary}"
        );

        // human: statistics + the node listing.
        assert!(human.contains("Top Nodes by Centrality"));
        assert!(!human.contains("Ranked by PageRank"));

        // detailed: human, plus the rankings --help promises.
        assert!(detailed.contains("Top Nodes by Centrality"));
        for measure in ["PageRank", "Betweenness", "Closeness", "Degree"] {
            assert!(
                detailed.contains(&format!("Ranked by {measure}")),
                "detailed is missing the {measure} ranking:\n{detailed}"
            );
        }
        assert!(detailed.len() > human.len());
    }

    /// Ties in a ranking break on the node name, so `-f detailed` is a function
    /// of the graph rather than of the order the nodes happened to arrive in.
    #[test]
    fn test_detailed_rankings_are_order_independent() {
        let forward = create_mock_result();
        let mut reversed = create_mock_result();
        reversed.nodes.reverse();

        let a = strip_ansi(
            &format_output(forward, crate::cli::GraphMetricsOutputFormat::Detailed).unwrap(),
        );
        let b = strip_ansi(
            &format_output(reversed, crate::cli::GraphMetricsOutputFormat::Detailed).unwrap(),
        );

        let rankings = |s: &str| {
            s.lines()
                .skip_while(|l| !l.contains("Ranked by"))
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert_eq!(rankings(&a), rankings(&b));
    }
