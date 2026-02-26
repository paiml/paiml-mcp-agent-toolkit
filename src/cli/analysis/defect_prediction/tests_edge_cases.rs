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
