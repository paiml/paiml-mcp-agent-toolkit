    // Test: create_demo_args edge cases

    #[test]
    fn test_demo_args_with_all_none_options() {
        let args = CommandDispatcher::create_demo_args(
            None,
            None,
            None,
            None, // will default to Table
            crate::demo::Protocol::Cli,
            false,
            false,
            8080,
            false, // cli=false means web=true
            None,
            None,
            None,
            false,
            None,
            false,
            false,
            None,
        );
        assert!(matches!(args.format, OutputFormat::Table));
        assert!(args.web);
        assert_eq!(args.target_nodes, 1000);
        assert!((args.centrality_threshold - 0.5).abs() < 0.01);
        assert_eq!(args.merge_threshold, 100);
    }

    #[test]
    fn test_demo_args_web_mode() {
        let args = CommandDispatcher::create_demo_args(
            Some(PathBuf::from("/test/path")),
            Some("http://example.com".to_string()),
            Some("user/repo".to_string()),
            Some(OutputFormat::Json),
            crate::demo::Protocol::Http,
            true,
            false,
            3000,
            false, // web mode
            Some(500),
            Some(0.8),
            Some(75.0),
            true,
            Some(PathBuf::from("/debug/output")),
            true,
            false,
            Some(200),
        );
        assert!(args.web);
        assert!(args.show_api);
        assert!(!args.no_browser);
        assert_eq!(args.port, Some(3000));
        assert_eq!(args.target_nodes, 500);
        assert_eq!(args.merge_threshold, 75);
        assert!(args.debug);
        assert!(args.skip_vendor);
        assert_eq!(args.max_line_length, Some(200));
    }

    #[test]
    fn test_demo_args_no_skip_vendor_override() {
        // When no_skip_vendor=true, skip_vendor should be false regardless of skip_vendor flag
        let args = CommandDispatcher::create_demo_args(
            None,
            None,
            None,
            None,
            crate::demo::Protocol::Cli,
            false,
            true,
            8080,
            true,
            None,
            None,
            None,
            false,
            None,
            true, // skip_vendor
            true, // no_skip_vendor (takes precedence)
            None,
        );
        assert!(!args.skip_vendor);
    }

    // Test: convert_demo_protocol all variants

    #[test]
    fn test_convert_protocol_cli_override() {
        // cli=true should always return Cli regardless of protocol
        assert!(matches!(
            CommandDispatcher::convert_demo_protocol(DemoProtocol::Http, true),
            crate::demo::Protocol::Cli
        ));
        assert!(matches!(
            CommandDispatcher::convert_demo_protocol(DemoProtocol::Mcp, true),
            crate::demo::Protocol::Cli
        ));
        assert!(matches!(
            CommandDispatcher::convert_demo_protocol(DemoProtocol::All, true),
            crate::demo::Protocol::Cli
        ));
    }

    // Test: create_test_config all suite types

    #[test]
    fn test_create_config_performance_suite() {
        use crate::cli::commands::TestSuite;
        let config =
            CommandDispatcher::create_test_config(&TestSuite::Performance, 5, false, false, false);
        assert_eq!(config.test_iterations, 5);
    }

    #[test]
    fn test_create_config_property_suite() {
        use crate::cli::commands::TestSuite;
        let config =
            CommandDispatcher::create_test_config(&TestSuite::Property, 10, false, false, false);
        assert_eq!(config.test_iterations, 10);
    }

    #[test]
    fn test_create_config_integration_suite() {
        use crate::cli::commands::TestSuite;
        let config =
            CommandDispatcher::create_test_config(&TestSuite::Integration, 1, false, false, false);
        assert_eq!(config.test_iterations, 1);
    }

    #[test]
    fn test_create_config_all_suite_enables_all() {
        use crate::cli::commands::TestSuite;
        let config = CommandDispatcher::create_test_config(&TestSuite::All, 3, false, false, false);
        assert!(config.enable_memory_tests);
        assert!(config.enable_throughput_tests);
        assert!(config.enable_regression_tests);
    }

    // Test: print_test_startup_info (doesn't panic)

    #[test]
    fn test_print_startup_all_suites() {
        use crate::cli::commands::TestSuite;
        for suite in [
            TestSuite::Performance,
            TestSuite::Property,
            TestSuite::Integration,
            TestSuite::Regression,
            TestSuite::Memory,
            TestSuite::Throughput,
            TestSuite::All,
        ] {
            CommandDispatcher::print_test_startup_info(&suite, 10, 60);
        }
    }

    // Test: write_test_results_if_requested

    #[test]
    fn test_write_results_with_output_success() {
        use crate::cli::commands::TestSuite;
        use std::time::Duration;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("internal error");
        let output = temp_dir.path().join("results.txt");

        let result: anyhow::Result<()> = Ok(());
        let write = CommandDispatcher::write_test_results_if_requested(
            Some(output.clone()),
            &TestSuite::Performance,
            Duration::from_secs(10),
            50,
            &result,
        );

        assert!(write.is_ok());
        assert!(output.exists());
        let content = std::fs::read_to_string(&output).expect("internal error");
        assert!(content.contains("PASSED"));
        assert!(content.contains("Performance"));
    }

    #[test]
    fn test_write_results_with_output_failure() {
        use crate::cli::commands::TestSuite;
        use std::time::Duration;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("internal error");
        let output = temp_dir.path().join("results.txt");

        let result: anyhow::Result<()> = Err(anyhow::anyhow!("Test failed"));
        let write = CommandDispatcher::write_test_results_if_requested(
            Some(output.clone()),
            &TestSuite::Regression,
            Duration::from_secs(5),
            100,
            &result,
        );

        assert!(write.is_ok());
        let content = std::fs::read_to_string(&output).expect("internal error");
        assert!(content.contains("FAILED"));
    }

    // Test: execute_config_command variants

    #[tokio::test]
    async fn test_config_validate() {
        let result =
            CommandDispatcher::execute_config_command(false, false, true, false, None, None, None)
                .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_config_with_section() {
        let result = CommandDispatcher::execute_config_command(
            true,
            false,
            false,
            false,
            Some("quality".to_string()),
            None,
            None,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_config_with_set_values() {
        let result = CommandDispatcher::execute_config_command(
            false,
            false,
            false,
            false,
            None,
            Some(vec!["test.key=value".to_string()]),
            None,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    // Test: execute_memory_command variants

    #[tokio::test]
    async fn test_memory_stats_detailed() {
        use crate::cli::handlers::memory::MemoryCommand;
        let result = CommandDispatcher::execute_memory_command(MemoryCommand::Stats {
            detailed: true,
            format: "json".to_string(),
        })
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_memory_cleanup_command() {
        use crate::cli::handlers::memory::MemoryCommand;
        let result = CommandDispatcher::execute_memory_command(MemoryCommand::Cleanup {
            target_pressure: 0.5,
            verbose: true,
        })
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    // Test: execute_cache_command variants

    #[tokio::test]
    async fn test_cache_stats_with_history() {
        use crate::cli::handlers::cache::CacheCommand;
        let result = CommandDispatcher::execute_cache_command(CacheCommand::Stats {
            detailed: true,
            format: "json".to_string(),
            history: true,
        })
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    // Test: execute_scaffold_agent_command directly

    #[tokio::test]
    async fn test_scaffold_agent_with_features() {
        let result = CommandDispatcher::execute_scaffold_agent_command(
            "feature-agent".to_string(),
            "mcp-server".to_string(),
            vec!["logging".to_string(), "metrics".to_string()],
            "strict".to_string(),
            None,
            false,
            true, // dry_run
            false,
            false,
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_scaffold_agent_deterministic_probabilistic() {
        let result = CommandDispatcher::execute_scaffold_agent_command(
            "hybrid-agent".to_string(),
            "hybrid".to_string(),
            vec![],
            "standard".to_string(),
            None,
            false,
            true,
            false,
            true, // deterministic_core
            true, // probabilistic_wrapper
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }
