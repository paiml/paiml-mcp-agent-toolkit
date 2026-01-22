#[cfg(test)]
mod property_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_complexity_config_from_args_preserves_values(
            max_cyc in 1u16..100,
            max_cog in 1u16..100,
            timeout in 1u64..1000,
            top_files in 0usize..100
        ) {
            let config = ComplexityConfig::from_args(
                PathBuf::from("/test"),
                Some("rust".to_string()),
                Some(max_cyc),
                Some(max_cog),
                vec![],
                timeout,
                top_files,
            );

            prop_assert_eq!(config.max_cyclomatic, max_cyc);
            prop_assert_eq!(config.max_cognitive, max_cog);
            prop_assert_eq!(config.timeout, timeout);
            prop_assert_eq!(config.top_files, top_files);
        }

        #[test]
        fn test_apply_top_files_limit_respects_limit(limit in 1usize..20) {
            let mut metrics: Vec<crate::services::complexity::FileComplexityMetrics> = (0..50)
                .map(|i| create_test_metrics_simple(&format!("file{}.rs", i), i as u16))
                .collect();

            apply_top_files_limit(&mut metrics, limit);

            prop_assert!(metrics.len() <= limit);
        }

        #[test]
        fn test_is_source_code_file_deterministic(ext in "[a-z]{2,4}") {
            let path = format!("file.{}", ext);
            let result1 = is_source_code_file(&path);
            let result2 = is_source_code_file(&path);

            prop_assert_eq!(result1, result2);
        }

        #[test]
        fn test_should_include_file_empty_patterns_always_includes(path in "[a-z/]+\\.rs") {
            prop_assert!(should_include_file(&path, &[]));
        }

        #[test]
        fn test_filter_count_never_exceeds_original(count in 1usize..50) {
            let mut metrics: Vec<crate::services::complexity::FileComplexityMetrics> = (0..count)
                .map(|i| create_test_metrics_simple(&format!("file{}.rs", i), (i % 30) as u16))
                .collect();

            let original_count = metrics.len();
            let filtered = apply_complexity_filters(&mut metrics, Some(15), Some(15));

            prop_assert!(filtered + metrics.len() == original_count);
        }
    }

    fn create_test_metrics_simple(
        path: &str,
        complexity: u16,
    ) -> crate::services::complexity::FileComplexityMetrics {
        crate::services::complexity::FileComplexityMetrics {
            path: path.to_string(),
            language: "rust".to_string(),
            total_complexity: crate::services::complexity::ComplexityMetrics {
                cyclomatic: complexity,
                cognitive: complexity,
                nesting_depth: 2,
                line_count: 100,
                function_count: 5,
            },
            functions: vec![crate::services::complexity::FunctionComplexity {
                name: "func".to_string(),
                line: 1,
                metrics: crate::services::complexity::ComplexityMetrics {
                    cyclomatic: complexity,
                    cognitive: complexity,
                    nesting_depth: 2,
                    line_count: 20,
                    function_count: 1,
                },
            }],
            function_count: 1,
        }
    }
}

// Extended Coverage Tests - Dead Code Formatting

