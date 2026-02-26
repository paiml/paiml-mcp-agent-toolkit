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
