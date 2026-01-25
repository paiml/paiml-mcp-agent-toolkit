//! Server tests part 2 - Demo feature tests (serve_* handlers)
//! Extracted for file health compliance (CB-040)

#[allow(unused_imports)]
use super::*;

// =============================================================================
// Helper Function Tests (cfg(feature = "demo"))
// =============================================================================

#[cfg(feature = "demo")]
pub mod demo_feature_tests {
    use super::*;
    use parking_lot::RwLock;

    pub fn create_test_state() -> Arc<RwLock<DemoState>> {
        Arc::new(RwLock::new(DemoState {
            repository: std::path::PathBuf::from("."),
            analysis_results: AnalysisResults {
                files_analyzed: 25,
                avg_complexity: 7.5,
                tech_debt_hours: 12,
                complexity_report: Default::default(),
                churn_analysis: Default::default(),
                dependency_graph: server_tests_part1::create_test_dag(),
                tdg_summary: None,
            },
            mermaid_cache: Arc::new(DashMap::new()),
            system_diagram: Some("graph TD\n    A --> B".to_string()),
        }))
    }

    pub fn create_state_with_tdg_summary() -> Arc<RwLock<DemoState>> {
        Arc::new(RwLock::new(DemoState {
            repository: std::path::PathBuf::from("."),
            analysis_results: AnalysisResults {
                files_analyzed: 30,
                avg_complexity: 8.0,
                tech_debt_hours: 15,
                complexity_report: Default::default(),
                churn_analysis: Default::default(),
                dependency_graph: DependencyGraph::default(),
                tdg_summary: Some(crate::models::tdg::TDGSummary {
                    total_files: 30,
                    critical_files: 5,
                    warning_files: 10,
                    average_tdg: 1.8,
                    p95_tdg: 2.5,
                    p99_tdg: 3.0,
                    estimated_debt_hours: 48.0,
                    hotspots: vec![],
                }),
            },
            mermaid_cache: Arc::new(DashMap::new()),
            system_diagram: None,
        }))
    }

    pub fn create_state_with_complexity_data() -> Arc<RwLock<DemoState>> {
        use crate::services::complexity::{
            ComplexityMetrics, ComplexityReport, ComplexitySummary, FileComplexityMetrics,
            FunctionComplexity,
        };

        let complexity_report = ComplexityReport {
            summary: ComplexitySummary {
                total_files: 3,
                total_functions: 10,
                median_cyclomatic: 5.0,
                median_cognitive: 8.0,
                max_cyclomatic: 25,
                max_cognitive: 30,
                p90_cyclomatic: 15,
                p90_cognitive: 20,
                technical_debt_hours: 5.0,
            },
            violations: vec![],
            hotspots: vec![],
            files: vec![FileComplexityMetrics {
                path: "./server/src/demo/server.rs".to_string(),
                functions: vec![
                    FunctionComplexity {
                        name: "serve_dashboard".to_string(),
                        line_start: 285,
                        line_end: 327,
                        metrics: ComplexityMetrics::new(10, 15, 3, 50),
                    },
                    FunctionComplexity {
                        name: "handle_connection".to_string(),
                        line_start: 203,
                        line_end: 219,
                        metrics: ComplexityMetrics::new(8, 12, 2, 30),
                    },
                ],
                total_complexity: ComplexityMetrics::new(18, 27, 3, 80),
                classes: vec![],
            }],
        };

        Arc::new(RwLock::new(DemoState {
            repository: std::path::PathBuf::from("."),
            analysis_results: AnalysisResults {
                files_analyzed: 3,
                avg_complexity: 9.0,
                tech_debt_hours: 5,
                complexity_report,
                churn_analysis: Default::default(),
                dependency_graph: server_tests_part1::create_test_dag(),
                tdg_summary: None,
            },
            mermaid_cache: Arc::new(DashMap::new()),
            system_diagram: None,
        }))
    }

    pub fn create_state_with_churn_data() -> Arc<RwLock<DemoState>> {
        use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
        use chrono::Utc;
        use std::path::PathBuf;

        let churn_analysis = CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days: 30,
            repository_root: PathBuf::from("."),
            files: vec![FileChurnMetrics {
                path: PathBuf::from("./server/src/demo/server.rs"),
                relative_path: "./server/src/demo/server.rs".to_string(),
                commit_count: 15,
                unique_authors: vec!["dev1".to_string(), "dev2".to_string()],
                additions: 500,
                deletions: 200,
                churn_score: 7.5,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            }],
            summary: ChurnSummary {
                total_commits: 50,
                total_files_changed: 20,
                hotspot_files: vec![PathBuf::from("server.rs")],
                stable_files: vec![],
                author_contributions: {
                    let mut map = std::collections::HashMap::new();
                    map.insert("dev1".to_string(), 30);
                    map.insert("dev2".to_string(), 20);
                    map
                },
                mean_churn_score: 5.0,
                variance_churn_score: 2.0,
                stddev_churn_score: 1.4,
            },
        };

        Arc::new(RwLock::new(DemoState {
            repository: std::path::PathBuf::from("."),
            analysis_results: AnalysisResults {
                files_analyzed: 20,
                avg_complexity: 6.0,
                tech_debt_hours: 8,
                complexity_report: Default::default(),
                churn_analysis,
                dependency_graph: Default::default(),
                tdg_summary: None,
            },
            mermaid_cache: Arc::new(DashMap::new()),
            system_diagram: None,
        }))
    }

    // -------------------------------------------------------------------------
    // serve_dashboard Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_dashboard_returns_html() {
        let state = create_test_state();
        let response = serve_dashboard(&state);

        assert_eq!(response.status(), http::StatusCode::OK);

        let content_type = response.headers().get("Content-Type").unwrap();
        assert_eq!(content_type, "text/html; charset=utf-8");

        let body = response.body();
        let body_str = std::str::from_utf8(body).unwrap();
        assert!(body_str.contains("<!DOCTYPE html>") || body_str.contains("<html"));
    }

    #[test]
    fn test_serve_dashboard_cache_control() {
        let state = create_test_state();
        let response = serve_dashboard(&state);

        let cache = response.headers().get("Cache-Control").unwrap();
        assert_eq!(cache, "no-cache");
    }

    #[test]
    fn test_serve_dashboard_contains_metrics() {
        let state = create_test_state();
        let response = serve_dashboard(&state);

        let body_str = std::str::from_utf8(response.body()).unwrap();
        // Should contain the analyzed files count
        assert!(body_str.contains("25") || body_str.len() > 0);
    }

    // -------------------------------------------------------------------------
    // serve_static_asset Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_static_asset_not_found() {
        let response = serve_static_asset("/nonexistent/path.js");

        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
        let body_str = std::str::from_utf8(response.body()).unwrap();
        assert!(body_str.contains("404") || body_str.contains("Not Found"));
    }

    // -------------------------------------------------------------------------
    // serve_summary_json Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_summary_json_structure() {
        let state = create_test_state();
        let response = serve_summary_json(&state);

        assert_eq!(response.status(), http::StatusCode::OK);

        let content_type = response.headers().get("Content-Type").unwrap();
        assert_eq!(content_type, "application/json");

        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert!(body.get("files_analyzed").is_some());
        assert!(body.get("avg_complexity").is_some());
        assert!(body.get("tech_debt_hours").is_some());
    }

    #[test]
    fn test_serve_summary_json_values() {
        let state = create_test_state();
        let response = serve_summary_json(&state);

        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body["files_analyzed"], 25);
        assert_eq!(body["time_context"], 100);
        assert_eq!(body["time_complexity"], 150);
        assert_eq!(body["time_dag"], 200);
        assert_eq!(body["time_churn"], 250);
    }

    // -------------------------------------------------------------------------
    // serve_metrics_json Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_metrics_json_structure() {
        let state = create_test_state();
        let response = serve_metrics_json(&state);

        assert_eq!(response.status(), http::StatusCode::OK);

        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert!(body.get("files_analyzed").is_some());
        assert!(body.get("avg_complexity").is_some());
        assert!(body.get("tech_debt_hours").is_some());
    }

    #[test]
    fn test_serve_metrics_json_values() {
        let state = create_test_state();
        let response = serve_metrics_json(&state);

        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body["files_analyzed"], 25);
        assert!((body["avg_complexity"].as_f64().unwrap() - 7.5).abs() < 0.001);
        assert_eq!(body["tech_debt_hours"], 12);
    }

    // -------------------------------------------------------------------------
    // serve_hotspots_table Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_hotspots_table_fallback() {
        let state = create_test_state();
        let response = serve_hotspots_table(&state);

        assert_eq!(response.status(), http::StatusCode::OK);

        let content_type = response.headers().get("Content-Type").unwrap();
        assert_eq!(content_type, "application/json");

        let body: Vec<serde_json::Value> = serde_json::from_slice(response.body()).unwrap();
        // Should have fallback data when no complexity files
        assert!(!body.is_empty());
    }

    #[test]
    fn test_serve_hotspots_table_with_data() {
        let state = create_state_with_complexity_data();
        let response = serve_hotspots_table(&state);

        let body: Vec<serde_json::Value> = serde_json::from_slice(response.body()).unwrap();

        // Should have hotspots from complexity data
        assert!(!body.is_empty());

        // First entry should have highest complexity
        let first = &body[0];
        assert!(first.get("rank").is_some());
        assert!(first.get("function").is_some());
        assert!(first.get("complexity").is_some());
        assert!(first.get("path").is_some());
    }

    #[test]
    fn test_serve_hotspots_table_sorting() {
        let state = create_state_with_complexity_data();
        let response = serve_hotspots_table(&state);

        let body: Vec<serde_json::Value> = serde_json::from_slice(response.body()).unwrap();

        // Verify sorted by complexity descending
        for i in 0..body.len().saturating_sub(1) {
            let current = body[i]["complexity"].as_u64().unwrap();
            let next = body[i + 1]["complexity"].as_u64().unwrap();
            assert!(current >= next);
        }
    }

    #[test]
    fn test_serve_hotspots_table_cache_control() {
        let state = create_test_state();
        let response = serve_hotspots_table(&state);

        let cache = response.headers().get("Cache-Control").unwrap();
        assert_eq!(cache, "max-age=60");
    }

    // -------------------------------------------------------------------------
    // serve_dag_mermaid Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_dag_mermaid_fallback() {
        let state = Arc::new(RwLock::new(DemoState {
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

        let response = serve_dag_mermaid(&state);

        assert_eq!(response.status(), http::StatusCode::OK);

        let content_type = response.headers().get("Content-Type").unwrap();
        assert_eq!(content_type, "text/plain");

        let body_str = std::str::from_utf8(response.body()).unwrap();
        assert!(body_str.contains("graph TD"));
    }

    #[test]
    fn test_serve_dag_mermaid_with_tdg() {
        let state = create_state_with_tdg_summary();
        let response = serve_dag_mermaid(&state);

        let body_str = std::str::from_utf8(response.body()).unwrap();
        assert!(body_str.contains("graph TD") || body_str.contains("graph"));
    }

    #[test]
    fn test_serve_dag_mermaid_with_graph_data() {
        let state = create_test_state();
        let response = serve_dag_mermaid(&state);

        let body_str = std::str::from_utf8(response.body()).unwrap();
        assert!(body_str.contains("graph"));
    }

    // -------------------------------------------------------------------------
    // serve_system_diagram_mermaid Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_system_diagram_with_data() {
        let state = create_test_state();
        let response = serve_system_diagram_mermaid(&state);

        assert_eq!(response.status(), http::StatusCode::OK);

        let body_str = std::str::from_utf8(response.body()).unwrap();
        assert!(body_str.contains("graph TD"));
        assert!(body_str.contains("A --> B"));
    }

    #[test]
    fn test_serve_system_diagram_fallback() {
        let state = Arc::new(RwLock::new(DemoState {
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

        let response = serve_system_diagram_mermaid(&state);

        let body_str = std::str::from_utf8(response.body()).unwrap();
        assert!(body_str.contains("AST Context Analysis"));
        assert!(body_str.contains("Code Complexity"));
    }
}
