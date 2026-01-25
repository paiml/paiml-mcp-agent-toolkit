// Tests for complexity service
// Extracted for file health compliance (CB-040)

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
