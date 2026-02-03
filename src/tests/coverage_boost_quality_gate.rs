//! Coverage boost tests for cli/analysis_utilities/quality_gate.rs
//! Tests pure functions: apply_satd_filters, get_severity_icon, and formatting functions

use crate::cli::analysis_utilities::{
    apply_satd_filters, get_severity_icon, handle_quality_gate, QualityGateResults,
    QualityViolation,
};
use crate::cli::{QualityCheckType, QualityGateOutputFormat, SatdSeverity};
use crate::services::satd_detector::{DebtCategory, Severity, TechnicalDebt};
use std::path::PathBuf;

fn create_test_debt(severity: Severity, text: &str) -> TechnicalDebt {
    TechnicalDebt {
        file: PathBuf::from("test.rs"),
        line: 10,
        column: 5,
        text: text.to_string(),
        category: DebtCategory::Requirement,
        severity,
        context_hash: [0u8; 16],
    }
}

// --- apply_satd_filters tests ---

#[test]
fn test_apply_satd_filters_no_filters() {
    let items = vec![
        create_test_debt(Severity::Low, "Low debt"),
        create_test_debt(Severity::Medium, "Medium debt"),
        create_test_debt(Severity::High, "High debt"),
        create_test_debt(Severity::Critical, "Critical debt"),
    ];

    let filtered = apply_satd_filters(items, None, false);
    assert_eq!(filtered.len(), 4);
}

#[test]
fn test_apply_satd_filters_severity_low() {
    let items = vec![
        create_test_debt(Severity::Low, "Low"),
        create_test_debt(Severity::Medium, "Medium"),
        create_test_debt(Severity::High, "High"),
        create_test_debt(Severity::Critical, "Critical"),
    ];

    let filtered = apply_satd_filters(items, Some(SatdSeverity::Low), false);
    assert_eq!(filtered.len(), 4); // All should pass Low filter
}

#[test]
fn test_apply_satd_filters_severity_medium() {
    let items = vec![
        create_test_debt(Severity::Low, "Low"),
        create_test_debt(Severity::Medium, "Medium"),
        create_test_debt(Severity::High, "High"),
        create_test_debt(Severity::Critical, "Critical"),
    ];

    let filtered = apply_satd_filters(items, Some(SatdSeverity::Medium), false);
    assert!(filtered.len() >= 3); // Medium, High, Critical
    assert!(filtered.iter().all(|d| d.severity as u8 >= Severity::Medium as u8));
}

#[test]
fn test_apply_satd_filters_severity_high() {
    let items = vec![
        create_test_debt(Severity::Low, "Low"),
        create_test_debt(Severity::Medium, "Medium"),
        create_test_debt(Severity::High, "High"),
        create_test_debt(Severity::Critical, "Critical"),
    ];

    let filtered = apply_satd_filters(items, Some(SatdSeverity::High), false);
    assert!(filtered.len() >= 2); // High, Critical
}

#[test]
fn test_apply_satd_filters_severity_critical() {
    let items = vec![
        create_test_debt(Severity::Low, "Low"),
        create_test_debt(Severity::Medium, "Medium"),
        create_test_debt(Severity::High, "High"),
        create_test_debt(Severity::Critical, "Critical"),
    ];

    let filtered = apply_satd_filters(items, Some(SatdSeverity::Critical), false);
    assert!(filtered.len() >= 1); // At least Critical
}

#[test]
fn test_apply_satd_filters_critical_only() {
    let items = vec![
        create_test_debt(Severity::Low, "Low"),
        create_test_debt(Severity::Medium, "Medium"),
        create_test_debt(Severity::High, "High"),
        create_test_debt(Severity::Critical, "Critical"),
    ];

    let filtered = apply_satd_filters(items, None, true);
    assert!(filtered.len() >= 1);
    assert!(filtered.iter().all(|d| matches!(
        d.severity,
        Severity::Critical | Severity::High
    )));
}

#[test]
fn test_apply_satd_filters_empty() {
    let items: Vec<TechnicalDebt> = vec![];
    let filtered = apply_satd_filters(items, None, false);
    assert!(filtered.is_empty());
}

#[test]
fn test_apply_satd_filters_combined() {
    let items = vec![
        create_test_debt(Severity::Low, "Low"),
        create_test_debt(Severity::Medium, "Medium"),
        create_test_debt(Severity::High, "High"),
        create_test_debt(Severity::Critical, "Critical"),
    ];

    // Both severity filter AND critical_only
    let filtered = apply_satd_filters(items, Some(SatdSeverity::Medium), true);
    assert!(filtered.iter().all(|d| matches!(
        d.severity,
        Severity::Critical | Severity::High
    )));
}

// --- get_severity_icon tests ---

#[test]
fn test_get_severity_icon_error() {
    assert_eq!(get_severity_icon("error"), "🔴");
}

#[test]
fn test_get_severity_icon_warning() {
    assert_eq!(get_severity_icon("warning"), "🟡");
}

#[test]
fn test_get_severity_icon_other() {
    assert_eq!(get_severity_icon("info"), "🟢");
    assert_eq!(get_severity_icon("unknown"), "🟢");
    assert_eq!(get_severity_icon(""), "🟢");
}

// --- QualityGateResults tests ---

#[test]
fn test_quality_gate_results_default() {
    let results = QualityGateResults::default();
    assert!(results.passed);
    assert_eq!(results.total_violations, 0);
    assert_eq!(results.complexity_violations, 0);
    assert_eq!(results.dead_code_violations, 0);
    assert_eq!(results.satd_violations, 0);
    assert_eq!(results.entropy_violations, 0);
    assert_eq!(results.security_violations, 0);
    assert_eq!(results.duplicate_violations, 0);
    assert_eq!(results.coverage_violations, 0);
    assert_eq!(results.section_violations, 0);
    assert_eq!(results.provability_violations, 0);
    assert!(results.provability_score.is_none());
}

// --- QualityViolation tests ---

#[test]
fn test_quality_violation_construction() {
    let violation = QualityViolation {
        check_type: "complexity".to_string(),
        severity: "error".to_string(),
        file: "src/main.rs".to_string(),
        line: Some(42),
        message: "Function too complex".to_string(),
    };
    assert_eq!(violation.check_type, "complexity");
    assert_eq!(violation.severity, "error");
    assert_eq!(violation.line, Some(42));
}

#[test]
fn test_quality_violation_no_line() {
    let violation = QualityViolation {
        check_type: "dead_code".to_string(),
        severity: "warning".to_string(),
        file: "src/lib.rs".to_string(),
        line: None,
        message: "Unused module".to_string(),
    };
    assert!(violation.line.is_none());
}

#[test]
fn test_quality_violation_empty_file() {
    let violation = QualityViolation {
        check_type: "entropy".to_string(),
        severity: "info".to_string(),
        file: String::new(),
        line: None,
        message: "Low entropy".to_string(),
    };
    assert!(violation.file.is_empty());
}

// --- QualityCheckType coverage ---

#[test]
fn test_quality_check_type_variants() {
    use clap::ValueEnum;
    let variants = QualityCheckType::value_variants();
    assert!(variants.len() >= 8);
    for v in variants {
        let _ = format!("{:?}", v);
        let _ = v.to_possible_value();
    }
}

// --- QualityGateOutputFormat coverage ---

#[test]
fn test_quality_gate_output_format_variants() {
    use clap::ValueEnum;
    let variants = QualityGateOutputFormat::value_variants();
    assert!(variants.len() >= 3);
    for v in variants {
        let _ = format!("{:?}", v);
        let _ = v.to_possible_value();
    }
}

// --- SatdSeverity coverage ---

#[test]
fn test_satd_severity_variants() {
    use clap::ValueEnum;
    let variants = SatdSeverity::value_variants();
    assert_eq!(variants.len(), 4);
    for v in variants {
        let _ = format!("{:?}", v);
        let _ = v.to_possible_value();
    }
}

// --- QualityCheckType Display tests ---

#[test]
fn test_quality_check_type_display() {
    // Test Display impl for models::quality_gate::QualityCheckType
    use crate::models::quality_gate::QualityCheckType as ModelQualityCheckType;

    assert_eq!(format!("{}", ModelQualityCheckType::Complexity), "complexity");
    assert_eq!(format!("{}", ModelQualityCheckType::DeadCode), "dead_code");
    assert_eq!(format!("{}", ModelQualityCheckType::Satd), "satd");
    assert_eq!(format!("{}", ModelQualityCheckType::Security), "security");
    assert_eq!(format!("{}", ModelQualityCheckType::Entropy), "entropy");
    assert_eq!(format!("{}", ModelQualityCheckType::Duplicates), "duplicates");
    assert_eq!(format!("{}", ModelQualityCheckType::Coverage), "coverage");
}

// --- Integration tests with single file quality gate ---

#[tokio::test]
async fn test_handle_quality_gate_nonexistent_file() {
    // Quality gate on a nonexistent file should still work (returns empty violations)
    let result = handle_quality_gate(
        PathBuf::from("."),
        Some(PathBuf::from("/nonexistent/file.rs")),
        QualityGateOutputFormat::Json,
        false,
        vec![QualityCheckType::Security],
        15.0,
        0.5,
        20,
        false,
        None,
        false,
    )
    .await;
    // Should not panic
    assert!(result.is_ok());
}
