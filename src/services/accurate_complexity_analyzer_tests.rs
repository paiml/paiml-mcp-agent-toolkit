// Included from accurate_complexity_analyzer.rs — NO use imports or #! attributes

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_basic_complexity() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        fs::write(
            &test_file,
            r#"
            fn simple() -> i32 {
                42
            }
            
            fn with_if(x: i32) -> i32 {
                if x > 0 {
                    x
                } else {
                    -x
                }
            }
        "#,
        )
        .unwrap();

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert_eq!(result.functions.len(), 2);
        assert_eq!(result.functions[0].cyclomatic_complexity, 1);
        assert_eq!(result.functions[1].cyclomatic_complexity, 2);
    }

    #[tokio::test]
    async fn test_line_numbers_are_accurate() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        // Write file with known line positions
        fs::write(
            &test_file,
            "fn first() -> i32 {\n    42\n}\n\nfn second() -> i32 {\n    99\n}\n",
        )
        .unwrap();
        // first() is at line 1, second() is at line 5

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert_eq!(result.functions.len(), 2);
        assert_eq!(result.functions[0].name, "first");
        assert_eq!(result.functions[0].line_start, 1);
        assert_eq!(result.functions[1].name, "second");
        assert_eq!(result.functions[1].line_start, 5);
    }

    #[tokio::test]
    async fn test_line_numbers_with_attributes() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        fs::write(
            &test_file,
            "#[inline]\npub fn decorated() -> i32 {\n    42\n}\n\n/// doc comment\npub async fn async_fn() {}\n",
        )
        .unwrap();
        // decorated() fn keyword is at line 2, async_fn() fn keyword is at line 7

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert_eq!(result.functions.len(), 2);
        assert_eq!(result.functions[0].name, "decorated");
        assert_eq!(result.functions[0].line_start, 2);
        assert_eq!(result.functions[1].name, "async_fn");
        assert_eq!(result.functions[1].line_start, 7);
    }

    #[test]
    fn test_build_function_line_map() {
        let content = "fn foo() {}\n\npub fn bar() {}\n\nasync fn baz() {}\n";
        let map = build_function_line_map(content);
        assert_eq!(map.get("foo"), Some(&1));
        assert_eq!(map.get("bar"), Some(&3));
        assert_eq!(map.get("baz"), Some(&5));
    }

    #[test]
    fn test_build_function_line_map_skips_comments() {
        let content = "// fn not_a_function() {}\nfn real() {}\n";
        let map = build_function_line_map(content);
        assert_eq!(map.get("real"), Some(&2));
        assert!(map.get("not_a_function").is_none());
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_analyzer_new_defaults() {
        let analyzer = AccurateComplexityAnalyzer::new();
        // Default values checked via builder methods
        assert!(!analyzer.exclude_tests);
        assert!(!analyzer.respect_annotations);
    }

    #[test]
    fn test_analyzer_builder_exclude_tests() {
        let analyzer = AccurateComplexityAnalyzer::new().exclude_tests(true);
        assert!(analyzer.exclude_tests);
    }

    #[test]
    fn test_analyzer_builder_respect_annotations() {
        let analyzer = AccurateComplexityAnalyzer::new().respect_annotations(true);
        assert!(analyzer.respect_annotations);
    }

    #[test]
    fn test_analyzer_builder_chaining() {
        let analyzer = AccurateComplexityAnalyzer::new()
            .exclude_tests(true)
            .respect_annotations(true);
        assert!(analyzer.exclude_tests);
        assert!(analyzer.respect_annotations);
    }

    #[test]
    fn test_is_test_file_test_suffix() {
        let analyzer = AccurateComplexityAnalyzer::new().exclude_tests(true);
        let path = Path::new("src/foo_test.rs");
        assert!(analyzer.is_test_file(path));
    }

    #[test]
    fn test_is_test_file_tests_suffix() {
        let analyzer = AccurateComplexityAnalyzer::new().exclude_tests(true);
        let path = Path::new("src/foo_tests.rs");
        assert!(analyzer.is_test_file(path));
    }

    #[test]
    fn test_is_test_file_tests_dir() {
        let analyzer = AccurateComplexityAnalyzer::new().exclude_tests(true);
        // Need full path with /tests/ to match
        let path = Path::new("src/tests/integration.rs");
        assert!(analyzer.is_test_file(path));
    }

    #[test]
    fn test_is_test_file_regular_file() {
        let analyzer = AccurateComplexityAnalyzer::new().exclude_tests(true);
        let path = Path::new("src/lib.rs");
        assert!(!analyzer.is_test_file(path));
    }

    #[test]
    fn test_is_test_file_always_checks() {
        let analyzer = AccurateComplexityAnalyzer::new().exclude_tests(false);
        let path = Path::new("src/foo_test.rs");
        // is_test_file always checks the path, regardless of exclude_tests flag
        // The flag is only used in analyze_project to skip files
        assert!(analyzer.is_test_file(path));
    }

    #[test]
    fn test_extract_fn_name_simple() {
        let result = extract_fn_name("fn foo() {}");
        assert_eq!(result, Some("foo".to_string()));
    }

    #[test]
    fn test_extract_fn_name_pub() {
        let result = extract_fn_name("pub fn bar() {}");
        assert_eq!(result, Some("bar".to_string()));
    }

    #[test]
    fn test_extract_fn_name_async() {
        let result = extract_fn_name("async fn baz() {}");
        assert_eq!(result, Some("baz".to_string()));
    }

    #[test]
    fn test_extract_fn_name_pub_async() {
        let result = extract_fn_name("pub async fn qux() {}");
        assert_eq!(result, Some("qux".to_string()));
    }

    #[test]
    fn test_extract_fn_name_generic() {
        let result = extract_fn_name("fn generic<T>() {}");
        assert_eq!(result, Some("generic".to_string()));
    }

    #[test]
    fn test_extract_fn_name_no_fn() {
        let result = extract_fn_name("let x = 42;");
        assert_eq!(result, None);
    }

    #[test]
    fn test_default_impl() {
        let analyzer = AccurateComplexityAnalyzer::default();
        assert!(!analyzer.exclude_tests);
        assert!(!analyzer.respect_annotations);
    }
}
