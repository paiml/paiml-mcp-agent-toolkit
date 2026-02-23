#![cfg_attr(coverage_nightly, coverage(off))]
//! Integration and property-based tests for defect helpers

use super::format_json::format_defect_json;
use super::format_markdown::{calculate_risk_counts, format_defect_markdown, format_defect_summary};
use super::format_sarif::format_defect_sarif;
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

// Integration-style tests
mod integration_tests {
    use super::*;

    #[test]
    fn test_all_formatters_handle_same_input() {
        let predictions = create_test_predictions();
        let project_path = Path::new("/test");

        // All formatters should succeed with the same input
        let json_result = format_defect_json(&predictions);
        let summary_result = format_defect_summary(&predictions);
        let markdown_result = format_defect_markdown(&predictions, true);
        let sarif_result = format_defect_sarif(&predictions, project_path);

        assert!(json_result.is_ok());
        assert!(summary_result.is_ok());
        assert!(markdown_result.is_ok());
        assert!(sarif_result.is_ok());
    }

    #[test]
    fn test_json_output_is_valid_json() {
        let predictions = create_test_predictions();
        let json_str = format_defect_json(&predictions).expect("Should format JSON");

        // Should be parseable as JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).expect("Should be valid JSON");

        assert!(parsed.get("defect_predictions").is_some());
        assert!(parsed.get("summary").is_some());
    }

    #[test]
    fn test_sarif_output_is_valid_json() {
        let predictions = create_test_predictions();
        let project_path = Path::new("/test");
        let sarif_str =
            format_defect_sarif(&predictions, project_path).expect("Should format SARIF");

        // Should be parseable as JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&sarif_str).expect("Should be valid JSON");

        assert_eq!(parsed["version"], "2.1.0");
        assert!(parsed.get("runs").is_some());
    }

    #[test]
    fn test_markdown_sections_order() {
        let predictions = create_test_predictions();
        let markdown =
            format_defect_markdown(&predictions, true).expect("Should format markdown");

        // Verify sections appear in correct order
        let summary_pos = markdown.find("## Summary").expect("Should have summary");
        let risk_dist_pos = markdown
            .find("### Risk Distribution")
            .expect("Should have risk distribution");
        let detailed_pos = markdown
            .find("## Detailed Predictions")
            .expect("Should have detailed predictions");

        assert!(summary_pos < risk_dist_pos);
        assert!(risk_dist_pos < detailed_pos);
    }

    #[test]
    fn test_predictions_sorted_by_probability() {
        let predictions = vec![
            ("low.rs".to_string(), create_mock_defect_score(0.25, 0.9)),
            ("high.rs".to_string(), create_mock_defect_score(0.85, 0.9)),
            ("medium.rs".to_string(), create_mock_defect_score(0.55, 0.9)),
        ];

        let json_str = format_defect_json(&predictions).expect("Should format JSON");

        // The JSON should show files in order of input (not sorted in format_defect_json)
        // The sorting happens in analyze_defect_probability, not in formatters
        assert!(json_str.contains("low.rs"));
        assert!(json_str.contains("high.rs"));
        assert!(json_str.contains("medium.rs"));
    }
}

// Property-based tests for robustness
mod property_tests_comprehensive {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_format_json_never_panics(
            probability in 0.0f32..=1.0,
            confidence in 0.0f32..=1.0
        ) {
            let predictions = vec![
                ("test.rs".to_string(), create_mock_defect_score(probability, confidence)),
            ];

            let result = format_defect_json(&predictions);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_format_summary_never_panics(
            probability in 0.0f32..=1.0,
            confidence in 0.0f32..=1.0
        ) {
            let predictions = vec![
                ("test.rs".to_string(), create_mock_defect_score(probability, confidence)),
            ];

            let result = format_defect_summary(&predictions);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_format_markdown_never_panics(
            probability in 0.0f32..=1.0,
            confidence in 0.0f32..=1.0,
            include_recommendations in any::<bool>()
        ) {
            let predictions = vec![
                ("test.rs".to_string(), create_mock_defect_score(probability, confidence)),
            ];

            let result = format_defect_markdown(&predictions, include_recommendations);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_calculate_risk_counts_invariant(
            high_count in 0usize..10,
            medium_count in 0usize..10,
            low_count in 0usize..10
        ) {
            let mut predictions = Vec::new();

            for i in 0..high_count {
                predictions.push((format!("high_{}.rs", i), create_mock_defect_score(0.85, 0.9)));
            }
            for i in 0..medium_count {
                predictions.push((format!("medium_{}.rs", i), create_mock_defect_score(0.55, 0.9)));
            }
            for i in 0..low_count {
                predictions.push((format!("low_{}.rs", i), create_mock_defect_score(0.25, 0.9)));
            }

            let (high, medium, low) = calculate_risk_counts(&predictions);

            // Total should equal sum of categories
            prop_assert_eq!(high + medium + low, predictions.len());
        }

        #[test]
        fn test_risk_distribution_matches_counts(num_predictions in 0usize..20) {
            let mut predictions = Vec::new();

            for i in 0..num_predictions {
                let prob = (i as f32) / 20.0; // Spread across 0.0 to ~0.95
                predictions.push((format!("file_{}.rs", i), create_mock_defect_score(prob, 0.9)));
            }

            let (high, medium, low) = calculate_risk_counts(&predictions);

            // Sum should equal total predictions
            prop_assert_eq!(high + medium + low, predictions.len());
        }
    }
}
