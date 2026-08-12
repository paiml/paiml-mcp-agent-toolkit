// Query engine tests: scoped queries, filters, ranking, get_function, find_similar.
// Lines 369–696 + 748–784 of the original tests.rs.

#[test]
fn test_query_empty_query_browses_all() {
    let index = build_test_index();
    let result = index.query("", QueryOptions::default());
    assert!(result.is_ok(), "Empty query should return browse results");
    let results = result.unwrap();
    // Browse mode returns functions sorted by PageRank
    assert!(!results.is_empty());
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
        assert!(
            r.file_path.contains("utils.rs"),
            "unexpected file: {}",
            r.file_path
        );
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
    // `--min-grade A` means "A or better", so A+ qualifies too. Asserting
    // `== "A"` pinned the old five-letter scale, where A+ did not exist.
    assert!(!results.is_empty(), "fixture must yield at least one A-or-better");
    for r in &results {
        assert!(
            crate::services::agent_context::query::grades::grade_meets_threshold(&r.tdg_grade, "A"),
            "grade {} should not have passed --min-grade A",
            r.tdg_grade
        );
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
        assert!(
            r.complexity <= 3,
            "complexity {} exceeds max 3",
            r.complexity
        );
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
    let handle_pos = results
        .iter()
        .position(|r| r.function_name == "handle_error");
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
    assert!(results
        .iter()
        .all(|r| !(r.file_path == "src/handler.rs" && r.function_name == "handle_error")));
}

#[test]
fn test_find_similar_not_found() {
    let index = build_test_index();
    let result = index.find_similar("nonexistent.rs", "foo", 3);
    assert!(result.is_err());
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
    let mut entry = create_test_entry("test_something", 1, 96.0);
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
fn test_query_path_pattern_accepts_globs() {
    // `path_pattern` is advertised in the MCP schema as a "Path glob pattern
    // filter", but the filter was `file_path.contains(pattern)`: any glob
    // metacharacter silently produced an empty result set instead of matching
    // the files a bare substring matched.
    let index = build_test_index();
    let glob_hits = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                path_pattern: Some("src/*.rs".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

    assert!(
        !glob_hits.is_empty(),
        "a glob over src/ must match the same files the substring 'src/' matches"
    );
    for r in &glob_hits {
        assert!(
            r.file_path.starts_with("src/") && !r.file_path["src/".len()..].contains('/'),
            "glob src/*.rs must not match {}",
            r.file_path
        );
    }

    // Substring patterns (no metacharacters) keep working unchanged.
    let substring_hits = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                path_pattern: Some("src/".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!substring_hits.is_empty());
}

/// R30 regression: `--min-grade` must not admit a grade it cannot order.
///
/// The old filter compared positions in a private `["A","B","C","D","F"]`
/// table and skipped the check entirely when either side was absent, so every
/// modifier grade — `A-`, `B+`, `C-` — passed `--min-grade A` unexamined. The
/// eleven-grade unification would have made that the common case.
#[test]
fn test_query_min_grade_rejects_modifier_grades_below_threshold() {
    let mut index = build_test_index();
    // Grade the two "handle" functions A- and B+: both are worse than A, and
    // neither appears in the old five-letter table.
    for f in index.functions.iter_mut() {
        if f.function_name == "handle_error" {
            f.quality.tdg_score = 86.0;
            f.quality.tdg_grade = "A-".to_string();
        } else if f.function_name == "handle_request" {
            f.quality.tdg_score = 81.0;
            f.quality.tdg_grade = "B+".to_string();
        }
    }

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

    for r in &results {
        assert!(
            r.tdg_grade != "A-" && r.tdg_grade != "B+",
            "grade {} passed --min-grade A ({} at {}:{})",
            r.tdg_grade,
            r.function_name,
            r.file_path,
            r.start_line
        );
    }
}
