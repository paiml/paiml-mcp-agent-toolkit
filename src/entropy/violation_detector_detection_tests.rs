#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod detection_tests {
    use super::*;
    use crate::entropy::entropy_calculator::EntropyMetrics;
    use crate::entropy::pattern_extractor::{AstPattern, Location, PatternCollection};
    use std::collections::BTreeMap;

    #[test]
    fn test_detect_cross_file_duplication() {
        let detector = ViolationDetector::new(EntropyConfig::default());

        let mut patterns = PatternCollection::new();
        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ControlFlow,
            pattern_hash: "crossfile".to_string(),
            frequency: 5,
            locations: vec![
                Location {
                    file: PathBuf::from("a.rs"),
                    line: 1,
                    column: 1,
                },
                Location {
                    file: PathBuf::from("b.rs"),
                    line: 2,
                    column: 1,
                },
                Location {
                    file: PathBuf::from("c.rs"),
                    line: 3,
                    column: 1,
                },
            ],
            variation_score: 0.1,
            example_code: "if else".to_string(),
            estimated_loc: 5,
        });

        let mut violations = Vec::new();
        detector
            .detect_cross_file_duplication(&patterns, &mut violations)
            .unwrap();

        assert!(!violations.is_empty());
        assert!(violations[0].message.contains("duplicated"));
    }

    #[test]
    fn test_detect_cross_file_many_files() {
        let detector = ViolationDetector::new(EntropyConfig::default());

        let mut patterns = PatternCollection::new();
        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::DataTransformation,
            pattern_hash: "manyfiles".to_string(),
            frequency: 10,
            locations: vec![
                Location {
                    file: PathBuf::from("a.rs"),
                    line: 1,
                    column: 1,
                },
                Location {
                    file: PathBuf::from("b.rs"),
                    line: 2,
                    column: 1,
                },
                Location {
                    file: PathBuf::from("c.rs"),
                    line: 3,
                    column: 1,
                },
                Location {
                    file: PathBuf::from("d.rs"),
                    line: 4,
                    column: 1,
                },
                Location {
                    file: PathBuf::from("e.rs"),
                    line: 5,
                    column: 1,
                },
                Location {
                    file: PathBuf::from("f.rs"),
                    line: 6,
                    column: 1,
                },
            ],
            variation_score: 0.1,
            example_code: "map filter".to_string(),
            estimated_loc: 3,
        });

        let mut violations = Vec::new();
        detector
            .detect_cross_file_duplication(&patterns, &mut violations)
            .unwrap();

        // Should be High severity for >5 files
        assert!(!violations.is_empty());
        assert_eq!(violations[0].severity, Severity::High);
    }

    #[test]
    fn test_detect_inconsistent_patterns() {
        let config = EntropyConfig {
            max_inconsistency_score: 0.5,
            ..EntropyConfig::default()
        };
        let detector = ViolationDetector::new(config);

        let mut patterns = PatternCollection::new();
        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ApiCall,
            pattern_hash: "inconsistent".to_string(),
            frequency: 5,
            locations: vec![Location {
                file: PathBuf::from("api.rs"),
                line: 1,
                column: 1,
            }],
            variation_score: 0.9, // High variation = inconsistent
            example_code: "client.call()".to_string(),
            estimated_loc: 4,
        });

        let mut violations = Vec::new();
        detector
            .detect_inconsistent_patterns(&patterns, &mut violations)
            .unwrap();

        assert!(!violations.is_empty());
        assert!(violations[0].message.contains("Inconsistent"));
    }

    #[test]
    fn test_deidentical_files() {
        let detector = ViolationDetector::new(EntropyConfig::default());

        let violations = vec![
            ActionableViolation {
                severity: Severity::Medium,
                pattern: Some(PatternSummary {
                    pattern_type: PatternType::ErrorHandling,
                    repetitions: 5,
                    variation_score: 0.1,
                    example_code: "code1".to_string(),
                }),
                message: "msg1".to_string(),
                fix_suggestion: "fix1".to_string(),
                estimated_loc_reduction: Some(10),
                affected_files: vec![],
                priority_score: 5.0,
            },
            ActionableViolation {
                severity: Severity::High,
                pattern: Some(PatternSummary {
                    pattern_type: PatternType::ErrorHandling,
                    repetitions: 5,
                    variation_score: 0.1,
                    example_code: "code1".to_string(), // Same length = duplicate
                }),
                message: "msg2".to_string(),
                fix_suggestion: "fix2".to_string(),
                estimated_loc_reduction: Some(20),
                affected_files: vec![],
                priority_score: 10.0, // Higher priority - should be kept
            },
        ];

        let deduped = detector.deidentical_files(violations);

        // Should keep only the higher priority one
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].priority_score, 10.0);
    }

    /// violation_detector_impl.rs:312-314 — "keep existing" arm fires when the
    /// newly-seen duplicate has priority_score <= existing. The prior test inserts
    /// low-then-high (always replaces); this inserts high-then-low (always keeps).
    #[test]
    fn test_deidentical_files_keeps_existing_higher_priority() {
        let detector = ViolationDetector::new(EntropyConfig::default());

        let violations = vec![
            ActionableViolation {
                severity: Severity::High,
                pattern: Some(PatternSummary {
                    pattern_type: PatternType::ErrorHandling,
                    repetitions: 5,
                    variation_score: 0.1,
                    example_code: "code1".to_string(),
                }),
                message: "higher_first".to_string(),
                fix_suggestion: "fix1".to_string(),
                estimated_loc_reduction: Some(20),
                affected_files: vec![],
                priority_score: 10.0, // Inserted first, must survive.
            },
            ActionableViolation {
                severity: Severity::Medium,
                pattern: Some(PatternSummary {
                    pattern_type: PatternType::ErrorHandling,
                    repetitions: 5,
                    variation_score: 0.1,
                    example_code: "code1".to_string(), // Same dedupe key.
                }),
                message: "lower_second".to_string(),
                fix_suggestion: "fix2".to_string(),
                estimated_loc_reduction: Some(10),
                affected_files: vec![],
                priority_score: 5.0, // Must be discarded via the keep-existing arm.
            },
        ];

        let deduped = detector.deidentical_files(violations);

        assert_eq!(deduped.len(), 1, "same key must collapse to one entry");
        assert_eq!(
            deduped[0].priority_score, 10.0,
            "keep-existing arm must preserve the first (higher-priority) insert"
        );
        assert_eq!(deduped[0].message, "higher_first");
    }

    #[test]
    fn test_violations_sorted_by_priority() {
        let config = EntropyConfig {
            max_pattern_repetition: 2,
            min_severity: Severity::Low,
            ..EntropyConfig::default()
        };
        let detector = ViolationDetector::new(config);

        let mut patterns = PatternCollection::new();

        // Low frequency pattern
        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ControlFlow,
            pattern_hash: "low".to_string(),
            frequency: 3,
            locations: vec![],
            variation_score: 0.0,
            example_code: "low".to_string(),
            estimated_loc: 5,
        });

        // High frequency pattern
        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "high".to_string(),
            frequency: 15,
            locations: vec![],
            variation_score: 0.0,
            example_code: "high".to_string(),
            estimated_loc: 10,
        });

        let metrics = EntropyMetrics {
            file_level_entropy: Some(0.8),
            module_level_entropy: Some(0.8),
            project_level_entropy: Some(0.8),
            pattern_diversity: Some(0.8),
            total_patterns: 2,
            total_instances: 18,
            total_loc: 100,
            patterns_by_type: BTreeMap::new(),
        };

        let violations = detector.detect_violations(&patterns, &metrics).unwrap();

        // Verify sorted by priority (highest first)
        if violations.len() >= 2 {
            assert!(violations[0].priority_score >= violations[1].priority_score);
        }
    }

    #[test]
    fn test_severity_filter() {
        let config = EntropyConfig {
            max_pattern_repetition: 2,
            min_severity: Severity::High,
            ..EntropyConfig::default()
        };
        let detector = ViolationDetector::new(config);

        let mut patterns = PatternCollection::new();
        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ControlFlow,
            pattern_hash: "test".to_string(),
            frequency: 4, // Would be Low severity
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 5,
        });

        let metrics = EntropyMetrics {
            file_level_entropy: Some(0.8),
            module_level_entropy: Some(0.8),
            project_level_entropy: Some(0.8),
            pattern_diversity: Some(0.8),
            total_patterns: 1,
            total_instances: 4,
            total_loc: 20,
            patterns_by_type: BTreeMap::new(),
        };

        let violations = detector.detect_violations(&patterns, &metrics).unwrap();

        // Low severity violations should be filtered out
        for v in &violations {
            assert!(v.severity >= Severity::High);
        }
    }
}
