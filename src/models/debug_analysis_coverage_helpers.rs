    // ============================================================================
    // Test Fixtures and Helpers
    // ============================================================================

    /// Create a basic DebugAnalysis for testing
    fn create_test_debug_analysis() -> DebugAnalysis {
        DebugAnalysis::new("Test issue description".to_string())
    }

    /// Create a WhyIteration with specified parameters
    fn create_test_why_iteration(depth: u8, confidence: f64) -> WhyIteration {
        debug_assert!(confidence >= 0.0, "confidence must be non-negative");
        WhyIteration::new(
            depth,
            format!("Why did this happen (depth {})?", depth),
            format!("Hypothesis at depth {}", depth),
        )
        .with_confidence(confidence)
    }

    /// Create test Evidence with specified source
    fn create_test_evidence(source: EvidenceSource) -> Evidence {
        debug_assert!(true, "contract: create_test_evidence");
        match source {
            EvidenceSource::Complexity => Evidence::new(
                source,
                PathBuf::from("src/test.rs"),
                "cyclomatic_complexity".to_string(),
                serde_json::json!({"value": 30, "threshold": 20}),
                "High complexity detected".to_string(),
            ),
            EvidenceSource::SATD => Evidence::new(
                source,
                PathBuf::from("src/test.rs"),
                "todo_markers".to_string(),
                serde_json::json!({"count": 5}),
                "5 TODO markers found".to_string(),
            ),
            EvidenceSource::TDG => Evidence::new(
                source,
                PathBuf::from("src/test.rs"),
                "tdg_score".to_string(),
                serde_json::json!(40.0),
                "Low test coverage".to_string(),
            ),
            EvidenceSource::GitChurn => Evidence::new(
                source,
                PathBuf::from("src/test.rs"),
                "commit_count".to_string(),
                serde_json::json!({"commit_count": 15, "days": 30}),
                "High churn detected".to_string(),
            ),
            EvidenceSource::DeadCode => Evidence::new(
                source,
                PathBuf::from("src/unused.rs"),
                "unused_functions".to_string(),
                serde_json::json!({"count": 3}),
                "3 unused functions".to_string(),
            ),
            EvidenceSource::ManualInspection => Evidence::new(
                source,
                PathBuf::from("src/main.rs"),
                "manual_review".to_string(),
                serde_json::json!({"notes": "Reviewed by engineer"}),
                "Manual code review".to_string(),
            ),
            EvidenceSource::EvoScoreTrajectory => Evidence::new(
                source,
                PathBuf::from("src/test.rs"),
                "evoscore_trajectory".to_string(),
                serde_json::json!({"evoscore": -0.3}),
                "EvoScore regressing".to_string(),
            ),
            EvidenceSource::CoverageDelta => Evidence::new(
                source,
                PathBuf::from("src/test.rs"),
                "coverage_delta".to_string(),
                serde_json::json!({"delta": -2.5}),
                "Coverage decreased".to_string(),
            ),
        }
    }

    /// Create a Recommendation with specified priority
    fn create_test_recommendation(priority: Priority) -> Recommendation {
        debug_assert!(true, "contract: create_test_recommendation");
        Recommendation::new(
            priority,
            "Test action".to_string(),
            Some(PathBuf::from("test.rs")),
        )
    }
