// Unit tests for quality-related proxy types: ViolationType, ViolationSeverity,
// QualityViolation, QualityMetrics, QualityReport, ProxyResponse,
// plus Clone and Debug trait verification tests.

use super::*;

// === ViolationType Tests ===

#[test]
fn test_violation_type_complexity() {
    let vt = ViolationType::Complexity;
    let json = serde_json::to_string(&vt).unwrap();
    assert_eq!(json, "\"complexity\"");
}

#[test]
fn test_violation_type_satd() {
    let vt = ViolationType::Satd;
    let json = serde_json::to_string(&vt).unwrap();
    assert_eq!(json, "\"satd\"");
}

#[test]
fn test_violation_type_lint() {
    let vt = ViolationType::Lint;
    let json = serde_json::to_string(&vt).unwrap();
    assert_eq!(json, "\"lint\"");
}

#[test]
fn test_violation_type_docs() {
    let vt = ViolationType::Docs;
    let json = serde_json::to_string(&vt).unwrap();
    assert_eq!(json, "\"docs\"");
}

// === ViolationSeverity Tests ===

#[test]
fn test_violation_severity_error() {
    let sev = ViolationSeverity::Error;
    let json = serde_json::to_string(&sev).unwrap();
    assert_eq!(json, "\"error\"");
}

#[test]
fn test_violation_severity_warning() {
    let sev = ViolationSeverity::Warning;
    let json = serde_json::to_string(&sev).unwrap();
    assert_eq!(json, "\"warning\"");
}

// === QualityViolation Tests ===

#[test]
fn test_quality_violation_creation() {
    let violation = QualityViolation {
        violation_type: ViolationType::Complexity,
        severity: ViolationSeverity::Error,
        location: "src/main.rs:42".to_string(),
        message: "Cyclomatic complexity of 25 exceeds limit of 20".to_string(),
        suggestion: Some("Extract helper functions".to_string()),
    };

    assert!(matches!(
        violation.violation_type,
        ViolationType::Complexity
    ));
    assert!(matches!(violation.severity, ViolationSeverity::Error));
    assert_eq!(violation.location, "src/main.rs:42");
    assert!(violation.message.contains("complexity"));
}

#[test]
fn test_quality_violation_without_suggestion() {
    let violation = QualityViolation {
        violation_type: ViolationType::Satd,
        severity: ViolationSeverity::Warning,
        location: "src/lib.rs:10".to_string(),
        message: "TODO marker found".to_string(),
        suggestion: None,
    };

    assert!(violation.suggestion.is_none());
}

#[test]
fn test_quality_violation_serialization() {
    let violation = QualityViolation {
        violation_type: ViolationType::Lint,
        severity: ViolationSeverity::Warning,
        location: "test.rs:5".to_string(),
        message: "Unused variable".to_string(),
        suggestion: None,
    };

    let json = serde_json::to_string(&violation).unwrap();
    assert!(json.contains("\"type\":\"lint\""));
    assert!(json.contains("\"severity\":\"warning\""));
    assert!(!json.contains("suggestion"));
}

// === QualityMetrics Tests ===

#[test]
fn test_quality_metrics_creation() {
    let metrics = QualityMetrics {
        max_complexity: 25,
        satd_count: 3,
        lint_violations: 5,
        coverage_percentage: Some(85.5),
    };

    assert_eq!(metrics.max_complexity, 25);
    assert_eq!(metrics.satd_count, 3);
    assert_eq!(metrics.lint_violations, 5);
    assert_eq!(metrics.coverage_percentage, Some(85.5));
}

#[test]
fn test_quality_metrics_no_coverage() {
    let metrics = QualityMetrics {
        max_complexity: 10,
        satd_count: 0,
        lint_violations: 0,
        coverage_percentage: None,
    };

    assert!(metrics.coverage_percentage.is_none());
}

#[test]
fn test_quality_metrics_serialization() {
    let metrics = QualityMetrics {
        max_complexity: 15,
        satd_count: 2,
        lint_violations: 1,
        coverage_percentage: None,
    };

    let json = serde_json::to_string(&metrics).unwrap();
    assert!(!json.contains("coverage_percentage"));
}

// === QualityReport Tests ===

#[test]
fn test_quality_report_passed() {
    let report = QualityReport {
        passed: true,
        metrics: QualityMetrics {
            max_complexity: 10,
            satd_count: 0,
            lint_violations: 0,
            coverage_percentage: Some(95.0),
        },
        violations: vec![],
        language: "rust".to_string(),
        gates_run: vec![
            "satd".to_string(),
            "complexity".to_string(),
            "lint".to_string(),
        ],
    };

    assert!(report.passed);
    assert!(report.violations.is_empty());
    assert_eq!(report.gates_run.len(), 3);
}

#[test]
fn test_quality_report_failed() {
    let report = QualityReport {
        passed: false,
        metrics: QualityMetrics {
            max_complexity: 30,
            satd_count: 5,
            lint_violations: 3,
            coverage_percentage: Some(60.0),
        },
        violations: vec![QualityViolation {
            violation_type: ViolationType::Complexity,
            severity: ViolationSeverity::Error,
            location: "test.rs:10".to_string(),
            message: "High complexity".to_string(),
            suggestion: None,
        }],
        language: "rust".to_string(),
        gates_run: vec!["satd".to_string(), "complexity".to_string()],
    };

    assert!(!report.passed);
    assert_eq!(report.violations.len(), 1);
}

// === ProxyResponse Tests ===

#[test]
fn test_proxy_response_accepted() {
    let response = ProxyResponse {
        status: ProxyStatus::Accepted,
        quality_report: QualityReport {
            passed: true,
            metrics: QualityMetrics {
                max_complexity: 10,
                satd_count: 0,
                lint_violations: 0,
                coverage_percentage: None,
            },
            violations: vec![],
            language: "rust".to_string(),
            gates_run: vec!["satd".to_string(), "complexity".to_string()],
        },
        final_content: "fn test() {}".to_string(),
        refactoring_applied: false,
        refactoring_plan: None,
    };

    assert!(matches!(response.status, ProxyStatus::Accepted));
    assert!(!response.refactoring_applied);
    assert!(response.refactoring_plan.is_none());
}

#[test]
fn test_proxy_response_modified_with_refactoring() {
    let mut plan_step = HashMap::new();
    plan_step.insert("action".to_string(), serde_json::json!("extract_function"));
    plan_step.insert("lines".to_string(), serde_json::json!([10, 20]));

    let response = ProxyResponse {
        status: ProxyStatus::Modified,
        quality_report: QualityReport {
            passed: true,
            metrics: QualityMetrics {
                max_complexity: 15,
                satd_count: 0,
                lint_violations: 0,
                coverage_percentage: None,
            },
            violations: vec![],
            language: "rust".to_string(),
            gates_run: vec!["satd".to_string(), "complexity".to_string()],
        },
        final_content: "fn helper() {}\nfn test() { helper(); }".to_string(),
        refactoring_applied: true,
        refactoring_plan: Some(vec![plan_step]),
    };

    assert!(matches!(response.status, ProxyStatus::Modified));
    assert!(response.refactoring_applied);
    assert!(response.refactoring_plan.is_some());
    assert_eq!(response.refactoring_plan.as_ref().unwrap().len(), 1);
}

#[test]
fn test_proxy_response_rejected() {
    let response = ProxyResponse {
        status: ProxyStatus::Rejected,
        quality_report: QualityReport {
            passed: false,
            metrics: QualityMetrics {
                max_complexity: 50,
                satd_count: 10,
                lint_violations: 5,
                coverage_percentage: Some(30.0),
            },
            violations: vec![QualityViolation {
                violation_type: ViolationType::Complexity,
                severity: ViolationSeverity::Error,
                location: "main.rs:1".to_string(),
                message: "Exceeds limits".to_string(),
                suggestion: Some("Refactor".to_string()),
            }],
            language: "rust".to_string(),
            gates_run: vec!["satd".to_string(), "complexity".to_string()],
        },
        final_content: String::new(),
        refactoring_applied: false,
        refactoring_plan: None,
    };

    assert!(matches!(response.status, ProxyStatus::Rejected));
    assert!(!response.quality_report.passed);
    assert_eq!(response.quality_report.violations.len(), 1);
}

#[test]
fn test_proxy_response_serialization_roundtrip() {
    let response = ProxyResponse {
        status: ProxyStatus::Accepted,
        quality_report: QualityReport {
            passed: true,
            metrics: QualityMetrics {
                max_complexity: 10,
                satd_count: 0,
                lint_violations: 0,
                coverage_percentage: Some(90.0),
            },
            violations: vec![],
            language: "rust".to_string(),
            gates_run: vec![
                "satd".to_string(),
                "complexity".to_string(),
                "lint".to_string(),
                "docs".to_string(),
            ],
        },
        final_content: "fn foo() {}".to_string(),
        refactoring_applied: false,
        refactoring_plan: None,
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: ProxyResponse = serde_json::from_str(&json).unwrap();

    assert!(matches!(deserialized.status, ProxyStatus::Accepted));
    assert!(deserialized.quality_report.passed);
    assert_eq!(deserialized.final_content, "fn foo() {}");
    assert_eq!(deserialized.quality_report.language, "rust");
    assert_eq!(deserialized.quality_report.gates_run.len(), 4);
}

// === Disclosure fields (language / gates_run) ===

/// A gate absent from `gates_run` did NOT run, and the zero beside it is not a
/// measurement. This is the whole contract the two fields carry, and it is the
/// reason a report can no longer say `passed: true` with nothing behind it.
#[test]
fn test_gates_run_distinguishes_measured_zero_from_unmeasured_zero() {
    let judged = QualityReport {
        passed: true,
        metrics: QualityMetrics {
            max_complexity: 0,
            satd_count: 0,
            lint_violations: 0,
            coverage_percentage: None,
        },
        violations: vec![],
        language: "rust".to_string(),
        gates_run: vec![
            "satd".to_string(),
            "complexity".to_string(),
            "lint".to_string(),
        ],
    };

    // Same metrics, same `passed`, entirely different meaning: nothing but the
    // comment scan ever looked at this content.
    let barely_looked_at = QualityReport {
        language: "unknown".to_string(),
        gates_run: vec!["satd".to_string()],
        ..judged.clone()
    };

    assert_eq!(judged.metrics.lint_violations, 0);
    assert_eq!(barely_looked_at.metrics.lint_violations, 0);
    assert!(judged.gates_run.iter().any(|g| g == "lint"));
    assert!(!barely_looked_at.gates_run.iter().any(|g| g == "lint"));
}

/// Both fields are `#[serde(default)]`, so a payload recorded before they
/// existed still deserializes — as an empty `gates_run`, which is the honest
/// reading of a report that claims nothing.
#[test]
fn test_report_without_disclosure_fields_still_deserializes() {
    let legacy = r#"{
        "passed": true,
        "metrics": {"max_complexity": 0, "satd_count": 0, "lint_violations": 0},
        "violations": []
    }"#;

    let report: QualityReport =
        serde_json::from_str(legacy).expect("old payloads must keep deserializing");

    assert!(report.passed);
    assert_eq!(report.language, "");
    assert!(
        report.gates_run.is_empty(),
        "a report from before the field existed claims no gate at all"
    );
}

// === Clone Tests ===

#[test]
fn test_proxy_operation_clone() {
    let op = ProxyOperation::Write;
    let cloned = op.clone();
    assert!(matches!(cloned, ProxyOperation::Write));
}

#[test]
fn test_quality_config_clone() {
    let config = QualityConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.max_complexity, config.max_complexity);
}

#[test]
fn test_quality_violation_clone() {
    let violation = QualityViolation {
        violation_type: ViolationType::Satd,
        severity: ViolationSeverity::Warning,
        location: "test.rs:1".to_string(),
        message: "TODO".to_string(),
        suggestion: None,
    };
    let cloned = violation.clone();
    assert_eq!(cloned.location, violation.location);
}

// === Debug Tests ===

#[test]
fn test_proxy_operation_debug() {
    let op = ProxyOperation::Edit;
    let debug = format!("{:?}", op);
    assert!(debug.contains("Edit"));
}

#[test]
fn test_proxy_mode_debug() {
    let mode = ProxyMode::Advisory;
    let debug = format!("{:?}", mode);
    assert!(debug.contains("Advisory"));
}

#[test]
fn test_quality_metrics_debug() {
    let metrics = QualityMetrics {
        max_complexity: 20,
        satd_count: 1,
        lint_violations: 2,
        coverage_percentage: Some(80.0),
    };
    let debug = format!("{:?}", metrics);
    assert!(debug.contains("max_complexity: 20"));
}
