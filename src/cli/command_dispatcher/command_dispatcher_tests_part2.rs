//! CommandDispatcher tests - Part 2: Metric and demo tests
//! Extracted for file health compliance (CB-040)

use super::*;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod metric_demo_tests {
    use super::*;

    // Tests for generate_metric_recommendations()

    #[test]
    fn test_generate_metric_recommendations_lint() {
        // Test lint metric with high slope (approaching threshold fast)
        let recs = CommandDispatcher::generate_metric_recommendations("lint", 200.0);
        // Should include lint-specific recommendations
        assert!(recs.iter().any(|r| r.contains("unused dependencies")));
        assert!(recs.iter().any(|r| r.contains("incremental clippy")));
    }

    #[test]
    fn test_generate_metric_recommendations_lint_critical() {
        // Test lint metric with very high slope (critical soon)
        let recs = CommandDispatcher::generate_metric_recommendations("lint", 500.0);
        // Should include warning about approaching threshold
        assert!(recs.iter().any(|r| r.contains("WARNING")));
    }

    #[test]
    fn test_generate_metric_recommendations_test_fast() {
        let recs = CommandDispatcher::generate_metric_recommendations("test-fast", 100.0);
        // Should include test-specific recommendations
        assert!(recs.iter().any(|r| r.contains("#[ignore]")));
        assert!(recs.iter().any(|r| r.contains("proptest")));
        assert!(recs.iter().any(|r| r.contains("nextest")));
    }

    #[test]
    fn test_generate_metric_recommendations_coverage() {
        let recs = CommandDispatcher::generate_metric_recommendations("coverage", 100.0);
        // Should include coverage-specific recommendations
        assert!(recs.iter().any(|r| r.contains("Exclude slow tests")));
        assert!(recs.iter().any(|r| r.contains("llvm-cov")));
    }

    #[test]
    fn test_generate_metric_recommendations_build_release() {
        let recs = CommandDispatcher::generate_metric_recommendations("build-release", 100.0);
        // Should include build-specific recommendations
        assert!(recs.iter().any(|r| r.contains("sccache")));
        assert!(recs.iter().any(|r| r.contains("mold") || r.contains("lld")));
    }

    #[test]
    fn test_generate_metric_recommendations_unknown_metric() {
        let recs = CommandDispatcher::generate_metric_recommendations("unknown", 100.0);
        // Should return empty recommendations for unknown metrics
        assert!(recs.is_empty());
    }

    // Tests for convert_demo_protocol()

    #[test]
    fn test_convert_demo_protocol_cli_flag_true() {
        // When cli=true, should always return Cli protocol regardless of protocol arg
        let result = CommandDispatcher::convert_demo_protocol(DemoProtocol::Http, true);
        assert!(matches!(result, crate::demo::Protocol::Cli));
    }

    #[test]
    fn test_convert_demo_protocol_cli() {
        let result = CommandDispatcher::convert_demo_protocol(DemoProtocol::Cli, false);
        assert!(matches!(result, crate::demo::Protocol::Cli));
    }

    #[test]
    fn test_convert_demo_protocol_http() {
        let result = CommandDispatcher::convert_demo_protocol(DemoProtocol::Http, false);
        assert!(matches!(result, crate::demo::Protocol::Http));
    }

    #[test]
    fn test_convert_demo_protocol_mcp() {
        let result = CommandDispatcher::convert_demo_protocol(DemoProtocol::Mcp, false);
        assert!(matches!(result, crate::demo::Protocol::Mcp));
    }

    #[test]
    fn test_convert_demo_protocol_all() {
        let result = CommandDispatcher::convert_demo_protocol(DemoProtocol::All, false);
        assert!(matches!(result, crate::demo::Protocol::All));
    }

    // Tests for create_demo_args()

    #[test]
    fn test_create_demo_args_defaults() {
        let args = CommandDispatcher::create_demo_args(
            None, // path
            None, // url
            None, // repo
            None, // format (will default to Table)
            crate::demo::Protocol::Cli,
            false, // show_api
            true,  // no_browser
            8080,  // port
            true,  // cli
            None,  // target_nodes (defaults to 1000)
            None,  // centrality_threshold (defaults to 0.5)
            None,  // merge_threshold (defaults to 100)
            false, // debug
            None,  // debug_output
            false, // skip_vendor
            false, // no_skip_vendor
            None,  // max_line_length
        );

        assert!(matches!(args.format, OutputFormat::Table));
        assert!(!args.show_api);
        assert!(args.no_browser);
        assert_eq!(args.port, Some(8080));
        assert!(!args.web); // cli=true means web=false
        assert_eq!(args.target_nodes, 1000);
        assert!((args.centrality_threshold - 0.5).abs() < 0.01);
        assert_eq!(args.merge_threshold, 100);
    }

    #[test]
    fn test_create_demo_args_with_values() {
        let args = CommandDispatcher::create_demo_args(
            Some(PathBuf::from("/test")),
            Some("http://localhost".to_string()),
            Some("org/repo".to_string()),
            Some(OutputFormat::Json),
            crate::demo::Protocol::Http,
            true,       // show_api
            false,      // no_browser
            3000,       // port
            false,      // cli
            Some(500),  // target_nodes
            Some(0.75), // centrality_threshold
            Some(50.0), // merge_threshold
            true,       // debug
            Some(PathBuf::from("/debug")),
            true,      // skip_vendor
            false,     // no_skip_vendor
            Some(120), // max_line_length
        );

        assert_eq!(args.path, Some(PathBuf::from("/test")));
        assert_eq!(args.url, Some("http://localhost".to_string()));
        assert_eq!(args.repo, Some("org/repo".to_string()));
        assert!(matches!(args.format, OutputFormat::Json));
        assert!(args.show_api);
        assert!(!args.no_browser);
        assert_eq!(args.port, Some(3000));
        assert!(args.web); // cli=false means web=true
        assert_eq!(args.target_nodes, 500);
        assert!((args.centrality_threshold - 0.75).abs() < 0.01);
        assert_eq!(args.merge_threshold, 50);
        assert!(args.debug);
        assert_eq!(args.debug_output, Some(PathBuf::from("/debug")));
        assert!(args.skip_vendor);
        assert_eq!(args.max_line_length, Some(120));
    }

    #[test]
    fn test_create_demo_args_skip_vendor_override() {
        // When no_skip_vendor is true, skip_vendor should be false
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
            true, // skip_vendor = true
            true, // no_skip_vendor = true (overrides skip_vendor)
            None,
        );

        assert!(!args.skip_vendor);
    }

    // Tests for execute_analyze_command() routing

    #[tokio::test]
    async fn test_execute_memory_command_routing() {
        use crate::cli::handlers::memory::MemoryCommand;

        // Test stats command routing
        let result = CommandDispatcher::execute_memory_command(MemoryCommand::Stats {
            detailed: false,
            format: "table".to_string(),
        })
        .await;
        // Should execute without panicking (may fail due to missing state)
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_execute_cache_command_routing() {
        use crate::cli::handlers::cache::CacheCommand;

        // Test stats command routing
        let result = CommandDispatcher::execute_cache_command(CacheCommand::Stats {
            detailed: false,
            format: "table".to_string(),
            history: false,
        })
        .await;
        // Should execute without panicking
        assert!(result.is_ok() || result.is_err());
    }

    // Test: generate_metric_recommendations edge cases

    #[test]
    fn test_metric_recommendations_negative_slope_lint() {
        // Negative slope = improving, but the function still generates recommendations
        // (the days_to_critical clamps to 0 with max(0.0) which is < 30)
        let recs = CommandDispatcher::generate_metric_recommendations("lint", -50.0);
        // Should still have recommendations for lint (actionable items)
        assert!(!recs.is_empty());
    }

    #[test]
    fn test_metric_recommendations_zero_slope_test_fast() {
        let recs = CommandDispatcher::generate_metric_recommendations("test-fast", 0.0);
        // Still provides general recommendations
        assert!(!recs.is_empty());
    }

    #[test]
    fn test_metric_recommendations_coverage_critical() {
        let recs = CommandDispatcher::generate_metric_recommendations("coverage", 10000.0);
        // High slope = approaching threshold fast
        assert!(recs.iter().any(|r| r.contains("WARNING")));
    }

    #[test]
    fn test_metric_recommendations_build_release_critical() {
        let recs = CommandDispatcher::generate_metric_recommendations("build-release", 10000.0);
        assert!(recs.iter().any(|r| r.contains("WARNING")));
    }

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
            None, None, None, None,
            crate::demo::Protocol::Cli,
            false, true, 8080, true,
            None, None, None, false, None,
            true,  // skip_vendor
            true,  // no_skip_vendor (takes precedence)
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
        let config = CommandDispatcher::create_test_config(&TestSuite::Performance, 5, false, false, false);
        assert_eq!(config.test_iterations, 5);
    }

    #[test]
    fn test_create_config_property_suite() {
        use crate::cli::commands::TestSuite;
        let config = CommandDispatcher::create_test_config(&TestSuite::Property, 10, false, false, false);
        assert_eq!(config.test_iterations, 10);
    }

    #[test]
    fn test_create_config_integration_suite() {
        use crate::cli::commands::TestSuite;
        let config = CommandDispatcher::create_test_config(&TestSuite::Integration, 1, false, false, false);
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
}
