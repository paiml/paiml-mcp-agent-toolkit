// Unit tests for query result formatters.
// Included into formatters.rs -- do NOT add `use` imports or `#!` inner attributes here.

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(name: &str, doc: Option<&str>) -> QueryResult {
        QueryResult {
            function_name: name.to_string(),
            file_path: "src/test.rs".to_string(),
            signature: format!("fn {}()", name),
            definition_type: "function".to_string(),
            doc_comment: doc.map(|s| s.to_string()),
            start_line: 1,
            end_line: 10,
            language: "Rust".to_string(),
            tdg_score: 80.0,
            tdg_grade: "A".to_string(),
            complexity: 5,
            big_o: "O(1)".to_string(),
            satd_count: 0,
            loc: 10,
            relevance_score: 0.95,
            source: None,
            calls: Vec::new(),
            called_by: Vec::new(),
            pagerank: 0.0,
            in_degree: 0,
            out_degree: 0,
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            duplication_score: 0.0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
            line_coverage_pct: 0.0,
            lines_covered: 0,
            lines_total: 0,
            missed_lines: 0,
            impact_score: 0.0,
            coverage_status: String::new(),
            coverage_diff: 0.0,
            coverage_exclusion: Default::default(),
            coverage_excluded: false,
            cross_project_callers: 0,
            io_classification: String::new(),
            io_patterns: Vec::new(),
            suggested_module: String::new(),
        }
    }

    /// Regression test for #157: UTF-8 multi-byte char boundary panic
    #[test]
    fn test_format_text_with_code_multibyte_doc_comment() {
        let result = make_result(
            "verify_output",
            Some("Verify output is correct: not empty, no garbage, contains expected answer (PMAT-QA-PROTOCOL-001 §7.5)  Order of checks is CRITICAL for safety"),
        );
        let output = format_text_with_code(&[result], None);
        assert!(output.contains("verify_output"));
        assert!(output.contains("..."));
    }

    #[test]
    fn test_format_text_short_doc_no_truncation() {
        let result = make_result("foo", Some("Short doc"));
        let output = format_text_with_code(&[result], None);
        assert!(output.contains("Short doc"));
    }

    #[test]
    fn test_highlight_matches_literal() {
        let line = "let result = unwrap();";
        let out = highlight_matches_in_line(line, "unwrap()", false);
        assert!(out.contains("\x1b[1;43m"), "missing highlight start");
        assert!(out.contains("unwrap()"), "missing matched text");
        assert!(out.contains("\x1b[0m"), "missing highlight end");
    }

    #[test]
    fn test_highlight_matches_literal_case_insensitive() {
        let line = "fn HandleRequest() {}";
        let out = highlight_matches_in_line(line, "handlerequest", false);
        // Should highlight preserving original case
        assert!(out.contains("HandleRequest"));
        assert!(out.contains("\x1b[1;43m"));
    }

    #[test]
    fn test_highlight_matches_regex() {
        let line = "fn handle_request(ctx: Context) {}";
        let out = highlight_matches_in_line(line, r"fn\s+handle_\w+", true);
        assert!(out.contains("\x1b[1;43m"));
        assert!(out.contains("fn handle_request"));
    }

    #[test]
    fn test_highlight_matches_no_match() {
        let line = "let x = 42;";
        let out = highlight_matches_in_line(line, "nonexistent", false);
        assert_eq!(out, line);
    }

    #[test]
    fn test_highlight_matches_invalid_regex() {
        let line = "some text here";
        let out = highlight_matches_in_line(line, "[invalid", true);
        assert_eq!(out, line);
    }

    #[test]
    fn test_format_text_with_code_literal_highlight() {
        let mut result = make_result("test_fn", None);
        result.source = Some("fn test_fn() {\n    unwrap();\n}".to_string());
        let output = format_text_with_code(&[result], Some(("unwrap()", false)));
        assert!(output.contains("\x1b[1;43m"), "missing highlight in output");
        // Should have line numbers in highlight mode
        assert!(output.contains("\u{2502}"), "missing line number separator");
    }
}
