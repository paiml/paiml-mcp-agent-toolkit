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

