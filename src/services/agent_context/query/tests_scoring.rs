// Scoring edge case tests: empty corpus, no query terms, scoped scoring.
// Lines 690–746 of the original tests.rs.

#[test]
fn test_scoped_scoring_empty_candidates() {
    let index = build_test_index();
    let results = index
        .calculate_relevance_scores_scoped("test", &[])
        .unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_scoped_scoring_no_query_terms() {
    let index = build_test_index();
    // Only special chars = no query terms after tokenization
    let results = index
        .calculate_relevance_scores_scoped("!@#$%", &[0, 1])
        .unwrap();
    // Returns all candidates with equal score when no terms
    assert_eq!(results.len(), 2);
    assert!((results[0].1 - 1.0).abs() < 0.01);
}

#[test]
fn test_full_scoring_empty_corpus() {
    let index = AgentContextIndex {
        functions: vec![],
        name_index: HashMap::new(),
        file_index: HashMap::new(),
        corpus: vec![],
        corpus_lower: vec![],
        name_frequency: HashMap::new(),
        calls: HashMap::new(),
        called_by: HashMap::new(),
        graph_metrics: vec![],
        project_root: std::path::PathBuf::from("/test"),
        manifest: crate::services::agent_context::IndexManifest {
            version: "1.2.0".to_string(),
            built_at: "2025-01-01T00:00:00Z".to_string(),
            project_root: "/test".to_string(),
            function_count: 0,
            file_count: 0,
            languages: vec![],
            avg_tdg_score: 0.0,
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        },
        db_path: None,
        coverage_off_files: HashSet::new(),
    };
    let results = index.calculate_relevance_scores("test").unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_full_scoring_no_query_terms() {
    let index = build_test_index();
    let results = index.calculate_relevance_scores("!!!").unwrap();
    assert!(results.is_empty());
}
