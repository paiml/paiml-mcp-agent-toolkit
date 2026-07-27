
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
