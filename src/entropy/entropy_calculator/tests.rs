#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for entropy calculator module.

#[cfg(test)]
mod tests {
    use crate::entropy::entropy_calculator::*;
    use crate::entropy::violation_detector::{ActionableViolation, PatternSummary};
    use crate::entropy::PatternType;
    use std::collections::BTreeMap;

    #[test]
    fn test_entropy_metrics_creation() {
        let metrics = EntropyMetrics {
            file_level_entropy: Some(0.5),
            module_level_entropy: Some(0.6),
            project_level_entropy: Some(0.7),
            pattern_diversity: Some(0.4),
            total_patterns: 10,
            total_instances: 50,
            total_loc: 1000,
            patterns_by_type: BTreeMap::new(),
        };

        assert_eq!(metrics.total_patterns, 10);
        assert_eq!(metrics.total_instances, 50);
    }

    #[test]
    fn test_entropy_report_calculations() {
        let report = EntropyReport {
            total_files_analyzed: 10,
            actionable_violations: vec![ActionableViolation {
                severity: crate::entropy::Severity::High,
                pattern: PatternSummary {
                    pattern_type: PatternType::ErrorHandling,
                    repetitions: 10,
                    variation_score: 0.0,
                    example_code: "test".to_string(),
                },
                message: "test".to_string(),
                fix_suggestion: "test".to_string(),
                estimated_loc_reduction: 100,
                affected_files: vec![],
                priority_score: 10.0,
            }],
            pattern_summary: PatternSummary {
                pattern_type: PatternType::ErrorHandling,
                repetitions: 10,
                variation_score: 0.0,
                example_code: "test".to_string(),
            },
            measurement_note: None,
            entropy_metrics: EntropyMetrics {
                file_level_entropy: Some(0.5),
                module_level_entropy: Some(0.6),
                project_level_entropy: Some(0.7),
                pattern_diversity: Some(0.4),
                total_patterns: 10,
                total_instances: 50,
                total_loc: 1000,
                patterns_by_type: BTreeMap::new(),
            },
        };

        assert_eq!(report.total_loc_reduction(), 100);
        assert_eq!(report.reduction_percentage(), 10.0);
    }
}

#[cfg(test)]
mod property_tests {
    use crate::entropy::entropy_calculator::EntropyMetrics;

    #[test]
    fn test_entropy_metrics_serialization() {
        use std::collections::BTreeMap;
        let metrics = EntropyMetrics {
            file_level_entropy: Some(2.5),
            module_level_entropy: Some(1.8),
            project_level_entropy: Some(3.2),
            pattern_diversity: Some(0.75),
            total_patterns: 10,
            total_instances: 50,
            total_loc: 1000,
            patterns_by_type: BTreeMap::new(),
        };

        let serialized = format!("{:?}", metrics);
        assert!(!serialized.is_empty());
        assert!(serialized.contains("EntropyMetrics"));
    }

    #[test]
    fn test_entropy_metrics_clone() {
        use std::collections::BTreeMap;
        let metrics = EntropyMetrics {
            file_level_entropy: Some(2.5),
            module_level_entropy: Some(1.8),
            project_level_entropy: Some(3.2),
            pattern_diversity: Some(0.75),
            total_patterns: 10,
            total_instances: 50,
            total_loc: 1000,
            patterns_by_type: BTreeMap::new(),
        };

        let cloned = metrics.clone();
        assert_eq!(format!("{:?}", metrics), format!("{:?}", cloned));
        assert_eq!(metrics.file_level_entropy, cloned.file_level_entropy);
        assert_eq!(metrics.pattern_diversity, cloned.pattern_diversity);
        assert_eq!(metrics.total_patterns, cloned.total_patterns);
    }

    #[test]
    fn test_entropy_metrics_memory_safety() {
        use std::collections::BTreeMap;
        let metrics = EntropyMetrics {
            file_level_entropy: Some(2.5),
            module_level_entropy: Some(1.8),
            project_level_entropy: Some(3.2),
            pattern_diversity: Some(0.75),
            total_patterns: 10,
            total_instances: 50,
            total_loc: 1000,
            patterns_by_type: BTreeMap::new(),
        };

        let _cloned = metrics.clone();
        let _size = std::mem::size_of_val(&metrics);

        // Memory safety verification - no panics or issues
        // Memory safety verification - no panics or issues in above calculations
    }
}

#[cfg(test)]
mod coverage_tests {
    use crate::entropy::entropy_calculator::*;
    use crate::entropy::pattern_extractor::{AstPattern, Location, PatternCollection};
    use crate::entropy::violation_detector::{ActionableViolation, PatternSummary};
    use crate::entropy::{EntropyConfig, PatternType, Severity};
    use proptest::prelude::*;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    // ============================================
    // EntropyMetrics tests
    // ============================================

    #[test]
    fn test_entropy_metrics_with_all_fields() {
        let mut patterns_by_type = BTreeMap::new();
        patterns_by_type.insert(PatternType::ErrorHandling, 10);
        patterns_by_type.insert(PatternType::DataValidation, 5);

        let metrics = EntropyMetrics {
            file_level_entropy: Some(0.85),
            module_level_entropy: Some(0.75),
            project_level_entropy: Some(0.70),
            pattern_diversity: Some(0.78),
            total_patterns: 42,
            total_instances: 156,
            total_loc: 2500,
            patterns_by_type,
        };

        assert_eq!(metrics.total_patterns, 42);
        assert_eq!(metrics.total_instances, 156);
        assert_eq!(metrics.total_loc, 2500);
        assert!(metrics.file_level_entropy.expect("measured") > 0.8);
        assert!(metrics.pattern_diversity.expect("measured") > 0.7);
        assert_eq!(
            metrics.patterns_by_type.get(&PatternType::ErrorHandling),
            Some(&10)
        );
    }

    #[test]
    fn test_entropy_metrics_serialization_json() {
        let metrics = EntropyMetrics {
            file_level_entropy: Some(0.5),
            module_level_entropy: Some(0.6),
            project_level_entropy: Some(0.7),
            pattern_diversity: Some(0.55),
            total_patterns: 5,
            total_instances: 25,
            total_loc: 500,
            patterns_by_type: BTreeMap::new(),
        };

        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("file_level_entropy"));
        assert!(json.contains("0.5"));

        let deserialized: EntropyMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(metrics.file_level_entropy, deserialized.file_level_entropy);
        assert_eq!(metrics.total_patterns, deserialized.total_patterns);
    }

    #[test]
    fn test_entropy_metrics_zero_values() {
        let metrics = EntropyMetrics {
            file_level_entropy: Some(0.0),
            module_level_entropy: Some(0.0),
            project_level_entropy: Some(0.0),
            pattern_diversity: Some(0.0),
            total_patterns: 0,
            total_instances: 0,
            total_loc: 0,
            patterns_by_type: BTreeMap::new(),
        };

        assert_eq!(metrics.total_loc, 0);
        assert_eq!(metrics.pattern_diversity, Some(0.0));
    }

    // ============================================
    // EntropyReport tests
    // ============================================

    fn create_test_report(violations: Vec<ActionableViolation>, total_loc: usize) -> EntropyReport {
        EntropyReport {
            total_files_analyzed: 10,
            actionable_violations: violations,
            pattern_summary: PatternSummary {
                pattern_type: PatternType::ErrorHandling,
                repetitions: 0,
                variation_score: 0.0,
                example_code: String::new(),
            },
            measurement_note: None,
            entropy_metrics: EntropyMetrics {
                file_level_entropy: Some(0.5),
                module_level_entropy: Some(0.6),
                project_level_entropy: Some(0.7),
                pattern_diversity: Some(0.55),
                total_patterns: 5,
                total_instances: 25,
                total_loc,
                patterns_by_type: BTreeMap::new(),
            },
        }
    }

    fn create_test_violation(severity: Severity, loc_reduction: usize) -> ActionableViolation {
        ActionableViolation {
            severity,
            pattern: PatternSummary {
                pattern_type: PatternType::ErrorHandling,
                repetitions: 5,
                variation_score: 0.1,
                example_code: "test code".to_string(),
            },
            message: "Test violation message".to_string(),
            fix_suggestion: "Fix suggestion".to_string(),
            estimated_loc_reduction: loc_reduction,
            affected_files: vec![PathBuf::from("test.rs")],
            priority_score: 5.0,
        }
    }

    #[test]
    fn test_entropy_report_total_loc_reduction_empty() {
        let report = create_test_report(vec![], 1000);
        assert_eq!(report.total_loc_reduction(), 0);
    }

    #[test]
    fn test_entropy_report_total_loc_reduction_single() {
        let violations = vec![create_test_violation(Severity::High, 100)];
        let report = create_test_report(violations, 1000);
        assert_eq!(report.total_loc_reduction(), 100);
    }

    #[test]
    fn test_entropy_report_total_loc_reduction_multiple() {
        let violations = vec![
            create_test_violation(Severity::High, 100),
            create_test_violation(Severity::Medium, 50),
            create_test_violation(Severity::Low, 25),
        ];
        let report = create_test_report(violations, 1000);
        assert_eq!(report.total_loc_reduction(), 175);
    }

    #[test]
    fn test_entropy_report_reduction_percentage_zero_loc() {
        let report = create_test_report(vec![], 0);
        assert_eq!(report.reduction_percentage(), 0.0);
    }

    #[test]
    fn test_entropy_report_reduction_percentage_normal() {
        let violations = vec![create_test_violation(Severity::High, 100)];
        let report = create_test_report(violations, 1000);
        assert!((report.reduction_percentage() - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_entropy_report_reduction_percentage_large() {
        let violations = vec![create_test_violation(Severity::High, 500)];
        let report = create_test_report(violations, 1000);
        assert!((report.reduction_percentage() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_entropy_report_format_report_header() {
        let report = create_test_report(vec![], 1000);
        let formatted = report.format_report();

        assert!(formatted.contains("Entropy Analysis Results"));
        assert!(formatted.contains("========================"));
        assert!(formatted.contains("Files Analyzed: 10"));
    }

    #[test]
    fn test_entropy_report_format_report_with_high_severity() {
        let violations = vec![create_test_violation(Severity::High, 100)];
        let report = create_test_report(violations, 1000);
        let formatted = report.format_report();

        assert!(formatted.contains("HIGH SEVERITY (1)"));
        assert!(formatted.contains("Test violation message"));
        assert!(formatted.contains("Fix suggestion"));
        assert!(formatted.contains("saves 100 lines"));
    }

    #[test]
    fn test_entropy_report_format_report_with_medium_severity() {
        let violations = vec![create_test_violation(Severity::Medium, 50)];
        let report = create_test_report(violations, 1000);
        let formatted = report.format_report();

        assert!(formatted.contains("MEDIUM SEVERITY (1)"));
    }

    #[test]
    fn test_entropy_report_format_report_with_low_severity() {
        // Low severity violations are not shown in the format_report
        let violations = vec![create_test_violation(Severity::Low, 25)];
        let report = create_test_report(violations, 1000);
        let formatted = report.format_report();

        // Low severity is not displayed in the report
        assert!(!formatted.contains("LOW SEVERITY"));
    }

    #[test]
    fn test_entropy_report_format_report_mixed_severity() {
        let violations = vec![
            create_test_violation(Severity::High, 100),
            create_test_violation(Severity::High, 75),
            create_test_violation(Severity::Medium, 50),
            create_test_violation(Severity::Medium, 30),
            create_test_violation(Severity::Low, 10),
        ];
        let report = create_test_report(violations, 1000);
        let formatted = report.format_report();

        assert!(formatted.contains("HIGH SEVERITY (2)"));
        assert!(formatted.contains("MEDIUM SEVERITY (2)"));
        assert!(formatted.contains("Total Potential Reduction: 265 lines"));
    }

    #[test]
    fn test_entropy_report_serialization() {
        let report = create_test_report(vec![], 1000);
        let json = serde_json::to_string(&report).unwrap();
        let deserialized: EntropyReport = serde_json::from_str(&json).unwrap();

        assert_eq!(
            report.total_files_analyzed,
            deserialized.total_files_analyzed
        );
        assert_eq!(
            report.entropy_metrics.total_loc,
            deserialized.entropy_metrics.total_loc
        );
    }

    #[test]
    fn test_entropy_report_clone() {
        let violations = vec![create_test_violation(Severity::High, 100)];
        let report = create_test_report(violations, 1000);
        let cloned = report.clone();

        assert_eq!(report.total_files_analyzed, cloned.total_files_analyzed);
        assert_eq!(
            report.actionable_violations.len(),
            cloned.actionable_violations.len()
        );
    }

    // ============================================
    // EntropyCalculator tests
    // ============================================

    #[test]
    fn test_entropy_calculator_new() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let _ = calculator; // Ensure it can be created
    }

    #[test]
    fn test_entropy_calculator_calculate_empty_patterns() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let patterns = PatternCollection::new();

        let metrics = calculator.calculate(&patterns).unwrap();

        assert_eq!(metrics.total_patterns, 0);
        assert_eq!(metrics.total_instances, 0);
        assert_eq!(metrics.total_loc, 0);
        // CONTRACT UPDATED (#650): with no patterns there is no distribution to
        // take the entropy of. This used to assert 0.0, i.e. "zero diversity" —
        // the worst possible score — for input that simply had nothing to repeat.
        assert_eq!(
            metrics.pattern_diversity, None,
            "an uncomputable diversity must be absent, not 0.0"
        );
    }

    #[test]
    fn test_entropy_calculator_calculate_single_pattern() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "hash1".to_string(),
            frequency: 5,
            locations: vec![Location {
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 1,
            }],
            variation_score: 0.1,
            example_code: "match result {}".to_string(),
            estimated_loc: 10,
        });

        let metrics = calculator.calculate(&patterns).unwrap();

        assert_eq!(metrics.total_patterns, 1);
        assert_eq!(metrics.total_instances, 5);
        // CONTRACT UPDATED (#650): total_loc is now the measured source size of
        // the analyzed input, not sum(estimated_loc * frequency). This collection
        // was hand-built with no files read, so nothing was measured.
        assert_eq!(metrics.total_loc, 0);
    }

    #[test]
    fn test_entropy_calculator_calculate_multiple_patterns() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "hash1".to_string(),
            frequency: 5,
            locations: vec![Location {
                file: PathBuf::from("test1.rs"),
                line: 10,
                column: 1,
            }],
            variation_score: 0.1,
            example_code: "error handling".to_string(),
            estimated_loc: 10,
        });

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::DataValidation,
            pattern_hash: "hash2".to_string(),
            frequency: 3,
            locations: vec![Location {
                file: PathBuf::from("test2.rs"),
                line: 20,
                column: 1,
            }],
            variation_score: 0.2,
            example_code: "validation".to_string(),
            estimated_loc: 5,
        });

        let metrics = calculator.calculate(&patterns).unwrap();

        assert_eq!(metrics.total_patterns, 2);
        assert_eq!(metrics.total_instances, 8); // 5 + 3
                                                // CONTRACT UPDATED (#650): measured source size, and nothing was read here.
        assert_eq!(metrics.total_loc, 0);
        assert!(metrics.pattern_diversity.expect("measurable") > 0.0);
    }

    #[test]
    fn test_entropy_calculator_patterns_by_type() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "hash1".to_string(),
            frequency: 5,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 10,
        });

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "hash2".to_string(),
            frequency: 3,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 5,
        });

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ControlFlow,
            pattern_hash: "hash3".to_string(),
            frequency: 2,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 8,
        });

        let metrics = calculator.calculate(&patterns).unwrap();

        assert_eq!(
            metrics.patterns_by_type.get(&PatternType::ErrorHandling),
            Some(&8)
        ); // 5+3
        assert_eq!(
            metrics.patterns_by_type.get(&PatternType::ControlFlow),
            Some(&2)
        );
    }

    #[test]
    fn test_calculate_pattern_diversity_empty() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let patterns = PatternCollection::new();

        let diversity = calculator.calculate_pattern_diversity(&patterns);
        assert_eq!(diversity, 0.0);
    }

    #[test]
    fn test_calculate_pattern_diversity_single_pattern() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "hash1".to_string(),
            frequency: 10,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 5,
        });

        let diversity = calculator.calculate_pattern_diversity(&patterns);
        // Single pattern = zero entropy (no diversity)
        assert_eq!(diversity, 0.0);
    }

    #[test]
    fn test_calculate_pattern_diversity_multiple_patterns() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        // Add multiple patterns with equal frequency for maximum diversity
        for i in 0..4 {
            patterns.add_pattern(AstPattern {
                pattern_type: PatternType::ErrorHandling,
                pattern_hash: format!("hash{}", i),
                frequency: 5,
                locations: vec![],
                variation_score: 0.0,
                example_code: "".to_string(),
                estimated_loc: 5,
            });
        }

        let diversity = calculator.calculate_pattern_diversity(&patterns);
        // Multiple patterns = positive entropy
        assert!(diversity > 0.0);
    }

    #[test]
    fn test_calculate_file_level_entropy_empty() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let patterns = PatternCollection::new();

        let entropy = calculator.calculate_file_level_entropy(&patterns);
        assert_eq!(entropy, 0.0);
    }

    #[test]
    fn test_calculate_module_level_entropy_empty() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let patterns = PatternCollection::new();

        let entropy = calculator.calculate_module_level_entropy(&patterns);
        assert_eq!(entropy, 0.0);
    }

    #[test]
    fn test_calculate_module_level_entropy_with_patterns() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        // Add patterns with locations in different modules
        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "hash1".to_string(),
            frequency: 5,
            locations: vec![
                Location {
                    file: PathBuf::from("src/mod1/file.rs"),
                    line: 10,
                    column: 1,
                },
                Location {
                    file: PathBuf::from("src/mod2/file.rs"),
                    line: 20,
                    column: 1,
                },
            ],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 5,
        });

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ControlFlow,
            pattern_hash: "hash2".to_string(),
            frequency: 3,
            locations: vec![Location {
                file: PathBuf::from("src/mod1/other.rs"),
                line: 15,
                column: 1,
            }],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 8,
        });

        let entropy = calculator.calculate_module_level_entropy(&patterns);
        // Should have some entropy from patterns in different modules
        assert!(entropy >= 0.0);
    }

    #[test]
    fn test_calculate_project_level_entropy() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        for i in 0..3 {
            patterns.add_pattern(AstPattern {
                pattern_type: PatternType::ErrorHandling,
                pattern_hash: format!("hash{}", i),
                frequency: 5,
                locations: vec![],
                variation_score: 0.0,
                example_code: "".to_string(),
                estimated_loc: 5,
            });
        }

        let entropy = calculator.calculate_project_level_entropy(&patterns);
        // Should be same as pattern diversity
        let diversity = calculator.calculate_pattern_diversity(&patterns);
        assert_eq!(entropy, diversity);
    }

    #[test]
    fn test_calculate_file_level_entropy_with_file_patterns() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        // Add pattern
        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "hash1".to_string(),
            frequency: 5,
            locations: vec![Location {
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 1,
            }],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 5,
        });

        // Manually add file patterns
        patterns.file_patterns.insert(
            PathBuf::from("test.rs"),
            vec![
                "hash1".to_string(),
                "hash1".to_string(),
                "hash2".to_string(),
            ],
        );

        let entropy = calculator.calculate_file_level_entropy(&patterns);
        // Should have some entropy from multiple patterns in the file
        assert!(entropy >= 0.0);
    }

    // ============================================
    // Property-based tests
    // ============================================

    proptest! {
        #[test]
        fn test_entropy_report_loc_reduction_non_negative(
            reductions in proptest::collection::vec(0usize..1000, 0..10)
        ) {
            let violations: Vec<ActionableViolation> = reductions
                .iter()
                .map(|&r| create_test_violation(Severity::Medium, r))
                .collect();
            let report = create_test_report(violations, 10000);

            prop_assert!(report.total_loc_reduction() <= 10000);
        }

        #[test]
        fn test_entropy_metrics_entropy_values_bounded(
            file_entropy in 0.0f64..=1.0,
            module_entropy in 0.0f64..=1.0,
            project_entropy in 0.0f64..=1.0,
            diversity in 0.0f64..=1.0,
        ) {
            let metrics = EntropyMetrics {
                file_level_entropy: Some(file_entropy),
                module_level_entropy: Some(module_entropy),
                project_level_entropy: Some(project_entropy),
                pattern_diversity: Some(diversity),
                total_patterns: 10,
                total_instances: 50,
                total_loc: 1000,
                patterns_by_type: BTreeMap::new(),
            };

            prop_assert!((0.0..=1.0).contains(&metrics.file_level_entropy.unwrap()));
            prop_assert!((0.0..=1.0).contains(&metrics.module_level_entropy.unwrap()));
            prop_assert!((0.0..=1.0).contains(&metrics.pattern_diversity.unwrap()));
        }

        #[test]
        fn test_reduction_percentage_bounded(
            loc_reduction in 0usize..1000,
            total_loc in 1usize..10000,
        ) {
            let violations = vec![create_test_violation(Severity::High, loc_reduction)];
            let report = create_test_report(violations, total_loc);

            let percentage = report.reduction_percentage();
            prop_assert!(percentage >= 0.0);
            // Percentage could be > 100% if reduction > total_loc (edge case)
        }

        #[test]
        fn test_pattern_collection_patterns_tracked(
            num_patterns in 1usize..20,
            frequency in 1usize..100,
        ) {
            let config = EntropyConfig::default();
            let calculator = EntropyCalculator::new(config);
            let mut patterns = PatternCollection::new();

            for i in 0..num_patterns {
                patterns.add_pattern(AstPattern {
                    pattern_type: PatternType::ErrorHandling,
                    pattern_hash: format!("hash{}", i),
                    frequency,
                    locations: vec![],
                    variation_score: 0.0,
                    example_code: "".to_string(),
                    estimated_loc: 5,
                });
            }

            let metrics = calculator.calculate(&patterns).unwrap();
            prop_assert_eq!(metrics.total_patterns, num_patterns);
            prop_assert_eq!(metrics.total_instances, num_patterns * frequency);
        }
    }

    // ============================================
    // Edge cases
    // ============================================

    #[test]
    fn test_entropy_calculator_zero_frequency_patterns() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "hash1".to_string(),
            frequency: 0, // Zero frequency
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 10,
        });

        let metrics = calculator.calculate(&patterns).unwrap();
        assert_eq!(metrics.total_instances, 0);
        assert_eq!(metrics.total_loc, 0);
    }

    #[test]
    fn test_entropy_calculator_large_pattern_collection() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        // Add many patterns
        for i in 0..100 {
            patterns.add_pattern(AstPattern {
                pattern_type: if i % 2 == 0 {
                    PatternType::ErrorHandling
                } else {
                    PatternType::ControlFlow
                },
                pattern_hash: format!("hash{}", i),
                frequency: (i % 10) + 1,
                locations: vec![],
                variation_score: (i as f64) / 100.0,
                example_code: format!("code{}", i),
                estimated_loc: (i % 20) + 1,
            });
        }

        let metrics = calculator.calculate(&patterns).unwrap();
        assert_eq!(metrics.total_patterns, 100);
        assert!(metrics.pattern_diversity.expect("measured") > 0.0);
    }

    #[test]
    fn test_entropy_report_format_empty_violations() {
        let report = create_test_report(vec![], 1000);
        let formatted = report.format_report();

        assert!(formatted.contains("Actionable Violations: 0"));
        assert!(formatted.contains("Total Potential Reduction: 0 lines"));
    }

    #[test]
    fn test_pattern_summary_debug() {
        let summary = PatternSummary {
            pattern_type: PatternType::DataTransformation,
            repetitions: 15,
            variation_score: 0.45,
            example_code: "transform(data)".to_string(),
        };

        let debug_str = format!("{:?}", summary);
        assert!(debug_str.contains("DataTransformation"));
        assert!(debug_str.contains("15"));
    }

    #[test]
    fn test_all_pattern_types_in_metrics() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        let all_types = [
            PatternType::ErrorHandling,
            PatternType::DataValidation,
            PatternType::ResourceManagement,
            PatternType::ControlFlow,
            PatternType::DataTransformation,
            PatternType::ApiCall,
        ];

        for (i, pt) in all_types.iter().enumerate() {
            patterns.add_pattern(AstPattern {
                pattern_type: *pt,
                pattern_hash: format!("hash{}", i),
                frequency: (i + 1) * 2,
                locations: vec![],
                variation_score: 0.0,
                example_code: "".to_string(),
                estimated_loc: 5,
            });
        }

        let metrics = calculator.calculate(&patterns).unwrap();

        assert_eq!(metrics.total_patterns, 6);
        assert_eq!(metrics.patterns_by_type.len(), 6);

        // Verify each pattern type is tracked
        for pt in &all_types {
            assert!(metrics.patterns_by_type.contains_key(pt));
        }
    }

    #[test]
    fn test_entropy_metrics_with_empty_patterns_by_type() {
        let metrics = EntropyMetrics {
            file_level_entropy: Some(0.5),
            module_level_entropy: Some(0.5),
            project_level_entropy: Some(0.5),
            pattern_diversity: Some(0.5),
            total_patterns: 0,
            total_instances: 0,
            total_loc: 0,
            patterns_by_type: BTreeMap::new(),
        };

        assert!(metrics.patterns_by_type.is_empty());
        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: EntropyMetrics = serde_json::from_str(&json).unwrap();
        assert!(deserialized.patterns_by_type.is_empty());
    }
}
