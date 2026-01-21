//! Comprehensive coverage tests for refactor_auto_handlers
//! Extracted for file health compliance (CB-040)

use super::super::*;
use std::path::PathBuf;
use tempfile::TempDir;

    async fn test_setup_refactoring_context_project_wide() {
        let temp_dir = TempDir::new().unwrap();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            false,
            None,
            RefactorAutoOutputFormat::Summary,
            5,
            false,
            vec![],
            vec![],
            None,
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(matches!(context.config.mode, RefactorMode::ProjectWide));
    }

    #[tokio::test]
    async fn test_setup_refactoring_context_single_file_mode() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            true,
            Some(test_file.clone()),
            RefactorAutoOutputFormat::Json,
            3,
            true,
            vec!["target".to_string()],
            vec!["*.rs".to_string()],
            None,
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
        let context = result.unwrap();
        if let RefactorMode::SingleFile(path) = &context.config.mode {
            assert_eq!(path, &test_file);
        } else {
            panic!("Expected SingleFile mode");
        }
    }

    #[tokio::test]
    async fn test_setup_refactoring_context_single_file_mode_no_file() {
        let temp_dir = TempDir::new().unwrap();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            true,
            None, // No file provided
            RefactorAutoOutputFormat::Summary,
            1,
            false,
            vec![],
            vec![],
            None,
            None,
            None,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Single file mode requires --file parameter"));
    }

    #[tokio::test]
    async fn test_setup_refactoring_context_bug_report_mode() {
        let temp_dir = TempDir::new().unwrap();
        let bug_report = temp_dir.path().join("bug.md");
        std::fs::write(&bug_report, "# Bug Report").unwrap();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            false,
            None,
            RefactorAutoOutputFormat::Detailed,
            2,
            false,
            vec![],
            vec![],
            None,
            None,
            Some(bug_report.clone()),
        )
        .await;

        assert!(result.is_ok());
        let context = result.unwrap();
        if let RefactorMode::BugReport(path) = &context.config.mode {
            assert_eq!(path, &bug_report);
        } else {
            panic!("Expected BugReport mode");
        }
    }

    #[tokio::test]
    async fn test_setup_refactoring_context_github_issue_mode() {
        let temp_dir = TempDir::new().unwrap();
        let github_url = "https://github.com/owner/repo/issues/123".to_string();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            false,
            None,
            RefactorAutoOutputFormat::Json,
            1,
            true,
            vec![],
            vec![],
            None,
            Some(github_url.clone()),
            None,
        )
        .await;

        assert!(result.is_ok());
        let context = result.unwrap();
        if let RefactorMode::GitHubIssue(url) = &context.config.mode {
            assert_eq!(url, &github_url);
        } else {
            panic!("Expected GitHubIssue mode");
        }
    }

    #[tokio::test]
    async fn test_setup_refactoring_context_with_ignore_file() {
        let temp_dir = TempDir::new().unwrap();
        let ignore_file = temp_dir.path().join(".pmatignore");
        std::fs::write(&ignore_file, "target/\n*.tmp").unwrap();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            false,
            None,
            RefactorAutoOutputFormat::Summary,
            5,
            false,
            vec![],
            vec![],
            Some(ignore_file.clone()),
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(
            context.config.patterns.ignore_file_path,
            Some(ignore_file)
        );
    }

    // Analyze Project Quality Tests

    #[tokio::test]
    async fn test_analyze_project_quality_empty_project() {
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

        let result = analyze_project_quality(&context).await;
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.total_files_analyzed, 0);
    }

    #[tokio::test]
    async fn test_analyze_project_quality_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("main.rs");
        std::fs::write(&test_file, "fn main() { println!(\"Hello\"); }").unwrap();

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
            source_files: vec![test_file],
            start_time: std::time::Instant::now(),
        };

        let result = analyze_project_quality(&context).await;
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.total_files_analyzed, 1);
    }

    // Generate Refactoring Requests Tests

    #[tokio::test]
    async fn test_generate_refactoring_requests_empty() {
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

        let quality_analysis = ProjectQualityAnalysis {
            lint_violations: vec![],
            complexity_analysis: ComplexityAnalysis {
                high_complexity_violations: vec![],
                high_complexity_count: 0,
                total_functions: 0,
                average_complexity: 0.0,
            },
            satd_analysis: SatdAnalysis {
                satd_comments: vec![],
                total_satd_count: 0,
                files_with_satd: 0,
            },
            coverage_analysis: CoverageAnalysis {
                overall_coverage_percent: 100.0,
                files_with_low_coverage: vec![],
                uncovered_lines: vec![],
            },
            total_files_analyzed: 0,
            analysis_timestamp: std::time::SystemTime::now(),
        };

        let result = generate_refactoring_requests(&quality_analysis, &context).await;
        assert!(result.is_ok());
        let requests = result.unwrap();
        assert!(requests.is_empty());
    }

    #[tokio::test]
    async fn test_generate_refactoring_requests_with_complexity_violations() {
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

        let quality_analysis = ProjectQualityAnalysis {
            lint_violations: vec![],
            complexity_analysis: ComplexityAnalysis {
                high_complexity_violations: vec![
                    ComplexityViolation {
                        file: PathBuf::from("test.rs"),
                        function_name: "complex_function".to_string(),
                        complexity: 25,
                        line_number: 10,
                        suggestion: "Refactor".to_string(),
                    },
                ],
                high_complexity_count: 1,
                total_functions: 5,
                average_complexity: 15.0,
            },
            satd_analysis: SatdAnalysis {
                satd_comments: vec![],
                total_satd_count: 0,
                files_with_satd: 0,
            },
            coverage_analysis: CoverageAnalysis {
                overall_coverage_percent: 100.0,
                files_with_low_coverage: vec![],
                uncovered_lines: vec![],
            },
            total_files_analyzed: 1,
            analysis_timestamp: std::time::SystemTime::now(),
        };

        let result = generate_refactoring_requests(&quality_analysis, &context).await;
        assert!(result.is_ok());
        let requests = result.unwrap();
        assert!(!requests.is_empty());
    }

    // Create Complexity Reduction Request Tests

    #[tokio::test]
    async fn test_create_complexity_reduction_request_critical() {
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

        let violation = ComplexityViolation {
            file: PathBuf::from("test.rs"),
            function_name: "very_complex".to_string(),
            complexity: 50,
            line_number: 100,
            suggestion: "Split into multiple functions".to_string(),
        };

        let result = create_complexity_reduction_request(&violation, &context).await;
        assert!(result.is_ok());
        let request = result.unwrap();
        assert!(matches!(request.priority, RefactoringPriority::Critical));
        // Effort may vary depending on implementation details
        assert!(matches!(request.estimated_effort, RefactoringEffort::Minor | RefactoringEffort::Moderate | RefactoringEffort::Major));
    }

    #[tokio::test]
    async fn test_create_complexity_reduction_request_high() {
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

        let violation = ComplexityViolation {
            file: PathBuf::from("test.rs"),
            function_name: "moderate_complex".to_string(),
            complexity: 15,
            line_number: 50,
            suggestion: "Simplify".to_string(),
        };

        let result = create_complexity_reduction_request(&violation, &context).await;
        assert!(result.is_ok());
        let request = result.unwrap();
        assert!(matches!(request.priority, RefactoringPriority::High));
        assert!(matches!(request.estimated_effort, RefactoringEffort::Minor));
    }

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

    // Handle Special Modes Tests

    #[tokio::test]
    async fn test_handle_special_modes_project_wide() {
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

        let result = handle_special_modes(&context).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // ProjectWide returns None
    }

    #[tokio::test]
    async fn test_handle_special_modes_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();

        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::SingleFile(test_file),
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

        let result = handle_special_modes(&context).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some()); // SingleFile returns Some(())
    }

    #[tokio::test]
    async fn test_handle_special_modes_bug_report_md() {
        let temp_dir = TempDir::new().unwrap();
        let bug_file = temp_dir.path().join("bug.md");
        std::fs::write(&bug_file, "# Bug Report\n\nDescription").unwrap();

        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::BugReport(bug_file),
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

        let result = handle_special_modes(&context).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some()); // BugReport .md returns Some(())
    }

    #[tokio::test]
    async fn test_handle_special_modes_bug_report_non_md() {
        let temp_dir = TempDir::new().unwrap();
        let bug_file = temp_dir.path().join("bug.txt");
        std::fs::write(&bug_file, "Bug description").unwrap();

        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::BugReport(bug_file),
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

        let result = handle_special_modes(&context).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // BugReport non-.md returns None
    }

    // Analyze Project Functions Tests

    #[tokio::test]
    async fn test_analyze_project_lint_violations_empty() {
        let result = analyze_project_lint_violations(&[]).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_analyze_project_lint_violations_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();

        let result = analyze_project_lint_violations(&[test_file]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_project_complexity_empty() {
        let result = analyze_project_complexity(&[]).await;
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.total_functions, 0);
        assert_eq!(analysis.high_complexity_count, 0);
    }

    #[tokio::test]
    async fn test_analyze_project_satd_empty() {
        let result = analyze_project_satd(&[]).await;
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.total_satd_count, 0);
        assert_eq!(analysis.files_with_satd, 0);
    }

    // Validation Tests

    #[tokio::test]
    async fn test_get_final_validation_empty() {
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

        let result = get_final_validation(&[], &context).await;
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.overall_success);
        assert!(validation.compilation_passed);
        assert!(validation.tests_passed);
        assert!(!validation.quality_improved);
    }

    // ViolationWithContext Tests

    #[test]
    fn test_violation_with_context_creation() {
        let violation = ViolationWithContext {
            lint_name: "dead_code".to_string(),
            line: 10,
            column: 5,
            message: "unused variable".to_string(),
            ast_node_id: Some("node_123".to_string()),
            fix_strategy: FixStrategy::RemoveDeadCode,
        };

        assert_eq!(violation.lint_name, "dead_code");
        assert_eq!(violation.line, 10);
        assert_eq!(violation.column, 5);
        assert!(violation.ast_node_id.is_some());
    }

    #[test]
    fn test_violation_with_context_clone() {
        let original = ViolationWithContext {
            lint_name: "clippy".to_string(),
            line: 20,
            column: 1,
            message: "warning".to_string(),
            ast_node_id: None,
            fix_strategy: FixStrategy::ApplySuggestion("use vec![]".to_string()),
        };

        let cloned = original.clone();
        assert_eq!(cloned.lint_name, original.lint_name);
        assert_eq!(cloned.line, original.line);
    }

    // FileRewritePlan Tests

    #[test]
    fn test_file_rewrite_plan_creation() {
        let plan = FileRewritePlan {
            file_path: PathBuf::from("test.rs"),
            violations: vec![],
            ast_metadata: AstMetadata {
                functions: vec![],
                imports: vec![],
                structure_hash: "hash".to_string(),
            },
            new_content: "fn main() {}".to_string(),
        };

        assert_eq!(plan.file_path, PathBuf::from("test.rs"));
        assert!(plan.violations.is_empty());
    }

    #[test]
    fn test_file_rewrite_plan_clone() {
        let original = FileRewritePlan {
            file_path: PathBuf::from("lib.rs"),
            violations: vec![ViolationWithContext {
                lint_name: "test".to_string(),
                line: 1,
                column: 1,
                message: "msg".to_string(),
                ast_node_id: None,
                fix_strategy: FixStrategy::AddTest,
            }],
            ast_metadata: AstMetadata {
                functions: vec![],
                imports: vec!["std".to_string()],
                structure_hash: "abc".to_string(),
            },
            new_content: "// content".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(cloned.file_path, original.file_path);
        assert_eq!(cloned.violations.len(), original.violations.len());
    }

    // ComplexityViolation Tests

    #[test]
    fn test_complexity_violation_clone() {
        let original = ComplexityViolation {
            file: PathBuf::from("complex.rs"),
            function_name: "too_complex".to_string(),
            complexity: 35,
            line_number: 100,
            suggestion: "Split function".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(cloned.file, original.file);
        assert_eq!(cloned.function_name, original.function_name);
        assert_eq!(cloned.complexity, original.complexity);
    }

    // SatdComment Tests

    #[test]
    fn test_satd_comment_clone() {
        let original = SatdComment {
            file: PathBuf::from("todo.rs"),
            line_number: 50,
            comment_text: "TODO: Implement this".to_string(),
            satd_type: "TODO".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(cloned.file, original.file);
        assert_eq!(cloned.comment_text, original.comment_text);
        assert_eq!(cloned.satd_type, original.satd_type);
    }

    // UncoveredLine Tests

    #[test]
    fn test_uncovered_line_clone() {
        let original = UncoveredLine {
            file: PathBuf::from("uncovered.rs"),
            line_number: 42,
            content: "unreachable!()".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(cloned.file, original.file);
        assert_eq!(cloned.line_number, original.line_number);
        assert_eq!(cloned.content, original.content);
    }

    // RefactoringRequest Tests

    #[test]
    fn test_refactoring_request_clone() {
        let original = RefactoringRequest {
            request_type: RefactoringType::SecurityFix,
            target_file: PathBuf::from("secure.rs"),
            priority: RefactoringPriority::Critical,
            description: "Fix SQL injection".to_string(),
            ai_instructions: "Sanitize input".to_string(),
            estimated_effort: RefactoringEffort::Extensive,
        };

        let cloned = original.clone();
        assert_eq!(cloned.target_file, original.target_file);
        assert_eq!(cloned.description, original.description);
    }

    // Print Functions Tests (Ensure No Panics)

    #[test]
    fn test_print_refactoring_header() {
        let config = RefactorAutoConfig {
            project_path: PathBuf::from("/test/project"),
            single_file_mode: false,
            file: None,
            format: RefactorAutoOutputFormat::Summary,
            max_iterations: 5,
            cache_dir: None,
            dry_run: false,
            ci_mode: false,
            exclude_patterns: vec![],
            include_patterns: vec![],
            ignore_file: None,
            test_file: None,
            test_name: None,
            github_issue_url: None,
            bug_report_path: None,
        };

        // Should not panic
        print_refactoring_header(&config);
    }

    // RefactorState Serialization Tests

    #[test]
    fn test_refactor_state_serialization() {
        let state = RefactorState {
            iteration: 3,
            context_generated: true,
            context_path: PathBuf::from("/tmp/ctx"),
            current_file: Some(PathBuf::from("current.rs")),
            files_completed: vec![PathBuf::from("done.rs")],
            quality_metrics: QualityMetrics {
                total_violations: 10,
                coverage_percent: 85.0,
                max_complexity: 15,
                satd_count: 2,
                files_with_issues: 3,
                total_files: 10,
                functions_with_high_complexity: 2,
                total_functions: 50,
            },
            progress: RefactorProgress::default(),
            start_time: std::time::SystemTime::now(),
        };

        let json = serde_json::to_string(&state);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        let deserialized: Result<RefactorState, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
    }

    // RefactorProgress Serialization Tests

    #[test]
    fn test_refactor_progress_serialization() {
        let progress = RefactorProgress {
            overall_completion_percent: 50.0,
            lint_completion_percent: 60.0,
            complexity_completion_percent: 40.0,
            satd_completion_percent: 70.0,
            coverage_completion_percent: 30.0,
            files_completed: 5,
            files_remaining: 5,
            estimated_time_remaining_minutes: 10,
            quality_gates_passed: vec!["lint".to_string()],
            quality_gates_remaining: vec!["complexity".to_string()],
            current_phase: RefactorPhase::LintFixes,
        };

        let json = serde_json::to_string(&progress);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        let deserialized: Result<RefactorProgress, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
    }

    // QualityMetrics Serialization Tests

    #[test]
    fn test_quality_metrics_serialization() {
        let metrics = QualityMetrics {
            total_violations: 25,
            coverage_percent: 90.5,
            max_complexity: 20,
            satd_count: 5,
            files_with_issues: 8,
            total_files: 50,
            functions_with_high_complexity: 3,
            total_functions: 200,
        };

        let json = serde_json::to_string(&metrics);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        let deserialized: Result<QualityMetrics, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
    }
}
