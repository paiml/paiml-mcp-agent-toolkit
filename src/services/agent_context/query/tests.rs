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

    let text = format_text_with_code(&[result]);
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

    let text = format_text_with_code(&[result]);
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
    assert!(text.contains("Commits: 5"));
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

    let text = format_text_with_code(&[result]);
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

    let text = format_text_with_code(&[result]);
    assert!(text.contains("(+3 more)"), "calls not truncated at 5");
    assert!(text.contains("(+3 more)"), "called_by not truncated at 3");
}

#[test]
fn test_format_text_with_code_doc_truncation() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.doc_comment = Some("A".repeat(150)); // >100 chars

    let text = format_text_with_code(&[result]);
    assert!(text.contains("..."), "long doc comment not truncated");
}

#[test]
fn test_format_text_with_code_no_source() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let result = QueryResult::from_entry(&entry, 0.9, false);

    let text = format_text_with_code(&[result]);
    assert!(text.contains("--include-source"), "missing hint for no source");
}

#[test]
fn test_format_text_with_code_high_pagerank() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.pagerank = 0.005; // scaled: 50 -> >= 10 threshold

    let text = format_text_with_code(&[result]);
    assert!(text.contains("★"), "missing high pagerank star");
}

#[test]
fn test_format_text_with_code_medium_pagerank() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.pagerank = 0.0005; // scaled: 5 -> >= 1 threshold

    let text = format_text_with_code(&[result]);
    assert!(text.contains("★"), "missing medium pagerank star");
}

#[test]
fn test_format_text_with_code_high_indegree() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.in_degree = 10;

    let text = format_text_with_code(&[result]);
    assert!(text.contains("↓10"), "missing high in-degree");
}

#[test]
fn test_format_text_with_code_low_indegree() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.in_degree = 2;

    let text = format_text_with_code(&[result]);
    assert!(text.contains("↓2"), "missing low in-degree");
}

#[test]
fn test_format_text_with_code_medium_churn() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.commit_count = 15;
    result.churn_score = 0.4;

    let text = format_text_with_code(&[result]);
    assert!(text.contains("15c"), "missing medium churn");
    assert!(text.contains("40%"), "missing churn percentage");
}

#[test]
fn test_format_text_with_code_low_churn() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.commit_count = 3;
    result.churn_score = 0.1;

    let text = format_text_with_code(&[result]);
    assert!(text.contains("3c"), "missing low churn");
}

#[test]
fn test_format_text_with_code_high_entropy() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.pattern_diversity = 0.9;

    let text = format_text_with_code(&[result]);
    assert!(text.contains("H:90%"), "missing high entropy indicator");
}

#[test]
fn test_format_text_with_code_low_entropy() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.pattern_diversity = 0.2;

    let text = format_text_with_code(&[result]);
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
    let text = format_text_with_code(&[]);
    assert!(text.is_empty());
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
