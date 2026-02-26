// Tests for create_defect_prediction_config, calculate_risk_statistics,
// get_risk_icon, format_risk_level_display, and filter_and_sort_predictions.
// Included into mod tests via include!() - no use imports or #! attrs allowed.

// ==================== Test create_defect_prediction_config ====================

#[test]
fn test_create_defect_prediction_config_default_values() {
    let config = create_defect_prediction_config(0.5, 10, false, false, true, None, None);

    assert!((config.confidence_threshold - 0.5).abs() < f32::EPSILON);
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

    assert!((config.confidence_threshold - 0.7).abs() < f32::EPSILON);
    assert_eq!(config.min_lines, 50);
    assert!(config.include_low_confidence);
    assert!(config.high_risk_only);
    assert!(!config.include_recommendations);
    assert_eq!(config.include, Some("src/".to_string()));
    assert_eq!(config.exclude, Some("test/".to_string()));
}

#[test]
fn test_create_defect_prediction_config_edge_threshold() {
    let config = create_defect_prediction_config(0.0, 0, false, false, false, None, None);
    assert!((config.confidence_threshold).abs() < f32::EPSILON);
    assert_eq!(config.min_lines, 0);

    let config = create_defect_prediction_config(1.0, 10000, true, true, true, None, None);
    assert!((config.confidence_threshold - 1.0).abs() < f32::EPSILON);
    assert_eq!(config.min_lines, 10000);
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
fn test_calculate_risk_statistics_all_medium_risk() {
    let predictions = vec![
        ("file1.rs".to_string(), create_medium_risk_score()),
        ("file2.rs".to_string(), create_medium_risk_score()),
    ];
    let stats = calculate_risk_statistics(&predictions);

    assert_eq!(stats.high_risk, 0);
    assert_eq!(stats.medium_risk, 2);
    assert_eq!(stats.low_risk, 0);
}

#[test]
fn test_calculate_risk_statistics_all_low_risk() {
    let predictions = vec![
        ("file1.rs".to_string(), create_low_risk_score()),
        ("file2.rs".to_string(), create_low_risk_score()),
        ("file3.rs".to_string(), create_low_risk_score()),
        ("file4.rs".to_string(), create_low_risk_score()),
    ];
    let stats = calculate_risk_statistics(&predictions);

    assert_eq!(stats.high_risk, 0);
    assert_eq!(stats.medium_risk, 0);
    assert_eq!(stats.low_risk, 4);
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
    assert_eq!(get_risk_icon(&RiskLevel::High), "\u{1f534}");
}

#[test]
fn test_get_risk_icon_medium() {
    assert_eq!(get_risk_icon(&RiskLevel::Medium), "\u{1f7e1}");
}

#[test]
fn test_get_risk_icon_low() {
    assert_eq!(get_risk_icon(&RiskLevel::Low), "\u{1f7e2}");
}

// ==================== Test format_risk_level_display ====================

#[test]
fn test_format_risk_level_display_high() {
    let display = format_risk_level_display(&RiskLevel::High);
    assert!(display.contains("HIGH"));
}

#[test]
fn test_format_risk_level_display_medium() {
    let display = format_risk_level_display(&RiskLevel::Medium);
    assert!(display.contains("MEDIUM"));
}

#[test]
fn test_format_risk_level_display_low() {
    let display = format_risk_level_display(&RiskLevel::Low);
    assert!(display.contains("LOW"));
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

#[test]
fn test_filter_and_sort_predictions_empty_input() {
    let predictions: Vec<(String, DefectScore)> = vec![];
    let filtered = filter_and_sort_predictions(predictions, false, true, 0.0, 10);
    assert!(filtered.is_empty());
}

#[test]
fn test_filter_and_sort_predictions_combined_filters() {
    // Test combining high_risk_only AND confidence filtering
    let predictions = vec![
        (
            "high_low_conf.rs".to_string(),
            create_mock_defect_score(0.9, 0.3, RiskLevel::High),
        ),
        (
            "high_high_conf.rs".to_string(),
            create_mock_defect_score(0.85, 0.9, RiskLevel::High),
        ),
        (
            "med_high_conf.rs".to_string(),
            create_mock_defect_score(0.5, 0.9, RiskLevel::Medium),
        ),
    ];

    let filtered = filter_and_sort_predictions(
        predictions,
        true,  // high_risk_only
        false, // filter low confidence
        0.5,   // confidence_threshold
        10,
    );

    // Only high risk with high confidence should remain
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].0, "high_high_conf.rs");
}
