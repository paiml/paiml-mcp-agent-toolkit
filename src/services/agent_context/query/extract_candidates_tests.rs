// Tests for extract_candidates module
// Included by extract_candidates.rs — no `use` imports allowed here.

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(name: &str, def_type: &str, source: &str, file: &str) -> QueryResult {
        QueryResult {
            file_path: file.to_string(),
            function_name: name.to_string(),
            signature: format!("fn {}()", name),
            definition_type: def_type.to_string(),
            doc_comment: None,
            start_line: 1,
            end_line: 10,
            language: "rust".to_string(),
            tdg_score: 0.5,
            tdg_grade: "C".to_string(),
            complexity: 5,
            big_o: "O(n)".to_string(),
            satd_count: 0,
            loc: 10,
            relevance_score: 0.0,
            source: Some(source.to_string()),
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
            coverage_exclusion:
                crate::services::agent_context::query::coverage_exclusion::CoverageExclusion::None,
            coverage_excluded: false,
            cross_project_callers: 0,
            io_classification: String::new(),
            io_patterns: Vec::new(),
            suggested_module: String::new(),
            contract_level: None,
            contract_equation: None,
        }
    }

    #[test]
    fn test_classify_io_pure_function() {
        let source = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let (class, patterns) = classify_io(source);
        assert_eq!(class, "PURE");
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_classify_io_print() {
        let source = r#"fn greet() { println!("hello"); }"#;
        let (class, patterns) = classify_io(source);
        assert_eq!(class, "IO");
        assert!(patterns.contains(&"PRINT".to_string()));
    }

    #[test]
    fn test_classify_io_filesystem() {
        let source = r#"fn read_file() { let f = File::open("test.txt"); }"#;
        let (class, patterns) = classify_io(source);
        assert_eq!(class, "IO");
        assert!(patterns.contains(&"FS".to_string()));
    }

    #[test]
    fn test_classify_io_multiple_patterns() {
        let source =
            r#"fn do_stuff() { println!("hi"); let f = File::open("x"); Command::new("ls"); }"#;
        let (class, patterns) = classify_io(source);
        assert_eq!(class, "IO");
        assert!(patterns.contains(&"PRINT".to_string()));
        assert!(patterns.contains(&"FS".to_string()));
        assert!(patterns.contains(&"PROCESS".to_string()));
    }

    #[test]
    fn test_classify_io_database() {
        let source = r#"fn query_db() { rusqlite::Connection::open("db"); }"#;
        let (class, patterns) = classify_io(source);
        assert_eq!(class, "IO");
        assert!(patterns.contains(&"DB".to_string()));
    }

    #[test]
    fn test_group_by_prefix_minimum_three() {
        let results = vec![
            make_result("handle_get", "function", "fn x() {}", "src/handlers.rs"),
            make_result("handle_post", "function", "fn x() {}", "src/handlers.rs"),
        ];
        let groups = group_by_prefix(&results);
        // Only 2 members, should be empty
        assert!(groups.is_empty());

        let results3 = vec![
            make_result("handle_get", "function", "fn x() {}", "src/handlers.rs"),
            make_result("handle_post", "function", "fn x() {}", "src/handlers.rs"),
            make_result("handle_delete", "function", "fn x() {}", "src/handlers.rs"),
        ];
        let groups3 = group_by_prefix(&results3);
        assert!(groups3.contains_key("handle"));
        assert_eq!(groups3["handle"].len(), 3);
    }

    #[test]
    fn test_group_by_prefix_ignores_non_functions() {
        let results = vec![
            make_result("handle_get", "function", "fn x() {}", "src/lib.rs"),
            make_result("handle_post", "function", "fn x() {}", "src/lib.rs"),
            make_result("handle_delete", "function", "fn x() {}", "src/lib.rs"),
            make_result(
                "HandleConfig",
                "struct",
                "struct HandleConfig {}",
                "src/lib.rs",
            ),
        ];
        let groups = group_by_prefix(&results);
        // HandleConfig is a struct, should not be in the "Handle" group
        // But "handle" prefix group should have 3 functions
        assert!(groups.contains_key("handle"));
        assert_eq!(groups["handle"].len(), 3);
    }

    #[test]
    fn test_build_extraction_groups_max_lines() {
        let mut results = vec![
            make_result("parse_header", "function", "fn x() {}", "src/parser.rs"),
            make_result("parse_body", "function", "fn x() {}", "src/parser.rs"),
            make_result("parse_footer", "function", "fn x() {}", "src/parser.rs"),
        ];
        // Set LOC to 200 each = 600 total
        for r in &mut results {
            r.loc = 200;
        }
        classify_all_results(&mut results);

        let prefix_groups = group_by_prefix(&results);
        let cluster_groups = HashMap::new();

        // With max_module_lines=500, should trim
        let groups = build_extraction_groups(&results, &prefix_groups, &cluster_groups, 500);
        // Group should exist but be trimmed to fit within 500 lines
        // 200 + 200 = 400 <= 500, so 2 functions fit, but < 3 means group is dropped
        assert!(groups.is_empty() || groups[0].total_loc <= 500);

        // With max_module_lines=700, all 3 fit
        let groups = build_extraction_groups(&results, &prefix_groups, &cluster_groups, 700);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].functions.len(), 3);
    }

    #[test]
    fn test_longest_common_prefix() {
        assert_eq!(
            longest_common_prefix(&["handle_get", "handle_post", "handle_delete"]),
            "handle"
        );
        assert_eq!(
            longest_common_prefix(&["parse_header", "parse_body"]),
            "parse"
        );
        assert_eq!(longest_common_prefix(&["abc", "def"]), "");
        assert_eq!(longest_common_prefix(&[]), "");
    }
}
