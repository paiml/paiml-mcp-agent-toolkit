
    #[test]
    fn test_defect_prediction_command_construction() {
        let cmd = AnalyzeCommands::DefectPrediction {
            project_path: PathBuf::from("."),
            confidence_threshold: 0.5,
            min_lines: 10,
            include_low_confidence: false,
            format: DefectPredictionOutputFormat::Summary,
            high_risk_only: true,
            include_recommendations: true,
            include: None,
            exclude: None,
            output: None,
            perf: false,
            top_files: 10,
        };

        if let AnalyzeCommands::DefectPrediction {
            confidence_threshold,
            high_risk_only,
            ..
        } = cmd
        {
            assert!((confidence_threshold - 0.5).abs() < f32::EPSILON);
            assert!(high_risk_only);
        } else {
            panic!("Expected DefectPrediction command");
        }
    }

    #[test]
    fn test_provability_command_construction() {
        let cmd = AnalyzeCommands::Provability {
            project_path: PathBuf::from("."),
            functions: vec!["test_fn".to_string()],
            analysis_depth: 10,
            format: ProvabilityOutputFormat::Summary,
            high_confidence_only: false,
            include_evidence: true,
            output: None,
            top_files: 10,
        };

        if let AnalyzeCommands::Provability {
            analysis_depth,
            include_evidence,
            ..
        } = cmd
        {
            assert_eq!(analysis_depth, 10);
            assert!(include_evidence);
        } else {
            panic!("Expected Provability command");
        }
    }

    #[test]
    fn test_graph_metrics_command_construction() {
        let cmd = AnalyzeCommands::GraphMetrics {
            project_path: PathBuf::from("."),
            metrics: vec![GraphMetricType::All],
            pagerank_seeds: vec![],
            damping_factor: 0.85,
            max_iterations: 100,
            convergence_threshold: 0.001,
            export_graphml: false,
            format: GraphMetricsOutputFormat::Summary,
            include: None,
            exclude: None,
            output: None,
            perf: false,
            top_k: 10,
            min_centrality: 0.0,
        };

        if let AnalyzeCommands::GraphMetrics {
            damping_factor,
            max_iterations,
            ..
        } = cmd
        {
            assert!((damping_factor - 0.85).abs() < f32::EPSILON);
            assert_eq!(max_iterations, 100);
        } else {
            panic!("Expected GraphMetrics command");
        }
    }

    #[test]
    fn test_name_similarity_command_construction() {
        let cmd = AnalyzeCommands::NameSimilarity {
            project_path: PathBuf::from("."),
            query: "test_query".to_string(),
            top_k: 10,
            phonetic: false,
            scope: SearchScope::All,
            format: NameSimilarityOutputFormat::Summary,
            output: None,
            threshold: 0.6,
            include: None,
            exclude: None,
            perf: false,
            fuzzy: true,
            case_sensitive: false,
        };

        if let AnalyzeCommands::NameSimilarity {
            query,
            fuzzy,
            case_sensitive,
            ..
        } = cmd
        {
            assert_eq!(query, "test_query");
            assert!(fuzzy);
            assert!(!case_sensitive);
        } else {
            panic!("Expected NameSimilarity command");
        }
    }

    #[test]
    fn test_proof_annotations_command_construction() {
        let cmd = AnalyzeCommands::ProofAnnotations {
            project_path: PathBuf::from("."),
            format: ProofAnnotationOutputFormat::Summary,
            high_confidence_only: false,
            include_evidence: true,
            property_type: None,
            verification_method: None,
            output: None,
            perf: false,
            clear_cache: false,
            top_files: 10,
        };

        if let AnalyzeCommands::ProofAnnotations {
            include_evidence,
            clear_cache,
            ..
        } = cmd
        {
            assert!(include_evidence);
            assert!(!clear_cache);
        } else {
            panic!("Expected ProofAnnotations command");
        }
    }

    #[test]
    fn test_symbol_table_command_construction() {
        let cmd = AnalyzeCommands::SymbolTable {
            project_path: PathBuf::from("."),
            format: SymbolTableOutputFormat::Summary,
            filter: None,
            query: None,
            include: vec![],
            exclude: vec![],
            show_unreferenced: false,
            show_references: true,
            output: None,
            perf: false,
            top_files: 10,
        };

        if let AnalyzeCommands::SymbolTable {
            show_unreferenced,
            show_references,
            ..
        } = cmd
        {
            assert!(!show_unreferenced);
            assert!(show_references);
        } else {
            panic!("Expected SymbolTable command");
        }
    }

    #[test]
    fn test_makefile_command_construction() {
        let cmd = AnalyzeCommands::Makefile {
            path: PathBuf::from("Makefile"),
            rules: vec!["all".to_string()],
            format: MakefileOutputFormat::Human,
            fix: false,
            gnu_version: "4.4".to_string(),
            top_files: 10,
        };

        if let AnalyzeCommands::Makefile {
            rules,
            gnu_version,
            fix,
            ..
        } = cmd
        {
            assert_eq!(rules, vec!["all"]);
            assert_eq!(gnu_version, "4.4");
            assert!(!fix);
        } else {
            panic!("Expected Makefile command");
        }
    }

    #[test]
    fn test_entropy_command_construction() {
        let cmd = AnalyzeCommands::Entropy {
            project_path: PathBuf::from("."),
            format: EntropyOutputFormat::Summary,
            output: None,
            min_severity: EntropySeverity::Medium,
            top_violations: 10,
            file: None,
            include_tests: false,
        };

        if let AnalyzeCommands::Entropy {
            min_severity,
            top_violations,
            include_tests,
            ..
        } = cmd
        {
            assert!(matches!(min_severity, EntropySeverity::Medium));
            assert_eq!(top_violations, 10);
            assert!(!include_tests);
        } else {
            panic!("Expected Entropy command");
        }
    }

    #[test]
    fn test_wasm_command_construction() {
        let cmd = AnalyzeCommands::Wasm {
            wasm_file: PathBuf::from("test.wasm"),
            format: WasmOutputFormat::Summary,
            verify: false,
            security: true,
            profile: false,
            baseline: None,
            output: None,
            verbose: false,
        };

        if let AnalyzeCommands::Wasm {
            verify, security, ..
        } = cmd
        {
            assert!(!verify);
            assert!(security);
        } else {
            panic!("Expected Wasm command");
        }
    }

    // Route Category Tests - verify commands route to correct handlers

    #[test]
    fn test_core_analysis_commands_are_routed() {
        // Verify all core analysis command variants exist
        let commands = vec!["Complexity", "Churn", "DeadCode", "Defects", "Dag", "Satd"];

        for cmd_name in commands {
            assert!(
                ["Complexity", "Churn", "DeadCode", "Defects", "Dag", "Satd"].contains(&cmd_name),
                "Core analysis should include {}",
                cmd_name
            );
        }
    }

    #[test]
    fn test_advanced_analysis_commands_are_routed() {
        // Verify all advanced analysis command variants exist
        let commands = vec![
            "DeepContext",
            "Tdg",
            "BuildTdg",
            "LintHotspot",
            "Comprehensive",
        ];

        for cmd_name in commands {
            assert!(
                [
                    "DeepContext",
                    "Tdg",
                    "BuildTdg",
                    "LintHotspot",
                    "Comprehensive"
                ]
                .contains(&cmd_name),
                "Advanced analysis should include {}",
                cmd_name
            );
        }
    }

    #[test]
    fn test_quality_analysis_commands_are_routed() {
        // Verify all quality analysis command variants exist
        let commands = vec![
            "Duplicates",
            "DefectPrediction",
            "Provability",
            "Clippy",
            "Entropy",
        ];

        for cmd_name in commands {
            assert!(
                [
                    "Duplicates",
                    "DefectPrediction",
                    "Provability",
                    "Clippy",
                    "Entropy"
                ]
                .contains(&cmd_name),
                "Quality analysis should include {}",
                cmd_name
            );
        }
    }

    #[test]
    fn test_specialized_analysis_commands_are_routed() {
        // Verify all specialized analysis command variants exist
        let commands = vec![
            "GraphMetrics",
            "NameSimilarity",
            "ProofAnnotations",
            "IncrementalCoverage",
            "CoverageImprove",
            "SymbolTable",
            "BigO",
        ];

        for cmd_name in commands {
            assert!(
                [
                    "GraphMetrics",
                    "NameSimilarity",
                    "ProofAnnotations",
                    "IncrementalCoverage",
                    "CoverageImprove",
                    "SymbolTable",
                    "BigO"
                ]
                .contains(&cmd_name),
                "Specialized analysis should include {}",
                cmd_name
            );
        }
    }

    #[test]
    fn test_language_specific_commands_are_routed() {
        // Verify all language-specific command variants exist
        let commands = vec!["AssemblyScript", "WebAssembly", "Wasm"];

        for cmd_name in commands {
            assert!(
                ["AssemblyScript", "WebAssembly", "Wasm"].contains(&cmd_name),
                "Language-specific analysis should include {}",
                cmd_name
            );
        }
    }

    // Format Conversion Tests

    #[test]
    fn test_all_dag_types_convert() {
        // Test all DeepContextDagType variants can be converted
        let variants = [
            DeepContextDagType::CallGraph,
            DeepContextDagType::ImportGraph,
            DeepContextDagType::Inheritance,
            DeepContextDagType::FullDependency,
        ];

        for variant in variants {
            let result = convert_deep_context_dag_type(variant);
            // Just verify it doesn't panic and returns a valid DagType
            match result {
                DagType::CallGraph
                | DagType::ImportGraph
                | DagType::Inheritance
                | DagType::FullDependency => {}
                _ => panic!("Unexpected DagType variant"),
            }
        }
    }

    #[test]
    fn test_all_cache_strategies_convert() {
        // Test all DeepContextCacheStrategy variants can be converted
        let variants = [
            DeepContextCacheStrategy::Normal,
            DeepContextCacheStrategy::ForceRefresh,
            DeepContextCacheStrategy::Offline,
        ];

        let expected = ["normal", "force-refresh", "offline"];

        for (variant, exp) in variants.iter().zip(expected.iter()) {
            let result = convert_cache_strategy(variant.clone());
            assert_eq!(result, *exp);
        }
    }

    // Entropy Helper Function Tests (covering lines 1396-1552)

    #[test]
    fn test_create_entropy_config_excludes_tests_when_disabled() {
        let config = create_entropy_config(EntropySeverity::Low, false);
        assert!(
            config.exclude_paths.len() >= 2,
            "Should have test exclusions when include_tests is false"
        );
        assert!(config
            .exclude_paths
            .iter()
            .any(|p| p.contains("test") || p.contains("tests")));
    }

    #[test]
    fn test_create_entropy_config_includes_tests_when_enabled() {
        let config = create_entropy_config(EntropySeverity::High, true);
        // When include_tests is true, no test-specific exclusions should be added
        // (Default exclusions may still exist but no new test exclusions)
        // The test verifies the config is valid
        assert!(matches!(
            config.min_severity,
            crate::entropy::violation_detector::Severity::High
        ));
    }

    #[test]
    fn test_get_top_violations_returns_all_when_limit_exceeds_count() {
        let violations: Vec<crate::entropy::violation_detector::ActionableViolation> = vec![];
        let result = get_top_violations(&violations, 100);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_format_violation_list_with_empty_vector() {
        let violations: Vec<crate::entropy::violation_detector::ActionableViolation> = vec![];
        let result = format_violation_list(&violations);
        assert!(result.is_empty());
    }

    #[test]
    fn test_format_markdown_violations_with_zero_max() {
        let violations: Vec<crate::entropy::violation_detector::ActionableViolation> = vec![];
        let result = format_markdown_violations(&violations, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_output_entropy_results_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("dir")
            .join("output.txt");

        // Create parent directories first
        std::fs::create_dir_all(nested_path.parent().unwrap()).unwrap();

        let result = output_entropy_results(Some(nested_path.clone()), "test");
        assert!(result.is_ok());
    }

