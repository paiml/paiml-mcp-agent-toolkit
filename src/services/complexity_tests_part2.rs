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
        assert!(output.contains("`complex_function` test.rs:42 - cyclomatic complexity: 25"));
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

