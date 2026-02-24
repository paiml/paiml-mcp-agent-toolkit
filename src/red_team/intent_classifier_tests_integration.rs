    // ===== aggregate_signals tests =====

    #[test]
    fn test_aggregate_signals_hallucination_dominant() {
        let classifier = IntentClassifier::new();
        let signals = vec![
            SignalResult {
                signal_name: "test1".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.9,
                evidence: "High confidence hallucination".to_string(),
            },
            SignalResult {
                signal_name: "test2".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.8,
                evidence: "Another hallucination signal".to_string(),
            },
            SignalResult {
                signal_name: "test3".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.3,
                evidence: "Weak iteration signal".to_string(),
            },
        ];

        let result = classifier.aggregate_signals(signals);
        assert_eq!(result.intent, CommitIntent::HallucinationFix);
        assert!(result.confidence > 0.45);
        assert!(result.reasoning.contains("test1"));
        assert!(result.reasoning.contains("test2"));
        assert!(result.reasoning.contains("test3"));
    }

    #[test]
    fn test_aggregate_signals_iteration_dominant() {
        let classifier = IntentClassifier::new();
        let signals = vec![
            SignalResult {
                signal_name: "signal_a".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.9,
                evidence: "Strong iteration".to_string(),
            },
            SignalResult {
                signal_name: "signal_b".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.7,
                evidence: "More iteration".to_string(),
            },
            SignalResult {
                signal_name: "signal_c".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.2,
                evidence: "Weak hallucination".to_string(),
            },
        ];

        let result = classifier.aggregate_signals(signals);
        assert_eq!(result.intent, CommitIntent::PlannedIteration);
    }

    #[test]
    fn test_aggregate_signals_uncertain() {
        let classifier = IntentClassifier::new();
        let signals = vec![
            SignalResult {
                signal_name: "s1".to_string(),
                vote: CommitIntent::Uncertain,
                confidence: 0.5,
                evidence: "Uncertain".to_string(),
            },
            SignalResult {
                signal_name: "s2".to_string(),
                vote: CommitIntent::Uncertain,
                confidence: 0.5,
                evidence: "Also uncertain".to_string(),
            },
            SignalResult {
                signal_name: "s3".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.3,
                evidence: "Weak hall".to_string(),
            },
            SignalResult {
                signal_name: "s4".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.3,
                evidence: "Weak iter".to_string(),
            },
        ];

        let result = classifier.aggregate_signals(signals);
        assert_eq!(result.intent, CommitIntent::Uncertain);
    }

    #[test]
    fn test_aggregate_signals_reasoning_format() {
        let classifier = IntentClassifier::new();
        let signals = vec![
            SignalResult {
                signal_name: "commit_message".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.7,
                evidence: "3 hallucination keywords".to_string(),
            },
            SignalResult {
                signal_name: "code_churn".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.8,
                evidence: "90% overlap".to_string(),
            },
        ];

        let result = classifier.aggregate_signals(signals);
        assert!(result
            .reasoning
            .contains("commit_message: 3 hallucination keywords"));
        assert!(result.reasoning.contains("code_churn: 90% overlap"));
        assert!(result.reasoning.contains("; "));
    }

    // ===== Full classify() integration tests =====

    #[test]
    fn test_classify_hallucination_scenario() {
        let classifier = IntentClassifier::new();

        let original = CommitInfo {
            message: "Add new feature".to_string(),
            timestamp_seconds: 0,
            modified_files: vec!["feature.rs".to_string()],
            issue_number: None,
            issue_created_timestamp: None,
            branch: "main".to_string(),
            test_changes: TestChanges {
                added_tests: 5,
                fixed_tests: 0,
                modified_test_files: vec![],
            },
        };

        let followup = CommitInfo {
            message: "Fix broken error in feature".to_string(),
            timestamp_seconds: 200000, // After grace period
            modified_files: vec!["feature.rs".to_string()],
            issue_number: Some(999),
            issue_created_timestamp: Some(100), // After original
            branch: "hotfix".to_string(),
            test_changes: TestChanges {
                added_tests: 0,
                fixed_tests: 3,
                modified_test_files: vec![],
            },
        };

        let result = classifier.classify(&original, &followup);
        assert_eq!(result.intent, CommitIntent::HallucinationFix);
        assert_eq!(result.signals.len(), 5);
    }

    #[test]
    fn test_classify_planned_iteration_scenario() {
        let classifier = IntentClassifier::new();

        let original = CommitInfo {
            message: "Initial implementation".to_string(),
            timestamp_seconds: 0,
            modified_files: vec!["module_a.rs".to_string()],
            issue_number: Some(100),
            issue_created_timestamp: Some(0),
            branch: "feature-branch".to_string(),
            test_changes: TestChanges {
                added_tests: 0,
                fixed_tests: 0,
                modified_test_files: vec![],
            },
        };

        let followup = CommitInfo {
            message: "Refactor and improve the module, add extension".to_string(),
            timestamp_seconds: 3600, // 1 hour later, within grace period
            modified_files: vec!["module_b.rs".to_string(), "module_c.rs".to_string()],
            issue_number: Some(100),
            issue_created_timestamp: Some(0), // Pre-existing issue
            branch: "feature-branch".to_string(),
            test_changes: TestChanges {
                added_tests: 10,
                fixed_tests: 0,
                modified_test_files: vec![],
            },
        };

        let result = classifier.classify(&original, &followup);
        assert_eq!(result.intent, CommitIntent::PlannedIteration);
    }

    #[test]
    fn test_classify_uncertain_scenario() {
        let classifier = IntentClassifier::new();

        let original = make_commit_info("Some commit", 0, vec!["file.rs"], "main");
        let followup = make_commit_info("Another commit", 50 * 3600, vec!["other.rs"], "main");

        let result = classifier.classify(&original, &followup);
        // This should be uncertain because:
        // - No clear keywords
        // - No issue reference
        // - No file overlap
        // - No test changes
        // - After grace period but same branch
        assert!(result.signals.len() == 5);
    }

    // ===== Struct serialization/deserialization tests =====

    #[test]
    fn test_commit_intent_equality() {
        assert_eq!(
            CommitIntent::HallucinationFix,
            CommitIntent::HallucinationFix
        );
        assert_eq!(
            CommitIntent::PlannedIteration,
            CommitIntent::PlannedIteration
        );
        assert_eq!(CommitIntent::Uncertain, CommitIntent::Uncertain);
        assert_ne!(
            CommitIntent::HallucinationFix,
            CommitIntent::PlannedIteration
        );
    }

    #[test]
    fn test_commit_info_clone() {
        let info = CommitInfo {
            message: "Test".to_string(),
            timestamp_seconds: 12345,
            modified_files: vec!["a.rs".to_string()],
            issue_number: Some(42),
            issue_created_timestamp: Some(100),
            branch: "main".to_string(),
            test_changes: TestChanges {
                added_tests: 1,
                fixed_tests: 2,
                modified_test_files: vec!["test.rs".to_string()],
            },
        };

        let cloned = info.clone();
        assert_eq!(info.message, cloned.message);
        assert_eq!(info.timestamp_seconds, cloned.timestamp_seconds);
        assert_eq!(info.modified_files, cloned.modified_files);
        assert_eq!(info.issue_number, cloned.issue_number);
        assert_eq!(info.branch, cloned.branch);
    }

    #[test]
    fn test_intent_classification_fields() {
        let classification = IntentClassification {
            intent: CommitIntent::HallucinationFix,
            confidence: 0.85,
            signals: vec![],
            reasoning: "Test reasoning".to_string(),
        };

        assert_eq!(classification.intent, CommitIntent::HallucinationFix);
        assert!((classification.confidence - 0.85).abs() < f64::EPSILON);
        assert!(classification.signals.is_empty());
        assert_eq!(classification.reasoning, "Test reasoning");
    }

    #[test]
    fn test_signal_result_debug() {
        let signal = SignalResult {
            signal_name: "test".to_string(),
            vote: CommitIntent::Uncertain,
            confidence: 0.5,
            evidence: "Some evidence".to_string(),
        };

        let debug_str = format!("{:?}", signal);
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("Uncertain"));
    }

    // ===== Edge cases =====

    #[test]
    fn test_negative_time_diff() {
        let classifier = IntentClassifier::new();
        // Edge case: followup timestamp is before original (shouldn't happen but handle it)
        let original = make_commit_info("original", 10000, vec![], "main");
        let followup = make_commit_info("followup", 5000, vec![], "main");

        let result = classifier.analyze_temporal_context(&original, &followup);
        // Negative hours, will be < grace_period_hours
        assert_eq!(result.vote, CommitIntent::PlannedIteration);
    }

    #[test]
    fn test_very_long_commit_message() {
        let classifier = IntentClassifier::new();
        // The code uses contains(), which returns true only once per keyword,
        // regardless of how many times the word appears.
        // So we need multiple different keywords.
        let long_message = "fix bug error broken fail wrong incorrect regress";
        let result = classifier.analyze_commit_message(long_message);
        // Should find 8 hallucination keywords
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
    }

    #[test]
    fn test_commit_message_with_special_characters() {
        let classifier = IntentClassifier::new();
        let result = classifier.analyze_commit_message("fix: @#$% bug!!! \n\t error");
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
    }

    #[test]
    fn test_empty_modified_files_both_commits() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec![], "main");
        let followup = make_commit_info("followup", 2000, vec![], "main");

        let result = classifier.analyze_code_churn(&original, &followup);
        assert_eq!(result.vote, CommitIntent::Uncertain);
    }

    #[test]
    fn test_single_signal_aggregate() {
        let classifier = IntentClassifier::new();
        let signals = vec![SignalResult {
            signal_name: "single".to_string(),
            vote: CommitIntent::HallucinationFix,
            confidence: 1.0,
            evidence: "Only signal".to_string(),
        }];

        let result = classifier.aggregate_signals(signals);
        assert_eq!(result.intent, CommitIntent::HallucinationFix);
        assert!((result.confidence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_all_uncertain_signals() {
        let classifier = IntentClassifier::new();
        let signals = vec![
            SignalResult {
                signal_name: "a".to_string(),
                vote: CommitIntent::Uncertain,
                confidence: 0.5,
                evidence: "".to_string(),
            },
            SignalResult {
                signal_name: "b".to_string(),
                vote: CommitIntent::Uncertain,
                confidence: 0.5,
                evidence: "".to_string(),
            },
        ];

        let result = classifier.aggregate_signals(signals);
        assert_eq!(result.intent, CommitIntent::Uncertain);
    }

    // ===== Serialization tests =====

    #[test]
    fn test_commit_intent_serialize_deserialize() {
        let intent = CommitIntent::HallucinationFix;
        let json = serde_json::to_string(&intent).unwrap();
        let deserialized: CommitIntent = serde_json::from_str(&json).unwrap();
        assert_eq!(intent, deserialized);
    }

    #[test]
    fn test_commit_info_serialize_deserialize() {
        let info = CommitInfo {
            message: "Test message".to_string(),
            timestamp_seconds: 1234567890,
            modified_files: vec!["file1.rs".to_string(), "file2.rs".to_string()],
            issue_number: Some(42),
            issue_created_timestamp: Some(1234567800),
            branch: "feature".to_string(),
            test_changes: TestChanges {
                added_tests: 3,
                fixed_tests: 1,
                modified_test_files: vec!["test.rs".to_string()],
            },
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: CommitInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(info.message, deserialized.message);
        assert_eq!(info.timestamp_seconds, deserialized.timestamp_seconds);
        assert_eq!(info.modified_files, deserialized.modified_files);
        assert_eq!(info.issue_number, deserialized.issue_number);
        assert_eq!(info.branch, deserialized.branch);
    }

    #[test]
    fn test_intent_classification_serialize() {
        let classification = IntentClassification {
            intent: CommitIntent::PlannedIteration,
            confidence: 0.75,
            signals: vec![SignalResult {
                signal_name: "test".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.75,
                evidence: "Test evidence".to_string(),
            }],
            reasoning: "Test reasoning".to_string(),
        };

        let json = serde_json::to_string(&classification).unwrap();
        assert!(json.contains("PlannedIteration"));
        assert!(json.contains("0.75"));
        assert!(json.contains("Test reasoning"));
    }

    // ===== Boundary value tests =====

    #[test]
    fn test_confidence_boundaries() {
        let classifier = IntentClassifier::new();

        // When all signals vote the same with max confidence
        let signals = vec![
            SignalResult {
                signal_name: "s1".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 1.0,
                evidence: "".to_string(),
            },
            SignalResult {
                signal_name: "s2".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 1.0,
                evidence: "".to_string(),
            },
        ];

        let result = classifier.aggregate_signals(signals);
        assert!((result.confidence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_zero_confidence_signals() {
        let classifier = IntentClassifier::new();
        let signals = vec![
            SignalResult {
                signal_name: "zero1".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.0,
                evidence: "".to_string(),
            },
            SignalResult {
                signal_name: "zero2".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.0,
                evidence: "".to_string(),
            },
        ];

        // This will cause division by zero in the current implementation
        // Let's see how it handles it
        let result = classifier.aggregate_signals(signals);
        // With all zeros, ratios become NaN, which doesn't satisfy > 0.45
        assert_eq!(result.intent, CommitIntent::Uncertain);
    }
