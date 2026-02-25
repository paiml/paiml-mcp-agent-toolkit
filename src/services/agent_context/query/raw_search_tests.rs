// Tests for raw file search functionality.
// Included into raw_search.rs under #[cfg(test)] — shares its module scope.

mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let dir = TempDir::new().unwrap();
        // Create a Rust file
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("main.rs"),
            "use serde::Serialize;\n\nconst TIMEOUT: u32 = 30;\n\nfn main() {\n    println!(\"hello\");\n}\n",
        ).unwrap();
        // Create a TOML file
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = \"1.0\"\n",
        )
        .unwrap();
        // Create a markdown file
        fs::write(
            dir.path().join("README.md"),
            "# Test Project\n\nThis has a TIMEOUT of 30 seconds.\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn test_raw_search_literal() {
        let dir = create_test_project();
        let opts = RawSearchOptions {
            pattern: "TIMEOUT",
            literal: true,
            case_insensitive: false,
            before_context: 0,
            after_context: 0,
            limit: 100,
            language_filter: None,
            exclude_file_pattern: None,
            exclude_pattern: None,
            files_with_matches: false,
            count_mode: false,
        };
        let result = raw_search(dir.path(), &opts).unwrap();
        match result {
            RawSearchOutput::Lines(lines) => {
                assert!(lines.len() >= 2, "Should find TIMEOUT in .rs and .md");
                let files: Vec<&str> = lines.iter().map(|r| r.file_path.as_str()).collect();
                assert!(files.iter().any(|f| f.ends_with(".rs")));
                assert!(files.iter().any(|f| f.ends_with(".md")));
            }
            _ => panic!("Expected Lines output"),
        }
    }

    #[test]
    fn test_raw_search_regex() {
        let dir = create_test_project();
        let opts = RawSearchOptions {
            pattern: r"version\s*=",
            literal: false,
            case_insensitive: false,
            before_context: 0,
            after_context: 0,
            limit: 100,
            language_filter: None,
            exclude_file_pattern: None,
            exclude_pattern: None,
            files_with_matches: false,
            count_mode: false,
        };
        let result = raw_search(dir.path(), &opts).unwrap();
        match result {
            RawSearchOutput::Lines(lines) => {
                assert!(!lines.is_empty());
                assert!(lines[0].file_path.contains("Cargo.toml"));
            }
            _ => panic!("Expected Lines output"),
        }
    }

    #[test]
    fn test_raw_search_files_with_matches() {
        let dir = create_test_project();
        let opts = RawSearchOptions {
            pattern: "serde",
            literal: true,
            case_insensitive: false,
            before_context: 0,
            after_context: 0,
            limit: 100,
            language_filter: None,
            exclude_file_pattern: None,
            exclude_pattern: None,
            files_with_matches: true,
            count_mode: false,
        };
        let result = raw_search(dir.path(), &opts).unwrap();
        match result {
            RawSearchOutput::Files(files) => {
                assert!(files.len() >= 2, "serde in main.rs and Cargo.toml");
            }
            _ => panic!("Expected Files output"),
        }
    }

    #[test]
    fn test_raw_search_count_mode() {
        let dir = create_test_project();
        let opts = RawSearchOptions {
            pattern: "serde",
            literal: true,
            case_insensitive: false,
            before_context: 0,
            after_context: 0,
            limit: 100,
            language_filter: None,
            exclude_file_pattern: None,
            exclude_pattern: None,
            files_with_matches: false,
            count_mode: true,
        };
        let result = raw_search(dir.path(), &opts).unwrap();
        match result {
            RawSearchOutput::Counts(counts) => {
                assert!(!counts.is_empty());
                for c in &counts {
                    assert!(c.count > 0);
                }
            }
            _ => panic!("Expected Counts output"),
        }
    }

    #[test]
    fn test_raw_search_language_filter() {
        let dir = create_test_project();
        let opts = RawSearchOptions {
            pattern: "TIMEOUT",
            literal: true,
            case_insensitive: false,
            before_context: 0,
            after_context: 0,
            limit: 100,
            language_filter: Some("rust"),
            exclude_file_pattern: None,
            exclude_pattern: None,
            files_with_matches: false,
            count_mode: false,
        };
        let result = raw_search(dir.path(), &opts).unwrap();
        match result {
            RawSearchOutput::Lines(lines) => {
                assert!(!lines.is_empty());
                for l in &lines {
                    assert!(l.file_path.ends_with(".rs"), "Should only find .rs files");
                }
            }
            _ => panic!("Expected Lines output"),
        }
    }

    #[test]
    fn test_raw_search_context_lines() {
        let dir = create_test_project();
        let opts = RawSearchOptions {
            pattern: "TIMEOUT",
            literal: true,
            case_insensitive: false,
            before_context: 1,
            after_context: 1,
            limit: 100,
            language_filter: Some("rust"),
            exclude_file_pattern: None,
            exclude_pattern: None,
            files_with_matches: false,
            count_mode: false,
        };
        let result = raw_search(dir.path(), &opts).unwrap();
        match result {
            RawSearchOutput::Lines(lines) => {
                assert!(!lines.is_empty());
                let first = &lines[0];
                assert!(first.line_content.contains("TIMEOUT"));
                // Should have context
                assert!(!first.context_before.is_empty() || first.line_number == 1);
                assert!(!first.context_after.is_empty());
            }
            _ => panic!("Expected Lines output"),
        }
    }

    #[test]
    fn test_raw_search_case_insensitive() {
        let dir = create_test_project();
        let opts = RawSearchOptions {
            pattern: "timeout",
            literal: true,
            case_insensitive: true,
            before_context: 0,
            after_context: 0,
            limit: 100,
            language_filter: None,
            exclude_file_pattern: None,
            exclude_pattern: None,
            files_with_matches: false,
            count_mode: false,
        };
        let result = raw_search(dir.path(), &opts).unwrap();
        match result {
            RawSearchOutput::Lines(lines) => {
                // Should match TIMEOUT (uppercase) with case-insensitive search
                assert!(lines.len() >= 2);
            }
            _ => panic!("Expected Lines output"),
        }
    }

    #[test]
    fn test_raw_search_exclude_pattern() {
        let dir = create_test_project();
        let opts = RawSearchOptions {
            pattern: "serde",
            literal: true,
            case_insensitive: false,
            before_context: 0,
            after_context: 0,
            limit: 100,
            language_filter: None,
            exclude_file_pattern: None,
            exclude_pattern: Some("Serialize"),
            files_with_matches: false,
            count_mode: false,
        };
        let result = raw_search(dir.path(), &opts).unwrap();
        match result {
            RawSearchOutput::Lines(lines) => {
                // The "use serde::Serialize" line should be excluded
                for l in &lines {
                    assert!(!l.line_content.contains("Serialize"));
                }
            }
            _ => panic!("Expected Lines output"),
        }
    }

    #[test]
    fn test_is_within_indexed_function() {
        let indexed = vec![super::super::types::QueryResult {
            file_path: "src/main.rs".to_string(),
            function_name: "main".to_string(),
            signature: "fn main()".to_string(),
            definition_type: "function".to_string(),
            doc_comment: None,
            start_line: 5,
            end_line: 7,
            language: "rust".to_string(),
            tdg_score: 1.0,
            tdg_grade: "A".to_string(),
            complexity: 1,
            big_o: "O(1)".to_string(),
            satd_count: 0,
            loc: 3,
            relevance_score: 1.0,
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
        }];

        // Line 6 is within the function (5-7)
        assert!(is_within_indexed_function("src/main.rs", 6, &indexed));
        // Line 3 is outside the function
        assert!(!is_within_indexed_function("src/main.rs", 3, &indexed));
        // Different file
        assert!(!is_within_indexed_function("src/lib.rs", 6, &indexed));
    }
}
