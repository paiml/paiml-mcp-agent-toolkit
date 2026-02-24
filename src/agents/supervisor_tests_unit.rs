use super::*;
use crate::modules::analyzer::Metrics;
use crate::modules::validator::{Severity, Thresholds, Violation};

// ========================================
// ValidateCode Message Tests
// ========================================

#[test]
fn test_validate_code_message_creation() {
    let msg = ValidateCode {
        code: "fn main() {}".to_string(),
        thresholds: Thresholds::default(),
    };
    assert_eq!(msg.code, "fn main() {}");
    assert_eq!(msg.thresholds.max_complexity, 10);
}

#[test]
fn test_validate_code_with_custom_thresholds() {
    let custom_thresholds = Thresholds {
        max_complexity: 5,
        max_functions: 10,
        max_lines: 100,
        min_test_coverage: 0.95,
    };
    let msg = ValidateCode {
        code: "fn test() {}".to_string(),
        thresholds: custom_thresholds,
    };
    assert_eq!(msg.thresholds.max_complexity, 5);
    assert_eq!(msg.thresholds.max_functions, 10);
    assert_eq!(msg.thresholds.max_lines, 100);
    assert!((msg.thresholds.min_test_coverage - 0.95).abs() < f64::EPSILON);
}

#[test]
fn test_validate_code_empty_code() {
    let msg = ValidateCode {
        code: String::new(),
        thresholds: Thresholds::default(),
    };
    assert!(msg.code.is_empty());
}

#[test]
fn test_validate_code_large_code() {
    let large_code = format!("fn main() {{\n{}}}", "    let x = 1;\n".repeat(1000));
    let msg = ValidateCode {
        code: large_code.clone(),
        thresholds: Thresholds::default(),
    };
    assert!(msg.code.len() > 10000);
}

#[test]
fn test_validate_code_unicode() {
    let msg = ValidateCode {
        code: "fn unicode_test() { let emoji = '\u{1F600}'; }".to_string(),
        thresholds: Thresholds::default(),
    };
    assert!(msg.code.contains('\u{1F600}'));
}

#[test]
fn test_validate_code_multiline() {
    let code = r#"
fn main() {
    let x = 1;
    let y = 2;
    println!("{} + {} = {}", x, y, x + y);
}
"#;
    let msg = ValidateCode {
        code: code.to_string(),
        thresholds: Thresholds::default(),
    };
    assert!(msg.code.contains('\n'));
    assert!(msg.code.lines().count() > 5);
}

// ========================================
// ValidationResult Tests
// ========================================

#[test]
fn test_validation_result_passed() {
    let result = ValidationResult {
        passed: true,
        metrics: Metrics {
            complexity: 5,
            lines_of_code: 100,
            functions: 10,
            classes: 2,
            imports: 5,
        },
        validation: crate::modules::validator::ValidationResult {
            passed: true,
            violations: vec![],
            score: 100.0,
        },
    };
    assert!(result.passed);
    assert_eq!(result.metrics.complexity, 5);
    assert_eq!(result.metrics.lines_of_code, 100);
    assert_eq!(result.metrics.functions, 10);
    assert_eq!(result.metrics.classes, 2);
    assert_eq!(result.metrics.imports, 5);
    assert!(result.validation.violations.is_empty());
    assert_eq!(result.validation.score, 100.0);
}

#[test]
fn test_validation_result_failed_with_violations() {
    let result = ValidationResult {
        passed: false,
        metrics: Metrics {
            complexity: 25,
            lines_of_code: 1000,
            functions: 100,
            classes: 20,
            imports: 50,
        },
        validation: crate::modules::validator::ValidationResult {
            passed: false,
            violations: vec![
                Violation {
                    rule: "complexity".to_string(),
                    severity: Severity::Error,
                    message: "Too complex".to_string(),
                    location: None,
                },
                Violation {
                    rule: "functions".to_string(),
                    severity: Severity::Warning,
                    message: "Too many functions".to_string(),
                    location: Some("module.rs:1".to_string()),
                },
            ],
            score: 50.0,
        },
    };
    assert!(!result.passed);
    assert_eq!(result.metrics.complexity, 25);
    assert_eq!(result.validation.violations.len(), 2);
    assert_eq!(result.validation.violations[0].rule, "complexity");
    assert_eq!(result.validation.violations[0].severity, Severity::Error);
    assert_eq!(result.validation.violations[1].rule, "functions");
    assert_eq!(result.validation.violations[1].severity, Severity::Warning);
    assert!(result.validation.violations[1].location.is_some());
}

#[test]
fn test_validation_result_with_zero_metrics() {
    let result = ValidationResult {
        passed: true,
        metrics: Metrics {
            complexity: 0,
            lines_of_code: 0,
            functions: 0,
            classes: 0,
            imports: 0,
        },
        validation: crate::modules::validator::ValidationResult {
            passed: true,
            violations: vec![],
            score: 100.0,
        },
    };
    assert!(result.passed);
    assert_eq!(result.metrics.complexity, 0);
    assert_eq!(result.metrics.lines_of_code, 0);
}

#[test]
fn test_validation_result_with_all_severity_levels() {
    let result = ValidationResult {
        passed: false,
        metrics: Metrics {
            complexity: 15,
            lines_of_code: 600,
            functions: 60,
            classes: 10,
            imports: 30,
        },
        validation: crate::modules::validator::ValidationResult {
            passed: false,
            violations: vec![
                Violation {
                    rule: "error_rule".to_string(),
                    severity: Severity::Error,
                    message: "Error level violation".to_string(),
                    location: None,
                },
                Violation {
                    rule: "warning_rule".to_string(),
                    severity: Severity::Warning,
                    message: "Warning level violation".to_string(),
                    location: None,
                },
                Violation {
                    rule: "info_rule".to_string(),
                    severity: Severity::Info,
                    message: "Info level violation".to_string(),
                    location: None,
                },
            ],
            score: 70.0,
        },
    };
    assert!(!result.passed);
    assert_eq!(result.validation.violations.len(), 3);
    assert!(result
        .validation
        .violations
        .iter()
        .any(|v| v.severity == Severity::Error));
    assert!(result
        .validation
        .violations
        .iter()
        .any(|v| v.severity == Severity::Warning));
    assert!(result
        .validation
        .violations
        .iter()
        .any(|v| v.severity == Severity::Info));
}

// ========================================
// Thresholds Tests (imported from validator)
// ========================================

#[test]
fn test_thresholds_default() {
    let thresholds = Thresholds::default();
    assert_eq!(thresholds.max_complexity, 10);
    assert_eq!(thresholds.max_functions, 50);
    assert_eq!(thresholds.max_lines, 500);
    assert!((thresholds.min_test_coverage - 0.8).abs() < f64::EPSILON);
}

#[test]
fn test_thresholds_custom() {
    let thresholds = Thresholds {
        max_complexity: 20,
        max_functions: 100,
        max_lines: 1000,
        min_test_coverage: 0.9,
    };
    assert_eq!(thresholds.max_complexity, 20);
    assert_eq!(thresholds.max_functions, 100);
    assert_eq!(thresholds.max_lines, 1000);
    assert!((thresholds.min_test_coverage - 0.9).abs() < f64::EPSILON);
}

#[test]
fn test_thresholds_zero_values() {
    let thresholds = Thresholds {
        max_complexity: 0,
        max_functions: 0,
        max_lines: 0,
        min_test_coverage: 0.0,
    };
    assert_eq!(thresholds.max_complexity, 0);
    assert_eq!(thresholds.max_functions, 0);
    assert_eq!(thresholds.max_lines, 0);
    assert!((thresholds.min_test_coverage - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_thresholds_max_values() {
    let thresholds = Thresholds {
        max_complexity: u32::MAX,
        max_functions: usize::MAX,
        max_lines: usize::MAX,
        min_test_coverage: 1.0,
    };
    assert_eq!(thresholds.max_complexity, u32::MAX);
    assert_eq!(thresholds.max_functions, usize::MAX);
    assert_eq!(thresholds.max_lines, usize::MAX);
    assert!((thresholds.min_test_coverage - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_thresholds_serialization_roundtrip() {
    let thresholds = Thresholds {
        max_complexity: 15,
        max_functions: 30,
        max_lines: 300,
        min_test_coverage: 0.85,
    };
    let serialized = serde_json::to_string(&thresholds).expect("serialization should succeed");
    let deserialized: Thresholds =
        serde_json::from_str(&serialized).expect("deserialization should succeed");
    assert_eq!(thresholds.max_complexity, deserialized.max_complexity);
    assert_eq!(thresholds.max_functions, deserialized.max_functions);
    assert_eq!(thresholds.max_lines, deserialized.max_lines);
    assert!(
        (thresholds.min_test_coverage - deserialized.min_test_coverage).abs() < f64::EPSILON
    );
}

#[test]
fn test_thresholds_clone() {
    let thresholds = Thresholds {
        max_complexity: 25,
        max_functions: 75,
        max_lines: 750,
        min_test_coverage: 0.7,
    };
    let cloned = thresholds.clone();
    assert_eq!(thresholds.max_complexity, cloned.max_complexity);
    assert_eq!(thresholds.max_functions, cloned.max_functions);
    assert_eq!(thresholds.max_lines, cloned.max_lines);
    assert!((thresholds.min_test_coverage - cloned.min_test_coverage).abs() < f64::EPSILON);
}

// ========================================
// Metrics Tests
// ========================================

#[test]
fn test_metrics_clone() {
    let metrics = Metrics {
        complexity: 10,
        lines_of_code: 200,
        functions: 5,
        classes: 3,
        imports: 8,
    };
    let cloned = metrics.clone();
    assert_eq!(metrics.complexity, cloned.complexity);
    assert_eq!(metrics.lines_of_code, cloned.lines_of_code);
    assert_eq!(metrics.functions, cloned.functions);
    assert_eq!(metrics.classes, cloned.classes);
    assert_eq!(metrics.imports, cloned.imports);
}

#[test]
fn test_metrics_serialization_roundtrip() {
    let metrics = Metrics {
        complexity: 12,
        lines_of_code: 350,
        functions: 20,
        classes: 5,
        imports: 15,
    };
    let serialized = serde_json::to_string(&metrics).expect("serialization should succeed");
    let deserialized: Metrics =
        serde_json::from_str(&serialized).expect("deserialization should succeed");
    assert_eq!(metrics.complexity, deserialized.complexity);
    assert_eq!(metrics.lines_of_code, deserialized.lines_of_code);
    assert_eq!(metrics.functions, deserialized.functions);
    assert_eq!(metrics.classes, deserialized.classes);
    assert_eq!(metrics.imports, deserialized.imports);
}

// ========================================
// AgentError Verification Tests
// ========================================

#[test]
fn test_agent_error_communication_failed() {
    let error = AgentError::CommunicationFailed("timeout".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("communication"));
    assert!(error_string.contains("timeout"));
}

#[test]
fn test_agent_error_processing_failed() {
    let error = AgentError::ProcessingFailed("syntax error".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("processing"));
    assert!(error_string.contains("syntax error"));
}

// ========================================
// Violation Tests
// ========================================

#[test]
fn test_violation_with_location() {
    let violation = Violation {
        rule: "complexity".to_string(),
        severity: Severity::Error,
        message: "Function too complex".to_string(),
        location: Some("src/lib.rs:42".to_string()),
    };
    assert_eq!(violation.rule, "complexity");
    assert_eq!(violation.severity, Severity::Error);
    assert!(violation.location.is_some());
    assert_eq!(violation.location.as_ref().unwrap(), "src/lib.rs:42");
}

#[test]
fn test_violation_without_location() {
    let violation = Violation {
        rule: "lines".to_string(),
        severity: Severity::Info,
        message: "File has many lines".to_string(),
        location: None,
    };
    assert_eq!(violation.rule, "lines");
    assert_eq!(violation.severity, Severity::Info);
    assert!(violation.location.is_none());
}

#[test]
fn test_severity_equality() {
    assert_eq!(Severity::Error, Severity::Error);
    assert_eq!(Severity::Warning, Severity::Warning);
    assert_eq!(Severity::Info, Severity::Info);
    assert_ne!(Severity::Error, Severity::Warning);
    assert_ne!(Severity::Warning, Severity::Info);
}
