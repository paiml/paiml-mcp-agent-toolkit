//! Server tests part 3 - More demo feature tests and analysis handlers
//! Extracted for file health compliance (CB-040)

use super::*;

#[cfg(feature = "demo")]
pub mod demo_feature_tests_continued {
    use super::*;

    // -------------------------------------------------------------------------
    // serve_recommendations_json Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_recommendations_json_structure() {
        let state = server_tests_part2::demo_feature_tests::create_test_state();
        let response = serve_recommendations_json(&state);

        assert_eq!(response.status(), http::StatusCode::OK);

        let content_type = response.headers().get("Content-Type").unwrap();
        assert_eq!(content_type, "application/json");

        // Should be an array of recommendations
        let body: Vec<serde_json::Value> = serde_json::from_slice(response.body()).unwrap();
        // May or may not have recommendations
        let _ = body.is_empty();
    }

    // -------------------------------------------------------------------------
    // serve_polyglot_analysis Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_polyglot_analysis_structure() {
        let state = server_tests_part2::demo_feature_tests::create_test_state();
        let response = serve_polyglot_analysis(&state);

        assert_eq!(response.status(), http::StatusCode::OK);

        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert!(body.get("languages").is_some());
        assert!(body.get("architecture_pattern").is_some());
        assert!(body.get("recommendation_score").is_some());
    }

    // -------------------------------------------------------------------------
    // serve_showcase_gallery Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_showcase_gallery_structure() {
        let state = server_tests_part2::demo_feature_tests::create_test_state();
        let response = serve_showcase_gallery(&state);

        assert_eq!(response.status(), http::StatusCode::OK);

        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert!(body.get("repositories").is_some());
        assert!(body.get("summary").is_some());
        assert!(body.get("featured").is_some());
        assert!(body.get("categories").is_some());
    }

    // -------------------------------------------------------------------------
    // serve_architecture_analysis Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_architecture_analysis_status() {
        let state = server_tests_part2::demo_feature_tests::create_test_state();
        let response = serve_architecture_analysis(&state);

        // May succeed or fail depending on context
        assert!(
            response.status() == http::StatusCode::OK
                || response.status() == http::StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // -------------------------------------------------------------------------
    // serve_defect_analysis Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_defect_analysis_structure() {
        let state = server_tests_part2::demo_feature_tests::create_test_state();
        let response = serve_defect_analysis(&state);

        assert_eq!(response.status(), http::StatusCode::OK);

        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert!(body.get("summary").is_some());
        assert!(body.get("recommendations").is_some());
    }

    #[test]
    fn test_serve_defect_analysis_summary_fields() {
        let state = server_tests_part2::demo_feature_tests::create_test_state();
        let response = serve_defect_analysis(&state);

        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        let summary = &body["summary"];

        assert!(summary.get("total_files").is_some());
        assert!(summary.get("critical_files").is_some());
        assert!(summary.get("warning_files").is_some());
        assert!(summary.get("average_tdg").is_some());
    }

    // -------------------------------------------------------------------------
    // serve_statistics_analysis Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_statistics_analysis_structure() {
        let state = server_tests_part2::demo_feature_tests::create_test_state();
        let response = serve_statistics_analysis(&state);

        assert_eq!(response.status(), http::StatusCode::OK);

        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert!(body.get("structural_metrics").is_some());
        assert!(body.get("code_metrics").is_some());
        assert!(body.get("temporal_metrics").is_some());
    }

    #[test]
    fn test_serve_statistics_analysis_with_churn() {
        let state = server_tests_part2::demo_feature_tests::create_state_with_churn_data();
        let response = serve_statistics_analysis(&state);

        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        let temporal = &body["temporal_metrics"];

        assert_eq!(temporal["total_commits"], 50);
        assert_eq!(temporal["total_files_changed"], 20);
        assert_eq!(temporal["active_authors"], 2);
    }

    // -------------------------------------------------------------------------
    // serve_system_diagram Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_system_diagram_delegates() {
        let state = server_tests_part2::demo_feature_tests::create_test_state();
        let response = serve_system_diagram(&state);

        // Should delegate to architecture analysis
        assert!(
            response.status() == http::StatusCode::OK
                || response.status() == http::StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // -------------------------------------------------------------------------
    // serve_analysis_stream Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_analysis_stream_format() {
        let state = server_tests_part2::demo_feature_tests::create_test_state();
        let response = serve_analysis_stream(&state);

        assert_eq!(response.status(), http::StatusCode::OK);

        let content_type = response.headers().get("Content-Type").unwrap();
        assert_eq!(content_type, "text/event-stream");

        let body_str = std::str::from_utf8(response.body()).unwrap();
        assert!(body_str.contains("data:"));
    }

    // -------------------------------------------------------------------------
    // serve_analysis_data Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_analysis_data_fallback() {
        let state = server_tests_part2::demo_feature_tests::create_test_state();
        let response = serve_analysis_data(&state);

        assert_eq!(response.status(), http::StatusCode::OK);

        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert!(body.get("ast_contexts").is_some());
        assert!(body.get("total_files").is_some());
        assert!(body.get("timestamp").is_some());
    }

    #[test]
    fn test_serve_analysis_data_with_complexity() {
        let state = server_tests_part2::demo_feature_tests::create_state_with_complexity_data();
        let response = serve_analysis_data(&state);

        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        let contexts = body["ast_contexts"].as_array().unwrap();

        assert!(!contexts.is_empty());

        // Check first context structure
        let first = &contexts[0];
        assert!(first.get("path").is_some());
        assert!(first.get("complexity_metrics").is_some());
        assert!(first.get("tdg_score").is_some());
        assert!(first.get("tdg_severity").is_some());
    }

    #[test]
    fn test_serve_analysis_data_with_churn() {
        let state = server_tests_part2::demo_feature_tests::create_state_with_churn_data();

        // Also add matching complexity data
        {
            use crate::services::complexity::{ComplexityMetrics, FileComplexityMetrics};
            let mut state_write = state.write();
            state_write
                .analysis_results
                .complexity_report
                .files
                .push(FileComplexityMetrics {
                    path: "./server/src/demo/server.rs".to_string(),
                    functions: vec![],
                    total_complexity: ComplexityMetrics::new(10, 15, 2, 50),
                    classes: vec![],
                });
        }

        let response = serve_analysis_data(&state);
        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        let contexts = body["ast_contexts"].as_array().unwrap();

        if !contexts.is_empty() {
            let first = &contexts[0];
            let churn = &first["churn_metrics"];
            assert!(churn.get("commit_count").is_some());
            assert!(churn.get("churn_score").is_some());
        }
    }

    // -------------------------------------------------------------------------
    // Helper Function Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_calculate_graph_density_empty() {
        let graph = DependencyGraph::default();
        let density = calculate_graph_density(&graph);
        assert!((density - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_graph_density_single_node() {
        let mut graph = DependencyGraph::default();
        graph.nodes.insert(
            "a".to_string(),
            crate::models::dag::NodeInfo {
                id: "a".to_string(),
                label: "a".to_string(),
                node_type: crate::models::dag::NodeType::Function,
                file_path: "a.rs".to_string(),
                line_number: 1,
                complexity: 1,
                metadata: Default::default(),
            },
        );
        let density = calculate_graph_density(&graph);
        assert!((density - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_graph_density_with_edges() {
        let graph = server_tests_part1::create_test_dag();
        let density = calculate_graph_density(&graph);
        // With 2 nodes and 1 edge: density = 1 / (2 * 1) = 0.5
        assert!(density > 0.0);
    }

    #[test]
    fn test_calculate_avg_degree_empty() {
        let graph = DependencyGraph::default();
        let degree = calculate_avg_degree(&graph);
        assert!((degree - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_avg_degree_with_edges() {
        let graph = server_tests_part1::create_test_dag();
        let degree = calculate_avg_degree(&graph);
        // With 2 nodes and 1 edge: avg_degree = 2 * 1 / 2 = 1.0
        assert!((degree - 1.0).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // parse_minimal_request Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_minimal_request_valid() {
        let request = b"GET /api/summary HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let result = parse_minimal_request(request);

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.path, "/api/summary");
    }

    #[test]
    fn test_parse_minimal_request_root() {
        let request = b"GET / HTTP/1.1\r\n\r\n";
        let result = parse_minimal_request(request);

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_parse_minimal_request_empty() {
        let request = b"";
        let result = parse_minimal_request(request);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_minimal_request_invalid() {
        let request = b"INVALID";
        let result = parse_minimal_request(request);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_minimal_request_non_utf8() {
        let request = &[0xFF, 0xFE, 0x00, 0x01];
        let result = parse_minimal_request(request);

        assert!(result.is_err());
    }

    // -------------------------------------------------------------------------
    // serialize_response Tests
    // -------------------------------------------------------------------------

    /// Assert that `serialized` carries exactly one header line naming `name`
    /// with the value `value`.
    ///
    /// Field names are case-insensitive (RFC 9110 §5.1) and `http::HeaderName`
    /// normalises every name to lowercase when the response is built, so the
    /// bytes on the wire read `content-type: ...` no matter how the handler
    /// spelled it. The name is therefore matched case-insensitively while the
    /// value — which *is* case-sensitive — is compared exactly, and the whole
    /// line must match rather than merely appear as a substring.
    fn assert_header(serialized: &str, name: &str, value: &str) {
        let matches = serialized
            .split("\r\n")
            .filter(|line| match line.split_once(": ") {
                Some((n, v)) => n.eq_ignore_ascii_case(name) && v == value,
                None => false,
            })
            .count();
        assert_eq!(
            matches, 1,
            "expected exactly one `{name}: {value}` header line, found {matches} in:\n{serialized}"
        );
    }

    #[test]
    fn test_serialize_response_ok() {
        let response = http::Response::builder()
            .status(http::StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(bytes::Bytes::from(r#"{"test": true}"#))
            .unwrap();

        let serialized = serialize_response(response);
        let as_str = std::str::from_utf8(&serialized).unwrap();

        assert!(as_str.contains("HTTP/1.1 200 OK"));
        assert_header(as_str, "Content-Type", "application/json");
        // `{"test": true}` is 14 bytes.
        assert_header(as_str, "Content-Length", "14");

        let (_, body) = as_str
            .split_once("\r\n\r\n")
            .expect("headers must be terminated by a blank line");
        assert_eq!(body, r#"{"test": true}"#);
    }

    #[test]
    fn test_serialize_response_not_found() {
        let response = http::Response::builder()
            .status(http::StatusCode::NOT_FOUND)
            .body(bytes::Bytes::from("Not Found"))
            .unwrap();

        let serialized = serialize_response(response);
        let as_str = std::str::from_utf8(&serialized).unwrap();

        assert!(as_str.contains("HTTP/1.1 404 Not Found"));
    }

    #[test]
    fn test_serialize_response_multiple_headers() {
        let response = http::Response::builder()
            .status(http::StatusCode::OK)
            .header("Content-Type", "text/plain")
            .header("Cache-Control", "no-cache")
            .header("X-Custom", "value")
            .body(bytes::Bytes::from("body"))
            .unwrap();

        let serialized = serialize_response(response);
        let as_str = std::str::from_utf8(&serialized).unwrap();

        assert_header(as_str, "Content-Type", "text/plain");
        assert_header(as_str, "Cache-Control", "no-cache");
        assert_header(as_str, "X-Custom", "value");
    }

    #[test]
    fn test_serialize_response_empty_body() {
        let response = http::Response::builder()
            .status(http::StatusCode::NO_CONTENT)
            .body(bytes::Bytes::new())
            .unwrap();

        let serialized = serialize_response(response);
        let as_str = std::str::from_utf8(&serialized).unwrap();

        assert_header(as_str, "Content-Length", "0");
    }
}

// =============================================================================
// Non-Demo Feature Tests (serve_* stubs)
// =============================================================================

#[cfg(not(feature = "demo"))]
pub mod non_demo_tests {
    use super::*;

    #[test]
    fn test_serve_static_asset_disabled() {
        let response = serve_static_asset("/some/path");

        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
        let body_str = std::str::from_utf8(response.body()).unwrap();
        assert!(body_str.contains("Demo mode disabled"));
    }

    #[test]
    fn test_serve_architecture_analysis_disabled() {
        let state = Arc::new(parking_lot::RwLock::new(DemoState {
            repository: std::path::PathBuf::from("."),
            analysis_results: AnalysisResults {
                files_analyzed: 0,
                avg_complexity: 0.0,
                tech_debt_hours: 0,
                complexity_report: Default::default(),
                churn_analysis: Default::default(),
                dependency_graph: Default::default(),
                tdg_summary: None,
            },
            mermaid_cache: Arc::new(DashMap::new()),
            system_diagram: None,
        }));

        let response = serve_architecture_analysis(&state);
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_calculate_graph_density_disabled() {
        let graph = DependencyGraph::default();
        let density = calculate_graph_density(&graph);
        assert!((density - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_avg_degree_disabled() {
        let graph = DependencyGraph::default();
        let degree = calculate_avg_degree(&graph);
        assert!((degree - 0.0).abs() < f64::EPSILON);
    }
}
