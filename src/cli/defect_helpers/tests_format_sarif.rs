#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for SARIF formatting of defect predictions

use super::format_sarif::{format_defect_sarif, generate_defect_rules};
use crate::services::defect_probability::{DefectScore, RiskLevel};
use std::path::Path;

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
fn test_format_defect_sarif_empty() {
    let predictions: Vec<(String, DefectScore)> = vec![];
    let project_path = Path::new("/test/project");
    let result =
        format_defect_sarif(&predictions, project_path).expect("Should format empty SARIF");

    assert!(result.contains("\"version\": \"2.1.0\""));
    assert!(result.contains("sarif-schema-2.1.0.json"));
    assert!(result.contains("paiml-defect-predictor"));
    assert!(result.contains("\"results\": []"));
}

#[test]
fn test_format_defect_sarif_with_predictions() {
    let predictions = create_test_predictions();
    let project_path = Path::new("/test/project");
    let result = format_defect_sarif(&predictions, project_path).expect("Should format SARIF");

    assert!(result.contains("\"version\": \"2.1.0\""));
    assert!(result.contains("src/high_risk.rs"));
    assert!(result.contains("src/low_risk.rs"));
}

#[test]
fn test_format_defect_sarif_high_risk_level() {
    let predictions = vec![(
        "high_risk.rs".to_string(),
        create_mock_defect_score(0.85, 0.9),
    )];
    let project_path = Path::new("/test/project");
    let result = format_defect_sarif(&predictions, project_path).expect("Should format SARIF");

    assert!(result.contains("\"ruleId\": \"high-defect-probability\""));
    assert!(result.contains("\"level\": \"error\""));
}

#[test]
fn test_format_defect_sarif_medium_risk_level() {
    let predictions = vec![(
        "medium_risk.rs".to_string(),
        create_mock_defect_score(0.55, 0.8),
    )];
    let project_path = Path::new("/test/project");
    let result = format_defect_sarif(&predictions, project_path).expect("Should format SARIF");

    assert!(result.contains("\"ruleId\": \"medium-defect-probability\""));
    assert!(result.contains("\"level\": \"warning\""));
}

#[test]
fn test_format_defect_sarif_low_risk_level() {
    let predictions = vec![(
        "low_risk.rs".to_string(),
        create_mock_defect_score(0.25, 0.9),
    )];
    let project_path = Path::new("/test/project");
    let result = format_defect_sarif(&predictions, project_path).expect("Should format SARIF");

    assert!(result.contains("\"ruleId\": \"low-defect-probability\""));
    assert!(result.contains("\"level\": \"note\""));
}

#[test]
fn test_format_defect_sarif_contains_rules() {
    let predictions = create_test_predictions();
    let project_path = Path::new("/test/project");
    let result = format_defect_sarif(&predictions, project_path).expect("Should format SARIF");

    assert!(result.contains("\"id\": \"high-defect-probability\""));
    assert!(result.contains("\"id\": \"medium-defect-probability\""));
    assert!(result.contains("\"id\": \"low-defect-probability\""));
    assert!(result.contains("\"name\": \"High Defect Probability\""));
}

#[test]
fn test_format_defect_sarif_location_format() {
    let predictions = vec![(
        "src/test.rs".to_string(),
        create_mock_defect_score(0.75, 0.9),
    )];
    let project_path = Path::new("/test/project");
    let result = format_defect_sarif(&predictions, project_path).expect("Should format SARIF");

    assert!(result.contains("\"locations\""));
    assert!(result.contains("\"physicalLocation\""));
    assert!(result.contains("\"artifactLocation\""));
    assert!(result.contains("\"uri\": \"src/test.rs\""));
}

// Tests for generate_defect_rules
#[test]
fn test_generate_defect_rules_returns_three_rules() {
    let rules = generate_defect_rules();
    assert_eq!(rules.len(), 3);
}

#[test]
fn test_generate_defect_rules_high_risk_rule() {
    let rules = generate_defect_rules();
    let high_rule = &rules[0];

    assert_eq!(high_rule["id"], "high-defect-probability");
    assert_eq!(high_rule["name"], "High Defect Probability");
    assert_eq!(high_rule["defaultConfiguration"]["level"], "error");
}

#[test]
fn test_generate_defect_rules_medium_risk_rule() {
    let rules = generate_defect_rules();
    let medium_rule = &rules[1];

    assert_eq!(medium_rule["id"], "medium-defect-probability");
    assert_eq!(medium_rule["name"], "Medium Defect Probability");
    assert_eq!(medium_rule["defaultConfiguration"]["level"], "warning");
}

#[test]
fn test_generate_defect_rules_low_risk_rule() {
    let rules = generate_defect_rules();
    let low_rule = &rules[2];

    assert_eq!(low_rule["id"], "low-defect-probability");
    assert_eq!(low_rule["name"], "Low Defect Probability");
    assert_eq!(low_rule["defaultConfiguration"]["level"], "note");
}

#[test]
fn test_generate_defect_rules_have_descriptions() {
    let rules = generate_defect_rules();

    for rule in rules {
        assert!(rule.get("shortDescription").is_some());
        assert!(rule.get("fullDescription").is_some());
        assert!(rule["shortDescription"]["text"].as_str().is_some());
        assert!(rule["fullDescription"]["text"].as_str().is_some());
    }
}
