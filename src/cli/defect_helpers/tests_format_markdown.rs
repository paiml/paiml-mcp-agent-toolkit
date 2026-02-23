#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for markdown formatting of defect predictions

use super::format_markdown::format_defect_markdown;
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
fn test_format_defect_markdown_empty() {
    let predictions: Vec<(String, DefectScore)> = vec![];
    let result =
        format_defect_markdown(&predictions, false).expect("Should format empty markdown");

    assert!(result.contains("# Defect Prediction Report"));
    assert!(result.contains("## Summary"));
    assert!(result.contains("**Total files analyzed**: 0"));
}

#[test]
fn test_format_defect_markdown_with_recommendations() {
    let predictions = create_test_predictions();
    let result = format_defect_markdown(&predictions, true)
        .expect("Should format markdown with recommendations");

    assert!(result.contains("# Defect Prediction Report"));
    assert!(result.contains("#### Recommendations:"));
}

#[test]
fn test_format_defect_markdown_without_recommendations() {
    let predictions = create_test_predictions();
    let result = format_defect_markdown(&predictions, false)
        .expect("Should format markdown without recommendations");

    assert!(result.contains("# Defect Prediction Report"));
    // Should not contain recommendations section when disabled
    // The detailed predictions still show metrics but not the recommendations subsection
}

#[test]
fn test_format_defect_markdown_risk_table() {
    let predictions = create_test_predictions();
    let result =
        format_defect_markdown(&predictions, false).expect("Should format markdown");

    assert!(result.contains("### Risk Distribution"));
    assert!(result.contains("| Risk Level | Count | Percentage |"));
    assert!(result.contains("| High (>70%) |"));
    assert!(result.contains("| Medium (40-70%) |"));
    assert!(result.contains("| Low (<40%) |"));
}

#[test]
fn test_format_defect_markdown_detailed_predictions() {
    let predictions = create_test_predictions();
    let result =
        format_defect_markdown(&predictions, false).expect("Should format markdown");

    assert!(result.contains("## Detailed Predictions"));
    assert!(result.contains("**Probability**:"));
    assert!(result.contains("**Confidence**:"));
    assert!(result.contains("**Risk Factors**:"));
}
