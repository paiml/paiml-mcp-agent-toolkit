    // ============================================================================
    // Recommendation Unit Tests
    // ============================================================================

    #[test]
    fn test_recommendation_new() {
        let rec = Recommendation::new(
            Priority::High,
            "Fix the bug".to_string(),
            Some(PathBuf::from("src/buggy.rs")),
        );

        assert_eq!(rec.priority, Priority::High);
        assert_eq!(rec.action, "Fix the bug");
        assert_eq!(rec.file, Some(PathBuf::from("src/buggy.rs")));
    }

    #[test]
    fn test_recommendation_high_factory() {
        let rec = Recommendation::high("Critical fix".to_string(), None);

        assert_eq!(rec.priority, Priority::High);
        assert_eq!(rec.action, "Critical fix");
        assert!(rec.file.is_none());
    }

    #[test]
    fn test_recommendation_medium_factory() {
        let rec = Recommendation::medium(
            "Improve test coverage".to_string(),
            Some(PathBuf::from("tests/")),
        );

        assert_eq!(rec.priority, Priority::Medium);
        assert_eq!(rec.action, "Improve test coverage");
    }

    #[test]
    fn test_recommendation_low_factory() {
        let rec = Recommendation::low("Refactor for readability".to_string(), None);

        assert_eq!(rec.priority, Priority::Low);
    }

    #[test]
    fn test_recommendation_without_file() {
        let rec = Recommendation::new(Priority::Medium, "General improvement".to_string(), None);

        assert!(rec.file.is_none());
    }

    #[test]
    fn test_recommendation_serialization_roundtrip() {
        let rec = create_test_recommendation(Priority::High);

        let json = serde_json::to_string(&rec).expect("Serialization should succeed");
        let deserialized: Recommendation =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        assert_eq!(deserialized.priority, rec.priority);
        assert_eq!(deserialized.action, rec.action);
        assert_eq!(deserialized.file, rec.file);
    }

    // ============================================================================
    // Priority Unit Tests
    // ============================================================================

    #[test]
    fn test_priority_equality() {
        assert_eq!(Priority::High, Priority::High);
        assert_eq!(Priority::Medium, Priority::Medium);
        assert_eq!(Priority::Low, Priority::Low);
        assert_ne!(Priority::High, Priority::Low);
    }

    #[test]
    fn test_priority_serialization_roundtrip() {
        for priority in [Priority::High, Priority::Medium, Priority::Low] {
            let json = serde_json::to_string(&priority).expect("Serialization should succeed");
            let deserialized: Priority =
                serde_json::from_str(&json).expect("Deserialization should succeed");
            assert_eq!(deserialized, priority);
        }
    }

    #[test]
    fn test_priority_copy() {
        let priority = Priority::High;
        let copied = priority;
        assert_eq!(priority, copied);
    }

    // ============================================================================
    // EvidenceSummary Unit Tests
    // ============================================================================

    #[test]
    fn test_evidence_summary_default() {
        let summary = EvidenceSummary::default();

        assert_eq!(summary.complexity_violations, 0);
        assert_eq!(summary.satd_markers, 0);
        assert_eq!(summary.tdg_score, 0.0);
        assert!(!summary.git_churn_high);
    }

    #[test]
    fn test_evidence_summary_from_empty_whys() {
        let summary = EvidenceSummary::from_whys(&[]);

        assert_eq!(summary.complexity_violations, 0);
        assert_eq!(summary.satd_markers, 0);
        assert_eq!(summary.tdg_score, 0.0);
        assert!(!summary.git_churn_high);
    }

    #[test]
    fn test_evidence_summary_counts_complexity_violations() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::Complexity,
            PathBuf::from("a.rs"),
            "complexity".to_string(),
            serde_json::json!({"value": 25, "threshold": 20}),
            "High".to_string(),
        ));
        why.add_evidence(Evidence::new(
            EvidenceSource::Complexity,
            PathBuf::from("b.rs"),
            "complexity".to_string(),
            serde_json::json!({"value": 30, "threshold": 20}),
            "Very high".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert_eq!(summary.complexity_violations, 2);
    }

    #[test]
    fn test_evidence_summary_ignores_below_threshold_complexity() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::Complexity,
            PathBuf::from("simple.rs"),
            "complexity".to_string(),
            serde_json::json!({"value": 15, "threshold": 20}),
            "OK".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert_eq!(summary.complexity_violations, 0);
    }

    #[test]
    fn test_evidence_summary_counts_satd_with_count() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::SATD,
            PathBuf::from("test.rs"),
            "satd".to_string(),
            serde_json::json!({"count": 7}),
            "TODOs".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert_eq!(summary.satd_markers, 7);
    }

    #[test]
    fn test_evidence_summary_counts_satd_without_count() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::SATD,
            PathBuf::from("test.rs"),
            "satd".to_string(),
            serde_json::json!({}), // No count field
            "Single marker".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert_eq!(summary.satd_markers, 1); // Defaults to 1
    }

    #[test]
    fn test_evidence_summary_accumulates_satd_across_whys() {
        let mut why1 = create_test_why_iteration(1, 0.5);
        why1.add_evidence(Evidence::new(
            EvidenceSource::SATD,
            PathBuf::from("a.rs"),
            "satd".to_string(),
            serde_json::json!({"count": 3}),
            "TODOs".to_string(),
        ));

        let mut why2 = create_test_why_iteration(2, 0.6);
        why2.add_evidence(Evidence::new(
            EvidenceSource::SATD,
            PathBuf::from("b.rs"),
            "satd".to_string(),
            serde_json::json!({"count": 5}),
            "FIXMEs".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why1, why2]);
        assert_eq!(summary.satd_markers, 8);
    }

    #[test]
    fn test_evidence_summary_extracts_tdg_score() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::TDG,
            PathBuf::from("test.rs"),
            "tdg".to_string(),
            serde_json::json!(75.5),
            "Moderate coverage".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert!((summary.tdg_score - 75.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_evidence_summary_detects_high_git_churn() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::GitChurn,
            PathBuf::from("test.rs"),
            "churn".to_string(),
            serde_json::json!({"commit_count": 15}),
            "High churn".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert!(summary.git_churn_high);
    }

    #[test]
    fn test_evidence_summary_low_git_churn() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::GitChurn,
            PathBuf::from("test.rs"),
            "churn".to_string(),
            serde_json::json!({"commit_count": 5}),
            "Low churn".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert!(!summary.git_churn_high);
    }

    #[test]
    fn test_evidence_summary_ignores_dead_code_and_manual() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(create_test_evidence(EvidenceSource::DeadCode));
        why.add_evidence(create_test_evidence(EvidenceSource::ManualInspection));

        let summary = EvidenceSummary::from_whys(&[why]);
        // These sources don't contribute to the summary fields
        assert_eq!(summary.complexity_violations, 0);
        assert_eq!(summary.satd_markers, 0);
    }

    #[test]
    fn test_evidence_summary_serialization_roundtrip() {
        let summary = EvidenceSummary {
            complexity_violations: 3,
            satd_markers: 7,
            tdg_score: 65.5,
            // A score is only meaningful alongside the flag saying it was
            // actually measured; see `EvidenceSummary::tdg_measured` (GH #637).
            tdg_measured: true,
            git_churn_high: true,
            evoscore_trajectory: 0.0,
            coverage_delta: 0.0,
        };

        let json = serde_json::to_string(&summary).expect("Serialization should succeed");
        let deserialized: EvidenceSummary =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        assert_eq!(deserialized.complexity_violations, 3);
        assert_eq!(deserialized.satd_markers, 7);
        assert!((deserialized.tdg_score - 65.5).abs() < f64::EPSILON);
        assert!(deserialized.git_churn_high);
    }
