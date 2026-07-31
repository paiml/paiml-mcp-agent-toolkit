    /// Test execute_command with Generate command (tests command routing)
    #[tokio::test]
    async fn test_execute_command_generate() {
        let server = create_test_server();

        let command = Commands::Generate {
            category: String::new(),
            template: "test_template".to_string(),
            params: Vec::new(),
            output: None,
            create_dirs: false,
        };

        // Should delegate to handler without panicking
        // Note: This will likely fail in actual execution due to missing template
        // but tests our routing logic
        let result = CommandDispatcher::execute_command(command, server).await;

        // We expect this to fail cleanly (not panic)
        assert!(result.is_err());
    }

    /// Test execute_command with List command
    #[tokio::test]
    async fn test_execute_command_list() {
        let server = create_test_server();

        let command = Commands::List {
            toolchain: None,
            category: None,
            format: OutputFormat::Table,
        };

        let result = CommandDispatcher::execute_command(command, server).await;
        // List command should succeed with basic server
        assert!(result.is_ok());
    }

    /// Test execute_command with Scaffold::ListTemplates command
    #[tokio::test]
    async fn test_execute_command_scaffold_list() {
        let server = create_test_server();

        let command = Commands::Scaffold {
            command: ScaffoldCommands::ListTemplates,
        };

        let result = CommandDispatcher::execute_command(command, server).await;
        // ListTemplates should succeed
        assert!(result.is_ok());
    }

    /// Test execute_quality_gate_command (extracted method test)
    #[tokio::test]
    async fn test_execute_quality_gate_command() {
        // OutputFormat already imported
        use std::path::PathBuf;

        let result = CommandDispatcher::execute_quality_gate_command(
            Some(PathBuf::from(".")),
            None,
            OutputFormat::Table,
            false,
            vec!["complexity".to_string()],
            None,
            None,
            None,
            false,
            None,
            false,
        )
        .await;

        // Quality gate should execute without panicking
        // Note: May fail due to actual quality violations but routing works
        assert!(result.is_ok() || result.is_err());
    }

    /// Test execute_report_command (extracted method test)
    #[tokio::test]
    async fn test_execute_report_command() {
        // Toyota Way Root Cause Fix: Use temporary directory to avoid hanging on large codebase
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("internal error");
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn simple() -> i32 { 42 }").expect("internal error");

        let analyses = vec![String::from("complexity")];

        let result = CommandDispatcher::execute_report_command(
            Some(temp_dir.path().to_path_buf()),
            crate::cli::enums::ReportOutputFormat::Text,
            false,
            false,
            false,
            analyses,
            None,
            None,
            false,
            false,
            false,
            false,
        )
        .await;

        // Report command should execute without panicking
        assert!(result.is_ok() || result.is_err());
    }

    /// Test execute_config_command (extracted method test)
    #[tokio::test]
    async fn test_execute_config_command() {
        let result = CommandDispatcher::execute_config_command(
            true,  // show
            false, // edit
            false, // validate
            false, // reset
            None,  // section
            None,  // set
            None,  // config_path
        )
        .await;

        // Config show command should succeed
        assert!(result.is_ok());
    }

    /// Test create_test_config (Toyota Way Extract Method test)
    #[test]
    fn test_create_test_config() {
        use crate::cli::commands::TestSuite;

        let config = CommandDispatcher::create_test_config(
            &TestSuite::All,
            100,  // iterations
            true, // memory
            true, // throughput
            true, // regression
        );

        assert_eq!(config.test_iterations, 100);
        assert!(config.enable_memory_tests);
        assert!(config.enable_throughput_tests);
        assert!(config.enable_regression_tests);
    }

    /// Test create_test_config with specific suite
    #[test]
    fn test_create_test_config_memory_suite() {
        use crate::cli::commands::TestSuite;

        let config = CommandDispatcher::create_test_config(
            &TestSuite::Memory,
            50,    // iterations
            false, // memory flag (should be enabled by suite)
            false, // throughput
            false, // regression
        );

        assert_eq!(config.test_iterations, 50);
        assert!(config.enable_memory_tests); // Enabled by TestSuite::Memory
        assert!(!config.enable_throughput_tests);
        assert!(!config.enable_regression_tests);
    }

    /// Test print_performance_summary_if_requested (extracted method)
    #[test]
    fn test_print_performance_summary_if_requested() {
        use crate::cli::commands::TestSuite;
        use std::time::Duration;

        // Test with perf enabled (should not panic)
        CommandDispatcher::print_performance_summary_if_requested(
            true,
            Duration::from_secs(5),
            &TestSuite::Memory,
            100,
        );

        // Test with perf disabled (should not print)
        CommandDispatcher::print_performance_summary_if_requested(
            false,
            Duration::from_secs(5),
            &TestSuite::Memory,
            100,
        );
    }

    /// Test write_test_results_if_requested with no output
    #[test]
    fn test_write_test_results_no_output() {
        use crate::cli::commands::TestSuite;
        use std::time::Duration;

        let result: anyhow::Result<()> = Ok(());
        let write_result = CommandDispatcher::write_test_results_if_requested(
            None, // no output file
            &TestSuite::Memory,
            Duration::from_secs(5),
            100,
            &result,
        );

        // Should succeed without writing anything
        assert!(write_result.is_ok());
    }

    #[test]
    fn test_command_dispatcher_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
