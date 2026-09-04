#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod path_detection_tests {
    //! Tests for path detection, source file classification, and edge cases.

    use super::*;
    use crate::models::churn::{CodeChurnAnalysis, FileChurnMetrics};
    use chrono::Utc;
    use std::path::PathBuf;

    // Shared fixtures (duplicated minimally for include!() isolation)

    fn create_empty_summary() -> crate::models::churn::ChurnSummary {
        crate::models::churn::ChurnSummary {
            total_commits: 0,
            total_files_changed: 0,
            hotspot_files: vec![],
            stable_files: vec![],
            author_contributions: std::collections::BTreeMap::new(),
            mean_churn_score: 0.0,
            variance_churn_score: 0.0,
            stddev_churn_score: 0.0,
        }
    }

    fn create_test_file_metrics(
        path: &str,
        commit_count: usize,
        churn_score: f32,
    ) -> FileChurnMetrics {
        let now = Utc::now();
        FileChurnMetrics {
            path: PathBuf::from(path),
            relative_path: path.to_string(),
            commit_count,
            unique_authors: vec!["dev1".to_string(), "dev2".to_string()],
            additions: 100,
            deletions: 50,
            churn_score,
            last_modified: now,
            first_seen: now,
        }
    }

    fn create_test_analysis(files: Vec<FileChurnMetrics>) -> CodeChurnAnalysis {
        CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days: 30,
            repository_root: PathBuf::from("/test/repo"),
            files,
            summary: create_empty_summary(),
        }
    }

    // ============================================================================
    // is_source_file Tests
    // ============================================================================

    mod is_source_file_tests {
        use super::*;

        #[test]
        fn test_rust_source_file() {
            assert!(is_source_file(Path::new("src/main.rs")));
            assert!(is_source_file(Path::new("lib.rs")));
        }

        #[test]
        fn test_javascript_source_file() {
            assert!(is_source_file(Path::new("src/index.js")));
            assert!(is_source_file(Path::new("app.js")));
        }

        #[test]
        fn test_typescript_source_file() {
            assert!(is_source_file(Path::new("src/app.ts")));
            assert!(is_source_file(Path::new("component.ts")));
        }

        #[test]
        fn test_python_source_file() {
            assert!(is_source_file(Path::new("src/main.py")));
            assert!(is_source_file(Path::new("app.py")));
        }

        #[test]
        fn test_java_source_file() {
            assert!(is_source_file(Path::new("src/Main.java")));
            assert!(is_source_file(Path::new("App.java")));
        }

        #[test]
        fn test_cpp_source_files() {
            assert!(is_source_file(Path::new("src/main.cpp")));
            assert!(is_source_file(Path::new("lib.c")));
        }

        #[test]
        fn test_go_source_file() {
            assert!(is_source_file(Path::new("main.go")));
            assert!(is_source_file(Path::new("pkg/server.go")));
        }

        #[test]
        fn test_kotlin_source_file() {
            assert!(is_source_file(Path::new("Main.kt")));
        }

        #[test]
        fn test_swift_source_file() {
            assert!(is_source_file(Path::new("App.swift")));
        }

        #[test]
        fn test_php_source_file() {
            assert!(is_source_file(Path::new("index.php")));
        }

        #[test]
        fn test_ruby_source_file() {
            assert!(is_source_file(Path::new("app.rb")));
        }

        #[test]
        fn test_scala_source_file() {
            assert!(is_source_file(Path::new("Main.scala")));
        }

        #[test]
        fn test_non_source_files() {
            assert!(!is_source_file(Path::new("README.md")));
            assert!(!is_source_file(Path::new("Cargo.toml")));
            assert!(!is_source_file(Path::new("package.json")));
            assert!(!is_source_file(Path::new("data.csv")));
            assert!(!is_source_file(Path::new("image.png")));
        }

        #[test]
        fn test_file_without_extension() {
            assert!(!is_source_file(Path::new("Makefile")));
            assert!(!is_source_file(Path::new("Dockerfile")));
        }

        #[test]
        fn test_hidden_files() {
            assert!(!is_source_file(Path::new(".gitignore")));
        }
    }

    // ============================================================================
    // Test Path Detection (is_test_path and is_test_filename)
    // ============================================================================

    mod test_path_detection_tests {
        use super::*;

        #[test]
        fn test_tests_directory() {
            assert!(!is_source_file(Path::new("/project/tests/test_main.rs")));
            assert!(!is_source_file(Path::new("/project/test/unit.rs")));
        }

        #[test]
        fn test_examples_directory() {
            assert!(!is_source_file(Path::new("/project/examples/demo.rs")));
        }

        #[test]
        fn test_benches_directory() {
            assert!(!is_source_file(Path::new("/project/benches/benchmark.rs")));
        }

        #[test]
        fn test_fixtures_directory() {
            assert!(!is_source_file(Path::new("/project/fixtures/data.rs")));
        }

        #[test]
        fn test_testdata_directory() {
            assert!(!is_source_file(Path::new("/project/testdata/mock.rs")));
            assert!(!is_source_file(Path::new("/project/test_data/mock.rs")));
        }

        #[test]
        fn test_debug_test_directory() {
            assert!(!is_source_file(Path::new("/project/debug_test/helper.rs")));
        }

        #[test]
        fn test_jest_tests_directory() {
            assert!(!is_source_file(Path::new("/project/__tests__/app.js")));
        }

        #[test]
        fn test_rust_test_suffix() {
            assert!(!is_source_file(Path::new("main_test.rs")));
            assert!(!is_source_file(Path::new("main_tests.rs")));
        }

        #[test]
        fn test_rust_test_prefix() {
            assert!(!is_source_file(Path::new("test_main.rs")));
        }

        #[test]
        fn test_rust_test_infix() {
            assert!(!is_source_file(Path::new("module_test_helper.rs")));
        }

        #[test]
        fn test_js_test_suffix() {
            assert!(!is_source_file(Path::new("app.test.js")));
            assert!(!is_source_file(Path::new("app.spec.js")));
        }

        #[test]
        fn test_python_test_suffix() {
            assert!(!is_source_file(Path::new("main_test.py")));
        }

        #[test]
        fn test_java_test_suffix() {
            assert!(!is_source_file(Path::new("MainTest.java")));
        }

        #[test]
        fn test_regular_source_in_src_directory() {
            assert!(is_source_file(Path::new("/project/src/main.rs")));
            assert!(is_source_file(Path::new("/project/src/lib/core.rs")));
        }
    }

    // ============================================================================
    // has_source_extension Tests (Private function tested via is_source_file)
    // ============================================================================

    mod extension_tests {
        use super::*;

        #[test]
        fn test_all_supported_extensions() {
            let extensions = [
                "rs", "js", "ts", "py", "java", "cpp", "c", "go", "kt", "swift", "php", "rb",
                "scala",
            ];

            for ext in extensions.iter() {
                let path = format!("file.{}", ext);
                assert!(
                    is_source_file(Path::new(&path)),
                    "Extension .{} should be recognized as source",
                    ext
                );
            }
        }

        #[test]
        fn test_uppercase_extensions() {
            let path = Path::new("FILE.RS");
            assert!(!is_source_file(path));
        }
    }

    // ============================================================================
    // Edge Cases and Error Paths
    // ============================================================================

    mod edge_cases {
        use super::*;

        #[test]
        fn test_path_with_no_components() {
            assert!(!is_source_file(Path::new("")));
        }

        #[test]
        fn test_path_with_dots_only() {
            assert!(!is_source_file(Path::new("...")));
            assert!(!is_source_file(Path::new("..")));
        }

        #[test]
        fn test_deeply_nested_path() {
            assert!(is_source_file(Path::new(
                "/a/very/deeply/nested/path/to/source.rs"
            )));
        }

        #[test]
        fn test_path_with_unicode() {
            assert!(is_source_file(Path::new("/src/模块.rs")));
        }

        #[test]
        fn test_format_with_unicode_in_file_names() {
            let files = vec![create_test_file_metrics("src/文件.rs", 10, 0.5)];
            let analysis = create_test_analysis(files);

            let result = format_churn_markdown(&analysis);
            assert!(result.is_ok());
        }

        #[test]
        fn test_format_with_special_characters_in_paths() {
            let files = vec![create_test_file_metrics("src/file-name_123.rs", 10, 0.5)];
            let analysis = create_test_analysis(files);

            let result = format_churn_markdown(&analysis);
            assert!(result.is_ok());
            assert!(result.unwrap().contains("file-name_123.rs"));
        }

        #[test]
        fn test_empty_author_contributions_skips_section() {
            let mut analysis = create_test_analysis(vec![]);
            analysis.summary.author_contributions = std::collections::BTreeMap::new();

            let result = format_churn_markdown(&analysis).unwrap();

            assert!(!result.contains("## Author Contributions"));
        }

        #[test]
        fn test_sorts_authors_by_contribution() {
            let mut contributions = std::collections::BTreeMap::new();
            contributions.insert("low_contributor".to_string(), 5);
            contributions.insert("high_contributor".to_string(), 100);
            contributions.insert("mid_contributor".to_string(), 50);

            let mut analysis = create_test_analysis(vec![]);
            analysis.summary.author_contributions = contributions;

            let result = format_churn_markdown(&analysis).unwrap();

            let high_pos = result.find("high_contributor").unwrap();
            let mid_pos = result.find("mid_contributor").unwrap();
            let low_pos = result.find("low_contributor").unwrap();

            assert!(
                high_pos < mid_pos,
                "high_contributor should appear before mid_contributor"
            );
            assert!(
                mid_pos < low_pos,
                "mid_contributor should appear before low_contributor"
            );
        }

        #[test]
        fn test_limits_author_list_to_15() {
            let mut contributions = std::collections::BTreeMap::new();
            for i in 0..20 {
                contributions.insert(format!("author_{:02}", i), 100 - i);
            }

            let mut analysis = create_test_analysis(vec![]);
            analysis.summary.author_contributions = contributions;

            let result = format_churn_markdown(&analysis).unwrap();

            let author_rows: Vec<&str> = result
                .lines()
                .filter(|line| line.starts_with("| author_"))
                .collect();

            assert!(
                author_rows.len() <= 15,
                "Should have at most 15 author rows"
            );
        }
    }
}
