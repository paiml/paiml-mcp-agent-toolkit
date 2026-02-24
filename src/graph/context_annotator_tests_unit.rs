// ============================================================
// Test Fixtures
// ============================================================

/// Create a minimal test node
fn create_test_node(path: &str, complexity: f64) -> NodeData {
    NodeData {
        path: PathBuf::from(path),
        module: path.to_string(),
        symbols: vec![],
        loc: 100,
        complexity,
        ast_hash: 0,
    }
}

/// Create a simple graph with connected nodes
fn create_connected_graph() -> DependencyGraph {
    let mut graph = DependencyGraph::new();

    let n1 = graph.add_node(create_test_node("src/main.rs", 5.0));
    let n2 = graph.add_node(create_test_node("src/lib.rs", 12.0));
    let n3 = graph.add_node(create_test_node("src/utils.rs", 3.0));
    let n4 = graph.add_node(create_test_node("src/complex.rs", 25.0));

    // Create import edges
    graph.add_edge(
        n1,
        n2,
        EdgeData::Import {
            weight: 1.0,
            visibility: Visibility::Public,
        },
    );
    graph.add_edge(
        n1,
        n3,
        EdgeData::Import {
            weight: 0.5,
            visibility: Visibility::Public,
        },
    );
    graph.add_edge(
        n2,
        n3,
        EdgeData::Import {
            weight: 1.0,
            visibility: Visibility::Public,
        },
    );
    graph.add_edge(
        n4,
        n2,
        EdgeData::Import {
            weight: 2.0,
            visibility: Visibility::Public,
        },
    );

    graph
}

/// Create a graph with multiple edge types
fn create_multi_edge_graph() -> DependencyGraph {
    let mut graph = DependencyGraph::new();

    let n1 = graph.add_node(create_test_node("src/caller.rs", 8.0));
    let n2 = graph.add_node(create_test_node("src/callee.rs", 6.0));
    let n3 = graph.add_node(create_test_node("src/base.rs", 4.0));

    // Import edge
    graph.add_edge(
        n1,
        n2,
        EdgeData::Import {
            weight: 1.5,
            visibility: Visibility::Public,
        },
    );

    // Function call edge
    graph.add_edge(
        n1,
        n2,
        EdgeData::FunctionCall {
            count: 5,
            async_call: true,
        },
    );

    // Type dependency edge
    graph.add_edge(
        n2,
        n3,
        EdgeData::TypeDependency {
            strength: 0.8,
            kind: TypeKind::Struct,
        },
    );

    // Inheritance edge
    graph.add_edge(n1, n3, EdgeData::Inheritance { depth: 2 });

    // DataFlow edge
    graph.add_edge(
        n2,
        n1,
        EdgeData::DataFlow {
            confidence: 0.9,
            direction: FlowDirection::Backward,
        },
    );

    graph
}

// ============================================================
// GraphContextAnnotator Tests
// ============================================================

#[test]
fn test_annotator_default_values() {
    let annotator = GraphContextAnnotator::default();
    assert!((annotator.pagerank_threshold - 0.1).abs() < f64::EPSILON);
    assert!((annotator.community_relevance - 0.8).abs() < f64::EPSILON);
}

#[test]
fn test_annotator_new_equals_default() {
    let new = GraphContextAnnotator::new();
    let default = GraphContextAnnotator::default();

    assert!((new.pagerank_threshold - default.pagerank_threshold).abs() < f64::EPSILON);
    assert!((new.community_relevance - default.community_relevance).abs() < f64::EPSILON);
}

#[test]
fn test_annotate_context_empty_graph() {
    let annotator = GraphContextAnnotator::new();
    let graph = DependencyGraph::new();

    let annotations = annotator.annotate_context(&graph);
    assert!(annotations.is_empty());
}

#[test]
fn test_annotate_context_single_node() {
    let annotator = GraphContextAnnotator::new();
    let mut graph = DependencyGraph::new();
    graph.add_node(create_test_node("src/only.rs", 7.5));

    let annotations = annotator.annotate_context(&graph);
    assert_eq!(annotations.len(), 1);
    assert!(annotations[0].file_path.contains("only.rs"));
    assert_eq!(annotations[0].complexity_rank, "Medium");
}

#[test]
fn test_annotate_context_connected_graph() {
    let annotator = GraphContextAnnotator::new();
    let graph = create_connected_graph();

    let annotations = annotator.annotate_context(&graph);

    // Should have 4 annotations
    assert_eq!(annotations.len(), 4);

    // Annotations should be sorted by importance (descending)
    for i in 1..annotations.len() {
        assert!(
            annotations[i - 1].importance_score >= annotations[i].importance_score,
            "Annotations should be sorted by importance descending"
        );
    }
}

#[test]
fn test_annotate_context_has_related_files() {
    let annotator = GraphContextAnnotator::new();
    let graph = create_connected_graph();

    let annotations = annotator.annotate_context(&graph);

    // Main.rs should have related files (it has outgoing edges to lib.rs and utils.rs)
    let main_annotation = annotations.iter().find(|a| a.file_path.contains("main.rs"));
    assert!(main_annotation.is_some());

    let related = &main_annotation.unwrap().related_files;
    assert!(!related.is_empty(), "main.rs should have related files");
}

#[test]
fn test_annotate_context_community_assignment() {
    let annotator = GraphContextAnnotator::new();
    let graph = create_connected_graph();

    let annotations = annotator.annotate_context(&graph);

    // All nodes should have a community ID
    for annotation in &annotations {
        // community_id is usize, so it's always >= 0
        // Just verify the field exists and is accessible
        let _ = annotation.community_id;
    }
}

// ============================================================
// Complexity Classification Tests
// ============================================================

#[test]
fn test_classify_complexity_low() {
    let annotator = GraphContextAnnotator::new();
    assert_eq!(annotator.classify_complexity(0.0), "Low");
    assert_eq!(annotator.classify_complexity(4.9), "Low");
    assert_eq!(annotator.classify_complexity(1.0), "Low");
}

#[test]
fn test_classify_complexity_medium() {
    let annotator = GraphContextAnnotator::new();
    assert_eq!(annotator.classify_complexity(5.0), "Medium");
    assert_eq!(annotator.classify_complexity(7.5), "Medium");
    assert_eq!(annotator.classify_complexity(9.9), "Medium");
}

#[test]
fn test_classify_complexity_high() {
    let annotator = GraphContextAnnotator::new();
    assert_eq!(annotator.classify_complexity(10.0), "High");
    assert_eq!(annotator.classify_complexity(15.0), "High");
    assert_eq!(annotator.classify_complexity(19.9), "High");
}

#[test]
fn test_classify_complexity_very_high() {
    let annotator = GraphContextAnnotator::new();
    assert_eq!(annotator.classify_complexity(20.0), "Very High");
    assert_eq!(annotator.classify_complexity(50.0), "Very High");
    assert_eq!(annotator.classify_complexity(100.0), "Very High");
}

// ============================================================
// High Importance Files Tests
// ============================================================

#[test]
fn test_get_high_importance_files_empty() {
    let annotator = GraphContextAnnotator::new();
    let annotations: Vec<ContextAnnotation> = vec![];

    let high_importance = annotator.get_high_importance_files(&annotations);
    assert!(high_importance.is_empty());
}

#[test]
fn test_get_high_importance_files_all_below_threshold() {
    let annotator = GraphContextAnnotator::new(); // threshold = 0.1

    let annotations = vec![
        ContextAnnotation {
            file_path: "low1.rs".to_string(),
            importance_score: 0.05,
            community_id: 0,
            related_files: vec![],
            complexity_rank: "Low".to_string(),
        },
        ContextAnnotation {
            file_path: "low2.rs".to_string(),
            importance_score: 0.09,
            community_id: 0,
            related_files: vec![],
            complexity_rank: "Low".to_string(),
        },
    ];

    let high_importance = annotator.get_high_importance_files(&annotations);
    assert!(high_importance.is_empty());
}

#[test]
fn test_get_high_importance_files_some_above_threshold() {
    let annotator = GraphContextAnnotator::new(); // threshold = 0.1

    let annotations = vec![
        ContextAnnotation {
            file_path: "high.rs".to_string(),
            importance_score: 0.5,
            community_id: 0,
            related_files: vec![],
            complexity_rank: "High".to_string(),
        },
        ContextAnnotation {
            file_path: "low.rs".to_string(),
            importance_score: 0.05,
            community_id: 0,
            related_files: vec![],
            complexity_rank: "Low".to_string(),
        },
        ContextAnnotation {
            file_path: "medium.rs".to_string(),
            importance_score: 0.2,
            community_id: 0,
            related_files: vec![],
            complexity_rank: "Medium".to_string(),
        },
    ];

    let high_importance = annotator.get_high_importance_files(&annotations);
    assert_eq!(high_importance.len(), 2);
    assert!(high_importance.contains(&"high.rs".to_string()));
    assert!(high_importance.contains(&"medium.rs".to_string()));
    assert!(!high_importance.contains(&"low.rs".to_string()));
}

#[test]
fn test_get_high_importance_files_at_threshold() {
    let annotator = GraphContextAnnotator::new(); // threshold = 0.1

    let annotations = vec![ContextAnnotation {
        file_path: "exactly_at.rs".to_string(),
        importance_score: 0.1, // exactly at threshold (not > threshold)
        community_id: 0,
        related_files: vec![],
        complexity_rank: "Low".to_string(),
    }];

    let high_importance = annotator.get_high_importance_files(&annotations);
    // 0.1 is NOT > 0.1, so it should be empty
    assert!(high_importance.is_empty());
}

// ============================================================
// Community Clusters Tests
// ============================================================

#[test]
fn test_get_community_clusters_empty() {
    let annotator = GraphContextAnnotator::new();
    let annotations: Vec<ContextAnnotation> = vec![];

    let clusters = annotator.get_community_clusters(&annotations);
    assert!(clusters.is_empty());
}

#[test]
fn test_get_community_clusters_single_community() {
    let annotator = GraphContextAnnotator::new();

    let annotations = vec![
        ContextAnnotation {
            file_path: "a.rs".to_string(),
            importance_score: 0.5,
            community_id: 0,
            related_files: vec![],
            complexity_rank: "Low".to_string(),
        },
        ContextAnnotation {
            file_path: "b.rs".to_string(),
            importance_score: 0.3,
            community_id: 0,
            related_files: vec![],
            complexity_rank: "Low".to_string(),
        },
        ContextAnnotation {
            file_path: "c.rs".to_string(),
            importance_score: 0.2,
            community_id: 0,
            related_files: vec![],
            complexity_rank: "Low".to_string(),
        },
    ];

    let clusters = annotator.get_community_clusters(&annotations);
    assert_eq!(clusters.len(), 1);
    assert!(clusters.contains_key(&0));
    assert_eq!(clusters.get(&0).unwrap().len(), 3);
}

#[test]
fn test_get_community_clusters_multiple_communities() {
    let annotator = GraphContextAnnotator::new();

    let annotations = vec![
        ContextAnnotation {
            file_path: "a1.rs".to_string(),
            importance_score: 0.5,
            community_id: 0,
            related_files: vec![],
            complexity_rank: "Low".to_string(),
        },
        ContextAnnotation {
            file_path: "a2.rs".to_string(),
            importance_score: 0.3,
            community_id: 0,
            related_files: vec![],
            complexity_rank: "Low".to_string(),
        },
        ContextAnnotation {
            file_path: "b1.rs".to_string(),
            importance_score: 0.4,
            community_id: 1,
            related_files: vec![],
            complexity_rank: "Medium".to_string(),
        },
        ContextAnnotation {
            file_path: "c1.rs".to_string(),
            importance_score: 0.2,
            community_id: 2,
            related_files: vec![],
            complexity_rank: "High".to_string(),
        },
    ];

    let clusters = annotator.get_community_clusters(&annotations);
    assert_eq!(clusters.len(), 3);
    assert_eq!(clusters.get(&0).unwrap().len(), 2);
    assert_eq!(clusters.get(&1).unwrap().len(), 1);
    assert_eq!(clusters.get(&2).unwrap().len(), 1);
}

// ============================================================
// ContextAnnotation Struct Tests
// ============================================================

#[test]
fn test_context_annotation_clone() {
    let annotation = ContextAnnotation {
        file_path: "test.rs".to_string(),
        importance_score: 0.5,
        community_id: 1,
        related_files: vec!["related.rs".to_string()],
        complexity_rank: "Medium".to_string(),
    };

    let cloned = annotation.clone();
    assert_eq!(cloned.file_path, annotation.file_path);
    assert!((cloned.importance_score - annotation.importance_score).abs() < f64::EPSILON);
    assert_eq!(cloned.community_id, annotation.community_id);
    assert_eq!(cloned.related_files, annotation.related_files);
    assert_eq!(cloned.complexity_rank, annotation.complexity_rank);
}

#[test]
fn test_context_annotation_debug() {
    let annotation = ContextAnnotation {
        file_path: "test.rs".to_string(),
        importance_score: 0.5,
        community_id: 1,
        related_files: vec![],
        complexity_rank: "Low".to_string(),
    };

    let debug = format!("{:?}", annotation);
    assert!(debug.contains("test.rs"));
    assert!(debug.contains("0.5"));
}

// ============================================================
// GraphContextAnnotator Clone Tests
// ============================================================

#[test]
fn test_annotator_clone() {
    let annotator = GraphContextAnnotator {
        pagerank_threshold: 0.2,
        community_relevance: 0.9,
    };

    let cloned = annotator.clone();
    assert!((cloned.pagerank_threshold - 0.2).abs() < f64::EPSILON);
    assert!((cloned.community_relevance - 0.9).abs() < f64::EPSILON);
}

#[test]
fn test_annotator_debug() {
    let annotator = GraphContextAnnotator::new();
    let debug = format!("{:?}", annotator);
    assert!(debug.contains("GraphContextAnnotator"));
    assert!(debug.contains("pagerank_threshold"));
}
