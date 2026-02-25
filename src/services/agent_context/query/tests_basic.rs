// Basic tests: QueryResult construction, formatters, parse_query_prefixes.
// Lines 44–178 of the original tests.rs.

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
    let (file, func, remaining) = parse_query_prefixes("file:foo.rs fn:bar baz");
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
