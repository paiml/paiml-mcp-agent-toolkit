// Tests for RiskStatistics struct, edge cases, probability boundary tests,
// additional format_defect_output and detailed/json/sarif/csv verification tests.
// Included into mod tests via include!() - no use imports or #! attrs allowed.

// ==================== Test RiskStatistics struct ====================

#[test]
fn test_risk_statistics_struct_construction() {
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

// ==================== Additional format_defect_output tests ====================

#[test]
fn test_format_defect_output_all_formats_empty_predictions() {
    let predictions: Vec<(String, DefectScore)> = vec![];
    let elapsed = Duration::from_millis(50);

    // Test all formats with empty predictions
    assert!(format_defect_output(
        DefectPredictionOutputFormat::Summary,
        &predictions,
        elapsed,
        false
    )
    .is_ok());
    assert!(format_defect_output(
        DefectPredictionOutputFormat::Json,
        &predictions,
        elapsed,
        false
    )
    .is_ok());
    assert!(format_defect_output(
        DefectPredictionOutputFormat::Detailed,
        &predictions,
        elapsed,
        true
    )
    .is_ok());
    assert!(format_defect_output(
        DefectPredictionOutputFormat::Sarif,
        &predictions,
        elapsed,
        false
    )
    .is_ok());
    assert!(format_defect_output(
        DefectPredictionOutputFormat::Csv,
        &predictions,
        elapsed,
        false
    )
    .is_ok());
}

// ==================== Detailed format tests ====================

#[test]
fn test_format_defect_detailed_empty() {
    let predictions: Vec<(String, DefectScore)> = vec![];
    let elapsed = Duration::from_millis(100);

    let result = format_defect_detailed(&predictions, elapsed, true).unwrap();

    assert!(result.contains("Detailed Report"));
    assert!(result.contains("Analysis time"));
}

#[test]
fn test_format_defect_detailed_many_files() {
    let predictions: Vec<_> = (0..50)
        .map(|i| (format!("file{i}.rs"), create_medium_risk_score()))
        .collect();
    let elapsed = Duration::from_millis(500);

    let result = format_defect_detailed(&predictions, elapsed, false).unwrap();

    // Verify all files are included
    assert!(result.contains("file0.rs"));
    assert!(result.contains("file49.rs"));
}

// ==================== JSON format verification ====================

#[test]
fn test_format_defect_json_contributing_factors() {
    let predictions = create_test_predictions();
    let elapsed = Duration::from_millis(100);

    let result = format_defect_json(&predictions, elapsed).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    let pred = &parsed["predictions"][0];
    let factors = pred["contributing_factors"].as_array().unwrap();

    // Should have 4 factors
    assert_eq!(factors.len(), 4);
}

#[test]
fn test_format_defect_json_recommendations() {
    let predictions = create_test_predictions();
    let elapsed = Duration::from_millis(100);

    let result = format_defect_json(&predictions, elapsed).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    let pred = &parsed["predictions"][0];
    let recs = pred["recommendations"].as_array().unwrap();

    // Should have 2 recommendations
    assert_eq!(recs.len(), 2);
}

// ==================== SARIF format verification ====================

#[test]
fn test_format_defect_sarif_properties() {
    let predictions = create_test_predictions();

    let result = format_defect_sarif(&predictions).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    let result_item = &parsed["runs"][0]["results"][0];
    let props = &result_item["properties"];

    assert!(props["probability"].is_f64());
    assert!(props["confidence"].is_f64());
    assert!(props["contributing_factors"].is_array());
    assert!(props["recommendations"].is_array());
}

#[test]
fn test_format_defect_sarif_locations() {
    let predictions = create_test_predictions();

    let result = format_defect_sarif(&predictions).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    let result_item = &parsed["runs"][0]["results"][0];
    let location = &result_item["locations"][0];

    assert!(location["physicalLocation"]["artifactLocation"]["uri"].is_string());
    assert_eq!(
        location["physicalLocation"]["artifactLocation"]["uriBaseId"],
        "%SRCROOT%"
    );
}

// ==================== CSV format edge cases ====================

#[test]
fn test_format_defect_csv_with_comma_in_factor() {
    let mut score = create_high_risk_score();
    score.contributing_factors = vec![("factor,with,commas".to_string(), 0.5)];
    let predictions = vec![("test.rs".to_string(), score)];

    let result = format_defect_csv(&predictions).unwrap();

    // CSV should still be parseable
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 2);
}

// ==================== Risk level calculation tests ====================

#[test]
fn test_risk_statistics_with_various_probabilities() {
    let predictions = vec![
        (
            "p_0.rs".to_string(),
            create_mock_defect_score(0.0, 0.9, RiskLevel::Low),
        ),
        (
            "p_15.rs".to_string(),
            create_mock_defect_score(0.15, 0.9, RiskLevel::Low),
        ),
        (
            "p_30.rs".to_string(),
            create_mock_defect_score(0.30, 0.9, RiskLevel::Low),
        ), // Exactly 0.3 -> Low
        (
            "p_35.rs".to_string(),
            create_mock_defect_score(0.35, 0.9, RiskLevel::Medium),
        ),
        (
            "p_50.rs".to_string(),
            create_mock_defect_score(0.50, 0.9, RiskLevel::Medium),
        ),
        (
            "p_70.rs".to_string(),
            create_mock_defect_score(0.70, 0.9, RiskLevel::Medium),
        ), // Exactly 0.7 -> Medium
        (
            "p_75.rs".to_string(),
            create_mock_defect_score(0.75, 0.9, RiskLevel::High),
        ),
        (
            "p_100.rs".to_string(),
            create_mock_defect_score(1.0, 0.9, RiskLevel::High),
        ),
    ];

    let stats = calculate_risk_statistics(&predictions);

    // High risk: > 0.7 (0.75, 1.0) = 2
    assert_eq!(stats.high_risk, 2);
    // Medium risk: > 0.3 and <= 0.7 (0.35, 0.50, 0.70) = 3
    assert_eq!(stats.medium_risk, 3);
    // Low risk: <= 0.3 (0.0, 0.15, 0.30) = 3
    assert_eq!(stats.low_risk, 3);
}
