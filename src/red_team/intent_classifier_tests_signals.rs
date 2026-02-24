    // Helper function to create a basic CommitInfo for testing
    fn make_commit_info(
        message: &str,
        timestamp: i64,
        files: Vec<&str>,
        branch: &str,
    ) -> CommitInfo {
        CommitInfo {
            message: message.to_string(),
            timestamp_seconds: timestamp,
            modified_files: files.into_iter().map(|s| s.to_string()).collect(),
            issue_number: None,
            issue_created_timestamp: None,
            branch: branch.to_string(),
            test_changes: TestChanges {
                added_tests: 0,
                fixed_tests: 0,
                modified_test_files: vec![],
            },
        }
    }

    // ===== IntentClassifier::new() tests =====

    #[test]
    fn test_intent_classifier_new_default_values() {
        let classifier = IntentClassifier::new();
        assert_eq!(classifier.grace_period_hours, 48);
        assert!((classifier.code_overlap_threshold - 0.8).abs() < f64::EPSILON);
        assert!(!classifier.hallucination_keywords.is_empty());
        assert!(!classifier.iteration_keywords.is_empty());
    }

    #[test]
    fn test_intent_classifier_default_trait() {
        let classifier = IntentClassifier::default();
        assert_eq!(classifier.grace_period_hours, 48);
    }

    #[test]
    fn test_hallucination_keywords_present() {
        let classifier = IntentClassifier::new();
        assert!(classifier
            .hallucination_keywords
            .contains(&"fix".to_string()));
        assert!(classifier
            .hallucination_keywords
            .contains(&"bug".to_string()));
        assert!(classifier
            .hallucination_keywords
            .contains(&"broken".to_string()));
        assert!(classifier
            .hallucination_keywords
            .contains(&"error".to_string()));
        assert!(classifier
            .hallucination_keywords
            .contains(&"regress".to_string()));
        assert!(classifier
            .hallucination_keywords
            .contains(&"fail".to_string()));
        assert!(classifier
            .hallucination_keywords
            .contains(&"incorrect".to_string()));
        assert!(classifier
            .hallucination_keywords
            .contains(&"wrong".to_string()));
    }

    #[test]
    fn test_iteration_keywords_present() {
        let classifier = IntentClassifier::new();
        assert!(classifier
            .iteration_keywords
            .contains(&"refactor".to_string()));
        assert!(classifier
            .iteration_keywords
            .contains(&"improve".to_string()));
        assert!(classifier
            .iteration_keywords
            .contains(&"enhance".to_string()));
        assert!(classifier
            .iteration_keywords
            .contains(&"optimize".to_string()));
        assert!(classifier
            .iteration_keywords
            .contains(&"cleanup".to_string()));
        assert!(classifier.iteration_keywords.contains(&"add".to_string()));
        assert!(classifier
            .iteration_keywords
            .contains(&"extend".to_string()));
    }

    // ===== analyze_commit_message tests =====

    #[test]
    fn test_commit_message_hallucination_keywords() {
        let classifier = IntentClassifier::new();
        let result = classifier.analyze_commit_message("Fix broken error in parser");
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
        assert!((result.confidence - 0.7).abs() < f64::EPSILON);
        assert_eq!(result.signal_name, "commit_message");
    }

    #[test]
    fn test_commit_message_iteration_keywords() {
        let classifier = IntentClassifier::new();
        let result = classifier.analyze_commit_message("Refactor and improve the optimizer");
        assert_eq!(result.vote, CommitIntent::PlannedIteration);
        assert!((result.confidence - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_commit_message_uncertain_no_keywords() {
        let classifier = IntentClassifier::new();
        let result = classifier.analyze_commit_message("Update documentation");
        assert_eq!(result.vote, CommitIntent::Uncertain);
        assert!((result.confidence - 0.3).abs() < f64::EPSILON);
        assert!(result.evidence.contains("No clear keyword pattern"));
    }

    #[test]
    fn test_commit_message_equal_keywords() {
        let classifier = IntentClassifier::new();
        // "fix" = hallucination, "add" = iteration -> equal count
        let result = classifier.analyze_commit_message("fix add");
        assert_eq!(result.vote, CommitIntent::Uncertain);
    }

    #[test]
    fn test_commit_message_case_insensitive() {
        let classifier = IntentClassifier::new();
        let result = classifier.analyze_commit_message("FIX BROKEN ERROR");
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
    }

    #[test]
    fn test_commit_message_empty() {
        let classifier = IntentClassifier::new();
        let result = classifier.analyze_commit_message("");
        assert_eq!(result.vote, CommitIntent::Uncertain);
    }

    // ===== analyze_issue_linkage tests =====

    #[test]
    fn test_issue_linkage_created_after_original() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec![], "main");
        let mut followup = make_commit_info("followup", 2000, vec![], "main");
        followup.issue_number = Some(42);
        followup.issue_created_timestamp = Some(1500); // After original commit

        let result = classifier.analyze_issue_linkage(&original, &followup);
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
        assert!((result.confidence - 0.9).abs() < f64::EPSILON);
        assert!(result
            .evidence
            .contains("Issue #42 created after original commit"));
    }

    #[test]
    fn test_issue_linkage_created_before_original() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec![], "main");
        let mut followup = make_commit_info("followup", 2000, vec![], "main");
        followup.issue_number = Some(123);
        followup.issue_created_timestamp = Some(500); // Before original commit

        let result = classifier.analyze_issue_linkage(&original, &followup);
        assert_eq!(result.vote, CommitIntent::PlannedIteration);
        assert!((result.confidence - 0.8).abs() < f64::EPSILON);
        assert!(result
            .evidence
            .contains("Issue #123 existed before original commit"));
    }

    #[test]
    fn test_issue_linkage_no_issue() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec![], "main");
        let followup = make_commit_info("followup", 2000, vec![], "main");

        let result = classifier.analyze_issue_linkage(&original, &followup);
        assert_eq!(result.vote, CommitIntent::Uncertain);
        assert!((result.confidence - 0.2).abs() < f64::EPSILON);
        assert!(result.evidence.contains("No issue reference"));
    }

    #[test]
    fn test_issue_linkage_issue_number_but_no_timestamp() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec![], "main");
        let mut followup = make_commit_info("followup", 2000, vec![], "main");
        followup.issue_number = Some(42);
        // No issue_created_timestamp

        let result = classifier.analyze_issue_linkage(&original, &followup);
        assert_eq!(result.vote, CommitIntent::Uncertain);
    }

    #[test]
    fn test_issue_linkage_issue_created_at_exact_same_time() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec![], "main");
        let mut followup = make_commit_info("followup", 2000, vec![], "main");
        followup.issue_number = Some(42);
        followup.issue_created_timestamp = Some(1000); // Same as original

        let result = classifier.analyze_issue_linkage(&original, &followup);
        // Not greater than, so it's PlannedIteration
        assert_eq!(result.vote, CommitIntent::PlannedIteration);
    }

    // ===== analyze_code_churn tests =====

    #[test]
    fn test_code_churn_high_overlap() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec!["a.rs", "b.rs", "c.rs"], "main");
        let followup = make_commit_info("followup", 2000, vec!["a.rs", "b.rs", "c.rs"], "main");

        let result = classifier.analyze_code_churn(&original, &followup);
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
        assert!((result.confidence - 0.8).abs() < f64::EPSILON);
        assert!(result.evidence.contains("100% file overlap"));
    }

    #[test]
    fn test_code_churn_low_overlap() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec!["a.rs"], "main");
        let followup = make_commit_info(
            "followup",
            2000,
            vec!["x.rs", "y.rs", "z.rs", "w.rs", "v.rs"],
            "main",
        );

        let result = classifier.analyze_code_churn(&original, &followup);
        assert_eq!(result.vote, CommitIntent::PlannedIteration);
        assert!((result.confidence - 0.7).abs() < f64::EPSILON);
        assert!(result.evidence.contains("overlap suggests new work"));
    }

    #[test]
    fn test_code_churn_moderate_overlap() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec!["a.rs", "b.rs"], "main");
        let followup = make_commit_info("followup", 2000, vec!["a.rs", "c.rs", "d.rs"], "main");
        // Overlap: 1 (a.rs), total: 3, ratio = 0.33

        let result = classifier.analyze_code_churn(&original, &followup);
        assert_eq!(result.vote, CommitIntent::Uncertain);
        assert!((result.confidence - 0.4).abs() < f64::EPSILON);
        assert!(result.evidence.contains("ambiguous"));
    }

    #[test]
    fn test_code_churn_no_modified_files() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec!["a.rs"], "main");
        let followup = make_commit_info("followup", 2000, vec![], "main");

        let result = classifier.analyze_code_churn(&original, &followup);
        assert_eq!(result.vote, CommitIntent::Uncertain);
        assert!((result.confidence - 0.1).abs() < f64::EPSILON);
        assert!(result.evidence.contains("No modified files"));
    }

    #[test]
    fn test_code_churn_exactly_at_threshold() {
        let classifier = IntentClassifier::new();
        // threshold is 0.8, so 80% overlap should still be HallucinationFix
        let original = make_commit_info(
            "original",
            1000,
            vec!["a.rs", "b.rs", "c.rs", "d.rs"],
            "main",
        );
        let followup = make_commit_info(
            "followup",
            2000,
            vec!["a.rs", "b.rs", "c.rs", "d.rs", "e.rs"],
            "main",
        );
        // Overlap: 4, total: 5, ratio = 0.8

        let result = classifier.analyze_code_churn(&original, &followup);
        // > 0.8 is false for 0.8, so it should be Uncertain
        assert_eq!(result.vote, CommitIntent::Uncertain);
    }

    #[test]
    fn test_code_churn_just_above_threshold() {
        let classifier = IntentClassifier::new();
        // 0.81 > 0.8
        let original = make_commit_info(
            "original",
            1000,
            vec![
                "a.rs", "b.rs", "c.rs", "d.rs", "e.rs", "f.rs", "g.rs", "h.rs", "i.rs",
            ],
            "main",
        );
        let followup = make_commit_info(
            "followup",
            2000,
            vec![
                "a.rs", "b.rs", "c.rs", "d.rs", "e.rs", "f.rs", "g.rs", "h.rs", "i.rs", "j.rs",
                "k.rs",
            ],
            "main",
        );
        // Overlap: 9, total: 11, ratio = 9/11 = 0.818

        let result = classifier.analyze_code_churn(&original, &followup);
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
    }

    // ===== analyze_test_changes tests =====

    #[test]
    fn test_test_changes_more_added_than_fixed() {
        let classifier = IntentClassifier::new();
        let test_changes = TestChanges {
            added_tests: 5,
            fixed_tests: 2,
            modified_test_files: vec!["tests/test_a.rs".to_string()],
        };

        let result = classifier.analyze_test_changes(&test_changes);
        assert_eq!(result.vote, CommitIntent::PlannedIteration);
        assert!((result.confidence - 0.7).abs() < f64::EPSILON);
        assert!(result.evidence.contains("expanding coverage"));
    }

    #[test]
    fn test_test_changes_more_fixed_than_added() {
        let classifier = IntentClassifier::new();
        let test_changes = TestChanges {
            added_tests: 1,
            fixed_tests: 4,
            modified_test_files: vec![],
        };

        let result = classifier.analyze_test_changes(&test_changes);
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
        assert!((result.confidence - 0.8).abs() < f64::EPSILON);
        assert!(result.evidence.contains("fixing broken tests"));
    }

    #[test]
    fn test_test_changes_equal() {
        let classifier = IntentClassifier::new();
        let test_changes = TestChanges {
            added_tests: 3,
            fixed_tests: 3,
            modified_test_files: vec![],
        };

        let result = classifier.analyze_test_changes(&test_changes);
        assert_eq!(result.vote, CommitIntent::Uncertain);
        assert!((result.confidence - 0.3).abs() < f64::EPSILON);
        assert!(result.evidence.contains("Equal test additions and fixes"));
    }

    #[test]
    fn test_test_changes_both_zero() {
        let classifier = IntentClassifier::new();
        let test_changes = TestChanges {
            added_tests: 0,
            fixed_tests: 0,
            modified_test_files: vec![],
        };

        let result = classifier.analyze_test_changes(&test_changes);
        assert_eq!(result.vote, CommitIntent::Uncertain);
    }

    // ===== analyze_temporal_context tests =====

    #[test]
    fn test_temporal_within_grace_period() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 0, vec![], "main");
        // 24 hours = 24 * 3600 = 86400 seconds
        let followup = make_commit_info("followup", 86400, vec![], "feature");

        let result = classifier.analyze_temporal_context(&original, &followup);
        assert_eq!(result.vote, CommitIntent::PlannedIteration);
        assert!((result.confidence - 0.8).abs() < f64::EPSILON);
        assert!(result.evidence.contains("Within 48-hour grace period"));
    }

    #[test]
    fn test_temporal_after_grace_same_branch() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 0, vec![], "main");
        // 72 hours = 72 * 3600 = 259200 seconds
        let followup = make_commit_info("followup", 259200, vec![], "main");

        let result = classifier.analyze_temporal_context(&original, &followup);
        assert_eq!(result.vote, CommitIntent::PlannedIteration);
        assert!((result.confidence - 0.6).abs() < f64::EPSILON);
        assert!(result.evidence.contains("Same branch"));
    }

    #[test]
    fn test_temporal_after_grace_different_branch() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 0, vec![], "main");
        // 100 hours = 100 * 3600 = 360000 seconds
        let followup = make_commit_info("followup", 360000, vec![], "hotfix");

        let result = classifier.analyze_temporal_context(&original, &followup);
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
        assert!((result.confidence - 0.4).abs() < f64::EPSILON);
        assert!(result.evidence.contains("after grace period"));
        assert!(result.evidence.contains("different branch"));
    }

    #[test]
    fn test_temporal_exactly_at_grace_period() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 0, vec![], "main");
        // 48 hours exactly
        let followup = make_commit_info("followup", 48 * 3600, vec![], "feature");

        let result = classifier.analyze_temporal_context(&original, &followup);
        // Not < 48, so it checks branch
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
    }
