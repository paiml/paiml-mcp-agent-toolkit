    // ==================== Security Evidence Tests ====================

    #[test]
    fn test_gather_security_evidence_no_vulnerabilities() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Security,
            text: "zero vulnerabilities".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_vulnerabilities(0);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let audit_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::CargoAudit)
            .expect("Should have cargo audit evidence");
        assert!(audit_evidence.supports_claim);
        assert!(audit_evidence.details.contains("No vulnerabilities found"));
    }

    #[test]
    fn test_gather_security_evidence_with_vulnerabilities() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Security,
            text: "zero vulnerabilities".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_vulnerabilities(3);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let audit_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::CargoAudit)
            .expect("Should have cargo audit evidence");
        assert!(!audit_evidence.supports_claim);
        assert!(audit_evidence.details.contains("3 vulnerabilities found"));
    }

    #[test]
    fn test_gather_security_evidence_with_security_fixes() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Security,
            text: "zero vulnerabilities".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["security: patch CVE-2024-1234".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
    }

    // ==================== RepositoryContext Builder Tests ====================

    #[test]
    fn test_repository_context_with_commit_timestamps() {
        let timestamps = vec![1000, 2000, 3000];
        let context = RepositoryContext::new_mock().with_commit_timestamps(timestamps.clone());

        assert_eq!(context.commit_timestamps, Some(timestamps));
        assert_eq!(context.latest_commit_timestamp, Some(3000));
    }

    #[test]
    fn test_repository_context_with_empty_commit_timestamps() {
        let context = RepositoryContext::new_mock().with_commit_timestamps(vec![]);

        assert_eq!(context.commit_timestamps, Some(vec![]));
        assert_eq!(context.latest_commit_timestamp, None);
    }

    #[test]
    fn test_repository_context_new_mock_defaults() {
        let context = RepositoryContext::new_mock();

        assert_eq!(context.subsequent_commits, Some(vec![]));
        assert_eq!(context.test_results, Some((true, 0)));
        assert_eq!(context.actual_coverage, None);
        assert_eq!(context.coverage_error, None);
        assert_eq!(context.broken_links_count, None);
        assert_eq!(context.vulnerabilities_count, None);
        assert_eq!(context.benchmark_results, None);
        assert_eq!(context.issue_status, None);
        assert_eq!(context.code_grep_results, None);
    }

    #[test]
    fn test_repository_context_has_git_history() {
        let context = RepositoryContext::new_mock();
        // Mock context has no git_repo set
        assert!(!context.has_git_history());
    }

    #[test]
    fn test_repository_context_has_coverage_report() {
        let context = RepositoryContext::new_mock();
        // Mock context has no coverage_path set
        assert!(!context.has_coverage_report());
    }

    #[test]
    fn test_repository_context_get_test_files() {
        let context = RepositoryContext::new_mock();
        // Mock context has empty test_files
        assert!(context.get_test_files().is_empty());
    }

    #[test]
    fn test_repository_context_get_coverage_percentage_no_report() {
        let context = RepositoryContext::new_mock();
        // Mock context has no coverage_path, should return 0.0
        assert!((context.get_coverage_percentage() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_repository_context_get_test_execution_info_no_results() {
        let context = RepositoryContext::new_mock();
        let info = context.get_test_execution_info();
        // Mock context has no test_results_path
        assert!(!info.has_results);
        assert_eq!(info.passed_count, 0);
        assert_eq!(info.failed_count, 0);
        assert_eq!(info.ignored_count, 0);
    }

    #[test]
    fn test_repository_context_grep_codebase() {
        let context = RepositoryContext::new_mock();
        // Build the needle at runtime so the joined form never appears in any
        // file. Spelling it as one literal made the test find *itself*: the
        // search root is ".", which includes this source file and every stale
        // copy of it under ./.claude/worktrees/, so the result was 26 files
        // while the comment claimed "should be empty" -- and the count varied
        // with how many worktrees the developer happened to have lying around.
        let needle = ["pmat", "absent", "needle", "7c1f"].join("_");
        let results = context.grep_codebase(&needle);
        assert!(
            results.is_empty(),
            "a pattern absent from the tree must match no files, got: {results:?}"
        );
    }

    #[test]
    fn test_repository_context_get_recent_commits_no_repo() {
        let context = RepositoryContext::new_mock();
        // Mock context has no git_repo set
        let commits = context.get_recent_commits(10);
        assert!(commits.is_empty());
    }

    // ==================== EvidenceSource Tests ====================

    #[test]
    fn test_evidence_source_serialization() {
        let sources = vec![
            EvidenceSource::GitHistory,
            EvidenceSource::TestExecution,
            EvidenceSource::CoverageReport,
            EvidenceSource::LinkValidation,
            EvidenceSource::CargoAudit,
            EvidenceSource::BenchmarkResults,
            EvidenceSource::IssueTracker,
            EvidenceSource::CodeGrep,
        ];

        for source in sources {
            let serialized = serde_json::to_string(&source).expect("Should serialize");
            let deserialized: EvidenceSource =
                serde_json::from_str(&serialized).expect("Should deserialize");
            assert_eq!(source, deserialized);
        }
    }

    // ==================== EvidenceResult Tests ====================

    #[test]
    fn test_evidence_result_serialization() {
        let result = EvidenceResult {
            source: EvidenceSource::GitHistory,
            supports_claim: true,
            confidence: 0.85,
            details: "No issues found".to_string(),
            timestamp: Some(1234567890),
        };

        let serialized = serde_json::to_string(&result).expect("Should serialize");
        let deserialized: EvidenceResult =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(result.source, deserialized.source);
        assert_eq!(result.supports_claim, deserialized.supports_claim);
        assert!((result.confidence - deserialized.confidence).abs() < f64::EPSILON);
        assert_eq!(result.details, deserialized.details);
        assert_eq!(result.timestamp, deserialized.timestamp);
    }

    // ==================== CommitInfo Tests ====================

    #[test]
    fn test_commit_info_fields() {
        let info = CommitInfo {
            message: "Fix bug".to_string(),
            timestamp: 1234567890,
            author: "Test Author".to_string(),
        };

        assert_eq!(info.message, "Fix bug");
        assert_eq!(info.timestamp, 1234567890);
        assert_eq!(info.author, "Test Author");
    }

    // ==================== TestExecutionInfo Tests ====================

    #[test]
    fn test_test_execution_info_default() {
        let info = TestExecutionInfo::default();

        assert!(!info.has_results);
        assert_eq!(info.passed_count, 0);
        assert_eq!(info.failed_count, 0);
        assert_eq!(info.ignored_count, 0);
    }
