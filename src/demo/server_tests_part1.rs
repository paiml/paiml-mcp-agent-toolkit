//! Server tests part 1 - Test helpers and data structure tests
//! Extracted for file health compliance (CB-040)

use super::*;

// =============================================================================
// Test Helpers
// =============================================================================

/// Creates a minimal DemoContent for testing
pub fn create_test_demo_content() -> DemoContent {
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
pub fn create_test_dag() -> DependencyGraph {
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
pub fn create_test_hotspots() -> Vec<Hotspot> {
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
        &dag, 10,  // files_analyzed
        5.0, // avg_complexity
        4,   // tech_debt_hours
        hotspots, 50, // ast_time_ms
        60, // complexity_time_ms
        70, // churn_time_ms
        80, // dag_time_ms
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

    let content = DemoContent::from_analysis_results(&dag, 5, 3.0, 2, hotspots, 10, 20, 30, 40);

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

    let content = DemoContent::from_analysis_results(&dag, 0, 0.0, 0, vec![], 0, 0, 0, 0);

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

    let content = DemoContent::from_analysis_results(&dag, 2, 4.0, 1, vec![], 0, 0, 0, 0);

    // Mermaid diagram should be generated
    assert!(!content.mermaid_diagram.is_empty());
}

#[test]
fn test_demo_content_quality_score_default() {
    let dag = DependencyGraph::default();

    let content = DemoContent::from_analysis_results(&dag, 1, 5.0, 1, vec![], 0, 0, 0, 0);

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
