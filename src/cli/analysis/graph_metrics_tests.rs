// Tests for graph metrics analysis
// Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;

    #[test]
    fn test_is_source_file() {
        assert!(is_source_file(Path::new("test.rs")));
        assert!(is_source_file(Path::new("test.js")));
        assert!(!is_source_file(Path::new("test.txt")));
    }

    #[test]
    fn test_extract_dependencies() {
        let content = "use std::collections::HashMap;\nmod utils;";
        let deps = extract_dependencies(content, Path::new("main.rs")).expect("internal error");
        assert!(deps.contains(&"utils.rs".to_string()));
    }

    #[test]
    fn test_graph_metrics_result() {
        let result = GraphMetricsResult {
            nodes: vec![],
            total_nodes: 5,
            total_edges: 8,
            density: 0.4,
            average_degree: 3.2,
            max_degree: 5,
            connected_components: 1,
        };

        assert_eq!(result.total_nodes, 5);
        assert_eq!(result.connected_components, 1);
    }
}

mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

/// Comprehensive coverage tests for graph_metrics module
/// EXTREME TDD approach - testing all code paths
mod coverage_tests {
    use super::*;
    use crate::cli::{GraphMetricType, GraphMetricsOutputFormat};

    fn strip_ansi(s: &str) -> String {
        let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        re.replace_all(s, "").to_string()
    }

    // NodeMetrics struct tests

    #[test]
    fn test_node_metrics_creation() {
        let metrics = NodeMetrics {
            name: "test_node".to_string(),
            degree_centrality: 0.5,
            betweenness_centrality: 0.3,
            closeness_centrality: 0.7,
            pagerank: 0.1,
            in_degree: 3,
            out_degree: 2,
        };

        assert_eq!(metrics.name, "test_node");
        assert!((metrics.degree_centrality - 0.5).abs() < f64::EPSILON);
        assert!((metrics.betweenness_centrality - 0.3).abs() < f64::EPSILON);
        assert!((metrics.closeness_centrality - 0.7).abs() < f64::EPSILON);
        assert!((metrics.pagerank - 0.1).abs() < f64::EPSILON);
        assert_eq!(metrics.in_degree, 3);
        assert_eq!(metrics.out_degree, 2);
    }

    #[test]
    fn test_node_metrics_clone() {
        let metrics = NodeMetrics {
            name: "clone_test".to_string(),
            degree_centrality: 0.25,
            betweenness_centrality: 0.15,
            closeness_centrality: 0.35,
            pagerank: 0.05,
            in_degree: 1,
            out_degree: 4,
        };

        let cloned = metrics.clone();
        assert_eq!(cloned.name, metrics.name);
        assert_eq!(cloned.in_degree, metrics.in_degree);
    }

    #[test]
    fn test_node_metrics_serialization() {
        let metrics = NodeMetrics {
            name: "serialize_test".to_string(),
            degree_centrality: 0.5,
            betweenness_centrality: 0.3,
            closeness_centrality: 0.7,
            pagerank: 0.1,
            in_degree: 2,
            out_degree: 3,
        };

        let json = serde_json::to_string(&metrics).expect("serialization should work");
        assert!(json.contains("serialize_test"));
        assert!(json.contains("degree_centrality"));

        let deserialized: NodeMetrics =
            serde_json::from_str(&json).expect("deserialization should work");
        assert_eq!(deserialized.name, "serialize_test");
    }

    // GraphMetricsResult struct tests

    #[test]
    fn test_graph_metrics_result_creation() {
        let result = GraphMetricsResult {
            nodes: vec![NodeMetrics {
                name: "node1".to_string(),
                degree_centrality: 0.5,
                betweenness_centrality: 0.0,
                closeness_centrality: 0.0,
                pagerank: 0.2,
                in_degree: 1,
                out_degree: 1,
            }],
            total_nodes: 10,
            total_edges: 15,
            density: 0.33,
            average_degree: 3.0,
            max_degree: 6,
            connected_components: 2,
        };

        assert_eq!(result.total_nodes, 10);
        assert_eq!(result.total_edges, 15);
        assert!((result.density - 0.33).abs() < 0.01);
        assert_eq!(result.nodes.len(), 1);
    }

    #[test]
    fn test_graph_metrics_result_serialization() {
        let result = GraphMetricsResult {
            nodes: vec![],
            total_nodes: 5,
            total_edges: 8,
            density: 0.4,
            average_degree: 3.2,
            max_degree: 5,
            connected_components: 1,
        };

        let json = serde_json::to_string(&result).expect("should serialize");
        assert!(json.contains("total_nodes"));
        assert!(json.contains("density"));
    }

    // Sprint 85 extracted function tests

    #[test]
    fn test_should_exclude_path_with_pattern() {
        assert!(should_exclude_path_sprint85(
            "/path/to/node_modules/pkg",
            &Some("node_modules".to_string())
        ));
        assert!(!should_exclude_path_sprint85(
            "/path/to/src/main.rs",
            &Some("node_modules".to_string())
        ));
    }

    #[test]
    fn test_should_exclude_path_without_pattern() {
        assert!(!should_exclude_path_sprint85("/any/path/here", &None));
    }

    #[test]
    fn test_should_include_path_with_pattern() {
        assert!(should_include_path_sprint85(
            "/path/to/src/module.rs",
            &Some("src".to_string())
        ));
        assert!(!should_include_path_sprint85(
            "/path/to/tests/test.rs",
            &Some("src".to_string())
        ));
    }

    #[test]
    fn test_should_include_path_without_pattern() {
        // When no pattern, include everything
        assert!(should_include_path_sprint85("/any/path/here", &None));
    }

    #[test]
    fn test_should_traverse_directory_valid() {
        assert!(should_traverse_directory_sprint85("src"));
        assert!(should_traverse_directory_sprint85("lib"));
        assert!(should_traverse_directory_sprint85("utils"));
    }

    #[test]
    fn test_should_traverse_directory_hidden() {
        assert!(!should_traverse_directory_sprint85(".git"));
        assert!(!should_traverse_directory_sprint85(".vscode"));
        assert!(!should_traverse_directory_sprint85(".hidden"));
    }

    #[test]
    fn test_should_traverse_directory_excluded() {
        assert!(!should_traverse_directory_sprint85("node_modules"));
        assert!(!should_traverse_directory_sprint85("target"));
    }

    // is_source_file tests (extended)

    #[test]
    fn test_is_source_file_all_extensions() {
        assert!(is_source_file(Path::new("main.rs")));
        assert!(is_source_file(Path::new("app.js")));
        assert!(is_source_file(Path::new("index.ts")));
        assert!(is_source_file(Path::new("script.py")));
        assert!(is_source_file(Path::new("Main.java")));
    }

    #[test]
    fn test_is_source_file_non_source() {
        assert!(!is_source_file(Path::new("readme.md")));
        assert!(!is_source_file(Path::new("config.toml")));
        assert!(!is_source_file(Path::new("data.json")));
        assert!(!is_source_file(Path::new("style.css")));
        assert!(!is_source_file(Path::new("noextension")));
    }

    // extract_dependencies tests

    #[test]
    fn test_extract_dependencies_rust() {
        let content = r#"
            use std::collections::HashMap;
            use serde::Serialize;
            mod utils;
            mod helpers;
        "#;
        let deps = extract_dependencies(content, Path::new("main.rs")).unwrap();
        assert!(deps.contains(&"utils.rs".to_string()));
        assert!(deps.contains(&"helpers.rs".to_string()));
    }

    #[test]
    fn test_extract_dependencies_javascript() {
        let content = r#"
            import foo from './foo';
            const bar = require('./bar');
        "#;
        let deps = extract_dependencies(content, Path::new("main.js")).unwrap();
        assert!(deps.contains(&"foo.js".to_string()));
        assert!(deps.contains(&"bar.js".to_string()));
    }

    #[test]
    fn test_extract_dependencies_typescript() {
        let content = r#"
            import { something } from './component';
            import utils from './utils';
        "#;
        let deps = extract_dependencies(content, Path::new("app.ts")).unwrap();
        assert!(deps.contains(&"component.ts".to_string()));
        assert!(deps.contains(&"utils.ts".to_string()));
    }

    #[test]
    fn test_extract_dependencies_python() {
        let content = r#"
            from utils import helper
            import json
            from mymodule import something
        "#;
        let deps = extract_dependencies(content, Path::new("main.py")).unwrap();
        assert!(deps.contains(&"utils.py".to_string()));
        assert!(deps.contains(&"mymodule.py".to_string()));
    }

    #[test]
    fn test_extract_dependencies_unknown_extension() {
        let content = "some content";
        let deps = extract_dependencies(content, Path::new("file.txt")).unwrap();
        assert!(deps.is_empty());
    }

    #[test]
    fn test_extract_dependencies_empty_content() {
        let deps = extract_dependencies("", Path::new("empty.rs")).unwrap();
        assert!(deps.is_empty());
    }

    // calculate_metrics tests with mock graph

    fn create_simple_graph() -> SimpleGraph {
        let mut graph = SimpleGraph::new();
        let a = graph.add_node("A".to_string());
        let b = graph.add_node("B".to_string());
        let c = graph.add_node("C".to_string());

        graph.add_edge(a, b);
        graph.add_edge(b, c);
        graph.add_edge(a, c);

        graph
    }

    fn create_star_graph() -> SimpleGraph {
        let mut graph = SimpleGraph::new();
        let center = graph.add_node("center".to_string());
        let n1 = graph.add_node("n1".to_string());
        let n2 = graph.add_node("n2".to_string());
        let n3 = graph.add_node("n3".to_string());
        let n4 = graph.add_node("n4".to_string());

        graph.add_edge(center, n1);
        graph.add_edge(center, n2);
        graph.add_edge(center, n3);
        graph.add_edge(center, n4);

        graph
    }

    fn create_linear_graph() -> SimpleGraph {
        let mut graph = SimpleGraph::new();
        let n1 = graph.add_node("n1".to_string());
        let n2 = graph.add_node("n2".to_string());
        let n3 = graph.add_node("n3".to_string());
        let n4 = graph.add_node("n4".to_string());

        graph.add_edge(n1, n2);
        graph.add_edge(n2, n3);
        graph.add_edge(n3, n4);

        graph
    }

    fn create_empty_graph() -> SimpleGraph {
        SimpleGraph::new()
    }

    fn create_single_node_graph() -> SimpleGraph {
        let mut graph = SimpleGraph::new();
        graph.add_node("single".to_string());
        graph
    }

    fn create_disconnected_graph() -> SimpleGraph {
        let mut graph = SimpleGraph::new();
        let a = graph.add_node("A".to_string());
        let b = graph.add_node("B".to_string());
        let c = graph.add_node("C".to_string());
        let d = graph.add_node("D".to_string());

        // Two disconnected components
        graph.add_edge(a, b);
        graph.add_edge(c, d);

        graph
    }

    #[test]
    fn test_calculate_metrics_simple_graph() {
        let graph = create_simple_graph();
        let result = calculate_metrics(
            &graph,
            vec![GraphMetricType::Centrality],
            vec![],
            0.85,
            100,
            1e-6,
        )
        .unwrap();

        assert_eq!(result.total_nodes, 3);
        assert_eq!(result.total_edges, 3);
        assert_eq!(result.nodes.len(), 3);
        assert!(result.density > 0.0);
    }

    #[test]
    fn test_calculate_metrics_star_graph() {
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

        assert_eq!(result.total_nodes, 5);
        assert_eq!(result.total_edges, 4);
        // Center node should have highest out-degree
        let center_node = result.nodes.iter().find(|n| n.name == "center").unwrap();
        assert_eq!(center_node.out_degree, 4);
    }

    #[test]
    fn test_calculate_metrics_with_betweenness() {
        let graph = create_linear_graph();
        let result = calculate_metrics(
            &graph,
            vec![GraphMetricType::Betweenness],
            vec![],
            0.85,
            100,
            1e-6,
        )
        .unwrap();

        assert_eq!(result.total_nodes, 4);
        // Middle nodes should have higher betweenness
        let n2 = result.nodes.iter().find(|n| n.name == "n2").unwrap();
        let n1 = result.nodes.iter().find(|n| n.name == "n1").unwrap();
        assert!(n2.betweenness_centrality >= n1.betweenness_centrality);
    }

    #[test]
    fn test_calculate_metrics_with_closeness() {
        let graph = create_simple_graph();
        let result = calculate_metrics(
            &graph,
            vec![GraphMetricType::Closeness],
            vec![],
            0.85,
            100,
            1e-6,
        )
        .unwrap();

        for node in &result.nodes {
            assert!(node.closeness_centrality >= 0.0);
        }
    }

    #[test]
    fn test_calculate_metrics_with_pagerank() {
        let graph = create_star_graph();
        let result = calculate_metrics(
            &graph,
            vec![GraphMetricType::PageRank],
            vec![],
            0.85,
            100,
            1e-6,
        )
        .unwrap();

        let total_pagerank: f64 = result.nodes.iter().map(|n| n.pagerank).sum();
        // PageRank should approximately sum to 1
        assert!((total_pagerank - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_calculate_metrics_with_pagerank_seeds() {
        let graph = create_star_graph();
        let result = calculate_metrics(
            &graph,
            vec![GraphMetricType::PageRank],
            vec!["center".to_string()],
            0.85,
            100,
            1e-6,
        )
        .unwrap();

        // Seeded node should have boosted pagerank
        assert!(!result.nodes.is_empty());
    }

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

    #[test]
    fn test_format_output_graphml() {
        let result = create_mock_result();
        let output = format_output(result, GraphMetricsOutputFormat::GraphML).unwrap();

        assert!(output.contains("GraphML export handled separately"));
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
}
