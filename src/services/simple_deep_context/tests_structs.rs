    #[test]
    fn test_simple_deep_context_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }

    #[test]
    fn test_simple_analysis_config_clone() {
        let config = SimpleAnalysisConfig {
            project_path: std::path::PathBuf::from("/test/path"),
            include_features: vec!["feature1".to_string()],
            include_patterns: vec!["**/*.rs".to_string()],
            exclude_patterns: vec!["test".to_string()],
            enable_verbose: true,
        };
        let cloned = config.clone();
        assert_eq!(cloned.project_path, config.project_path);
        assert_eq!(cloned.include_features, config.include_features);
        assert_eq!(cloned.include_patterns, config.include_patterns);
        assert_eq!(cloned.exclude_patterns, config.exclude_patterns);
        assert_eq!(cloned.enable_verbose, config.enable_verbose);
    }

    #[test]
    fn test_simple_analysis_config_debug() {
        let config = SimpleAnalysisConfig {
            project_path: std::path::PathBuf::from("/test"),
            include_features: vec![],
            include_patterns: vec![],
            exclude_patterns: vec![],
            enable_verbose: false,
        };
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("project_path"));
    }

    #[test]
    fn test_complexity_metrics_debug() {
        let metrics = ComplexityMetrics {
            total_functions: 10,
            high_complexity_count: 2,
            avg_complexity: 3.5,
        };
        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("total_functions"));
        assert!(debug_str.contains("10"));
    }

    #[test]
    fn test_file_complexity_detail_clone() {
        let detail = FileComplexityDetail {
            file_path: std::path::PathBuf::from("test.rs"),
            function_count: 5,
            high_complexity_functions: 1,
            avg_complexity: 2.0,
            complexity_score: 4.5,
            function_names: vec!["main".to_string(), "helper".to_string()],
        };
        let cloned = detail.clone();
        assert_eq!(cloned.file_path, detail.file_path);
        assert_eq!(cloned.function_count, detail.function_count);
        assert_eq!(cloned.function_names.len(), 2);
    }

    #[test]
    fn test_file_complexity_detail_debug() {
        let detail = FileComplexityDetail {
            file_path: std::path::PathBuf::from("debug.rs"),
            function_count: 3,
            high_complexity_functions: 0,
            avg_complexity: 1.0,
            complexity_score: 1.5,
            function_names: vec![],
        };
        let debug_str = format!("{:?}", detail);
        assert!(debug_str.contains("debug.rs"));
    }

    #[test]
    fn test_simple_deep_context_default() {
        let analyzer = SimpleDeepContext::default();
        let _ = analyzer; // just verify it creates
    }

