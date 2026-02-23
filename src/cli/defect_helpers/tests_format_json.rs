#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for JSON formatting of defect predictions

use super::format_json::format_defect_json;
use crate::services::defect_probability::{DefectScore, RiskLevel};

// Helper function to create a mock DefectScore
fn create_mock_defect_score(probability: f32, confidence: f32) -> DefectScore {
    DefectScore {
        probability,
        confidence,
        contributing_factors: vec![
            ("complexity".to_string(), 0.3),
            ("churn".to_string(), 0.2),
            ("duplication".to_string(), 0.1),
            ("coupling".to_string(), 0.05),
        ],
        risk_level: if probability > 0.7 {
            RiskLevel::High
        } else if probability > 0.3 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        },
        recommendations: vec!["Test recommendation".to_string()],
    }
}

fn create_test_predictions() -> Vec<(String, DefectScore)> {
    vec![
        (
            "src/high_risk.rs".to_string(),
            create_mock_defect_score(0.85, 0.9),
        ),
        (
            "src/medium_risk.rs".to_string(),
            create_mock_defect_score(0.55, 0.8),
        ),
        (
            "src/low_risk.rs".to_string(),
            create_mock_defect_score(0.25, 0.95),
        ),
        (
            "src/another_high.rs".to_string(),
            create_mock_defect_score(0.75, 0.85),
        ),
        (
            "src/very_low.rs".to_string(),
            create_mock_defect_score(0.15, 0.7),
        ),
    ]
}

#[test]
fn test_format_defect_json_empty_predictions() {
    let predictions: Vec<(String, DefectScore)> = vec![];
    let result = format_defect_json(&predictions).expect("Should format empty predictions");

    assert!(result.contains("defect_predictions"));
    assert!(result.contains("\"total_files\": 0"));
    assert!(result.contains("\"high_risk_files\": 0"));
    assert!(result.contains("\"medium_risk_files\": 0"));
    assert!(result.contains("\"low_risk_files\": 0"));
}

#[test]
fn test_format_defect_json_with_predictions() {
    let predictions = create_test_predictions();
    let result = format_defect_json(&predictions).expect("Should format predictions");

    assert!(result.contains("defect_predictions"));
    assert!(result.contains("src/high_risk.rs"));
    assert!(result.contains("src/medium_risk.rs"));
    assert!(result.contains("src/low_risk.rs"));
    assert!(result.contains("\"total_files\": 5"));
}

#[test]
fn test_format_defect_json_risk_counts() {
    let predictions = create_test_predictions();
    let result = format_defect_json(&predictions).expect("Should format predictions");

    // 2 high risk (>0.7): 0.85 and 0.75
    assert!(result.contains("\"high_risk_files\": 2"));
    // 1 medium risk (0.4-0.7): 0.55
    assert!(result.contains("\"medium_risk_files\": 1"));
    // 2 low risk (<=0.4): 0.25 and 0.15
    assert!(result.contains("\"low_risk_files\": 2"));
}

#[test]
fn test_format_defect_json_contains_file_data() {
    let predictions = vec![(
        "test_file.rs".to_string(),
        create_mock_defect_score(0.75, 0.9),
    )];
    let result = format_defect_json(&predictions).expect("Should format predictions");

    assert!(result.contains("\"file\": \"test_file.rs\""));
    assert!(result.contains("\"probability\":"));
    assert!(result.contains("\"confidence\":"));
    assert!(result.contains("risk_factors"));
}
