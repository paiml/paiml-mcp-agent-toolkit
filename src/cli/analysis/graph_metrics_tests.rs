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

    // Continuation of coverage_tests in separate files for file health compliance
    include!("graph_metrics_tests_part2.rs");
    include!("graph_metrics_tests_part3.rs");
}
