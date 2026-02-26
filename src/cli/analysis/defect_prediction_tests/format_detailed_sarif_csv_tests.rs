// Tests for format_defect_detailed, write_detailed_header, write_file_details,
// write_risk_level, write_confidence_level, write_contributing_factors,
// write_recommendations, write_analysis_footer, format_defect_sarif,
// and format_defect_csv.
// Included into mod tests via include!() - no use imports or #! attrs allowed.

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

#[test]
fn test_format_defect_sarif_empty() {
    let predictions: Vec<(String, DefectScore)> = vec![];

    let result = format_defect_sarif(&predictions).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    let results = parsed["runs"][0]["results"].as_array().unwrap();
    assert!(results.is_empty());
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

#[test]
fn test_format_defect_csv_empty_predictions() {
    let predictions: Vec<(String, DefectScore)> = vec![];

    let result = format_defect_csv(&predictions).unwrap();

    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 1); // Only header
}
