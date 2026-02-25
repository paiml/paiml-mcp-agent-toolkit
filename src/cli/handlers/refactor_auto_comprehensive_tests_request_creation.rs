    // Create Lint Fix Requests Tests

    #[tokio::test]
    async fn test_create_lint_fix_requests_error_severity() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let violations = vec![ViolationDetailJson {
            file: PathBuf::from("test.rs"),
            line: 10,
            column: 5,
            end_line: 10,
            end_column: 15,
            lint_name: "clippy::unwrap_used".to_string(),
            message: "used unwrap on Result".to_string(),
            severity: "error".to_string(),
            suggestion: Some("Use ? operator".to_string()),
            machine_applicable: true,
        }];

        let result = create_lint_fix_requests(&violations, &context).await;
        assert!(result.is_ok());
        let requests = result.unwrap();
        assert_eq!(requests.len(), 1);
        assert!(matches!(requests[0].priority, RefactoringPriority::High));
    }

    #[tokio::test]
    async fn test_create_lint_fix_requests_warning_severity() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let violations = vec![ViolationDetailJson {
            file: PathBuf::from("test.rs"),
            line: 20,
            column: 1,
            end_line: 20,
            end_column: 10,
            lint_name: "dead_code".to_string(),
            message: "unused function".to_string(),
            severity: "warning".to_string(),
            suggestion: None,
            machine_applicable: false,
        }];

        let result = create_lint_fix_requests(&violations, &context).await;
        assert!(result.is_ok());
        let requests = result.unwrap();
        assert_eq!(requests.len(), 1);
        assert!(matches!(requests[0].priority, RefactoringPriority::Medium));
    }

    // Create SATD Cleanup Requests Tests

    #[tokio::test]
    async fn test_create_satd_cleanup_requests_fixme() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let satd_analysis = SatdAnalysis {
            satd_comments: vec![SatdComment {
                file: PathBuf::from("test.rs"),
                line_number: 5,
                comment_text: "FIXME: This needs to be fixed".to_string(),
                satd_type: "FIXME".to_string(),
            }],
            total_satd_count: 1,
            files_with_satd: 1,
        };

        let result = create_satd_cleanup_requests(&satd_analysis, &context).await;
        assert!(result.is_ok());
        let requests = result.unwrap();
        assert_eq!(requests.len(), 1);
        assert!(matches!(requests[0].priority, RefactoringPriority::High));
    }

    #[tokio::test]
    async fn test_create_satd_cleanup_requests_todo() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let satd_analysis = SatdAnalysis {
            satd_comments: vec![SatdComment {
                file: PathBuf::from("test.rs"),
                line_number: 10,
                comment_text: "TODO: Add tests".to_string(),
                satd_type: "TODO".to_string(),
            }],
            total_satd_count: 1,
            files_with_satd: 1,
        };

        let result = create_satd_cleanup_requests(&satd_analysis, &context).await;
        assert!(result.is_ok());
        let requests = result.unwrap();
        assert_eq!(requests.len(), 1);
        assert!(matches!(requests[0].priority, RefactoringPriority::Medium));
    }

    // Create Coverage Improvement Requests Tests

    #[tokio::test]
    async fn test_create_coverage_improvement_requests() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let coverage_analysis = CoverageAnalysis {
            overall_coverage_percent: 50.0,
            files_with_low_coverage: vec![
                PathBuf::from("src/main.rs"),
                PathBuf::from("src/lib.rs"),
            ],
            uncovered_lines: vec![],
        };

        let result = create_coverage_improvement_requests(&coverage_analysis, &context).await;
        assert!(result.is_ok());
        let requests = result.unwrap();
        assert_eq!(requests.len(), 2);
        for request in &requests {
            assert!(matches!(request.priority, RefactoringPriority::Medium));
            assert!(matches!(
                request.request_type,
                RefactoringType::CoverageImprovement
            ));
        }
    }

    // Calculate Quality Improvement Tests

    #[tokio::test]
    async fn test_calculate_quality_improvement_empty() {
        let result = calculate_quality_improvement(&[]).await;
        assert!(result.is_ok());
        let improvement = result.unwrap();
        assert_eq!(improvement.complexity_reduced, 0);
        assert_eq!(improvement.violations_fixed, 0);
        assert_eq!(improvement.satd_resolved, 0);
        assert_eq!(improvement.coverage_increased, 0.0);
        assert_eq!(improvement.overall_score, 0.0);
    }

    #[tokio::test]
    async fn test_calculate_quality_improvement_with_successes() {
        let successes = vec![
            RefactoringSuccess {
                request: RefactoringRequest {
                    request_type: RefactoringType::ComplexityReduction,
                    target_file: PathBuf::from("test.rs"),
                    priority: RefactoringPriority::High,
                    description: "Reduce complexity".to_string(),
                    ai_instructions: "Refactor".to_string(),
                    estimated_effort: RefactoringEffort::Moderate,
                },
                changes_made: vec!["Change 1".to_string()],
                application_duration: std::time::Duration::from_secs(1),
                verification_status: VerificationStatus::Verified,
            },
            RefactoringSuccess {
                request: RefactoringRequest {
                    request_type: RefactoringType::LintFix,
                    target_file: PathBuf::from("test2.rs"),
                    priority: RefactoringPriority::Medium,
                    description: "Fix lint".to_string(),
                    ai_instructions: "Fix".to_string(),
                    estimated_effort: RefactoringEffort::Trivial,
                },
                changes_made: vec!["Change 2".to_string()],
                application_duration: std::time::Duration::from_secs(1),
                verification_status: VerificationStatus::Verified,
            },
            RefactoringSuccess {
                request: RefactoringRequest {
                    request_type: RefactoringType::SatdCleanup,
                    target_file: PathBuf::from("test3.rs"),
                    priority: RefactoringPriority::Low,
                    description: "Clean SATD".to_string(),
                    ai_instructions: "Clean".to_string(),
                    estimated_effort: RefactoringEffort::Minor,
                },
                changes_made: vec!["Change 3".to_string()],
                application_duration: std::time::Duration::from_secs(1),
                verification_status: VerificationStatus::Verified,
            },
            RefactoringSuccess {
                request: RefactoringRequest {
                    request_type: RefactoringType::CoverageImprovement,
                    target_file: PathBuf::from("test4.rs"),
                    priority: RefactoringPriority::Medium,
                    description: "Add tests".to_string(),
                    ai_instructions: "Test".to_string(),
                    estimated_effort: RefactoringEffort::Moderate,
                },
                changes_made: vec!["Change 4".to_string()],
                application_duration: std::time::Duration::from_secs(1),
                verification_status: VerificationStatus::Verified,
            },
            RefactoringSuccess {
                request: RefactoringRequest {
                    request_type: RefactoringType::SecurityFix,
                    target_file: PathBuf::from("test5.rs"),
                    priority: RefactoringPriority::Critical,
                    description: "Fix security".to_string(),
                    ai_instructions: "Secure".to_string(),
                    estimated_effort: RefactoringEffort::Major,
                },
                changes_made: vec!["Change 5".to_string()],
                application_duration: std::time::Duration::from_secs(1),
                verification_status: VerificationStatus::Verified,
            },
        ];

        let result = calculate_quality_improvement(&successes).await;
        assert!(result.is_ok());
        let improvement = result.unwrap();
        assert_eq!(improvement.complexity_reduced, 1);
        assert_eq!(improvement.violations_fixed, 2); // LintFix + SecurityFix
        assert_eq!(improvement.satd_resolved, 1);
        assert_eq!(improvement.coverage_increased, 5.0);
    }

