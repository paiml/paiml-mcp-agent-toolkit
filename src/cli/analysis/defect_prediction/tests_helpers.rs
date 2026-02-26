    // Helper function to create mock DefectScore for testing
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
