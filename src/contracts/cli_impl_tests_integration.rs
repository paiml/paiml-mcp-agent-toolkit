    // ==========================================================================
    // Integration-like tests for analysis handlers
    // These tests require full service setup and are marked as ignored for unit tests
    // ==========================================================================

    #[tokio::test]
    #[ignore = "requires full service integration"]
    async fn test_handle_complexity_analysis_integration() {
        // This test verifies the error path when contract downcasting fails
        // In practice this shouldn't happen with correct adapter implementation
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let temp_dir = create_test_dir();

        // Create a valid Rust file for analysis
        create_test_file(&temp_dir, "test.rs", "fn main() { println!(\"Hello\"); }");

        let cmd = AnalyzeCommands::Complexity {
            path: temp_dir.path().to_path_buf(),
            project_path: None,
            file: None,
            files: vec![],
            toolchain: None,
            format: crate::cli::ComplexityOutputFormat::Summary,
            output: None,
            max_cyclomatic: None,
            max_cognitive: None,
            include: vec![],
            watch: false,
            top_files: 10,
            fail_on_violation: false,
            timeout: 60,
            ml: false,
        };

        // This should succeed as the adapter creates the correct contract type
        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires full service integration"]
    async fn test_handle_satd_analysis_integration() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let temp_dir = create_test_dir();

        // Create a file with SATD comments
        create_test_file(
            &temp_dir,
            "test.rs",
            "// TODO: Fix this later\nfn main() { /* FIXME: urgent */ }",
        );

        let cmd = AnalyzeCommands::Satd {
            path: temp_dir.path().to_path_buf(),
            format: crate::cli::SatdOutputFormat::Summary,
            severity: None,
            critical_only: false,
            include_tests: false,
            strict: false,
            evolution: false,
            days: 30,
            metrics: false,
            output: None,
            top_files: 10,
            fail_on_violation: false,
            timeout: 60,
            include: vec![],
            exclude: vec![],
            extended: false,
        };

        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires full service integration"]
    async fn test_handle_dead_code_analysis_integration() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let temp_dir = create_test_dir();

        create_test_file(
            &temp_dir,
            "test.rs",
            "\nfn unused() {}\nfn main() {}",
        );

        let cmd = AnalyzeCommands::DeadCode {
            path: temp_dir.path().to_path_buf(),
            format: crate::cli::DeadCodeOutputFormat::Summary,
            top_files: Some(10),
            include_unreachable: false,
            min_dead_lines: 1,
            include_tests: false,
            output: None,
            fail_on_violation: false,
            max_percentage: 50.0,
            timeout: 60,
            include: vec![],
            exclude: vec![],
            max_depth: 8,
            no_cache: false,
        };

        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires full service integration"]
    async fn test_handle_tdg_analysis_integration() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let temp_dir = create_test_dir();

        create_test_file(&temp_dir, "lib.rs", "pub fn foo() -> i32 { 42 }");

        let cmd = AnalyzeCommands::Tdg {
            path: temp_dir.path().to_path_buf(),
            threshold: 0.0,
            top_files: 10,
            format: crate::cli::TdgOutputFormat::Table,
            include_components: false,
            output: None,
            critical_only: false,
            verbose: false,
            ml: false,
        };

        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires full service integration"]
    async fn test_handle_lint_hotspot_analysis_integration() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let temp_dir = create_test_dir();

        create_test_file(&temp_dir, "main.rs", "fn main() { println!(\"test\"); }");

        let cmd = AnalyzeCommands::LintHotspot {
            path: PathBuf::from("."),
            project_path: Some(temp_dir.path().to_path_buf()),
            file: None,
            format: crate::cli::LintHotspotOutputFormat::Summary,
            max_density: 10.0,
            min_confidence: 0.5,
            enforce: false,
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: "-W warnings".to_string(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_ok());
    }

    // ==========================================================================
    // Error condition tests
    // ==========================================================================

    #[tokio::test]
    async fn test_handle_complexity_analysis_invalid_path() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");

        let cmd = AnalyzeCommands::Complexity {
            path: PathBuf::from("/nonexistent/path/that/does/not/exist"),
            project_path: None,
            file: None,
            files: vec![],
            toolchain: None,
            format: crate::cli::ComplexityOutputFormat::Summary,
            output: None,
            max_cyclomatic: None,
            max_cognitive: None,
            include: vec![],
            watch: false,
            top_files: 10,
            fail_on_violation: false,
            timeout: 60,
            ml: false,
        };

        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_err(), "Should fail with invalid path");
    }

    #[tokio::test]
    async fn test_handle_satd_analysis_invalid_path() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");

        let cmd = AnalyzeCommands::Satd {
            path: PathBuf::from("/nonexistent/satd/path"),
            format: crate::cli::SatdOutputFormat::Summary,
            severity: None,
            critical_only: false,
            include_tests: false,
            strict: false,
            evolution: false,
            days: 30,
            metrics: false,
            output: None,
            top_files: 10,
            fail_on_violation: false,
            timeout: 60,
            include: vec![],
            exclude: vec![],
            extended: false,
        };

        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_err(), "Should fail with invalid path");
    }

    #[tokio::test]
    async fn test_handle_dead_code_analysis_invalid_path() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");

        let cmd = AnalyzeCommands::DeadCode {
            path: PathBuf::from("/nonexistent/dead/code/path"),
            format: crate::cli::DeadCodeOutputFormat::Summary,
            top_files: Some(10),
            include_unreachable: false,
            min_dead_lines: 10,
            include_tests: false,
            output: None,
            fail_on_violation: false,
            max_percentage: 15.0,
            timeout: 60,
            include: vec![],
            exclude: vec![],
            max_depth: 8,
            no_cache: false,
        };

        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_err(), "Should fail with invalid path");
    }

    #[tokio::test]
    async fn test_handle_tdg_analysis_invalid_path() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");

        let cmd = AnalyzeCommands::Tdg {
            path: PathBuf::from("/nonexistent/tdg/path"),
            threshold: 1.5,
            top_files: 10,
            format: crate::cli::TdgOutputFormat::Table,
            include_components: false,
            output: None,
            critical_only: false,
            verbose: false,
            ml: false,
        };

        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_err(), "Should fail with invalid path");
    }

    #[tokio::test]
    async fn test_handle_lint_hotspot_analysis_invalid_path() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");

        let cmd = AnalyzeCommands::LintHotspot {
            path: PathBuf::from("."),
            project_path: Some(PathBuf::from("/nonexistent/lint/path")),
            file: None,
            format: crate::cli::LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.8,
            enforce: false,
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: "-W warnings".to_string(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_err(), "Should fail with invalid path");
    }

    // ==========================================================================
    // Edge case tests
    // ==========================================================================

    #[test]
    fn test_output_result_write_to_nested_dir() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let result = serde_json::json!({"nested": "test"});
        let temp_dir = create_test_dir();

        // Create nested directory path
        let nested_dir = temp_dir.path().join("nested").join("subdir");
        std::fs::create_dir_all(&nested_dir).expect("Failed to create nested dirs");
        let output_path = nested_dir.join("output.json");

        let cmd = AnalyzeCommands::Complexity {
            path: temp_dir.path().to_path_buf(),
            project_path: None,
            file: None,
            files: vec![],
            toolchain: None,
            format: crate::cli::ComplexityOutputFormat::Json,
            output: Some(output_path.clone()),
            max_cyclomatic: None,
            max_cognitive: None,
            include: vec![],
            watch: false,
            top_files: 10,
            fail_on_violation: false,
            timeout: 60,
            ml: false,
        };

        let output = handler.output_result(result, &cmd);
        assert!(output.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_output_result_with_empty_json_object() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let result = serde_json::json!({});
        let temp_dir = create_test_dir();

        let cmd = AnalyzeCommands::Complexity {
            path: temp_dir.path().to_path_buf(),
            project_path: None,
            file: None,
            files: vec![],
            toolchain: None,
            format: crate::cli::ComplexityOutputFormat::Summary,
            output: None,
            max_cyclomatic: None,
            max_cognitive: None,
            include: vec![],
            watch: false,
            top_files: 10,
            fail_on_violation: false,
            timeout: 60,
            ml: false,
        };

        let output = handler.output_result(result, &cmd);
        assert!(output.is_ok());
    }

    #[test]
    fn test_output_result_with_deeply_nested_json() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let result = serde_json::json!({
            "level1": {
                "level2": {
                    "level3": {
                        "value": 123
                    }
                }
            }
        });
        let temp_dir = create_test_dir();

        let cmd = AnalyzeCommands::Satd {
            path: temp_dir.path().to_path_buf(),
            format: crate::cli::SatdOutputFormat::Summary,
            severity: None,
            critical_only: false,
            include_tests: false,
            strict: false,
            evolution: false,
            days: 30,
            metrics: false,
            output: None,
            top_files: 10,
            fail_on_violation: false,
            timeout: 60,
            include: vec![],
            exclude: vec![],
            extended: false,
        };

        let output = handler.output_result(result, &cmd);
        assert!(output.is_ok());
    }

    #[test]
    fn test_output_result_with_special_characters_in_string() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let result =
            serde_json::Value::String("Test with\nnewlines\tand\ttabs \"quotes\"".to_string());
        let temp_dir = create_test_dir();

        let cmd = AnalyzeCommands::Complexity {
            path: temp_dir.path().to_path_buf(),
            project_path: None,
            file: None,
            files: vec![],
            toolchain: None,
            format: crate::cli::ComplexityOutputFormat::Summary,
            output: None,
            max_cyclomatic: None,
            max_cognitive: None,
            include: vec![],
            watch: false,
            top_files: 10,
            fail_on_violation: false,
            timeout: 60,
            ml: false,
        };

        let output = handler.output_result(result, &cmd);
        assert!(output.is_ok());
    }

    #[test]
    fn test_output_result_with_unicode_content() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let result = serde_json::json!({
            "unicode": "Hello, \u{4e16}\u{754c}! \u{1f600}"
        });
        let temp_dir = create_test_dir();
        let output_path = temp_dir.path().join("unicode.json");

        let cmd = AnalyzeCommands::DeadCode {
            path: temp_dir.path().to_path_buf(),
            format: crate::cli::DeadCodeOutputFormat::Json,
            top_files: Some(10),
            include_unreachable: false,
            min_dead_lines: 10,
            include_tests: false,
            output: Some(output_path.clone()),
            fail_on_violation: false,
            max_percentage: 15.0,
            timeout: 60,
            include: vec![],
            exclude: vec![],
            max_depth: 8,
            no_cache: false,
        };

        let output = handler.output_result(result, &cmd);
        assert!(output.is_ok());
        assert!(output_path.exists());
    }

    // ==========================================================================
    // Commands::Context test (not analyzed by ContractCliHandler)
    // ==========================================================================

    #[tokio::test]
    async fn test_handle_command_context_returns_ok() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let temp_dir = create_test_dir();

        let cmd = Commands::Context {
            toolchain: None,
            project_path: temp_dir.path().to_path_buf(),
            output: None,
            format: crate::cli::ContextFormat::Markdown,
            include_large_files: false,
            skip_expensive_metrics: false,
            language: None,
            languages: None,
        };

        let result = handler.handle_command(cmd).await;
        assert!(result.is_ok());
    }

    // ==========================================================================
    // Tests with various AnalyzeCommands configurations
    // These tests require full service integration
    // ==========================================================================

    #[tokio::test]
    #[ignore = "requires full service integration"]
    async fn test_handle_complexity_with_all_options() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let temp_dir = create_test_dir();
        create_test_file(&temp_dir, "src.rs", "fn complex() { if true { } else { } }");
        let output_path = temp_dir.path().join("out.json");

        let cmd = AnalyzeCommands::Complexity {
            path: temp_dir.path().to_path_buf(),
            project_path: None,
            file: None,
            files: vec![],
            toolchain: Some("rust".to_string()),
            format: crate::cli::ComplexityOutputFormat::Json,
            output: Some(output_path),
            max_cyclomatic: Some(10),
            max_cognitive: Some(15),
            include: vec!["**/*.rs".to_string()],
            watch: false,
            top_files: 5,
            fail_on_violation: true,
            timeout: 120,
            ml: false,
        };

        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires full service integration"]
    async fn test_handle_satd_with_strict_mode() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let temp_dir = create_test_dir();
        create_test_file(&temp_dir, "code.rs", "// TODO: implement\nfn stub() {}");

        let cmd = AnalyzeCommands::Satd {
            path: temp_dir.path().to_path_buf(),
            format: crate::cli::SatdOutputFormat::Json,
            severity: Some(crate::cli::SatdSeverity::High),
            critical_only: true,
            include_tests: true,
            strict: true,
            evolution: false,
            days: 30,
            metrics: true,
            output: None,
            top_files: 20,
            fail_on_violation: true,
            timeout: 90,
            include: vec![],
            exclude: vec!["target/**".to_string()],
            extended: false,
        };

        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires full service integration"]
    async fn test_handle_dead_code_with_include_unreachable() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let temp_dir = create_test_dir();
        create_test_file(
            &temp_dir,
            "dead.rs",
            "fn used() {}\nfn unused() { unreachable!(); }",
        );

        let cmd = AnalyzeCommands::DeadCode {
            path: temp_dir.path().to_path_buf(),
            format: crate::cli::DeadCodeOutputFormat::Json,
            top_files: None,
            include_unreachable: true,
            min_dead_lines: 1,
            include_tests: true,
            output: None,
            fail_on_violation: true,
            max_percentage: 5.0,
            timeout: 30,
            include: vec![],
            exclude: vec![],
            max_depth: 5,
            no_cache: false,
        };

        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires full service integration"]
    async fn test_handle_tdg_with_critical_only() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let temp_dir = create_test_dir();
        create_test_file(&temp_dir, "tdg.rs", "pub fn api() -> i32 { 1 + 2 }");

        let cmd = AnalyzeCommands::Tdg {
            path: temp_dir.path().to_path_buf(),
            threshold: 2.5,
            top_files: 5,
            format: crate::cli::TdgOutputFormat::Json,
            include_components: true,
            output: None,
            critical_only: true,
            verbose: true,
            ml: false,
        };

        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires full service integration"]
    async fn test_handle_lint_hotspot_with_enforce() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let temp_dir = create_test_dir();
        create_test_file(&temp_dir, "lint.rs", "fn test() { let x = 1; }");

        let cmd = AnalyzeCommands::LintHotspot {
            path: PathBuf::from("."),
            project_path: Some(temp_dir.path().to_path_buf()),
            file: None,
            format: crate::cli::LintHotspotOutputFormat::Json,
            max_density: 2.0,
            min_confidence: 0.9,
            enforce: true,
            dry_run: true,
            enforcement_metadata: true,
            output: None,
            perf: true,
            clippy_flags: "-W clippy::pedantic".to_string(),
            top_files: 15,
            include: vec!["src/**".to_string()],
            exclude: vec![],
        };

        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires full service integration"]
    async fn test_handle_lint_hotspot_with_specific_file() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let temp_dir = create_test_dir();
        let file_path = create_test_file(&temp_dir, "specific.rs", "fn main() {}");

        let cmd = AnalyzeCommands::LintHotspot {
            path: PathBuf::from("."),
            project_path: Some(temp_dir.path().to_path_buf()),
            file: Some(file_path),
            format: crate::cli::LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.8,
            enforce: false,
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: "-W warnings".to_string(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_ok());
    }
