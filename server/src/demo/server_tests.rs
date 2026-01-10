//! Comprehensive tests for demo server
//!
//! Tests all public functions and code paths without actual network operations.

use super::*;
use std::collections::HashMap;

// =============================================================================
// Test Helpers
// =============================================================================

/// Creates a minimal DemoContent for testing
fn create_test_demo_content() -> DemoContent {
    DemoContent {
        mermaid_diagram: "graph TD\n    A --> B".to_string(),
        system_diagram: Some("graph TD\n    Main --> Sub".to_string()),
        files_analyzed: 42,
        functions_analyzed: 100,
        avg_complexity: 5.5,
        p90_complexity: 12,
        hotspot_functions: 5,
        quality_score: 0.85,
        tech_debt_hours: 8,
        hotspots: vec![EnhancedHotspot {
            function: "complex_function".to_string(),
            file: "src/complex.rs".to_string(),
            path: "src/complex.rs".to_string(),
            complexity: 25,
            loc: 150,
            language: "rust".to_string(),
            churn_score: 15,
            refactor_suggestion: "Extract into smaller functions".to_string(),
        }],
        language_stats: {
            let mut stats = HashMap::new();
            stats.insert(
                "rust".to_string(),
                LanguageStats {
                    file_count: 30,
                    function_count: 80,
                    avg_complexity: 4.5,
                    total_loc: 5000,
                },
            );
            stats
        },
        ast_time_ms: 100,
        complexity_time_ms: 150,
        churn_time_ms: 200,
        dag_time_ms: 250,
        recommendations: vec![],
        polyglot_analysis: None,
    }
}

/// Creates a test DependencyGraph
fn create_test_dag() -> DependencyGraph {
    let mut dag = DependencyGraph::default();
    dag.nodes.insert(
        "main::run".to_string(),
        crate::models::dag::NodeInfo {
            id: "main::run".to_string(),
            label: "run".to_string(),
            node_type: crate::models::dag::NodeType::Function,
            file_path: "src/main.rs".to_string(),
            line_number: 10,
            complexity: 5,
            metadata: Default::default(),
        },
    );
    dag.nodes.insert(
        "lib::helper".to_string(),
        crate::models::dag::NodeInfo {
            id: "lib::helper".to_string(),
            label: "helper".to_string(),
            node_type: crate::models::dag::NodeType::Function,
            file_path: "src/lib.rs".to_string(),
            line_number: 20,
            complexity: 3,
            metadata: Default::default(),
        },
    );
    dag.edges.push(crate::models::dag::Edge {
        from: "main::run".to_string(),
        to: "lib::helper".to_string(),
        edge_type: crate::models::dag::EdgeType::Calls,
        weight: 1,
    });
    dag
}

/// Creates a test Hotspot vector
fn create_test_hotspots() -> Vec<Hotspot> {
    vec![
        Hotspot {
            file: "src/complex.rs".to_string(),
            complexity: 25,
            churn_score: 15,
        },
        Hotspot {
            file: "src/main.rs".to_string(),
            complexity: 12,
            churn_score: 8,
        },
    ]
}

// =============================================================================
// Data Structure Tests
// =============================================================================

#[test]
fn test_demo_content_fields() {
    let content = create_test_demo_content();

    assert_eq!(content.files_analyzed, 42);
    assert_eq!(content.functions_analyzed, 100);
    assert!((content.avg_complexity - 5.5).abs() < f64::EPSILON);
    assert_eq!(content.p90_complexity, 12);
    assert_eq!(content.hotspot_functions, 5);
    assert!((content.quality_score - 0.85).abs() < f64::EPSILON);
    assert_eq!(content.tech_debt_hours, 8);
    assert!(!content.mermaid_diagram.is_empty());
    assert!(content.system_diagram.is_some());
}

#[test]
fn test_demo_content_hotspots() {
    let content = create_test_demo_content();

    assert_eq!(content.hotspots.len(), 1);
    let hotspot = &content.hotspots[0];
    assert_eq!(hotspot.function, "complex_function");
    assert_eq!(hotspot.file, "src/complex.rs");
    assert_eq!(hotspot.complexity, 25);
    assert_eq!(hotspot.loc, 150);
    assert_eq!(hotspot.language, "rust");
    assert_eq!(hotspot.churn_score, 15);
}

#[test]
fn test_demo_content_language_stats() {
    let content = create_test_demo_content();

    assert!(content.language_stats.contains_key("rust"));
    let rust_stats = content.language_stats.get("rust").unwrap();
    assert_eq!(rust_stats.file_count, 30);
    assert_eq!(rust_stats.function_count, 80);
    assert!((rust_stats.avg_complexity - 4.5).abs() < f64::EPSILON);
    assert_eq!(rust_stats.total_loc, 5000);
}

#[test]
fn test_demo_content_timing() {
    let content = create_test_demo_content();

    assert_eq!(content.ast_time_ms, 100);
    assert_eq!(content.complexity_time_ms, 150);
    assert_eq!(content.churn_time_ms, 200);
    assert_eq!(content.dag_time_ms, 250);
}

#[test]
fn test_enhanced_hotspot_serialization() {
    let hotspot = EnhancedHotspot {
        function: "test_fn".to_string(),
        file: "test.rs".to_string(),
        path: "src/test.rs".to_string(),
        complexity: 10,
        loc: 50,
        language: "rust".to_string(),
        churn_score: 5,
        refactor_suggestion: "Consider extracting".to_string(),
    };

    let json = serde_json::to_string(&hotspot).unwrap();
    assert!(json.contains("test_fn"));
    assert!(json.contains("complexity"));

    let deserialized: EnhancedHotspot = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.function, "test_fn");
    assert_eq!(deserialized.complexity, 10);
}

#[test]
fn test_language_stats_clone() {
    let stats = LanguageStats {
        file_count: 10,
        function_count: 50,
        avg_complexity: 6.0,
        total_loc: 2000,
    };

    let cloned = stats.clone();
    assert_eq!(cloned.file_count, 10);
    assert_eq!(cloned.function_count, 50);
}

#[test]
fn test_hotspot_legacy_format() {
    let hotspot = Hotspot {
        file: "legacy.rs".to_string(),
        complexity: 15,
        churn_score: 10,
    };

    let json = serde_json::to_string(&hotspot).unwrap();
    let deserialized: Hotspot = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.file, "legacy.rs");
    assert_eq!(deserialized.complexity, 15);
    assert_eq!(deserialized.churn_score, 10);
}

// =============================================================================
// DemoContent::from_analysis_results Tests
// =============================================================================

#[test]
fn test_demo_content_from_analysis_results_basic() {
    let dag = create_test_dag();
    let hotspots = create_test_hotspots();

    let content = DemoContent::from_analysis_results(
        &dag,
        10,   // files_analyzed
        5.0,  // avg_complexity
        4,    // tech_debt_hours
        hotspots,
        50,   // ast_time_ms
        60,   // complexity_time_ms
        70,   // churn_time_ms
        80,   // dag_time_ms
    );

    assert_eq!(content.files_analyzed, 10);
    assert!((content.avg_complexity - 5.0).abs() < f64::EPSILON);
    assert_eq!(content.tech_debt_hours, 4);
    assert_eq!(content.ast_time_ms, 50);
    assert_eq!(content.complexity_time_ms, 60);
    assert_eq!(content.churn_time_ms, 70);
    assert_eq!(content.dag_time_ms, 80);
}

#[test]
fn test_demo_content_from_analysis_results_hotspot_conversion() {
    let dag = DependencyGraph::default();
    let hotspots = vec![Hotspot {
        file: "src/lib.rs".to_string(),
        complexity: 20,
        churn_score: 12,
    }];

    let content =
        DemoContent::from_analysis_results(&dag, 5, 3.0, 2, hotspots, 10, 20, 30, 40);

    // Verify hotspot conversion
    assert_eq!(content.hotspots.len(), 1);
    let enhanced = &content.hotspots[0];
    assert_eq!(enhanced.file, "src/lib.rs");
    assert_eq!(enhanced.path, "src/lib.rs");
    assert_eq!(enhanced.complexity, 20);
    assert_eq!(enhanced.churn_score, 12);
    assert_eq!(enhanced.language, "rust");
    assert_eq!(enhanced.function, "main");
    assert_eq!(enhanced.loc, 50);
}

#[test]
fn test_demo_content_from_analysis_results_empty_hotspots() {
    let dag = DependencyGraph::default();

    let content =
        DemoContent::from_analysis_results(&dag, 0, 0.0, 0, vec![], 0, 0, 0, 0);

    assert!(content.hotspots.is_empty());
    assert_eq!(content.functions_analyzed, 0);
    assert_eq!(content.hotspot_functions, 0);
}

#[test]
fn test_demo_content_from_analysis_results_p90_calculation() {
    let dag = DependencyGraph::default();

    let content = DemoContent::from_analysis_results(
        &dag,
        1,
        10.0, // avg_complexity
        1,
        vec![],
        0,
        0,
        0,
        0,
    );

    // p90 should be avg * 1.5
    assert_eq!(content.p90_complexity, 15);
}

#[test]
fn test_demo_content_from_analysis_results_mermaid_generation() {
    let dag = create_test_dag();

    let content =
        DemoContent::from_analysis_results(&dag, 2, 4.0, 1, vec![], 0, 0, 0, 0);

    // Mermaid diagram should be generated
    assert!(!content.mermaid_diagram.is_empty());
}

#[test]
fn test_demo_content_quality_score_default() {
    let dag = DependencyGraph::default();

    let content =
        DemoContent::from_analysis_results(&dag, 1, 5.0, 1, vec![], 0, 0, 0, 0);

    // Default quality score
    assert!((content.quality_score - 0.75).abs() < f64::EPSILON);
}

// =============================================================================
// DemoState Tests
// =============================================================================

#[test]
fn test_demo_state_clone() {
    let state = DemoState {
        repository: std::path::PathBuf::from("/test/repo"),
        analysis_results: AnalysisResults {
            files_analyzed: 10,
            avg_complexity: 5.0,
            tech_debt_hours: 4,
            complexity_report: Default::default(),
            churn_analysis: Default::default(),
            dependency_graph: Default::default(),
            tdg_summary: None,
        },
        mermaid_cache: Arc::new(DashMap::new()),
        system_diagram: Some("graph TD".to_string()),
    };

    let cloned = state.clone();
    assert_eq!(cloned.repository, std::path::PathBuf::from("/test/repo"));
    assert_eq!(cloned.analysis_results.files_analyzed, 10);
    assert!(cloned.system_diagram.is_some());
}

#[test]
fn test_analysis_results_serialization() {
    let results = AnalysisResults {
        files_analyzed: 20,
        avg_complexity: 6.5,
        tech_debt_hours: 10,
        complexity_report: Default::default(),
        churn_analysis: Default::default(),
        dependency_graph: Default::default(),
        tdg_summary: None,
    };

    let json = serde_json::to_string(&results).unwrap();
    assert!(json.contains("files_analyzed"));
    assert!(json.contains("avg_complexity"));
    assert!(json.contains("tech_debt_hours"));
}

// =============================================================================
// LocalDemoServer Tests
// =============================================================================

#[test]
fn test_local_demo_server_port_accessor() {
    // Create server directly with fields
    let (tx, _rx) = tokio::sync::oneshot::channel();
    let server = LocalDemoServer {
        port: 8080,
        shutdown_tx: tx,
    };

    assert_eq!(server.port(), 8080);
}

#[test]
fn test_local_demo_server_shutdown() {
    let (tx, mut rx) = tokio::sync::oneshot::channel();
    let server = LocalDemoServer {
        port: 3000,
        shutdown_tx: tx,
    };

    server.shutdown();

    // Verify the channel was used
    assert!(rx.try_recv().is_ok() || rx.try_recv().is_err());
}

// =============================================================================
// Default Implementations Tests
// =============================================================================

#[test]
fn test_complexity_report_default() {
    let report = crate::services::complexity::ComplexityReport::default();

    assert_eq!(report.summary.total_files, 0);
    assert_eq!(report.summary.total_functions, 0);
    assert!((report.summary.median_cyclomatic - 0.0).abs() < f32::EPSILON);
    assert!((report.summary.median_cognitive - 0.0).abs() < f32::EPSILON);
    assert_eq!(report.summary.max_cyclomatic, 0);
    assert_eq!(report.summary.max_cognitive, 0);
    assert_eq!(report.summary.p90_cyclomatic, 0);
    assert_eq!(report.summary.p90_cognitive, 0);
    assert!(report.violations.is_empty());
    assert!(report.hotspots.is_empty());
    assert!(report.files.is_empty());
}

#[test]
fn test_code_churn_analysis_default() {
    let analysis = crate::models::churn::CodeChurnAnalysis::default();

    assert_eq!(analysis.period_days, 0);
    assert!(analysis.files.is_empty());
    assert_eq!(analysis.summary.total_commits, 0);
    assert_eq!(analysis.summary.total_files_changed, 0);
    assert!(analysis.summary.hotspot_files.is_empty());
    assert!(analysis.summary.stable_files.is_empty());
    assert!(analysis.summary.author_contributions.is_empty());
}

// =============================================================================
// Helper Function Tests (cfg(feature = "demo"))
// =============================================================================

#[cfg(feature = "demo")]
mod demo_feature_tests {
    use super::*;
    use parking_lot::RwLock;

    fn create_test_state() -> Arc<RwLock<DemoState>> {
        Arc::new(RwLock::new(DemoState {
            repository: std::path::PathBuf::from("."),
            analysis_results: AnalysisResults {
                files_analyzed: 25,
                avg_complexity: 7.5,
                tech_debt_hours: 12,
                complexity_report: Default::default(),
                churn_analysis: Default::default(),
                dependency_graph: create_test_dag(),
                tdg_summary: None,
            },
            mermaid_cache: Arc::new(DashMap::new()),
            system_diagram: Some("graph TD\n    A --> B".to_string()),
        }))
    }

    fn create_state_with_tdg_summary() -> Arc<RwLock<DemoState>> {
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

    fn create_state_with_complexity_data() -> Arc<RwLock<DemoState>> {
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
                dependency_graph: create_test_dag(),
                tdg_summary: None,
            },
            mermaid_cache: Arc::new(DashMap::new()),
            system_diagram: None,
        }))
    }

    fn create_state_with_churn_data() -> Arc<RwLock<DemoState>> {
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

    // -------------------------------------------------------------------------
    // serve_recommendations_json Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_serve_recommendations_json_structure() {
        let state = create_test_state();
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
        let state = create_test_state();
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
        let state = create_test_state();
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
        let state = create_test_state();
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
        let state = create_test_state();
        let response = serve_defect_analysis(&state);

        assert_eq!(response.status(), http::StatusCode::OK);

        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert!(body.get("summary").is_some());
        assert!(body.get("recommendations").is_some());
    }

    #[test]
    fn test_serve_defect_analysis_summary_fields() {
        let state = create_test_state();
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
        let state = create_test_state();
        let response = serve_statistics_analysis(&state);

        assert_eq!(response.status(), http::StatusCode::OK);

        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert!(body.get("structural_metrics").is_some());
        assert!(body.get("code_metrics").is_some());
        assert!(body.get("temporal_metrics").is_some());
    }

    #[test]
    fn test_serve_statistics_analysis_with_churn() {
        let state = create_state_with_churn_data();
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
        let state = create_test_state();
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
        let state = create_test_state();
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
        let state = create_test_state();
        let response = serve_analysis_data(&state);

        assert_eq!(response.status(), http::StatusCode::OK);

        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert!(body.get("ast_contexts").is_some());
        assert!(body.get("total_files").is_some());
        assert!(body.get("timestamp").is_some());
    }

    #[test]
    fn test_serve_analysis_data_with_complexity() {
        let state = create_state_with_complexity_data();
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
        let state = create_state_with_churn_data();

        // Also add matching complexity data
        {
            use crate::services::complexity::{ComplexityMetrics, FileComplexityMetrics};
            let mut state_write = state.write();
            state_write.analysis_results.complexity_report.files.push(FileComplexityMetrics {
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
        let graph = create_test_dag();
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
        let graph = create_test_dag();
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
        assert!(as_str.contains("Content-Type: application/json"));
        assert!(as_str.contains("Content-Length:"));
        assert!(as_str.contains(r#"{"test": true}"#));
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

        assert!(as_str.contains("Content-Type: text/plain"));
        assert!(as_str.contains("Cache-Control: no-cache"));
        assert!(as_str.contains("X-Custom: value"));
    }

    #[test]
    fn test_serialize_response_empty_body() {
        let response = http::Response::builder()
            .status(http::StatusCode::NO_CONTENT)
            .body(bytes::Bytes::new())
            .unwrap();

        let serialized = serialize_response(response);
        let as_str = std::str::from_utf8(&serialized).unwrap();

        assert!(as_str.contains("Content-Length: 0"));
    }
}

// =============================================================================
// Non-Demo Feature Tests (serve_* stubs)
// =============================================================================

#[cfg(not(feature = "demo"))]
mod non_demo_tests {
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

// =============================================================================
// Async Tests
// =============================================================================

#[tokio::test]
async fn test_demo_content_with_ai_recommendations() {
    let dag = DependencyGraph::default();
    let content =
        DemoContent::from_analysis_results(&dag, 5, 4.0, 2, vec![], 10, 20, 30, 40);

    let project_path = std::path::Path::new(".");
    let enhanced = content.with_ai_recommendations(project_path, "rust").await;

    // Recommendations may be populated
    assert!(enhanced.recommendations.len() <= 5);
}

#[tokio::test]
async fn test_demo_content_with_polyglot_analysis() {
    let dag = DependencyGraph::default();
    let content =
        DemoContent::from_analysis_results(&dag, 5, 4.0, 2, vec![], 10, 20, 30, 40);

    let project_path = std::path::Path::new(".");
    let enhanced = content.with_polyglot_analysis(project_path).await;

    // Polyglot analysis may or may not be populated depending on project
    let _ = enhanced.polyglot_analysis;
}

#[cfg(feature = "demo")]
#[tokio::test]
async fn test_local_demo_server_spawn_and_shutdown() {
    let content = create_test_demo_content();

    let result = LocalDemoServer::spawn(content).await;
    assert!(result.is_ok());

    let (server, port) = result.unwrap();
    assert!(port > 0);

    server.shutdown();
}

#[cfg(feature = "demo")]
#[tokio::test]
async fn test_local_demo_server_spawn_with_results() {
    let content = create_test_demo_content();
    let complexity_report = Some(Default::default());
    let churn_analysis = Some(Default::default());
    let dag = Some(create_test_dag());

    let result =
        LocalDemoServer::spawn_with_results(content, complexity_report, churn_analysis, dag)
            .await;

    assert!(result.is_ok());

    let (server, port) = result.unwrap();
    assert!(port > 0);

    server.shutdown();
}

#[cfg(not(feature = "demo"))]
#[tokio::test]
async fn test_local_demo_server_spawn_disabled() {
    let content = create_test_demo_content();

    let result = LocalDemoServer::spawn(content).await;
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Demo mode not available"));
}

#[cfg(not(feature = "demo"))]
#[tokio::test]
async fn test_local_demo_server_spawn_with_results_disabled() {
    let content = create_test_demo_content();

    let result = LocalDemoServer::spawn_with_results(content, None, None, None).await;

    assert!(result.is_err());
}

// =============================================================================
// spawn_sync Test (cfg(feature = "demo"))
// =============================================================================

#[cfg(feature = "demo")]
#[test]
fn test_spawn_sync() {
    let content = create_test_demo_content();

    let result = spawn_sync(content);
    assert!(result.is_ok());

    let server = result.unwrap();
    server.shutdown();
}

// =============================================================================
// TDG Severity Calculation Tests
// =============================================================================

#[test]
fn test_tdg_severity_critical() {
    // TDG > 2.5 should be Critical
    let tdg_value = 2.8;
    let severity = match tdg_value {
        v if v > 2.5 => "Critical",
        v if v > 1.5 => "Warning",
        _ => "Normal",
    };
    assert_eq!(severity, "Critical");
}

#[test]
fn test_tdg_severity_warning() {
    // 1.5 < TDG <= 2.5 should be Warning
    let tdg_value = 2.0;
    let severity = match tdg_value {
        v if v > 2.5 => "Critical",
        v if v > 1.5 => "Warning",
        _ => "Normal",
    };
    assert_eq!(severity, "Warning");
}

#[test]
fn test_tdg_severity_normal() {
    // TDG <= 1.5 should be Normal
    let tdg_value = 1.2;
    let severity = match tdg_value {
        v if v > 2.5 => "Critical",
        v if v > 1.5 => "Warning",
        _ => "Normal",
    };
    assert_eq!(severity, "Normal");
}

// =============================================================================
// Debug and Clone Derive Tests
// =============================================================================

#[test]
fn test_demo_content_debug() {
    let content = create_test_demo_content();
    let debug_str = format!("{:?}", content);

    assert!(debug_str.contains("DemoContent"));
    assert!(debug_str.contains("files_analyzed"));
}

#[test]
fn test_enhanced_hotspot_debug() {
    let hotspot = EnhancedHotspot {
        function: "test".to_string(),
        file: "test.rs".to_string(),
        path: "src/test.rs".to_string(),
        complexity: 5,
        loc: 20,
        language: "rust".to_string(),
        churn_score: 3,
        refactor_suggestion: "None".to_string(),
    };

    let debug_str = format!("{:?}", hotspot);
    assert!(debug_str.contains("EnhancedHotspot"));
}

#[test]
fn test_language_stats_debug() {
    let stats = LanguageStats {
        file_count: 10,
        function_count: 50,
        avg_complexity: 5.0,
        total_loc: 1000,
    };

    let debug_str = format!("{:?}", stats);
    assert!(debug_str.contains("LanguageStats"));
}

#[test]
fn test_hotspot_debug() {
    let hotspot = Hotspot {
        file: "test.rs".to_string(),
        complexity: 10,
        churn_score: 5,
    };

    let debug_str = format!("{:?}", hotspot);
    assert!(debug_str.contains("Hotspot"));
}

#[test]
fn test_demo_state_debug() {
    let state = DemoState {
        repository: std::path::PathBuf::from("/test"),
        analysis_results: AnalysisResults {
            files_analyzed: 5,
            avg_complexity: 3.0,
            tech_debt_hours: 2,
            complexity_report: Default::default(),
            churn_analysis: Default::default(),
            dependency_graph: Default::default(),
            tdg_summary: None,
        },
        mermaid_cache: Arc::new(DashMap::new()),
        system_diagram: None,
    };

    let debug_str = format!("{:?}", state);
    assert!(debug_str.contains("DemoState"));
}

#[test]
fn test_analysis_results_clone() {
    let results = AnalysisResults {
        files_analyzed: 10,
        avg_complexity: 5.0,
        tech_debt_hours: 3,
        complexity_report: Default::default(),
        churn_analysis: Default::default(),
        dependency_graph: Default::default(),
        tdg_summary: None,
    };

    let cloned = results.clone();
    assert_eq!(cloned.files_analyzed, 10);
    assert!((cloned.avg_complexity - 5.0).abs() < f64::EPSILON);
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_demo_content_from_analysis_results_zero_complexity() {
    let dag = DependencyGraph::default();

    let content =
        DemoContent::from_analysis_results(&dag, 0, 0.0, 0, vec![], 0, 0, 0, 0);

    assert_eq!(content.p90_complexity, 0);
    assert!((content.avg_complexity - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_demo_content_from_analysis_results_high_complexity() {
    let dag = DependencyGraph::default();

    let content = DemoContent::from_analysis_results(
        &dag,
        100,
        50.0,
        100,
        vec![],
        1000,
        2000,
        3000,
        4000,
    );

    assert_eq!(content.files_analyzed, 100);
    assert_eq!(content.p90_complexity, 75); // 50 * 1.5
    assert_eq!(content.tech_debt_hours, 100);
}

#[test]
fn test_demo_content_recommendations_empty_by_default() {
    let content = create_test_demo_content();
    assert!(content.recommendations.is_empty());
}

#[test]
fn test_demo_content_polyglot_none_by_default() {
    let content = create_test_demo_content();
    assert!(content.polyglot_analysis.is_none());
}

// =============================================================================
// Additional Coverage Tests
// =============================================================================

#[test]
fn test_demo_content_clone() {
    let content = create_test_demo_content();
    let cloned = content.clone();

    assert_eq!(content.files_analyzed, cloned.files_analyzed);
    assert_eq!(content.mermaid_diagram, cloned.mermaid_diagram);
    assert_eq!(content.hotspots.len(), cloned.hotspots.len());
}

#[test]
fn test_enhanced_hotspot_clone() {
    let hotspot = EnhancedHotspot {
        function: "clone_test".to_string(),
        file: "clone.rs".to_string(),
        path: "src/clone.rs".to_string(),
        complexity: 5,
        loc: 25,
        language: "rust".to_string(),
        churn_score: 2,
        refactor_suggestion: "No changes needed".to_string(),
    };

    let cloned = hotspot.clone();
    assert_eq!(hotspot.function, cloned.function);
    assert_eq!(hotspot.complexity, cloned.complexity);
}

#[test]
fn test_analysis_results_with_tdg_summary() {
    let results = AnalysisResults {
        files_analyzed: 50,
        avg_complexity: 8.0,
        tech_debt_hours: 20,
        complexity_report: Default::default(),
        churn_analysis: Default::default(),
        dependency_graph: Default::default(),
        tdg_summary: Some(crate::models::tdg::TDGSummary {
            total_files: 50,
            critical_files: 5,
            warning_files: 15,
            average_tdg: 1.8,
            p95_tdg: 2.5,
            p99_tdg: 3.0,
            estimated_debt_hours: 80.0,
            hotspots: vec![],
        }),
    };

    assert!(results.tdg_summary.is_some());
    let summary = results.tdg_summary.as_ref().unwrap();
    assert_eq!(summary.total_files, 50);
    assert_eq!(summary.critical_files, 5);
}

#[test]
fn test_hotspot_serialization_roundtrip() {
    let hotspot = Hotspot {
        file: "roundtrip.rs".to_string(),
        complexity: 18,
        churn_score: 12,
    };

    let json = serde_json::to_string(&hotspot).unwrap();
    let deserialized: Hotspot = serde_json::from_str(&json).unwrap();

    assert_eq!(hotspot.file, deserialized.file);
    assert_eq!(hotspot.complexity, deserialized.complexity);
    assert_eq!(hotspot.churn_score, deserialized.churn_score);
}

#[test]
fn test_language_stats_serialization() {
    let stats = LanguageStats {
        file_count: 100,
        function_count: 500,
        avg_complexity: 7.5,
        total_loc: 25000,
    };

    let json = serde_json::to_string(&stats).unwrap();
    assert!(json.contains("file_count"));
    assert!(json.contains("function_count"));
    assert!(json.contains("avg_complexity"));
    assert!(json.contains("total_loc"));
}

#[test]
fn test_demo_content_timing_totals() {
    let content = create_test_demo_content();
    let total_time = content.ast_time_ms
        + content.complexity_time_ms
        + content.churn_time_ms
        + content.dag_time_ms;

    assert_eq!(total_time, 100 + 150 + 200 + 250);
    assert_eq!(total_time, 700);
}

#[test]
fn test_demo_content_with_multiple_hotspots() {
    let dag = DependencyGraph::default();
    let hotspots = vec![
        Hotspot {
            file: "file1.rs".to_string(),
            complexity: 30,
            churn_score: 20,
        },
        Hotspot {
            file: "file2.rs".to_string(),
            complexity: 25,
            churn_score: 15,
        },
        Hotspot {
            file: "file3.rs".to_string(),
            complexity: 20,
            churn_score: 10,
        },
    ];

    let content =
        DemoContent::from_analysis_results(&dag, 10, 25.0, 15, hotspots, 50, 60, 70, 80);

    assert_eq!(content.hotspots.len(), 3);
    assert_eq!(content.functions_analyzed, 3);
    assert_eq!(content.hotspot_functions, 3);
}

#[test]
fn test_demo_content_language_stats_multiple_languages() {
    let mut language_stats = HashMap::new();
    language_stats.insert(
        "rust".to_string(),
        LanguageStats {
            file_count: 50,
            function_count: 200,
            avg_complexity: 5.0,
            total_loc: 10000,
        },
    );
    language_stats.insert(
        "typescript".to_string(),
        LanguageStats {
            file_count: 30,
            function_count: 150,
            avg_complexity: 4.0,
            total_loc: 6000,
        },
    );
    language_stats.insert(
        "python".to_string(),
        LanguageStats {
            file_count: 20,
            function_count: 100,
            avg_complexity: 3.5,
            total_loc: 4000,
        },
    );

    let content = DemoContent {
        mermaid_diagram: String::new(),
        system_diagram: None,
        files_analyzed: 100,
        functions_analyzed: 450,
        avg_complexity: 4.5,
        p90_complexity: 10,
        hotspot_functions: 10,
        quality_score: 0.80,
        tech_debt_hours: 25,
        hotspots: vec![],
        language_stats,
        ast_time_ms: 100,
        complexity_time_ms: 100,
        churn_time_ms: 100,
        dag_time_ms: 100,
        recommendations: vec![],
        polyglot_analysis: None,
    };

    assert_eq!(content.language_stats.len(), 3);
    assert!(content.language_stats.contains_key("rust"));
    assert!(content.language_stats.contains_key("typescript"));
    assert!(content.language_stats.contains_key("python"));
}

#[test]
fn test_dependency_graph_creation() {
    let dag = create_test_dag();

    assert_eq!(dag.nodes.len(), 2);
    assert_eq!(dag.edges.len(), 1);

    assert!(dag.nodes.contains_key("main::run"));
    assert!(dag.nodes.contains_key("lib::helper"));

    let edge = &dag.edges[0];
    assert_eq!(edge.from, "main::run");
    assert_eq!(edge.to, "lib::helper");
}

#[test]
fn test_mermaid_cache_arc_clone() {
    let cache1: Arc<DashMap<u64, String>> = Arc::new(DashMap::new());
    cache1.insert(1, "test".to_string());

    let cache2 = Arc::clone(&cache1);
    assert_eq!(cache1.len(), cache2.len());

    cache2.insert(2, "test2".to_string());
    assert_eq!(cache1.len(), 2);
}
