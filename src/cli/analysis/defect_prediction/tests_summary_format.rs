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
        assert!(result.contains("0 files")); // Risk distribution shows 0
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
            .map(|i| (format!("file{i}.rs"), create_high_risk_score()))
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
