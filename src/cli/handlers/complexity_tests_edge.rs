#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod complexity_config_additional_tests {
    use super::*;

    #[test]
    fn test_complexity_config_debug_format() {
        let config = ComplexityConfig::from_args(
            PathBuf::from("/test"),
            Some("python".to_string()),
            Some(15),
            Some(20),
            vec!["*.py".to_string()],
            60,
            10,
        );

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ComplexityConfig"));
        assert!(debug_str.contains("python"));
        assert!(debug_str.contains("15"));
        assert!(debug_str.contains("20"));
    }

    #[test]
    fn test_complexity_config_with_multiple_include_patterns() {
        let config = ComplexityConfig::from_args(
            PathBuf::from("/project"),
            None,
            None,
            None,
            vec![
                "src/**/*.rs".to_string(),
                "lib/**/*.rs".to_string(),
                "crates/**/*.rs".to_string(),
            ],
            120,
            20,
        );

        assert_eq!(config.include.len(), 3);
        assert!(config.include.contains(&"src/**/*.rs".to_string()));
        assert!(config.include.contains(&"lib/**/*.rs".to_string()));
        assert!(config.include.contains(&"crates/**/*.rs".to_string()));
    }
}

// Extended Coverage Tests - Edge Cases

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod edge_case_tests {
    use super::*;
    use crate::models::dead_code::{
        ConfidenceLevel, DeadCodeItem, DeadCodeResult, DeadCodeSummary, DeadCodeType,
        FileDeadCodeMetrics,
    };

    #[test]
    fn test_format_dead_code_as_summary_empty_result() {
        let result = DeadCodeResult {
            summary: DeadCodeSummary {
                total_files_analyzed: 0,
                files_with_dead_code: 0,
                total_dead_lines: 0,
                dead_percentage: 0.0,
                dead_functions: 0,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 0,
            },
            files: vec![],
            total_files: 0,
            analyzed_files: 0,
        };

        let summary = format_dead_code_as_summary(&result).expect("Should handle empty result");

        assert!(summary.contains("# Dead Code Analysis Summary"));
        assert!(summary.contains("**Files analyzed**: 0"));
        assert!(!summary.contains("## Top Files")); // Should not have this section
    }

    #[test]
    fn test_apply_complexity_filters_edge_cases() {
        // Test with exact threshold values
        let mut metrics = vec![create_test_file_metrics_simple("exact.rs", 20, 15)];

        // Threshold is exclusive (must exceed, not equal)
        let filtered = apply_complexity_filters(&mut metrics, Some(20), Some(15));
        assert_eq!(filtered, 1); // Should be filtered out since not exceeding
        assert!(metrics.is_empty());
    }

    #[test]
    fn test_apply_top_files_limit_larger_than_input() {
        let mut metrics = vec![
            create_test_file_metrics_simple("a.rs", 10, 10),
            create_test_file_metrics_simple("b.rs", 20, 20),
        ];

        apply_top_files_limit(&mut metrics, 100);

        // Should keep all files
        assert_eq!(metrics.len(), 2);
    }

    #[test]
    fn test_dead_code_sarif_with_empty_items() {
        let result = DeadCodeResult {
            summary: DeadCodeSummary {
                total_files_analyzed: 5,
                files_with_dead_code: 0,
                total_dead_lines: 0,
                dead_percentage: 0.0,
                dead_functions: 0,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 0,
            },
            files: vec![],
            total_files: 5,
            analyzed_files: 5,
        };

        let sarif = format_dead_code_as_sarif(&result).expect("Should handle empty files");
        let parsed: serde_json::Value = serde_json::from_str(&sarif).unwrap();

        let results = parsed["runs"][0]["results"].as_array().unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_dead_code_markdown_with_many_files() {
        // Test that file details section limits to 20 files
        let files: Vec<FileDeadCodeMetrics> = (0..30)
            .map(|i| FileDeadCodeMetrics {
                path: format!("src/file_{}.rs", i),
                dead_lines: 10,
                total_lines: 100,
                dead_percentage: 10.0,
                dead_functions: 1,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 0,
                dead_score: 10.0,
                confidence: ConfidenceLevel::Medium,
                items: vec![],
            })
            .collect();

        let section = format_dead_code_file_details_section(&files);

        // Should only contain first 20 files
        assert!(section.contains("file_0.rs"));
        assert!(section.contains("file_19.rs"));
        assert!(!section.contains("file_20.rs")); // Should not appear
    }

    #[test]
    fn test_has_complexity_violations_empty_metrics() {
        let metrics: Vec<crate::services::complexity::FileComplexityMetrics> = vec![];

        let has_violations = has_complexity_violations(&metrics, Some(10), Some(10));
        assert!(!has_violations);
    }

    fn create_test_file_metrics_simple(
        path: &str,
        cyclomatic: u16,
        cognitive: u16,
    ) -> crate::services::complexity::FileComplexityMetrics {
        crate::services::complexity::FileComplexityMetrics {
            path: path.to_string(),
            language: "rust".to_string(),
            total_complexity: crate::services::complexity::ComplexityMetrics {
                cyclomatic,
                cognitive,
                nesting_depth: 2,
                line_count: 100,
                function_count: 5,
            },
            functions: vec![crate::services::complexity::FunctionComplexity {
                name: "test_fn".to_string(),
                line: 1,
                metrics: crate::services::complexity::ComplexityMetrics {
                    cyclomatic,
                    cognitive,
                    nesting_depth: 2,
                    line_count: 20,
                    function_count: 1,
                },
            }],
            function_count: 1,
        }
    }
}

// Extended Property Tests

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod extended_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_dead_code_summary_section_always_valid(
            total_files in 0usize..1000,
            files_with_dead in 0usize..1000,
            dead_lines in 0usize..10000,
            percentage in 0.0f32..100.0f32
        ) {
            let result = crate::models::dead_code::DeadCodeResult {
                summary: crate::models::dead_code::DeadCodeSummary {
                    total_files_analyzed: total_files,
                    files_with_dead_code: files_with_dead,
                    total_dead_lines: dead_lines,
                    dead_percentage: percentage,
                    dead_functions: 0,
                    dead_classes: 0,
                    dead_modules: 0,
                    unreachable_blocks: 0,
                },
                files: vec![],
                total_files,
                analyzed_files: total_files,
            };

            let section = format_dead_code_summary_section(&result);

            prop_assert!(section.contains("Dead Code Analysis Report"));
            prop_assert!(!section.is_empty());
        }

        #[test]
        fn test_top_files_limit_never_exceeds_original(
            file_count in 1usize..100,
            limit in 1usize..50
        ) {
            let mut metrics: Vec<crate::services::complexity::FileComplexityMetrics> = (0..file_count)
                .map(|i| crate::services::complexity::FileComplexityMetrics {
                    path: format!("file{}.rs", i),
                    language: "rust".to_string(),
                    total_complexity: crate::services::complexity::ComplexityMetrics {
                        cyclomatic: (i as u16) + 1,
                        cognitive: (i as u16) + 1,
                        nesting_depth: 2,
                        line_count: 100,
                        function_count: 1,
                    },
                    functions: vec![],
                    function_count: 0,
                })
                .collect();

            let original_count = metrics.len();
            apply_top_files_limit(&mut metrics, limit);

            prop_assert!(metrics.len() <= limit.min(original_count));
        }

        #[test]
        fn test_complexity_filters_preserve_invariant(
            file_count in 1usize..50,
            threshold in 1u16..100
        ) {
            let mut metrics: Vec<crate::services::complexity::FileComplexityMetrics> = (0..file_count)
                .map(|i| crate::services::complexity::FileComplexityMetrics {
                    path: format!("file{}.rs", i),
                    language: "rust".to_string(),
                    total_complexity: crate::services::complexity::ComplexityMetrics {
                        cyclomatic: (i as u16) % 50 + 1,
                        cognitive: (i as u16) % 50 + 1,
                        nesting_depth: 2,
                        line_count: 100,
                        function_count: 1,
                    },
                    functions: vec![crate::services::complexity::FunctionComplexity {
                        name: "fn".to_string(),
                        line: 1,
                        metrics: crate::services::complexity::ComplexityMetrics {
                            cyclomatic: (i as u16) % 50 + 1,
                            cognitive: (i as u16) % 50 + 1,
                            nesting_depth: 2,
                            line_count: 20,
                            function_count: 1,
                        },
                    }],
                    function_count: 1,
                })
                .collect();

            let original_count = metrics.len();
            let filtered = apply_complexity_filters(&mut metrics, Some(threshold), Some(threshold));

            // Invariant: filtered + remaining = original
            prop_assert_eq!(filtered + metrics.len(), original_count);
        }
    }
}
