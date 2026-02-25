    // Tests for get_template_variant()

    #[test]
    fn test_get_template_variant_makefile_rust() {
        assert_eq!(get_template_variant("makefile", "rust"), Some("cli"));
    }

    #[test]
    fn test_get_template_variant_makefile_deno() {
        assert_eq!(get_template_variant("makefile", "deno"), Some("cli"));
    }

    #[test]
    fn test_get_template_variant_makefile_python() {
        assert_eq!(get_template_variant("makefile", "python-uv"), Some("cli"));
    }

    #[test]
    fn test_get_template_variant_readme_rust() {
        assert_eq!(get_template_variant("readme", "rust"), Some("cli"));
    }

    #[test]
    fn test_get_template_variant_gitignore_rust() {
        assert_eq!(get_template_variant("gitignore", "rust"), Some("cli"));
    }

    #[test]
    fn test_get_template_variant_unknown_template() {
        assert_eq!(get_template_variant("unknown", "rust"), None);
    }

    #[test]
    fn test_get_template_variant_unknown_toolchain() {
        assert_eq!(get_template_variant("makefile", "java"), None);
    }

    // Tests for calculate_relevance()

    fn create_test_template_resource(name: &str, desc: &str) -> TemplateResource {
        TemplateResource {
            uri: format!("template://{}", name),
            name: name.to_string(),
            description: desc.to_string(),
            category: "test".to_string(),
            toolchain: "rust".to_string(),
            mime_type: "text/plain".to_string(),
            parameters: vec![ParameterSpec { param_type: crate::models::template::ParameterType::String,
                name: "project_name".to_string(),
                description: "Project name".to_string(),
                required: true,
                default_value: None,
                validation_pattern: None,
            }],
        }
    }

    #[test]
    fn test_calculate_relevance_exact_name_match() {
        let template = create_test_template_resource("makefile", "A makefile template");
        let score = calculate_relevance(&template, "makefile");
        assert!(score >= 10.0, "Exact match should score at least 10");
    }

    #[test]
    fn test_calculate_relevance_partial_name_match() {
        let template = create_test_template_resource("makefile-rust", "A makefile template");
        let score = calculate_relevance(&template, "make");
        assert!(score >= 5.0, "Partial name match should score at least 5");
    }

    #[test]
    fn test_calculate_relevance_description_match() {
        let template = create_test_template_resource("some_template", "A testing framework setup");
        let score = calculate_relevance(&template, "testing");
        assert!(score >= 3.0, "Description match should score at least 3");
    }

    #[test]
    fn test_calculate_relevance_no_match() {
        let template = create_test_template_resource("makefile", "Build configuration");
        let score = calculate_relevance(&template, "xyz123");
        assert_eq!(score, 0.0, "No match should score 0");
    }

    // Tests for resolve_project_path() and related path functions

    #[test]
    fn test_resolve_project_path_with_explicit_path() {
        let path = resolve_project_path(&Some("/custom/path".to_string()));
        assert_eq!(path, PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_resolve_project_path_none() {
        let path = resolve_project_path(&None);
        // Should return current dir or fallback
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn test_resolve_project_path_complexity_with_path() {
        let path = resolve_project_path_complexity(Some("/my/project".to_string()));
        assert_eq!(path, PathBuf::from("/my/project"));
    }

    #[test]
    fn test_resolve_project_path_complexity_none() {
        let path = resolve_project_path_complexity(None);
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn test_resolve_deep_context_project_path_some() {
        let path = resolve_deep_context_project_path(Some("/deep/context".to_string()));
        assert_eq!(path, PathBuf::from("/deep/context"));
    }

    #[test]
    fn test_resolve_deep_context_project_path_none() {
        let path = resolve_deep_context_project_path(None);
        assert!(!path.as_os_str().is_empty());
    }

    // Tests for detect_toolchain()

    #[test]
    fn test_detect_toolchain_explicit_rust() {
        let toolchain = detect_toolchain(&Some("rust".to_string()), Path::new("/tmp"));
        assert_eq!(toolchain, "rust");
    }

    #[test]
    fn test_detect_toolchain_explicit_deno() {
        let toolchain = detect_toolchain(&Some("deno".to_string()), Path::new("/tmp"));
        assert_eq!(toolchain, "deno");
    }

    #[test]
    fn test_detect_toolchain_explicit_python() {
        let toolchain = detect_toolchain(&Some("python-uv".to_string()), Path::new("/tmp"));
        assert_eq!(toolchain, "python-uv");
    }

    #[test]
    fn test_detect_toolchain_default_rust() {
        // When no files exist and no explicit toolchain, defaults to rust
        let toolchain = detect_toolchain(&None, Path::new("/nonexistent/path"));
        assert_eq!(toolchain, "rust");
    }

    // Tests for should_analyze_file()

    #[test]
    fn test_should_analyze_file_rust() {
        assert!(should_analyze_file(Path::new("src/main.rs"), "rust"));
        assert!(!should_analyze_file(Path::new("src/main.py"), "rust"));
        assert!(!should_analyze_file(Path::new("src/main.ts"), "rust"));
    }

    #[test]
    fn test_should_analyze_file_deno() {
        assert!(should_analyze_file(Path::new("src/main.ts"), "deno"));
        assert!(should_analyze_file(Path::new("src/main.tsx"), "deno"));
        assert!(should_analyze_file(Path::new("src/main.js"), "deno"));
        assert!(should_analyze_file(Path::new("src/main.jsx"), "deno"));
        assert!(!should_analyze_file(Path::new("src/main.rs"), "deno"));
    }

    #[test]
    fn test_should_analyze_file_python() {
        assert!(should_analyze_file(Path::new("src/main.py"), "python-uv"));
        assert!(!should_analyze_file(Path::new("src/main.rs"), "python-uv"));
    }

    #[test]
    fn test_should_analyze_file_unknown_toolchain() {
        assert!(!should_analyze_file(Path::new("src/main.rs"), "unknown"));
    }

    // Tests for matches_include_filters() and matches_pattern()

    #[test]
    fn test_matches_include_filters_none() {
        // No patterns means everything matches
        assert!(matches_include_filters(Path::new("src/main.rs"), &None));
    }

    #[test]
    fn test_matches_include_filters_empty_vec() {
        // Empty patterns means everything matches
        assert!(matches_include_filters(
            Path::new("src/main.rs"),
            &Some(vec![])
        ));
    }

    #[test]
    fn test_matches_include_filters_matching_pattern() {
        let patterns = Some(vec!["*.rs".to_string()]);
        assert!(matches_include_filters(Path::new("src/main.rs"), &patterns));
    }

    #[test]
    fn test_matches_include_filters_non_matching() {
        let patterns = Some(vec!["*.py".to_string()]);
        assert!(!matches_include_filters(
            Path::new("src/main.rs"),
            &patterns
        ));
    }

    #[test]
    fn test_matches_pattern_extension() {
        assert!(matches_pattern("src/main.rs", "*.rs"));
        assert!(!matches_pattern("src/main.py", "*.rs"));
    }

    #[test]
    fn test_matches_pattern_glob_star() {
        assert!(matches_pattern("src/lib/module.rs", "**/module.rs"));
        assert!(matches_pattern("deep/nested/module.rs", "**/module.rs"));
    }

    #[test]
    fn test_matches_pattern_substring() {
        assert!(matches_pattern("src/main.rs", "main"));
        assert!(matches_pattern("src/main_test.rs", "test"));
        assert!(!matches_pattern("src/lib.rs", "main"));
    }

    // Tests for build_complexity_thresholds()

    #[test]
    fn test_build_complexity_thresholds_defaults() {
        let args = AnalyzeComplexityArgs {
            project_path: None,
            toolchain: None,
            format: None,
            max_cyclomatic: None,
            max_cognitive: None,
            include: None,
            top_files: None,
        };
        let thresholds = build_complexity_thresholds(&args);
        // Default thresholds should be reasonable values
        assert!(thresholds.cyclomatic_error > 0);
        assert!(thresholds.cognitive_error > 0);
    }

    #[test]
    fn test_build_complexity_thresholds_custom_cyclomatic() {
        let args = AnalyzeComplexityArgs {
            project_path: None,
            toolchain: None,
            format: None,
            max_cyclomatic: Some(20),
            max_cognitive: None,
            include: None,
            top_files: None,
        };
        let thresholds = build_complexity_thresholds(&args);
        assert_eq!(thresholds.cyclomatic_error, 20);
        // Warning should be 3/4 of error threshold
        assert_eq!(thresholds.cyclomatic_warn, 15);
    }

    #[test]
    fn test_build_complexity_thresholds_custom_cognitive() {
        let args = AnalyzeComplexityArgs {
            project_path: None,
            toolchain: None,
            format: None,
            max_cyclomatic: None,
            max_cognitive: Some(30),
            include: None,
            top_files: None,
        };
        let thresholds = build_complexity_thresholds(&args);
        assert_eq!(thresholds.cognitive_error, 30);
        // Warning should be 3/4 of error threshold
        assert_eq!(thresholds.cognitive_warn, 22);
    }

    // Tests for parse_dag_type()

    #[test]
    fn test_parse_dag_type_call_graph() {
        let dag_type = parse_dag_type(Some("call-graph"));
        assert!(matches!(dag_type, crate::cli::DagType::CallGraph));
    }

    #[test]
    fn test_parse_dag_type_import_graph() {
        let dag_type = parse_dag_type(Some("import-graph"));
        assert!(matches!(dag_type, crate::cli::DagType::ImportGraph));
    }

    #[test]
    fn test_parse_dag_type_inheritance() {
        let dag_type = parse_dag_type(Some("inheritance"));
        assert!(matches!(dag_type, crate::cli::DagType::Inheritance));
    }

    #[test]
    fn test_parse_dag_type_full_dependency() {
        let dag_type = parse_dag_type(Some("full-dependency"));
        assert!(matches!(dag_type, crate::cli::DagType::FullDependency));
    }

    #[test]
    fn test_parse_dag_type_default() {
        let dag_type = parse_dag_type(None);
        assert!(matches!(dag_type, crate::cli::DagType::CallGraph));
    }

    #[test]
    fn test_parse_dag_type_unknown() {
        let dag_type = parse_dag_type(Some("unknown"));
        assert!(matches!(dag_type, crate::cli::DagType::CallGraph));
    }

    // Tests for parse_deep_context_dag_type()

    #[test]
    fn test_parse_deep_context_dag_type_call_graph() {
        let dag_type = parse_deep_context_dag_type(Some("call-graph".to_string()));
        assert!(matches!(
            dag_type,
            crate::services::deep_context::DagType::CallGraph
        ));
    }

    #[test]
    fn test_parse_deep_context_dag_type_import() {
        let dag_type = parse_deep_context_dag_type(Some("import-graph".to_string()));
        assert!(matches!(
            dag_type,
            crate::services::deep_context::DagType::ImportGraph
        ));
    }

    #[test]
    fn test_parse_deep_context_dag_type_inheritance() {
        let dag_type = parse_deep_context_dag_type(Some("inheritance".to_string()));
        assert!(matches!(
            dag_type,
            crate::services::deep_context::DagType::Inheritance
        ));
    }

    #[test]
    fn test_parse_deep_context_dag_type_full() {
        let dag_type = parse_deep_context_dag_type(Some("full-dependency".to_string()));
        assert!(matches!(
            dag_type,
            crate::services::deep_context::DagType::FullDependency
        ));
    }

    #[test]
    fn test_parse_deep_context_dag_type_default() {
        let dag_type = parse_deep_context_dag_type(None);
        assert!(matches!(
            dag_type,
            crate::services::deep_context::DagType::CallGraph
        ));
    }

    // Tests for parse_cache_strategy()

    #[test]
    fn test_parse_cache_strategy_normal() {
        let strategy = parse_cache_strategy(Some("normal".to_string()));
        assert!(matches!(
            strategy,
            crate::services::deep_context::CacheStrategy::Normal
        ));
    }

    #[test]
    fn test_parse_cache_strategy_force_refresh() {
        let strategy = parse_cache_strategy(Some("force-refresh".to_string()));
        assert!(matches!(
            strategy,
            crate::services::deep_context::CacheStrategy::ForceRefresh
        ));
    }

    #[test]
    fn test_parse_cache_strategy_offline() {
        let strategy = parse_cache_strategy(Some("offline".to_string()));
        assert!(matches!(
            strategy,
            crate::services::deep_context::CacheStrategy::Offline
        ));
    }

    #[test]
    fn test_parse_cache_strategy_default() {
        let strategy = parse_cache_strategy(None);
        assert!(matches!(
            strategy,
            crate::services::deep_context::CacheStrategy::Normal
        ));
    }

    // Tests for parse_analysis_type_string() and parse_analysis_types()

    #[test]
    fn test_parse_analysis_type_string_ast() {
        let analysis_type = parse_analysis_type_string("ast");
        assert!(matches!(
            analysis_type,
            Some(crate::services::deep_context::AnalysisType::Ast)
        ));
    }

    #[test]
    fn test_parse_analysis_type_string_complexity() {
        let analysis_type = parse_analysis_type_string("complexity");
        assert!(matches!(
            analysis_type,
            Some(crate::services::deep_context::AnalysisType::Complexity)
        ));
    }

    #[test]
    fn test_parse_analysis_type_string_churn() {
        let analysis_type = parse_analysis_type_string("churn");
        assert!(matches!(
            analysis_type,
            Some(crate::services::deep_context::AnalysisType::Churn)
        ));
    }

    #[test]
    fn test_parse_analysis_type_string_dag() {
        let analysis_type = parse_analysis_type_string("dag");
        assert!(matches!(
            analysis_type,
            Some(crate::services::deep_context::AnalysisType::Dag)
        ));
    }

    #[test]
    fn test_parse_analysis_type_string_dead_code() {
        let analysis_type = parse_analysis_type_string("dead_code");
        assert!(matches!(
            analysis_type,
            Some(crate::services::deep_context::AnalysisType::DeadCode)
        ));
    }

    #[test]
    fn test_parse_analysis_type_string_satd() {
        let analysis_type = parse_analysis_type_string("satd");
        assert!(matches!(
            analysis_type,
            Some(crate::services::deep_context::AnalysisType::Satd)
        ));
    }

    #[test]
    fn test_parse_analysis_type_string_tdg() {
        let analysis_type = parse_analysis_type_string("tdg");
        assert!(matches!(
            analysis_type,
            Some(crate::services::deep_context::AnalysisType::TechnicalDebtGradient)
        ));
    }

    #[test]
    fn test_parse_analysis_type_string_unknown() {
        let analysis_type = parse_analysis_type_string("unknown");
        assert!(analysis_type.is_none());
    }

    #[test]
    fn test_parse_analysis_types_some() {
        let types = parse_analysis_types(Some(vec!["ast".to_string(), "complexity".to_string()]));
        assert_eq!(types.len(), 2);
    }

    #[test]
    fn test_parse_analysis_types_none() {
        let types = parse_analysis_types(None);
        // Default types should include ast, complexity, churn
        assert!(!types.is_empty());
    }

    #[test]
    fn test_get_default_analysis_types() {
        let types = get_default_analysis_types();
        assert_eq!(types.len(), 3);
    }

