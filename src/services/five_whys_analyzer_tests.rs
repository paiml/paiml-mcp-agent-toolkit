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
        // Analyse a fixture tree, not the live repo. The issue text has to name
        // terms that appear in the source or the analyzer correctly declines to
        // give a root cause ("Test issue", the previous input, was entirely
        // stopwords) — and pointing this at `.` would make the test depend on
        // the crate's own identifiers, so any rename would break it in a way
        // that looks unrelated.
        let dir = tempfile::TempDir::new().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
        std::fs::write(
            dir.path().join("src/widget.rs"),
            "fn widget_throttle(retries: u32) -> u32 {\n    \
             if retries > 3 { retries * 2 } else { retries }\n}\n",
        )
        .expect("write fixture");

        let analyzer = FiveWhysAnalyzer::new();
        let issue = "widget_throttle retries miscounted";
        let result = analyzer
            .analyze(issue, dir.path(), 5)
            .await
            .expect("internal error");

        assert_eq!(result.issue, issue);
        assert!(!result.whys.is_empty());
        assert!(
            result.root_cause.is_some(),
            "an issue naming terms present in the source must be located and \
             yield a cause; got none"
        );
        assert!(
            result
                .root_cause
                .as_deref()
                .is_some_and(|rc| rc.contains("widget.rs")),
            "the cause should cite where the terms were found, got: {:?}",
            result.root_cause
        );
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
                EvidenceSource::IssueLocation => {
                    json!({"terms": ["stdio", "transport"],
                           "locations": [{"file": "src/x.rs", "line": 12,
                                          "terms_matched": 2, "term": "stdio+transport"}]})
                }
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
        // An empty temp dir contains no source, so the issue cannot be located
        // and a root cause must be withheld rather than invented from
        // repo-wide metrics (GH #637).
        assert_eq!(
            analysis.root_cause, None,
            "nothing to locate the issue in -> no root cause may be claimed"
        );
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

        // Strictly greater, not `>=`. The old assertion passed while the score
        // was structurally pinned to exactly 1.0 for every input (each source
        // contributed `weight * (1.0 + severity)` over a divisor of `weight`),
        // so `1.0 >= 1.0` hid the fact that confidence never varied at all.
        assert!(
            high_confidence > low_confidence,
            "confidence must respond to severity: low={low_confidence}, high={high_confidence}"
        );
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

        // No IssueLocation evidence -> the hypothesis came from repo-wide
        // metrics alone, so it must not be presented as the root cause.
        let unlocated = analyzer.extract_root_cause(&whys).expect("should succeed");
        assert_eq!(
            unlocated, None,
            "a root cause must not be asserted when the issue was never located"
        );

        // With evidence tied to the issue, the final hypothesis stands.
        let mut located = whys.clone();
        located[0].evidence = vec![create_evidence_with_values(
            EvidenceSource::IssueLocation,
            json!({"terms": ["hypothesis"], "locations": [{"file": "a.rs", "line": 1}]}),
        )];
        let root_cause = analyzer
            .extract_root_cause(&located)
            .expect("should succeed")
            .expect("located evidence should yield a cause");
        assert!(
            root_cause.starts_with("The hypothesis"),
            "got: {root_cause}"
        );
        assert!(
            root_cause.contains("no causal chain was derived"),
            "must state its own limits rather than implying a derived chain, got: {root_cause}"
        );
    }

    #[test]
    fn test_extract_root_cause_multiple_whys() {
        let analyzer = create_analyzer();
        let whys = vec![
            WhyIteration::new(1, "Q1".to_string(), "Hypothesis 1".to_string()),
            WhyIteration::new(2, "Q2".to_string(), "Hypothesis 2".to_string()),
            WhyIteration::new(3, "Q3".to_string(), "Final hypothesis".to_string()),
        ];

        // Locating the issue in any single why is enough to report a cause.
        let mut whys = whys;
        whys[1].evidence = vec![create_evidence_with_values(
            EvidenceSource::IssueLocation,
            json!({"terms": ["final"], "locations": [{"file": "a.rs", "line": 1}]}),
        )];
        let root_cause = analyzer
            .extract_root_cause(&whys)
            .expect("should succeed")
            .expect("located evidence should yield a cause");
        assert!(
            root_cause.starts_with("Final hypothesis"),
            "got: {root_cause}"
        );
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

        let result = analyzer
            .gather_evidence("test issue", temp_dir.path())
            .await;
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
            .gather_evidence("test issue", temp_dir.path())
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
            .gather_evidence("test issue", temp_dir.path())
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
        let count = satd
            .value
            .get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        assert!(count >= 2, "Expected >=2 SATD markers, got {}", count);
    }

    #[tokio::test]
    async fn test_gather_evidence_no_longer_includes_tdg() {
        // v2 (PMAT-510): TDG removed from evidence gathering (redundant with complexity+churn)
        let analyzer = create_analyzer();
        let temp_dir = create_evidence_temp_dir();

        let evidence = analyzer
            .gather_evidence("test issue", temp_dir.path())
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
            .gather_evidence("test issue", temp_dir.path())
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

        /// `generate_recommendations` echoes the root cause back — **unless the
        /// cause is blank**, in which case it says so instead of printing a
        /// bare "Address root cause: " with nothing after it
        /// (`five_whys_analyzer.rs:1118`).
        ///
        /// This test used to assert only the first half, and it was flaky: the
        /// generator is `\PC{1,50}` (1-50 NON-CONTROL chars), and "non-control"
        /// is not "non-blank". U+2028 LINE SEPARATOR is category Zl, so `\PC`
        /// emits it, and it is `White_Space=yes`, so `trim().is_empty()` is
        /// true for it. Roughly one CI run in several drew a whitespace-only
        /// string, took the blank branch, and failed — `minimal failing input:
        /// root_cause = "\u{2028}"`, after 47 successes.
        ///
        /// The product was right and the property was wrong, so the property is
        /// what changed. It is not narrowed to dodge the case: both branches are
        /// asserted, so the blank path is now pinned rather than merely avoided.
        #[test]
        fn prop_recommendations_always_include_root_cause(root_cause in "\\PC{1,50}") {
            let analyzer = create_analyzer();

            let result = analyzer.generate_recommendations(&[], &root_cause);
            prop_assert!(result.is_ok());

            let recommendations = result.expect("should succeed");
            if root_cause.trim().is_empty() {
                prop_assert!(
                    recommendations
                        .iter()
                        .any(|r| r.action.contains("No root cause was determined")),
                    "a blank root cause must be reported as undetermined, not echoed: {recommendations:?}"
                );
            } else {
                prop_assert!(
                    recommendations.iter().any(|r| r.action.contains(&root_cause)),
                    "a real root cause must be echoed back: {recommendations:?}"
                );
            }
        }


        #[test]
        fn prop_extract_root_cause_returns_last_hypothesis(
            hypotheses in proptest::collection::vec("\\PC{1,50}", 1..5)
        ) {
            let analyzer = create_analyzer();
            let mut whys: Vec<WhyIteration> = hypotheses
                .iter()
                .enumerate()
                .map(|(i, h)| WhyIteration::new(
                    (i + 1) as u8,
                    format!("Q{}", i + 1),
                    h.clone(),
                ))
                .collect();
            // Attach issue-specific evidence: without it the analyzer withholds
            // the root cause entirely, which is a separate contract covered by
            // `test_extract_root_cause_single_why`.
            whys[0].evidence = vec![create_evidence_with_values(
                EvidenceSource::IssueLocation,
                json!({"terms": ["x"], "locations": [{"file": "a.rs", "line": 1}]}),
            )];

            let result = analyzer.extract_root_cause(&whys);
            prop_assert!(result.is_ok());

            let root_cause = result.expect("should succeed");
            prop_assert!(root_cause.is_some());
            // The reported cause now leads with the deepest issue-derived
            // hypothesis and appends an explicit statement of what was not
            // derived, so match the prefix rather than the whole string.
            let root_cause = root_cause.unwrap();
            prop_assert!(root_cause.starts_with(hypotheses.last().unwrap()));
            prop_assert!(root_cause.contains("no causal chain was derived"));
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
        // Empty temp dir: the issue is unlocatable, so no root cause. The
        // recommendations still stand — they are generic remediation advice.
        assert_eq!(analysis.root_cause, None);
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
        // 25 against a threshold of 20 is *mild* (severity 0.25), so confidence
        // must be mild too. This previously asserted ~1.0 and explained it as
        // "weight/weight_sum" — which was the saturation bug stated as the
        // goal: every severity multiplier was `1.0 + s`, so the ratio was
        // always >= 1.0 and clamped to exactly 100% for any input (GH #637).
        assert!(
            (0.2..=0.3).contains(&confidence),
            "mild severity should give mild confidence, got {}",
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
        // TDG carries zero weight, so weight_sum is 0 and the neutral 0.5
        // fallback applies — but TDG says nothing about the reported issue, so
        // the relevance cap then applies on top of it.
        assert_eq!(
            confidence,
            FiveWhysAnalyzer::NO_ISSUE_EVIDENCE_CEILING,
            "TDG-only evidence never locates the issue, so confidence must be \
             capped rather than sitting at the 0.5 neutral fallback"
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
        // Every source present but all at *zero* severity: nothing is wrong, so
        // confidence must be near zero. The old assertion demanded ~1.0 because
        // neutral severity was encoded as the multiplier 1.0, which meant a
        // completely healthy repository still reported 100% confidence in a
        // causal claim (GH #637).
        assert!(
            confidence <= 0.05,
            "all-neutral evidence should give near-zero confidence, got {}",
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
        assert!(
            evoscore > 0.0,
            "Improving trend should have positive evoscore, got {}",
            evoscore
        );
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
        assert!(
            evidence.is_none(),
            "Should return None without .pmat-metrics/"
        );
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
        let coverage_pct = ev
            .value
            .get("coverage_pct")
            .and_then(|v| v.as_f64())
            .unwrap();
        // 3 covered out of 5 = 60%
        assert!(
            (coverage_pct - 60.0).abs() < 0.1,
            "Expected 60% coverage, got {}",
            coverage_pct
        );
        let delta = ev.value.get("delta").and_then(|v| v.as_f64()).unwrap();
        assert!(
            delta < 0.0,
            "60% is below 85% baseline, delta should be negative"
        );
    }

    #[test]
    fn test_v2_gather_coverage_delta_no_cache() {
        let temp_dir = TempDir::new().expect("tempdir");
        let evidence = FiveWhysAnalyzer::gather_coverage_delta_evidence(temp_dir.path());
        assert!(
            evidence.is_none(),
            "Should return None without coverage cache"
        );
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
            hypothesis.contains("trajectory")
                || hypothesis.contains("declining")
                || hypothesis.contains("worse"),
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

    /// The blank branch of `generate_recommendations`, pinned exhaustively.
    ///
    /// A plain loop rather than a proptest strategy on purpose: this is a fixed,
    /// finite set of inputs, and `prop::sample::select` would only *probably*
    /// reach each one. The case that broke CI (U+2028) is in the list by name,
    /// so it is checked on every run instead of when the seed cooperates.
    #[test]
    fn blank_root_cause_is_reported_as_undetermined() {
        let analyzer = create_analyzer();
        // ASCII blanks, then the Unicode ones `\PC` can emit: LINE SEPARATOR,
        // PARAGRAPH SEPARATOR, NO-BREAK SPACE. All are White_Space=yes, so all
        // of them make `root_cause.trim().is_empty()` true.
        for blank in ["", " ", "\t", "   \t  ", "\u{2028}", "\u{2029}", "\u{00a0}"] {
            let recommendations = analyzer
                .generate_recommendations(&[], blank)
                .expect("generate_recommendations should succeed");
            assert!(
                recommendations
                    .iter()
                    .any(|r| r.action.contains("No root cause was determined")),
                "blank {blank:?} must be reported as undetermined, not echoed: {recommendations:?}"
            );
            assert!(
                !recommendations
                    .iter()
                    .any(|r| r.action == format!("Address root cause: {blank}")),
                "blank {blank:?} produced a bare 'Address root cause: ' with nothing \
                 after it — the exact output the guard at five_whys_analyzer.rs:1118 exists to prevent"
            );
        }
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

    // ── GH #637: the analysis must be about the issue it was given ──────────────
    //
    // Before these, `five-whys` produced identical repo-wide evidence for any
    // input, asserted 100% confidence from it, and named a truism as the root
    // cause. See `FiveWhysAnalyzer::calculate_confidence` for the two structural
    // faults.

    #[test]
    fn issue_terms_drop_noise_and_keep_identifiers() {
        let terms = FiveWhysAnalyzer::issue_terms(
            "MCP stdio server drops responses when the client closes stdin and this fails",
        );
        for kept in [
            "stdio",
            "server",
            "drops",
            "responses",
            "client",
            "closes",
            "stdin",
        ] {
            assert!(
                terms.contains(&kept.to_string()),
                "should keep {kept}: {terms:?}"
            );
        }
        for dropped in ["the", "when", "this", "fails", "mcp"] {
            assert!(
                !terms.contains(&dropped.to_string()),
                "should drop {dropped} (stopword or <4 chars): {terms:?}"
            );
        }
    }

    #[test]
    fn confidence_is_capped_when_the_issue_was_never_located() {
        let analyzer = create_analyzer();
        // Every repo-wide source at maximum severity, but nothing issue-specific.
        let evidence = vec![
            create_evidence_with_values(
                EvidenceSource::Complexity,
                json!({"value": 500, "threshold": 20}),
            ),
            create_evidence_with_values(EvidenceSource::SATD, json!({"count": 5000})),
            create_evidence_with_values(EvidenceSource::GitChurn, json!({"commit_count": 900})),
            create_evidence_with_values(EvidenceSource::CoverageDelta, json!({"delta": -80.0})),
        ];
        let confidence = analyzer
            .calculate_confidence(&evidence)
            .expect("should succeed");
        assert!(
            confidence <= FiveWhysAnalyzer::NO_ISSUE_EVIDENCE_CEILING,
            "piling up repo-wide metrics must not buy confidence about a specific \
         issue; got {confidence}"
        );
    }

    #[test]
    fn confidence_can_exceed_the_cap_once_the_issue_is_located() {
        let analyzer = create_analyzer();
        let locations: Vec<_> = (0..8)
            .map(|i| json!({"file": "src/x.rs", "line": i, "terms_matched": 2, "term": "a+b"}))
            .collect();
        let evidence = vec![
            create_evidence_with_values(
                EvidenceSource::IssueLocation,
                json!({"terms": ["stdio", "transport"], "locations": locations}),
            ),
            create_evidence_with_values(
                EvidenceSource::Complexity,
                json!({"value": 500, "threshold": 20}),
            ),
        ];
        let confidence = analyzer
            .calculate_confidence(&evidence)
            .expect("should succeed");
        assert!(
            confidence > FiveWhysAnalyzer::NO_ISSUE_EVIDENCE_CEILING,
            "located evidence should support real confidence; got {confidence}"
        );
    }

    #[test]
    fn confidence_is_never_pinned_to_one() {
        // The old formula divided `weight * (1.0 + severity)` by `weight`, so the
        // result was always >= 1.0 and clamped to exactly 1.0 for every input.
        let analyzer = create_analyzer();
        let mild = vec![create_evidence_with_values(
            EvidenceSource::GitChurn,
            json!({"commit_count": 1}),
        )];
        let confidence = analyzer
            .calculate_confidence(&mild)
            .expect("should succeed");
        assert!(
            confidence < 0.9,
            "a single low-severity signal must not read as near-certainty; got {confidence}"
        );
    }

    /// five-whys reported 808 SATD markers for this repo while `pmat analyze satd`
    /// reported 39 for the same path in the same session — the old
    /// `count_satd_markers` was a raw substring scan with no comment awareness, so
    /// it counted pattern tables, fixtures and doc prose. Both surfaces now read the
    /// same detector.
    #[tokio::test]
    async fn satd_evidence_agrees_with_the_analyze_satd_detector() {
        use crate::services::satd_detector::SATDDetector;

        let dir = tempfile::tempdir().expect("tempdir");
        // A named subdirectory: `tempfile` roots are dot-prefixed and source walks
        // skip hidden entries.
        let root = dir.path().join("proj");
        std::fs::create_dir_all(root.join("src")).expect("mkdir");
        std::fs::write(
            root.join("src/lib.rs"),
            // One genuine SATD comment, plus three lines that a bare substring
            // scan counts and a detector does not: a string literal, a doc mention
            // and an identifier.
            "// TODO: implement retry\n\
         pub const MARKERS: &[&str] = &[\"TODO\", \"FIXME\", \"HACK\"];\n\
         /// Explains why the word FIXME appears in the pattern table above.\n\
         pub fn xxx_helper() -> u32 { 1 }\n",
        )
        .expect("write");

        let evidence = FiveWhysAnalyzer::gather_satd_evidence(&root)
            .await
            .expect("SATD evidence must be gathered");
        let five_whys_count = evidence.value["count"].as_u64().expect("count key");

        let detector_count = SATDDetector::new()
            .analyze_project(&root, false)
            .await
            .expect("detector must run")
            .summary
            .total_items as u64;

        assert_eq!(
            five_whys_count, detector_count,
            "five-whys must report the SATD number `analyze satd` reports"
        );
    }

    /// `complexity_violations` was structurally always 0: this producer wrote
    /// `{total_lines, deep_nesting, threshold}` while
    /// `EvidenceSummary::process_complexity_evidence` reads a `value` key nobody
    /// wrote, so the comparison was always `0.0 > 20.0`. The producer and the
    /// consumer must agree on the key.
    #[test]
    fn complexity_evidence_carries_the_key_the_summary_reads() {
        use crate::models::debug_analysis::{EvidenceSummary, WhyIteration};

        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join("proj");
        std::fs::create_dir_all(root.join("src")).expect("mkdir");
        // Three of four lines sit deeper than the nesting threshold, so the
        // estimated density is far above 20/1000 lines.
        std::fs::write(root.join("src/deep.rs"), "{{{{{{\nfoo\nbar\n}}}}}}\n").expect("write");

        let evidence = FiveWhysAnalyzer::gather_complexity_evidence(&root)
            .expect("a src/ tree must yield complexity evidence");

        let value = evidence
            .value
            .get("value")
            .and_then(serde_json::Value::as_f64)
            .expect("the payload must carry the `value` key the summary reads");
        let threshold = evidence
            .value
            .get("threshold")
            .and_then(serde_json::Value::as_f64)
            .expect("threshold");
        assert!(value > threshold, "{value} vs {threshold}");

        let mut why = WhyIteration::new(1, "Why?".to_string(), "H".to_string());
        why.add_evidence(evidence);
        let summary = EvidenceSummary::from_whys(&[why]);
        assert_eq!(
            summary.complexity_violations, 1,
            "the summary must see the violation the evidence describes"
        );
        assert!(summary.complexity_measured);
    }
}

#[cfg(test)]
mod depth_and_saturation_tests {
    //! REGRESSION (#962): `--depth` was inert above 3 on any real repository.
    //!
    //! Every severity scale was a hard clamp that real repos blew past — pmat
    //! reports 62 SATD markers against a cap of 10, 29 commits against 20, 12
    //! matched locations against 6 — so every severity pinned to 1.0, the
    //! weighted mean was exactly 1.0, and the `i >= 3 && confidence > 0.9` early
    //! exit fired on iteration 3 every time. `--depth 5`, `7` and `10` all
    //! returned three whys, each stamped 100.0% directly above the report's own
    //! sentence disclaiming them as "repo-wide signals, not findings about this
    //! defect".
    use super::*;

    /// The scale must distinguish codebases it is there to compare. Under the
    /// old `count.min(10.0) / 10.0` these three were all exactly 1.0.
    #[test]
    fn severity_discriminates_past_the_old_clamp() {
        let at_10 = FiveWhysAnalyzer::saturating_severity(10.0, 10.0);
        let at_62 = FiveWhysAnalyzer::saturating_severity(62.0, 10.0);
        let at_200 = FiveWhysAnalyzer::saturating_severity(200.0, 10.0);
        assert!(
            at_10 < at_62,
            "10 and 62 markers must differ: {at_10} vs {at_62}"
        );
        assert!(at_62 < at_200, "62 and 200 markers must differ");
        assert!(
            (at_10 - 0.5).abs() < 1e-9,
            "`half` is the half-severity point, got {at_10}"
        );
        assert!(
            at_200 < 1.0,
            "no finite count may reach certainty, got {at_200}"
        );
    }

    /// Monotone and bounded, including the degenerate inputs.
    #[test]
    fn severity_is_monotone_and_bounded() {
        assert_eq!(FiveWhysAnalyzer::saturating_severity(0.0, 10.0), 0.0);
        assert_eq!(FiveWhysAnalyzer::saturating_severity(-5.0, 10.0), 0.0);
        assert_eq!(FiveWhysAnalyzer::saturating_severity(5.0, 0.0), 0.0);
        let mut prev = 0.0;
        for n in 1..500 {
            let s = FiveWhysAnalyzer::saturating_severity(f64::from(n), 20.0);
            assert!(s > prev, "must rise at n={n}");
            assert!(s < 1.0, "must stay below 1.0 at n={n}");
            prev = s;
        }
    }

    /// The ceiling has to sit BELOW the early-exit threshold, or capping the
    /// repo-level rungs would not restore `--depth` at all. This is the exact
    /// coupling that made the bug: a constant on one side, a literal on the
    /// other, with nothing asserting the relation.
    #[test]
    fn repo_level_ceiling_cannot_trip_the_early_exit() {
        assert!(
            FiveWhysAnalyzer::REPO_LEVEL_CEILING <= 0.9,
            "a chain of repo-wide signals must not be able to terminate the analysis"
        );
        assert!(
            FiveWhysAnalyzer::REPO_LEVEL_CEILING > FiveWhysAnalyzer::NO_ISSUE_EVIDENCE_CEILING,
            "these runs DID locate the issue; the localisation is real evidence"
        );
    }

    /// A hypothesis the report disclaims must not be stamped near-certain, no
    /// matter how much repo-wide evidence piled up behind it.
    #[tokio::test]
    async fn depth_is_honoured_and_repo_level_rungs_are_capped() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("src")).expect("src");
        // Enough SATD to have saturated every old scale several times over.
        let mut body = String::from("fn main() {}\n");
        for i in 0..80 {
            body.push_str(&format!("// TODO: parser stack overflow case {i}\n"));
        }
        std::fs::write(dir.path().join("src/main.rs"), body).expect("write");
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"t\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("write");

        let analyzer = FiveWhysAnalyzer::new();
        let analysis = analyzer
            .analyze("parser stack overflow", dir.path(), 10)
            .await
            .expect("analyze");

        assert_eq!(
            analysis.whys.len(),
            10,
            "--depth 10 must produce 10 whys; it produced 3 for two releases"
        );
        for why in &analysis.whys {
            if why.hypothesis.contains(FiveWhysAnalyzer::REPO_LEVEL_TAG) {
                assert!(
                    why.confidence <= FiveWhysAnalyzer::REPO_LEVEL_CEILING,
                    "a rung the report disclaims was stamped {:.3}: {}",
                    why.confidence,
                    why.hypothesis
                );
            }
        }
    }
}
