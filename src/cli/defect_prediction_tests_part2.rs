// Filter, risk distribution, and summary output tests
// Included from defect_prediction_tests.rs (inside mod tests) - shares test module scope

// ==================== Filter Predictions Tests ====================

#[test]
fn test_filter_predictions_empty() {
    let config = DefectPredictionConfig {
        confidence_threshold: 0.5,
        min_lines: 10,
        include_low_confidence: true,
        high_risk_only: false,
        include_recommendations: false,
        include: None,
        exclude: None,
    };
    let predictions: Vec<(String, DefectScore)> = vec![];
    let filtered = filter_predictions(predictions, &config);
    assert!(filtered.is_empty());
}

#[test]
fn test_filter_predictions_confidence_filter() {
    let config = DefectPredictionConfig {
        confidence_threshold: 0.8,
        min_lines: 10,
        include_low_confidence: false,
        high_risk_only: false,
        include_recommendations: false,
        include: None,
        exclude: None,
    };
    let predictions = vec![
        (
            "file1.rs".to_string(),
            DefectScore {
                probability: 0.5,
                confidence: 0.9,
                risk_level: RiskLevel::Medium,
                contributing_factors: vec![],
                recommendations: vec![],
            },
        ),
        (
            "file2.rs".to_string(),
            DefectScore {
                probability: 0.5,
                confidence: 0.3,
                risk_level: RiskLevel::Medium,
                contributing_factors: vec![],
                recommendations: vec![],
            },
        ),
    ];
    let filtered = filter_predictions(predictions, &config);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].0, "file1.rs");
}

#[test]
fn test_filter_predictions_high_risk_only() {
    let config = DefectPredictionConfig {
        confidence_threshold: 0.5,
        min_lines: 10,
        include_low_confidence: true,
        high_risk_only: true,
        include_recommendations: false,
        include: None,
        exclude: None,
    };
    let predictions = vec![
        (
            "file1.rs".to_string(),
            DefectScore {
                probability: 0.8,
                confidence: 0.9,
                risk_level: RiskLevel::High,
                contributing_factors: vec![],
                recommendations: vec![],
            },
        ),
        (
            "file2.rs".to_string(),
            DefectScore {
                probability: 0.4,
                confidence: 0.9,
                risk_level: RiskLevel::Medium,
                contributing_factors: vec![],
                recommendations: vec![],
            },
        ),
    ];
    let filtered = filter_predictions(predictions, &config);
    assert_eq!(filtered.len(), 1);
    assert!(filtered[0].1.probability >= 0.7);
}

#[test]
fn test_filter_predictions_sorted_by_probability() {
    let config = DefectPredictionConfig {
        confidence_threshold: 0.5,
        min_lines: 10,
        include_low_confidence: true,
        high_risk_only: false,
        include_recommendations: false,
        include: None,
        exclude: None,
    };
    let predictions = vec![
        (
            "file1.rs".to_string(),
            DefectScore {
                probability: 0.3,
                confidence: 0.9,
                risk_level: RiskLevel::Low,
                contributing_factors: vec![],
                recommendations: vec![],
            },
        ),
        (
            "file2.rs".to_string(),
            DefectScore {
                probability: 0.9,
                confidence: 0.9,
                risk_level: RiskLevel::High,
                contributing_factors: vec![],
                recommendations: vec![],
            },
        ),
        (
            "file3.rs".to_string(),
            DefectScore {
                probability: 0.6,
                confidence: 0.9,
                risk_level: RiskLevel::Medium,
                contributing_factors: vec![],
                recommendations: vec![],
            },
        ),
    ];
    let filtered = filter_predictions(predictions, &config);
    assert_eq!(filtered.len(), 3);
    assert_eq!(filtered[0].0, "file2.rs"); // Highest probability first
    assert_eq!(filtered[1].0, "file3.rs");
    assert_eq!(filtered[2].0, "file1.rs");
}

// ==================== Risk Distribution Tests ====================

#[test]
fn test_calculate_risk_distribution_empty() {
    let predictions: Vec<(String, DefectScore)> = vec![];
    let dist = calculate_risk_distribution(&predictions);
    assert_eq!(dist.high_risk_count, 0);
    assert_eq!(dist.medium_risk_count, 0);
    assert_eq!(dist.low_risk_count, 0);
}

#[test]
fn test_calculate_risk_distribution_all_levels() {
    let predictions = vec![
        (
            "high.rs".to_string(),
            DefectScore {
                probability: 0.8,
                confidence: 0.9,
                risk_level: RiskLevel::High,
                contributing_factors: vec![],
                recommendations: vec![],
            },
        ),
        (
            "medium.rs".to_string(),
            DefectScore {
                probability: 0.5,
                confidence: 0.9,
                risk_level: RiskLevel::Medium,
                contributing_factors: vec![],
                recommendations: vec![],
            },
        ),
        (
            "low.rs".to_string(),
            DefectScore {
                probability: 0.2,
                confidence: 0.9,
                risk_level: RiskLevel::Low,
                contributing_factors: vec![],
                recommendations: vec![],
            },
        ),
    ];
    let dist = calculate_risk_distribution(&predictions);
    assert_eq!(dist.high_risk_count, 1);
    assert_eq!(dist.medium_risk_count, 1);
    assert_eq!(dist.low_risk_count, 1);
}

// ==================== Summary Output Tests ====================

#[test]
fn test_format_summary_output_basic() {
    let predictions: Vec<(String, DefectScore)> = vec![];
    let dist = RiskDistribution {
        high_risk_count: 0,
        medium_risk_count: 0,
        low_risk_count: 0,
    };
    let output = format_summary_output(
        10,
        &predictions,
        &dist,
        true,
        std::time::Duration::from_millis(100),
    );
    assert!(output.contains("Defect Prediction Analysis Summary"));
    assert!(output.contains("Files analyzed: 10"));
}

#[test]
fn test_format_summary_output_with_predictions() {
    let predictions = vec![(
        "file.rs".to_string(),
        DefectScore {
            probability: 0.8,
            confidence: 0.9,
            risk_level: RiskLevel::High,
            contributing_factors: vec![("complexity".to_string(), 0.5)],
            recommendations: vec![],
        },
    )];
    let dist = RiskDistribution {
        high_risk_count: 1,
        medium_risk_count: 0,
        low_risk_count: 0,
    };
    let output = format_summary_output(
        5,
        &predictions,
        &dist,
        false,
        std::time::Duration::from_secs(1),
    );
    assert!(output.contains("High-Risk Files"));
}
