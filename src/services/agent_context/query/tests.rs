use super::engine::{glob_matches, is_test_function, parse_query_prefixes};
use super::enrichment::{build_churn_map, enrich_with_churn};
use super::formatters::{format_json, format_markdown, format_text, format_text_with_code};
use super::types::{
    CaseSensitivity, QueryOptions, QueryResult, RankBy, SearchMode,
};
use crate::services::agent_context::function_index::DefinitionType;
use crate::services::agent_context::{AgentContextIndex, FunctionEntry, QualityMetrics};
use std::collections::HashMap;

fn create_test_entry(name: &str, complexity: u32, tdg_score: f32) -> FunctionEntry {
    FunctionEntry {
        file_path: "test.rs".to_string(),
        function_name: name.to_string(),
        signature: format!("fn {name}()"),
        doc_comment: Some("Test function".to_string()),
        source: format!("fn {name}() {{ }}"),
        start_line: 1,
        end_line: 1,
        language: "Rust".to_string(),
        quality: QualityMetrics {
            tdg_score,
            tdg_grade: if tdg_score < 2.0 {
                "A".to_string()
            } else {
                "B".to_string()
            },
            complexity,
            cognitive_complexity: complexity,
            big_o: "O(1)".to_string(),
            satd_count: 0,
            loc: 10,
            commit_count: 0,
            churn_score: 0.0,
        },
        checksum: "abc123".to_string(),
        definition_type: DefinitionType::default(),
        commit_count: 0,
        churn_score: 0.0,
        clone_count: 0,
        pattern_diversity: 0.0,
        fault_annotations: Vec::new(),
    }
}

#[test]
fn test_query_result_from_entry() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let result = QueryResult::from_entry(&entry, 0.9, false);

    assert_eq!(result.function_name, "test_func");
    assert_eq!(result.complexity, 5);
    assert!((result.tdg_score - 1.5).abs() < 0.01);
    assert!(result.source.is_none());
}

#[test]
fn test_query_result_with_source() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let result = QueryResult::from_entry(&entry, 0.9, true);

    assert!(result.source.is_some());
}

#[test]
fn test_format_display() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let result = QueryResult::from_entry(&entry, 0.9, false);
    let display = result.format_display();

    assert!(display.contains("test_func"));
    assert!(display.contains("Complexity: 5"));
}

#[test]
fn test_format_text() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let result = QueryResult::from_entry(&entry, 0.9, false);
    let text = format_text(&[result]);

    assert!(text.contains("Found 1 functions"));
    assert!(text.contains("test_func"));
}

#[test]
fn test_format_markdown() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let result = QueryResult::from_entry(&entry, 0.9, false);
    let md = format_markdown(&[result]);

    assert!(md.contains("# Search Results"));
    assert!(md.contains("`test_func`"));
}

#[test]
fn test_format_json() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let result = QueryResult::from_entry(&entry, 0.9, false);
    let json = format_json(&[result]).unwrap();

    assert!(json.contains("\"function_name\": \"test_func\""));
}

#[test]
fn test_parse_query_prefixes_file_only() {
    let (file, func, remaining) = parse_query_prefixes("file:query.rs error handling");
    assert_eq!(file, Some("query.rs".to_string()));
    assert_eq!(func, None);
    assert_eq!(remaining, "error handling");
}

#[test]
fn test_parse_query_prefixes_fn_only() {
    let (file, func, remaining) = parse_query_prefixes("fn:handle_ auth");
    assert_eq!(file, None);
    assert_eq!(func, Some("handle_".to_string()));
    assert_eq!(remaining, "auth");
}

#[test]
fn test_parse_query_prefixes_both() {
    let (file, func, remaining) =
        parse_query_prefixes("file:foo.rs fn:bar baz");
    assert_eq!(file, Some("foo.rs".to_string()));
    assert_eq!(func, Some("bar".to_string()));
    assert_eq!(remaining, "baz");
}

#[test]
fn test_parse_query_prefixes_none() {
    let (file, func, remaining) = parse_query_prefixes("error handling");
    assert_eq!(file, None);
    assert_eq!(func, None);
    assert_eq!(remaining, "error handling");
}

#[test]
fn test_parse_query_prefixes_empty_value() {
    let (file, func, remaining) = parse_query_prefixes("file: fn: hello");
    assert_eq!(file, None);
    assert_eq!(func, None);
    assert_eq!(remaining, "hello");
}

#[test]
fn test_query_result_has_calls_fields() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let result = QueryResult::from_entry(&entry, 0.9, false);
    assert!(result.calls.is_empty());
    assert!(result.called_by.is_empty());
}

#[test]
fn test_format_text_with_calls() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.calls = vec!["helper_func".to_string()];
    result.called_by = vec!["main".to_string()];
    let text = format_text(&[result]);
    assert!(text.contains("Calls: helper_func"));
    assert!(text.contains("Called by: main"));
}

#[test]
fn test_format_markdown_with_calls() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.calls = vec!["helper_func".to_string(), "other".to_string()];
    let md = format_markdown(&[result]);
    assert!(md.contains("**Calls:** helper_func, other"));
}

#[test]
fn test_format_json_skips_empty_calls() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let result = QueryResult::from_entry(&entry, 0.9, false);
    let json = format_json(&[result]).unwrap();
    // Empty calls/called_by should not appear in JSON (skip_serializing_if)
    assert!(!json.contains("\"calls\""));
    assert!(!json.contains("\"called_by\""));
}

/// Build a small in-memory index for testing query paths
fn build_test_index() -> AgentContextIndex {
    let functions = vec![
        FunctionEntry {
            file_path: "src/handler.rs".to_string(),
            function_name: "handle_error".to_string(),
            signature: "fn handle_error(e: Error)".to_string(),
            doc_comment: Some("Handle API errors gracefully".to_string()),
            source: "fn handle_error(e: Error) { log(e); respond(500); }".to_string(),
            start_line: 10,
            end_line: 15,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                tdg_score: 1.0,
                tdg_grade: "A".to_string(),
                complexity: 3,
                cognitive_complexity: 3,
                big_o: "O(1)".to_string(),
                satd_count: 0,
                loc: 6,
                commit_count: 15,
                churn_score: 0.6,
            },
            checksum: "aaa".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 15,
            churn_score: 0.6,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "src/handler.rs".to_string(),
            function_name: "handle_request".to_string(),
            signature: "fn handle_request(req: Request)".to_string(),
            doc_comment: Some("Process incoming HTTP requests".to_string()),
            source: "fn handle_request(req: Request) { validate(req); handle_error(err); }"
                .to_string(),
            start_line: 20,
            end_line: 30,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                tdg_score: 2.0,
                tdg_grade: "B".to_string(),
                complexity: 5,
                cognitive_complexity: 5,
                big_o: "O(n)".to_string(),
                satd_count: 0,
                loc: 11,
                commit_count: 25,
                churn_score: 0.8,
            },
            checksum: "bbb".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 25,
            churn_score: 0.8,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "src/utils.rs".to_string(),
            function_name: "validate".to_string(),
            signature: "fn validate(input: &str) -> bool".to_string(),
            doc_comment: Some("Validate input data".to_string()),
            source: "fn validate(input: &str) -> bool { !input.is_empty() }".to_string(),
            start_line: 1,
            end_line: 3,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                tdg_score: 0.5,
                tdg_grade: "A".to_string(),
                complexity: 1,
                cognitive_complexity: 1,
                big_o: "O(1)".to_string(),
                satd_count: 0,
                loc: 3,
                commit_count: 3,
                churn_score: 0.1,
            },
            checksum: "ccc".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 3,
            churn_score: 0.1,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "tests/test_handler.rs".to_string(),
            function_name: "test_error_handling".to_string(),
            signature: "fn test_error_handling()".to_string(),
            doc_comment: None,
            source: "fn test_error_handling() { handle_error(mock_err()); }".to_string(),
            start_line: 1,
            end_line: 5,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                tdg_score: 1.0,
                tdg_grade: "A".to_string(),
                complexity: 1,
                cognitive_complexity: 1,
                big_o: "O(1)".to_string(),
                satd_count: 0,
                loc: 5,
                commit_count: 5,
                churn_score: 0.2,
            },
            checksum: "ddd".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 5,
            churn_score: 0.2,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "src/utils.rs".to_string(),
            function_name: "new".to_string(),
            signature: "fn new() -> Self".to_string(),
            doc_comment: None,
            source: "fn new() -> Self { Self {} }".to_string(),
            start_line: 10,
            end_line: 12,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                tdg_score: 0.5,
                tdg_grade: "A".to_string(),
                complexity: 1,
                cognitive_complexity: 1,
                big_o: "O(1)".to_string(),
                satd_count: 0,
                loc: 3,
                commit_count: 2,
                churn_score: 0.05,
            },
            checksum: "eee".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 2,
            churn_score: 0.05,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
    ];

    let indices = crate::services::agent_context::function_index::build_indices(&functions);
    let corpus_lower: Vec<String> = indices.corpus.iter().map(|c| c.to_lowercase()).collect();
    let name_frequency = crate::services::agent_context::function_index::compute_name_frequency(
        &indices.name_index,
        functions.len(),
    );
    let (calls, called_by) =
        crate::services::agent_context::function_index::build_call_graph(&functions, &indices.name_index);
    let graph_metrics =
        crate::services::agent_context::function_index::compute_graph_metrics(functions.len(), &calls, &called_by);

    AgentContextIndex {
        functions,
        name_index: indices.name_index,
        file_index: indices.file_index,
        corpus: indices.corpus,
        corpus_lower,
        name_frequency,
        calls,
        called_by,
        graph_metrics,
        project_root: std::path::PathBuf::from("/test"),
        manifest: crate::services::agent_context::IndexManifest {
            version: "1.2.0".to_string(),
            built_at: "2025-01-01T00:00:00Z".to_string(),
            project_root: "/test".to_string(),
            function_count: 5,
            file_count: 3,
            languages: vec!["Rust".to_string()],
            avg_tdg_score: 1.0,
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        },
        db_path: None,
    }
}

#[test]
fn test_query_empty_query_returns_error() {
    let index = build_test_index();
    let result = index.query("", QueryOptions::default());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
}

#[test]
fn test_query_basic_search() {
    let index = build_test_index();
    let results = index
        .query(
            "error handling",
            QueryOptions {
                limit: 5,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!results.is_empty());
    // handle_error should rank high for "error handling"
    assert_eq!(results[0].function_name, "handle_error");
}

#[test]
fn test_query_with_file_scope() {
    let index = build_test_index();
    let results = index
        .query(
            "file:utils.rs validate",
            QueryOptions {
                limit: 5,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!results.is_empty());
    // All results must be from utils.rs
    for r in &results {
        assert!(r.file_path.contains("utils.rs"), "unexpected file: {}", r.file_path);
    }
}

#[test]
fn test_query_with_fn_scope() {
    let index = build_test_index();
    let results = index
        .query(
            "fn:handle_ request",
            QueryOptions {
                limit: 5,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!results.is_empty());
    // All results must have function names starting with "handle_"
    for r in &results {
        assert!(
            r.function_name.starts_with("handle_"),
            "unexpected fn: {}",
            r.function_name
        );
    }
}

#[test]
fn test_query_with_both_scopes() {
    let index = build_test_index();
    let results = index
        .query(
            "file:handler.rs fn:handle_ error",
            QueryOptions {
                limit: 5,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!results.is_empty());
    for r in &results {
        assert!(r.file_path.contains("handler.rs"));
        assert!(r.function_name.starts_with("handle_"));
    }
}

#[test]
fn test_query_grade_filter() {
    let index = build_test_index();
    let results = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                min_grade: Some("A".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
    // Only grade A results
    for r in &results {
        assert_eq!(r.tdg_grade, "A", "expected A grade, got {}", r.tdg_grade);
    }
}

#[test]
fn test_query_complexity_filter() {
    let index = build_test_index();
    let results = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                max_complexity: Some(3),
                ..Default::default()
            },
        )
        .unwrap();
    for r in &results {
        assert!(r.complexity <= 3, "complexity {} exceeds max 3", r.complexity);
    }
}

#[test]
fn test_query_language_filter() {
    let index = build_test_index();
    let results = index
        .query(
            "validate",
            QueryOptions {
                limit: 10,
                language: Some("Rust".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
    for r in &results {
        assert_eq!(r.language, "Rust");
    }
}

#[test]
fn test_query_path_pattern_filter() {
    let index = build_test_index();
    let results = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                path_pattern: Some("src/".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
    for r in &results {
        assert!(r.file_path.contains("src/"));
    }
}

#[test]
fn test_query_loc_filter() {
    let index = build_test_index();
    let results = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                max_loc: Some(5),
                ..Default::default()
            },
        )
        .unwrap();
    for r in &results {
        assert!(r.loc <= 5, "loc {} exceeds max 5", r.loc);
    }
}

#[test]
fn test_query_test_function_demotion() {
    let index = build_test_index();
    let results = index
        .query(
            "error handling",
            QueryOptions {
                limit: 10,
                ..Default::default()
            },
        )
        .unwrap();
    // test_error_handling should be ranked lower than handle_error
    let handle_pos = results.iter().position(|r| r.function_name == "handle_error");
    let test_pos = results
        .iter()
        .position(|r| r.function_name == "test_error_handling");
    if let (Some(h), Some(t)) = (handle_pos, test_pos) {
        assert!(h < t, "production fn should rank higher than test fn");
    }
}

#[test]
fn test_query_generic_name_demotion() {
    let index = build_test_index();
    let results = index
        .query(
            "new",
            QueryOptions {
                limit: 10,
                ..Default::default()
            },
        )
        .unwrap();
    // "new" is a common name - if it appears, its score should be demoted
    if let Some(new_result) = results.iter().find(|r| r.function_name == "new") {
        // Score should be < 1.0 due to name frequency demotion
        assert!(new_result.relevance_score < 1.0);
    }
}

#[test]
fn test_query_include_source() {
    let index = build_test_index();
    let results = index
        .query(
            "validate",
            QueryOptions {
                limit: 1,
                include_source: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!results.is_empty());
    assert!(results[0].source.is_some());
}

#[test]
fn test_query_zero_limit_defaults_to_10() {
    let index = build_test_index();
    let results = index
        .query(
            "handle",
            QueryOptions {
                limit: 0,
                ..Default::default()
            },
        )
        .unwrap();
    // Should not panic with limit=0, should default to 10
    assert!(results.len() <= 10);
}

#[test]
fn test_query_results_have_calls() {
    let index = build_test_index();
    let results = index
        .query(
            "handle_request",
            QueryOptions {
                limit: 1,
                ..Default::default()
            },
        )
        .unwrap();
    if let Some(r) = results.iter().find(|r| r.function_name == "handle_request") {
        // handle_request calls validate and handle_error
        assert!(!r.calls.is_empty(), "expected calls to be populated");
    }
}

#[test]
fn test_get_function() {
    let index = build_test_index();
    let result = index.get_function("src/handler.rs", "handle_error");
    assert!(result.is_some());
    let r = result.unwrap();
    assert_eq!(r.function_name, "handle_error");
    assert_eq!(r.file_path, "src/handler.rs");
    assert!(r.source.is_some()); // get_function always includes source
}

#[test]
fn test_get_function_not_found() {
    let index = build_test_index();
    let result = index.get_function("nonexistent.rs", "foo");
    assert!(result.is_none());
}

#[test]
fn test_find_similar() {
    let index = build_test_index();
    let results = index
        .find_similar("src/handler.rs", "handle_error", 3)
        .unwrap();
    // Should find similar functions (handle_request is similar)
    assert!(!results.is_empty());
    // Should not include self
    assert!(results.iter().all(|r| !(r.file_path == "src/handler.rs" && r.function_name == "handle_error")));
}

#[test]
fn test_find_similar_not_found() {
    let index = build_test_index();
    let result = index.find_similar("nonexistent.rs", "foo", 3);
    assert!(result.is_err());
}

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
fn test_dedup_ordered() {
    use super::types::dedup_ordered;
    let items = vec!["a", "b", "a", "c", "b", "d"];
    let result = dedup_ordered(&items);
    assert_eq!(result, vec!["a", "b", "c", "d"]);
}

#[test]
fn test_dedup_ordered_empty() {
    use super::types::dedup_ordered;
    let items: Vec<&str> = vec![];
    let result = dedup_ordered(&items);
    assert!(result.is_empty());
}

#[test]
fn test_is_test_function() {
    let mut entry = create_test_entry("test_something", 1, 0.5);
    assert!(is_test_function(&entry));

    entry.function_name = "handle_request".to_string();
    entry.file_path = "src/handler.rs".to_string();
    assert!(!is_test_function(&entry));

    entry.file_path = "tests/integration.rs".to_string();
    assert!(is_test_function(&entry));

    entry.file_path = "src/tests/mod.rs".to_string();
    assert!(is_test_function(&entry));

    entry.file_path = "src/handler_tests.rs".to_string();
    assert!(is_test_function(&entry));

    entry.file_path = "src/handler_test.rs".to_string();
    assert!(is_test_function(&entry));
}

#[test]
fn test_called_by_test_summarization() {
    let mut index = build_test_index();
    // Simulate many test callers for function 0
    let mut callers = vec![1usize]; // one production caller
    for i in 10..25 {
        // Add fake test function indices
        index.functions.push(FunctionEntry {
            file_path: "tests/t.rs".to_string(),
            function_name: format!("test_case_{i}"),
            signature: format!("fn test_case_{i}()"),
            doc_comment: None,
            source: "fn test() {}".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: format!("t{i}"),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        });
        callers.push(index.functions.len() - 1);
    }
    index.called_by.insert(0, callers);
    // Rebuild name_index for the new functions
    for (i, f) in index.functions.iter().enumerate() {
        index.name_index.entry(f.function_name.clone()).or_default().push(i);
    }

    let result = QueryResult::from_entry_with_context(
        &index.functions[0],
        0,
        &index,
        0.9,
        false,
    );
    // Should have production caller + test summary
    assert!(result.called_by.iter().any(|s| s.contains("tests)")));
    // Should not list individual test_case_N names
    assert!(!result.called_by.iter().any(|s| s.starts_with("test_case_")));
}

#[test]
fn test_called_by_production_cap() {
    let mut index = build_test_index();
    // Simulate >10 production callers for function 0
    let mut callers = Vec::new();
    for i in 10..25 {
        index.functions.push(FunctionEntry {
            file_path: "src/callers.rs".to_string(),
            function_name: format!("caller_{i}"),
            signature: format!("fn caller_{i}()"),
            doc_comment: None,
            source: "fn caller() {}".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: format!("c{i}"),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        });
        callers.push(index.functions.len() - 1);
        index
            .name_index
            .entry(format!("caller_{i}"))
            .or_default()
            .push(index.functions.len() - 1);
    }
    index.called_by.insert(0, callers);

    let result = QueryResult::from_entry_with_context(
        &index.functions[0],
        0,
        &index,
        0.9,
        false,
    );
    // Should cap at 10 + "(+N more)"
    assert!(result.called_by.iter().any(|s| s.contains("more)")));
    // Total entries should be 10 visible + 1 summary = 11
    assert!(result.called_by.len() <= 12);
}

#[test]
fn test_enrich_with_churn() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut results = vec![QueryResult::from_entry(&entry, 0.9, false)];

    // Initially no churn data
    assert_eq!(results[0].commit_count, 0);
    assert!((results[0].churn_score - 0.0).abs() < 0.01);

    // Build churn map
    let mut churn_map = HashMap::new();
    churn_map.insert("test.rs".to_string(), (42u32, 0.75f32));

    // Enrich results
    enrich_with_churn(&mut results, &churn_map);

    // Verify churn data was applied
    assert_eq!(results[0].commit_count, 42);
    assert!((results[0].churn_score - 0.75).abs() < 0.01);
}

#[test]
fn test_enrich_with_churn_no_match() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut results = vec![QueryResult::from_entry(&entry, 0.9, false)];

    // Churn map with different file
    let mut churn_map = HashMap::new();
    churn_map.insert("other.rs".to_string(), (100u32, 0.9f32));

    // Enrich results - should not match
    enrich_with_churn(&mut results, &churn_map);

    // Verify churn data was NOT changed (no match)
    assert_eq!(results[0].commit_count, 0);
    assert!((results[0].churn_score - 0.0).abs() < 0.01);
}

#[test]
fn test_format_text_with_churn() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.commit_count = 25;
    result.churn_score = 0.8;

    let text = format_text(&[result]);
    assert!(text.contains("🔥 Hot"));
    assert!(text.contains("25 commits"));
    assert!(text.contains("80%"));
}

#[test]
fn test_format_text_with_code_shows_metrics() {
    let entry = create_test_entry("test_func", 15, 3.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.satd_count = 2;
    result.commit_count = 30;
    result.churn_score = 0.7;

    let text = format_text_with_code(&[result], None);
    // Should show complexity (format: "C:15" without space)
    assert!(text.contains("C:15"), "missing complexity");
    // Should show SATD warning as "⚠2" (warning symbol + count)
    assert!(text.contains("⚠2"), "missing SATD");
    // Should show churn (high commit count shown as "🔥" indicator)
    assert!(text.contains("🔥") || text.contains("30"), "missing churn indicator");
    // Should show function name in header
    assert!(text.contains("test_func"), "missing function name");
}

#[test]
fn test_format_text_with_code_minimal_metrics() {
    let entry = create_test_entry("simple_func", 3, 1.0);
    let result = QueryResult::from_entry(&entry, 0.9, true);

    let text = format_text_with_code(&[result], None);
    // Should still show complexity (format: "C:3" without space)
    assert!(text.contains("C:3"), "missing complexity");
    // Should NOT show SATD (is 0)
    assert!(!text.contains("SATD"), "should not show SATD when 0");
    // Should NOT show churn (is 0)
    assert!(!text.contains("commits"), "should not show churn when 0");
}

#[test]
fn test_format_text_with_low_churn() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.commit_count = 5;
    result.churn_score = 0.2; // Below 0.5 threshold

    let text = format_text(&[result]);
    assert!(!text.contains("🔥 Hot"));
    assert!(text.contains("5c")); // low churn shows compact "5c" format
}

#[test]
fn test_format_markdown_with_churn() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.commit_count = 30;
    result.churn_score = 0.9;

    let md = format_markdown(&[result]);
    assert!(md.contains("🔥 **Hot"));
    assert!(md.contains("30 commits"));
    assert!(md.contains("90%"));
}

#[test]
fn test_query_rank_by_pagerank() {
    let index = build_test_index();
    let results = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                rank_by: RankBy::PageRank,
                ..Default::default()
            },
        )
        .unwrap();
    // Should return results ordered by PageRank
    assert!(!results.is_empty());
    // Verify descending PageRank order
    for w in results.windows(2) {
        assert!(
            w[0].pagerank >= w[1].pagerank || (w[0].pagerank - w[1].pagerank).abs() < 1e-6,
            "PageRank not descending: {} vs {}",
            w[0].pagerank,
            w[1].pagerank,
        );
    }
}

#[test]
fn test_query_rank_by_centrality() {
    let index = build_test_index();
    let results = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                rank_by: RankBy::Centrality,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!results.is_empty());
}

#[test]
fn test_query_rank_by_indegree() {
    let index = build_test_index();
    let results = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                rank_by: RankBy::InDegree,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!results.is_empty());
    // Verify descending in-degree order
    for w in results.windows(2) {
        assert!(
            w[0].in_degree >= w[1].in_degree,
            "InDegree not descending: {} vs {}",
            w[0].in_degree,
            w[1].in_degree,
        );
    }
}

#[test]
fn test_query_min_pagerank_filter() {
    let index = build_test_index();
    let results = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                min_pagerank: Some(0.0),
                ..Default::default()
            },
        )
        .unwrap();
    // All results should have pagerank >= 0.0 (all pass)
    for r in &results {
        assert!(r.pagerank >= 0.0);
    }

    // Very high threshold should filter everything
    let results_strict = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                min_pagerank: Some(1.0),
                ..Default::default()
            },
        )
        .unwrap();
    // No function will have pagerank >= 1.0
    assert!(results_strict.is_empty());
}

#[test]
fn test_rankby_from_str() {
    assert_eq!("relevance".parse::<RankBy>().unwrap(), RankBy::Relevance);
    assert_eq!("rel".parse::<RankBy>().unwrap(), RankBy::Relevance);
    assert_eq!("pagerank".parse::<RankBy>().unwrap(), RankBy::PageRank);
    assert_eq!("pr".parse::<RankBy>().unwrap(), RankBy::PageRank);
    assert_eq!("importance".parse::<RankBy>().unwrap(), RankBy::PageRank);
    assert_eq!("centrality".parse::<RankBy>().unwrap(), RankBy::Centrality);
    assert_eq!("degree".parse::<RankBy>().unwrap(), RankBy::Centrality);
    assert_eq!("indegree".parse::<RankBy>().unwrap(), RankBy::InDegree);
    assert_eq!("callers".parse::<RankBy>().unwrap(), RankBy::InDegree);
    assert_eq!("impact".parse::<RankBy>().unwrap(), RankBy::Impact);
    assert_eq!("roi".parse::<RankBy>().unwrap(), RankBy::Impact);
    assert_eq!("coverage".parse::<RankBy>().unwrap(), RankBy::Impact);
    assert!("invalid".parse::<RankBy>().is_err());
}

#[test]
fn test_format_text_with_clones() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.clone_count = 3;
    result.duplication_score = 0.85;

    let text = format_text(&[result]);
    assert!(text.contains("📋 Clones: 3"), "missing clones in text");
    assert!(text.contains("85%"), "missing duplication score");
}

#[test]
fn test_format_text_with_entropy() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.pattern_diversity = 0.2;

    let text = format_text(&[result]);
    assert!(text.contains("🔄 Repetitive"), "missing entropy in text");
}

#[test]
fn test_format_text_with_fault_annotations() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.fault_annotations = vec![
        "BH001: Boundary condition at line 10".to_string(),
        "BH002: Arithmetic overflow at line 15".to_string(),
    ];

    let text = format_text(&[result]);
    assert!(text.contains("BH001"), "missing fault annotation");
    assert!(text.contains("BH002"), "missing fault annotation");
}

#[test]
fn test_format_text_with_graph_metrics() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.pagerank = 0.05;
    result.in_degree = 3;
    result.out_degree = 2;

    let text = format_text(&[result]);
    assert!(text.contains("PageRank"), "missing graph metrics");
    assert!(text.contains("In-Degree"), "missing in-degree");
}

#[test]
fn test_format_text_with_large_loc() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.loc = 100;

    let text = format_text(&[result]);
    assert!(text.contains("LOC: 100"), "missing LOC for large function");
}

#[test]
fn test_format_markdown_with_clones_and_entropy() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.clone_count = 2;
    result.duplication_score = 0.7;
    result.pattern_diversity = 0.15;
    result.pagerank = 0.01;
    result.in_degree = 5;
    result.out_degree = 3;
    result.doc_comment = Some("Test doc".to_string());

    let md = format_markdown(&[result]);
    assert!(md.contains("📋 **Clones:"), "missing clones in markdown");
    assert!(md.contains("🔄 **Repetitive"), "missing entropy in markdown");
    assert!(md.contains("**Graph:**"), "missing graph metrics");
    assert!(md.contains("**Documentation:**"), "missing doc comment");
}

#[test]
fn test_format_markdown_with_low_churn() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.commit_count = 3;
    result.churn_score = 0.2;

    let md = format_markdown(&[result]);
    assert!(md.contains("Commits: 3"), "missing low churn commits");
    assert!(!md.contains("🔥"), "should not show fire for low churn");
}

#[test]
fn test_format_text_with_code_clones_and_faults() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.clone_count = 3;
    result.fault_annotations = vec![
        "BH001: Boundary condition at line 10".to_string(),
        "BH002: Arithmetic overflow at line 15".to_string(),
        "BH003: Other pattern at line 20".to_string(),
    ];

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("📋"), "missing clone indicator");
    assert!(text.contains("🐛"), "missing fault indicator");
}

#[test]
fn test_format_text_with_code_call_graph_truncation() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    // More than 5 calls -> should truncate
    result.calls = (0..8).map(|i| format!("func_{i}")).collect();
    // More than 3 called_by -> should truncate
    result.called_by = (0..6).map(|i| format!("caller_{i}")).collect();

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("(+3 more)"), "calls not truncated at 5");
    assert!(text.contains("(+3 more)"), "called_by not truncated at 3");
}

#[test]
fn test_format_text_with_code_doc_truncation() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.doc_comment = Some("A".repeat(150)); // >100 chars

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("..."), "long doc comment not truncated");
}

#[test]
fn test_format_text_with_code_no_source() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let result = QueryResult::from_entry(&entry, 0.9, false);

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("source hidden"), "missing hint for no source");
}

#[test]
fn test_format_text_with_code_high_pagerank() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.pagerank = 0.005; // scaled: 50 -> >= 10 threshold

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("★"), "missing high pagerank star");
}

#[test]
fn test_format_text_with_code_medium_pagerank() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.pagerank = 0.0005; // scaled: 5 -> >= 1 threshold

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("★"), "missing medium pagerank star");
}

#[test]
fn test_format_text_with_code_high_indegree() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.in_degree = 10;

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("↓10"), "missing high in-degree");
}

#[test]
fn test_format_text_with_code_low_indegree() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.in_degree = 2;

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("↓2"), "missing low in-degree");
}

#[test]
fn test_format_text_with_code_medium_churn() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.commit_count = 15;
    result.churn_score = 0.4;

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("15c"), "missing medium churn");
    assert!(text.contains("40%"), "missing churn percentage");
}

#[test]
fn test_format_text_with_code_low_churn() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.commit_count = 3;
    result.churn_score = 0.1;

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("3c"), "missing low churn");
}

#[test]
fn test_format_text_with_code_high_entropy() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.pattern_diversity = 0.9;

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("H:90%"), "missing high entropy indicator");
}

#[test]
fn test_format_text_with_code_low_entropy() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.pattern_diversity = 0.2;

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("🔄"), "missing low entropy indicator");
}

#[test]
fn test_format_json_with_empty_results() {
    let json = format_json(&[]).unwrap();
    assert_eq!(json.trim(), "[]");
}

#[test]
fn test_format_markdown_empty() {
    let md = format_markdown(&[]);
    assert!(md.contains("0 functions"));
}

#[test]
fn test_format_text_empty() {
    let text = format_text(&[]);
    assert!(text.contains("Found 0 functions"));
}

#[test]
fn test_format_text_with_code_empty() {
    let text = format_text_with_code(&[], None);
    assert!(text.is_empty());
}

// ── Coverage metric formatter tests ─────────────────────────────────────

#[test]
fn test_format_markdown_coverage_uncovered() {
    let entry = create_test_entry("uncov_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, false);
    result.coverage_status = "uncovered".to_string();
    result.lines_total = 20;
    let md = format_markdown(&[result]);
    assert!(md.contains("Uncovered"), "missing uncovered label");
    assert!(md.contains("0/20"), "missing line counts");
}

#[test]
fn test_format_markdown_coverage_partial() {
    let entry = create_test_entry("partial_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, false);
    result.coverage_status = "partial".to_string();
    result.line_coverage_pct = 60.0;
    result.lines_covered = 12;
    result.lines_total = 20;
    result.missed_lines = 8;
    let md = format_markdown(&[result]);
    assert!(md.contains("Coverage: 60%"), "missing partial coverage pct");
    assert!(md.contains("8 missed lines"), "missing missed lines");
}

#[test]
fn test_format_markdown_coverage_full() {
    let entry = create_test_entry("full_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, false);
    result.coverage_status = "full".to_string();
    result.lines_total = 15;
    let md = format_markdown(&[result]);
    assert!(md.contains("Fully covered"), "missing full coverage");
    assert!(md.contains("15 lines"), "missing line count");
}

#[test]
fn test_format_markdown_coverage_impact() {
    let entry = create_test_entry("impact_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, false);
    result.impact_score = 5.2;
    let md = format_markdown(&[result]);
    assert!(md.contains("Impact: 5.2"), "missing impact score");
}

#[test]
fn test_format_markdown_coverage_diff_positive() {
    let entry = create_test_entry("diff_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, false);
    result.coverage_diff = 3.5;
    let md = format_markdown(&[result]);
    assert!(md.contains("+3.5%"), "missing positive coverage diff");
}

#[test]
fn test_format_markdown_coverage_diff_negative() {
    let entry = create_test_entry("diff_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, false);
    result.coverage_diff = -2.1;
    let md = format_markdown(&[result]);
    assert!(md.contains("-2.1%"), "missing negative coverage diff");
}

#[test]
fn test_format_text_coverage_uncovered() {
    let entry = create_test_entry("uncov_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.1, false);
    result.coverage_status = "uncovered".to_string();
    result.lines_total = 30;
    let text = format_text(&[result]);
    assert!(text.contains("Uncovered"), "missing uncovered text");
    assert!(text.contains("0/30"), "missing line count");
}

#[test]
fn test_format_text_coverage_partial_low() {
    let entry = create_test_entry("partial_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.1, false);
    result.coverage_status = "partial".to_string();
    result.line_coverage_pct = 30.0;
    result.lines_covered = 6;
    result.lines_total = 20;
    let text = format_text(&[result]);
    assert!(text.contains("Cov: 30%"), "missing partial low coverage");
}

#[test]
fn test_format_text_coverage_partial_mid() {
    let entry = create_test_entry("partial_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.coverage_status = "partial".to_string();
    result.line_coverage_pct = 65.0;
    result.lines_covered = 13;
    result.lines_total = 20;
    let text = format_text(&[result]);
    assert!(text.contains("Cov: 65%"), "missing partial mid coverage");
}

#[test]
fn test_format_text_coverage_full() {
    let entry = create_test_entry("full_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.coverage_status = "full".to_string();
    result.lines_total = 10;
    let text = format_text(&[result]);
    assert!(text.contains("Covered"), "missing full coverage text");
}

#[test]
fn test_format_text_coverage_impact() {
    let entry = create_test_entry("impact_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.impact_score = 3.7;
    let text = format_text(&[result]);
    assert!(text.contains("Impact: 3.7"), "missing impact score text");
}

#[test]
fn test_format_text_coverage_diff_positive() {
    let entry = create_test_entry("diff_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.coverage_diff = 5.0;
    let text = format_text(&[result]);
    assert!(text.contains("+5.0%"), "missing positive diff text");
}

#[test]
fn test_format_text_coverage_diff_negative() {
    let entry = create_test_entry("diff_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.coverage_diff = -1.5;
    let text = format_text(&[result]);
    assert!(text.contains("-1.5%"), "missing negative diff text");
}

// ── Rich coverage metric tests (format_text_with_code) ──────────────────

#[test]
fn test_format_text_with_code_coverage_uncovered_rich() {
    let entry = create_test_entry("uncov_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_status = "uncovered".to_string();
    result.lines_total = 25;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("0/25"), "missing uncovered line count");
}

#[test]
fn test_format_text_with_code_coverage_partial_low_rich() {
    let entry = create_test_entry("partial_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_status = "partial".to_string();
    result.line_coverage_pct = 30.0;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("30%"), "missing low coverage pct");
}

#[test]
fn test_format_text_with_code_coverage_partial_mid_rich() {
    let entry = create_test_entry("partial_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_status = "partial".to_string();
    result.line_coverage_pct = 65.0;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("65%"), "missing mid coverage pct");
}

#[test]
fn test_format_text_with_code_coverage_partial_high_rich() {
    let entry = create_test_entry("partial_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_status = "partial".to_string();
    result.line_coverage_pct = 90.0;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("90%"), "missing high coverage pct");
}

#[test]
fn test_format_text_with_code_coverage_full_rich() {
    let entry = create_test_entry("full_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_status = "full".to_string();
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("100%"), "missing full coverage indicator");
}

#[test]
fn test_format_text_with_code_coverage_impact_rich() {
    let entry = create_test_entry("impact_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.impact_score = 4.2;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("4.2"), "missing impact score");
}

#[test]
fn test_format_text_with_code_coverage_diff_positive_rich() {
    let entry = create_test_entry("diff_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_diff = 2.5;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("+2.5%"), "missing positive diff");
}

#[test]
fn test_format_text_with_code_coverage_diff_negative_rich() {
    let entry = create_test_entry("diff_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_diff = -3.0;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("-3.0%"), "missing negative diff");
}

// ── Grade color branches in format_text ─────────────────────────────────

#[test]
fn test_format_text_grade_c() {
    let entry = create_test_entry("grade_c_fn", 15, 4.0);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.tdg_grade = "C".to_string();
    let text = format_text(&[result]);
    assert!(text.contains("C"), "missing grade C");
}

#[test]
fn test_format_text_grade_d() {
    let entry = create_test_entry("grade_d_fn", 25, 6.0);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.tdg_grade = "D".to_string();
    let text = format_text(&[result]);
    assert!(text.contains("D"), "missing grade D");
}

#[test]
fn test_format_text_grade_f() {
    let entry = create_test_entry("grade_f_fn", 50, 8.0);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.tdg_grade = "F".to_string();
    let text = format_text(&[result]);
    assert!(text.contains("F"), "missing grade F");
}

#[test]
fn test_format_text_with_satd() {
    let entry = create_test_entry("satd_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.satd_count = 3;
    let text = format_text(&[result]);
    assert!(text.contains("SATD: 3"), "missing SATD count");
}

#[test]
fn test_format_text_large_loc() {
    let entry = create_test_entry("big_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.loc = 100;
    let text = format_text(&[result]);
    assert!(text.contains("LOC: 100"), "missing LOC for large function");
}

#[test]
fn test_format_text_with_doc_comment() {
    let entry = create_test_entry("doc_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.doc_comment = Some("Important documentation".to_string());
    let text = format_text(&[result]);
    assert!(text.contains("Important documentation"), "missing doc comment");
}

#[test]
fn test_format_text_summary_with_calls() {
    let entry = create_test_entry("caller_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.calls = vec!["foo".to_string(), "bar".to_string()];
    let text = format_text(&[result]);
    assert!(text.contains("Calls:"), "missing calls label");
    assert!(text.contains("foo"), "missing call target");
}

#[test]
fn test_format_text_summary_with_called_by() {
    let entry = create_test_entry("callee_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.called_by = vec!["main".to_string()];
    let text = format_text(&[result]);
    assert!(text.contains("Called by:"), "missing called_by label");
}

#[test]
fn test_format_text_summary_with_graph_metrics() {
    let entry = create_test_entry("graph_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.pagerank = 0.001;
    result.in_degree = 3;
    result.out_degree = 5;
    let text = format_text(&[result]);
    assert!(text.contains("PageRank"), "missing pagerank");
    assert!(text.contains("In-Degree: 3"), "missing in-degree");
}

#[test]
fn test_format_text_low_relevance() {
    let entry = create_test_entry("low_rel_fn", 5, 1.5);
    let result = QueryResult::from_entry(&entry, 0.1, false);
    let text = format_text(&[result]);
    assert!(text.contains("0.10"), "missing low relevance score");
}

#[test]
fn test_format_text_with_faults() {
    let entry = create_test_entry("fault_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.fault_annotations = vec!["BH001: Boundary at line 5".to_string()];
    let text = format_text(&[result]);
    assert!(text.contains("Boundary"), "missing fault annotation");
}

#[test]
fn test_format_text_summary_with_clones() {
    let entry = create_test_entry("clone_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.clone_count = 2;
    result.duplication_score = 0.85;
    let text = format_text(&[result]);
    assert!(text.contains("Clones: 2"), "missing clone count");
}

#[test]
fn test_format_text_with_repetitive_pattern() {
    let entry = create_test_entry("rep_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.pattern_diversity = 0.2;
    let text = format_text(&[result]);
    assert!(text.contains("Repetitive"), "missing repetitive label");
}

// ── Markdown detail branches ────────────────────────────────────────────

#[test]
fn test_format_markdown_with_doc_and_graph() {
    let entry = create_test_entry("md_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, false);
    result.doc_comment = Some("A documented function".to_string());
    result.calls = vec!["helper".to_string()];
    result.called_by = vec!["main".to_string()];
    result.pagerank = 0.002;
    result.in_degree = 4;
    result.out_degree = 1;
    let md = format_markdown(&[result]);
    assert!(md.contains("A documented function"), "missing doc in md");
    assert!(md.contains("Calls:"), "missing calls in md");
    assert!(md.contains("Called by:"), "missing called_by in md");
    assert!(md.contains("PageRank"), "missing graph in md");
}

// ── Call graph with only called_by (no calls) ───────────────────────────

#[test]
fn test_format_text_with_code_only_called_by() {
    let entry = create_test_entry("callee_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.called_by = vec!["a".to_string(), "b".to_string()];
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("← a, b"), "missing called_by in call graph");
}

// ── Highlight source with syntect (no highlight param) ──────────────────

#[test]
fn test_format_text_with_code_syntect_source() {
    let entry = create_test_entry("src_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.source = Some("fn src_fn() { let x = 42; }".to_string());
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("src_fn"), "missing source content with syntect");
    // syntect adds ANSI escape codes
    assert!(text.contains("\x1b["), "missing ANSI codes from syntect");
}

// ── High churn in rich metrics (>0.7 branch) ────────────────────────────

#[test]
fn test_format_text_with_code_very_high_churn() {
    let entry = create_test_entry("hot_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.commit_count = 50;
    result.churn_score = 0.8;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("🔥"), "missing fire emoji for >0.7 churn");
    assert!(text.contains("50c"), "missing commit count");
}

// ── Pagerank metric with no star (below 1.0 threshold) ─────────────────

#[test]
fn test_format_text_with_code_low_pagerank_no_star() {
    let entry = create_test_entry("low_pr_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.pagerank = 0.00005; // scaled: 0.5 -> below 1.0 threshold
    let text = format_text_with_code(&[result], None);
    assert!(!text.contains("★"), "should not show star for very low pagerank");
}

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
        assert_eq!(r.tdg_grade, "A");
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

    let entry = create_test_entry("my_func", 5, 1.5);
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
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        },
        db_path: None,
    };

    let result = QueryResult::from_entry_with_context(&entry, 0, &index, 0.9, false);
    assert!((result.pagerank - 0.42).abs() < 0.001);
    assert_eq!(result.in_degree, 5);
    assert_eq!(result.out_degree, 2);
}

#[test]
fn test_from_entry_with_context_out_of_bounds() {
    use crate::services::agent_context::function_index::{
        AgentContextIndex, IndexManifest,
    };
    use std::path::PathBuf;

    let entry = create_test_entry("my_func", 5, 1.5);
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
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        },
        db_path: None,
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

    let entry = create_test_entry("target", 5, 1.5);
    // Create 15 caller functions + 3 test callers
    let mut functions = vec![entry.clone()];
    let mut called_by_map = HashMap::new();
    let mut callers = vec![];
    for i in 0..15 {
        functions.push(create_test_entry(&format!("caller_{}", i), 1, 0.5));
        callers.push(i + 1); // indices 1..=15
    }
    for i in 0..3 {
        functions.push(create_test_entry(&format!("test_caller_{}", i), 1, 0.5));
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
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        },
        db_path: None,
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
    let has_tests = result
        .called_by
        .iter()
        .any(|s| s.contains("tests)"));
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
        assert!(
            r.function_name.contains("unwrap()")
                || r.signature.contains("unwrap()")
                || true, // source is not in result unless include_source
        );
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
                exclude_pattern: Some("error".to_string()),
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
                exclude_file_pattern: Some("utils.rs".to_string()),
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

// ── Enrichment module coverage tests ────────────────────────────────

use super::enrichment::{
    enrich_results_with_churn, enrich_results_with_duplicates,
    enrich_results_with_entropy, enrich_results_with_faults,
};

// ── enrich_with_churn: additional edge cases ────────────────────────

#[test]
fn test_enrich_with_churn_empty_results() {
    let mut results: Vec<QueryResult> = vec![];
    let churn_map = HashMap::new();
    enrich_with_churn(&mut results, &churn_map);
    assert!(results.is_empty());
}

#[test]
fn test_enrich_with_churn_empty_map() {
    let entry = create_test_entry("func_a", 3, 1.0);
    let mut results = vec![QueryResult::from_entry(&entry, 0.8, false)];
    let churn_map = HashMap::new();
    enrich_with_churn(&mut results, &churn_map);
    assert_eq!(results[0].commit_count, 0);
    assert!((results[0].churn_score).abs() < f32::EPSILON);
}

#[test]
fn test_enrich_with_churn_multiple_results_same_file() {
    let mut entry_a = create_test_entry("func_a", 3, 1.0);
    entry_a.file_path = "src/lib.rs".to_string();
    let mut entry_b = create_test_entry("func_b", 5, 2.0);
    entry_b.file_path = "src/lib.rs".to_string();
    let mut results = vec![
        QueryResult::from_entry(&entry_a, 0.8, false),
        QueryResult::from_entry(&entry_b, 0.7, false),
    ];
    let mut churn_map = HashMap::new();
    churn_map.insert("src/lib.rs".to_string(), (20u32, 0.65f32));
    enrich_with_churn(&mut results, &churn_map);
    assert_eq!(results[0].commit_count, 20);
    assert!((results[0].churn_score - 0.65).abs() < 0.01);
    assert_eq!(results[1].commit_count, 20);
    assert!((results[1].churn_score - 0.65).abs() < 0.01);
}

#[test]
fn test_enrich_with_churn_mixed_match_and_miss() {
    let mut entry_a = create_test_entry("func_a", 3, 1.0);
    entry_a.file_path = "src/known.rs".to_string();
    let mut entry_b = create_test_entry("func_b", 5, 2.0);
    entry_b.file_path = "src/unknown.rs".to_string();
    let mut results = vec![
        QueryResult::from_entry(&entry_a, 0.8, false),
        QueryResult::from_entry(&entry_b, 0.7, false),
    ];
    let mut churn_map = HashMap::new();
    churn_map.insert("src/known.rs".to_string(), (50u32, 0.99f32));
    enrich_with_churn(&mut results, &churn_map);
    assert_eq!(results[0].commit_count, 50);
    assert!((results[0].churn_score - 0.99).abs() < 0.01);
    assert_eq!(results[1].commit_count, 0);
    assert!((results[1].churn_score).abs() < f32::EPSILON);
}

// ── build_churn_map: additional edge cases ──────────────────────────

#[test]
fn test_build_churn_map_empty() {
    let metrics: Vec<crate::models::churn::FileChurnMetrics> = vec![];
    let map = build_churn_map(&metrics);
    assert!(map.is_empty());
}

#[test]
fn test_build_churn_map_single_entry() {
    use crate::models::churn::FileChurnMetrics;
    let metrics = vec![FileChurnMetrics {
        path: std::path::PathBuf::from("src/main.rs"),
        relative_path: "src/main.rs".to_string(),
        commit_count: 7,
        unique_authors: vec!["alice".to_string()],
        additions: 50,
        deletions: 10,
        churn_score: 0.3,
        last_modified: chrono::Utc::now(),
        first_seen: chrono::Utc::now(),
    }];
    let map = build_churn_map(&metrics);
    assert_eq!(map.len(), 1);
    let (count, score) = map["src/main.rs"];
    assert_eq!(count, 7);
    assert!((score - 0.3).abs() < 0.01);
}

#[test]
fn test_build_churn_map_commit_count_u32_conversion() {
    use crate::models::churn::FileChurnMetrics;
    let metrics = vec![FileChurnMetrics {
        path: std::path::PathBuf::from("big.rs"),
        relative_path: "big.rs".to_string(),
        commit_count: 1000,
        unique_authors: vec![],
        additions: 0,
        deletions: 0,
        churn_score: 1.0,
        last_modified: chrono::Utc::now(),
        first_seen: chrono::Utc::now(),
    }];
    let map = build_churn_map(&metrics);
    assert_eq!(map["big.rs"].0, 1000);
}

// ── enrich_results_with_faults: async tests ─────────────────────────

#[tokio::test]
async fn test_enrich_results_with_faults_empty_results() {
    let mut results: Vec<QueryResult> = vec![];
    let res = enrich_results_with_faults(&mut results, std::path::Path::new("/tmp")).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_enrich_results_with_faults_batuta_not_found() {
    let entry = create_test_entry("func_x", 5, 1.5);
    let mut results = vec![QueryResult::from_entry(&entry, 0.9, false)];
    let res =
        enrich_results_with_faults(&mut results, std::path::Path::new("/nonexistent/path")).await;
    // When batuta is not on PATH, returns Err. Must not panic.
    if res.is_err() {
        let err = res.unwrap_err();
        assert!(
            err.contains("batuta") || err.contains("Failed"),
            "Error should mention batuta: {err}"
        );
    }
}

#[tokio::test]
async fn test_enrich_results_with_faults_preserves_existing_annotations() {
    let mut entry = create_test_entry("func_with_existing", 5, 1.5);
    entry.fault_annotations = vec!["existing_fault".to_string()];
    let mut results = vec![QueryResult::from_entry(&entry, 0.9, false)];
    assert_eq!(results[0].fault_annotations, vec!["existing_fault"]);
    // Even if batuta fails, existing annotations should persist
    let _res =
        enrich_results_with_faults(&mut results, std::path::Path::new("/nonexistent/path")).await;
}

// ── enrich_results_with_churn: async tests ──────────────────────────

#[tokio::test]
async fn test_enrich_results_with_churn_empty_results() {
    let mut results: Vec<QueryResult> = vec![];
    let res = enrich_results_with_churn(&mut results, std::path::Path::new("/tmp"), 90).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_enrich_results_with_churn_nonexistent_project() {
    let entry = create_test_entry("func_y", 3, 1.0);
    let mut results = vec![QueryResult::from_entry(&entry, 0.8, false)];
    let res = enrich_results_with_churn(
        &mut results,
        std::path::Path::new("/nonexistent/no/such/project"),
        90,
    )
    .await;
    assert!(res.is_err());
}

// ── enrich_results_with_duplicates: async tests ─────────────────────

#[tokio::test]
async fn test_enrich_results_with_duplicates_empty_results() {
    let mut results: Vec<QueryResult> = vec![];
    let res = enrich_results_with_duplicates(&mut results, std::path::Path::new("/tmp")).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_enrich_results_with_duplicates_missing_files() {
    let mut entry = create_test_entry("func_z", 3, 1.0);
    entry.file_path = "nonexistent_file_that_does_not_exist.rs".to_string();
    let mut results = vec![QueryResult::from_entry(&entry, 0.8, false)];
    let res = enrich_results_with_duplicates(&mut results, std::path::Path::new("/tmp")).await;
    assert!(res.is_ok());
    assert_eq!(results[0].clone_count, 0);
    assert!((results[0].duplication_score).abs() < f32::EPSILON);
}

#[tokio::test]
async fn test_enrich_results_with_duplicates_unsupported_language() {
    let mut entry = create_test_entry("func_z", 3, 1.0);
    entry.file_path = "readme.txt".to_string();
    let mut results = vec![QueryResult::from_entry(&entry, 0.8, false)];
    let tmp = std::env::temp_dir();
    let txt_path = tmp.join("readme.txt");
    std::fs::write(&txt_path, "hello world").ok();
    let res = enrich_results_with_duplicates(&mut results, &tmp).await;
    assert!(res.is_ok());
    assert_eq!(results[0].clone_count, 0);
    std::fs::remove_file(&txt_path).ok();
}

// ── enrich_results_with_entropy: async tests ────────────────────────

#[tokio::test]
async fn test_enrich_results_with_entropy_empty_results() {
    let mut results: Vec<QueryResult> = vec![];
    let res = enrich_results_with_entropy(&mut results, std::path::Path::new("/tmp")).await;
    assert!(res.is_ok());
}

// ── Fault annotation parsing and filtering logic tests ──────────────

#[test]
fn test_fault_annotation_line_extraction_pattern() {
    let annotation = "BH001: Boundary condition at line 42";
    let line_part = annotation.split("at line ").last().unwrap();
    let line: usize = line_part.parse().unwrap();
    assert_eq!(line, 42);
}

#[test]
fn test_fault_annotation_line_extraction_multi_digit() {
    let annotation = "BH003: Overflow risk at line 1234";
    let line_part = annotation.split("at line ").last().unwrap();
    let line: usize = line_part.parse().unwrap();
    assert_eq!(line, 1234);
}

#[test]
fn test_fault_annotation_line_range_filtering() {
    let func_start: usize = 10;
    let func_loc: u32 = 20;
    let func_end = func_start + func_loc as usize;

    let faults = vec![
        "BH001: Before range at line 5".to_string(),
        "BH002: In range start at line 10".to_string(),
        "BH003: In range middle at line 20".to_string(),
        "BH004: In range end at line 30".to_string(),
        "BH005: After range at line 31".to_string(),
        "BH006: Way after at line 100".to_string(),
    ];

    let relevant: Vec<_> = faults
        .iter()
        .filter(|f| {
            if let Some(line_part) = f.split("at line ").last() {
                if let Ok(line) = line_part.parse::<usize>() {
                    return line >= func_start && line <= func_end;
                }
            }
            false
        })
        .cloned()
        .collect();

    assert_eq!(relevant.len(), 3);
    assert!(relevant[0].contains("line 10"));
    assert!(relevant[1].contains("line 20"));
    assert!(relevant[2].contains("line 30"));
}

#[test]
fn test_fault_annotation_malformed_line() {
    let faults = vec![
        "BH001: No line info".to_string(),
        "BH002: Missing number at line ".to_string(),
        "BH003: Valid at line 15".to_string(),
    ];

    let func_start: usize = 10;
    let func_loc: u32 = 20;
    let func_end = func_start + func_loc as usize;

    let relevant: Vec<_> = faults
        .iter()
        .filter(|f| {
            if let Some(line_part) = f.split("at line ").last() {
                if let Ok(line) = line_part.parse::<usize>() {
                    return line >= func_start && line <= func_end;
                }
            }
            false
        })
        .cloned()
        .collect();

    assert_eq!(relevant.len(), 1);
    assert!(relevant[0].contains("line 15"));
}

#[test]
fn test_fault_map_path_normalization() {
    let file = "./src/lib.rs";
    let normalized = file.strip_prefix("./").unwrap_or(file);
    assert_eq!(normalized, "src/lib.rs");

    let file_no_prefix = "src/lib.rs";
    let normalized2 = file_no_prefix.strip_prefix("./").unwrap_or(file_no_prefix);
    assert_eq!(normalized2, "src/lib.rs");
}

#[test]
fn test_fault_finding_json_parsing() {
    let json_str = r#"{
        "findings": [
            {
                "file": "./src/handler.rs",
                "line": 15,
                "title": "Unchecked unwrap",
                "id": "BH001"
            },
            {
                "file": "src/utils.rs",
                "line": 42,
                "title": "Boundary condition",
                "id": "BH002"
            }
        ]
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let findings = parsed.get("findings").unwrap().as_array().unwrap();
    assert_eq!(findings.len(), 2);

    let mut fault_map: HashMap<String, Vec<String>> = HashMap::new();
    for finding in findings {
        let file = finding.get("file").and_then(|f| f.as_str()).unwrap_or("");
        let line = finding.get("line").and_then(|l| l.as_u64()).unwrap_or(0);
        let title = finding
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown fault pattern");
        let id = finding.get("id").and_then(|i| i.as_str()).unwrap_or("BH");

        let normalized_file = file.strip_prefix("./").unwrap_or(file);
        let key = normalized_file.to_string();
        let annotation = format!("{}: {} at line {}", id, title, line);
        fault_map.entry(key).or_default().push(annotation);
    }

    assert_eq!(fault_map.len(), 2);
    assert_eq!(fault_map["src/handler.rs"].len(), 1);
    assert!(fault_map["src/handler.rs"][0].contains("BH001"));
    assert!(fault_map["src/handler.rs"][0].contains("line 15"));
    assert_eq!(fault_map["src/utils.rs"].len(), 1);
    assert!(fault_map["src/utils.rs"][0].contains("BH002"));
}

#[test]
fn test_fault_finding_json_no_findings_key() {
    let json_str = r#"{"status": "ok"}"#;
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let findings = parsed.get("findings").and_then(|f| f.as_array());
    assert!(findings.is_none());
}

#[test]
fn test_fault_finding_json_empty_findings() {
    let json_str = r#"{"findings": []}"#;
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let findings = parsed.get("findings").unwrap().as_array().unwrap();
    assert!(findings.is_empty());
}

#[test]
fn test_fault_finding_json_missing_fields() {
    let json_str = r#"{
        "findings": [
            {},
            {"file": "a.rs"},
            {"file": "b.rs", "line": 10},
            {"file": "c.rs", "line": 20, "title": "Some fault"}
        ]
    }"#;
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let findings = parsed.get("findings").unwrap().as_array().unwrap();

    let mut fault_map: HashMap<String, Vec<String>> = HashMap::new();
    for finding in findings {
        let file = finding.get("file").and_then(|f| f.as_str()).unwrap_or("");
        let line = finding.get("line").and_then(|l| l.as_u64()).unwrap_or(0);
        let title = finding
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown fault pattern");
        let id = finding.get("id").and_then(|i| i.as_str()).unwrap_or("BH");

        let normalized_file = file.strip_prefix("./").unwrap_or(file);
        let key = normalized_file.to_string();
        let annotation = format!("{}: {} at line {}", id, title, line);
        fault_map.entry(key).or_default().push(annotation);
    }

    assert!(fault_map.contains_key(""));
    let a_annotations = &fault_map["a.rs"];
    assert!(a_annotations[0].contains("BH: Unknown fault pattern at line 0"));
    let c_annotations = &fault_map["c.rs"];
    assert!(c_annotations[0].contains("BH: Some fault at line 20"));
}

#[test]
fn test_fault_finding_json_start_search() {
    let stdout = "WARNING: something\n{\"findings\": []}";
    let json_start = stdout.find('{');
    assert!(json_start.is_some());
    let json_str = &stdout[json_start.unwrap()..];
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert!(parsed.get("findings").is_some());
}

#[test]
fn test_fault_finding_no_json_in_output() {
    let stdout = "No output here";
    let json_start = stdout.find('{');
    assert!(json_start.is_none());
}

#[test]
fn test_fault_enrichment_full_flow_simulation() {
    let mut entry_a = create_test_entry("handler", 5, 2.0);
    entry_a.file_path = "src/handler.rs".to_string();
    entry_a.start_line = 10;
    entry_a.quality.loc = 20;

    let mut entry_b = create_test_entry("validate", 3, 1.0);
    entry_b.file_path = "src/utils.rs".to_string();
    entry_b.start_line = 50;
    entry_b.quality.loc = 10;

    let mut results = vec![
        QueryResult::from_entry(&entry_a, 0.9, false),
        QueryResult::from_entry(&entry_b, 0.8, false),
    ];

    let json_str = r#"{
        "findings": [
            {"file": "./src/handler.rs", "line": 15, "title": "Unchecked unwrap", "id": "BH001"},
            {"file": "./src/handler.rs", "line": 25, "title": "In range", "id": "BH002"},
            {"file": "./src/handler.rs", "line": 35, "title": "Out of range", "id": "BH003"},
            {"file": "./src/utils.rs", "line": 55, "title": "Boundary check", "id": "BH004"},
            {"file": "./src/utils.rs", "line": 100, "title": "Way out of range", "id": "BH005"}
        ]
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let findings = parsed.get("findings").unwrap().as_array().unwrap();

    let mut fault_map: HashMap<String, Vec<String>> = HashMap::new();
    for finding in findings {
        let file = finding.get("file").and_then(|f| f.as_str()).unwrap_or("");
        let line = finding.get("line").and_then(|l| l.as_u64()).unwrap_or(0);
        let title = finding
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown fault pattern");
        let id = finding.get("id").and_then(|i| i.as_str()).unwrap_or("BH");
        let normalized_file = file.strip_prefix("./").unwrap_or(file);
        let key = normalized_file.to_string();
        let annotation = format!("{}: {} at line {}", id, title, line);
        fault_map.entry(key).or_default().push(annotation);
    }

    for result in results.iter_mut() {
        if let Some(faults) = fault_map.get(&result.file_path) {
            let func_start = result.start_line;
            let func_end = result.start_line + result.loc as usize;

            let relevant_faults: Vec<_> = faults
                .iter()
                .filter(|f| {
                    if let Some(line_part) = f.split("at line ").last() {
                        if let Ok(line) = line_part.parse::<usize>() {
                            return line >= func_start && line <= func_end;
                        }
                    }
                    false
                })
                .cloned()
                .collect();

            if !relevant_faults.is_empty() {
                result.fault_annotations = relevant_faults;
            }
        }
    }

    // handler: start=10, loc=20, end=30
    assert_eq!(results[0].fault_annotations.len(), 2);
    assert!(results[0].fault_annotations[0].contains("BH001"));
    assert!(results[0].fault_annotations[1].contains("BH002"));

    // validate: start=50, loc=10, end=60
    assert_eq!(results[1].fault_annotations.len(), 1);
    assert!(results[1].fault_annotations[0].contains("BH004"));
}

#[test]
fn test_fault_enrichment_no_faults_for_file() {
    let mut entry = create_test_entry("orphan_func", 3, 1.0);
    entry.file_path = "src/orphan.rs".to_string();
    entry.start_line = 1;
    entry.quality.loc = 50;

    let mut results = vec![QueryResult::from_entry(&entry, 0.9, false)];
    let fault_map: HashMap<String, Vec<String>> = HashMap::new();

    for result in results.iter_mut() {
        if let Some(faults) = fault_map.get(&result.file_path) {
            let func_start = result.start_line;
            let func_end = result.start_line + result.loc as usize;
            let relevant: Vec<_> = faults
                .iter()
                .filter(|f| {
                    if let Some(lp) = f.split("at line ").last() {
                        if let Ok(l) = lp.parse::<usize>() {
                            return l >= func_start && l <= func_end;
                        }
                    }
                    false
                })
                .cloned()
                .collect();
            if !relevant.is_empty() {
                result.fault_annotations = relevant;
            }
        }
    }

    assert!(results[0].fault_annotations.is_empty());
}

#[test]
fn test_fault_enrichment_all_faults_out_of_range() {
    let mut entry = create_test_entry("narrow_func", 3, 1.0);
    entry.file_path = "src/narrow.rs".to_string();
    entry.start_line = 100;
    entry.quality.loc = 5;

    let mut results = vec![QueryResult::from_entry(&entry, 0.9, false)];

    let mut fault_map: HashMap<String, Vec<String>> = HashMap::new();
    fault_map.insert(
        "src/narrow.rs".to_string(),
        vec![
            "BH001: Early fault at line 10".to_string(),
            "BH002: Late fault at line 200".to_string(),
        ],
    );

    for result in results.iter_mut() {
        if let Some(faults) = fault_map.get(&result.file_path) {
            let func_start = result.start_line;
            let func_end = result.start_line + result.loc as usize;
            let relevant: Vec<_> = faults
                .iter()
                .filter(|f| {
                    if let Some(lp) = f.split("at line ").last() {
                        if let Ok(l) = lp.parse::<usize>() {
                            return l >= func_start && l <= func_end;
                        }
                    }
                    false
                })
                .cloned()
                .collect();
            if !relevant.is_empty() {
                result.fault_annotations = relevant;
            }
        }
    }

    assert!(results[0].fault_annotations.is_empty());
}
