    // ============================================================================
    // DebugAnalysis Unit Tests
    // ============================================================================

    #[test]
    fn test_debug_analysis_new_initializes_correctly() {
        let issue = "Memory leak in parser".to_string();
        let analysis = DebugAnalysis::new(issue.clone());

        assert_eq!(analysis.issue, issue);
        assert!(analysis.whys.is_empty());
        assert!(analysis.root_cause.is_none());
        assert!(analysis.recommendations.is_empty());
        assert_eq!(analysis.evidence_summary.complexity_violations, 0);
        assert_eq!(analysis.evidence_summary.satd_markers, 0);
        assert_eq!(analysis.evidence_summary.tdg_score, 0.0);
        assert!(!analysis.evidence_summary.git_churn_high);
    }

    #[test]
    fn test_debug_analysis_with_empty_issue() {
        let analysis = DebugAnalysis::new(String::new());
        assert_eq!(analysis.issue, "");
    }

    #[test]
    fn test_debug_analysis_with_unicode_issue() {
        let issue = "Error in 日本語 module: 🔥 critical failure".to_string();
        let analysis = DebugAnalysis::new(issue.clone());
        assert_eq!(analysis.issue, issue);
    }

    #[test]
    fn test_debug_analysis_serialization_roundtrip() {
        let mut analysis = create_test_debug_analysis();
        analysis.root_cause = Some("Root cause identified".to_string());
        analysis.whys.push(create_test_why_iteration(1, 0.8));
        analysis
            .recommendations
            .push(Recommendation::high("Fix the issue".to_string(), None));

        let json = serde_json::to_string(&analysis).expect("Serialization should succeed");
        let deserialized: DebugAnalysis =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        assert_eq!(deserialized.issue, analysis.issue);
        assert_eq!(deserialized.root_cause, analysis.root_cause);
        assert_eq!(deserialized.whys.len(), 1);
        assert_eq!(deserialized.recommendations.len(), 1);
    }

    #[test]
    fn test_debug_analysis_clone() {
        let mut analysis = create_test_debug_analysis();
        analysis.root_cause = Some("Test root cause".to_string());

        let cloned = analysis.clone();
        assert_eq!(cloned.issue, analysis.issue);
        assert_eq!(cloned.root_cause, analysis.root_cause);
    }

    // ============================================================================
    // WhyIteration Unit Tests
    // ============================================================================

    #[test]
    fn test_why_iteration_new_defaults() {
        let why = WhyIteration::new(1, "Why?".to_string(), "Because".to_string());

        assert_eq!(why.depth, 1);
        assert_eq!(why.question, "Why?");
        assert_eq!(why.hypothesis, "Because");
        assert!(why.evidence.is_empty());
        assert_eq!(why.confidence, 0.5); // Default confidence
    }

    #[test]
    fn test_why_iteration_with_confidence_valid_range() {
        let why = WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string())
            .with_confidence(0.75);
        assert_eq!(why.confidence, 0.75);
    }

    #[test]
    fn test_why_iteration_with_confidence_zero() {
        let why =
            WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string()).with_confidence(0.0);
        assert_eq!(why.confidence, 0.0);
    }

    #[test]
    fn test_why_iteration_with_confidence_one() {
        let why =
            WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string()).with_confidence(1.0);
        assert_eq!(why.confidence, 1.0);
    }

    #[test]
    fn test_why_iteration_with_confidence_clamps_above_one() {
        let why =
            WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string()).with_confidence(2.5);
        assert_eq!(why.confidence, 1.0);
    }

    #[test]
    fn test_why_iteration_with_confidence_clamps_below_zero() {
        let why = WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string())
            .with_confidence(-1.0);
        assert_eq!(why.confidence, 0.0);
    }

    #[test]
    fn test_why_iteration_add_evidence() {
        let mut why = WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string());
        assert!(why.evidence.is_empty());

        why.add_evidence(create_test_evidence(EvidenceSource::Complexity));
        assert_eq!(why.evidence.len(), 1);

        why.add_evidence(create_test_evidence(EvidenceSource::SATD));
        assert_eq!(why.evidence.len(), 2);
    }

    #[test]
    fn test_why_iteration_depth_range() {
        for depth in 1..=10 {
            let why = WhyIteration::new(
                depth,
                format!("Why {}?", depth),
                format!("Hypothesis {}", depth),
            );
            assert_eq!(why.depth, depth);
        }
    }

    #[test]
    fn test_why_iteration_serialization_roundtrip() {
        let mut why = create_test_why_iteration(3, 0.85);
        why.add_evidence(create_test_evidence(EvidenceSource::Complexity));
        why.add_evidence(create_test_evidence(EvidenceSource::TDG));

        let json = serde_json::to_string(&why).expect("Serialization should succeed");
        let deserialized: WhyIteration =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        assert_eq!(deserialized.depth, why.depth);
        assert_eq!(deserialized.question, why.question);
        assert_eq!(deserialized.hypothesis, why.hypothesis);
        assert_eq!(deserialized.confidence, why.confidence);
        assert_eq!(deserialized.evidence.len(), 2);
    }

    // ============================================================================
    // Evidence Unit Tests
    // ============================================================================

    #[test]
    fn test_evidence_new_complexity() {
        let evidence = create_test_evidence(EvidenceSource::Complexity);

        assert_eq!(evidence.source, EvidenceSource::Complexity);
        assert_eq!(evidence.file, PathBuf::from("src/test.rs"));
        assert_eq!(evidence.metric, "cyclomatic_complexity");
        assert!(evidence.value.get("value").is_some());
        assert!(evidence.value.get("threshold").is_some());
    }

    #[test]
    fn test_evidence_new_satd() {
        let evidence = create_test_evidence(EvidenceSource::SATD);

        assert_eq!(evidence.source, EvidenceSource::SATD);
        assert_eq!(evidence.metric, "todo_markers");
        assert_eq!(evidence.value.get("count").unwrap().as_u64(), Some(5));
    }

    #[test]
    fn test_evidence_new_tdg() {
        let evidence = create_test_evidence(EvidenceSource::TDG);

        assert_eq!(evidence.source, EvidenceSource::TDG);
        assert_eq!(evidence.value.as_f64(), Some(40.0));
    }

    #[test]
    fn test_evidence_new_git_churn() {
        let evidence = create_test_evidence(EvidenceSource::GitChurn);

        assert_eq!(evidence.source, EvidenceSource::GitChurn);
        assert_eq!(
            evidence.value.get("commit_count").unwrap().as_u64(),
            Some(15)
        );
        assert_eq!(evidence.value.get("days").unwrap().as_u64(), Some(30));
    }

    #[test]
    fn test_evidence_new_dead_code() {
        let evidence = create_test_evidence(EvidenceSource::DeadCode);

        assert_eq!(evidence.source, EvidenceSource::DeadCode);
        assert_eq!(evidence.file, PathBuf::from("src/unused.rs"));
    }

    #[test]
    fn test_evidence_new_manual_inspection() {
        let evidence = create_test_evidence(EvidenceSource::ManualInspection);

        assert_eq!(evidence.source, EvidenceSource::ManualInspection);
        assert!(evidence.value.get("notes").is_some());
    }

    #[test]
    fn test_evidence_serialization_roundtrip() {
        let evidence = create_test_evidence(EvidenceSource::Complexity);

        let json = serde_json::to_string(&evidence).expect("Serialization should succeed");
        let deserialized: Evidence =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        assert_eq!(deserialized.source, evidence.source);
        assert_eq!(deserialized.file, evidence.file);
        assert_eq!(deserialized.metric, evidence.metric);
        assert_eq!(deserialized.interpretation, evidence.interpretation);
    }

    #[test]
    fn test_evidence_with_null_value() {
        let evidence = Evidence::new(
            EvidenceSource::ManualInspection,
            PathBuf::from("test.rs"),
            "observation".to_string(),
            serde_json::Value::Null,
            "No additional data".to_string(),
        );

        assert!(evidence.value.is_null());
    }

    #[test]
    fn test_evidence_with_complex_json_value() {
        let complex_value = serde_json::json!({
            "nested": {
                "array": [1, 2, 3],
                "object": {"key": "value"}
            },
            "boolean": true
        });

        let evidence = Evidence::new(
            EvidenceSource::ManualInspection,
            PathBuf::from("test.rs"),
            "complex".to_string(),
            complex_value.clone(),
            "Complex structure".to_string(),
        );

        assert_eq!(evidence.value, complex_value);
    }

    // ============================================================================
    // EvidenceSource Unit Tests
    // ============================================================================

    #[test]
    fn test_evidence_source_equality() {
        assert_eq!(EvidenceSource::Complexity, EvidenceSource::Complexity);
        assert_ne!(EvidenceSource::Complexity, EvidenceSource::SATD);
    }

    #[test]
    fn test_evidence_source_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();

        set.insert(EvidenceSource::Complexity);
        set.insert(EvidenceSource::SATD);
        set.insert(EvidenceSource::TDG);
        set.insert(EvidenceSource::Complexity); // Duplicate

        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_evidence_source_all_variants_serialization() {
        let sources = vec![
            EvidenceSource::Complexity,
            EvidenceSource::SATD,
            EvidenceSource::DeadCode,
            EvidenceSource::GitChurn,
            EvidenceSource::TDG,
            EvidenceSource::ManualInspection,
        ];

        for source in sources {
            let json = serde_json::to_string(&source).expect("Serialization should succeed");
            let deserialized: EvidenceSource =
                serde_json::from_str(&json).expect("Deserialization should succeed");
            assert_eq!(deserialized, source);
        }
    }

    #[test]
    fn test_evidence_source_copy() {
        let source = EvidenceSource::Complexity;
        let copied = source;
        assert_eq!(source, copied);
    }
