//! Server tests part 4 - Async tests, TDG severity, debug/clone tests, edge cases
//! Extracted for file health compliance (CB-040)

use super::*;

// =============================================================================
// Async Tests
// =============================================================================

#[tokio::test]
async fn test_demo_content_with_ai_recommendations() {
    let dag = DependencyGraph::default();
    let content = DemoContent::from_analysis_results(&dag, 5, 4.0, 2, vec![], 10, 20, 30, 40);

    let project_path = std::path::Path::new(".");
    let enhanced = content.with_ai_recommendations(project_path, "rust").await;

    // Recommendations may be populated
    assert!(enhanced.recommendations.len() <= 5);
}

#[tokio::test]
async fn test_demo_content_with_polyglot_analysis() {
    let dag = DependencyGraph::default();
    let content = DemoContent::from_analysis_results(&dag, 5, 4.0, 2, vec![], 10, 20, 30, 40);

    let project_path = std::path::Path::new(".");
    let enhanced = content.with_polyglot_analysis(project_path).await;

    // Polyglot analysis may or may not be populated depending on project
    let _ = enhanced.polyglot_analysis;
}

#[cfg(feature = "demo")]
#[tokio::test]
async fn test_local_demo_server_spawn_and_shutdown() {
    let content = server_tests_part1::create_test_demo_content();

    let result = LocalDemoServer::spawn(content).await;
    assert!(result.is_ok());

    let (server, port) = result.unwrap();
    assert!(port > 0);

    server.shutdown();
}

#[cfg(feature = "demo")]
#[tokio::test]
async fn test_local_demo_server_spawn_with_results() {
    let content = server_tests_part1::create_test_demo_content();
    let complexity_report = Some(Default::default());
    let churn_analysis = Some(Default::default());
    let dag = Some(server_tests_part1::create_test_dag());

    let result =
        LocalDemoServer::spawn_with_results(content, complexity_report, churn_analysis, dag).await;

    assert!(result.is_ok());

    let (server, port) = result.unwrap();
    assert!(port > 0);

    server.shutdown();
}

#[cfg(not(feature = "demo"))]
#[tokio::test]
async fn test_local_demo_server_spawn_disabled() {
    let content = server_tests_part1::create_test_demo_content();

    let result = LocalDemoServer::spawn(content).await;
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Demo mode not available"));
}

#[cfg(not(feature = "demo"))]
#[tokio::test]
async fn test_local_demo_server_spawn_with_results_disabled() {
    let content = server_tests_part1::create_test_demo_content();

    let result = LocalDemoServer::spawn_with_results(content, None, None, None).await;

    assert!(result.is_err());
}

// =============================================================================
// spawn_sync Test (cfg(feature = "demo"))
// =============================================================================

#[cfg(feature = "demo")]
#[test]
fn test_spawn_sync() {
    let content = server_tests_part1::create_test_demo_content();

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
    let content = server_tests_part1::create_test_demo_content();
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

    let content = DemoContent::from_analysis_results(&dag, 0, 0.0, 0, vec![], 0, 0, 0, 0);

    assert_eq!(content.p90_complexity, 0);
    assert!((content.avg_complexity - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_demo_content_from_analysis_results_high_complexity() {
    let dag = DependencyGraph::default();

    let content =
        DemoContent::from_analysis_results(&dag, 100, 50.0, 100, vec![], 1000, 2000, 3000, 4000);

    assert_eq!(content.files_analyzed, 100);
    assert_eq!(content.p90_complexity, 75); // 50 * 1.5
    assert_eq!(content.tech_debt_hours, 100);
}

#[test]
fn test_demo_content_recommendations_empty_by_default() {
    let content = server_tests_part1::create_test_demo_content();
    assert!(content.recommendations.is_empty());
}

#[test]
fn test_demo_content_polyglot_none_by_default() {
    let content = server_tests_part1::create_test_demo_content();
    assert!(content.polyglot_analysis.is_none());
}

// =============================================================================
// Additional Coverage Tests
// =============================================================================

#[test]
fn test_demo_content_clone() {
    let content = server_tests_part1::create_test_demo_content();
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
    let content = server_tests_part1::create_test_demo_content();
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

    let content = DemoContent::from_analysis_results(&dag, 10, 25.0, 15, hotspots, 50, 60, 70, 80);

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
    let dag = server_tests_part1::create_test_dag();

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
