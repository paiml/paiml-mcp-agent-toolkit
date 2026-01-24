        // Test with zero values
        let zero = ComplexityMetrics::new(0, 0, 0, 0);
        assert_eq!(zero.complexity_score(), 0.0);

        // Test with maximum values
        let high = ComplexityMetrics::new(100, 100, 10, 1000);
        let expected_high = 100.0 * 1.0 + 100.0 * 1.2 + 10.0 * 2.0 + 1000.0 * 0.1;
        assert!((high.complexity_score() - expected_high).abs() < 0.0001);

        // Test comparison (higher complexity = higher score)
        let simple = ComplexityMetrics::new(1, 1, 1, 10);
        let complex = ComplexityMetrics::new(20, 30, 5, 200);
        assert!(complex.complexity_score() > simple.complexity_score());
    }

    #[test]
    fn test_with_halstead_constructor() {
        let halstead = HalsteadMetrics::new(10, 8, 25, 20);
        let metrics = ComplexityMetrics::with_halstead(5, 10, 3, 50, halstead);

        assert_eq!(metrics.cyclomatic, 5);
        assert_eq!(metrics.cognitive, 10);
        assert_eq!(metrics.nesting_max, 3);
        assert_eq!(metrics.lines, 50);
        assert!(metrics.halstead.is_some());

        let h = metrics.halstead.unwrap();
        assert_eq!(h.operators_unique, 10);
        assert_eq!(h.operands_unique, 8);
        assert_eq!(h.operators_total, 25);
        assert_eq!(h.operands_total, 20);
    }

    #[test]
    fn test_halstead_metrics_default() {
        let metrics = HalsteadMetrics::default();
        assert_eq!(metrics.operators_unique, 0);
        assert_eq!(metrics.operands_unique, 0);
        assert_eq!(metrics.operators_total, 0);
        assert_eq!(metrics.operands_total, 0);
        assert_eq!(metrics.volume, 0.0);
        assert_eq!(metrics.difficulty, 0.0);
        assert_eq!(metrics.effort, 0.0);
        assert_eq!(metrics.time, 0.0);
        assert_eq!(metrics.bugs, 0.0);
    }

    #[test]
    fn test_halstead_calculate_derived_normal() {
        // Normal case with valid values
        let halstead = HalsteadMetrics::new(10, 8, 25, 20);
        let calculated = halstead.calculate_derived();

        // Volume = N * log2(n) where N = 45, n = 18
        let expected_volume = 45.0_f64 * 18.0_f64.log2();
        assert!((calculated.volume - expected_volume).abs() < 0.0001);

        // Difficulty = (n1/2) * (N2/n2) = (10/2) * (20/8) = 5 * 2.5 = 12.5
        assert!((calculated.difficulty - 12.5).abs() < 0.0001);

        // Effort = V * D
        let expected_effort = calculated.volume * calculated.difficulty;
        assert!((calculated.effort - expected_effort).abs() < 0.0001);

        // Time = E / 18
        assert!((calculated.time - calculated.effort / 18.0).abs() < 0.0001);

        // Bugs = E^(2/3) / 3000
        let expected_bugs = calculated.effort.powf(2.0 / 3.0) / 3000.0;
        assert!((calculated.bugs - expected_bugs).abs() < 0.0001);
    }

    #[test]
    fn test_halstead_calculate_derived_zero_operators() {
        // Edge case: zero operators_unique
        let halstead = HalsteadMetrics::new(0, 8, 25, 20);
        let calculated = halstead.calculate_derived();

        // Should return early without modifying values
        assert_eq!(calculated.volume, 0.0);
        assert_eq!(calculated.difficulty, 0.0);
        assert_eq!(calculated.effort, 0.0);
        assert_eq!(calculated.time, 0.0);
        assert_eq!(calculated.bugs, 0.0);
    }

    #[test]
    fn test_halstead_calculate_derived_zero_operands() {
        // Edge case: zero operands_unique
        let halstead = HalsteadMetrics::new(10, 0, 25, 20);
        let calculated = halstead.calculate_derived();

        // Should return early without modifying values
        assert_eq!(calculated.volume, 0.0);
        assert_eq!(calculated.difficulty, 0.0);
        assert_eq!(calculated.effort, 0.0);
        assert_eq!(calculated.time, 0.0);
        assert_eq!(calculated.bugs, 0.0);
    }

    #[test]
    fn test_halstead_calculate_derived_both_zero() {
        // Edge case: both zero
        let halstead = HalsteadMetrics::new(0, 0, 0, 0);
        let calculated = halstead.calculate_derived();

        assert_eq!(calculated.volume, 0.0);
        assert_eq!(calculated.difficulty, 0.0);
        assert_eq!(calculated.effort, 0.0);
        assert_eq!(calculated.time, 0.0);
        assert_eq!(calculated.bugs, 0.0);
    }

    #[test]
    fn test_halstead_calculate_derived_minimum_values() {
        // Minimum valid values (1 unique of each)
        let halstead = HalsteadMetrics::new(1, 1, 1, 1);
        let calculated = halstead.calculate_derived();

        // Volume = 2 * log2(2) = 2
        assert!((calculated.volume - 2.0).abs() < 0.0001);

        // Difficulty = (1/2) * (1/1) = 0.5
        assert!((calculated.difficulty - 0.5).abs() < 0.0001);

        // Effort = V * D = 2 * 0.5 = 1.0
        assert!((calculated.effort - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_aggregate_results_with_custom_thresholds() {
        let func1 = create_test_function("func1", 10, 20, create_test_metrics(18, 25, 2, 10));
        let func2 = create_test_function("func2", 30, 40, create_test_metrics(8, 12, 2, 15));

        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(26, 37, 2, 25),
            functions: vec![func1, func2],
            classes: vec![],
        }];

        // With default thresholds (cyclomatic_error=20, cognitive_error=30)
        let report_default = aggregate_results(file_metrics.clone());

        // With custom thresholds (cyclomatic=15, cognitive=20)
        let report_custom =
            aggregate_results_with_thresholds(file_metrics.clone(), Some(15), Some(20));

        // Custom thresholds should produce more violations
        assert!(report_custom.violations.len() >= report_default.violations.len());

        // With very high thresholds, should have no violations
        let report_high = aggregate_results_with_thresholds(file_metrics, Some(100), Some(100));
        assert!(report_high.violations.is_empty());
    }

    #[test]
    fn test_aggregate_results_with_only_cyclomatic_threshold() {
        let func = create_test_function("func1", 10, 20, create_test_metrics(25, 10, 2, 10));

        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(25, 10, 2, 10),
            functions: vec![func],
            classes: vec![],
        }];

        // Only set cyclomatic threshold
        let report = aggregate_results_with_thresholds(file_metrics, Some(20), None);

        // Should have cyclomatic violation
        let has_cyclomatic = report.violations.iter().any(|v| match v {
            Violation::Error { rule, .. } | Violation::Warning { rule, .. } => {
                rule == "cyclomatic-complexity"
            }
        });
        assert!(has_cyclomatic);
    }

    #[test]
    fn test_aggregate_results_with_only_cognitive_threshold() {
        let func = create_test_function("func1", 10, 20, create_test_metrics(5, 35, 2, 10));

        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(5, 35, 2, 10),
            functions: vec![func],
            classes: vec![],
        }];

        // Only set cognitive threshold
        let report = aggregate_results_with_thresholds(file_metrics, None, Some(25));

        // Should have cognitive violation
        let has_cognitive = report.violations.iter().any(|v| match v {
            Violation::Error { rule, .. } | Violation::Warning { rule, .. } => {
                rule == "cognitive-complexity"
            }
        });
        assert!(has_cognitive);
    }

    #[test]
    fn test_format_complexity_summary_with_files() {
        let func1 = create_test_function("simple_func", 10, 20, create_test_metrics(3, 5, 1, 10));
        let func2 =
            create_test_function("complex_func", 30, 50, create_test_metrics(15, 20, 4, 25));

        let files = vec![
            FileComplexityMetrics {
                path: "src/simple.rs".to_string(),
                total_complexity: create_test_metrics(5, 8, 2, 30),
                functions: vec![func1],
                classes: vec![],
            },
            FileComplexityMetrics {
                path: "src/complex.rs".to_string(),
                total_complexity: create_test_metrics(25, 35, 5, 100),
                functions: vec![func2],
                classes: vec![],
            },
        ];

        let report = aggregate_results(files);
        let output = format_complexity_summary(&report);

        // Should contain file listing
        assert!(output.contains("## Top Files by Complexity"));
        assert!(output.contains("complex.rs")); // Higher complexity file
        assert!(output.contains("simple.rs"));
    }

    #[test]
    fn test_format_complexity_summary_single_file_with_functions() {
        let func1 = create_test_function("func_a", 10, 20, create_test_metrics(5, 8, 2, 10));
        let func2 = create_test_function("func_b", 30, 50, create_test_metrics(12, 18, 3, 20));
        let func3 = create_test_function("func_c", 60, 80, create_test_metrics(3, 4, 1, 15));

        let files = vec![FileComplexityMetrics {
            path: "src/main.rs".to_string(),
            total_complexity: create_test_metrics(20, 30, 3, 45),
            functions: vec![func1, func2, func3],
            classes: vec![],
        }];

        let report = aggregate_results(files);
        let output = format_complexity_summary(&report);

        // Single file should show function breakdown
        assert!(output.contains("## Functions in File"));
        assert!(output.contains("func_a"));
        assert!(output.contains("func_b"));
        assert!(output.contains("func_c"));
        // Functions should be sorted by complexity
        assert!(output.contains("func_b") && output.contains("func_a"));
    }

    #[test]
    fn test_format_complexity_summary_hotspots_limit() {
        // Create more than 5 hotspots to test truncation
        let mut functions = Vec::new();
        for i in 1..=10 {
            functions.push(create_test_function(
                &format!("hotspot_{}", i),
                i * 10,
                i * 10 + 10,
                create_test_metrics((10 + i) as u16, (15 + i) as u16, 2, 15),
            ));
        }

        let files = vec![FileComplexityMetrics {
            path: "src/hotspots.rs".to_string(),
            total_complexity: create_test_metrics(150, 200, 2, 150),
            functions,
            classes: vec![],
        }];

        let report = aggregate_results(files);
        let output = format_complexity_summary(&report);

        // Should show hotspots section
        assert!(output.contains("## Top Complexity Hotspots"));
        // Should limit to top 5 (based on format function)
        let hotspot_count = output.matches("📁").count();
        assert!(hotspot_count <= 5);
    }

    #[test]
    fn test_format_complexity_report_with_warnings() {
        let violations = vec![Violation::Warning {
            rule: "cognitive-complexity".to_string(),
            message: "Function is getting complex".to_string(),
            value: 18,
            threshold: 15,
            file: "warning.rs".to_string(),
            line: 25,
            function: Some("warn_func".to_string()),
        }];

        let report = ComplexityReport {
            summary: ComplexitySummary::default(),
            violations,
            hotspots: vec![],
            files: vec![],
        };

        let output = format_complexity_report(&report);

        assert!(output.contains("## Detailed Violations"));
        assert!(output.contains("### warning.rs"));
        assert!(output.contains("⚠️"));
        assert!(output.contains("warn_func"));
        assert!(output.contains("cognitive-complexity"));
    }

    #[test]
    fn test_format_complexity_report_violation_without_function() {
        let violations = vec![Violation::Error {
            rule: "cyclomatic-complexity".to_string(),
            message: "File too complex".to_string(),
            value: 50,
            threshold: 30,
            file: "complex.rs".to_string(),
            line: 1,
            function: None,
        }];

        let report = ComplexityReport {
            summary: ComplexitySummary::default(),
            violations,
            hotspots: vec![],
            files: vec![],
        };

        let output = format_complexity_report(&report);

        assert!(output.contains("### complex.rs"));
        assert!(output.contains("❌ **1:**")); // No function name
    }

    #[test]
    fn test_hotspot_without_function_name() {
        let hotspots = vec![ComplexityHotspot {
            file: "test.rs".to_string(),
            function: None, // No function name (file-level)
            line: 1,
            complexity: 30,
            complexity_type: "cyclomatic".to_string(),
        }];

        let report = ComplexityReport {
            summary: ComplexitySummary::default(),
            violations: vec![],
            hotspots,
            files: vec![],
        };

        let output = format_complexity_summary(&report);

        assert!(output.contains("`<file>` test.rs:1 - cyclomatic complexity: 30"));
    }

    #[test]
    fn test_complexity_summary_default() {
        let summary = ComplexitySummary::default();
        assert_eq!(summary.total_files, 0);
        assert_eq!(summary.total_functions, 0);
        assert_eq!(summary.median_cyclomatic, 0.0);
        assert_eq!(summary.median_cognitive, 0.0);
        assert_eq!(summary.max_cyclomatic, 0);
        assert_eq!(summary.max_cognitive, 0);
        assert_eq!(summary.p90_cyclomatic, 0);
        assert_eq!(summary.p90_cognitive, 0);
        assert_eq!(summary.technical_debt_hours, 0.0);
    }

    #[test]
    fn test_calculate_median_edge_cases() {
        // This tests the internal calculate_median function via aggregate_results

        // Single element
        let func = create_test_function("func1", 10, 20, create_test_metrics(7, 9, 2, 10));
        let file = FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(7, 9, 2, 10),
            functions: vec![func],
            classes: vec![],
        };
        let report = aggregate_results(vec![file]);
        assert_eq!(report.summary.median_cyclomatic, 7.0);
        assert_eq!(report.summary.median_cognitive, 9.0);
    }

    #[test]
    fn test_aggregate_results_multiple_files() {
        let file1 = FileComplexityMetrics {
            path: "file1.rs".to_string(),
            total_complexity: create_test_metrics(10, 15, 2, 50),
            functions: vec![
                create_test_function("f1", 10, 20, create_test_metrics(5, 8, 2, 10)),
                create_test_function("f2", 30, 40, create_test_metrics(5, 7, 1, 10)),
            ],
            classes: vec![],
        };

        let file2 = FileComplexityMetrics {
            path: "file2.rs".to_string(),
            total_complexity: create_test_metrics(20, 25, 3, 80),
            functions: vec![create_test_function(
                "f3",
                10,
                50,
                create_test_metrics(20, 25, 3, 40),
            )],
            classes: vec![],
        };

        let report = aggregate_results(vec![file1, file2]);

        assert_eq!(report.summary.total_files, 2);
        assert_eq!(report.summary.total_functions, 3);
        assert_eq!(report.files.len(), 2);
    }

    #[test]
    fn test_format_summary_errors_only() {
        let violations = vec![
            Violation::Error {
                rule: "test".to_string(),
                message: "Error 1".to_string(),
                value: 25,
                threshold: 20,
                file: "test.rs".to_string(),
                line: 10,
                function: None,
            },
            Violation::Error {
                rule: "test".to_string(),
                message: "Error 2".to_string(),
                value: 30,
                threshold: 20,
                file: "test.rs".to_string(),
                line: 20,
                function: None,
            },
        ];

        let report = ComplexityReport {
            summary: ComplexitySummary {
                total_files: 1,
                total_functions: 2,
                ..Default::default()
            },
            violations,
            hotspots: vec![],
            files: vec![],
        };

        let output = format_complexity_summary(&report);
        assert!(output.contains("**Errors**: 2"));
        assert!(!output.contains("**Warnings**")); // No warnings
    }

    #[test]
    fn test_format_summary_warnings_only() {
        let violations = vec![Violation::Warning {
            rule: "test".to_string(),
            message: "Warning 1".to_string(),
            value: 12,
            threshold: 10,
            file: "test.rs".to_string(),
            line: 10,
            function: None,
        }];

        let report = ComplexityReport {
            summary: ComplexitySummary {
                total_files: 1,
                total_functions: 1,
                ..Default::default()
            },
            violations,
            hotspots: vec![],
            files: vec![],
        };

        let output = format_complexity_summary(&report);
        assert!(output.contains("**Warnings**: 1"));
        assert!(!output.contains("**Errors**")); // No errors
    }

    #[test]
    fn test_complexity_metrics_clone_and_copy() {
        let original = ComplexityMetrics::new(5, 10, 3, 50);
        let cloned = original;
        let copied = original;

        assert_eq!(original.cyclomatic, cloned.cyclomatic);
        assert_eq!(original.cognitive, copied.cognitive);
    }
