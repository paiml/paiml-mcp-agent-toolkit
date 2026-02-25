    // Helper Function Tests - convert_deep_context_dag_type

    #[test]
    fn test_convert_deep_context_dag_type_call_graph() {
        let result = convert_deep_context_dag_type(DeepContextDagType::CallGraph);
        assert!(matches!(result, DagType::CallGraph));
    }

    #[test]
    fn test_convert_deep_context_dag_type_import_graph() {
        let result = convert_deep_context_dag_type(DeepContextDagType::ImportGraph);
        assert!(matches!(result, DagType::ImportGraph));
    }

    #[test]
    fn test_convert_deep_context_dag_type_inheritance() {
        let result = convert_deep_context_dag_type(DeepContextDagType::Inheritance);
        assert!(matches!(result, DagType::Inheritance));
    }

    #[test]
    fn test_convert_deep_context_dag_type_full_dependency() {
        let result = convert_deep_context_dag_type(DeepContextDagType::FullDependency);
        assert!(matches!(result, DagType::FullDependency));
    }

    // Helper Function Tests - convert_cache_strategy

    #[test]
    fn test_convert_cache_strategy_normal() {
        let result = convert_cache_strategy(DeepContextCacheStrategy::Normal);
        assert_eq!(result, "normal");
    }

    #[test]
    fn test_convert_cache_strategy_force_refresh() {
        let result = convert_cache_strategy(DeepContextCacheStrategy::ForceRefresh);
        assert_eq!(result, "force-refresh");
    }

    #[test]
    fn test_convert_cache_strategy_offline() {
        let result = convert_cache_strategy(DeepContextCacheStrategy::Offline);
        assert_eq!(result, "offline");
    }

    // Helper Function Tests - Entropy Report Formatting

    #[test]
    fn test_create_entropy_config_defaults() {
        let config = create_entropy_config(EntropySeverity::Medium, true);
        assert!(matches!(
            config.min_severity,
            crate::entropy::violation_detector::Severity::Medium
        ));
        // When include_tests is true, no additional exclusions are added
    }

    #[test]
    fn test_create_entropy_config_low_severity() {
        let config = create_entropy_config(EntropySeverity::Low, false);
        assert!(matches!(
            config.min_severity,
            crate::entropy::violation_detector::Severity::Low
        ));
        // When include_tests is false, test paths are excluded
        assert!(config.exclude_paths.iter().any(|p| p.contains("test")));
    }

    #[test]
    fn test_create_entropy_config_high_severity() {
        let config = create_entropy_config(EntropySeverity::High, true);
        assert!(matches!(
            config.min_severity,
            crate::entropy::violation_detector::Severity::High
        ));
    }

    #[test]
    fn test_get_top_violations_zero_limit() {
        let violations = vec![];
        let result = get_top_violations(&violations, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_top_violations_with_limit() {
        // Test with empty violations
        let violations = vec![];
        let result = get_top_violations(&violations, 5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_format_violation_list_empty() {
        let violations = vec![];
        let result = format_violation_list(&violations);
        assert!(result.is_empty());
    }

    #[test]
    fn test_format_markdown_violations_empty() {
        let violations = vec![];
        let result = format_markdown_violations(&violations, 10);
        assert!(result.is_empty());
    }

    #[test]
    fn test_output_entropy_results_to_stdout() {
        // Test stdout output (no file path)
        let result = output_entropy_results(None, "test content");
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_entropy_results_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test_output.txt");

        let result = output_entropy_results(Some(output_path.clone()), "test content");
        assert!(result.is_ok());
        assert!(output_path.exists());

        let content = std::fs::read_to_string(output_path).unwrap();
        assert_eq!(content, "test content");
    }

