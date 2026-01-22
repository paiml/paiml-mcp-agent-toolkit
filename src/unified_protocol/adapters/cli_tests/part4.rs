        assert!(result.is_ok());
        let (method, path, _, _) = result.unwrap();
        assert_eq!(method, Method::POST);
        assert_eq!(path, "/api/v1/analyze/defect-prediction");
    }

    #[test]
    fn test_decode_analyze_graph_metrics() {
        let result = CliAdapter::decode_analyze_graph_metrics(
            &PathBuf::from("."),
            &[GraphMetricType::PageRank, GraphMetricType::Centrality],
            &["main".to_string()],
            &0.85,
            &100,
            &1e-6,
            &GraphMetricsOutputFormat::Json,
            &None,
            &None,
            &None,
            &false,
            &false,
            &10,
            &0.01,
        );

        assert!(result.is_ok());
        let (method, path, _, _) = result.unwrap();
        assert_eq!(method, Method::POST);
        assert_eq!(path, "/api/v1/analyze/graph-metrics");
    }

    #[test]
    fn test_decode_analyze_name_similarity() {
        let result = CliAdapter::decode_analyze_name_similarity(
            &PathBuf::from("."),
            "parse_config",
            &10,
            &false,
            &SearchScope::Functions,
            &0.8,
            &NameSimilarityOutputFormat::Json,
            &None,
            &None,
            &None,
            &false,
            &true,
            &false,
        );

        assert!(result.is_ok());
        let (method, path, _, _) = result.unwrap();
        assert_eq!(method, Method::POST);
        assert_eq!(path, "/api/v1/analyze/name-similarity");
    }

    #[test]
    fn test_decode_analyze_symbol_table() {
        let result = CliAdapter::decode_analyze_symbol_table(
            &PathBuf::from("."),
            &SymbolTableOutputFormat::Json,
            &Some("Config".to_string()),
            &Some(SymbolTypeFilter::Types),
            &vec!["**/*.rs".to_string()],
            &vec!["test_*".to_string()],
            &true,
            &true,
            &None,
            &false,
            &10,
        );

        assert!(result.is_ok());
        let (method, path, _, _) = result.unwrap();
        assert_eq!(method, Method::POST);
        assert_eq!(path, "/api/v1/analyze/symbol-table");
    }

    #[test]
    fn test_decode_analyze_proof_annotations() {
        let result = CliAdapter::decode_analyze_proof_annotations(
            &PathBuf::from("."),
            &ProofAnnotationOutputFormat::Json,
            &true,
            &true,
            &Some(PropertyTypeFilter::MemorySafety),
            &Some(VerificationMethodFilter::BorrowChecker),
            &None,
            &false,
            &false,
            &10,
        );

        assert!(result.is_ok());
        let (method, path, _, _) = result.unwrap();
        assert_eq!(method, Method::POST);
        assert_eq!(path, "/api/v1/analyze/proof-annotations");
    }

    #[test]
    fn test_decode_analyze_incremental_coverage() {
        let result = CliAdapter::decode_analyze_incremental_coverage(
            &PathBuf::from("."),
            &"main".to_string(),
            &Some("feature".to_string()),
            &IncrementalCoverageOutputFormat::Json,
            &80.0,
            &true,
            &true,
            &None,
            &false,
            &None,
            &false,
            &10,
        );

        assert!(result.is_ok());
        let (method, path, _, _) = result.unwrap();
        assert_eq!(method, Method::POST);
        assert_eq!(path, "/api/v1/analyze/incremental-coverage");
    }

    #[test]
    fn test_decode_analyze_big_o() {
        let result = CliAdapter::decode_analyze_big_o(
            &PathBuf::from("."),
            &BigOOutputFormat::Json,
            &80,
            &true,
            &vec![],
            &vec![],
            &None,
            &false,
            &true,
            &10,
        );

        assert!(result.is_ok());
        let (method, path, _, _) = result.unwrap();
        assert_eq!(method, Method::POST);
        assert_eq!(path, "/api/v1/analyze/big-o");
    }

    #[test]
    fn test_decode_analyze_lint_hotspot() {
        let result = CliAdapter::decode_analyze_lint_hotspot(
            &PathBuf::from("."),
            &None,
            &LintHotspotOutputFormat::Json,
            &100.0,
            &0.5,
            &false,
            &false,
            &false,
            &None,
            &false,
            &String::new(),
            &10,
        );

        assert!(result.is_ok());
        let (method, path, _, _) = result.unwrap();
        assert_eq!(method, Method::POST);
        assert_eq!(path, "/api/v1/analyze/lint-hotspot");
    }

    #[test]
    fn test_decode_analyze_makefile() {
        let result = CliAdapter::decode_analyze_makefile(
            &PathBuf::from("Makefile"),
            &vec!["all".to_string(), "clean".to_string()],
            &MakefileOutputFormat::Json,
            &false,
            &"4.3".to_string(),
            &5,
        );

        assert!(result.is_ok());
        let (method, path, _, _) = result.unwrap();
        assert_eq!(method, Method::POST);
        assert_eq!(path, "/api/v1/analyze/makefile");
    }

    #[test]
    fn test_decode_analyze_assemblyscript() {
        let result = CliAdapter::decode_analyze_assemblyscript();

        assert!(result.is_ok());
        let (method, path, _, _) = result.unwrap();
        assert_eq!(method, Method::POST);
        assert_eq!(path, "/api/v1/analyze/assemblyscript");
    }

    #[test]
    fn test_decode_analyze_webassembly() {
        let result = CliAdapter::decode_analyze_webassembly();

        assert!(result.is_ok());
        let (method, path, _, _) = result.unwrap();
        assert_eq!(method, Method::POST);
        assert_eq!(path, "/api/v1/analyze/webassembly");
    }

    #[test]
    fn test_cli_only_command_error() {
        let result = CliAdapter::cli_only_command_error();

        assert!(result.is_err());
        match result {
            Err(ProtocolError::InvalidFormat(msg)) => {
                assert!(msg.contains("CLI"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    // === Command Category Tests ===

    #[test]
    fn test_get_analyze_command_category_basic() {
        let churn_cmd = AnalyzeCommands::Churn {
            project_path: PathBuf::from("."),
            days: 30,
            format: crate::models::churn::ChurnOutputFormat::Json,
            output: None,
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };
        let category = CliAdapter::get_analyze_command_category(&churn_cmd);
        assert!(matches!(category, AnalyzeCommandCategory::Basic));
    }

    #[test]
    fn test_get_analyze_command_category_advanced() {
        let deep_context_cmd = AnalyzeCommands::DeepContext {
            project_path: PathBuf::from("."),
            output: None,
            format: crate::cli::DeepContextOutputFormat::Json,
            full: false,
            include: vec![],
            exclude: vec![],
            period_days: 30,
            dag_type: crate::cli::DeepContextDagType::CallGraph,
            max_depth: None,
            include_patterns: vec![],
            exclude_patterns: vec![],
            cache_strategy: crate::cli::DeepContextCacheStrategy::Normal,
            parallel: None,
            verbose: false,
            top_files: 10,
        };
        let category = CliAdapter::get_analyze_command_category(&deep_context_cmd);
        assert!(matches!(category, AnalyzeCommandCategory::Advanced));
    }

    #[test]
    fn test_get_analyze_command_category_structural() {
        let dag_cmd = AnalyzeCommands::Dag {
            dag_type: DagType::CallGraph,
            project_path: PathBuf::from("."),
            output: None,
            max_depth: None,
            target_nodes: None,
            filter_external: false,
            show_complexity: false,
            include_duplicates: false,
            include_dead_code: false,
            enhanced: false,
        };
        let category = CliAdapter::get_analyze_command_category(&dag_cmd);
        assert!(matches!(category, AnalyzeCommandCategory::Structural));
    }

    #[test]
    fn test_get_analyze_command_category_specialized() {
        let makefile_cmd = AnalyzeCommands::Makefile {
            path: PathBuf::from("Makefile"),
            rules: vec![],
            format: MakefileOutputFormat::Json,
            fix: false,
            gnu_version: "4.3".to_string(),
            top_files: 5,
        };
        let category = CliAdapter::get_analyze_command_category(&makefile_cmd);
        assert!(matches!(category, AnalyzeCommandCategory::Specialized));
    }

    // === CliInput Tests ===

    #[test]
    fn test_cli_input_new() {
        let command = Commands::List {
            toolchain: None,
            category: None,
            format: OutputFormat::Json,
        };
        let input = CliInput::new(command, "list".to_string(), vec!["pmat".to_string()]);

        assert_eq!(input.command_name, "list");
        assert_eq!(input.raw_args.len(), 1);
    }

    #[test]
    fn test_get_qdd_command_name_create() {
        let qdd_cmd = crate::cli::commands::QddCommands::Create {
            code_type: crate::cli::commands::QddCodeType::Function,
            name: "test".to_string(),
            purpose: "test purpose".to_string(),
            profile: crate::cli::commands::QddQualityProfile::Standard,
            input: vec![],
            output: "()".to_string(),
            output_file: None,
        };
        assert_eq!(CliInput::get_qdd_command_name(&qdd_cmd), "qdd-create");
    }

    #[test]
    fn test_get_qdd_command_name_refactor() {
        let qdd_cmd = crate::cli::commands::QddCommands::Refactor {
            file: PathBuf::from("test.rs"),
            function: None,
            profile: crate::cli::commands::QddQualityProfile::Standard,
            max_complexity: None,
            min_coverage: None,
            output: None,
            dry_run: false,
        };
        assert_eq!(CliInput::get_qdd_command_name(&qdd_cmd), "qdd-refactor");
    }

    #[test]
    fn test_get_qdd_command_name_validate() {
        let qdd_cmd = crate::cli::commands::QddCommands::Validate {
            path: PathBuf::from("."),
            profile: crate::cli::commands::QddQualityProfile::Standard,
            format: crate::cli::commands::QddOutputFormat::Summary,
            output: None,
            strict: false,
        };
        assert_eq!(CliInput::get_qdd_command_name(&qdd_cmd), "qdd-validate");
    }

    // === CliOutput Tests ===

    #[test]
    fn test_cli_output_success_content() {
        let output = CliOutput::Success {
            content: "test output".to_string(),
            exit_code: 0,
        };

        assert_eq!(output.content(), "test output");
        assert_eq!(output.exit_code(), 0);
    }

    #[test]
    fn test_cli_output_error_content() {
        let output = CliOutput::Error {
            message: "test error".to_string(),
            exit_code: 2,
        };

        assert_eq!(output.content(), "test error");
        assert_eq!(output.exit_code(), 2);
    }

    // === CliRunner Tests ===

    #[test]
    fn test_cli_runner_creation() {
        let runner = CliRunner::new();
        // Verify the runner was created successfully by checking the adapter
        assert_eq!(runner.adapter.protocol(), Protocol::Cli);
    }

    #[test]
    fn test_cli_runner_default_creation() {
        let runner = CliRunner::default();
        assert_eq!(runner.adapter.protocol(), Protocol::Cli);
    }

    // === Protocol Adapter Trait Tests ===

    #[test]
    fn test_cli_adapter_protocol() {
        let adapter = CliAdapter::new();
        assert_eq!(adapter.protocol(), Protocol::Cli);
    }

    // === Dispatch Error Handling Tests ===

    #[test]
    fn test_dispatch_structural_with_basic_command_fails() {
        let churn_cmd = AnalyzeCommands::Churn {
            project_path: PathBuf::from("."),
            days: 30,
            format: crate::models::churn::ChurnOutputFormat::Json,
            output: None,
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let result = CliAdapter::dispatch_structural_analysis(&churn_cmd);
        assert!(result.is_err());
    }

    #[test]
    fn test_dispatch_specialized_with_basic_command_fails() {
        let complexity_cmd = AnalyzeCommands::Complexity {
            path: PathBuf::from("."),
            project_path: None,
            file: None,
            files: vec![],
            toolchain: None,
            format: ComplexityOutputFormat::Json,
            output: None,
            max_cyclomatic: None,
            max_cognitive: None,
            include: vec![],
            watch: false,
            top_files: 0,
            fail_on_violation: false,
            timeout: 60,
            ml: false,
        };

        let result = CliAdapter::dispatch_specialized_analysis(&complexity_cmd);
        assert!(result.is_err());
    }
}
