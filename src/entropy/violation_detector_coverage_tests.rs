#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::entropy::entropy_calculator::EntropyMetrics;
    use crate::entropy::pattern_extractor::{AstPattern, PatternCollection};
    use std::collections::BTreeMap;

    // Severity tests
    #[test]
    fn test_severity_partial_ord() {
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
        assert!(Severity::High >= Severity::High);
    }

    #[test]
    fn test_severity_clone_and_copy() {
        let s = Severity::High;
        let s2 = s; // Copy
        let s3 = s; // Clone
        assert_eq!(s, s2);
        assert_eq!(s, s3);
    }

    #[test]
    fn test_severity_serialization() {
        let s = Severity::Medium;
        let json = serde_json::to_string(&s).unwrap();
        let deserialized: Severity = serde_json::from_str(&json).unwrap();
        assert_eq!(s, deserialized);
    }

    // ActionableViolation tests
    #[test]
    fn test_actionable_violation_creation() {
        let violation = ActionableViolation {
            severity: Severity::High,
            pattern: Some(PatternSummary {
                pattern_type: PatternType::ErrorHandling,
                repetitions: 5,
                variation_score: 0.3,
                example_code: "match result {}".to_string(),
            }),
            message: "Test message".to_string(),
            fix_suggestion: "Fix it".to_string(),
            estimated_loc_reduction: Some(50),
            affected_files: vec![PathBuf::from("test.rs")],
            priority_score: 10.0,
        };
        assert_eq!(violation.severity, Severity::High);
        assert_eq!(violation.estimated_loc_reduction, Some(50));
    }

    #[test]
    fn test_actionable_violation_serialization() {
        let violation = ActionableViolation {
            severity: Severity::Low,
            pattern: Some(PatternSummary {
                pattern_type: PatternType::ControlFlow,
                repetitions: 3,
                variation_score: 0.5,
                example_code: "if else".to_string(),
            }),
            message: "msg".to_string(),
            fix_suggestion: "fix".to_string(),
            estimated_loc_reduction: Some(10),
            affected_files: vec![],
            priority_score: 5.0,
        };

        let json = serde_json::to_string(&violation).unwrap();
        let deserialized: ActionableViolation = serde_json::from_str(&json).unwrap();
        assert_eq!(violation.priority_score, deserialized.priority_score);
    }

    // PatternSummary tests
    #[test]
    fn test_pattern_summary_creation() {
        let summary = PatternSummary {
            pattern_type: PatternType::DataValidation,
            repetitions: 10,
            variation_score: 0.2,
            example_code: "validate()".to_string(),
        };
        assert_eq!(summary.repetitions, 10);
        assert_eq!(summary.variation_score, 0.2);
    }

    #[test]
    fn test_pattern_summary_clone() {
        let summary = PatternSummary {
            pattern_type: PatternType::ApiCall,
            repetitions: 7,
            variation_score: 0.8,
            example_code: "api.call()".to_string(),
        };
        let cloned = summary.clone();
        assert_eq!(summary.repetitions, cloned.repetitions);
        assert_eq!(summary.example_code, cloned.example_code);
    }

    // ViolationDetector tests
    #[test]
    fn test_violation_detector_creation() {
        let config = EntropyConfig::default();
        let detector = ViolationDetector::new(config);
        let _ = detector;
    }

    #[test]
    fn test_calculate_repetition_severity_low() {
        let detector = ViolationDetector::new(EntropyConfig::default());
        assert_eq!(detector.calculate_repetition_severity(1), Severity::Low);
        assert_eq!(detector.calculate_repetition_severity(5), Severity::Low);
    }

    #[test]
    fn test_calculate_repetition_severity_medium() {
        let detector = ViolationDetector::new(EntropyConfig::default());
        assert_eq!(detector.calculate_repetition_severity(6), Severity::Medium);
        assert_eq!(detector.calculate_repetition_severity(10), Severity::Medium);
    }

    #[test]
    fn test_calculate_repetition_severity_high() {
        let detector = ViolationDetector::new(EntropyConfig::default());
        assert_eq!(detector.calculate_repetition_severity(11), Severity::High);
        assert_eq!(detector.calculate_repetition_severity(100), Severity::High);
    }

    #[test]
    fn test_estimate_loc_reduction() {
        let detector = ViolationDetector::new(EntropyConfig::default());
        let pattern = AstPattern {
            pattern_type: PatternType::ControlFlow,
            pattern_hash: "test".to_string(),
            frequency: 5,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 10,
        };

        let reduction = detector.estimate_loc_reduction(&pattern);
        // CONTRACT UPDATED: `estimated_loc` is the LOC covered by all `frequency`
        // instances together (10 lines over 5 instances = 2 lines each), so
        // collapsing 4 of them saves 4 * 2 * 0.8 = 6 lines. The old expectation of
        // 32 treated the whole-group figure as a per-instance size and so
        // "saved" three times more lines than the pattern occupies.
        assert_eq!(reduction, 6);
    }

    #[test]
    fn test_estimate_loc_reduction_single_instance() {
        let detector = ViolationDetector::new(EntropyConfig::default());
        let pattern = AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "test".to_string(),
            frequency: 1,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 10,
        };

        let reduction = detector.estimate_loc_reduction(&pattern);
        assert_eq!(reduction, 0);
    }

    #[test]
    fn test_calculate_priority_high_severity() {
        let detector = ViolationDetector::new(EntropyConfig::default());
        let priority = detector.calculate_priority(Severity::High, 100);
        assert!(priority > 10.0);
    }

    #[test]
    fn test_calculate_priority_medium_severity() {
        let detector = ViolationDetector::new(EntropyConfig::default());
        let priority = detector.calculate_priority(Severity::Medium, 50);
        assert!(priority > 5.0);
    }

    #[test]
    fn test_calculate_priority_low_severity() {
        let detector = ViolationDetector::new(EntropyConfig::default());
        let priority = detector.calculate_priority(Severity::Low, 10);
        assert!(priority > 1.0);
    }

    #[test]
    fn test_suggest_module_name() {
        let detector = ViolationDetector::new(EntropyConfig::default());

        assert_eq!(
            detector.suggest_module_name(PatternType::ErrorHandling),
            "error_handler"
        );
        assert_eq!(
            detector.suggest_module_name(PatternType::DataValidation),
            "validators"
        );
        assert_eq!(
            detector.suggest_module_name(PatternType::ResourceManagement),
            "resource_guards"
        );
        assert_eq!(
            detector.suggest_module_name(PatternType::ControlFlow),
            "control_flow"
        );
        assert_eq!(
            detector.suggest_module_name(PatternType::DataTransformation),
            "transformers"
        );
        assert_eq!(
            detector.suggest_module_name(PatternType::ApiCall),
            "api_client"
        );
    }

    #[test]
    fn test_pattern_name() {
        let detector = ViolationDetector::new(EntropyConfig::default());

        assert_eq!(
            detector.pattern_name(PatternType::ErrorHandling),
            "error handling"
        );
        assert_eq!(
            detector.pattern_name(PatternType::DataValidation),
            "validation"
        );
        assert_eq!(
            detector.pattern_name(PatternType::ResourceManagement),
            "resource management"
        );
        assert_eq!(
            detector.pattern_name(PatternType::ControlFlow),
            "control flow"
        );
        assert_eq!(
            detector.pattern_name(PatternType::DataTransformation),
            "data transformation"
        );
        assert_eq!(detector.pattern_name(PatternType::ApiCall), "API call");
    }

    #[test]
    fn test_context_name() {
        let detector = ViolationDetector::new(EntropyConfig::default());
        let pattern = AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "test".to_string(),
            frequency: 1,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 10,
        };

        assert_eq!(detector.context_name(&pattern), "context");
    }

    #[test]
    fn test_generate_fix_suggestion_error_handling() {
        let detector = ViolationDetector::new(EntropyConfig::default());
        let pattern = AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "test".to_string(),
            frequency: 5,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 10,
        };

        let suggestion = detector.generate_fix_suggestion(&pattern);
        assert!(suggestion.contains("handle_"));
        assert!(suggestion.contains("_error"));
    }

    #[test]
    fn test_generate_fix_suggestion_data_validation() {
        let detector = ViolationDetector::new(EntropyConfig::default());
        let pattern = AstPattern {
            pattern_type: PatternType::DataValidation,
            pattern_hash: "test".to_string(),
            frequency: 5,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 10,
        };

        let suggestion = detector.generate_fix_suggestion(&pattern);
        assert!(suggestion.contains("validation"));
    }

    #[test]
    fn test_generate_fix_suggestion_resource_management() {
        let detector = ViolationDetector::new(EntropyConfig::default());
        let pattern = AstPattern {
            pattern_type: PatternType::ResourceManagement,
            pattern_hash: "test".to_string(),
            frequency: 5,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 10,
        };

        let suggestion = detector.generate_fix_suggestion(&pattern);
        assert!(suggestion.contains("RAII"));
    }

    #[test]
    fn test_generate_fix_suggestion_control_flow() {
        let detector = ViolationDetector::new(EntropyConfig::default());
        let pattern = AstPattern {
            pattern_type: PatternType::ControlFlow,
            pattern_hash: "test".to_string(),
            frequency: 5,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 10,
        };

        let suggestion = detector.generate_fix_suggestion(&pattern);
        assert!(suggestion.contains("strategy") || suggestion.contains("polymorphism"));
    }

    #[test]
    fn test_generate_fix_suggestion_data_transformation() {
        let detector = ViolationDetector::new(EntropyConfig::default());
        let pattern = AstPattern {
            pattern_type: PatternType::DataTransformation,
            pattern_hash: "test".to_string(),
            frequency: 5,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 10,
        };

        let suggestion = detector.generate_fix_suggestion(&pattern);
        assert!(suggestion.contains("pipeline") || suggestion.contains("transformation"));
    }

    #[test]
    fn test_generate_fix_suggestion_api_call() {
        let detector = ViolationDetector::new(EntropyConfig::default());
        let pattern = AstPattern {
            pattern_type: PatternType::ApiCall,
            pattern_hash: "test".to_string(),
            frequency: 5,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 10,
        };

        let suggestion = detector.generate_fix_suggestion(&pattern);
        assert!(suggestion.contains("API") || suggestion.contains("client"));
    }

    // Detection tests
    #[test]
    fn test_detect_violations_empty_collection() {
        let detector = ViolationDetector::new(EntropyConfig::default());
        let patterns = PatternCollection::new();
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

        let violations = detector.detect_violations(&patterns, &metrics).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_detect_repetitive_patterns() {
        let config = EntropyConfig {
            max_pattern_repetition: 3,
            ..EntropyConfig::default()
        };
        let detector = ViolationDetector::new(config);

        let mut patterns = PatternCollection::new();
        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "test".to_string(),
            frequency: 10, // More than threshold
            locations: vec![],
            variation_score: 0.1,
            example_code: "match result".to_string(),
            estimated_loc: 5,
        });

        let mut violations = Vec::new();
        detector
            .detect_repetitive_patterns(&patterns, &mut violations)
            .unwrap();

        assert!(!violations.is_empty());
    }

    #[test]
    fn test_detect_low_diversity() {
        let config = EntropyConfig {
            min_pattern_diversity: 0.8,
            ..EntropyConfig::default()
        };
        let detector = ViolationDetector::new(config);

        let patterns = PatternCollection::new();
        let metrics = EntropyMetrics {
            file_level_entropy: Some(0.5),
            module_level_entropy: Some(0.5),
            project_level_entropy: Some(0.5),
            pattern_diversity: Some(0.3), // Below threshold
            total_patterns: 10,
            total_instances: 100,
            total_loc: 1000,
            patterns_by_type: BTreeMap::new(),
        };

        let mut violations = Vec::new();
        detector
            .detect_low_diversity(&patterns, &metrics, &mut violations)
            .unwrap();

        assert!(!violations.is_empty());
        assert!(violations[0].message.contains("diversity"));
    }
}
