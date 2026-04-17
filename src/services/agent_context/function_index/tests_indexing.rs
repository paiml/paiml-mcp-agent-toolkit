// Index building, call graph construction, graph metrics, and index lookup tests

#[test]
fn test_build_indices() {
    let functions = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "foo".to_string(),
            signature: "fn foo()".to_string(),
            doc_comment: None,
            source: "fn foo() {}".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "abc".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "bar".to_string(),
            signature: "fn bar()".to_string(),
            doc_comment: None,
            source: "fn bar() {}".to_string(),
            start_line: 3,
            end_line: 3,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "def".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
    ];
    let indices = build_indices(&functions);
    assert_eq!(indices.name_index["foo"], vec![0]);
    assert_eq!(indices.name_index["bar"], vec![1]);
    assert_eq!(indices.file_index["a.rs"], vec![0, 1]);
    assert_eq!(indices.corpus.len(), 2);
}

#[test]
fn test_build_call_graph() {
    // foo calls bar (bar appears as identifier in foo's source)
    let functions = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "foo".to_string(),
            signature: "fn foo()".to_string(),
            doc_comment: None,
            source: "fn foo() { bar(); }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "abc".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "bar".to_string(),
            signature: "fn bar()".to_string(),
            doc_comment: None,
            source: "fn bar() { println!(\"hello\"); }".to_string(),
            start_line: 3,
            end_line: 3,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "def".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
    ];
    let indices = build_indices(&functions);
    let (calls, called_by) = build_call_graph(&functions, &indices.name_index);

    // foo calls bar
    assert!(calls.get(&0).is_some_and(|v| v.contains(&1)));
    // bar is called by foo
    assert!(called_by.get(&1).is_some_and(|v| v.contains(&0)));
    // bar does not call foo
    assert!(!calls.get(&1).is_some_and(|v| v.contains(&0)));
}

#[test]
fn test_get_calls_and_called_by() {
    let functions = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "caller".to_string(),
            signature: "fn caller()".to_string(),
            doc_comment: None,
            source: "fn caller() { callee(); }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "aaa".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "callee".to_string(),
            signature: "fn callee()".to_string(),
            doc_comment: None,
            source: "fn callee() { println!(\"hello\"); }".to_string(),
            start_line: 3,
            end_line: 3,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "bbb".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(), linked_definition: None,
        },
    ];

    let indices = build_indices(&functions);
    let corpus_lower: Vec<String> = indices.corpus.iter().map(|c| c.to_lowercase()).collect();
    let (calls, called_by) = build_call_graph(&functions, &indices.name_index);
    let graph_metrics = compute_graph_metrics(functions.len(), &calls, &called_by);

    let index = AgentContextIndex {
        functions,
        name_index: indices.name_index,
        file_index: indices.file_index,
        corpus: indices.corpus,
        corpus_lower,
        name_frequency: HashMap::new(),
        calls,
        called_by,
        graph_metrics,
        project_root: PathBuf::from("/test"),
        manifest: IndexManifest {
            version: "1.3.0".to_string(),
            built_at: "2025-01-01T00:00:00Z".to_string(),
            project_root: "/test".to_string(),
            function_count: 2,
            file_count: 1,
            languages: vec!["Rust".to_string()],
            avg_tdg_score: 0.0,
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        },
        db_path: None,
        coverage_off_files: HashSet::new(),
    };

    let calls_of_0 = index.get_calls(0);
    assert!(calls_of_0.contains(&"callee"), "caller should call callee");

    let called_by_1 = index.get_called_by(1);
    assert!(
        called_by_1.contains(&"caller"),
        "callee should be called by caller"
    );

    // Non-existent index
    assert!(index.get_calls(999).is_empty());
    assert!(index.get_called_by(999).is_empty());
}

#[test]
fn test_find_function_index() {
    let functions = vec![FunctionEntry {
        file_path: "a.rs".to_string(),
        function_name: "foo".to_string(),
        signature: "fn foo()".to_string(),
        doc_comment: None,
        source: "fn foo() {}".to_string(),
        start_line: 1,
        end_line: 1,
        language: "Rust".to_string(),
        quality: QualityMetrics::default(),
        checksum: "aaa".to_string(),
        definition_type: DefinitionType::default(),
        commit_count: 0,
        churn_score: 0.0,
        clone_count: 0,
        pattern_diversity: 0.0,
        fault_annotations: Vec::new(), linked_definition: None,
    }];

    let indices = build_indices(&functions);
    let corpus_lower: Vec<String> = indices.corpus.iter().map(|c| c.to_lowercase()).collect();

    let index = AgentContextIndex {
        functions,
        name_index: indices.name_index,
        file_index: indices.file_index,
        corpus: indices.corpus,
        corpus_lower,
        name_frequency: HashMap::new(),
        calls: HashMap::new(),
        called_by: HashMap::new(),
        graph_metrics: vec![GraphMetrics::default()],
        project_root: PathBuf::from("/test"),
        manifest: IndexManifest {
            version: "1.3.0".to_string(),
            built_at: "2025-01-01T00:00:00Z".to_string(),
            project_root: "/test".to_string(),
            function_count: 1,
            file_count: 1,
            languages: vec!["Rust".to_string()],
            avg_tdg_score: 0.0,
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        },
        db_path: None,
        coverage_off_files: HashSet::new(),
    };

    assert_eq!(index.find_function_index("a.rs", "foo"), Some(0));
    assert_eq!(index.find_function_index("a.rs", "bar"), None);
    assert_eq!(index.find_function_index("b.rs", "foo"), None);
}

#[test]
fn test_compute_graph_metrics_empty() {
    let metrics = compute_graph_metrics(0, &HashMap::new(), &HashMap::new());
    assert!(metrics.is_empty());
}

#[test]
fn test_compute_graph_metrics_isolated_nodes() {
    // No calls between nodes -> dangling node handling
    let metrics = compute_graph_metrics(3, &HashMap::new(), &HashMap::new());
    assert_eq!(metrics.len(), 3);
    // All nodes are dangling, PageRank should be uniform
    for m in &metrics {
        assert!(
            m.pagerank > 0.0,
            "isolated node should have positive pagerank"
        );
        assert_eq!(m.in_degree, 0);
        assert_eq!(m.out_degree, 0);
    }
    // PageRank should be approximately equal for all
    let diff = (metrics[0].pagerank - metrics[1].pagerank).abs();
    assert!(
        diff < 0.001,
        "isolated nodes should have near-equal pagerank"
    );
}

#[test]
fn test_compute_graph_metrics_chain() {
    // 0 -> 1 -> 2 (chain)
    let mut calls = HashMap::new();
    calls.insert(0, vec![1]);
    calls.insert(1, vec![2]);
    let mut called_by = HashMap::new();
    called_by.insert(1, vec![0]);
    called_by.insert(2, vec![1]);

    let metrics = compute_graph_metrics(3, &calls, &called_by);
    assert_eq!(metrics.len(), 3);
    // Node 2 (end of chain) should have highest PageRank (most "important" via link structure)
    assert!(
        metrics[2].pagerank > metrics[0].pagerank,
        "end of chain should have higher pagerank: {} vs {}",
        metrics[2].pagerank,
        metrics[0].pagerank
    );
    // In/out degree checks
    assert_eq!(metrics[0].out_degree, 1);
    assert_eq!(metrics[0].in_degree, 0);
    assert_eq!(metrics[1].in_degree, 1);
    assert_eq!(metrics[1].out_degree, 1);
    assert_eq!(metrics[2].in_degree, 1);
    assert_eq!(metrics[2].out_degree, 0);
}
