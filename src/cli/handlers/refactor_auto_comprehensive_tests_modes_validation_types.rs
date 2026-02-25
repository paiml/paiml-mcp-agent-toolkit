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
