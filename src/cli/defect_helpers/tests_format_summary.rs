#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for summary formatting of defect predictions

use super::format_markdown::format_defect_summary;
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
fn test_format_defect_summary_empty() {
    let predictions: Vec<(String, DefectScore)> = vec![];
    let result = format_defect_summary(&predictions).expect("Should format empty summary");

    assert!(result.contains("# Defect Prediction Summary"));
    assert!(result.contains("**Total files analyzed**: 0"));
}

#[test]
fn test_format_defect_summary_with_predictions() {
    let predictions = create_test_predictions();
    let result = format_defect_summary(&predictions).expect("Should format summary");

    assert!(result.contains("# Defect Prediction Summary"));
    assert!(result.contains("**Total files analyzed**: 5"));
    assert!(result.contains("## Risk Distribution:"));
    assert!(result.contains("High Risk (>70%): 2 files"));
    assert!(result.contains("Medium Risk (40-70%): 1 files"));
    assert!(result.contains("Low Risk (<40%): 2 files"));
}

#[test]
fn test_format_defect_summary_top_files() {
    let predictions = create_test_predictions();
    let result = format_defect_summary(&predictions).expect("Should format summary");

    assert!(result.contains("## Top 10 High-Risk Files:"));
    // First file should be listed (highest probability)
    assert!(result.contains("src/high_risk.rs"));
}

#[test]
fn test_format_defect_summary_shows_probability_percentages() {
    let predictions = vec![(
        "high_risk.rs".to_string(),
        create_mock_defect_score(0.85, 0.9),
    )];
    let result = format_defect_summary(&predictions).expect("Should format summary");

    // Should show probability as percentage
    assert!(result.contains("85.0% probability"));
}
