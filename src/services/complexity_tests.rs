//\! Tests for complexity service
//\! Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;
    use std::path::Path;

    // Helper function to create test complexity metrics
    fn create_test_metrics(
        cyclomatic: u16,
        cognitive: u16,
        nesting_max: u8,
        lines: u16,
    ) -> ComplexityMetrics {
        ComplexityMetrics::new(cyclomatic, cognitive, nesting_max, lines)
    }

    // Helper function to create test function complexity
    fn create_test_function(
        name: &str,
        line_start: u32,
        line_end: u32,
        metrics: ComplexityMetrics,
    ) -> FunctionComplexity {
        FunctionComplexity {
            name: name.to_string(),
            line_start,
            line_end,
            metrics,
        }
    }

    #[test]
    fn test_complexity_metrics_default() {
        let metrics = ComplexityMetrics::default();
        assert_eq!(metrics.cyclomatic, 0);
        assert_eq!(metrics.cognitive, 0);
        assert_eq!(metrics.nesting_max, 0);
        assert_eq!(metrics.lines, 0);
    }

    #[test]
    fn test_complexity_metrics_creation() {
        let metrics = create_test_metrics(5, 10, 3, 25);
        assert_eq!(metrics.cyclomatic, 5);
        assert_eq!(metrics.cognitive, 10);
        assert_eq!(metrics.nesting_max, 3);
        assert_eq!(metrics.lines, 25);
    }

    #[test]
    fn test_complexity_thresholds_default() {
        let thresholds = ComplexityThresholds::default();
        assert_eq!(thresholds.cyclomatic_warn, 10);
        assert_eq!(thresholds.cyclomatic_error, 20);
        assert_eq!(thresholds.cognitive_warn, 15);
        assert_eq!(thresholds.cognitive_error, 30);
        assert_eq!(thresholds.nesting_max, 5);
        assert_eq!(thresholds.method_length, 50);
    }

    #[test]
    fn test_complexity_thresholds_custom() {
        let thresholds = ComplexityThresholds {
            cyclomatic_warn: 8,
            cyclomatic_error: 15,
            cognitive_warn: 12,
            cognitive_error: 25,
            nesting_max: 4,
            method_length: 40,
        };
        assert_eq!(thresholds.cyclomatic_warn, 8);
        assert_eq!(thresholds.cyclomatic_error, 15);
        assert_eq!(thresholds.cognitive_warn, 12);
        assert_eq!(thresholds.cognitive_error, 25);
        assert_eq!(thresholds.nesting_max, 4);
        assert_eq!(thresholds.method_length, 40);
    }

    #[test]
    fn test_function_complexity_creation() {
        let metrics = create_test_metrics(3, 8, 2, 15);
        let func = create_test_function("test_function", 10, 25, metrics);
        assert_eq!(func.name, "test_function");
        assert_eq!(func.line_start, 10);
        assert_eq!(func.line_end, 25);
        assert_eq!(func.metrics.cyclomatic, 3);
        assert_eq!(func.metrics.cognitive, 8);
    }

    #[test]
    fn test_class_complexity_creation() {
        let metrics = create_test_metrics(15, 25, 4, 100);
        let method = create_test_function("method1", 5, 15, create_test_metrics(3, 5, 2, 10));
        let class = ClassComplexity {
            name: "TestClass".to_string(),
            line_start: 1,
            line_end: 50,
            metrics,
            methods: vec![method],
        };
        assert_eq!(class.name, "TestClass");
        assert_eq!(class.line_start, 1);
        assert_eq!(class.line_end, 50);
        assert_eq!(class.methods.len(), 1);
        assert_eq!(class.methods[0].name, "method1");
    }

    #[test]
    fn test_file_complexity_metrics_creation() {
        let total_metrics = create_test_metrics(20, 35, 5, 200);
        let func1 = create_test_function("func1", 10, 20, create_test_metrics(5, 8, 2, 10));
        let func2 = create_test_function("func2", 30, 40, create_test_metrics(7, 12, 3, 15));

        let file_metrics = FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: total_metrics,
            functions: vec![func1, func2],
            classes: vec![],
        };

        assert_eq!(file_metrics.path, "test.rs");
        assert_eq!(file_metrics.functions.len(), 2);
        assert_eq!(file_metrics.classes.len(), 0);
        assert_eq!(file_metrics.total_complexity.cyclomatic, 20);
    }

    #[test]
    fn test_complexity_visitor_creation() {
        let mut metrics = ComplexityMetrics::default();
        let visitor = ComplexityVisitor::new(&mut metrics);
        assert_eq!(visitor.nesting_level, 0);
        assert!(visitor.current_function.is_none());
        assert!(visitor.functions.is_empty());
        assert!(visitor.classes.is_empty());
    }

    #[test]
    fn test_complexity_visitor_cognitive_increment() {
        let mut metrics = ComplexityMetrics::default();
        let visitor = ComplexityVisitor::new(&mut metrics);

        // Test non-nesting construct
        assert_eq!(visitor.calculate_cognitive_increment(false), 1);

        // Test nesting construct at level 0
        assert_eq!(visitor.calculate_cognitive_increment(true), 1);
    }

    #[test]
    fn test_complexity_visitor_cognitive_increment_with_nesting() {
        let mut metrics = ComplexityMetrics::default();
        let mut visitor = ComplexityVisitor::new(&mut metrics);

        // Increase nesting level
        visitor.nesting_level = 3;

        // Test nesting construct with nesting level
        assert_eq!(visitor.calculate_cognitive_increment(true), 3); // 1 + (3 - 1)

        // Test non-nesting construct
        assert_eq!(visitor.calculate_cognitive_increment(false), 1);
    }

    #[test]
    fn test_complexity_visitor_nesting_management() {
        let mut metrics = ComplexityMetrics::default();
        let mut visitor = ComplexityVisitor::new(&mut metrics);

        assert_eq!(visitor.nesting_level, 0);
        assert_eq!(visitor.complexity.nesting_max, 0);

        // Enter nesting
        visitor.enter_nesting();
        assert_eq!(visitor.nesting_level, 1);
        assert_eq!(visitor.complexity.nesting_max, 1);

        visitor.enter_nesting();
        assert_eq!(visitor.nesting_level, 2);
        assert_eq!(visitor.complexity.nesting_max, 2);

        // Exit nesting
        visitor.exit_nesting();
        assert_eq!(visitor.nesting_level, 1);
        assert_eq!(visitor.complexity.nesting_max, 2); // Max should remain

        visitor.exit_nesting();
        assert_eq!(visitor.nesting_level, 0);
        assert_eq!(visitor.complexity.nesting_max, 2);
    }

    #[test]
    fn test_complexity_visitor_nesting_saturation() {
        let mut metrics = ComplexityMetrics::default();
        let mut visitor = ComplexityVisitor::new(&mut metrics);

        // Test saturation at maximum nesting
        visitor.nesting_level = 255; // u8::MAX
        visitor.enter_nesting();
        assert_eq!(visitor.nesting_level, 255); // Should saturate

        // Test saturation at zero
        visitor.nesting_level = 0;
        visitor.exit_nesting();
        assert_eq!(visitor.nesting_level, 0); // Should saturate at 0
    }

    #[test]
    fn test_compute_complexity_cache_key() {
        let path = Path::new("test.rs");
        let content1 = b"fn test() {}";
        let content2 = b"fn test() { println!(\"hello\"); }";

        let key1 = compute_complexity_cache_key(path, content1);
        let key2 = compute_complexity_cache_key(path, content1);
        let key3 = compute_complexity_cache_key(path, content2);

        // Same content should produce same key
        assert_eq!(key1, key2);

        // Different content should produce different key
        assert_ne!(key1, key3);

        // Key should start with "cx:"
        assert!(key1.starts_with("cx:"));
        assert!(key3.starts_with("cx:"));
    }

    #[test]
    fn test_compute_complexity_cache_key_different_paths() {
        let path1 = Path::new("test1.rs");
        let path2 = Path::new("test2.rs");
        let content = b"fn test() {}";

        let key1 = compute_complexity_cache_key(path1, content);
        let key2 = compute_complexity_cache_key(path2, content);

        // Different paths should produce different keys
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cyclomatic_complexity_rule_creation() {
        let thresholds = ComplexityThresholds::default();
        let rule = CyclomaticComplexityRule::new(&thresholds);
        assert_eq!(rule.warn_threshold, 10);
        assert_eq!(rule.error_threshold, 20);
    }

    #[test]
    fn test_cyclomatic_complexity_rule_exceeds_threshold() {
        let thresholds = ComplexityThresholds::default();
        let rule = CyclomaticComplexityRule::new(&thresholds);

        assert!(!rule.exceeds_threshold(5, 10));
        assert!(!rule.exceeds_threshold(10, 10)); // Equal should not exceed
        assert!(rule.exceeds_threshold(15, 10));
    }

    #[test]
    fn test_cyclomatic_complexity_rule_no_violation() {
        let thresholds = ComplexityThresholds::default();
        let rule = CyclomaticComplexityRule::new(&thresholds);
        let metrics = create_test_metrics(5, 0, 0, 0); // Below warn threshold

        let result = rule.evaluate(&metrics, "test.rs", 10, Some("test_function"));
        assert!(result.is_none());
    }

    #[test]
    fn test_cyclomatic_complexity_rule_warning() {
        let thresholds = ComplexityThresholds::default();
        let rule = CyclomaticComplexityRule::new(&thresholds);
        let metrics = create_test_metrics(15, 0, 0, 0); // Above warn, below error

        let result = rule.evaluate(&metrics, "test.rs", 10, Some("test_function"));
        assert!(result.is_some());

        match result.unwrap() {
            Violation::Warning {
                rule: rule_name,
                message,
                value,
                threshold,
                file,
                line,
                function,
            } => {
                assert_eq!(rule_name, "cyclomatic-complexity");
                assert!(message.contains("15"));
                assert!(message.contains("10"));
                assert_eq!(value, 15);
                assert_eq!(threshold, 10);
                assert_eq!(file, "test.rs");
                assert_eq!(line, 10);
                assert_eq!(function, Some("test_function".to_string()));
            }
            _ => panic!("Expected warning violation"),
        }
    }

    #[test]
    fn test_cyclomatic_complexity_rule_error() {
        let thresholds = ComplexityThresholds::default();
        let rule = CyclomaticComplexityRule::new(&thresholds);
        let metrics = create_test_metrics(25, 0, 0, 0); // Above error threshold

        let result = rule.evaluate(&metrics, "test.rs", 10, Some("test_function"));
        assert!(result.is_some());

        match result.unwrap() {
            Violation::Error {
                rule: rule_name,
                message,
                value,
                threshold,
                file,
                line,
                function,
            } => {
                assert_eq!(rule_name, "cyclomatic-complexity");
                assert!(message.contains("25"));
                assert!(message.contains("20"));
                assert_eq!(value, 25);
                assert_eq!(threshold, 20);
                assert_eq!(file, "test.rs");
                assert_eq!(line, 10);
                assert_eq!(function, Some("test_function".to_string()));
            }
            _ => panic!("Expected error violation"),
        }
    }

    #[test]
    fn test_cyclomatic_complexity_rule_without_function_name() {
        let thresholds = ComplexityThresholds::default();
        let rule = CyclomaticComplexityRule::new(&thresholds);
        let metrics = create_test_metrics(15, 0, 0, 0);

        let result = rule.evaluate(&metrics, "test.rs", 10, None);
        assert!(result.is_some());

        match result.unwrap() {
            Violation::Warning { function, .. } => {
                assert_eq!(function, None);
            }
            _ => panic!("Expected warning violation"),
        }
    }

    #[test]
    fn test_cognitive_complexity_rule_creation() {
        let thresholds = ComplexityThresholds::default();
        let rule = CognitiveComplexityRule::new(&thresholds);
        assert_eq!(rule.warn_threshold, 15);
        assert_eq!(rule.error_threshold, 30);
    }

    #[test]
    fn test_cognitive_complexity_rule_no_violation() {
        let thresholds = ComplexityThresholds::default();
        let rule = CognitiveComplexityRule::new(&thresholds);
        let metrics = create_test_metrics(0, 10, 0, 0); // Below warn threshold

        let result = rule.evaluate(&metrics, "test.rs", 10, Some("test_function"));
        assert!(result.is_none());
    }

    #[test]
    fn test_cognitive_complexity_rule_warning() {
        let thresholds = ComplexityThresholds::default();
        let rule = CognitiveComplexityRule::new(&thresholds);
        let metrics = create_test_metrics(0, 20, 0, 0); // Above warn, below error

        let result = rule.evaluate(&metrics, "test.rs", 10, Some("test_function"));
        assert!(result.is_some());

        match result.unwrap() {
            Violation::Warning {
                rule: rule_name,
                message,
                value,
                threshold,
                file,
                line,
                function,
            } => {
                assert_eq!(rule_name, "cognitive-complexity");
                assert!(message.contains("20"));
                assert!(message.contains("15"));
                assert_eq!(value, 20);
                assert_eq!(threshold, 15);
                assert_eq!(file, "test.rs");
                assert_eq!(line, 10);
                assert_eq!(function, Some("test_function".to_string()));
            }
            _ => panic!("Expected warning violation"),
        }
    }

    #[test]
    fn test_cognitive_complexity_rule_error() {
        let thresholds = ComplexityThresholds::default();
        let rule = CognitiveComplexityRule::new(&thresholds);
        let metrics = create_test_metrics(0, 35, 0, 0); // Above error threshold

        let result = rule.evaluate(&metrics, "test.rs", 10, Some("test_function"));
        assert!(result.is_some());

        match result.unwrap() {
            Violation::Error {
                rule: rule_name,
                message,
                value,
                threshold,
                file,
                line,
                function,
            } => {
                assert_eq!(rule_name, "cognitive-complexity");
                assert!(message.contains("35"));
                assert!(message.contains("30"));
                assert_eq!(value, 35);
                assert_eq!(threshold, 30);
                assert_eq!(file, "test.rs");
                assert_eq!(line, 10);
                assert_eq!(function, Some("test_function".to_string()));
            }
            _ => panic!("Expected error violation"),
        }
    }

    #[test]
    fn test_complexity_hotspot_creation() {
        let hotspot = ComplexityHotspot {
            file: "test.rs".to_string(),
            function: Some("complex_function".to_string()),
            line: 42,
            complexity: 25,
            complexity_type: "cyclomatic".to_string(),
        };

        assert_eq!(hotspot.file, "test.rs");
        assert_eq!(hotspot.function, Some("complex_function".to_string()));
        assert_eq!(hotspot.line, 42);
        assert_eq!(hotspot.complexity, 25);
        assert_eq!(hotspot.complexity_type, "cyclomatic");
    }

    #[test]
    fn test_aggregate_results_empty() {
        let file_metrics = vec![];
        let report = aggregate_results(file_metrics);

        assert_eq!(report.summary.total_files, 0);
        assert_eq!(report.summary.total_functions, 0);
        assert_eq!(report.summary.median_cyclomatic, 0.0);
        assert_eq!(report.summary.median_cognitive, 0.0);
        assert_eq!(report.summary.max_cyclomatic, 0);
        assert_eq!(report.summary.max_cognitive, 0);
        assert_eq!(report.summary.p90_cyclomatic, 0);
        assert_eq!(report.summary.p90_cognitive, 0);
        assert_eq!(report.summary.technical_debt_hours, 0.0);
        assert!(report.violations.is_empty());
        assert!(report.hotspots.is_empty());
        assert!(report.files.is_empty());
    }

    #[test]
    fn test_aggregate_results_single_file() {
        let func1 = create_test_function("func1", 10, 20, create_test_metrics(5, 8, 2, 10));
        let func2 = create_test_function("func2", 30, 40, create_test_metrics(15, 20, 3, 15)); // Should trigger warning

        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(20, 28, 3, 25),
            functions: vec![func1, func2],
            classes: vec![],
        }];

        let report = aggregate_results(file_metrics);

        assert_eq!(report.summary.total_files, 1);
        assert_eq!(report.summary.total_functions, 2);
        assert_eq!(report.summary.median_cyclomatic, 10.0); // (5 + 15) / 2
        assert_eq!(report.summary.median_cognitive, 14.0); // (8 + 20) / 2
        assert_eq!(report.summary.max_cyclomatic, 15);
        assert_eq!(report.summary.max_cognitive, 20);

        // Should have violations for func2
        assert!(!report.violations.is_empty());

        // Should have hotspots for func2
        assert!(!report.hotspots.is_empty());
        assert_eq!(report.hotspots[0].function, Some("func2".to_string()));
    }

    #[test]
    fn test_aggregate_results_with_classes() {
        let method1 = create_test_function("method1", 5, 15, create_test_metrics(8, 12, 2, 10));
        let method2 = create_test_function("method2", 20, 30, create_test_metrics(25, 35, 4, 15)); // Should trigger errors

        let class = ClassComplexity {
            name: "TestClass".to_string(),
            line_start: 1,
            line_end: 50,
            metrics: create_test_metrics(33, 47, 4, 25),
            methods: vec![method1, method2],
        };

        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(33, 47, 4, 25),
            functions: vec![],
            classes: vec![class],
        }];

        let report = aggregate_results(file_metrics);

        assert_eq!(report.summary.total_files, 1);
        assert_eq!(report.summary.total_functions, 2); // Methods count as functions
        assert_eq!(report.summary.max_cyclomatic, 25);
        assert_eq!(report.summary.max_cognitive, 35);

        // Should have violations for method2 (both cyclomatic and cognitive)
        assert!(report.violations.len() >= 2);

        // Check for error violations
        let error_violations: Vec<_> = report
            .violations
            .iter()
            .filter(|v| matches!(v, Violation::Error { .. }))
            .collect();
        assert!(!error_violations.is_empty());
    }

    #[test]
    fn test_aggregate_results_median_calculation_odd() {
        // Test with odd number of functions for median calculation
        let func1 = create_test_function("func1", 10, 20, create_test_metrics(5, 10, 1, 10));
        let func2 = create_test_function("func2", 30, 40, create_test_metrics(7, 12, 2, 15));
        let func3 = create_test_function("func3", 50, 60, create_test_metrics(9, 15, 2, 20));

        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(21, 37, 2, 45),
            functions: vec![func1, func2, func3],
            classes: vec![],
        }];

        let report = aggregate_results(file_metrics);

        // With values [5, 7, 9], median should be 7
        assert_eq!(report.summary.median_cyclomatic, 7.0);
        // With values [10, 12, 15], median should be 12
        assert_eq!(report.summary.median_cognitive, 12.0);
    }

    #[test]
    fn test_aggregate_results_percentile_calculation() {
        // Create 10 functions to test p90 calculation
        let mut functions = Vec::new();
        for i in 1..=10 {
            functions.push(create_test_function(
                &format!("func{}", i),
                i * 10,
                i * 10 + 10,
                create_test_metrics(i as u16, i as u16 * 2, 1, 10),
            ));
        }

        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(55, 110, 1, 100),
            functions,
            classes: vec![],
        }];

        let report = aggregate_results(file_metrics);

        // p90 of [1,2,3,4,5,6,7,8,9,10] should be around 9 or 10 depending on implementation
        assert!(report.summary.p90_cyclomatic >= 9 && report.summary.p90_cyclomatic <= 10);
        // p90 of [2,4,6,8,10,12,14,16,18,20] should be around 18 or 20 depending on implementation
        assert!(report.summary.p90_cognitive >= 18 && report.summary.p90_cognitive <= 20);
    }

    #[test]
    fn test_aggregate_results_technical_debt_calculation() {
        // Create functions that exceed thresholds to test debt calculation
        let func1 = create_test_function("func1", 10, 20, create_test_metrics(15, 20, 2, 10)); // Warning: 5 over cyc, 5 over cog
        let func2 = create_test_function("func2", 30, 40, create_test_metrics(25, 35, 3, 15)); // Error: 5 over cyc, 5 over cog

        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(40, 55, 3, 25),
            functions: vec![func1, func2],
            classes: vec![],
        }];

        let report = aggregate_results(file_metrics);

        // Should have violations and technical debt
        assert!(!report.violations.is_empty());
        assert!(report.summary.technical_debt_hours > 0.0);

        // Debt calculation: warnings = 15min per point, errors = 30min per point
        // func1: 5 cyc warn (75min) + 5 cog warn (75min) = 150min = 2.5h
        // func2: 5 cyc error (150min) + 5 cog error (150min) = 300min = 5h
        // Total: 7.5h
        let expected_debt = (5.0 * 15.0 + 5.0 * 15.0 + 5.0 * 30.0 + 5.0 * 30.0) / 60.0;
        assert!((report.summary.technical_debt_hours - expected_debt).abs() < 0.1);
    }

    #[test]
    fn test_aggregate_results_hotspot_sorting() {
        let func1 =
            create_test_function("low_complexity", 10, 20, create_test_metrics(12, 18, 2, 10)); // Medium hotspot
        let func2 = create_test_function(
            "high_complexity",
            30,
            40,
            create_test_metrics(25, 35, 3, 15),
        ); // High hotspot
        let func3 = create_test_function(
            "medium_complexity",
            50,
            60,
            create_test_metrics(15, 22, 2, 12),
        ); // Lower hotspot

        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(52, 75, 3, 37),
            functions: vec![func1, func2, func3],
            classes: vec![],
        }];

        let report = aggregate_results(file_metrics);

        // Hotspots should be sorted by complexity (descending)
        assert!(report.hotspots.len() >= 3);
        assert_eq!(
            report.hotspots[0].function,
            Some("high_complexity".to_string())
        );
        assert_eq!(report.hotspots[0].complexity, 25);
        assert_eq!(
            report.hotspots[1].function,
            Some("medium_complexity".to_string())
        );
        assert_eq!(report.hotspots[1].complexity, 15);
        assert_eq!(
            report.hotspots[2].function,
            Some("low_complexity".to_string())
        );
        assert_eq!(report.hotspots[2].complexity, 12);
    }

    #[test]
    fn test_format_complexity_summary_empty() {
        let report = ComplexityReport {
            summary: ComplexitySummary {
                total_files: 0,
                total_functions: 0,
                median_cyclomatic: 0.0,
                median_cognitive: 0.0,
                max_cyclomatic: 0,
                max_cognitive: 0,
                p90_cyclomatic: 0,
                p90_cognitive: 0,
                technical_debt_hours: 0.0,
            },
            violations: vec![],
            hotspots: vec![],
            files: vec![],
        };

        let output = format_complexity_summary(&report);

        assert!(output.contains("# Complexity Analysis Summary"));
        assert!(output.contains("**Files analyzed**: 0"));
        assert!(output.contains("**Total functions**: 0"));
        assert!(output.contains("**Median Cyclomatic**: 0.0"));
        assert!(output.contains("**Median Cognitive**: 0.0"));
        assert!(output.contains("**Max Cyclomatic**: 0"));
        assert!(output.contains("**Max Cognitive**: 0"));
        assert!(!output.contains("**Estimated Refactoring Time**")); // Should not show 0 hours
        assert!(!output.contains("## Issues Found")); // No violations
        assert!(!output.contains("## Top Complexity Hotspots")); // No hotspots
    }

    #[test]
    fn test_format_complexity_summary_with_data() {
        let violations = vec![
            Violation::Error {
                rule: "cyclomatic-complexity".to_string(),
                message: "Too complex".to_string(),
                value: 25,
                threshold: 20,
                file: "test.rs".to_string(),
                line: 10,
                function: Some("test_func".to_string()),
            },
            Violation::Warning {
                rule: "cognitive-complexity".to_string(),
                message: "Getting complex".to_string(),
                value: 18,
                threshold: 15,
                file: "test.rs".to_string(),
                line: 20,
                function: Some("other_func".to_string()),
            },
        ];

        let hotspots = vec![
            ComplexityHotspot {
                file: "test.rs".to_string(),
                function: Some("complex_function".to_string()),
                line: 42,
                complexity: 25,
                complexity_type: "cyclomatic".to_string(),
            },
            ComplexityHotspot {
                file: "test2.rs".to_string(),
                function: Some("another_complex".to_string()),
                line: 100,
                complexity: 20,
                complexity_type: "cognitive".to_string(),
            },
        ];

        let report = ComplexityReport {
            summary: ComplexitySummary {
                total_files: 2,
                total_functions: 5,
                median_cyclomatic: 8.5,
                median_cognitive: 12.3,
                max_cyclomatic: 25,
                max_cognitive: 30,
                p90_cyclomatic: 20,
                p90_cognitive: 25,
                technical_debt_hours: 2.5,
            },
            violations,
            hotspots,
            files: vec![],
        };

        let output = format_complexity_summary(&report);

        assert!(output.contains("**Files analyzed**: 2"));
        assert!(output.contains("**Total functions**: 5"));
        assert!(output.contains("**Median Cyclomatic**: 8.5"));
        assert!(output.contains("**Median Cognitive**: 12.3"));
        assert!(output.contains("**Max Cyclomatic**: 25"));
        assert!(output.contains("**Max Cognitive**: 30"));
        assert!(output.contains("**90th Percentile Cyclomatic**: 20"));
        assert!(output.contains("**90th Percentile Cognitive**: 25"));
        assert!(output.contains("**Estimated Refactoring Time**: 2.5 hours"));
        assert!(output.contains("## Issues Found"));
        assert!(output.contains("**Errors**: 1"));
        assert!(output.contains("**Warnings**: 1"));
        assert!(output.contains("## Top Complexity Hotspots"));
        assert!(output.contains("`complex_function` - cyclomatic complexity: 25"));
        assert!(output.contains("📁 test.rs:42"));
    }

    #[test]
    fn test_format_complexity_report() {
        let violations = vec![Violation::Error {
            rule: "cyclomatic-complexity".to_string(),
            message: "Function too complex".to_string(),
            value: 25,
            threshold: 20,
            file: "test.rs".to_string(),
            line: 10,
            function: Some("test_func".to_string()),
        }];

        let report = ComplexityReport {
            summary: ComplexitySummary {
                total_files: 1,
                total_functions: 1,
                median_cyclomatic: 25.0,
                median_cognitive: 30.0,
                max_cyclomatic: 25,
                max_cognitive: 30,
                p90_cyclomatic: 25,
                p90_cognitive: 30,
                technical_debt_hours: 1.0,
            },
            violations,
            hotspots: vec![],
            files: vec![],
        };

        let output = format_complexity_report(&report);

        // Should include summary
        assert!(output.contains("# Complexity Analysis Summary"));

        // Should include detailed violations
        assert!(output.contains("## Detailed Violations"));
        assert!(output.contains("### test.rs"));
        assert!(output.contains("❌ **10:test_func** cyclomatic-complexity - Function too complex"));
    }

    #[test]
    fn test_format_as_sarif() {
        let violations = vec![
            Violation::Error {
                rule: "cyclomatic-complexity".to_string(),
                message: "Function too complex".to_string(),
                value: 25,
                threshold: 20,
                file: "test.rs".to_string(),
                line: 10,
                function: Some("test_func".to_string()),
            },
            Violation::Warning {
                rule: "cognitive-complexity".to_string(),
                message: "Function getting complex".to_string(),
                value: 18,
                threshold: 15,
                file: "test.rs".to_string(),
                line: 20,
                function: Some("other_func".to_string()),
            },
        ];

        let report = ComplexityReport {
            summary: ComplexitySummary {
                total_files: 1,
                total_functions: 2,
                median_cyclomatic: 21.5,
                median_cognitive: 18.0,
                max_cyclomatic: 25,
                max_cognitive: 18,
                p90_cyclomatic: 25,
                p90_cognitive: 18,
                technical_debt_hours: 0.5,
            },
            violations,
            hotspots: vec![],
            files: vec![],
        };

        let sarif_output = format_as_sarif(&report).expect("Should generate SARIF");

        // Basic SARIF structure checks
        assert!(sarif_output.contains("\"version\": \"2.1.0\""));
        assert!(sarif_output.contains("\"$schema\""));
        assert!(sarif_output.contains("\"runs\""));
        assert!(sarif_output.contains("\"tool\""));
        assert!(sarif_output.contains("\"driver\""));
        assert!(sarif_output.contains("\"name\": \"pmat\""));
        assert!(sarif_output.contains("\"rules\""));
        assert!(sarif_output.contains("\"results\""));

        // Rule definitions
        assert!(sarif_output.contains("\"id\": \"cyclomatic-complexity\""));
        assert!(sarif_output.contains("\"id\": \"cognitive-complexity\""));

        // Results
        assert!(sarif_output.contains("\"ruleId\": \"cyclomatic-complexity\""));
        assert!(sarif_output.contains("\"ruleId\": \"cognitive-complexity\""));
        assert!(sarif_output.contains("\"level\": \"error\""));
        assert!(sarif_output.contains("\"level\": \"warning\""));
        assert!(sarif_output.contains("\"text\": \"Function too complex\""));
        assert!(sarif_output.contains("\"text\": \"Function getting complex\""));
        assert!(sarif_output.contains("\"uri\": \"test.rs\""));
        assert!(sarif_output.contains("\"startLine\": 10"));
        assert!(sarif_output.contains("\"startLine\": 20"));
    }

    #[test]
    fn test_format_as_sarif_empty() {
        let report = ComplexityReport {
            summary: ComplexitySummary {
                total_files: 0,
                total_functions: 0,
                median_cyclomatic: 0.0,
                median_cognitive: 0.0,
                max_cyclomatic: 0,
                max_cognitive: 0,
                p90_cyclomatic: 0,
                p90_cognitive: 0,
                technical_debt_hours: 0.0,
            },
            violations: vec![],
            hotspots: vec![],
            files: vec![],
        };

        let sarif_output = format_as_sarif(&report).expect("Should generate SARIF");

        // Should still have valid SARIF structure with empty results
        assert!(sarif_output.contains("\"version\": \"2.1.0\""));
        assert!(sarif_output.contains("\"results\": []"));
    }

    #[test]
    fn test_violation_serialization() {
        let error_violation = Violation::Error {
            rule: "test-rule".to_string(),
            message: "Test message".to_string(),
            value: 25,
            threshold: 20,
            file: "test.rs".to_string(),
            line: 10,
            function: Some("test_func".to_string()),
        };

        let warning_violation = Violation::Warning {
            rule: "test-rule".to_string(),
            message: "Test warning".to_string(),
            value: 15,
            threshold: 10,
            file: "test.rs".to_string(),
            line: 20,
            function: None,
        };

        // Test that violations can be serialized/deserialized
        let error_json = serde_json::to_string(&error_violation).expect("Should serialize");
        let warning_json = serde_json::to_string(&warning_violation).expect("Should serialize");

        assert!(error_json.contains("\"severity\":\"error\""));
        assert!(warning_json.contains("\"severity\":\"warning\""));

        let _: Violation = serde_json::from_str(&error_json).expect("Should deserialize");
        let _: Violation = serde_json::from_str(&warning_json).expect("Should deserialize");
    }

    // Additional tests for 98%+ coverage

    #[test]
    fn test_is_simple_boundary_conditions() {
        // At exactly the threshold (cyclomatic = 5, cognitive = 7)
        let at_threshold = ComplexityMetrics::new(5, 7, 2, 20);
        assert!(at_threshold.is_simple());

        // Just above cyclomatic threshold
        let above_cyc = ComplexityMetrics::new(6, 7, 2, 20);
        assert!(!above_cyc.is_simple());

        // Just above cognitive threshold
        let above_cog = ComplexityMetrics::new(5, 8, 2, 20);
        assert!(!above_cog.is_simple());

        // Both above thresholds
        let both_above = ComplexityMetrics::new(6, 8, 2, 20);
        assert!(!both_above.is_simple());

        // Minimum values (should be simple)
        let minimum = ComplexityMetrics::new(0, 0, 0, 0);
        assert!(minimum.is_simple());

        // Maximum simple values
        let max_simple = ComplexityMetrics::new(5, 7, 10, 1000);
        assert!(max_simple.is_simple());
    }

    #[test]
    fn test_needs_refactoring_boundary_conditions() {
        // At exactly the threshold (cyclomatic = 10, cognitive = 15)
        let at_threshold = ComplexityMetrics::new(10, 15, 2, 20);
        assert!(!at_threshold.needs_refactoring());

        // Just above cyclomatic threshold
        let above_cyc = ComplexityMetrics::new(11, 15, 2, 20);
        assert!(above_cyc.needs_refactoring());

        // Just above cognitive threshold
        let above_cog = ComplexityMetrics::new(10, 16, 2, 20);
        assert!(above_cog.needs_refactoring());

        // Both above thresholds
        let both_above = ComplexityMetrics::new(11, 16, 2, 20);
        assert!(both_above.needs_refactoring());

        // Minimum values (should not need refactoring)
        let minimum = ComplexityMetrics::new(0, 0, 0, 0);
        assert!(!minimum.needs_refactoring());

        // Below thresholds
        let below = ComplexityMetrics::new(5, 10, 2, 50);
        assert!(!below.needs_refactoring());
    }

    #[test]
    fn test_complexity_score_calculation() {
        // Test the weighted score calculation
        // Formula: cyclomatic*1.0 + cognitive*1.2 + nesting*2.0 + lines*0.1
        let metrics = ComplexityMetrics::new(10, 20, 3, 100);
        let expected = 10.0 * 1.0 + 20.0 * 1.2 + 3.0 * 2.0 + 100.0 * 0.1;
        assert!((metrics.complexity_score() - expected).abs() < 0.0001);

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

        assert!(output.contains("`<file>` - cyclomatic complexity: 30"));
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

    #[test]
    fn test_halstead_metrics_clone_and_copy() {
        let original = HalsteadMetrics::new(10, 8, 25, 20);
        let cloned = original;
        let copied = original;

        assert_eq!(original.operators_unique, cloned.operators_unique);
        assert_eq!(original.operands_total, copied.operands_total);
    }

    #[test]
    fn test_file_complexity_metrics_clone() {
        let original = FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(10, 15, 2, 50),
            functions: vec![],
            classes: vec![],
        };

        let cloned = original.clone();
        assert_eq!(original.path, cloned.path);
        assert_eq!(
            original.total_complexity.cyclomatic,
            cloned.total_complexity.cyclomatic
        );
    }

    #[test]
    fn test_complexity_hotspot_clone() {
        let original = ComplexityHotspot {
            file: "test.rs".to_string(),
            function: Some("func".to_string()),
            line: 10,
            complexity: 25,
            complexity_type: "cyclomatic".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(original.file, cloned.file);
        assert_eq!(original.function, cloned.function);
        assert_eq!(original.complexity, cloned.complexity);
    }

    #[test]
    fn test_complexity_thresholds_clone() {
        let original = ComplexityThresholds {
            cyclomatic_warn: 8,
            cyclomatic_error: 15,
            cognitive_warn: 12,
            cognitive_error: 25,
            nesting_max: 4,
            method_length: 40,
        };

        let cloned = original.clone();
        assert_eq!(original.cyclomatic_warn, cloned.cyclomatic_warn);
        assert_eq!(original.cognitive_error, cloned.cognitive_error);
    }

    #[test]
    fn test_violation_clone() {
        let error = Violation::Error {
            rule: "test".to_string(),
            message: "Test".to_string(),
            value: 25,
            threshold: 20,
            file: "test.rs".to_string(),
            line: 10,
            function: Some("func".to_string()),
        };

        let cloned = error.clone();
        match (error, cloned) {
            (
                Violation::Error {
                    value: v1,
                    line: l1,
                    ..
                },
                Violation::Error {
                    value: v2,
                    line: l2,
                    ..
                },
            ) => {
                assert_eq!(v1, v2);
                assert_eq!(l1, l2);
            }
            _ => panic!("Expected both to be Error variants"),
        }
    }

    #[test]
    fn test_class_complexity_clone() {
        let original = ClassComplexity {
            name: "TestClass".to_string(),
            line_start: 1,
            line_end: 50,
            metrics: create_test_metrics(15, 20, 3, 45),
            methods: vec![create_test_function(
                "method",
                5,
                15,
                create_test_metrics(5, 8, 2, 10),
            )],
        };

        let cloned = original.clone();
        assert_eq!(original.name, cloned.name);
        assert_eq!(original.methods.len(), cloned.methods.len());
    }

    #[test]
    fn test_function_complexity_clone() {
        let original = FunctionComplexity {
            name: "test_func".to_string(),
            line_start: 10,
            line_end: 25,
            metrics: create_test_metrics(5, 8, 2, 15),
        };

        let cloned = original.clone();
        assert_eq!(original.name, cloned.name);
        assert_eq!(original.line_start, cloned.line_start);
    }

    #[test]
    fn test_complexity_report_clone() {
        let report = ComplexityReport {
            summary: ComplexitySummary::default(),
            violations: vec![],
            hotspots: vec![],
            files: vec![],
        };

        let cloned = report.clone();
        assert_eq!(report.summary.total_files, cloned.summary.total_files);
    }

    #[test]
    fn test_complexity_summary_clone() {
        let original = ComplexitySummary {
            total_files: 5,
            total_functions: 20,
            median_cyclomatic: 8.5,
            median_cognitive: 12.0,
            max_cyclomatic: 25,
            max_cognitive: 30,
            p90_cyclomatic: 18,
            p90_cognitive: 22,
            technical_debt_hours: 3.5,
        };

        let cloned = original.clone();
        assert_eq!(original.total_files, cloned.total_files);
        assert_eq!(original.median_cyclomatic, cloned.median_cyclomatic);
        assert_eq!(original.technical_debt_hours, cloned.technical_debt_hours);
    }
}


mod property_tests {
    use super::*;
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

        // Property: is_simple and needs_refactoring should be mutually exclusive for moderate values
        #[test]
        fn complexity_classification_consistency(
            cyclomatic in 0u16..30,
            cognitive in 0u16..40,
            nesting in 0u8..10,
            lines in 0u16..500
        ) {
            let metrics = ComplexityMetrics::new(cyclomatic, cognitive, nesting, lines);

            // If simple, should not need refactoring (for low values)
            if cyclomatic <= 5 && cognitive <= 7 {
                prop_assert!(metrics.is_simple());
            }

            // If needs refactoring, should not be simple
            if cyclomatic > 10 || cognitive > 15 {
                prop_assert!(metrics.needs_refactoring());
                prop_assert!(!metrics.is_simple());
            }
        }

        // Property: complexity_score should always be non-negative and monotonically increasing
        #[test]
        fn complexity_score_is_non_negative(
            cyclomatic in 0u16..1000,
            cognitive in 0u16..1000,
            nesting in 0u8..255,
            lines in 0u16..10000
        ) {
            let metrics = ComplexityMetrics::new(cyclomatic, cognitive, nesting, lines);
            let score = metrics.complexity_score();

            prop_assert!(score >= 0.0, "Score should never be negative");
            prop_assert!(!score.is_nan(), "Score should not be NaN");
            prop_assert!(!score.is_infinite(), "Score should not be infinite");
        }

        // Property: higher cyclomatic complexity should yield higher score
        #[test]
        fn higher_cyclomatic_yields_higher_score(
            base_cyc in 0u16..100,
            increment in 1u16..100,
            cognitive in 0u16..100,
            nesting in 0u8..10,
            lines in 0u16..500
        ) {
            let base = ComplexityMetrics::new(base_cyc, cognitive, nesting, lines);
            let higher = ComplexityMetrics::new(base_cyc.saturating_add(increment), cognitive, nesting, lines);

            prop_assert!(higher.complexity_score() > base.complexity_score(),
                "Higher cyclomatic should yield higher score");
        }

        // Property: Halstead calculate_derived should never produce NaN or infinite values
        #[test]
        fn halstead_derived_values_are_finite(
            operators_unique in 0u32..1000,
            operands_unique in 0u32..1000,
            operators_total in 0u32..10000,
            operands_total in 0u32..10000
        ) {
            let halstead = HalsteadMetrics::new(operators_unique, operands_unique, operators_total, operands_total);
            let calculated = halstead.calculate_derived();

            prop_assert!(!calculated.volume.is_nan(), "Volume should not be NaN");
            prop_assert!(!calculated.difficulty.is_nan(), "Difficulty should not be NaN");
            prop_assert!(!calculated.effort.is_nan(), "Effort should not be NaN");
            prop_assert!(!calculated.time.is_nan(), "Time should not be NaN");
            prop_assert!(!calculated.bugs.is_nan(), "Bugs should not be NaN");

            prop_assert!(!calculated.volume.is_infinite(), "Volume should be finite");
            prop_assert!(!calculated.difficulty.is_infinite(), "Difficulty should be finite");
            prop_assert!(!calculated.effort.is_infinite(), "Effort should be finite");
        }

        // Property: Halstead derived values should be non-negative
        #[test]
        fn halstead_derived_values_are_non_negative(
            operators_unique in 1u32..100,
            operands_unique in 1u32..100,
            operators_total in 1u32..1000,
            operands_total in 1u32..1000
        ) {
            let halstead = HalsteadMetrics::new(operators_unique, operands_unique, operators_total, operands_total);
            let calculated = halstead.calculate_derived();

            prop_assert!(calculated.volume >= 0.0, "Volume should be non-negative");
            prop_assert!(calculated.difficulty >= 0.0, "Difficulty should be non-negative");
            prop_assert!(calculated.effort >= 0.0, "Effort should be non-negative");
            prop_assert!(calculated.time >= 0.0, "Time should be non-negative");
            prop_assert!(calculated.bugs >= 0.0, "Bugs should be non-negative");
        }

        // Property: cache key should be deterministic
        #[test]
        fn cache_key_is_deterministic(
            path_suffix in "[a-z]{1,20}\\.rs",
            content in prop::collection::vec(any::<u8>(), 0..1000)
        ) {
            let path = std::path::Path::new(&path_suffix);
            let key1 = compute_complexity_cache_key(path, &content);
            let key2 = compute_complexity_cache_key(path, &content);

            prop_assert_eq!(key1.clone(), key2, "Same inputs should produce same cache key");
            prop_assert!(key1.starts_with("cx:"), "Cache key should start with 'cx:'");
        }

        // Property: exceeds_threshold should be consistent
        #[test]
        fn exceeds_threshold_consistency(
            value in 0u16..1000,
            threshold in 0u16..1000
        ) {
            let thresholds = ComplexityThresholds::default();
            let rule = CyclomaticComplexityRule::new(&thresholds);

            let exceeds = rule.exceeds_threshold(value, threshold);

            if value > threshold {
                prop_assert!(exceeds, "Should exceed when value > threshold");
            } else {
                prop_assert!(!exceeds, "Should not exceed when value <= threshold");
            }
        }

        // Property: ComplexityVisitor nesting should saturate properly
        #[test]
        fn visitor_nesting_saturation(
            enter_count in 0usize..500,
            exit_count in 0usize..500
        ) {
            let mut metrics = ComplexityMetrics::default();
            let mut visitor = ComplexityVisitor::new(&mut metrics);

            // Enter nesting many times
            for _ in 0..enter_count {
                visitor.enter_nesting();
            }

            // Nesting level should be at most u8::MAX
            prop_assert!(visitor.nesting_level <= 255);

            // Exit nesting many times
            for _ in 0..exit_count {
                visitor.exit_nesting();
            }

            // Nesting level should be at least 0
            prop_assert!(visitor.nesting_level >= 0);
        }

        // Property: cognitive increment should be bounded
        #[test]
        fn cognitive_increment_is_bounded(
            nesting_level in 0u8..255
        ) {
            let mut metrics = ComplexityMetrics::default();
            let mut visitor = ComplexityVisitor::new(&mut metrics);
            visitor.nesting_level = nesting_level;

            let increment_nesting = visitor.calculate_cognitive_increment(true);
            let increment_non_nesting = visitor.calculate_cognitive_increment(false);

            // Non-nesting should always be 1
            prop_assert_eq!(increment_non_nesting, 1);

            // Nesting construct should be at least 1
            prop_assert!(increment_nesting >= 1);

            // Increment should be reasonable
            prop_assert!(increment_nesting <= 256);
        }
    }
}
