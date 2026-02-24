// Tests for QualityAssessment, Violation, and Severity types
// Included from baseline.rs — shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests_violations {
    use super::*;

    // ==================== QualityAssessment Tests ====================

    #[test]
    fn test_quality_assessment_is_passing_no_violations() {
        let assessment = QualityAssessment {
            violations: vec![],
            overall_health: 100.0,
            recommendation: "All good".to_string(),
        };

        assert!(assessment.is_passing());
    }

    #[test]
    fn test_quality_assessment_is_passing_with_warnings() {
        let assessment = QualityAssessment {
            violations: vec![Violation::ComplexityCreep {
                current: 15,
                baseline: 10,
                severity: Severity::Warning,
            }],
            overall_health: 85.0,
            recommendation: "Minor issues".to_string(),
        };

        assert!(assessment.is_passing());
    }

    #[test]
    fn test_quality_assessment_not_passing_with_errors() {
        let assessment = QualityAssessment {
            violations: vec![Violation::ComplexityRegression {
                current: 25,
                limit: 20,
                severity: Severity::Error,
            }],
            overall_health: 50.0,
            recommendation: "Critical issues".to_string(),
        };

        assert!(!assessment.is_passing());
    }

    #[test]
    fn test_quality_assessment_serialization() {
        let assessment = QualityAssessment {
            violations: vec![],
            overall_health: 95.0,
            recommendation: "Looking good".to_string(),
        };

        let serialized = serde_json::to_string(&assessment).unwrap();
        let deserialized: QualityAssessment = serde_json::from_str(&serialized).unwrap();

        assert_eq!(assessment.overall_health, deserialized.overall_health);
        assert_eq!(assessment.recommendation, deserialized.recommendation);
    }

    // ==================== Violation Tests ====================

    #[test]
    fn test_violation_severity_complexity_regression() {
        let violation = Violation::ComplexityRegression {
            current: 25,
            limit: 20,
            severity: Severity::Error,
        };

        assert_eq!(violation.severity(), &Severity::Error);
    }

    #[test]
    fn test_violation_severity_complexity_creep() {
        let violation = Violation::ComplexityCreep {
            current: 15,
            baseline: 10,
            severity: Severity::Warning,
        };

        assert_eq!(violation.severity(), &Severity::Warning);
    }

    #[test]
    fn test_violation_severity_quality_erosion() {
        let violation = Violation::QualityErosion {
            slope: 0.15,
            severity: Severity::Warning,
        };

        assert_eq!(violation.severity(), &Severity::Warning);
    }

    #[test]
    fn test_violation_severity_binary_size() {
        let violation = Violation::BinarySizeIncrease {
            current: 1_500_000,
            baseline: 1_000_000,
            increase_percent: 50.0,
            severity: Severity::Warning,
        };

        assert_eq!(violation.severity(), &Severity::Warning);
    }

    #[test]
    fn test_violation_severity_performance() {
        let violation = Violation::PerformanceRegression {
            metric: "init_time".to_string(),
            current: 20.0,
            baseline: 10.0,
            severity: Severity::Error,
        };

        assert_eq!(violation.severity(), &Severity::Error);
    }

    #[test]
    fn test_violation_description_complexity_regression() {
        let violation = Violation::ComplexityRegression {
            current: 25,
            limit: 20,
            severity: Severity::Error,
        };

        assert_eq!(
            violation.description(),
            "Complexity regression: 25 exceeds limit 20"
        );
    }

    #[test]
    fn test_violation_description_complexity_creep() {
        let violation = Violation::ComplexityCreep {
            current: 15,
            baseline: 10,
            severity: Severity::Warning,
        };

        assert_eq!(
            violation.description(),
            "Complexity creep: 15 exceeds baseline 10"
        );
    }

    #[test]
    fn test_violation_description_quality_erosion() {
        let violation = Violation::QualityErosion {
            slope: 0.15,
            severity: Severity::Warning,
        };

        assert_eq!(
            violation.description(),
            "Quality erosion detected with slope 0.15"
        );
    }

    #[test]
    fn test_violation_description_binary_size() {
        let violation = Violation::BinarySizeIncrease {
            current: 1_500_000,
            baseline: 1_000_000,
            increase_percent: 50.0,
            severity: Severity::Warning,
        };

        assert_eq!(violation.description(), "Binary size increased by 50.0%");
    }

    #[test]
    fn test_violation_description_performance() {
        let violation = Violation::PerformanceRegression {
            metric: "initialization".to_string(),
            current: 20.0,
            baseline: 10.0,
            severity: Severity::Error,
        };

        assert_eq!(
            violation.description(),
            "initialization regression: 20.0ms exceeds baseline 10.0ms"
        );
    }

    #[test]
    fn test_violation_serialization() {
        let violation = Violation::ComplexityRegression {
            current: 25,
            limit: 20,
            severity: Severity::Error,
        };

        let serialized = serde_json::to_string(&violation).unwrap();
        let deserialized: Violation = serde_json::from_str(&serialized).unwrap();

        assert_eq!(violation.severity(), deserialized.severity());
    }

    // ==================== Severity Tests ====================

    #[test]
    fn test_severity_equality() {
        assert_eq!(Severity::Info, Severity::Info);
        assert_eq!(Severity::Warning, Severity::Warning);
        assert_eq!(Severity::Error, Severity::Error);
    }

    #[test]
    fn test_severity_inequality() {
        assert_ne!(Severity::Info, Severity::Warning);
        assert_ne!(Severity::Warning, Severity::Error);
        assert_ne!(Severity::Info, Severity::Error);
    }

    #[test]
    fn test_severity_serialization() {
        let severity = Severity::Warning;

        let serialized = serde_json::to_string(&severity).unwrap();
        let deserialized: Severity = serde_json::from_str(&serialized).unwrap();

        assert_eq!(severity, deserialized);
    }

    #[test]
    fn test_severity_clone() {
        let severity = Severity::Error;
        let cloned = severity.clone();
        assert_eq!(severity, cloned);
    }
}
