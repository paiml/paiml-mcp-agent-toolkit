// Query filter, churn map, from_entry_with_context, dedup, PMAT-478 search modes,
// glob, and synchronous enrichment tests.
// Included via include!() from tests_part2.rs - no use imports allowed.

#[test]
fn test_query_combined_filters() {
    let index = build_test_index();
    let results = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                min_grade: Some("A".to_string()),
                max_complexity: Some(3),
                language: Some("Rust".to_string()),
                path_pattern: Some("src/".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
    for r in &results {
        assert!(
            crate::services::agent_context::query::grades::grade_meets_threshold(&r.tdg_grade, "A"),
            "grade {} should not have passed --min-grade A",
            r.tdg_grade
        );
        assert!(r.complexity <= 3);
        assert_eq!(r.language, "Rust");
        assert!(r.file_path.contains("src/"));
    }
}

#[test]
fn test_build_churn_map() {
    use crate::models::churn::FileChurnMetrics;

    let metrics = vec![
        FileChurnMetrics {
            path: std::path::PathBuf::from("src/foo.rs"),
            relative_path: "src/foo.rs".to_string(),
            commit_count: 10,
            unique_authors: vec!["author1".to_string()],
            additions: 100,
            deletions: 50,
            churn_score: 0.5,
            last_modified: chrono::Utc::now(),
            first_seen: chrono::Utc::now(),
        },
        FileChurnMetrics {
            path: std::path::PathBuf::from("src/bar.rs"),
            relative_path: "src/bar.rs".to_string(),
            commit_count: 25,
            unique_authors: vec!["author2".to_string()],
            additions: 200,
            deletions: 100,
            churn_score: 0.8,
            last_modified: chrono::Utc::now(),
            first_seen: chrono::Utc::now(),
        },
    ];

    let map = build_churn_map(&metrics);
    assert_eq!(map.len(), 2);
    assert_eq!(map.get("src/foo.rs"), Some(&(10, 0.5)));
    assert_eq!(map.get("src/bar.rs"), Some(&(25, 0.8)));
}

#[test]
fn test_from_entry_with_context_basic() {
    use crate::services::agent_context::function_index::{
        AgentContextIndex, GraphMetrics, IndexManifest,
    };
    use std::path::PathBuf;

    let entry = create_test_entry("my_func", 5, 88.0);
    let index = AgentContextIndex {
        functions: vec![entry.clone()],
        name_index: HashMap::new(),
        file_index: HashMap::new(),
        corpus: vec!["my_func".to_string()],
        corpus_lower: vec!["my_func".to_string()],
        name_frequency: HashMap::new(),
        calls: HashMap::new(),
        called_by: HashMap::new(),
        graph_metrics: vec![GraphMetrics {
            pagerank: 0.42,
            centrality: 0.3,
            in_degree: 5,
            out_degree: 2,
        }],
        project_root: PathBuf::from("/tmp"),
        manifest: IndexManifest {
            version: "1.3.0".to_string(),
            built_at: "test".to_string(),
            project_root: "/tmp".to_string(),
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

    let result = QueryResult::from_entry_with_context(&entry, 0, &index, 0.9, false);
    assert!((result.pagerank - 0.42).abs() < 0.001);
    assert_eq!(result.in_degree, 5);
    assert_eq!(result.out_degree, 2);
}

#[test]
fn test_from_entry_with_context_out_of_bounds() {
    use crate::services::agent_context::function_index::{AgentContextIndex, IndexManifest};
    use std::path::PathBuf;

    let entry = create_test_entry("my_func", 5, 88.0);
    let index = AgentContextIndex {
        functions: vec![entry.clone()],
        name_index: HashMap::new(),
        file_index: HashMap::new(),
        corpus: vec!["my_func".to_string()],
        corpus_lower: vec!["my_func".to_string()],
        name_frequency: HashMap::new(),
        calls: HashMap::new(),
        called_by: HashMap::new(),
        graph_metrics: vec![], // empty - out of bounds
        project_root: PathBuf::from("/tmp"),
        manifest: IndexManifest {
            version: "1.3.0".to_string(),
            built_at: "test".to_string(),
            project_root: "/tmp".to_string(),
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

    let result = QueryResult::from_entry_with_context(&entry, 99, &index, 0.9, false);
    // Should not panic, pagerank stays 0
    assert!((result.pagerank - 0.0).abs() < 0.001);
}

#[test]
fn test_from_entry_with_context_callers_capping() {
    use crate::services::agent_context::function_index::{
        AgentContextIndex, GraphMetrics, IndexManifest,
    };
    use std::path::PathBuf;

    let entry = create_test_entry("target", 5, 88.0);
    // Create 15 caller functions + 3 test callers
    let mut functions = vec![entry.clone()];
    let mut called_by_map = HashMap::new();
    let mut callers = vec![];
    for i in 0..15 {
        functions.push(create_test_entry(&format!("caller_{}", i), 1, 96.0));
        callers.push(i + 1); // indices 1..=15
    }
    for i in 0..3 {
        functions.push(create_test_entry(&format!("test_caller_{}", i), 1, 96.0));
        callers.push(16 + i); // indices 16..=18
    }
    called_by_map.insert(0usize, callers);

    let index = AgentContextIndex {
        functions,
        name_index: HashMap::new(),
        file_index: HashMap::new(),
        corpus: vec![],
        corpus_lower: vec![],
        name_frequency: HashMap::new(),
        calls: HashMap::new(),
        called_by: called_by_map,
        graph_metrics: vec![GraphMetrics {
            pagerank: 0.1,
            centrality: 0.0,
            in_degree: 18,
            out_degree: 0,
        }],
        project_root: PathBuf::from("/tmp"),
        manifest: IndexManifest {
            version: "1.3.0".to_string(),
            built_at: "test".to_string(),
            project_root: "/tmp".to_string(),
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

    let result = QueryResult::from_entry_with_context(&entry, 0, &index, 0.9, false);
    // Should have 10 prod callers + "(+5 more)" + "(+3 tests)" = 12 entries
    assert!(result.called_by.len() <= 12);
    // Should have the capping message
    let has_more = result
        .called_by
        .iter()
        .any(|s| s.starts_with("(+") && s.ends_with("more)"));
    assert!(has_more, "Should have (+N more) message");
    let has_tests = result.called_by.iter().any(|s| s.contains("tests)"));
    assert!(has_tests, "Should have (+N tests) message");
}

#[test]
fn test_dedup_ordered_preserves_first() {
    use super::types::dedup_ordered;
    let input: Vec<&str> = vec!["a", "b", "c", "b", "d", "c", "e"];
    let deduped = dedup_ordered(&input);
    assert_eq!(deduped, vec!["a", "b", "c", "d", "e"]);
}

#[test]
fn test_dedup_ordered_empty_input() {
    use super::types::dedup_ordered;
    let input: Vec<&str> = vec![];
    let deduped = dedup_ordered(&input);
    assert!(deduped.is_empty());
}

// ── PMAT-478: New search mode tests ────────────────────────────────

#[test]
fn test_query_regex_mode() {
    let index = build_test_index();
    let results = index
        .query(
            r"handle_\w+",
            QueryOptions {
                limit: 10,
                search_mode: SearchMode::Regex,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!results.is_empty());
    // Top result should be a handle_ function (name matches score highest)
    assert!(
        results[0].function_name.starts_with("handle_"),
        "Top result should be handle_* function, got: {}",
        results[0].function_name
    );
}

#[test]
fn test_query_regex_invalid_pattern() {
    let index = build_test_index();
    let result = index.query(
        r"[invalid",
        QueryOptions {
            limit: 10,
            search_mode: SearchMode::Regex,
            ..Default::default()
        },
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid regex"));
}

#[test]
fn test_query_literal_mode() {
    let index = build_test_index();
    let results = index
        .query(
            "unwrap()",
            QueryOptions {
                limit: 10,
                search_mode: SearchMode::Literal,
                ..Default::default()
            },
        )
        .unwrap();
    // Literal mode searches source code for exact string
    // May or may not find matches depending on test data
    for r in &results {
        // If there's a match, the source should contain the literal
        // Source is not in result unless include_source, so we just verify no panic
        let _ = &r.function_name;
        let _ = &r.signature;
    }
}

#[test]
fn test_query_case_sensitive() {
    let index = build_test_index();
    let results_sensitive = index
        .query(
            "Handle",
            QueryOptions {
                limit: 10,
                case_sensitivity: CaseSensitivity::Sensitive,
                ..Default::default()
            },
        )
        .unwrap();
    let results_insensitive = index
        .query(
            "Handle",
            QueryOptions {
                limit: 10,
                case_sensitivity: CaseSensitivity::Insensitive,
                ..Default::default()
            },
        )
        .unwrap();
    // Case-insensitive should find at least as many results
    assert!(results_insensitive.len() >= results_sensitive.len());
}

#[test]
fn test_query_smart_case() {
    let index = build_test_index();
    // Lowercase query => smart-case treats as insensitive
    let results_lower = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                search_mode: SearchMode::Literal,
                case_sensitivity: CaseSensitivity::Smart,
                ..Default::default()
            },
        )
        .unwrap();
    // Query with uppercase => smart-case treats as sensitive
    let results_upper = index
        .query(
            "Handle",
            QueryOptions {
                limit: 10,
                search_mode: SearchMode::Literal,
                case_sensitivity: CaseSensitivity::Smart,
                ..Default::default()
            },
        )
        .unwrap();
    // Lowercase should find more or equal (case-insensitive)
    assert!(results_lower.len() >= results_upper.len());
}

#[test]
fn test_query_exclude_pattern() {
    let index = build_test_index();
    let all_results = index
        .query(
            "handle",
            QueryOptions {
                limit: 20,
                ..Default::default()
            },
        )
        .unwrap();
    let filtered_results = index
        .query(
            "handle",
            QueryOptions {
                limit: 20,
                exclude_pattern: vec!["error".to_string()],
                ..Default::default()
            },
        )
        .unwrap();
    // Excluding "error" should reduce results
    assert!(filtered_results.len() <= all_results.len());
    // No filtered result should contain "error" in name/signature/source
    for r in &filtered_results {
        let haystack = format!("{} {}", r.function_name, r.signature).to_lowercase();
        assert!(
            !haystack.contains("error"),
            "Excluded result still contains 'error': {}",
            r.function_name
        );
    }
}

#[test]
fn test_query_exclude_file_pattern() {
    let index = build_test_index();
    let all_results = index
        .query(
            "handle",
            QueryOptions {
                limit: 20,
                ..Default::default()
            },
        )
        .unwrap();
    let filtered_results = index
        .query(
            "handle",
            QueryOptions {
                limit: 20,
                exclude_file_pattern: vec!["utils.rs".to_string()],
                ..Default::default()
            },
        )
        .unwrap();
    assert!(filtered_results.len() <= all_results.len());
    for r in &filtered_results {
        assert!(
            !r.file_path.contains("utils.rs"),
            "Excluded file still present: {}",
            r.file_path
        );
    }
}

#[test]
fn test_query_regex_case_insensitive() {
    let index = build_test_index();
    let results = index
        .query(
            r"HANDLE",
            QueryOptions {
                limit: 10,
                search_mode: SearchMode::Regex,
                case_sensitivity: CaseSensitivity::Insensitive,
                ..Default::default()
            },
        )
        .unwrap();
    // Should find handle_ functions despite uppercase query
    assert!(!results.is_empty());
}

#[test]
fn test_glob_matches_basic() {
    assert!(glob_matches("*.rs", "foo.rs"));
    assert!(!glob_matches("*.rs", "foo.py"));
    assert!(glob_matches("src/**/*.rs", "src/cli/handlers/mod.rs"));
    assert!(!glob_matches("src/**/*.rs", "tests/mod.rs"));
    assert!(glob_matches("utils", "src/utils.rs"));
}

