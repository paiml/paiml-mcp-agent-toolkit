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
