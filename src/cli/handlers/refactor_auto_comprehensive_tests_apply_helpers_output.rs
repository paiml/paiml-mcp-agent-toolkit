    // Apply Refactoring Functions Tests

    #[tokio::test]
    async fn test_apply_complexity_reduction() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = apply_complexity_reduction(&test_file, "Reduce complexity").await;
        assert!(result.is_ok());
        let changes = result.unwrap();
        assert!(!changes.is_empty());
    }

    #[tokio::test]
    async fn test_apply_lint_fixes() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = apply_lint_fixes(&test_file, "Fix clippy warnings").await;
        assert!(result.is_ok());
        let changes = result.unwrap();
        assert!(!changes.is_empty());
    }

    #[tokio::test]
    async fn test_apply_satd_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = apply_satd_cleanup(&test_file, "Remove TODOs").await;
        assert!(result.is_ok());
        let changes = result.unwrap();
        assert!(!changes.is_empty());
    }

    #[tokio::test]
    async fn test_apply_coverage_improvements() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = apply_coverage_improvements(&test_file, "Add tests").await;
        assert!(result.is_ok());
        let changes = result.unwrap();
        assert!(!changes.is_empty());
    }

    #[tokio::test]
    async fn test_apply_security_fixes() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = apply_security_fixes(&test_file, "Fix security issue").await;
        assert!(result.is_ok());
        let changes = result.unwrap();
        assert!(!changes.is_empty());
    }

    // Helper Function Tests

    #[tokio::test]
    async fn test_get_single_file_lint_violations() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = get_single_file_lint_violations(&test_file).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_count_file_satd() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = count_file_satd(&test_file).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_analyze_file_complexity() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = analyze_file_complexity(&test_file).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_single_file_refactor_request() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = generate_single_file_refactor_request(
            &test_file,
            vec![],
            QualityMetrics::default(),
            0,
        );

        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.is_object());
    }

    // Markdown Analysis Tests

    #[tokio::test]
    async fn test_handle_markdown_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("README.md");
        std::fs::write(&md_file, "# Title\n\nContent here").unwrap();

        let result = handle_markdown_analysis(&md_file, RefactorAutoOutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_markdown_issues_valid() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("test.md");
        std::fs::write(&md_file, "# Header\n\n```rust\ncode\n```").unwrap();

        let result = analyze_markdown_issues(&md_file, "# Header\n\n```rust\ncode\n```");
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_markdown_issues_no_headers() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("test.md");

        let result = analyze_markdown_issues(&md_file, "No headers here");
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert!(issues.contains(&"Missing proper header structure"));
    }

    #[test]
    fn test_analyze_markdown_issues_unspecified_code_block() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("test.md");

        let result = analyze_markdown_issues(&md_file, "# Header\n\n```\ncode\n```");
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert!(issues.contains(&"Code blocks without language specification"));
    }

    #[test]
    fn test_create_markdown_refactor_request() {
        let path = Path::new("test.md");
        let issues = vec!["Issue 1", "Issue 2"];
        let content = "# Content";

        let result = create_markdown_refactor_request(path, &issues, content);
        assert!(result.is_object());
        assert_eq!(result["file_type"], "markdown");
    }

    #[test]
    fn test_print_markdown_summary() {
        let request = serde_json::json!({
            "issues": ["Issue 1", "Issue 2"]
        });

        // Should not panic
        print_markdown_summary(&request);
    }

    // Output Function Tests

    #[test]
    fn test_output_regular_file_results_json() {
        let request = serde_json::json!({
            "file": "test.rs",
            "refactoring_needed": true
        });

        // Should not panic
        output_regular_file_results(&request, RefactorAutoOutputFormat::Json);
    }

    #[test]
    fn test_output_regular_file_results_summary() {
        let request = serde_json::json!({
            "file": "test.rs",
            "refactoring_needed": false
        });

        // Should not panic
        output_regular_file_results(&request, RefactorAutoOutputFormat::Summary);
    }

    #[test]
    fn test_output_regular_file_results_detailed() {
        let request = serde_json::json!({
            "file": "test.rs",
            "violations": []
        });

        // Should not panic
        output_regular_file_results(&request, RefactorAutoOutputFormat::Detailed);
    }

    #[test]
    fn test_print_single_file_summary() {
        let request = serde_json::json!({});

        // Should not panic
        print_single_file_summary(&request);
    }

    #[test]
    fn test_print_single_file_detailed() {
        let request = serde_json::json!({});

        // Should not panic
        print_single_file_detailed(&request);
    }

    // Filter Successful Requests Tests

    #[test]
    fn test_filter_successful_requests_all_success() {
        let requests = vec![
            RefactoringRequest {
                request_type: RefactoringType::LintFix,
                target_file: PathBuf::from("test1.rs"),
                priority: RefactoringPriority::High,
                description: "Fix 1".to_string(),
                ai_instructions: "Instructions 1".to_string(),
                estimated_effort: RefactoringEffort::Trivial,
            },
            RefactoringRequest {
                request_type: RefactoringType::LintFix,
                target_file: PathBuf::from("test2.rs"),
                priority: RefactoringPriority::Medium,
                description: "Fix 2".to_string(),
                ai_instructions: "Instructions 2".to_string(),
                estimated_effort: RefactoringEffort::Minor,
            },
        ];

        let iteration_result = IterationResult {
            iteration_number: 1,
            successful_requests: vec![
                RefactoringSuccess {
                    request: requests[0].clone(),
                    changes_made: vec![],
                    application_duration: std::time::Duration::from_secs(1),
                    verification_status: VerificationStatus::Verified,
                },
                RefactoringSuccess {
                    request: requests[1].clone(),
                    changes_made: vec![],
                    application_duration: std::time::Duration::from_secs(1),
                    verification_status: VerificationStatus::Verified,
                },
            ],
            failed_requests: vec![],
            iteration_duration: std::time::Duration::from_secs(2),
            quality_improvement: QualityImprovement {
                complexity_reduced: 0,
                violations_fixed: 2,
                satd_resolved: 0,
                coverage_increased: 0.0,
                overall_score: 2.0,
            },
        };

        let remaining = filter_successful_requests(&requests, &iteration_result);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_filter_successful_requests_partial_success() {
        let requests = vec![
            RefactoringRequest {
                request_type: RefactoringType::LintFix,
                target_file: PathBuf::from("test1.rs"),
                priority: RefactoringPriority::High,
                description: "Fix 1".to_string(),
                ai_instructions: "Instructions 1".to_string(),
                estimated_effort: RefactoringEffort::Trivial,
            },
            RefactoringRequest {
                request_type: RefactoringType::LintFix,
                target_file: PathBuf::from("test2.rs"),
                priority: RefactoringPriority::Medium,
                description: "Fix 2".to_string(),
                ai_instructions: "Instructions 2".to_string(),
                estimated_effort: RefactoringEffort::Minor,
            },
        ];

        let iteration_result = IterationResult {
            iteration_number: 1,
            successful_requests: vec![RefactoringSuccess {
                request: requests[0].clone(),
                changes_made: vec![],
                application_duration: std::time::Duration::from_secs(1),
                verification_status: VerificationStatus::Verified,
            }],
            failed_requests: vec![],
            iteration_duration: std::time::Duration::from_secs(1),
            quality_improvement: QualityImprovement {
                complexity_reduced: 0,
                violations_fixed: 1,
                satd_resolved: 0,
                coverage_increased: 0.0,
                overall_score: 1.0,
            },
        };

        let remaining = filter_successful_requests(&requests, &iteration_result);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].target_file, PathBuf::from("test2.rs"));
    }

    // Broken Links Tests

    #[test]
    fn test_has_broken_relative_links_no_links() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("test.md");

        let result = has_broken_relative_links(&md_file, "No links here");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_has_broken_relative_links_with_broken_link() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("test.md");

        let result = has_broken_relative_links(&md_file, "See [docs](../nonexistent.md)");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_has_broken_relative_links_with_valid_link() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("test.md");
        let linked_file = temp_dir.path().join("other.md");
        std::fs::write(&linked_file, "# Other").unwrap();

        let result = has_broken_relative_links(&md_file, "See [other](./other.md)");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

