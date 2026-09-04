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
            tdg_scale: crate::services::agent_context::TDG_SCALE.to_string(),
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
            run_counter: 0,
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

#[test]
fn test_rank_order_breaks_ties_by_file_path_and_start_line() {
    // PMAT-665: equal scores used to come back in candidate-collection order,
    // which is the order the filesystem walk produced — so two checkouts of one
    // commit answered the same query differently and `--limit N` returned
    // different functions. The tie-break is a property of the tree.
    let mut later = create_test_entry("tied_b", 1, 90.0);
    later.file_path = "src/zz.rs".to_string();
    later.start_line = 5;
    let mut earlier = create_test_entry("tied_a", 1, 90.0);
    earlier.file_path = "src/aa.rs".to_string();
    earlier.start_line = 99;
    let mut same_file_later = create_test_entry("tied_c", 1, 90.0);
    same_file_later.file_path = "src/aa.rs".to_string();
    same_file_later.start_line = 100;

    let index = AgentContextIndex {
        functions: vec![later, earlier, same_file_later],
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
            version: "1.4.0".to_string(),
            built_at: "2025-01-01T00:00:00Z".to_string(),
            project_root: "/test".to_string(),
            function_count: 3,
            file_count: 2,
            languages: vec![],
            avg_tdg_score: 0.0,
            tdg_scale: crate::services::agent_context::TDG_SCALE.to_string(),
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
            run_counter: 0,
        },
        db_path: None,
        coverage_off_files: HashSet::new(),
    };

    let expected = vec![1usize, 2, 0]; // aa.rs:99, aa.rs:100, zz.rs:5
    for input in [vec![0usize, 1, 2], vec![2usize, 1, 0], vec![1usize, 0, 2]] {
        let mut ranked: Vec<(usize, f32)> = input.iter().map(|&i| (i, 0.5f32)).collect();
        ranked.sort_by(|a, b| index.rank_order(a, b));
        let order: Vec<usize> = ranked.iter().map(|(i, _)| *i).collect();
        assert_eq!(
            order, expected,
            "tied entries must come out in (file_path, start_line) order, whatever order they went in ({input:?})"
        );
    }

    // A higher score still wins: the tie-break is secondary, not primary.
    let mut mixed = [(1usize, 0.1f32), (0usize, 0.9f32)];
    mixed.sort_by(|a, b| index.rank_order(a, b));
    assert_eq!(mixed[0].0, 0, "score descending remains the primary key");
}
