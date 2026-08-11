    #[test]
    fn test_debug_analysis_creation() {
        let analysis = DebugAnalysis::new("Test issue".to_string());
        assert_eq!(analysis.issue, "Test issue");
        assert!(analysis.whys.is_empty());
        assert!(analysis.root_cause.is_none());
    }

    #[test]
    fn test_why_iteration_confidence_clamping() {
        let why =
            WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string()).with_confidence(1.5);
        assert_eq!(why.confidence, 1.0);

        let why = WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string())
            .with_confidence(-0.5);
        assert_eq!(why.confidence, 0.0);
    }

    #[test]
    fn test_evidence_summary_from_whys() {
        let mut why = WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string());

        why.add_evidence(Evidence::new(
            EvidenceSource::Complexity,
            PathBuf::from("test.rs"),
            "cyclomatic".to_string(),
            serde_json::json!({"value": 50, "threshold": 20}),
            "High complexity".to_string(),
        ));

        why.add_evidence(Evidence::new(
            EvidenceSource::SATD,
            PathBuf::from("test.rs"),
            "todo_count".to_string(),
            serde_json::json!({"count": 3}),
            "3 TODO markers".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert_eq!(summary.complexity_violations, 1);
        assert_eq!(summary.satd_markers, 3);
    }

    /// debug_analysis.rs:224-229 — the `EvidenceSource::EvoScoreTrajectory` arm
    /// of process_evidence. CB-142 field, only populated by its own arm. Existing
    /// tests only drive Complexity + SATD, so EvoScoreTrajectory stayed at 0%.
    #[test]
    fn test_evidence_summary_picks_up_evoscore_trajectory() {
        let mut why = WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string());
        why.add_evidence(Evidence::new(
            EvidenceSource::EvoScoreTrajectory,
            PathBuf::from("src/evo.rs"),
            "evoscore_delta".to_string(),
            serde_json::json!({"evoscore": -1.75}),
            "EvoScore is regressing".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert!(
            (summary.evoscore_trajectory - (-1.75)).abs() < f64::EPSILON,
            "evoscore field on the nested object must be read via .get(\"evoscore\").as_f64(), \
             got {}",
            summary.evoscore_trajectory
        );
    }

    /// debug_analysis.rs:231-236 — the `EvidenceSource::CoverageDelta` arm of
    /// process_evidence. Positive delta = coverage increased; the value lives
    /// under the nested "delta" key.
    #[test]
    fn test_evidence_summary_picks_up_coverage_delta() {
        let mut why = WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string());
        why.add_evidence(Evidence::new(
            EvidenceSource::CoverageDelta,
            PathBuf::from("src/lib.rs"),
            "line_coverage_delta".to_string(),
            serde_json::json!({"delta": 3.25}),
            "Coverage up 3.25%".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert!(
            (summary.coverage_delta - 3.25).abs() < f64::EPSILON,
            "delta field must be read via .get(\"delta\").as_f64(), got {}",
            summary.coverage_delta
        );
    }

    /// Both the EvoScoreTrajectory and CoverageDelta arms also cover their
    /// `.unwrap_or(0.0)` fallbacks when the expected nested key is missing.
    #[test]
    fn test_evidence_summary_evoscore_and_coverage_missing_keys_default_to_zero() {
        let mut why = WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string());
        why.add_evidence(Evidence::new(
            EvidenceSource::EvoScoreTrajectory,
            PathBuf::from("x.rs"),
            "evo".to_string(),
            serde_json::json!({"wrong_key": 7.5}),
            "noise".to_string(),
        ));
        why.add_evidence(Evidence::new(
            EvidenceSource::CoverageDelta,
            PathBuf::from("x.rs"),
            "cov".to_string(),
            serde_json::json!({"also_wrong": 9.9}),
            "noise".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert_eq!(summary.evoscore_trajectory, 0.0);
        assert_eq!(summary.coverage_delta, 0.0);
    }

/// The producer/consumer key mismatch that made `complexity_violations`
/// structurally 0: `gather_complexity_evidence` wrote
/// `{total_lines, deep_nesting, threshold}` while this consumer reads `value`,
/// so the comparison was always `0.0 > 20.0`.
#[test]
fn complexity_evidence_is_read_under_the_key_the_producer_writes() {
    let mut why = WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string());
    why.add_evidence(Evidence::new(
        EvidenceSource::Complexity,
        PathBuf::from("."),
        "estimated_complexity".to_string(),
        // Exactly what five_whys_analyzer::gather_complexity_evidence emits.
        serde_json::json!({
            "value": 37,
            "threshold": 20,
            "total_lines": 979_249,
            "deep_nesting": 16_386,
        }),
        "979249 source lines, 16386 deeply-nested blocks".to_string(),
    ));

    let summary = EvidenceSummary::from_whys(&[why]);
    assert_eq!(
        summary.complexity_violations, 1,
        "a density of 37 against a threshold of 20 is a violation"
    );
    assert!(summary.complexity_measured);
}

#[test]
fn complexity_below_threshold_is_a_measured_zero_not_an_absent_one() {
    let mut why = WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string());
    why.add_evidence(Evidence::new(
        EvidenceSource::Complexity,
        PathBuf::from("."),
        "estimated_complexity".to_string(),
        serde_json::json!({"value": 17, "threshold": 20}),
        "est. complexity density: 17/1000 lines".to_string(),
    ));

    let summary = EvidenceSummary::from_whys(&[why]);
    assert_eq!(summary.complexity_violations, 0);
    assert!(
        summary.complexity_measured,
        "0 here means measured-and-under-threshold, not no-data"
    );
}

#[test]
fn absent_complexity_and_evoscore_evidence_is_flagged_unmeasured() {
    let summary = EvidenceSummary::from_whys(&[]);
    assert_eq!(summary.complexity_violations, 0);
    assert!(!summary.complexity_measured);
    assert_eq!(summary.evoscore_trajectory, 0.0);
    assert!(!summary.evoscore_measured);
}
