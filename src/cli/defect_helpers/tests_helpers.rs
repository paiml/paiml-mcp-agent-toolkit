#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for internal helper functions, edge cases, and write_recommendations

use super::format_json::format_defect_json;
use super::format_markdown::{
    calculate_risk_counts, format_defect_markdown, format_defect_summary,
    write_detailed_predictions, write_prediction_metrics, write_recommendations,
    write_risk_distribution_table, write_risk_row, write_single_prediction, write_summary_section,
};
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

// Tests for internal helper functions
mod helper_function_tests {
    use super::*;

    #[test]
    fn test_write_summary_section() {
        let predictions = create_test_predictions();
        let mut output = String::new();
        write_summary_section(&mut output, &predictions).expect("Should write summary");

        assert!(output.contains("## Summary"));
        assert!(output.contains("**Total files analyzed**: 5"));
    }

    #[test]
    fn test_write_risk_distribution_table() {
        let predictions = create_test_predictions();
        let mut output = String::new();
        write_risk_distribution_table(&mut output, &predictions).expect("Should write risk table");

        assert!(output.contains("### Risk Distribution"));
        assert!(output.contains("| Risk Level | Count | Percentage |"));
    }

    #[test]
    fn test_calculate_risk_counts() {
        let predictions = create_test_predictions();
        let (high, medium, low) = calculate_risk_counts(&predictions);

        assert_eq!(high, 2); // 0.85 and 0.75
        assert_eq!(medium, 1); // 0.55
        assert_eq!(low, 2); // 0.25 and 0.15
    }

    #[test]
    fn test_calculate_risk_counts_empty() {
        let predictions: Vec<(String, DefectScore)> = vec![];
        let (high, medium, low) = calculate_risk_counts(&predictions);

        assert_eq!(high, 0);
        assert_eq!(medium, 0);
        assert_eq!(low, 0);
    }

    #[test]
    fn test_calculate_risk_counts_all_high_risk() {
        let predictions = vec![
            ("a.rs".to_string(), create_mock_defect_score(0.95, 0.9)),
            ("b.rs".to_string(), create_mock_defect_score(0.85, 0.9)),
            ("c.rs".to_string(), create_mock_defect_score(0.75, 0.9)),
        ];
        let (high, medium, low) = calculate_risk_counts(&predictions);

        assert_eq!(high, 3);
        assert_eq!(medium, 0);
        assert_eq!(low, 0);
    }

    #[test]
    fn test_write_risk_row() {
        let mut output = String::new();
        write_risk_row(&mut output, "High (>70%)", 5, 10.0).expect("Should write risk row");

        assert!(output.contains("| High (>70%) | 5 | 50.0% |"));
    }

    #[test]
    fn test_write_risk_row_zero_total() {
        let mut output = String::new();
        // When total is 0.0, division will produce NaN or inf, but that's expected
        // The function handles this gracefully
        let result = write_risk_row(&mut output, "Test", 0, 0.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_detailed_predictions() {
        let predictions = create_test_predictions();
        let mut output = String::new();
        write_detailed_predictions(&mut output, &predictions, false)
            .expect("Should write detailed predictions");

        assert!(output.contains("## Detailed Predictions"));
        // Should contain file headers
        assert!(output.contains("### src/high_risk.rs"));
    }

    #[test]
    fn test_write_detailed_predictions_limits_to_20() {
        // Create more than 20 predictions
        let mut predictions = Vec::new();
        for i in 0..25 {
            predictions.push((format!("file_{}.rs", i), create_mock_defect_score(0.5, 0.8)));
        }

        let mut output = String::new();
        write_detailed_predictions(&mut output, &predictions, false)
            .expect("Should write limited predictions");

        // Should only contain up to 20 files
        assert!(output.contains("file_0.rs"));
        assert!(output.contains("file_19.rs"));
        // file_20.rs and above should not be included
        assert!(!output.contains("file_20.rs"));
    }

    #[test]
    fn test_write_single_prediction() {
        let score = create_mock_defect_score(0.75, 0.9);
        let mut output = String::new();
        write_single_prediction(&mut output, "test.rs", &score, false)
            .expect("Should write prediction");

        assert!(output.contains("### test.rs"));
        assert!(output.contains("**Probability**:"));
        assert!(output.contains("**Confidence**:"));
        assert!(output.contains("**Risk Factors**:"));
    }

    #[test]
    fn test_write_single_prediction_with_recommendations() {
        let score = create_mock_defect_score(0.85, 0.9);
        let mut output = String::new();
        write_single_prediction(&mut output, "test.rs", &score, true)
            .expect("Should write prediction");

        assert!(output.contains("#### Recommendations:"));
        assert!(output.contains("High priority for code review"));
    }

    #[test]
    fn test_write_prediction_metrics() {
        let score = create_mock_defect_score(0.75, 0.85);
        let mut output = String::new();
        write_prediction_metrics(&mut output, &score).expect("Should write metrics");

        assert!(output.contains("**Probability**: 75.0%"));
        assert!(output.contains("**Confidence**: 85.0%"));
        assert!(output.contains("**Risk Factors**:"));
    }
}

// Tests for write_recommendations
mod write_recommendations_tests {
    use super::*;

    #[test]
    fn test_write_recommendations_high_risk() {
        let mut output = String::new();
        write_recommendations(&mut output, 0.85).expect("Should write high risk recommendations");

        assert!(output.contains("#### Recommendations:"));
        assert!(output.contains("High priority for code review"));
        assert!(output.contains("Add comprehensive test coverage"));
        assert!(output.contains("Consider refactoring to reduce complexity"));
    }

    #[test]
    fn test_write_recommendations_medium_risk() {
        let mut output = String::new();
        write_recommendations(&mut output, 0.55).expect("Should write medium risk recommendations");

        assert!(output.contains("#### Recommendations:"));
        assert!(output.contains("Schedule for regular review"));
        assert!(output.contains("Improve test coverage"));
        assert!(!output.contains("High priority")); // Should not have high priority message
    }

    #[test]
    fn test_write_recommendations_low_risk() {
        let mut output = String::new();
        write_recommendations(&mut output, 0.25).expect("Should write low risk recommendations");

        assert!(output.contains("#### Recommendations:"));
        assert!(output.contains("Monitor during regular maintenance"));
        assert!(!output.contains("High priority"));
        assert!(!output.contains("Schedule for regular review"));
    }

    #[test]
    fn test_write_recommendations_boundary_high() {
        let mut output = String::new();
        // Exactly at boundary (>0.7)
        write_recommendations(&mut output, 0.71).expect("Should write recommendations");
        assert!(output.contains("High priority"));
    }

    #[test]
    fn test_write_recommendations_boundary_medium() {
        let mut output = String::new();
        // Exactly at boundary (>0.4 and <=0.7)
        write_recommendations(&mut output, 0.41).expect("Should write recommendations");
        assert!(output.contains("Schedule for regular review"));
    }

    #[test]
    fn test_write_recommendations_boundary_low() {
        let mut output = String::new();
        // At boundary (<=0.4)
        write_recommendations(&mut output, 0.4).expect("Should write recommendations");
        assert!(output.contains("Monitor during regular maintenance"));
    }
}

// Edge case tests
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_format_json_single_prediction() {
        let predictions = vec![("single.rs".to_string(), create_mock_defect_score(0.5, 0.8))];
        let result = format_defect_json(&predictions).expect("Should format single prediction");

        assert!(result.contains("\"total_files\": 1"));
        assert!(result.contains("single.rs"));
    }

    #[test]
    fn test_format_summary_single_prediction() {
        let predictions = vec![("single.rs".to_string(), create_mock_defect_score(0.5, 0.8))];
        let result = format_defect_summary(&predictions).expect("Should format single prediction");

        assert!(result.contains("**Total files analyzed**: 1"));
    }

    #[test]
    fn test_format_markdown_boundary_probabilities() {
        // Test with probabilities exactly at boundaries
        let predictions = vec![
            (
                "exact_70.rs".to_string(),
                create_mock_defect_score(0.70, 0.9),
            ),
            (
                "exact_40.rs".to_string(),
                create_mock_defect_score(0.40, 0.9),
            ),
        ];
        let result = format_defect_markdown(&predictions, false).expect("Should format markdown");

        assert!(result.contains("# Defect Prediction Report"));
    }

    #[test]
    fn test_format_sarif_special_characters_in_filename() {
        let predictions = vec![(
            "src/path with spaces/file.rs".to_string(),
            create_mock_defect_score(0.75, 0.9),
        )];
        let project_path = Path::new("/test");
        let result =
            format_defect_sarif(&predictions, project_path).expect("Should handle special chars");

        assert!(result.contains("path with spaces"));
    }

    #[test]
    fn test_format_json_zero_probability() {
        let mut score = create_mock_defect_score(0.0, 0.9);
        score.probability = 0.0;
        let predictions = vec![("zero.rs".to_string(), score)];
        let result = format_defect_json(&predictions).expect("Should handle zero probability");

        assert!(result.contains("\"probability\": 0"));
    }

    #[test]
    fn test_format_json_max_probability() {
        let mut score = create_mock_defect_score(1.0, 1.0);
        score.probability = 1.0;
        score.confidence = 1.0;
        let predictions = vec![("max.rs".to_string(), score)];
        let result = format_defect_json(&predictions).expect("Should handle max probability");

        assert!(result.contains("\"probability\": 1"));
        assert!(result.contains("\"confidence\": 1"));
    }

    #[test]
    fn test_format_markdown_empty_contributing_factors() {
        let mut score = create_mock_defect_score(0.75, 0.9);
        score.contributing_factors = vec![];
        let predictions = vec![("empty_factors.rs".to_string(), score)];

        let result =
            format_defect_markdown(&predictions, false).expect("Should handle empty factors");
        assert!(result.contains("# Defect Prediction Report"));
    }

    #[test]
    fn test_sarif_boundary_probability_values() {
        // Test boundary values: 0.7 should be "warning", >0.7 should be "error"
        let predictions = vec![
            ("at_70.rs".to_string(), create_mock_defect_score(0.70, 0.9)),
            (
                "above_70.rs".to_string(),
                create_mock_defect_score(0.71, 0.9),
            ),
            ("at_40.rs".to_string(), create_mock_defect_score(0.40, 0.9)),
            (
                "above_40.rs".to_string(),
                create_mock_defect_score(0.41, 0.9),
            ),
        ];
        let project_path = Path::new("/test");
        let result =
            format_defect_sarif(&predictions, project_path).expect("Should handle boundaries");

        // The function should have processed all predictions
        assert!(result.contains("at_70.rs"));
        assert!(result.contains("above_70.rs"));
    }
}
