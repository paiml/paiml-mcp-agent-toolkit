    // ==================== Feature Completion Evidence Tests ====================

    #[test]
    fn test_gather_feature_completion_evidence_with_fixes() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::FeatureCompletion,
            text: "complete implementation".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["fix: handle edge case".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
    }

    #[test]
    fn test_gather_feature_completion_evidence_with_reverts() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::FeatureCompletion,
            text: "complete implementation".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["revert: broken feature".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
    }

    #[test]
    fn test_gather_feature_completion_evidence_no_issues() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::FeatureCompletion,
            text: "complete implementation".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["chore: update deps".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(git_evidence.supports_claim);
        assert!(git_evidence.details.contains("No subsequent fixes found"));
    }

    // ==================== Migration Evidence Tests ====================

    #[test]
    fn test_gather_migration_evidence_with_rollbacks() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Migration,
            text: "migration to v2 complete".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["rollback: migration broke prod".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
        assert!(git_evidence.details.contains("1 rollback commits found"));
    }

    #[test]
    fn test_gather_migration_evidence_with_old_system_references() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Migration,
            text: "migration to v2 complete".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_code_grep_results("old_api", 5);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let grep_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::CodeGrep)
            .expect("Should have code grep evidence");
        assert!(!grep_evidence.supports_claim);
        assert!(grep_evidence
            .details
            .contains("5 files still reference 'old_api'"));
    }

    #[test]
    fn test_gather_migration_evidence_no_old_references() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Migration,
            text: "migration to v2 complete".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_code_grep_results("old_api", 0);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let grep_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::CodeGrep)
            .expect("Should have code grep evidence");
        assert!(grep_evidence.supports_claim);
        assert!(grep_evidence
            .details
            .contains("No references to 'old_api' found"));
    }

    // ==================== Bug Fix Evidence Tests ====================

    #[test]
    fn test_gather_bugfix_evidence_issue_closed() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::BugFix,
            text: "fixed bug #123".to_string(),
            is_absolute: false,
            numeric_value: None,
            issue_number: Some(123),
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_issue_status(123, "closed");
        let evidence = gatherer.gather_evidence(&claim, &context);

        let issue_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::IssueTracker)
            .expect("Should have issue tracker evidence");
        assert!(issue_evidence.supports_claim);
        assert!(issue_evidence.details.contains("Issue #123 is closed"));
    }

    #[test]
    fn test_gather_bugfix_evidence_issue_reopened() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::BugFix,
            text: "fixed bug #123".to_string(),
            is_absolute: false,
            numeric_value: None,
            issue_number: Some(123),
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_issue_status(123, "reopened");
        let evidence = gatherer.gather_evidence(&claim, &context);

        let issue_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::IssueTracker)
            .expect("Should have issue tracker evidence");
        assert!(!issue_evidence.supports_claim);
        assert!(issue_evidence.details.contains("Issue #123 was reopened"));
    }

    #[test]
    fn test_gather_bugfix_evidence_with_regressions() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::BugFix,
            text: "fixed bug #123".to_string(),
            is_absolute: false,
            numeric_value: None,
            issue_number: Some(123),
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["re-fix: regression in fix #123".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
        assert!(git_evidence.details.contains("1 regression commits found"));
    }

    // ==================== Performance Evidence Tests ====================

    #[test]
    fn test_gather_performance_evidence_with_benchmark_data() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Performance,
            text: "50% faster".to_string(),
            is_absolute: false,
            numeric_value: Some(50.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_benchmarks(Some("baseline: 100ms, new: 50ms".to_string()));
        let evidence = gatherer.gather_evidence(&claim, &context);

        let bench_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::BenchmarkResults)
            .expect("Should have benchmark evidence");
        assert!(bench_evidence.supports_claim);
        assert!(bench_evidence.details.contains("baseline: 100ms"));
    }

    #[test]
    fn test_gather_performance_evidence_without_benchmark_data() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Performance,
            text: "50% faster".to_string(),
            is_absolute: false,
            numeric_value: Some(50.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock();
        let evidence = gatherer.gather_evidence(&claim, &context);

        // This test used to require that absent benchmark data be reported as
        // BenchmarkResults evidence *against* the claim — the assertion that
        // made every Performance claim exit 1 (#958).
        let note = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::NotMeasured)
            .expect("Should record that no benchmark data was read");
        assert!(!note.contradicts());
        assert!(note.details.contains("no benchmark data was read"));
        assert!(!evidence.iter().any(EvidenceResult::contradicts));
    }

    #[test]
    fn test_gather_performance_evidence_with_regressions() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Performance,
            text: "50% faster".to_string(),
            is_absolute: false,
            numeric_value: Some(50.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_subsequent_commits(vec![
            "fix: performance regression causing timeout".to_string(),
        ]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
    }

    #[test]
    fn test_gather_performance_evidence_no_numeric_no_benchmark() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Performance,
            text: "performance improved".to_string(),
            is_absolute: false,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock();
        let evidence = gatherer.gather_evidence(&claim, &context);

        let note = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::NotMeasured)
            .expect("Should record that no benchmark data was read");
        assert!(!note.contradicts());
        assert!(note.details.contains("no benchmark data was read"));
    }

