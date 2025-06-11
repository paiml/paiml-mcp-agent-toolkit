//! Comprehensive test suite for complexity module to achieve 80% code coverage

#[cfg(test)]
mod tests {
    use super::super::complexity::*;
    use std::path::Path;

    // Helper function to create test complexity metrics
    fn create_test_metrics(cyclomatic: u16, cognitive: u16, nesting_max: u8, lines: u16) -> ComplexityMetrics {
        ComplexityMetrics {
            cyclomatic,
            cognitive,
            nesting_max,
            lines,
        }
    }

    // Helper function to create test function complexity
    fn create_test_function(name: &str, line_start: u32, line_end: u32, metrics: ComplexityMetrics) -> FunctionComplexity {
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
            Violation::Warning { rule: rule_name, message, value, threshold, file, line, function } => {
                assert_eq!(rule_name, "cyclomatic-complexity");
                assert!(message.contains("15"));
                assert!(message.contains("10"));
                assert_eq!(value, 15);
                assert_eq!(threshold, 10);
                assert_eq!(file, "test.rs");
                assert_eq!(line, 10);
                assert_eq!(function, Some("test_function".to_string()));
            },
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
            Violation::Error { rule: rule_name, message, value, threshold, file, line, function } => {
                assert_eq!(rule_name, "cyclomatic-complexity");
                assert!(message.contains("25"));
                assert!(message.contains("20"));
                assert_eq!(value, 25);
                assert_eq!(threshold, 20);
                assert_eq!(file, "test.rs");
                assert_eq!(line, 10);
                assert_eq!(function, Some("test_function".to_string()));
            },
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
            },
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
            Violation::Warning { rule: rule_name, message, value, threshold, file, line, function } => {
                assert_eq!(rule_name, "cognitive-complexity");
                assert!(message.contains("20"));
                assert!(message.contains("15"));
                assert_eq!(value, 20);
                assert_eq!(threshold, 15);
                assert_eq!(file, "test.rs");
                assert_eq!(line, 10);
                assert_eq!(function, Some("test_function".to_string()));
            },
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
            Violation::Error { rule: rule_name, message, value, threshold, file, line, function } => {
                assert_eq!(rule_name, "cognitive-complexity");
                assert!(message.contains("35"));
                assert!(message.contains("30"));
                assert_eq!(value, 35);
                assert_eq!(threshold, 30);
                assert_eq!(file, "test.rs");
                assert_eq!(line, 10);
                assert_eq!(function, Some("test_function".to_string()));
            },
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
        let error_violations: Vec<_> = report.violations.iter()
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
                create_test_metrics(i as u16, i as u16 * 2, 1, 10)
            ));
        }
        
        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(55, 110, 1, 100),
            functions,
            classes: vec![],
        }];
        
        let report = aggregate_results(file_metrics);
        
        // p90 of [1,2,3,4,5,6,7,8,9,10] should be around 9
        assert_eq!(report.summary.p90_cyclomatic, 9);
        // p90 of [2,4,6,8,10,12,14,16,18,20] should be around 18
        assert_eq!(report.summary.p90_cognitive, 18);
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
        let func1 = create_test_function("low_complexity", 10, 20, create_test_metrics(12, 18, 2, 10)); // Medium hotspot
        let func2 = create_test_function("high_complexity", 30, 40, create_test_metrics(25, 35, 3, 15)); // High hotspot
        let func3 = create_test_function("medium_complexity", 50, 60, create_test_metrics(15, 22, 2, 12)); // Lower hotspot
        
        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(52, 75, 3, 37),
            functions: vec![func1, func2, func3],
            classes: vec![],
        }];
        
        let report = aggregate_results(file_metrics);
        
        // Hotspots should be sorted by complexity (descending)
        assert!(report.hotspots.len() >= 3);
        assert_eq!(report.hotspots[0].function, Some("high_complexity".to_string()));
        assert_eq!(report.hotspots[0].complexity, 25);
        assert_eq!(report.hotspots[1].function, Some("medium_complexity".to_string()));
        assert_eq!(report.hotspots[1].complexity, 15);
        assert_eq!(report.hotspots[2].function, Some("low_complexity".to_string()));
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
        ];
        
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
        assert!(sarif_output.contains("\"name\": \"paiml-mcp-agent-toolkit\""));
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
}