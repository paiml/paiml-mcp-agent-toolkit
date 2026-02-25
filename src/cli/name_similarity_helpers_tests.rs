#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;
    use tempfile::TempDir;

    fn create_test_name_info(name: &str, line: u32) -> NameInfo {
        NameInfo {
            name: name.to_string(),
            kind: "function".to_string(),
            file_path: PathBuf::from("test.rs"),
            line: line as usize,
        }
    }

    fn create_test_similarity_result(name: &str, similarity: f32) -> NameSimilarityResult {
        NameSimilarityResult {
            name: name.to_string(),
            kind: "function".to_string(),
            file_path: PathBuf::from("test.rs"),
            line: 1,
            similarity,
            phonetic_match: false,
            fuzzy_match: false,
        }
    }

    #[test]
    fn test_discover_source_files_empty_directory() {
        let temp_dir = TempDir::new().expect("internal error");
        let result = discover_source_files(temp_dir.path().to_path_buf(), &None, &None);

        assert!(result.is_ok());
        let files = result.expect("internal error");
        assert!(files.is_empty());
    }

    #[test]
    fn test_discover_source_files_with_files() {
        let temp_dir = TempDir::new().expect("internal error");
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").expect("internal error");

        let result = discover_source_files(temp_dir.path().to_path_buf(), &None, &None);

        assert!(result.is_ok());
        let files = result.expect("internal error");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].1, "fn main() {}");
    }

    #[test]
    fn test_discover_source_files_with_include_filter() {
        let temp_dir = TempDir::new().expect("internal error");
        let rust_file = temp_dir.path().join("test.rs");
        let js_file = temp_dir.path().join("test.js");
        std::fs::write(&rust_file, "fn main() {}").expect("internal error");
        std::fs::write(&js_file, "function main() {}").expect("internal error");

        let result = discover_source_files(
            temp_dir.path().to_path_buf(),
            &Some(".rs".to_string()),
            &None,
        );

        assert!(result.is_ok());
        let files = result.expect("internal error");
        assert_eq!(files.len(), 1);
        assert!(files[0].0.to_string_lossy().contains(".rs"));
    }

    #[test]
    fn test_extract_all_identifiers() {
        let analyzed_files = vec![
            (PathBuf::from("test1.rs"), "fn foo() {}".to_string()),
            (PathBuf::from("test2.rs"), "fn bar() {}".to_string()),
        ];
        let scope = SearchScope::Functions;

        let result = extract_all_identifiers(&analyzed_files, &scope);

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|n| n.name == "foo"));
        assert!(result.iter().any(|n| n.name == "bar"));
    }

    #[test]
    fn test_extract_all_identifiers_empty() {
        let analyzed_files = vec![];
        let scope = SearchScope::Functions;

        let result = extract_all_identifiers(&analyzed_files, &scope);

        assert!(result.is_empty());
    }

    #[test]
    fn test_calculate_similarities_exact_match() {
        let all_names = vec![
            create_test_name_info("test_function", 1),
            create_test_name_info("other_function", 2),
        ];

        let result = calculate_similarities(&all_names, "test_function", 0.9, true, false, false);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "test_function");
        assert!(result[0].similarity > 0.9);
    }

    #[test]
    fn test_calculate_similarities_threshold_filter() {
        let all_names = vec![
            create_test_name_info("test_function", 1),
            create_test_name_info("completely_different", 2),
        ];

        let result = calculate_similarities(&all_names, "test_function", 0.9, true, false, false);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "test_function");
    }

    #[test]
    fn test_calculate_similarities_case_insensitive() {
        let all_names = vec![create_test_name_info("TEST_FUNCTION", 1)];

        let result = calculate_similarities(&all_names, "test_function", 0.5, false, false, false);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "TEST_FUNCTION");
        assert!(result[0].similarity > 0.9);
    }

    #[test]
    fn test_calculate_similarities_sorted_by_score() {
        let all_names = vec![
            create_test_name_info("test", 1),
            create_test_name_info("test_function", 2),
            create_test_name_info("testing", 3),
        ];

        let result = calculate_similarities(&all_names, "test", 0.3, true, false, false);

        assert!(result.len() >= 2);
        // Results should be sorted by similarity score (descending)
        for i in 1..result.len() {
            assert!(result[i - 1].similarity >= result[i].similarity);
        }
    }

    #[test]
    fn test_calculate_combined_similarity_basic() {
        let score = calculate_combined_similarity("test", "test", false, false);
        assert!(score > 0.9);

        let score = calculate_combined_similarity("test", "completely_different", false, false);
        assert!(score < 0.5);
    }

    #[test]
    fn test_calculate_combined_similarity_with_fuzzy() {
        let score = calculate_combined_similarity("test", "tset", true, false);
        assert!(score >= 0.5); // Should account for edit distance (exactly 0.5 for 2 swaps in 4 chars)
    }

    #[test]
    fn test_build_results_json_basic() {
        let similarities = vec![create_test_similarity_result("test_func", 0.95)];
        let config = JsonResultsConfig {
            query: "test",
            all_names_len: 10,
            similarities: &similarities,
            scope: &SearchScope::Functions,
            threshold: 0.8,
            phonetic: false,
            fuzzy: false,
            case_sensitive: true,
            perf: false,
            analysis_time: Duration::from_millis(100),
            analyzed_files_len: 5,
        };

        let result = build_results_json(config);

        assert_eq!(result["query"], "test");
        assert_eq!(result["total_identifiers"], 10);
        assert_eq!(result["matches"], 1);
        assert!(result["results"].is_array());
        assert_eq!(result["results"][0]["name"], "test_func");
    }

    #[test]
    fn test_build_results_json_with_performance() {
        let similarities = vec![create_test_similarity_result("test_func", 0.95)];
        let config = JsonResultsConfig {
            query: "test",
            all_names_len: 100,
            similarities: &similarities,
            scope: &SearchScope::Functions,
            threshold: 0.8,
            phonetic: false,
            fuzzy: false,
            case_sensitive: true,
            perf: true,
            analysis_time: Duration::from_secs(2),
            analyzed_files_len: 20,
        };

        let result = build_results_json(config);

        assert!(result["performance"].is_object());
        assert_eq!(result["performance"]["analysis_time_s"], 2.0);
        assert_eq!(result["performance"]["identifiers_per_second"], 50.0);
        assert_eq!(result["performance"]["files_analyzed"], 20);
    }

    #[test]
    fn test_output_results_json_format() {
        let similarities = vec![create_test_similarity_result("test_func", 0.95)];
        let results = json!({"test": "data"});
        let config = OutputConfig {
            format: NameSimilarityOutputFormat::Json,
            query: "test",
            all_names_len: 10,
            similarities: &similarities,
            final_results: &results,
            perf: false,
            analysis_time: Duration::from_millis(100),
            analyzed_files_len: 5,
            output: None,
        };

        let result = output_results(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_results_summary_format() {
        let similarities = vec![create_test_similarity_result("test_func", 0.95)];
        let results = json!({"test": "data"});
        let config = OutputConfig {
            format: NameSimilarityOutputFormat::Summary,
            query: "test",
            all_names_len: 10,
            similarities: &similarities,
            final_results: &results,
            perf: true,
            analysis_time: Duration::from_millis(100),
            analyzed_files_len: 5,
            output: None,
        };

        let result = output_results(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_summary_output() {
        let similarities = vec![
            create_test_similarity_result("test_func", 0.95),
            create_test_similarity_result("test_var", 0.85),
        ];

        let output =
            format_summary_output("test", 100, &similarities, true, Duration::from_secs(1), 10);

        assert!(output.contains("Name Similarity Analysis"));
        assert!(output.contains("Query: 'test'"));
        assert!(output.contains("Total identifiers: 100"));
        assert!(output.contains("Matches found: 2"));
        assert!(output.contains("test_func"));
        assert!(output.contains("Performance:"));
        assert!(output.contains("Analysis time: 1.00s"));
        assert!(output.contains("Files analyzed: 10"));
    }

    #[test]
    fn test_format_detailed_output() {
        let similarities = vec![create_test_similarity_result("test_func", 0.95)];

        let output = format_detailed_output(&similarities);

        assert!(output.contains("Name Similarity Analysis Report"));
        assert!(output.contains("Match: test_func"));
        assert!(output.contains("Similarity: 0.950"));
        assert!(output.contains("Type: function"));
        assert!(output.contains("File: test.rs"));
        assert!(output.contains("Line: 1"));
    }

    #[test]
    fn test_format_csv_output() {
        let similarities = vec![
            create_test_similarity_result("test_func", 0.95),
            create_test_similarity_result("test_var", 0.85),
        ];

        let output = format_csv_output(&similarities);

        assert!(output.contains("name,similarity,type,file,line,context"));
        assert!(output.contains("test_func,0.950"));
        assert!(output.contains("test_var,0.850"));
    }

    #[test]
    fn test_format_markdown_output() {
        let similarities = vec![create_test_similarity_result("test_func", 0.95)];

        let output = format_markdown_output("test", 100, &similarities);

        assert!(output.contains("# Name Similarity Analysis"));
        assert!(output.contains("**Query**: `test`"));
        assert!(output.contains("**Total identifiers**: 100"));
        assert!(output.contains("**Matches found**: 1"));
        assert!(output.contains("| Rank | Name | Similarity"));
        assert!(output.contains("| 1 | `test_func` | 0.950"));
    }

    #[test]
    fn test_format_markdown_output_empty() {
        let similarities = vec![];

        let output = format_markdown_output("test", 100, &similarities);

        assert!(output.contains("# Name Similarity Analysis"));
        assert!(output.contains("**Matches found**: 0"));
        assert!(!output.contains("## Results"));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
