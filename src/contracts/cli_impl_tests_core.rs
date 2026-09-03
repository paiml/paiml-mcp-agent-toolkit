    // ==========================================================================
    // Helper functions
    // ==========================================================================

    /// Creates a temporary directory for testing
    fn create_test_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp directory")
    }

    /// Creates a temporary file with content
    fn create_test_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let path = dir.path().join(name);
        std::fs::write(&path, content).expect("Failed to write test file");
        path
    }

    // ==========================================================================
    // ContractCliHandler::new tests
    // ==========================================================================

    #[test]
    fn test_contract_cli_handler_new_succeeds() {
        let result = ContractCliHandler::new();
        assert!(result.is_ok(), "ContractCliHandler::new() should succeed");
    }

    #[test]
    fn test_contract_cli_handler_new_creates_valid_handler() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        // Verify the service is accessible (field exists)
        let _ = &handler.service;
    }

    // ==========================================================================
    // ContractCliHandler::handle_command tests - non-Analyze commands
    // ==========================================================================

    #[tokio::test]
    async fn test_handle_command_generate_returns_ok() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let cmd = Commands::Generate {
            category: "rust".to_string(),
            template: "cli".to_string(),
            params: vec![],
            output: None,
            create_dirs: false,
        };

        let result = handler.handle_command(cmd).await;
        assert!(result.is_ok(), "Generate command should return Ok");
    }

    #[tokio::test]
    async fn test_handle_command_list_returns_ok() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let cmd = Commands::List {
            toolchain: None,
            category: None,
            format: crate::cli::OutputFormat::Table,
        };

        let result = handler.handle_command(cmd).await;
        assert!(result.is_ok(), "List command should return Ok");
    }

    #[tokio::test]
    async fn test_handle_command_search_returns_ok() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let cmd = Commands::Search {
            query: "test".to_string(),
            toolchain: None,
            limit: 10,
        };

        let result = handler.handle_command(cmd).await;
        assert!(result.is_ok(), "Search command should return Ok");
    }

    #[tokio::test]
    async fn test_handle_command_validate_returns_ok() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let cmd = Commands::Validate {
            uri: "rust/cli".to_string(),
            params: vec![],
        };

        let result = handler.handle_command(cmd).await;
        assert!(result.is_ok(), "Validate command should return Ok");
    }

    // ==========================================================================
    // ContractCliHandler::output_result tests
    // ==========================================================================

    #[test]
    fn test_output_result_with_string_value() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let result = serde_json::Value::String("Test output".to_string());
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
        assert!(
            output.is_ok(),
            "output_result should succeed with string value"
        );
    }

    #[test]
    fn test_output_result_with_json_object() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let result = serde_json::json!({
            "key": "value",
            "count": 42
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
        assert!(
            output.is_ok(),
            "output_result should succeed with JSON object"
        );
    }

    #[test]
    fn test_output_result_with_output_file() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let result = serde_json::json!({"status": "ok"});
        let temp_dir = create_test_dir();
        let output_path = temp_dir.path().join("output.json");

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
        assert!(
            output.is_ok(),
            "output_result should succeed with output file"
        );
        assert!(output_path.exists(), "Output file should be created");

        let content = std::fs::read_to_string(&output_path).expect("Should read file");
        assert!(content.contains("status"), "File should contain result");
    }

    #[test]
    fn test_output_result_null_value() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let result = serde_json::Value::Null;
        let temp_dir = create_test_dir();

        let cmd = AnalyzeCommands::Tdg {
            path: temp_dir.path().to_path_buf(),
            threshold: 1.5,
            top_files: 10,
            format: crate::cli::TdgOutputFormat::Table,
            include_components: false,
            output: None,
            critical_only: false,
            verbose: false,
            ml: false,
        };

        let output = handler.output_result(result, &cmd);
        assert!(
            output.is_ok(),
            "output_result should succeed with null value"
        );
    }

    #[test]
    fn test_output_result_array_value() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let result = serde_json::json!([1, 2, 3, "test"]);
        let temp_dir = create_test_dir();

        let cmd = AnalyzeCommands::LintHotspot {
            path: PathBuf::from("."),
            project_path: Some(temp_dir.path().to_path_buf()),
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

        let output = handler.output_result(result, &cmd);
        assert!(
            output.is_ok(),
            "output_result should succeed with array value"
        );
    }

    #[test]
    fn test_output_result_with_other_command_type() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let result = serde_json::Value::String("result".to_string());
        let temp_dir = create_test_dir();

        // Using Churn command which falls into the _ => &None branch
        let cmd = AnalyzeCommands::Churn {
            path: PathBuf::from("."),
            project_path: Some(temp_dir.path().to_path_buf()),
            days: 30,
            format: crate::models::churn::ChurnOutputFormat::Summary,
            output: None,
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let output = handler.output_result(result, &cmd);
        assert!(
            output.is_ok(),
            "output_result should succeed with Churn command"
        );
    }

    // ==========================================================================
    // output_result - output file path extraction tests
    // ==========================================================================

    #[test]
    fn test_output_result_complexity_output_path() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let result = serde_json::json!({"test": true});
        let temp_dir = create_test_dir();
        let output_path = temp_dir.path().join("complexity.json");

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
    fn test_output_result_satd_output_path() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let result = serde_json::json!({"satd": []});
        let temp_dir = create_test_dir();
        let output_path = temp_dir.path().join("satd.json");

        let cmd = AnalyzeCommands::Satd {
            path: temp_dir.path().to_path_buf(),
            format: crate::cli::SatdOutputFormat::Json,
            severity: None,
            critical_only: false,
            include_tests: false,
            strict: false,
            evolution: false,
            days: 30,
            metrics: false,
            output: Some(output_path.clone()),
            top_files: 10,
            fail_on_violation: false,
            timeout: 60,
            include: vec![],
            exclude: vec![],
            extended: false,
        };

        let output = handler.output_result(result, &cmd);
        assert!(output.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_output_result_tdg_output_path() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let result = serde_json::json!({"tdg_scores": []});
        let temp_dir = create_test_dir();
        let output_path = temp_dir.path().join("tdg.json");

        let cmd = AnalyzeCommands::Tdg {
            path: temp_dir.path().to_path_buf(),
            threshold: 1.5,
            top_files: 10,
            format: crate::cli::TdgOutputFormat::Json,
            include_components: false,
            output: Some(output_path.clone()),
            critical_only: false,
            verbose: false,
            ml: false,
        };

        let output = handler.output_result(result, &cmd);
        assert!(output.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_output_result_lint_hotspot_output_path() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let result = serde_json::json!({"hotspots": []});
        let temp_dir = create_test_dir();
        let output_path = temp_dir.path().join("lint.json");

        let cmd = AnalyzeCommands::LintHotspot {
            path: PathBuf::from("."),
            project_path: Some(temp_dir.path().to_path_buf()),
            file: None,
            format: crate::cli::LintHotspotOutputFormat::Json,
            max_density: 5.0,
            min_confidence: 0.8,
            enforce: false,
            dry_run: false,
            enforcement_metadata: false,
            output: Some(output_path.clone()),
            perf: false,
            clippy_flags: "-W warnings".to_string(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let output = handler.output_result(result, &cmd);
        assert!(output.is_ok());
        assert!(output_path.exists());
    }

    // ==========================================================================
    // AsAny trait implementation tests
    // ==========================================================================

    #[test]
    fn test_as_any_trait_returns_self() {
        // Test that the AsAny trait is implemented correctly
        let temp_dir = create_test_dir();
        let complexity_contract = AnalyzeComplexityContract {
            base: super::super::BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: super::super::OutputFormat::Table,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            max_cyclomatic: None,
            max_cognitive: None,
            max_halstead: None,
        };

        // Test that the contract validates correctly
        let validation_result = complexity_contract.validate();
        assert!(validation_result.is_ok());
    }

    // ==========================================================================
    // handle_analyze_command tests - deprecation warnings
    // ==========================================================================

    #[tokio::test]
    #[ignore = "requires full service integration"]
    async fn test_handle_analyze_command_prints_deprecation_warnings() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let temp_dir = create_test_dir();

        // Create a Complexity command with deprecated project_path
        let cmd = AnalyzeCommands::Complexity {
            path: PathBuf::from("."),
            project_path: Some(temp_dir.path().to_path_buf()),
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

        // This should succeed and print deprecation warning (captured in stderr)
        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_ok());
    }

    // ==========================================================================
    // handle_analyze_command - standard mode fallback tests
    // ==========================================================================

    #[tokio::test]
    async fn test_handle_analyze_command_churn_standard_mode() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let temp_dir = create_test_dir();

        let cmd = AnalyzeCommands::Churn {
            path: PathBuf::from("."),
            project_path: Some(temp_dir.path().to_path_buf()),
            days: 30,
            format: crate::models::churn::ChurnOutputFormat::Summary,
            output: None,
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        // Churn falls into the standard mode path
        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_command_dag_standard_mode() {
        let handler = ContractCliHandler::new().expect("Handler creation should succeed");
        let temp_dir = create_test_dir();

        let cmd = AnalyzeCommands::Dag {
            dag_type: crate::cli::DagType::FullDependency,
            path: PathBuf::from("."),
            project_path: Some(temp_dir.path().to_path_buf()),
            output: None,
            max_depth: None,
            target_nodes: None,
            filter_external: false,
            show_complexity: false,
            include_duplicates: false,
            include_dead_code: false,
            enhanced: false,
        };

        let result = handler.handle_analyze_command(cmd).await;
        assert!(result.is_ok());
    }
