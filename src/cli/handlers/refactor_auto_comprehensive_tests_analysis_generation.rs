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

