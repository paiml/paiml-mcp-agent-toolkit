#![cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use crate::cli::handlers::big_o_handlers::filters::{
        apply_high_complexity_filter, apply_report_filters, apply_top_files_filter,
        calculate_file_complexity_scores, get_complexity_class_score, get_top_file_paths,
        group_functions_by_file, is_high_complexity_class,
    };
    use crate::cli::handlers::big_o_handlers::handlers::build_analysis_config;
    use crate::cli::handlers::big_o_handlers::output::format_big_o_detailed;
    use crate::cli::handlers::big_o_handlers::output::{
        format_analysis_output, format_big_o_summary, write_analysis_output,
    };
    use crate::cli::BigOOutputFormat;
    use crate::models::complexity_bound::{BigOClass, ComplexityBound};
    use crate::services::big_o_analyzer::{
        BigOAnalysisReport, BigOAnalyzer, ComplexityDistribution, FunctionComplexity, PatternMatch,
    };
    use std::path::{Path, PathBuf};

    // ============================================
    // Helper: strip ANSI escape sequences
    // ============================================

    fn strip_ansi(s: &str) -> String {
        let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        re.replace_all(s, "").to_string()
    }

    // ============================================
    // Helper functions for creating test data
    // ============================================

    fn create_test_function_complexity(
        name: &str,
        file: &str,
        line: usize,
        time_class: BigOClass,
        confidence: u8,
    ) -> FunctionComplexity {
        FunctionComplexity {
            function_name: name.to_string(),
            file_path: PathBuf::from(file),
            line_number: line,
            time_complexity: ComplexityBound::new(
                time_class,
                1,
                crate::models::complexity_bound::InputVariable::N,
            )
            .with_confidence(confidence),
            space_complexity: ComplexityBound::constant(),
            confidence,
            notes: vec![],
        }
    }

    fn create_test_distribution() -> ComplexityDistribution {
        ComplexityDistribution {
            constant: 10,
            logarithmic: 5,
            linear: 20,
            linearithmic: 3,
            quadratic: 7,
            cubic: 2,
            exponential: 1,
            unknown: 2,
        }
    }

    fn create_test_report_empty() -> BigOAnalysisReport {
        BigOAnalysisReport {
            analyzed_functions: 0,
            complexity_distribution: ComplexityDistribution {
                constant: 0,
                logarithmic: 0,
                linear: 0,
                linearithmic: 0,
                quadratic: 0,
                cubic: 0,
                exponential: 0,
                unknown: 0,
            },
            high_complexity_functions: vec![],
            pattern_matches: vec![],
            recommendations: vec![],
        }
    }

    fn create_test_report_with_functions() -> BigOAnalysisReport {
        BigOAnalysisReport {
            analyzed_functions: 50,
            complexity_distribution: create_test_distribution(),
            high_complexity_functions: vec![
                create_test_function_complexity(
                    "bubble_sort",
                    "src/sort.rs",
                    42,
                    BigOClass::Quadratic,
                    85,
                ),
                create_test_function_complexity(
                    "matrix_mult",
                    "src/math.rs",
                    100,
                    BigOClass::Cubic,
                    90,
                ),
                create_test_function_complexity(
                    "fib_exp",
                    "src/algo.rs",
                    15,
                    BigOClass::Exponential,
                    95,
                ),
                create_test_function_complexity(
                    "permute",
                    "src/algo.rs",
                    50,
                    BigOClass::Factorial,
                    80,
                ),
            ],
            pattern_matches: vec![
                PatternMatch {
                    pattern_name: "Sorting operation".to_string(),
                    occurrences: 5,
                    typical_complexity: BigOClass::Linearithmic,
                },
                PatternMatch {
                    pattern_name: "Binary search".to_string(),
                    occurrences: 3,
                    typical_complexity: BigOClass::Logarithmic,
                },
            ],
            recommendations: vec![
                "Consider optimizing quadratic algorithms".to_string(),
                "Review exponential complexity functions".to_string(),
            ],
        }
    }

    // ============================================
    // Tests for is_high_complexity_class
    // ============================================

    #[test]
    fn test_is_high_complexity_class_quadratic() {
        assert!(is_high_complexity_class(&BigOClass::Quadratic));
    }

    #[test]
    fn test_is_high_complexity_class_cubic() {
        assert!(is_high_complexity_class(&BigOClass::Cubic));
    }

    #[test]
    fn test_is_high_complexity_class_exponential() {
        assert!(is_high_complexity_class(&BigOClass::Exponential));
    }

    #[test]
    fn test_is_high_complexity_class_factorial() {
        assert!(is_high_complexity_class(&BigOClass::Factorial));
    }

    #[test]
    fn test_is_high_complexity_class_constant_is_not_high() {
        assert!(!is_high_complexity_class(&BigOClass::Constant));
    }

    #[test]
    fn test_is_high_complexity_class_logarithmic_is_not_high() {
        assert!(!is_high_complexity_class(&BigOClass::Logarithmic));
    }

    #[test]
    fn test_is_high_complexity_class_linear_is_not_high() {
        assert!(!is_high_complexity_class(&BigOClass::Linear));
    }

    #[test]
    fn test_is_high_complexity_class_linearithmic_is_not_high() {
        assert!(!is_high_complexity_class(&BigOClass::Linearithmic));
    }

    #[test]
    fn test_is_high_complexity_class_unknown_is_not_high() {
        assert!(!is_high_complexity_class(&BigOClass::Unknown));
    }

    // ============================================
    // Tests for get_complexity_class_score
    // ============================================

    #[test]
    fn test_get_complexity_class_score_constant() {
        assert!((get_complexity_class_score(&BigOClass::Constant) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_complexity_class_score_logarithmic() {
        assert!((get_complexity_class_score(&BigOClass::Logarithmic) - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_complexity_class_score_linear() {
        assert!((get_complexity_class_score(&BigOClass::Linear) - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_complexity_class_score_linearithmic() {
        assert!((get_complexity_class_score(&BigOClass::Linearithmic) - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_complexity_class_score_quadratic() {
        assert!((get_complexity_class_score(&BigOClass::Quadratic) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_complexity_class_score_cubic() {
        assert!((get_complexity_class_score(&BigOClass::Cubic) - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_complexity_class_score_exponential() {
        assert!((get_complexity_class_score(&BigOClass::Exponential) - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_complexity_class_score_factorial() {
        assert!((get_complexity_class_score(&BigOClass::Factorial) - 8.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_complexity_class_score_unknown() {
        // Unknown has same score as Linear
        assert!((get_complexity_class_score(&BigOClass::Unknown) - 3.0).abs() < f64::EPSILON);
    }

    // ============================================
    // Tests for build_analysis_config
    // ============================================

    #[test]
    fn test_build_analysis_config_basic() {
        let config = build_analysis_config(
            PathBuf::from("/test/project"),
            vec!["*.rs".to_string()],
            vec!["test_*".to_string()],
            75,
            true,
        );

        assert_eq!(config.project_path, PathBuf::from("/test/project"));
        assert_eq!(config.include_patterns, vec!["*.rs".to_string()]);
        assert_eq!(config.exclude_patterns, vec!["test_*".to_string()]);
        assert_eq!(config.confidence_threshold, 75);
        assert!(config.analyze_space_complexity);
    }

    #[test]
    fn test_build_analysis_config_empty_patterns() {
        let config = build_analysis_config(PathBuf::from("."), vec![], vec![], 50, false);

        assert!(config.include_patterns.is_empty());
        assert!(config.exclude_patterns.is_empty());
        assert!(!config.analyze_space_complexity);
    }

    #[test]
    fn test_build_analysis_config_multiple_patterns() {
        let config = build_analysis_config(
            PathBuf::from("/project"),
            vec!["*.rs".to_string(), "*.py".to_string(), "*.js".to_string()],
            vec!["target/*".to_string(), "node_modules/*".to_string()],
            90,
            true,
        );

        assert_eq!(config.include_patterns.len(), 3);
        assert_eq!(config.exclude_patterns.len(), 2);
        assert_eq!(config.confidence_threshold, 90);
    }

    // ============================================
    // Tests for group_functions_by_file
    // ============================================

    #[test]
    fn test_group_functions_by_file_empty() {
        let functions: Vec<FunctionComplexity> = vec![];
        let grouped = group_functions_by_file(&functions);
        assert!(grouped.is_empty());
    }

    #[test]
    fn test_group_functions_by_file_single_file() {
        let functions = vec![
            create_test_function_complexity("fn1", "src/lib.rs", 10, BigOClass::Quadratic, 80),
            create_test_function_complexity("fn2", "src/lib.rs", 20, BigOClass::Cubic, 85),
        ];
        let grouped = group_functions_by_file(&functions);

        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped.get(&PathBuf::from("src/lib.rs")).unwrap().len(), 2);
    }

    #[test]
    fn test_group_functions_by_file_multiple_files() {
        let functions = vec![
            create_test_function_complexity("fn1", "src/lib.rs", 10, BigOClass::Quadratic, 80),
            create_test_function_complexity("fn2", "src/main.rs", 20, BigOClass::Cubic, 85),
            create_test_function_complexity("fn3", "src/lib.rs", 30, BigOClass::Exponential, 90),
        ];
        let grouped = group_functions_by_file(&functions);

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get(&PathBuf::from("src/lib.rs")).unwrap().len(), 2);
        assert_eq!(grouped.get(&PathBuf::from("src/main.rs")).unwrap().len(), 1);
    }

    // ============================================
    // Tests for calculate_file_complexity_scores
    // ============================================

    #[test]
    fn test_calculate_file_complexity_scores_empty() {
        let file_functions = std::collections::HashMap::new();
        let scores = calculate_file_complexity_scores(&file_functions);
        assert!(scores.is_empty());
    }

    #[test]
    fn test_calculate_file_complexity_scores_single_file() {
        let functions = vec![create_test_function_complexity(
            "fn1",
            "src/lib.rs",
            10,
            BigOClass::Quadratic,
            80,
        )];
        let grouped = group_functions_by_file(&functions);
        let scores = calculate_file_complexity_scores(&grouped);

        assert_eq!(scores.len(), 1);
        assert_eq!(scores[0].0, PathBuf::from("src/lib.rs"));
        assert!((scores[0].1 - 5.0).abs() < f64::EPSILON); // Quadratic = 5.0
    }

    #[test]
    fn test_calculate_file_complexity_scores_sorted_by_descending() {
        let functions = vec![
            create_test_function_complexity("fn1", "src/low.rs", 10, BigOClass::Linear, 80),
            create_test_function_complexity("fn2", "src/high.rs", 20, BigOClass::Exponential, 85),
            create_test_function_complexity("fn3", "src/mid.rs", 30, BigOClass::Quadratic, 90),
        ];
        let grouped = group_functions_by_file(&functions);
        let scores = calculate_file_complexity_scores(&grouped);

        assert_eq!(scores.len(), 3);
        // Should be sorted descending by score
        assert_eq!(scores[0].0, PathBuf::from("src/high.rs")); // Exponential = 7.0
        assert_eq!(scores[1].0, PathBuf::from("src/mid.rs")); // Quadratic = 5.0
        assert_eq!(scores[2].0, PathBuf::from("src/low.rs")); // Linear = 3.0
    }

    #[test]
    fn test_calculate_file_complexity_scores_aggregates_multiple_functions() {
        let functions = vec![
            create_test_function_complexity("fn1", "src/lib.rs", 10, BigOClass::Quadratic, 80),
            create_test_function_complexity("fn2", "src/lib.rs", 20, BigOClass::Cubic, 85),
        ];
        let grouped = group_functions_by_file(&functions);
        let scores = calculate_file_complexity_scores(&grouped);

        assert_eq!(scores.len(), 1);
        // Quadratic(5.0) + Cubic(6.0) = 11.0
        assert!((scores[0].1 - 11.0).abs() < f64::EPSILON);
    }

    // ============================================
    // Tests for get_top_file_paths
    // ============================================

    #[test]
    fn test_get_top_file_paths_empty() {
        let scores: Vec<(PathBuf, f64)> = vec![];
        let top = get_top_file_paths(scores, 5);
        assert!(top.is_empty());
    }

    #[test]
    fn test_get_top_file_paths_fewer_than_requested() {
        let scores = vec![(PathBuf::from("a.rs"), 10.0), (PathBuf::from("b.rs"), 5.0)];
        let top = get_top_file_paths(scores, 10);
        assert_eq!(top.len(), 2);
        assert!(top.contains(&PathBuf::from("a.rs")));
        assert!(top.contains(&PathBuf::from("b.rs")));
    }

    #[test]
    fn test_get_top_file_paths_exact_count() {
        let scores = vec![
            (PathBuf::from("a.rs"), 10.0),
            (PathBuf::from("b.rs"), 8.0),
            (PathBuf::from("c.rs"), 5.0),
        ];
        let top = get_top_file_paths(scores, 2);
        assert_eq!(top.len(), 2);
        assert!(top.contains(&PathBuf::from("a.rs")));
        assert!(top.contains(&PathBuf::from("b.rs")));
        assert!(!top.contains(&PathBuf::from("c.rs")));
    }

    #[test]
    fn test_get_top_file_paths_zero_requested() {
        let scores = vec![(PathBuf::from("a.rs"), 10.0)];
        let top = get_top_file_paths(scores, 0);
        assert!(top.is_empty());
    }

    // ============================================
    // Tests for apply_high_complexity_filter
    // ============================================

    #[test]
    fn test_apply_high_complexity_filter_removes_low_complexity() {
        let mut report = BigOAnalysisReport {
            analyzed_functions: 5,
            complexity_distribution: create_test_distribution(),
            high_complexity_functions: vec![
                create_test_function_complexity("fn1", "a.rs", 1, BigOClass::Linear, 80),
                create_test_function_complexity("fn2", "a.rs", 2, BigOClass::Quadratic, 80),
                create_test_function_complexity("fn3", "a.rs", 3, BigOClass::Constant, 80),
            ],
            pattern_matches: vec![],
            recommendations: vec![],
        };

        apply_high_complexity_filter(&mut report, false);

        assert_eq!(report.high_complexity_functions.len(), 1);
        assert_eq!(report.high_complexity_functions[0].function_name, "fn2");
    }

    #[test]
    fn test_apply_high_complexity_filter_keeps_all_high() {
        let mut report = BigOAnalysisReport {
            analyzed_functions: 4,
            complexity_distribution: create_test_distribution(),
            high_complexity_functions: vec![
                create_test_function_complexity("fn1", "a.rs", 1, BigOClass::Quadratic, 80),
                create_test_function_complexity("fn2", "a.rs", 2, BigOClass::Cubic, 80),
                create_test_function_complexity("fn3", "a.rs", 3, BigOClass::Exponential, 80),
                create_test_function_complexity("fn4", "a.rs", 4, BigOClass::Factorial, 80),
            ],
            pattern_matches: vec![],
            recommendations: vec![],
        };

        apply_high_complexity_filter(&mut report, false);

        assert_eq!(report.high_complexity_functions.len(), 4);
    }

    #[test]
    fn test_apply_high_complexity_filter_empty_input() {
        let mut report = create_test_report_empty();
        apply_high_complexity_filter(&mut report, false);
        assert!(report.high_complexity_functions.is_empty());
    }

    // ============================================
    // Tests for apply_top_files_filter
    // ============================================

    #[test]
    fn test_apply_top_files_filter_limits_files() {
        let mut report = BigOAnalysisReport {
            analyzed_functions: 6,
            complexity_distribution: create_test_distribution(),
            high_complexity_functions: vec![
                create_test_function_complexity("fn1", "high.rs", 1, BigOClass::Exponential, 80),
                create_test_function_complexity("fn2", "high.rs", 2, BigOClass::Cubic, 80),
                create_test_function_complexity("fn3", "mid.rs", 3, BigOClass::Quadratic, 80),
                create_test_function_complexity("fn4", "low.rs", 4, BigOClass::Quadratic, 80),
            ],
            pattern_matches: vec![],
            recommendations: vec![],
        };

        apply_top_files_filter(&mut report, 1);

        // Only high.rs should remain (highest total score: 7+6 = 13)
        assert_eq!(report.high_complexity_functions.len(), 2);
        assert!(report
            .high_complexity_functions
            .iter()
            .all(|f| f.file_path == Path::new("high.rs")));
    }

    #[test]
    fn test_apply_top_files_filter_empty_report() {
        let mut report = create_test_report_empty();
        apply_top_files_filter(&mut report, 5);
        assert!(report.high_complexity_functions.is_empty());
    }

    // ============================================
    // Tests for apply_report_filters
    // ============================================

    #[test]
    fn test_apply_report_filters_no_filters() {
        let mut report = create_test_report_with_functions();
        let original_count = report.high_complexity_functions.len();

        apply_report_filters(&mut report, false, 0, false);

        assert_eq!(report.high_complexity_functions.len(), original_count);
    }

    #[test]
    fn test_apply_report_filters_high_complexity_only() {
        let mut report = BigOAnalysisReport {
            analyzed_functions: 3,
            complexity_distribution: create_test_distribution(),
            high_complexity_functions: vec![
                create_test_function_complexity("fn1", "a.rs", 1, BigOClass::Linear, 80),
                create_test_function_complexity("fn2", "a.rs", 2, BigOClass::Quadratic, 80),
            ],
            pattern_matches: vec![],
            recommendations: vec![],
        };

        apply_report_filters(&mut report, true, 0, false);

        assert_eq!(report.high_complexity_functions.len(), 1);
        assert_eq!(report.high_complexity_functions[0].function_name, "fn2");
    }

    #[test]
    fn test_apply_report_filters_top_files_only() {
        let mut report = BigOAnalysisReport {
            analyzed_functions: 4,
            complexity_distribution: create_test_distribution(),
            high_complexity_functions: vec![
                create_test_function_complexity("fn1", "high.rs", 1, BigOClass::Exponential, 80),
                create_test_function_complexity("fn2", "low.rs", 2, BigOClass::Quadratic, 80),
            ],
            pattern_matches: vec![],
            recommendations: vec![],
        };

        apply_report_filters(&mut report, false, 1, false);

        assert_eq!(report.high_complexity_functions.len(), 1);
        assert_eq!(
            report.high_complexity_functions[0].file_path,
            PathBuf::from("high.rs")
        );
    }

    #[test]
    fn test_apply_report_filters_combined() {
        let mut report = BigOAnalysisReport {
            analyzed_functions: 5,
            complexity_distribution: create_test_distribution(),
            high_complexity_functions: vec![
                create_test_function_complexity("fn1", "high.rs", 1, BigOClass::Exponential, 80),
                create_test_function_complexity("fn2", "high.rs", 2, BigOClass::Linear, 80),
                create_test_function_complexity("fn3", "low.rs", 3, BigOClass::Quadratic, 80),
            ],
            pattern_matches: vec![],
            recommendations: vec![],
        };

        // First filter to high complexity, then top 1 file
        apply_report_filters(&mut report, true, 1, false);

        // Should keep only high complexity functions from the top file
        assert_eq!(report.high_complexity_functions.len(), 1);
        assert_eq!(report.high_complexity_functions[0].function_name, "fn1");
    }

    // ============================================
    // Tests for format_big_o_summary
    // ============================================

    #[test]
    fn test_format_big_o_summary_header() {
        let report = create_test_report_empty();
        let output = format_big_o_summary(&report);
        let plain = strip_ansi(&output);

        assert!(plain.contains("Big-O Complexity Analysis Summary"));
    }

    #[test]
    fn test_format_big_o_summary_total_functions() {
        let report = BigOAnalysisReport {
            analyzed_functions: 100,
            ..create_test_report_empty()
        };
        let output = format_big_o_summary(&report);
        let plain = strip_ansi(&output);

        assert!(plain.contains("Total Functions Analyzed"));
        assert!(plain.contains("100"));
    }

    #[test]
    fn test_format_big_o_summary_high_complexity_count() {
        let report = create_test_report_with_functions();
        let output = format_big_o_summary(&report);
        let plain = strip_ansi(&output);

        assert!(plain.contains("High Complexity Functions"));
        assert!(plain.contains("4"));
    }

    #[test]
    fn test_format_big_o_summary_distribution() {
        let report = BigOAnalysisReport {
            analyzed_functions: 50,
            complexity_distribution: create_test_distribution(),
            ..create_test_report_empty()
        };
        let output = format_big_o_summary(&report);

        assert!(output.contains("Complexity Distribution:"));
        assert!(output.contains("O(1)"));
        assert!(output.contains("O(log n)"));
        assert!(output.contains("O(n)"));
        assert!(output.contains("O(n log n)"));
        assert!(output.contains("O(n²)"));
        assert!(output.contains("O(n³)"));
        assert!(output.contains("O(2^n)"));
        assert!(output.contains("Unknown"));
    }

    #[test]
    fn test_format_big_o_summary_with_recommendations() {
        let report = create_test_report_with_functions();
        let output = format_big_o_summary(&report);

        assert!(output.contains("Recommendations:"));
        assert!(output.contains("Consider optimizing quadratic algorithms"));
    }

    #[test]
    fn test_format_big_o_summary_top_files() {
        let report = create_test_report_with_functions();
        let output = format_big_o_summary(&report);

        assert!(output.contains("Top Files by Complexity:"));
        // Should show file names
        assert!(
            output.contains("sort.rs") || output.contains("math.rs") || output.contains("algo.rs")
        );
    }

    #[test]
    fn test_format_big_o_summary_no_recommendations_when_empty() {
        let report = create_test_report_empty();
        let output = format_big_o_summary(&report);

        assert!(!output.contains("Recommendations:"));
    }

    #[test]
    fn test_format_big_o_summary_no_top_files_when_no_functions() {
        let report = create_test_report_empty();
        let output = format_big_o_summary(&report);

        assert!(!output.contains("Top Files by Complexity:"));
    }

    // ============================================
    // Tests for format_big_o_detailed
    // ============================================

    #[test]
    fn test_format_big_o_detailed_includes_summary() {
        let report = create_test_report_with_functions();
        let output = format_big_o_detailed(&report);
        let plain = strip_ansi(&output);

        // Should contain summary section
        assert!(plain.contains("Big-O Complexity Analysis Summary"));
        assert!(plain.contains("Total Functions Analyzed:"));
    }

    #[test]
    fn test_format_big_o_detailed_function_list() {
        let report = create_test_report_with_functions();
        let output = format_big_o_detailed(&report);
        let plain = strip_ansi(&output);

        assert!(plain.contains("High Complexity Functions:"));
        assert!(plain.contains("bubble_sort"));
        assert!(plain.contains("matrix_mult"));
    }

    #[test]
    fn test_format_big_o_detailed_function_location() {
        let report = create_test_report_with_functions();
        let output = format_big_o_detailed(&report);
        let plain = strip_ansi(&output);

        // Should show file path and line number
        assert!(plain.contains("src/sort.rs"));
        assert!(plain.contains("42"));
        assert!(plain.contains("src/math.rs"));
        assert!(plain.contains("100"));
    }

    #[test]
    fn test_format_big_o_detailed_complexity_info() {
        let report = create_test_report_with_functions();
        let output = format_big_o_detailed(&report);
        let plain = strip_ansi(&output);

        assert!(plain.contains("Time Complexity:"));
        assert!(plain.contains("Space Complexity:"));
    }

    #[test]
    fn test_format_big_o_detailed_with_notes() {
        let mut report = create_test_report_with_functions();
        report.high_complexity_functions[0].notes = vec![
            "Nested loop detected".to_string(),
            "Consider using hash map".to_string(),
        ];
        let output = format_big_o_detailed(&report);
        let plain = strip_ansi(&output);

        assert!(plain.contains("Notes:"));
        assert!(plain.contains("Nested loop detected"));
        assert!(plain.contains("Consider using hash map"));
    }

    #[test]
    fn test_format_big_o_detailed_pattern_matches() {
        let report = create_test_report_with_functions();
        let output = format_big_o_detailed(&report);
        let plain = strip_ansi(&output);

        assert!(plain.contains("Pattern Matches:"));
        assert!(plain.contains("Sorting operation"));
        assert!(plain.contains("5 occurrences"));
    }

    #[test]
    fn test_format_big_o_detailed_empty_pattern_matches() {
        let mut report = create_test_report_with_functions();
        report.pattern_matches = vec![];
        let output = format_big_o_detailed(&report);

        assert!(!output.contains("Pattern Matches:"));
    }

    // ============================================
    // Tests for format_analysis_output
    // ============================================

    #[test]
    fn test_format_analysis_output_json() {
        let analyzer = BigOAnalyzer::new();
        let report = create_test_report_with_functions();
        let result = format_analysis_output(&analyzer, &report, BigOOutputFormat::Json, false);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("\"analyzed_functions\""));
        assert!(output.contains("\"distribution\""));
    }

    #[test]
    fn test_format_analysis_output_markdown() {
        let analyzer = BigOAnalyzer::new();
        let report = create_test_report_with_functions();
        let result = format_analysis_output(&analyzer, &report, BigOOutputFormat::Markdown, false);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("# Big-O Complexity Analysis Report"));
        assert!(output.contains("## Summary"));
    }

    #[test]
    fn test_format_analysis_output_summary() {
        let analyzer = BigOAnalyzer::new();
        let report = create_test_report_with_functions();
        let result = format_analysis_output(&analyzer, &report, BigOOutputFormat::Summary, false);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Big-O Complexity Analysis Summary"));
    }

    #[test]
    fn test_format_analysis_output_detailed() {
        let analyzer = BigOAnalyzer::new();
        let report = create_test_report_with_functions();
        let result = format_analysis_output(&analyzer, &report, BigOOutputFormat::Detailed, false);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("High Complexity Functions:"));
    }

    // ============================================
    // Tests for write_analysis_output (async)
    // ============================================

    #[tokio::test]
    async fn test_write_analysis_output_stdout() {
        let content = "Test output content";
        let result = write_analysis_output(content, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_write_analysis_output_to_file() {
        use std::fs;

        let temp_dir = std::env::temp_dir();
        let output_file = temp_dir.join("big_o_test_output.txt");

        let content = "Test output to file";
        let result = write_analysis_output(content, Some(output_file.clone())).await;

        assert!(result.is_ok());

        // Verify file was written
        let file_content = fs::read_to_string(&output_file).unwrap();
        assert_eq!(file_content, content);

        // Clean up
        let _ = fs::remove_file(&output_file);
    }

    // ============================================
    // Tests for print_analysis_header (coverage)
    // ============================================

    #[test]
    fn test_print_analysis_header_does_not_panic() {
        use crate::cli::handlers::big_o_handlers::handlers::print_analysis_header;
        // This test ensures the function doesn't panic
        print_analysis_header(&PathBuf::from("/test/path"), 75);
    }

    // ============================================
    // Tests for print_analysis_summary (coverage)
    // ============================================

    #[test]
    fn test_print_analysis_summary_empty_report() {
        use crate::cli::handlers::big_o_handlers::handlers::print_analysis_summary;
        let report = create_test_report_empty();
        let elapsed = std::time::Duration::from_millis(100);
        // Should not panic
        print_analysis_summary(&report, elapsed, false);
    }

    #[test]
    fn test_print_analysis_summary_with_functions() {
        use crate::cli::handlers::big_o_handlers::handlers::print_analysis_summary;
        let report = create_test_report_with_functions();
        let elapsed = std::time::Duration::from_millis(500);
        // Should not panic
        print_analysis_summary(&report, elapsed, false);
    }

    #[test]
    fn test_print_analysis_summary_with_perf() {
        use crate::cli::handlers::big_o_handlers::handlers::print_analysis_summary;
        let report = create_test_report_with_functions();
        let elapsed = std::time::Duration::from_millis(1000);
        // Should not panic and should print performance metrics
        print_analysis_summary(&report, elapsed, true);
    }

    // ============================================
    // Edge case and integration tests
    // ============================================

    #[test]
    fn test_summary_formatting_preserves_order() {
        let report = BigOAnalysisReport {
            analyzed_functions: 100,
            complexity_distribution: ComplexityDistribution {
                constant: 50,
                logarithmic: 20,
                linear: 15,
                linearithmic: 5,
                quadratic: 5,
                cubic: 3,
                exponential: 1,
                unknown: 1,
            },
            high_complexity_functions: vec![],
            pattern_matches: vec![],
            recommendations: vec![],
        };

        let output = format_big_o_summary(&report);

        // Verify the order of complexity classes in output
        let o1_pos = output.find("O(1)").unwrap();
        let ologn_pos = output.find("O(log n)").unwrap();
        let on_pos = output.find("O(n)").unwrap();

        assert!(o1_pos < ologn_pos);
        assert!(ologn_pos < on_pos);
    }

    #[test]
    fn test_file_grouping_handles_duplicate_paths() {
        let functions = vec![
            create_test_function_complexity("fn1", "src/lib.rs", 10, BigOClass::Quadratic, 80),
            create_test_function_complexity("fn2", "src/lib.rs", 20, BigOClass::Quadratic, 85),
            create_test_function_complexity("fn3", "src/lib.rs", 30, BigOClass::Quadratic, 90),
        ];
        let grouped = group_functions_by_file(&functions);

        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped.get(&PathBuf::from("src/lib.rs")).unwrap().len(), 3);
    }

    #[test]
    fn test_complexity_score_accumulation() {
        let functions = vec![
            create_test_function_complexity("fn1", "file.rs", 10, BigOClass::Constant, 80), // 1.0
            create_test_function_complexity("fn2", "file.rs", 20, BigOClass::Logarithmic, 80), // 2.0
            create_test_function_complexity("fn3", "file.rs", 30, BigOClass::Linear, 80), // 3.0
        ];
        let grouped = group_functions_by_file(&functions);
        let scores = calculate_file_complexity_scores(&grouped);

        assert_eq!(scores.len(), 1);
        // 1.0 + 2.0 + 3.0 = 6.0
        assert!((scores[0].1 - 6.0).abs() < f64::EPSILON);
    }

    // ============================================
    // DETERMINISM (round-3 sweep)
    // ============================================

    /// `analyze big-o --format json` returned a DIFFERENT set of
    /// `high_complexity_functions` on every run over an unchanged tree: 6 runs
    /// gave 6 distinct membership sets, only 16 of 24 slots stable, union 42
    /// functions. Cause: `--top-files` ranks files by a score on which nearly
    /// every file TIES (one O(n^2) function is 5.0 everywhere), the ranking is
    /// built from a `HashMap`, and `sort_by` is stable — so `.take(n)` kept
    /// whichever tied files that process's hash seed visited first.
    ///
    /// Each iteration builds a FRESH `HashMap`, which gets its own
    /// `RandomState` and therefore its own iteration order: the in-process
    /// stand-in for "run the binary again".
    #[test]
    fn top_files_selection_is_stable_across_fresh_hash_maps() {
        fn top_two() -> Vec<PathBuf> {
            // Eight files, all tied at exactly one Quadratic function (5.0).
            let functions: Vec<FunctionComplexity> = (0..8)
                .map(|i| {
                    create_test_function_complexity(
                        "f",
                        &format!("src/tied_{i}.rs"),
                        1,
                        BigOClass::Quadratic,
                        90,
                    )
                })
                .collect();
            let grouped = group_functions_by_file(&functions);
            let scores = calculate_file_complexity_scores(&grouped);
            let mut kept: Vec<PathBuf> = get_top_file_paths(scores, 2).into_iter().collect();
            kept.sort();
            kept
        }

        let first = top_two();
        assert_eq!(first.len(), 2);
        for i in 0..10 {
            assert_eq!(
                top_two(),
                first,
                "iteration {i}: tied files must be ranked by path, not by hash seed"
            );
        }
        // Ties resolve to the lexicographically smallest paths, so the answer
        // is a property of the tree rather than of the process.
        assert_eq!(
            first,
            vec![
                PathBuf::from("src/tied_0.rs"),
                PathBuf::from("src/tied_1.rs")
            ]
        );
    }

    /// A COUNT MUST AGREE WITH THE LIST IT HEADS, and a total that is secretly
    /// a cap must say so. `summary.high_complexity_count` was the length of the
    /// TRUNCATED list while the distribution beside it counted the whole
    /// project: 24 vs 106 on pmat's own tree, with no truncation flag.
    #[test]
    fn json_summary_names_both_the_listed_and_the_found_high_complexity_counts() {
        let analyzer = BigOAnalyzer::new();
        let report = BigOAnalysisReport {
            analyzed_functions: 100,
            complexity_distribution: ComplexityDistribution {
                constant: 40,
                logarithmic: 0,
                linear: 30,
                linearithmic: 0,
                // 20 + 5 + 5 = 30 functions qualified as high complexity...
                quadratic: 20,
                cubic: 5,
                exponential: 5,
                unknown: 0,
            },
            // ...but only two survived `--top-files`.
            high_complexity_functions: vec![
                create_test_function_complexity("a", "src/a.rs", 1, BigOClass::Quadratic, 90),
                create_test_function_complexity("b", "src/b.rs", 2, BigOClass::Cubic, 90),
            ],
            pattern_matches: vec![],
            recommendations: vec![],
        };

        let json: serde_json::Value =
            serde_json::from_str(&analyzer.format_as_json(&report).unwrap()).unwrap();
        let summary = &json["summary"];
        let listed = json["high_complexity_functions"].as_array().unwrap().len();

        assert_eq!(
            summary["high_complexity_count"].as_u64().unwrap() as usize,
            listed,
            "the count must equal the length of the list it heads"
        );
        assert_eq!(summary["high_complexity_found"].as_u64().unwrap(), 30);
        assert_eq!(
            summary["high_complexity_truncated"],
            serde_json::json!(true)
        );

        let markdown = analyzer.format_as_markdown(&report);
        assert!(
            markdown.contains("2 listed of 30 found"),
            "markdown must disclose the truncation too: {markdown}"
        );
    }

    /// No part may exceed its whole: an untruncated report reports the same
    /// number twice and does not claim to be truncated.
    #[test]
    fn json_summary_does_not_claim_truncation_when_nothing_was_cut() {
        let analyzer = BigOAnalyzer::new();
        let report = BigOAnalysisReport {
            analyzed_functions: 10,
            complexity_distribution: ComplexityDistribution {
                constant: 8,
                logarithmic: 0,
                linear: 0,
                linearithmic: 0,
                quadratic: 2,
                cubic: 0,
                exponential: 0,
                unknown: 0,
            },
            high_complexity_functions: vec![
                create_test_function_complexity("a", "src/a.rs", 1, BigOClass::Quadratic, 90),
                create_test_function_complexity("b", "src/b.rs", 2, BigOClass::Quadratic, 90),
            ],
            pattern_matches: vec![],
            recommendations: vec![],
        };

        let json: serde_json::Value =
            serde_json::from_str(&analyzer.format_as_json(&report).unwrap()).unwrap();
        assert_eq!(
            json["summary"]["high_complexity_count"].as_u64().unwrap(),
            2
        );
        assert_eq!(
            json["summary"]["high_complexity_found"].as_u64().unwrap(),
            2
        );
        assert_eq!(
            json["summary"]["high_complexity_truncated"],
            serde_json::json!(false)
        );
    }

    // ============================================
    // --high-complexity-only must change the report
    // ============================================

    /// The flag was applied by `retain`ing `high_complexity_functions` on the
    /// SAME predicate `build_report` had already used to build that list, so it
    /// could not drop an element for any input: `analyze big-o` and
    /// `analyze big-o --high-complexity-only` were byte-identical in summary,
    /// detailed, markdown AND json on a corpus with 13 qualifying functions.
    #[test]
    fn high_complexity_only_narrows_every_format() {
        let analyzer = BigOAnalyzer::new();
        let report = create_test_report_with_functions();

        for format in [
            BigOOutputFormat::Summary,
            BigOOutputFormat::Detailed,
            BigOOutputFormat::Markdown,
            BigOOutputFormat::Json,
        ] {
            let plain = format_analysis_output(&analyzer, &report, format.clone(), false).unwrap();
            let scoped = format_analysis_output(&analyzer, &report, format.clone(), true).unwrap();
            assert_ne!(
                plain, scoped,
                "--high-complexity-only produced identical {format:?} output"
            );
        }
    }

    /// What it narrows, precisely: the distribution keeps the O(n^2)-or-worse
    /// rows and drops the rest. `analyzed_functions` is NOT narrowed — 50
    /// functions were analysed either way, and saying otherwise would be a
    /// fabricated total.
    #[test]
    fn high_complexity_only_keeps_the_high_rows_and_the_analysed_total() {
        let analyzer = BigOAnalyzer::new();
        let report = create_test_report_with_functions();

        let json: serde_json::Value =
            serde_json::from_str(&analyzer.format_as_json_scoped(&report, true).unwrap()).unwrap();
        let dist = json["distribution"].as_object().expect("distribution");

        for kept in ["O(n^2)", "O(n^3)", "O(2^n)"] {
            assert!(dist.contains_key(kept), "{kept} must survive the filter");
        }
        for dropped in ["O(1)", "O(log n)", "O(n)", "O(n log n)", "O(?)"] {
            assert!(
                !dist.contains_key(dropped),
                "{dropped} is not high complexity and must be filtered out"
            );
        }
        assert_eq!(dist["O(n^2)"], serde_json::json!(7));
        assert_eq!(
            json["summary"]["analyzed_functions"],
            serde_json::json!(50),
            "the analysed total is not scoped by a display filter"
        );
        assert_eq!(
            json["summary"]["high_complexity_only"],
            serde_json::json!(true)
        );
    }

    /// Without the flag every row is still present, byte for byte — the fix
    /// must not change what a default run prints.
    #[test]
    fn default_run_still_carries_every_distribution_row() {
        let analyzer = BigOAnalyzer::new();
        let report = create_test_report_with_functions();
        let json: serde_json::Value =
            serde_json::from_str(&analyzer.format_as_json(&report).unwrap()).unwrap();
        let dist = json["distribution"].as_object().expect("distribution");
        assert_eq!(dist.len(), 8, "default distribution has all eight classes");
        assert_eq!(dist["O(?)"], serde_json::json!(2));
    }
}
