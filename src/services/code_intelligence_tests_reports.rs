// === ComplexityReport tests ===

#[test]
fn test_complexity_report_creation() {
    let report = ComplexityReport {
        total_files: 10,
        average_complexity: 5.5,
        hotspots: vec![],
    };

    assert_eq!(report.total_files, 10);
    assert_eq!(report.average_complexity, 5.5);
    assert!(report.hotspots.is_empty());
}

#[test]
fn test_complexity_report_with_hotspots() {
    let hotspot = ComplexityHotspot {
        file_path: "src/main.rs".to_string(),
        function_name: "process_data".to_string(),
        cyclomatic_complexity: 15,
        cognitive_complexity: 20,
    };

    let report = ComplexityReport {
        total_files: 1,
        average_complexity: 15.0,
        hotspots: vec![hotspot],
    };

    assert_eq!(report.hotspots.len(), 1);
    assert_eq!(report.hotspots[0].cyclomatic_complexity, 15);
}

// === ComplexityHotspot tests ===

#[test]
fn test_complexity_hotspot_creation() {
    let hotspot = ComplexityHotspot {
        file_path: "lib.rs".to_string(),
        function_name: "calculate".to_string(),
        cyclomatic_complexity: 10,
        cognitive_complexity: 8,
    };

    assert_eq!(hotspot.file_path, "lib.rs");
    assert_eq!(hotspot.function_name, "calculate");
}

#[test]
fn test_complexity_hotspot_clone() {
    let hotspot = ComplexityHotspot {
        file_path: "test.rs".to_string(),
        function_name: "test_fn".to_string(),
        cyclomatic_complexity: 5,
        cognitive_complexity: 3,
    };

    let cloned = hotspot.clone();
    assert_eq!(hotspot.file_path, cloned.file_path);
    assert_eq!(hotspot.cyclomatic_complexity, cloned.cyclomatic_complexity);
}

// === DependencyGraphReport tests ===

#[test]
fn test_dependency_graph_report_creation() {
    let report = DependencyGraphReport {
        nodes: 50,
        edges: 100,
        circular_dependencies: vec![],
        mermaid_diagram: "graph TD".to_string(),
    };

    assert_eq!(report.nodes, 50);
    assert_eq!(report.edges, 100);
    assert!(report.circular_dependencies.is_empty());
}

#[test]
fn test_dependency_graph_with_cycles() {
    let report = DependencyGraphReport {
        nodes: 3,
        edges: 3,
        circular_dependencies: vec![vec!["A".to_string(), "B".to_string(), "C".to_string()]],
        mermaid_diagram: "".to_string(),
    };

    assert_eq!(report.circular_dependencies.len(), 1);
    assert_eq!(report.circular_dependencies[0].len(), 3);
}

// === DefectScore tests ===

#[test]
fn test_defect_score_creation() {
    let score = DefectScore {
        entity: "main.rs".to_string(),
        score: 0.75,
        confidence: 0.9,
        reasons: vec!["High complexity".to_string()],
    };

    assert_eq!(score.entity, "main.rs");
    assert_eq!(score.score, 0.75);
    assert_eq!(score.confidence, 0.9);
}

#[test]
fn test_defect_score_multiple_reasons() {
    let score = DefectScore {
        entity: "lib.rs".to_string(),
        score: 0.5,
        confidence: 0.8,
        reasons: vec![
            "High churn".to_string(),
            "Large function".to_string(),
            "Deep nesting".to_string(),
        ],
    };

    assert_eq!(score.reasons.len(), 3);
}

// === GraphMetricsReport tests ===

#[test]
fn test_graph_metrics_report_creation() {
    let report = GraphMetricsReport {
        centrality_scores: vec![],
        clustering_coefficient: 0.5,
        modularity: 0.7,
    };

    assert!(report.centrality_scores.is_empty());
    assert_eq!(report.clustering_coefficient, 0.5);
    assert_eq!(report.modularity, 0.7);
}

// === CentralityScore tests ===

#[test]
fn test_centrality_score_creation() {
    let score = CentralityScore {
        node: "main_module".to_string(),
        degree: 10.0,
        betweenness: 0.5,
        closeness: 0.8,
        pagerank: 0.1,
    };

    assert_eq!(score.node, "main_module");
    assert_eq!(score.degree, 10.0);
    assert_eq!(score.betweenness, 0.5);
}

#[test]
fn test_centrality_score_clone() {
    let score = CentralityScore {
        node: "test".to_string(),
        degree: 5.0,
        betweenness: 0.3,
        closeness: 0.6,
        pagerank: 0.05,
    };

    let cloned = score.clone();
    assert_eq!(score.node, cloned.node);
    assert_eq!(score.pagerank, cloned.pagerank);
}

// === AnalysisReport tests ===

#[test]
fn test_analysis_report_empty() {
    let report = AnalysisReport {
        duplicates: None,
        dead_code: None,
        complexity_metrics: None,
        dependency_graph: None,
        defect_predictions: None,
        graph_metrics: None,
        timestamp: Utc::now(),
    };

    assert!(report.duplicates.is_none());
    assert!(report.dead_code.is_none());
    assert!(report.complexity_metrics.is_none());
}

#[test]
fn test_analysis_report_with_complexity() {
    let complexity = ComplexityReport {
        total_files: 5,
        average_complexity: 7.5,
        hotspots: vec![],
    };

    let report = AnalysisReport {
        duplicates: None,
        dead_code: None,
        complexity_metrics: Some(complexity),
        dependency_graph: None,
        defect_predictions: None,
        graph_metrics: None,
        timestamp: Utc::now(),
    };

    assert!(report.complexity_metrics.is_some());
    assert_eq!(report.complexity_metrics.as_ref().unwrap().total_files, 5);
}

// === CodeIntelligence tests ===

#[tokio::test]
async fn test_code_intelligence_default() {
    let intel = CodeIntelligence::default();
    let (nodes, gen) = intel.get_dag_stats().await;

    assert_eq!(nodes, 0);
    assert_eq!(gen, 0);
}
