#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use crate::services::defect_probability::RiskLevel;
use std::time::Duration;

// Helper function to create mock DefectScore
fn create_mock_defect_score(
    probability: f32,
    confidence: f32,
    risk_level: RiskLevel,
) -> DefectScore {
    DefectScore {
        probability,
        confidence,
        risk_level,
        contributing_factors: vec![
            ("complexity".to_string(), 0.25),
            ("churn".to_string(), 0.20),
            ("duplication".to_string(), 0.15),
            ("coupling".to_string(), 0.10),
        ],
        recommendations: vec![
            "Consider refactoring this file".to_string(),
            "Increase test coverage".to_string(),
        ],
    }
}

fn create_high_risk_score() -> DefectScore {
    create_mock_defect_score(0.85, 0.90, RiskLevel::High)
}

fn create_medium_risk_score() -> DefectScore {
    create_mock_defect_score(0.50, 0.85, RiskLevel::Medium)
}

fn create_low_risk_score() -> DefectScore {
    create_mock_defect_score(0.20, 0.80, RiskLevel::Low)
}

fn create_test_predictions() -> Vec<(String, DefectScore)> {
    vec![
        ("src/high_risk.rs".to_string(), create_high_risk_score()),
        ("src/medium_risk.rs".to_string(), create_medium_risk_score()),
        ("src/low_risk.rs".to_string(), create_low_risk_score()),
    ]
}

// ==================== Test create_defect_prediction_config ====================

#[test]
fn test_create_defect_prediction_config_default_values() {
    let config = create_defect_prediction_config(0.5, 10, false, false, true, None, None);

    assert_eq!(config.confidence_threshold, 0.5);
    assert_eq!(config.min_lines, 10);
    assert!(!config.include_low_confidence);
    assert!(!config.high_risk_only);
    assert!(config.include_recommendations);
    assert!(config.include.is_none());
    assert!(config.exclude.is_none());
}

#[test]
fn test_create_defect_prediction_config_with_patterns() {
    let config = create_defect_prediction_config(
        0.7,
        50,
        true,
        true,
        false,
        Some("src/".to_string()),
        Some("test/".to_string()),
    );

    assert_eq!(config.confidence_threshold, 0.7);
    assert_eq!(config.min_lines, 50);
    assert!(config.include_low_confidence);
    assert!(config.high_risk_only);
    assert!(!config.include_recommendations);
    assert_eq!(config.include, Some("src/".to_string()));
    assert_eq!(config.exclude, Some("test/".to_string()));
}

// ==================== Test calculate_risk_statistics ====================

#[test]
fn test_calculate_risk_statistics_all_categories() {
    let predictions = create_test_predictions();
    let stats = calculate_risk_statistics(&predictions);

    assert_eq!(stats.high_risk, 1);
    assert_eq!(stats.medium_risk, 1);
    assert_eq!(stats.low_risk, 1);
}

#[test]
fn test_calculate_risk_statistics_empty() {
    let predictions: Vec<(String, DefectScore)> = vec![];
    let stats = calculate_risk_statistics(&predictions);

    assert_eq!(stats.high_risk, 0);
    assert_eq!(stats.medium_risk, 0);
    assert_eq!(stats.low_risk, 0);
}

#[test]
fn test_calculate_risk_statistics_all_high_risk() {
    let predictions = vec![
        ("file1.rs".to_string(), create_high_risk_score()),
        ("file2.rs".to_string(), create_high_risk_score()),
        ("file3.rs".to_string(), create_high_risk_score()),
    ];
    let stats = calculate_risk_statistics(&predictions);

    assert_eq!(stats.high_risk, 3);
    assert_eq!(stats.medium_risk, 0);
    assert_eq!(stats.low_risk, 0);
}

#[test]
fn test_calculate_risk_statistics_boundary_values() {
    // Test boundary at 0.7 (high/medium)
    let high_boundary = create_mock_defect_score(0.7, 0.9, RiskLevel::Medium);
    // Test boundary at 0.3 (medium/low)
    let low_boundary = create_mock_defect_score(0.3, 0.9, RiskLevel::Low);

    let predictions = vec![
        ("high_boundary.rs".to_string(), high_boundary),
        ("low_boundary.rs".to_string(), low_boundary),
    ];
    let stats = calculate_risk_statistics(&predictions);

    // 0.7 is NOT > 0.7, so it's medium
    assert_eq!(stats.high_risk, 0);
    assert_eq!(stats.medium_risk, 1); // 0.7 is in (0.3, 0.7]
    assert_eq!(stats.low_risk, 1); // 0.3 is <= 0.3
}

// ==================== Test get_risk_icon ====================

#[test]
fn test_get_risk_icon_high() {
    assert_eq!(get_risk_icon(&RiskLevel::High), "🔴");
}

#[test]
fn test_get_risk_icon_medium() {
    assert_eq!(get_risk_icon(&RiskLevel::Medium), "🟡");
}

#[test]
fn test_get_risk_icon_low() {
    assert_eq!(get_risk_icon(&RiskLevel::Low), "🟢");
}

// ==================== Test format_risk_level_display ====================

#[test]
fn test_format_risk_level_display_high() {
    assert_eq!(format_risk_level_display(&RiskLevel::High), "🔴 HIGH");
}

#[test]
fn test_format_risk_level_display_medium() {
    assert_eq!(format_risk_level_display(&RiskLevel::Medium), "🟡 MEDIUM");
}

#[test]
fn test_format_risk_level_display_low() {
    assert_eq!(format_risk_level_display(&RiskLevel::Low), "🟢 LOW");
}

// ==================== Test filter_and_sort_predictions ====================

#[test]
fn test_filter_and_sort_predictions_high_risk_only() {
    let predictions = create_test_predictions();
    let filtered = filter_and_sort_predictions(
        predictions,
        true,  // high_risk_only
        false, // include_low_confidence
        0.5,   // confidence_threshold
        10,    // top_files
    );

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].0, "src/high_risk.rs");
}

#[test]
fn test_filter_and_sort_predictions_low_confidence_filtered() {
    let mut predictions = create_test_predictions();
    // Add a low confidence prediction
    predictions.push((
        "low_conf.rs".to_string(),
        create_mock_defect_score(0.9, 0.3, RiskLevel::High),
    ));

    let filtered = filter_and_sort_predictions(
        predictions,
        false, // high_risk_only
        false, // include_low_confidence (filter out low confidence)
        0.5,   // confidence_threshold
        10,    // top_files
    );

    // Should filter out the low confidence file
    assert_eq!(filtered.len(), 3);
    assert!(filtered.iter().all(|(f, _)| f != "low_conf.rs"));
}

#[test]
fn test_filter_and_sort_predictions_include_low_confidence() {
    let mut predictions = create_test_predictions();
    predictions.push((
        "low_conf.rs".to_string(),
        create_mock_defect_score(0.9, 0.3, RiskLevel::High),
    ));

    let filtered = filter_and_sort_predictions(
        predictions,
        false, // high_risk_only
        true,  // include_low_confidence
        0.5,   // confidence_threshold
        10,    // top_files
    );

    // Should include all files
    assert_eq!(filtered.len(), 4);
}

#[test]
fn test_filter_and_sort_predictions_sorted_by_probability() {
    let predictions = create_test_predictions();
    let filtered = filter_and_sort_predictions(
        predictions,
        false, // high_risk_only
        true,  // include_low_confidence
        0.0,   // confidence_threshold
        10,    // top_files
    );

    // Verify sorted by probability descending
    assert_eq!(filtered[0].0, "src/high_risk.rs");
    assert_eq!(filtered[1].0, "src/medium_risk.rs");
    assert_eq!(filtered[2].0, "src/low_risk.rs");
}

#[test]
fn test_filter_and_sort_predictions_top_files_limit() {
    let predictions = create_test_predictions();
    let filtered = filter_and_sort_predictions(
        predictions,
        false, // high_risk_only
        true,  // include_low_confidence
        0.0,   // confidence_threshold
        2,     // top_files - limit to 2
    );

    assert_eq!(filtered.len(), 2);
}

#[test]
fn test_filter_and_sort_predictions_top_files_zero_means_unlimited() {
    let predictions = create_test_predictions();
    let filtered = filter_and_sort_predictions(
        predictions,
        false, // high_risk_only
        true,  // include_low_confidence
        0.0,   // confidence_threshold
        0,     // top_files - 0 means unlimited
    );

    assert_eq!(filtered.len(), 3);
}

// ==================== Test format_defect_output ====================

#[test]
fn test_format_defect_output_summary() {
    let predictions = create_test_predictions();
    let elapsed = Duration::from_millis(100);

    let result = format_defect_output(
        DefectPredictionOutputFormat::Summary,
        &predictions,
        elapsed,
        false,
    );

    assert!(result.is_ok());
    let content = result.unwrap();
    assert!(content.contains("Defect Prediction Summary"));
    assert!(content.contains("Risk Distribution"));
}

#[test]
fn test_format_defect_output_json() {
    let predictions = create_test_predictions();
    let elapsed = Duration::from_millis(100);

    let result = format_defect_output(
        DefectPredictionOutputFormat::Json,
        &predictions,
        elapsed,
        false,
    );

    assert!(result.is_ok());
    let content = result.unwrap();
    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(parsed.get("analysis_type").is_some());
    assert!(parsed.get("predictions").is_some());
}

#[test]
fn test_format_defect_output_detailed() {
    let predictions = create_test_predictions();
    let elapsed = Duration::from_millis(100);

    let result = format_defect_output(
        DefectPredictionOutputFormat::Detailed,
        &predictions,
        elapsed,
        true,
    );

    assert!(result.is_ok());
    let content = result.unwrap();
    assert!(content.contains("Detailed Report"));
    assert!(content.contains("Recommendations"));
}

#[test]
fn test_format_defect_output_sarif() {
    let predictions = create_test_predictions();
    let elapsed = Duration::from_millis(100);

    let result = format_defect_output(
        DefectPredictionOutputFormat::Sarif,
        &predictions,
        elapsed,
        false,
    );

    assert!(result.is_ok());
    let content = result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.get("version").unwrap(), "2.1.0");
    assert!(parsed.get("runs").is_some());
}

#[test]
fn test_format_defect_output_csv() {
    let predictions = create_test_predictions();
    let elapsed = Duration::from_millis(100);

    let result = format_defect_output(
        DefectPredictionOutputFormat::Csv,
        &predictions,
        elapsed,
        false,
    );

    assert!(result.is_ok());
    let content = result.unwrap();
    assert!(content.contains("file,probability,confidence,risk_level"));
    assert!(content.contains("high_risk.rs"));
}

// ==================== Test format_defect_summary ====================

#[test]
fn test_format_defect_summary_with_predictions() {
    let predictions = create_test_predictions();
    let elapsed = Duration::from_millis(250);

    let result = format_defect_summary(&predictions, elapsed).unwrap();

    assert!(result.contains("Defect Prediction Summary"));
    assert!(result.contains("Risk Distribution"));
    assert!(result.contains("Top Risk Files"));
    assert!(result.contains("Analysis time"));
}

#[test]
fn test_format_defect_summary_empty_predictions() {
    let predictions: Vec<(String, DefectScore)> = vec![];
    let elapsed = Duration::from_millis(50);

    let result = format_defect_summary(&predictions, elapsed).unwrap();

    assert!(result.contains("Defect Prediction Summary"));
    assert!(result.contains("0 files")); // Risk distribution shows 0, no "Top Risk Files" section
}

// ==================== Test write_summary_header ====================

#[test]
fn test_write_summary_header() {
    let mut output = String::new();
    write_summary_header(&mut output).unwrap();

    assert!(output.contains("Defect Prediction Summary"));
    assert!(output.contains("==="));
}

// ==================== Test write_risk_distribution ====================

#[test]
fn test_write_risk_distribution() {
    let predictions = create_test_predictions();
    let mut output = String::new();

    write_risk_distribution(&mut output, &predictions).unwrap();

    assert!(output.contains("Risk Distribution"));
    assert!(output.contains("High risk"));
    assert!(output.contains("Medium risk"));
    assert!(output.contains("Low risk"));
}

// ==================== Test write_top_risk_files ====================

#[test]
fn test_write_top_risk_files_with_data() {
    let predictions = create_test_predictions();
    let mut output = String::new();

    write_top_risk_files(&mut output, &predictions).unwrap();

    assert!(output.contains("Top Risk Files"));
    assert!(output.contains("src/high_risk.rs"));
}

#[test]
fn test_write_top_risk_files_empty() {
    let predictions: Vec<(String, DefectScore)> = vec![];
    let mut output = String::new();

    write_top_risk_files(&mut output, &predictions).unwrap();

    // Should not write anything for empty predictions
    assert!(!output.contains("Top Risk Files"));
}

#[test]
fn test_write_top_risk_files_more_than_10() {
    // Create more than 10 predictions
    let predictions: Vec<_> = (0..15)
        .map(|i| (format!("file{}.rs", i), create_high_risk_score()))
        .collect();

    let mut output = String::new();
    write_top_risk_files(&mut output, &predictions).unwrap();

    // Should only show 10 files
    let file_count = output.matches("file").count();
    assert_eq!(file_count, 10);
}

// ==================== Test write_summary_footer ====================

#[test]
fn test_write_summary_footer() {
    let elapsed = Duration::from_millis(1234);
    let mut output = String::new();

    write_summary_footer(&mut output, elapsed).unwrap();

    assert!(output.contains("Analysis time"));
}

// ==================== Test format_defect_json ====================

#[test]
fn test_format_defect_json_structure() {
    let predictions = create_test_predictions();
    let elapsed = Duration::from_millis(500);

    let result = format_defect_json(&predictions, elapsed).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // Check structure
    assert_eq!(parsed["analysis_type"], "defect_prediction");
    assert!(parsed["summary"]["total_files_analyzed"].as_u64().is_some());
    assert!(parsed["summary"]["high_risk_files"].as_u64().is_some());
    assert!(parsed["summary"]["medium_risk_files"].as_u64().is_some());
    assert!(parsed["summary"]["low_risk_files"].as_u64().is_some());
    assert!(parsed["summary"]["analysis_time_ms"].as_u64().is_some());

    // Check predictions array
    let preds = parsed["predictions"].as_array().unwrap();
    assert_eq!(preds.len(), 3);

    // Check individual prediction structure
    let first = &preds[0];
    assert!(first["file"].is_string());
    assert!(first["probability"].is_f64());
    assert!(first["confidence"].is_f64());
    assert!(first["risk_level"].is_string());
}

#[test]
fn test_format_defect_json_empty() {
    let predictions: Vec<(String, DefectScore)> = vec![];
    let elapsed = Duration::from_millis(10);

    let result = format_defect_json(&predictions, elapsed).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert_eq!(parsed["summary"]["total_files_analyzed"], 0);
    assert!(parsed["predictions"].as_array().unwrap().is_empty());
}

// ==================== Test format_defect_detailed ====================

#[test]
fn test_format_defect_detailed_with_recommendations() {
    let predictions = create_test_predictions();
    let elapsed = Duration::from_millis(100);

    let result = format_defect_detailed(&predictions, elapsed, true).unwrap();

    assert!(result.contains("Detailed Report"));
    assert!(result.contains("File:"));
    assert!(result.contains("Risk Level:"));
    assert!(result.contains("Confidence:"));
    assert!(result.contains("Contributing Factors:"));
    assert!(result.contains("Recommendations:"));
}

#[test]
fn test_format_defect_detailed_without_recommendations() {
    let predictions = create_test_predictions();
    let elapsed = Duration::from_millis(100);

    let result = format_defect_detailed(&predictions, elapsed, false).unwrap();

    assert!(result.contains("Detailed Report"));
    assert!(!result.contains("Recommendations:"));
}

// ==================== Test write_detailed_header ====================

#[test]
fn test_write_detailed_header() {
    let mut output = String::new();
    write_detailed_header(&mut output).unwrap();

    assert!(output.contains("Detailed Report"));
    assert!(output.contains("==="));
}

// ==================== Test write_file_details ====================

#[test]
fn test_write_file_details_with_recommendations() {
    let score = create_high_risk_score();
    let mut output = String::new();

    write_file_details(&mut output, "test.rs", &score, true).unwrap();

    assert!(output.contains("test.rs"));
    assert!(output.contains("Risk Level:"));
    assert!(output.contains("Confidence:"));
    assert!(output.contains("Recommendations:"));
}

#[test]
fn test_write_file_details_without_recommendations() {
    let score = create_high_risk_score();
    let mut output = String::new();

    write_file_details(&mut output, "test.rs", &score, false).unwrap();

    assert!(output.contains("test.rs"));
    assert!(!output.contains("Recommendations:"));
}

// ==================== Test write_risk_level ====================

#[test]
fn test_write_risk_level() {
    let score = create_high_risk_score();
    let mut output = String::new();

    write_risk_level(&mut output, &score).unwrap();

    assert!(output.contains("Risk Level:"));
    assert!(output.contains("HIGH"));
    assert!(output.contains("85.0%"));
}

// ==================== Test write_confidence_level ====================

#[test]
fn test_write_confidence_level() {
    let score = create_high_risk_score();
    let mut output = String::new();

    write_confidence_level(&mut output, &score).unwrap();

    assert!(output.contains("Confidence:"));
    assert!(output.contains("90.0%"));
}

// ==================== Test write_contributing_factors ====================

#[test]
fn test_write_contributing_factors() {
    let score = create_high_risk_score();
    let mut output = String::new();

    write_contributing_factors(&mut output, &score).unwrap();

    assert!(output.contains("Contributing Factors:"));
    assert!(output.contains("complexity"));
    assert!(output.contains("churn"));
}

#[test]
fn test_write_contributing_factors_empty() {
    let mut score = create_high_risk_score();
    score.contributing_factors = vec![];
    let mut output = String::new();

    write_contributing_factors(&mut output, &score).unwrap();

    // Should not write anything for empty factors
    assert!(output.is_empty());
}

// ==================== Test write_recommendations ====================

#[test]
fn test_write_recommendations() {
    let score = create_high_risk_score();
    let mut output = String::new();

    write_recommendations(&mut output, &score).unwrap();

    assert!(output.contains("Recommendations:"));
    assert!(output.contains("refactoring"));
}

#[test]
fn test_write_recommendations_empty() {
    let mut score = create_high_risk_score();
    score.recommendations = vec![];
    let mut output = String::new();

    write_recommendations(&mut output, &score).unwrap();

    // Should not write anything for empty recommendations
    assert!(output.is_empty());
}

// ==================== Test write_analysis_footer ====================

#[test]
fn test_write_analysis_footer() {
    let elapsed = Duration::from_secs(2);
    let mut output = String::new();

    write_analysis_footer(&mut output, elapsed).unwrap();

    assert!(output.contains("Analysis time:"));
}

// ==================== Test format_defect_sarif ====================

#[test]
fn test_format_defect_sarif_structure() {
    let predictions = create_test_predictions();

    let result = format_defect_sarif(&predictions).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // Check SARIF schema
    assert_eq!(parsed["version"], "2.1.0");
    assert!(parsed["$schema"].as_str().unwrap().contains("sarif-schema"));

    // Check runs
    let runs = parsed["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 1);

    // Check tool info
    let tool = &runs[0]["tool"]["driver"];
    assert_eq!(tool["name"], "pmat-defect-prediction");
    assert!(tool["version"].is_string());

    // Check results
    let results = runs[0]["results"].as_array().unwrap();
    assert_eq!(results.len(), 3);
}

#[test]
fn test_format_defect_sarif_risk_levels() {
    let predictions = create_test_predictions();

    let result = format_defect_sarif(&predictions).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    let results = parsed["runs"][0]["results"].as_array().unwrap();

    // High risk should be "error"
    let high_risk = results
        .iter()
        .find(|r| {
            r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                .as_str()
                .unwrap()
                .contains("high_risk")
        })
        .unwrap();
    assert_eq!(high_risk["level"], "error");

    // Medium risk should be "warning"
    let medium_risk = results
        .iter()
        .find(|r| {
            r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                .as_str()
                .unwrap()
                .contains("medium_risk")
        })
        .unwrap();
    assert_eq!(medium_risk["level"], "warning");

    // Low risk should be "note"
    let low_risk = results
        .iter()
        .find(|r| {
            r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                .as_str()
                .unwrap()
                .contains("low_risk")
        })
        .unwrap();
    assert_eq!(low_risk["level"], "note");
}

// ==================== Test format_defect_csv ====================

#[test]
fn test_format_defect_csv_header() {
    let predictions = create_test_predictions();

    let result = format_defect_csv(&predictions).unwrap();

    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(
        lines[0],
        "file,probability,confidence,risk_level,top_factor,top_factor_weight"
    );
}

#[test]
fn test_format_defect_csv_data_rows() {
    let predictions = create_test_predictions();

    let result = format_defect_csv(&predictions).unwrap();

    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 4); // 1 header + 3 data rows

    // Check first data row
    assert!(lines[1].contains("high_risk.rs"));
    assert!(lines[1].contains("0.850"));
}

#[test]
fn test_format_defect_csv_empty_factors() {
    let mut score = create_high_risk_score();
    score.contributing_factors = vec![];
    let predictions = vec![("test.rs".to_string(), score)];

    let result = format_defect_csv(&predictions).unwrap();

    // Should handle missing top factor gracefully
    assert!(result.contains("test.rs"));
    assert!(result.contains("0.000")); // Default weight
}

// ==================== Test RiskStatistics struct ====================

#[test]
fn test_risk_statistics_struct() {
    let stats = RiskStatistics {
        high_risk: 5,
        medium_risk: 10,
        low_risk: 15,
    };

    assert_eq!(stats.high_risk, 5);
    assert_eq!(stats.medium_risk, 10);
    assert_eq!(stats.low_risk, 15);
}

// ==================== Edge case tests ====================

#[test]
fn test_format_defect_summary_single_file() {
    let predictions = vec![("only_file.rs".to_string(), create_high_risk_score())];
    let elapsed = Duration::from_millis(50);

    let result = format_defect_summary(&predictions, elapsed).unwrap();

    assert!(result.contains("1 files"));
    assert!(result.contains("only_file.rs"));
}

#[test]
fn test_format_with_special_characters_in_filename() {
    let predictions = vec![("src/my-file_v2.0.rs".to_string(), create_high_risk_score())];
    let elapsed = Duration::from_millis(50);

    let result = format_defect_summary(&predictions, elapsed).unwrap();
    assert!(result.contains("my-file_v2.0.rs"));

    let csv = format_defect_csv(&predictions).unwrap();
    assert!(csv.contains("my-file_v2.0.rs"));
}

#[test]
fn test_format_with_unicode_filename() {
    let predictions = vec![(
        "src/archivo_espa\u{00f1}ol.rs".to_string(),
        create_medium_risk_score(),
    )];
    let elapsed = Duration::from_millis(50);

    let result = format_defect_json(&predictions, elapsed).unwrap();
    assert!(result.contains("archivo_espa\u{00f1}ol.rs"));
}

#[test]
fn test_format_with_zero_duration() {
    let predictions = create_test_predictions();
    let elapsed = Duration::from_millis(0);

    let result = format_defect_summary(&predictions, elapsed).unwrap();
    assert!(result.contains("Analysis time"));

    let json = format_defect_json(&predictions, elapsed).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["summary"]["analysis_time_ms"], 0);
}

#[test]
fn test_format_with_very_long_duration() {
    let predictions = create_test_predictions();
    let elapsed = Duration::from_secs(3600); // 1 hour

    let result = format_defect_summary(&predictions, elapsed).unwrap();
    assert!(result.contains("Analysis time"));
}

// ==================== Probability boundary tests ====================

#[test]
fn test_probability_exactly_zero() {
    let score = create_mock_defect_score(0.0, 0.9, RiskLevel::Low);
    let predictions = vec![("zero.rs".to_string(), score)];

    let stats = calculate_risk_statistics(&predictions);
    assert_eq!(stats.low_risk, 1);
    assert_eq!(stats.medium_risk, 0);
    assert_eq!(stats.high_risk, 0);
}

#[test]
fn test_probability_exactly_one() {
    let score = create_mock_defect_score(1.0, 0.9, RiskLevel::High);
    let predictions = vec![("max.rs".to_string(), score)];

    let stats = calculate_risk_statistics(&predictions);
    assert_eq!(stats.high_risk, 1);
    assert_eq!(stats.medium_risk, 0);
    assert_eq!(stats.low_risk, 0);
}
