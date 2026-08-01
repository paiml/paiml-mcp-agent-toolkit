    // ============================================================================
    // Edge Cases and Error Paths
    // ============================================================================

    #[test]
    fn test_evidence_summary_handles_missing_value_in_complexity() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::Complexity,
            PathBuf::from("test.rs"),
            "complexity".to_string(),
            serde_json::json!({"threshold": 20}), // Missing value
            "Unknown".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert_eq!(summary.complexity_violations, 0); // Should not count as violation
    }

    #[test]
    fn test_evidence_summary_handles_missing_threshold_in_complexity() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::Complexity,
            PathBuf::from("test.rs"),
            "complexity".to_string(),
            serde_json::json!({"value": 25}), // Missing threshold
            "High".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        // Missing threshold defaults to 20.0; value 25 > 20 = violation
        assert_eq!(summary.complexity_violations, 1);
    }

    #[test]
    fn test_evidence_summary_handles_non_numeric_tdg() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::TDG,
            PathBuf::from("test.rs"),
            "tdg".to_string(),
            serde_json::json!("not a number"),
            "Invalid".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert_eq!(summary.tdg_score, 0.0); // Default when parsing fails
    }

    #[test]
    fn test_why_iteration_with_many_evidence_items() {
        let mut why = create_test_why_iteration(1, 0.5);

        for _ in 0..100 {
            why.add_evidence(create_test_evidence(EvidenceSource::SATD));
        }

        assert_eq!(why.evidence.len(), 100);
    }

    #[test]
    fn test_debug_analysis_with_many_whys() {
        let mut analysis = create_test_debug_analysis();

        for depth in 1..=10 {
            analysis
                .whys
                .push(create_test_why_iteration(depth, depth as f64 / 10.0));
        }

        assert_eq!(analysis.whys.len(), 10);
        assert_eq!(analysis.whys[9].depth, 10);
    }

    #[test]
    fn test_evidence_with_very_long_strings() {
        let long_metric = "a".repeat(10000);
        let long_interpretation = "b".repeat(10000);

        let evidence = Evidence::new(
            EvidenceSource::ManualInspection,
            PathBuf::from("test.rs"),
            long_metric.clone(),
            serde_json::json!(null),
            long_interpretation.clone(),
        );

        assert_eq!(evidence.metric.len(), 10000);
        assert_eq!(evidence.interpretation.len(), 10000);
    }

    #[test]
    fn test_recommendation_with_very_long_action() {
        let long_action = "x".repeat(10000);
        let rec = Recommendation::high(long_action.clone(), None);
        assert_eq!(rec.action.len(), 10000);
    }

    // ============================================================================
    // Property-Based Tests
    // ============================================================================

    proptest! {
        #[test]
        fn prop_confidence_always_clamped(confidence in -100.0f64..100.0) {
            let why = WhyIteration::new(
                1,
                "Why?".to_string(),
                "Hypothesis".to_string(),
            ).with_confidence(confidence);

            prop_assert!(why.confidence >= 0.0);
            prop_assert!(why.confidence <= 1.0);
        }

        #[test]
        fn prop_depth_preserved(depth in 1u8..=10) {
            let why = WhyIteration::new(
                depth,
                "Question".to_string(),
                "Hypothesis".to_string(),
            );
            prop_assert_eq!(why.depth, depth);
        }

        #[test]
        fn prop_evidence_count_preserved(count in 0usize..50) {
            let mut why = WhyIteration::new(1, "Q".to_string(), "H".to_string());

            for _ in 0..count {
                why.add_evidence(Evidence::new(
                    EvidenceSource::ManualInspection,
                    PathBuf::from("test.rs"),
                    "metric".to_string(),
                    serde_json::json!(null),
                    "interpretation".to_string(),
                ));
            }

            prop_assert_eq!(why.evidence.len(), count);
        }

        #[test]
        fn prop_serialization_roundtrip_debug_analysis(issue in "\\PC{1,100}") {
            let analysis = DebugAnalysis::new(issue.clone());
            let json = serde_json::to_string(&analysis)
                .expect("Serialization should succeed");
            let deserialized: DebugAnalysis = serde_json::from_str(&json)
                .expect("Deserialization should succeed");
            prop_assert_eq!(deserialized.issue, issue);
        }

        #[test]
        /// GH #670: DISTINCT SATD findings accumulate; the same finding cited by
        /// several whys does not.
        ///
        /// This property previously put the same measurement on every why (same
        /// file, same metric) and asserted the sum, which is the bug: it made
        /// `satd_markers` a function of `--depth` (4 markers reported as 4/12/20
        /// at depth 1/3/5). Each why now carries evidence about a different
        /// file, so summing is the right answer for a real reason.
        fn prop_satd_count_accumulates(counts in proptest::collection::vec(0u64..100, 1..10)) {
            let mut whys = Vec::new();

            for (i, count) in counts.iter().enumerate() {
                let mut why = create_test_why_iteration((i + 1) as u8, 0.5);
                why.add_evidence(Evidence::new(
                    EvidenceSource::SATD,
                    PathBuf::from(format!("file{i}.rs")),
                    "satd".to_string(),
                    serde_json::json!({"count": count}),
                    "markers".to_string(),
                ));
                whys.push(why);
            }

            let summary = EvidenceSummary::from_whys(&whys);
            let expected: u64 = counts.iter().sum();
            prop_assert_eq!(summary.satd_markers, expected as usize);
        }

        /// The dual property: repeating ONE finding across N whys still counts
        /// it once, whatever N is.
        fn prop_satd_count_is_independent_of_depth(
            count in 0u64..100,
            depth in 1usize..10
        ) {
            let whys: Vec<_> = (0..depth)
                .map(|i| {
                    let mut why = create_test_why_iteration((i + 1) as u8, 0.5);
                    why.add_evidence(Evidence::new(
                        EvidenceSource::SATD,
                        PathBuf::from("test.rs"),
                        "satd".to_string(),
                        serde_json::json!({"count": count}),
                        "markers".to_string(),
                    ));
                    why
                })
                .collect();

            let summary = EvidenceSummary::from_whys(&whys);
            prop_assert_eq!(summary.satd_markers, count as usize);
        }

        #[test]
        fn prop_complexity_violations_count_correctly(
            values in proptest::collection::vec(0.0f64..100.0, 1..20)
        ) {
            let mut why = create_test_why_iteration(1, 0.5);

            for (i, value) in values.iter().enumerate() {
                why.add_evidence(Evidence::new(
                    EvidenceSource::Complexity,
                    PathBuf::from(format!("file{}.rs", i)),
                    "complexity".to_string(),
                    serde_json::json!({"value": value, "threshold": 20.0}),
                    "metrics".to_string(),
                ));
            }

            let summary = EvidenceSummary::from_whys(&[why]);
            let expected = values.iter().filter(|&&v| v > 20.0).count();
            prop_assert_eq!(summary.complexity_violations, expected);
        }

        #[test]
        fn prop_git_churn_threshold_at_10(commit_count in 0u64..100) {
            let mut why = create_test_why_iteration(1, 0.5);
            why.add_evidence(Evidence::new(
                EvidenceSource::GitChurn,
                PathBuf::from("test.rs"),
                "churn".to_string(),
                serde_json::json!({"commit_count": commit_count}),
                "churn".to_string(),
            ));

            let summary = EvidenceSummary::from_whys(&[why]);
            prop_assert_eq!(summary.git_churn_high, commit_count > 10);
        }

        #[test]
        fn prop_priority_serialization_stable(priority in prop::sample::select(vec![
            Priority::High,
            Priority::Medium,
            Priority::Low,
        ])) {
            let json = serde_json::to_string(&priority).unwrap();
            let roundtrip: Priority = serde_json::from_str(&json).expect("serde roundtrip");
            prop_assert_eq!(roundtrip, priority);
        }
    }

    // ============================================================================
    // Integration-Style Tests
    // ============================================================================

    #[test]
    fn test_complete_analysis_workflow() {
        // Simulate a complete Five Whys analysis workflow
        let mut analysis = DebugAnalysis::new("Memory leak in parser".to_string());

        // Why 1
        let mut why1 = WhyIteration::new(
            1,
            "Why is there a memory leak?".to_string(),
            "Buffer not freed after use".to_string(),
        )
        .with_confidence(0.6);
        why1.add_evidence(create_test_evidence(EvidenceSource::Complexity));
        analysis.whys.push(why1);

        // Why 2
        let mut why2 = WhyIteration::new(
            2,
            "Why is the buffer not freed?".to_string(),
            "Early return bypasses cleanup".to_string(),
        )
        .with_confidence(0.7);
        why2.add_evidence(create_test_evidence(EvidenceSource::SATD));
        analysis.whys.push(why2);

        // Why 3
        let mut why3 = WhyIteration::new(
            3,
            "Why does early return bypass cleanup?".to_string(),
            "Missing RAII pattern".to_string(),
        )
        .with_confidence(0.85);
        why3.add_evidence(create_test_evidence(EvidenceSource::TDG));
        analysis.whys.push(why3);

        // Set root cause
        analysis.root_cause = Some("Missing RAII pattern for resource management".to_string());

        // Add recommendations
        analysis.recommendations.push(Recommendation::high(
            "Implement Drop trait for buffer wrapper".to_string(),
            Some(PathBuf::from("src/parser/buffer.rs")),
        ));
        analysis.recommendations.push(Recommendation::medium(
            "Add unit tests for cleanup paths".to_string(),
            Some(PathBuf::from("tests/parser_tests.rs")),
        ));

        // Update evidence summary
        analysis.evidence_summary = EvidenceSummary::from_whys(&analysis.whys);

        // Verify the complete analysis
        assert_eq!(analysis.issue, "Memory leak in parser");
        assert_eq!(analysis.whys.len(), 3);
        assert!(analysis.root_cause.is_some());
        assert_eq!(analysis.recommendations.len(), 2);
        assert!(analysis.evidence_summary.complexity_violations > 0);
        assert!(analysis.evidence_summary.satd_markers > 0);
    }

    #[test]
    fn test_all_evidence_sources_in_summary() {
        let mut why = create_test_why_iteration(1, 0.5);

        // Add evidence from all sources
        why.add_evidence(create_test_evidence(EvidenceSource::Complexity));
        why.add_evidence(create_test_evidence(EvidenceSource::SATD));
        why.add_evidence(create_test_evidence(EvidenceSource::TDG));
        why.add_evidence(create_test_evidence(EvidenceSource::GitChurn));
        why.add_evidence(create_test_evidence(EvidenceSource::DeadCode));
        why.add_evidence(create_test_evidence(EvidenceSource::ManualInspection));

        let summary = EvidenceSummary::from_whys(&[why]);

        assert_eq!(summary.complexity_violations, 1);
        assert_eq!(summary.satd_markers, 5);
        assert!((summary.tdg_score - 40.0).abs() < f64::EPSILON);
        assert!(summary.git_churn_high);
    }
