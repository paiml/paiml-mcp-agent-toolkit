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
