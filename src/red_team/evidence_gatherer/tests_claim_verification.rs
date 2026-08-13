    // ==================== EvidenceGatherer Tests ====================

    #[test]
    fn test_evidence_gatherer_new_sets_defaults() {
        let gatherer = EvidenceGatherer::new();
        assert_eq!(gatherer.git_history_window_days, 30);
        assert!((gatherer.confidence_threshold - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_evidence_gatherer_default_trait() {
        let gatherer = EvidenceGatherer::default();
        assert_eq!(gatherer.git_history_window_days, 30);
        assert!((gatherer.confidence_threshold - 0.7).abs() < f64::EPSILON);
    }

    // ==================== Test Status Evidence Tests ====================

    #[test]
    fn test_gather_test_status_evidence_with_no_subsequent_commits() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::TestStatus,
            text: "all tests passing".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock();
        let evidence = gatherer.gather_evidence(&claim, &context);

        // Should have git history evidence (empty commits) and test results
        assert!(!evidence.is_empty());
    }

    #[test]
    fn test_gather_test_status_evidence_with_test_fixes() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::TestStatus,
            text: "all tests passing".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_subsequent_commits(vec![
            "fix test failure".to_string(),
            "update docs".to_string(),
        ]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        // Should find git history evidence that doesn't support claim
        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
        assert!((git_evidence.confidence - 0.85).abs() < 0.01);
        assert!(git_evidence
            .details
            .contains("1 subsequent test fixes found"));
    }

    #[test]
    fn test_gather_test_status_evidence_with_ignore_fixes() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::TestStatus,
            text: "all tests passing".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["fix: ignore flaky test".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
    }

    #[test]
    fn test_gather_test_status_evidence_with_test_results_all_passing() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::TestStatus,
            text: "all tests passing".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_test_results(true, 0);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let test_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::TestExecution)
            .expect("Should have test execution evidence");
        assert!(test_evidence.supports_claim);
        assert!((test_evidence.confidence - 0.9).abs() < 0.01);
        assert!(test_evidence.details.contains("All tests passing"));
    }

    #[test]
    fn test_gather_test_status_evidence_with_ignored_tests_absolute_claim() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::TestStatus,
            text: "all tests passing".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_test_results(true, 5); // 5 ignored tests
        let evidence = gatherer.gather_evidence(&claim, &context);

        let test_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::TestExecution)
            .expect("Should have test execution evidence");
        // Absolute claim should not be supported if tests are ignored
        assert!(!test_evidence.supports_claim);
        assert!(test_evidence.details.contains("5 tests ignored"));
    }

    #[test]
    fn test_gather_test_status_evidence_with_ignored_tests_qualified_claim() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::TestStatus,
            text: "tests passing".to_string(),
            is_absolute: false, // Not absolute
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_test_results(true, 5); // 5 ignored tests
        let evidence = gatherer.gather_evidence(&claim, &context);

        let test_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::TestExecution)
            .expect("Should have test execution evidence");
        // Qualified claim should be supported even with ignored tests
        assert!(test_evidence.supports_claim);
    }

    #[test]
    fn test_gather_test_status_evidence_with_failing_tests() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::TestStatus,
            text: "all tests passing".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_test_results(false, 0);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let test_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::TestExecution)
            .expect("Should have test execution evidence");
        assert!(!test_evidence.supports_claim);
        assert!(test_evidence.details.contains("Tests failing"));
    }

    // ==================== Documentation Evidence Tests ====================

    #[test]
    fn test_gather_documentation_evidence_with_doc_fixes() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Documentation,
            text: "fixed all broken links".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["docs: fix broken link".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
        assert!(git_evidence
            .details
            .contains("1 subsequent documentation fixes found"));
    }

    #[test]
    fn test_gather_documentation_evidence_with_404_fixes() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Documentation,
            text: "fixed all broken links".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["fix 404 in readme".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
    }

    #[test]
    fn test_gather_documentation_evidence_with_broken_links() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Documentation,
            text: "fixed all broken links".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_broken_links(3);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let link_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::LinkValidation)
            .expect("Should have link validation evidence");
        assert!(!link_evidence.supports_claim);
        assert!(link_evidence.details.contains("3 broken links found"));
    }

    #[test]
    fn test_gather_documentation_evidence_with_no_broken_links() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Documentation,
            text: "fixed all broken links".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_broken_links(0);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let link_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::LinkValidation)
            .expect("Should have link validation evidence");
        assert!(link_evidence.supports_claim);
        assert!(link_evidence.details.contains("All links valid"));
    }

    // ==================== Coverage Evidence Tests ====================

    #[test]
    fn test_gather_coverage_evidence_with_coverage_fixes() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Coverage,
            text: "coverage stable at 85%".to_string(),
            is_absolute: false,
            numeric_value: Some(85.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["fix coverage regression".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
    }

    #[test]
    fn test_gather_coverage_evidence_actual_matches_claimed() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Coverage,
            text: "coverage stable at 85%".to_string(),
            is_absolute: false,
            numeric_value: Some(85.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_coverage(85.5); // Within 2% tolerance
        let evidence = gatherer.gather_evidence(&claim, &context);

        let coverage_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::CoverageReport)
            .expect("Should have coverage report evidence");
        assert!(coverage_evidence.supports_claim);
        assert!((coverage_evidence.confidence - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_gather_coverage_evidence_actual_differs_from_claimed() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Coverage,
            text: "coverage stable at 85%".to_string(),
            is_absolute: false,
            numeric_value: Some(85.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_coverage(80.0); // Differs by 5%, outside tolerance
        let evidence = gatherer.gather_evidence(&claim, &context);

        let coverage_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::CoverageReport)
            .expect("Should have coverage report evidence");
        assert!(!coverage_evidence.supports_claim);
        assert!(coverage_evidence
            .details
            .contains("Claimed: 85.0%, Actual: 80.0%"));
    }

    #[test]
    fn test_gather_coverage_evidence_with_coverage_error() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Coverage,
            text: "coverage stable at 85%".to_string(),
            is_absolute: false,
            numeric_value: Some(85.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_coverage_error("Tool failed to run");
        let evidence = gatherer.gather_evidence(&claim, &context);

        // A coverage tool that failed measured nothing. This test used to
        // assert the failure was CoverageReport evidence with supports_claim
        // false — i.e. that a tool crash contradicted the claim.
        let coverage_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::NotMeasured)
            .expect("Should record that coverage was not measured");
        assert!(!coverage_evidence.measured());
        assert!(!coverage_evidence.contradicts());
        assert!(coverage_evidence.details.contains("Tool failed to run"));
    }

