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
