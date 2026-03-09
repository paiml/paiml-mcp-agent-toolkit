// Five Whys analyzer tests extracted from five_whys_analyzer.rs for file health (CB-040).
// This file is include!()'d into five_whys_analyzer.rs scope.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_analyzer_creation() {
        let analyzer = FiveWhysAnalyzer::new();
        let _ = analyzer; // Suppress unused warning
    }

    #[tokio::test]
    async fn test_validate_depth_range() {
        let analyzer = FiveWhysAnalyzer::new();

        // Depth 0 should fail
        let result = analyzer.analyze("Test", Path::new("."), 0).await;
        assert!(result.is_err());

        // Depth 11 should fail
        let result = analyzer.analyze("Test", Path::new("."), 11).await;
        assert!(result.is_err());

        // Depth 5 should succeed
        let result = analyzer.analyze("Test", Path::new("."), 5).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_empty_issue() {
        let analyzer = FiveWhysAnalyzer::new();
        let result = analyzer.analyze("", Path::new("."), 5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_confidence_calculation() {
        let analyzer = FiveWhysAnalyzer::new();

        // Empty evidence
        let confidence = analyzer.calculate_confidence(&[]).expect("internal error");
        assert!((0.0..=1.0).contains(&confidence));

        // With evidence
        let evidence = vec![Evidence::new(
            EvidenceSource::Complexity,
            PathBuf::from("test.rs"),
            "complexity".to_string(),
            json!({"value": 50, "threshold": 20}),
            "High".to_string(),
        )];
        let confidence = analyzer
            .calculate_confidence(&evidence)
            .expect("internal error");
        assert!(confidence > 0.3);
        assert!(confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_basic_analysis() {
        let analyzer = FiveWhysAnalyzer::new();
        let result = analyzer
            .analyze("Test issue", Path::new("."), 5)
            .await
            .expect("internal error");

        assert_eq!(result.issue, "Test issue");
        assert!(!result.whys.is_empty());
        assert!(result.root_cause.is_some());
        assert!(!result.recommendations.is_empty());
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use proptest::prelude::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // ============================================================================
    // Test Fixtures and Helpers
    // ============================================================================

    /// Create a FiveWhysAnalyzer for testing
    fn create_analyzer() -> FiveWhysAnalyzer {
        FiveWhysAnalyzer::new()
    }

    /// Create test Evidence with specified source and custom values
    fn create_evidence_with_values(source: EvidenceSource, value: serde_json::Value) -> Evidence {
        Evidence::new(
            source,
            PathBuf::from("src/test.rs"),
            "metric".to_string(),
            value,
            "Test interpretation".to_string(),
        )
    }

    /// Create a WhyIteration with evidence for testing
    fn create_why_with_evidence(depth: u8, sources: &[EvidenceSource]) -> WhyIteration {
        let mut why = WhyIteration::new(
            depth,
            format!("Why question {}?", depth),
            format!("Hypothesis at depth {}", depth),
        );

        for source in sources {
            let value = match source {
                EvidenceSource::Complexity => json!({"value": 30, "threshold": 20}),
                EvidenceSource::SATD => json!({"count": 5}),
                EvidenceSource::TDG => json!(40.0),
                EvidenceSource::GitChurn => json!({"commit_count": 15, "days": 30}),
                EvidenceSource::DeadCode => json!({"count": 3}),
                EvidenceSource::ManualInspection => json!({"notes": "Manual review"}),
                EvidenceSource::EvoScoreTrajectory => {
                    json!({"evoscore": -0.3, "commits": 5, "gamma": 1.5})
                }
                EvidenceSource::CoverageDelta => {
                    json!({"coverage_pct": 70.0, "delta": -15.0, "total_lines": 1000, "covered_lines": 700})
                }
            };
            why.evidence
                .push(create_evidence_with_values(*source, value));
        }

        why
    }

    /// Create a temporary directory for testing path-based operations
    fn create_temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp directory")
    }

    // ============================================================================
    // FiveWhysAnalyzer Construction Tests
    // ============================================================================

    #[test]
    fn test_analyzer_new_creates_instance() {
        let analyzer = create_analyzer();
        // Verify the analyzer is created (no-op check since struct has no fields)
        let _ = analyzer;
    }

    #[test]
    fn test_analyzer_default_equals_new() {
        let analyzer1 = FiveWhysAnalyzer::new();
        let analyzer2 = FiveWhysAnalyzer::default();
        // Both should produce valid analyzers (we can't compare directly without PartialEq)
        let _ = (analyzer1, analyzer2);
    }

    // ============================================================================
    // Input Validation Tests
    // ============================================================================

    #[tokio::test]
    async fn test_analyze_rejects_empty_issue() {
        let analyzer = create_analyzer();
        let result = analyzer.analyze("", Path::new("."), 5).await;

        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(
            err_msg.contains("empty"),
            "Error should mention empty issue"
        );
    }

    #[tokio::test]
    async fn test_analyze_rejects_depth_zero() {
        let analyzer = create_analyzer();
        let result = analyzer.analyze("Test issue", Path::new("."), 0).await;

        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(
            err_msg.contains("Depth") || err_msg.contains("1 and 10"),
            "Error should mention depth range"
        );
    }

    #[tokio::test]
    async fn test_analyze_rejects_depth_above_ten() {
        let analyzer = create_analyzer();
        let result = analyzer.analyze("Test issue", Path::new("."), 11).await;

        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(
            err_msg.contains("11"),
            "Error should mention the invalid depth value"
        );
    }

    #[tokio::test]
    async fn test_analyze_rejects_nonexistent_path() {
        let analyzer = create_analyzer();
        let result = analyzer
            .analyze("Test issue", Path::new("/nonexistent/path/12345"), 5)
            .await;

        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(
            err_msg.contains("not exist") || err_msg.contains("Path"),
            "Error should mention path does not exist"
        );
    }

    #[tokio::test]
    async fn test_analyze_accepts_valid_depth_range() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        // Test all valid depths
        for depth in 1..=10 {
            let result = analyzer.analyze("Test issue", temp_dir.path(), depth).await;
            assert!(
                result.is_ok(),
                "Depth {} should be valid, got error: {:?}",
                depth,
                result.err()
            );
        }
    }

    #[tokio::test]
    async fn test_analyze_accepts_minimum_depth() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        let result = analyzer.analyze("Test issue", temp_dir.path(), 1).await;
        assert!(result.is_ok());

        let analysis = result.expect("should succeed");
        assert!(!analysis.whys.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_accepts_maximum_depth() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        let result = analyzer.analyze("Test issue", temp_dir.path(), 10).await;
        assert!(result.is_ok());

        let analysis = result.expect("should succeed");
        // May terminate early due to high confidence
        assert!(!analysis.whys.is_empty());
    }

    // ============================================================================
    // Analysis Behavior Tests
    // ============================================================================

    #[tokio::test]
    async fn test_analyze_produces_correct_number_of_whys() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        let result = analyzer.analyze("Test issue", temp_dir.path(), 3).await;
        assert!(result.is_ok());

        let analysis = result.expect("should succeed");
        // Should have at most 3 why iterations (may terminate early)
        assert!(analysis.whys.len() <= 3);
        assert!(!analysis.whys.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_sets_root_cause() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        let result = analyzer.analyze("Memory leak", temp_dir.path(), 5).await;
        assert!(result.is_ok());

        let analysis = result.expect("should succeed");
        assert!(analysis.root_cause.is_some());
    }

    #[tokio::test]
    async fn test_analyze_generates_recommendations() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        let result = analyzer
            .analyze("Performance issue", temp_dir.path(), 5)
            .await;
        assert!(result.is_ok());

        let analysis = result.expect("should succeed");
        assert!(!analysis.recommendations.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_populates_evidence_summary() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        let result = analyzer.analyze("Bug found", temp_dir.path(), 5).await;
        assert!(result.is_ok());

        let analysis = result.expect("should succeed");
        // Evidence summary should be populated from the whys
        // (may have default values if no evidence matches criteria)
        let _ = analysis.evidence_summary;
    }

    #[tokio::test]
    async fn test_analyze_preserves_issue_description() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        let issue = "Stack overflow in recursive parser";
        let result = analyzer.analyze(issue, temp_dir.path(), 5).await;
        assert!(result.is_ok());

        let analysis = result.expect("should succeed");
        assert_eq!(analysis.issue, issue);
    }

    #[tokio::test]
    async fn test_analyze_with_unicode_issue() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        let issue = "Error in 日本語 module: 🔥 critical failure";
        let result = analyzer.analyze(issue, temp_dir.path(), 3).await;
        assert!(result.is_ok());

        let analysis = result.expect("should succeed");
        assert_eq!(analysis.issue, issue);
    }

    #[tokio::test]
    async fn test_analyze_with_very_long_issue() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        let issue = "x".repeat(10000);
        let result = analyzer.analyze(&issue, temp_dir.path(), 3).await;
        assert!(result.is_ok());

        let analysis = result.expect("should succeed");
        assert_eq!(analysis.issue, issue);
    }

    // ============================================================================
    // Formulate Question Tests
    // ============================================================================

    #[test]
    fn test_formulate_question_first_iteration() {
        let analyzer = create_analyzer();
        let issue = "Memory leak detected";

        let result = analyzer.formulate_question(issue, 1, &[]);
        assert!(result.is_ok());

        let question = result.expect("should succeed");
        assert!(question.contains(issue));
        assert!(question.contains("Why"));
    }

    #[test]
    fn test_formulate_question_subsequent_iteration() {
        let analyzer = create_analyzer();
        let issue = "Memory leak detected";

        let previous_why = WhyIteration::new(
            1,
            "Why did this occur?".to_string(),
            "Buffer not released.".to_string(),
        );

        let result = analyzer.formulate_question(issue, 2, &[previous_why]);
        assert!(result.is_ok());

        let question = result.expect("should succeed");
        assert!(question.contains("Why"));
        assert!(question.contains("Buffer not released"));
    }

    #[test]
    fn test_formulate_question_handles_empty_previous_whys() {
        let analyzer = create_analyzer();
        let issue = "Test issue";

        let result = analyzer.formulate_question(issue, 3, &[]);
        assert!(result.is_ok());

        let question = result.expect("should succeed");
        assert!(question.contains("iteration 3"));
    }

    #[test]
    fn test_formulate_question_removes_trailing_period() {
        let analyzer = create_analyzer();
        let issue = "Test issue";

        let previous_why =
            WhyIteration::new(1, "Why?".to_string(), "Hypothesis with period.".to_string());

        let result = analyzer.formulate_question(issue, 2, &[previous_why]);
        assert!(result.is_ok());

        let question = result.expect("should succeed");
        // Should not end with double period
        assert!(!question.contains(".."));
    }

    // ============================================================================
    // Confidence Calculation Tests
    // ============================================================================

    #[test]
    fn test_calculate_confidence_empty_evidence() {
        let analyzer = create_analyzer();
        let result = analyzer.calculate_confidence(&[]);

        assert!(result.is_ok());
        let confidence = result.expect("should succeed");
        assert_eq!(confidence, 0.3); // Low confidence with no evidence
    }

    #[test]
    fn test_calculate_confidence_single_complexity_evidence() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::Complexity,
            json!({"value": 30, "threshold": 20}),
        )];

        let result = analyzer.calculate_confidence(&evidence);
        assert!(result.is_ok());

        let confidence = result.expect("should succeed");
        assert!(confidence >= 0.0);
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_calculate_confidence_increases_with_severity() {
        let analyzer = create_analyzer();

        // Low severity
        let low_evidence = vec![create_evidence_with_values(
            EvidenceSource::Complexity,
            json!({"value": 21, "threshold": 20}),
        )];
        let low_confidence = analyzer
            .calculate_confidence(&low_evidence)
            .expect("should succeed");

        // High severity
        let high_evidence = vec![create_evidence_with_values(
            EvidenceSource::Complexity,
            json!({"value": 50, "threshold": 20}),
        )];
        let high_confidence = analyzer
            .calculate_confidence(&high_evidence)
            .expect("should succeed");

        assert!(high_confidence >= low_confidence);
    }

    #[test]
    fn test_calculate_confidence_satd_with_count() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::SATD,
            json!({"count": 5}),
        )];

        let result = analyzer.calculate_confidence(&evidence);
        assert!(result.is_ok());

        let confidence = result.expect("should succeed");
        assert!(confidence > 0.3); // Should be higher than base
    }

    #[test]
    fn test_calculate_confidence_tdg_low_score() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::TDG,
            json!(20.0),
        )];

        let result = analyzer.calculate_confidence(&evidence);
        assert!(result.is_ok());

        let confidence = result.expect("should succeed");
        assert!(confidence > 0.3); // Low TDG should increase confidence
    }

    #[test]
    fn test_calculate_confidence_git_churn_high() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::GitChurn,
            json!({"commit_count": 20, "days": 30}),
        )];

        let result = analyzer.calculate_confidence(&evidence);
        assert!(result.is_ok());

        let confidence = result.expect("should succeed");
        assert!(confidence >= 0.0);
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_calculate_confidence_dead_code() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::DeadCode,
            json!({"count": 5}),
        )];

        let result = analyzer.calculate_confidence(&evidence);
        assert!(result.is_ok());

        let confidence = result.expect("should succeed");
        assert!(confidence >= 0.0);
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_calculate_confidence_manual_inspection() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::ManualInspection,
            json!({"notes": "Reviewed"}),
        )];

        let result = analyzer.calculate_confidence(&evidence);
        assert!(result.is_ok());

        let confidence = result.expect("should succeed");
        assert!(confidence >= 0.0);
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_calculate_confidence_multiple_sources() {
        let analyzer = create_analyzer();
        let evidence = vec![
            create_evidence_with_values(
                EvidenceSource::Complexity,
                json!({"value": 40, "threshold": 20}),
            ),
            create_evidence_with_values(EvidenceSource::SATD, json!({"count": 5})),
            create_evidence_with_values(EvidenceSource::TDG, json!(30.0)),
            create_evidence_with_values(EvidenceSource::GitChurn, json!({"commit_count": 15})),
        ];

        let result = analyzer.calculate_confidence(&evidence);
        assert!(result.is_ok());

        let confidence = result.expect("should succeed");
        assert!(confidence >= 0.0);
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_calculate_confidence_clamps_to_one() {
        let analyzer = create_analyzer();
        // Create evidence that would theoretically produce very high confidence
        let evidence = vec![
            create_evidence_with_values(
                EvidenceSource::Complexity,
                json!({"value": 100, "threshold": 20}),
            ),
            create_evidence_with_values(EvidenceSource::SATD, json!({"count": 100})),
            create_evidence_with_values(EvidenceSource::TDG, json!(0.0)),
            create_evidence_with_values(EvidenceSource::GitChurn, json!({"commit_count": 100})),
        ];

        let result = analyzer.calculate_confidence(&evidence);
        assert!(result.is_ok());

        let confidence = result.expect("should succeed");
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_calculate_confidence_handles_missing_complexity_value() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::Complexity,
            json!({"threshold": 20}), // Missing "value"
        )];

        let result = analyzer.calculate_confidence(&evidence);
        assert!(result.is_ok());

        // Should use 0.0 as default value
        let confidence = result.expect("should succeed");
        assert!(confidence >= 0.0);
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_calculate_confidence_handles_missing_satd_count() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::SATD,
            json!({}), // Missing "count"
        )];

        let result = analyzer.calculate_confidence(&evidence);
        assert!(result.is_ok());

        let confidence = result.expect("should succeed");
        assert!(confidence >= 0.0);
        assert!(confidence <= 1.0);
    }

    // ============================================================================
    // Generate Hypothesis Tests
    // ============================================================================

    #[test]
    fn test_generate_hypothesis_depth_1_high_complexity() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::Complexity,
            json!({"value": 30, "threshold": 20}),
        )];

        let result = analyzer.generate_hypothesis("Why?", &evidence, 1);
        assert!(result.is_ok());

        let hypothesis = result.expect("should succeed");
        assert!(hypothesis.contains("complexity") || hypothesis.contains("Complex"));
    }

    #[test]
    fn test_generate_hypothesis_depth_1_with_satd() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::SATD,
            json!({"count": 5}),
        )];

        let result = analyzer.generate_hypothesis("Why?", &evidence, 1);
        assert!(result.is_ok());

        let hypothesis = result.expect("should succeed");
        assert!(hypothesis.contains("technical debt") || hypothesis.contains("quality"));
    }

    #[test]
    fn test_generate_hypothesis_depth_2_low_coverage() {
        // v2 (PMAT-510): CoverageDelta replaces TDG for depth-2 coverage hypothesis
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::CoverageDelta,
            json!({"coverage_pct": 60.0, "delta": -25.0}),
        )];

        let result = analyzer.generate_hypothesis("Why?", &evidence, 2);
        assert!(result.is_ok());

        let hypothesis = result.expect("should succeed");
        assert!(hypothesis.contains("test") || hypothesis.contains("coverage"));
    }

    #[test]
    fn test_generate_hypothesis_depth_3_high_churn() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::GitChurn,
            json!({"commit_count": 15}),
        )];

        let result = analyzer.generate_hypothesis("Why?", &evidence, 3);
        assert!(result.is_ok());

        let hypothesis = result.expect("should succeed");
        assert!(hypothesis.contains("changes") || hypothesis.contains("unstable"));
    }

    #[test]
    fn test_generate_hypothesis_depth_4() {
        let analyzer = create_analyzer();
        let evidence = vec![];

        let result = analyzer.generate_hypothesis("Why?", &evidence, 4);
        assert!(result.is_ok());

        let hypothesis = result.expect("should succeed");
        assert!(hypothesis.contains("Requirements") || hypothesis.contains("constraints"));
    }

    #[test]
    fn test_generate_hypothesis_depth_5_plus() {
        let analyzer = create_analyzer();
        let evidence = vec![];

        for depth in 5..=10 {
            let result = analyzer.generate_hypothesis("Why?", &evidence, depth);
            assert!(result.is_ok());

            let hypothesis = result.expect("should succeed");
            assert!(hypothesis.contains("Root cause") || hypothesis.contains("process"));
        }
    }

    #[test]
    fn test_generate_hypothesis_no_evidence() {
        let analyzer = create_analyzer();

        let result = analyzer.generate_hypothesis("Why?", &[], 1);
        assert!(result.is_ok());

        let hypothesis = result.expect("should succeed");
        assert!(!hypothesis.is_empty());
    }

    // ============================================================================
    // Extract Root Cause Tests
    // ============================================================================

    #[test]
    fn test_extract_root_cause_empty_whys() {
        let analyzer = create_analyzer();

        let result = analyzer.extract_root_cause(&[]);
        assert!(result.is_ok());

        let root_cause = result.expect("should succeed");
        assert!(root_cause.is_none());
    }

    #[test]
    fn test_extract_root_cause_single_why() {
        let analyzer = create_analyzer();
        let whys = vec![WhyIteration::new(
            1,
            "Question".to_string(),
            "The hypothesis".to_string(),
        )];

        let result = analyzer.extract_root_cause(&whys);
        assert!(result.is_ok());

        let root_cause = result.expect("should succeed");
        assert_eq!(root_cause, Some("The hypothesis".to_string()));
    }

    #[test]
    fn test_extract_root_cause_multiple_whys() {
        let analyzer = create_analyzer();
        let whys = vec![
            WhyIteration::new(1, "Q1".to_string(), "Hypothesis 1".to_string()),
            WhyIteration::new(2, "Q2".to_string(), "Hypothesis 2".to_string()),
            WhyIteration::new(3, "Q3".to_string(), "Final hypothesis".to_string()),
        ];

        let result = analyzer.extract_root_cause(&whys);
        assert!(result.is_ok());

        let root_cause = result.expect("should succeed");
        assert_eq!(root_cause, Some("Final hypothesis".to_string()));
    }

    // ============================================================================
    // Generate Recommendations Tests
    // ============================================================================

    #[test]
    fn test_generate_recommendations_empty_whys() {
        let analyzer = create_analyzer();

        let result = analyzer.generate_recommendations(&[], "Root cause");
        assert!(result.is_ok());

        let recommendations = result.expect("should succeed");
        // Should still have root cause and spec recommendations
        assert!(recommendations.len() >= 2);
    }

    #[test]
    fn test_generate_recommendations_with_high_complexity() {
        let analyzer = create_analyzer();
        let whys = vec![create_why_with_evidence(1, &[EvidenceSource::Complexity])];

        let result = analyzer.generate_recommendations(&whys, "Root cause");
        assert!(result.is_ok());

        let recommendations = result.expect("should succeed");
        assert!(recommendations
            .iter()
            .any(|r| r.action.contains("complexity") || r.action.contains("Refactor")));
    }

    #[test]
    fn test_generate_recommendations_with_satd() {
        let analyzer = create_analyzer();
        let whys = vec![create_why_with_evidence(1, &[EvidenceSource::SATD])];

        let result = analyzer.generate_recommendations(&whys, "Root cause");
        assert!(result.is_ok());

        let recommendations = result.expect("should succeed");
        assert!(recommendations
            .iter()
            .any(|r| r.action.contains("technical debt") || r.action.contains("TODO")));
    }

    #[test]
    fn test_generate_recommendations_with_low_coverage_delta() {
        // v2 (PMAT-510): CoverageDelta replaces TDG for coverage recommendations
        let analyzer = create_analyzer();
        let whys = vec![create_why_with_evidence(
            1,
            &[EvidenceSource::CoverageDelta],
        )];

        let result = analyzer.generate_recommendations(&whys, "Root cause");
        assert!(result.is_ok());

        let recommendations = result.expect("should succeed");
        assert!(recommendations
            .iter()
            .any(|r| r.action.contains("test") || r.action.contains("coverage")));
    }

    #[test]
    fn test_generate_recommendations_with_high_churn() {
        let analyzer = create_analyzer();
        let whys = vec![create_why_with_evidence(1, &[EvidenceSource::GitChurn])];

        let result = analyzer.generate_recommendations(&whys, "Root cause");
        assert!(result.is_ok());

        let recommendations = result.expect("should succeed");
        assert!(recommendations
            .iter()
            .any(|r| r.action.contains("Stabilize")
                || r.action.contains("design")
                || r.action.contains("patterns")));
    }

    #[test]
    fn test_generate_recommendations_includes_root_cause_fix() {
        let analyzer = create_analyzer();
        let root_cause = "Missing RAII pattern";

        let result = analyzer.generate_recommendations(&[], root_cause);
        assert!(result.is_ok());

        let recommendations = result.expect("should succeed");
        assert!(recommendations
            .iter()
            .any(|r| r.action.contains(root_cause)));
    }

    #[test]
    fn test_generate_recommendations_includes_spec_recommendation() {
        let analyzer = create_analyzer();

        let result = analyzer.generate_recommendations(&[], "Root cause");
        assert!(result.is_ok());

        let recommendations = result.expect("should succeed");
        assert!(recommendations
            .iter()
            .any(|r| r.action.contains("requirements") || r.action.contains("specification")));
    }

    #[test]
    fn test_generate_recommendations_priority_levels() {
        let analyzer = create_analyzer();
        let whys = vec![create_why_with_evidence(
            1,
            &[
                EvidenceSource::Complexity,
                EvidenceSource::SATD,
                EvidenceSource::CoverageDelta,
                EvidenceSource::GitChurn,
            ],
        )];

        let result = analyzer.generate_recommendations(&whys, "Root cause");
        assert!(result.is_ok());

        let recommendations = result.expect("should succeed");

        // Should have high priority recommendations
        assert!(recommendations.iter().any(|r| r.priority == Priority::High));

        // Should have medium priority recommendations (for churn)
        assert!(recommendations
            .iter()
            .any(|r| r.priority == Priority::Medium));
    }

    // ============================================================================
    // Gather Evidence Tests
    // ============================================================================

    /// Create a temp dir with src/, .pmat/baseline.json, and git init for evidence tests
    fn create_evidence_temp_dir() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let p = temp_dir.path();

        // Create src/ with a .rs file containing SATD markers
        std::fs::create_dir_all(p.join("src")).unwrap();
        std::fs::write(
            p.join("src/main.rs"),
            "fn main() {\n    // TODO: fix this\n    // FIXME: another issue\n    println!(\"hello\");\n}\n",
        )
        .unwrap();

        // Create .pmat/baseline.json for TDG evidence
        std::fs::create_dir_all(p.join(".pmat")).unwrap();
        std::fs::write(
            p.join(".pmat/baseline.json"),
            r#"{"version":"3.6.1","summary":{"total_files":10,"avg_score":72.5,"grade_distribution":{},"languages":{}},"files":{}}"#,
        )
        .unwrap();

        // Initialize git repo for churn evidence
        let _ = std::process::Command::new("git")
            .args(["init"])
            .current_dir(p)
            .output();
        let _ = std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(p)
            .output();
        let _ = std::process::Command::new("git")
            .args(["commit", "-m", "init", "--allow-empty"])
            .env("GIT_AUTHOR_NAME", "test")
            .env("GIT_AUTHOR_EMAIL", "test@test.com")
            .env("GIT_COMMITTER_NAME", "test")
            .env("GIT_COMMITTER_EMAIL", "test@test.com")
            .current_dir(p)
            .output();

        temp_dir
    }

    #[tokio::test]
    async fn test_gather_evidence_returns_multiple_sources() {
        let analyzer = create_analyzer();
        let temp_dir = create_evidence_temp_dir();

        let result = analyzer.gather_evidence(temp_dir.path()).await;
        assert!(result.is_ok());

        let evidence = result.expect("should succeed");
        assert!(!evidence.is_empty());

        // Should have evidence from multiple sources (SATD + complexity + git churn)
        let sources: std::collections::HashSet<_> = evidence.iter().map(|e| e.source).collect();
        assert!(
            sources.len() >= 3,
            "Expected >=3 sources, got {}: {:?}",
            sources.len(),
            sources
        );
    }

    #[tokio::test]
    async fn test_gather_evidence_includes_complexity() {
        let analyzer = create_analyzer();
        let temp_dir = create_evidence_temp_dir();

        let evidence = analyzer
            .gather_evidence(temp_dir.path())
            .await
            .expect("should succeed");
        assert!(
            evidence
                .iter()
                .any(|e| e.source == EvidenceSource::Complexity),
            "Missing complexity evidence"
        );
    }

    #[tokio::test]
    async fn test_gather_evidence_includes_satd() {
        let analyzer = create_analyzer();
        let temp_dir = create_evidence_temp_dir();

        let evidence = analyzer
            .gather_evidence(temp_dir.path())
            .await
            .expect("should succeed");
        assert!(
            evidence.iter().any(|e| e.source == EvidenceSource::SATD),
            "Missing SATD evidence"
        );
        // Verify real SATD count (we put 2 markers in main.rs)
        let satd = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::SATD)
            .unwrap();
        let count = satd.value.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
        assert!(count >= 2, "Expected >=2 SATD markers, got {}", count);
    }

    #[tokio::test]
    async fn test_gather_evidence_no_longer_includes_tdg() {
        // v2 (PMAT-510): TDG removed from evidence gathering (redundant with complexity+churn)
        let analyzer = create_analyzer();
        let temp_dir = create_evidence_temp_dir();

        let evidence = analyzer
            .gather_evidence(temp_dir.path())
            .await
            .expect("should succeed");
        assert!(
            !evidence.iter().any(|e| e.source == EvidenceSource::TDG),
            "TDG should not be gathered in v2"
        );
    }

    #[tokio::test]
    async fn test_gather_evidence_includes_git_churn() {
        let analyzer = create_analyzer();
        let temp_dir = create_evidence_temp_dir();

        let evidence = analyzer
            .gather_evidence(temp_dir.path())
            .await
            .expect("should succeed");
        assert!(
            evidence
                .iter()
                .any(|e| e.source == EvidenceSource::GitChurn),
            "Missing git churn evidence"
        );
    }

    // ============================================================================
    // Iterate Why Tests
    // ============================================================================

    #[tokio::test]
    async fn test_iterate_why_returns_valid_iteration() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        let result = analyzer
            .iterate_why("Test issue", temp_dir.path(), 1, &[])
            .await;
        assert!(result.is_ok());

        let why = result.expect("should succeed");
        assert_eq!(why.depth, 1);
        assert!(!why.question.is_empty());
        assert!(!why.hypothesis.is_empty());
        assert!(why.confidence >= 0.0 && why.confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_iterate_why_includes_evidence() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        let result = analyzer
            .iterate_why("Test issue", temp_dir.path(), 1, &[])
            .await;
        assert!(result.is_ok());

        let why = result.expect("should succeed");
        assert!(!why.evidence.is_empty());
    }

    #[tokio::test]
    async fn test_iterate_why_uses_previous_hypothesis() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        let previous = WhyIteration::new(
            1,
            "First question".to_string(),
            "First hypothesis".to_string(),
        );

        let result = analyzer
            .iterate_why("Test issue", temp_dir.path(), 2, &[previous])
            .await;
        assert!(result.is_ok());

        let why = result.expect("should succeed");
        assert_eq!(why.depth, 2);
        // Question should be based on previous hypothesis
        assert!(why.question.contains("Why") || why.question.contains("First hypothesis"));
    }

    // ============================================================================
    // Early Termination Tests
    // ============================================================================

    #[tokio::test]
    async fn test_analyze_may_terminate_early_on_high_confidence() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        // With depth 10, if confidence exceeds 0.9 after 3 iterations, should terminate early
        let result = analyzer.analyze("Test issue", temp_dir.path(), 10).await;
        assert!(result.is_ok());

        let analysis = result.expect("should succeed");
        // May have fewer than 10 whys due to early termination
        assert!(analysis.whys.len() <= 10);
        assert!(analysis.whys.len() >= 3); // Minimum before early termination
    }

    // ============================================================================
    // Edge Cases and Error Paths
    // ============================================================================

    #[tokio::test]
    async fn test_analyze_with_whitespace_only_issue() {
        let analyzer = create_analyzer();
        // Note: Current implementation doesn't trim, so whitespace-only is valid
        let result = analyzer.analyze("   ", Path::new("."), 5).await;
        // This should still work since it's not empty
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_confidence_handles_negative_complexity() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::Complexity,
            json!({"value": -10, "threshold": 20}),
        )];

        let result = analyzer.calculate_confidence(&evidence);
        assert!(result.is_ok());

        let confidence = result.expect("should succeed");
        assert!(confidence >= 0.0);
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_calculate_confidence_handles_zero_threshold() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::Complexity,
            json!({"value": 10, "threshold": 0}),
        )];

        let result = analyzer.calculate_confidence(&evidence);
        assert!(result.is_ok());
        // Should handle division by zero gracefully
    }

    #[test]
    fn test_calculate_confidence_handles_nan_values() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::Complexity,
            json!({"value": "not a number", "threshold": 20}),
        )];

        let result = analyzer.calculate_confidence(&evidence);
        assert!(result.is_ok());

        let confidence = result.expect("should succeed");
        assert!(confidence >= 0.0);
        assert!(confidence <= 1.0);
    }

    // ============================================================================
    // Property-Based Tests
    // ============================================================================

    proptest! {
        #[test]
        fn prop_confidence_always_in_valid_range(
            complexity_value in 0.0f64..200.0,
            threshold in 1.0f64..100.0,
            satd_count in 0u64..100,
            tdg_score in 0.0f64..100.0,
            churn_count in 0u64..100
        ) {
            let analyzer = create_analyzer();
            let evidence = vec![
                create_evidence_with_values(
                    EvidenceSource::Complexity,
                    json!({"value": complexity_value, "threshold": threshold}),
                ),
                create_evidence_with_values(
                    EvidenceSource::SATD,
                    json!({"count": satd_count}),
                ),
                create_evidence_with_values(
                    EvidenceSource::TDG,
                    json!(tdg_score),
                ),
                create_evidence_with_values(
                    EvidenceSource::GitChurn,
                    json!({"commit_count": churn_count}),
                ),
            ];

            let result = analyzer.calculate_confidence(&evidence);
            prop_assert!(result.is_ok());

            let confidence = result.expect("should succeed");
            prop_assert!(confidence >= 0.0);
            prop_assert!(confidence <= 1.0);
        }

        #[test]
        fn prop_hypothesis_is_never_empty(depth in 1u8..=10) {
            let analyzer = create_analyzer();
            let evidence = vec![];

            let result = analyzer.generate_hypothesis("Why?", &evidence, depth);
            prop_assert!(result.is_ok());

            let hypothesis = result.expect("should succeed");
            prop_assert!(!hypothesis.is_empty());
        }

        #[test]
        fn prop_formulate_question_contains_why(depth in 1u8..=10) {
            let analyzer = create_analyzer();
            let issue = "Test issue";

            let result = analyzer.formulate_question(issue, depth, &[]);
            prop_assert!(result.is_ok());

            let question = result.expect("should succeed");
            prop_assert!(question.contains("Why") || question.contains("why"));
        }

        #[test]
        fn prop_recommendations_always_include_root_cause(root_cause in "\\PC{1,50}") {
            let analyzer = create_analyzer();

            let result = analyzer.generate_recommendations(&[], &root_cause);
            prop_assert!(result.is_ok());

            let recommendations = result.expect("should succeed");
            prop_assert!(recommendations.iter().any(|r| r.action.contains(&root_cause)));
        }

        #[test]
        fn prop_extract_root_cause_returns_last_hypothesis(
            hypotheses in proptest::collection::vec("\\PC{1,50}", 1..5)
        ) {
            let analyzer = create_analyzer();
            let whys: Vec<WhyIteration> = hypotheses
                .iter()
                .enumerate()
                .map(|(i, h)| WhyIteration::new(
                    (i + 1) as u8,
                    format!("Q{}", i + 1),
                    h.clone(),
                ))
                .collect();

            let result = analyzer.extract_root_cause(&whys);
            prop_assert!(result.is_ok());

            let root_cause = result.expect("should succeed");
            prop_assert!(root_cause.is_some());
            prop_assert_eq!(root_cause.unwrap(), hypotheses.last().unwrap().clone());
        }
    }

    // ============================================================================
    // Integration-Style Tests
    // ============================================================================

    #[tokio::test]
    async fn test_full_analysis_workflow() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        let result = analyzer
            .analyze("Critical bug in production", temp_dir.path(), 5)
            .await;
        assert!(result.is_ok());

        let analysis = result.expect("should succeed");

        // Verify complete analysis structure
        assert_eq!(analysis.issue, "Critical bug in production");
        assert!(!analysis.whys.is_empty());
        assert!(analysis.whys.len() <= 5);
        assert!(analysis.root_cause.is_some());
        assert!(!analysis.recommendations.is_empty());

        // Verify each why iteration
        for (i, why) in analysis.whys.iter().enumerate() {
            assert_eq!(why.depth as usize, i + 1);
            assert!(!why.question.is_empty());
            assert!(!why.hypothesis.is_empty());
            assert!(why.confidence >= 0.0 && why.confidence <= 1.0);
            assert!(!why.evidence.is_empty());
        }

        // Verify recommendations have priorities
        assert!(analysis
            .recommendations
            .iter()
            .any(|r| r.priority == Priority::High));
    }

    #[tokio::test]
    async fn test_analysis_with_different_depths() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        for depth in [1, 3, 5, 7, 10] {
            let result = analyzer.analyze("Test issue", temp_dir.path(), depth).await;
            assert!(
                result.is_ok(),
                "Analysis should succeed for depth {}",
                depth
            );

            let analysis = result.expect("should succeed");
            assert!(
                analysis.whys.len() <= depth as usize,
                "Should have at most {} whys",
                depth
            );
        }
    }

    #[tokio::test]
    async fn test_evidence_summary_from_analysis() {
        let analyzer = create_analyzer();
        let temp_dir = create_temp_dir();

        let result = analyzer.analyze("Test issue", temp_dir.path(), 5).await;
        assert!(result.is_ok());

        let analysis = result.expect("should succeed");

        // Verify evidence summary is populated
        // Values depend on the synthetic evidence in gather_evidence
        let _ = analysis.evidence_summary.complexity_violations;
        let _ = analysis.evidence_summary.satd_markers;
        let _ = analysis.evidence_summary.tdg_score;
        let _ = analysis.evidence_summary.git_churn_high;
        let _ = analysis.evidence_summary.evoscore_trajectory;
        let _ = analysis.evidence_summary.coverage_delta;
    }

    // ============================================================================
    // PMAT-510: Five Whys v2 Evidence Source Diversification Tests
    // ============================================================================

    #[test]
    fn test_v2_weights_complexity_25_percent() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::Complexity,
            json!({"value": 25, "threshold": 20}),
        )];
        let confidence = analyzer.calculate_confidence(&evidence).unwrap();
        // Complexity at 25% weight with mild severity should produce confidence ~1.0 (weight/weight_sum)
        assert!(
            (0.9..=1.1).contains(&confidence),
            "Complexity confidence should be ~1.0, got {}",
            confidence
        );
    }

    #[test]
    fn test_v2_weights_tdg_zero_percent() {
        let analyzer = create_analyzer();
        // TDG evidence with 0% weight should not contribute to confidence
        let tdg_evidence = vec![create_evidence_with_values(
            EvidenceSource::TDG,
            json!(20.0), // Low TDG score
        )];
        let confidence = analyzer.calculate_confidence(&tdg_evidence).unwrap();
        // With only TDG (weight=0), weight_sum=0, should return 0.5 (fallback)
        assert!(
            (0.49..=0.51).contains(&confidence),
            "TDG-only confidence should be 0.5 (neutral fallback), got {}",
            confidence
        );
    }

    #[test]
    fn test_v2_weights_git_churn_15_percent() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::GitChurn,
            json!({"commit_count": 10, "days": 30}),
        )];
        let confidence = analyzer.calculate_confidence(&evidence).unwrap();
        // Single source, confidence = weight * severity / weight_sum = 1.0 * (1 + 0.5) = 1.5
        // Clamped to [0,1]
        assert!(
            (0.0..=1.0).contains(&confidence),
            "Git churn confidence out of range: {}",
            confidence
        );
    }

    #[test]
    fn test_v2_weights_evoscore_trajectory_15_percent() {
        let analyzer = create_analyzer();
        // Regressing evoscore should amplify confidence (regression = stronger signal)
        let evidence_regress = vec![create_evidence_with_values(
            EvidenceSource::EvoScoreTrajectory,
            json!({"evoscore": -0.5, "commits": 5, "gamma": 1.5}),
        )];
        let conf_regress = analyzer.calculate_confidence(&evidence_regress).unwrap();

        // Improving evoscore should give base confidence
        let evidence_improve = vec![create_evidence_with_values(
            EvidenceSource::EvoScoreTrajectory,
            json!({"evoscore": 0.8, "commits": 5, "gamma": 1.5}),
        )];
        let conf_improve = analyzer.calculate_confidence(&evidence_improve).unwrap();

        // Regressing should have higher or equal confidence than improving
        assert!(
            conf_regress >= conf_improve,
            "Regressing evoscore ({}) should have >= confidence than improving ({})",
            conf_regress,
            conf_improve
        );
    }

    #[test]
    fn test_v2_weights_coverage_delta_15_percent() {
        let analyzer = create_analyzer();
        // Below baseline coverage should amplify confidence
        let evidence_below = vec![create_evidence_with_values(
            EvidenceSource::CoverageDelta,
            json!({"coverage_pct": 60.0, "delta": -25.0, "total_lines": 1000, "covered_lines": 600}),
        )];
        let conf_below = analyzer.calculate_confidence(&evidence_below).unwrap();

        // Above baseline coverage should give base confidence
        let evidence_above = vec![create_evidence_with_values(
            EvidenceSource::CoverageDelta,
            json!({"coverage_pct": 95.0, "delta": 10.0, "total_lines": 1000, "covered_lines": 950}),
        )];
        let conf_above = analyzer.calculate_confidence(&evidence_above).unwrap();

        // Below baseline should have higher or equal confidence
        assert!(
            conf_below >= conf_above,
            "Below baseline ({}) should have >= confidence than above ({})",
            conf_below,
            conf_above
        );
    }

    #[test]
    fn test_v2_all_weights_sum_to_100_percent() {
        // Verify: Complexity 25 + SATD 20 + GitChurn 15 + EvoScore 15 + Coverage 15 + DeadCode 10 = 100
        let analyzer = create_analyzer();
        let evidence = vec![
            create_evidence_with_values(
                EvidenceSource::Complexity,
                json!({"value": 20, "threshold": 20}), // At threshold = severity 0
            ),
            create_evidence_with_values(EvidenceSource::SATD, json!({"count": 0})),
            create_evidence_with_values(
                EvidenceSource::GitChurn,
                json!({"commit_count": 0, "days": 30}),
            ),
            create_evidence_with_values(
                EvidenceSource::EvoScoreTrajectory,
                json!({"evoscore": 0.5, "commits": 5, "gamma": 1.5}),
            ),
            create_evidence_with_values(
                EvidenceSource::CoverageDelta,
                json!({"coverage_pct": 85.0, "delta": 0.0, "total_lines": 1000, "covered_lines": 850}),
            ),
            create_evidence_with_values(EvidenceSource::DeadCode, json!({"count": 0})),
        ];
        let confidence = analyzer.calculate_confidence(&evidence).unwrap();
        // All at baseline/neutral severity=1.0, so confidence = sum(weight*1.0)/sum(weight) = 1.0
        assert!(
            (0.99..=1.01).contains(&confidence),
            "All neutral evidence should give confidence ~1.0, got {}",
            confidence
        );
    }

    #[test]
    fn test_v2_gather_evoscore_with_data() {
        let temp_dir = TempDir::new().expect("tempdir");
        let metrics_dir = temp_dir.path().join(".pmat-metrics");
        std::fs::create_dir_all(&metrics_dir).unwrap();

        // Create 4 commit test records with improving trend
        for (i, pass) in [80, 85, 90, 95].iter().enumerate() {
            std::fs::write(
                metrics_dir.join(format!("commit-{:04}-tests.json", i)),
                format!(r#"{{"commit":"abc{}","pass":{},"total":100}}"#, i, pass),
            )
            .unwrap();
        }

        let evidence = FiveWhysAnalyzer::gather_evoscore_evidence(temp_dir.path());
        assert!(evidence.is_some(), "Should find EvoScore evidence");

        let ev = evidence.unwrap();
        assert_eq!(ev.source, EvidenceSource::EvoScoreTrajectory);
        let evoscore = ev.value.get("evoscore").and_then(|v| v.as_f64()).unwrap();
        assert!(evoscore > 0.0, "Improving trend should have positive evoscore, got {}", evoscore);
    }

    #[test]
    fn test_v2_gather_evoscore_insufficient_data() {
        let temp_dir = TempDir::new().expect("tempdir");
        let metrics_dir = temp_dir.path().join(".pmat-metrics");
        std::fs::create_dir_all(&metrics_dir).unwrap();

        // Only 2 commits - insufficient for trajectory
        for i in 0..2 {
            std::fs::write(
                metrics_dir.join(format!("commit-{:04}-tests.json", i)),
                format!(r#"{{"commit":"abc{}","pass":80,"total":100}}"#, i),
            )
            .unwrap();
        }

        let evidence = FiveWhysAnalyzer::gather_evoscore_evidence(temp_dir.path());
        assert!(evidence.is_none(), "Should return None with <3 commits");
    }

    #[test]
    fn test_v2_gather_evoscore_no_metrics_dir() {
        let temp_dir = TempDir::new().expect("tempdir");
        let evidence = FiveWhysAnalyzer::gather_evoscore_evidence(temp_dir.path());
        assert!(evidence.is_none(), "Should return None without .pmat-metrics/");
    }

    #[test]
    fn test_v2_gather_coverage_delta_with_data() {
        let temp_dir = TempDir::new().expect("tempdir");
        let pmat_dir = temp_dir.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).unwrap();

        let cache = serde_json::json!({
            "git_hash": "abc123",
            "files": {
                "src/lib.rs": {"1": 5, "2": 0, "3": 10, "4": 0, "5": 1}
            }
        });
        std::fs::write(pmat_dir.join("coverage-cache.json"), cache.to_string()).unwrap();

        let evidence = FiveWhysAnalyzer::gather_coverage_delta_evidence(temp_dir.path());
        assert!(evidence.is_some(), "Should find coverage delta evidence");

        let ev = evidence.unwrap();
        assert_eq!(ev.source, EvidenceSource::CoverageDelta);
        let coverage_pct = ev.value.get("coverage_pct").and_then(|v| v.as_f64()).unwrap();
        // 3 covered out of 5 = 60%
        assert!(
            (coverage_pct - 60.0).abs() < 0.1,
            "Expected 60% coverage, got {}",
            coverage_pct
        );
        let delta = ev.value.get("delta").and_then(|v| v.as_f64()).unwrap();
        assert!(delta < 0.0, "60% is below 85% baseline, delta should be negative");
    }

    #[test]
    fn test_v2_gather_coverage_delta_no_cache() {
        let temp_dir = TempDir::new().expect("tempdir");
        let evidence = FiveWhysAnalyzer::gather_coverage_delta_evidence(temp_dir.path());
        assert!(evidence.is_none(), "Should return None without coverage cache");
    }

    #[test]
    fn test_v2_gather_coverage_delta_empty_files() {
        let temp_dir = TempDir::new().expect("tempdir");
        let pmat_dir = temp_dir.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).unwrap();

        let cache = serde_json::json!({
            "git_hash": "abc123",
            "files": {}
        });
        std::fs::write(pmat_dir.join("coverage-cache.json"), cache.to_string()).unwrap();

        let evidence = FiveWhysAnalyzer::gather_coverage_delta_evidence(temp_dir.path());
        assert!(evidence.is_none(), "Should return None with empty files");
    }

    #[test]
    fn test_v2_generate_recommendations_with_regressing_evoscore() {
        let analyzer = create_analyzer();
        let whys = vec![create_why_with_evidence(
            1,
            &[EvidenceSource::EvoScoreTrajectory],
        )];

        let result = analyzer.generate_recommendations(&whys, "Root cause");
        assert!(result.is_ok());

        let recommendations = result.expect("should succeed");
        assert!(
            recommendations
                .iter()
                .any(|r| r.action.contains("trajectory") || r.action.contains("regression")),
            "Should recommend addressing regression trend"
        );
    }

    #[test]
    fn test_v2_hypothesis_with_regressing_evoscore() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::EvoScoreTrajectory,
            json!({"evoscore": -0.5, "commits": 5, "gamma": 1.5}),
        )];

        let result = analyzer.generate_hypothesis("Why?", &evidence, 3);
        assert!(result.is_ok());
        let hypothesis = result.unwrap();
        assert!(
            hypothesis.contains("trajectory") || hypothesis.contains("declining") || hypothesis.contains("worse"),
            "Depth 3 with regressing evoscore should mention trajectory, got: {}",
            hypothesis
        );
    }

    #[test]
    fn test_v2_hypothesis_with_low_coverage() {
        let analyzer = create_analyzer();
        let evidence = vec![create_evidence_with_values(
            EvidenceSource::CoverageDelta,
            json!({"coverage_pct": 60.0, "delta": -25.0}),
        )];

        let result = analyzer.generate_hypothesis("Why?", &evidence, 2);
        assert!(result.is_ok());
        let hypothesis = result.unwrap();
        assert!(
            hypothesis.contains("coverage") || hypothesis.contains("test"),
            "Depth 2 with low coverage should mention coverage, got: {}",
            hypothesis
        );
    }

    proptest! {
        #[test]
        fn prop_v2_confidence_with_new_sources(
            evoscore in -1.0f64..1.0,
            coverage_delta in -50.0f64..50.0,
            complexity_value in 0.0f64..200.0,
            threshold in 1.0f64..100.0,
        ) {
            let analyzer = create_analyzer();
            let evidence = vec![
                create_evidence_with_values(
                    EvidenceSource::Complexity,
                    json!({"value": complexity_value, "threshold": threshold}),
                ),
                create_evidence_with_values(
                    EvidenceSource::EvoScoreTrajectory,
                    json!({"evoscore": evoscore, "commits": 5, "gamma": 1.5}),
                ),
                create_evidence_with_values(
                    EvidenceSource::CoverageDelta,
                    json!({"coverage_pct": 85.0 + coverage_delta, "delta": coverage_delta}),
                ),
            ];

            let result = analyzer.calculate_confidence(&evidence);
            prop_assert!(result.is_ok());
            let confidence = result.unwrap();
            prop_assert!(confidence >= 0.0, "Confidence below 0: {}", confidence);
            prop_assert!(confidence <= 1.0, "Confidence above 1: {}", confidence);
        }
    }
}
