        let result = output_entropy_results(Some(output_path.clone()), "test content");
        assert!(result.is_ok());
        assert!(output_path.exists());

        let content = std::fs::read_to_string(output_path).unwrap();
        assert_eq!(content, "test content");
    }

    // AnalyzeCommands Enum Variant Construction Tests

    #[test]
    fn test_complexity_command_construction() {
        let cmd = AnalyzeCommands::Complexity {
            path: PathBuf::from("."),
            project_path: None,
            file: None,
            files: vec![],
            toolchain: None,
            format: ComplexityOutputFormat::Summary,
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

        // Verify command can be pattern matched
        if let AnalyzeCommands::Complexity { path, top_files, .. } = cmd {
            assert_eq!(path, PathBuf::from("."));
            assert_eq!(top_files, 10);
        } else {
            panic!("Expected Complexity command");
        }
    }

    #[test]
    fn test_churn_command_construction() {
        let cmd = AnalyzeCommands::Churn {
            project_path: PathBuf::from("."),
            days: 30,
            format: ChurnOutputFormat::Summary,
            output: None,
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        if let AnalyzeCommands::Churn { days, .. } = cmd {
            assert_eq!(days, 30);
        } else {
            panic!("Expected Churn command");
        }
    }

    #[test]
    fn test_dead_code_command_construction() {
        let cmd = AnalyzeCommands::DeadCode {
            path: PathBuf::from("."),
            format: DeadCodeOutputFormat::Summary,
            top_files: Some(10),
            include_unreachable: false,
            min_dead_lines: 10,
            include_tests: false,
            output: None,
            fail_on_violation: false,
            max_percentage: 15.0,
            timeout: 60,
            include: vec![],
            exclude: vec![],
            max_depth: 8,
        };

        if let AnalyzeCommands::DeadCode {
            min_dead_lines,
            max_percentage,
            ..
        } = cmd
        {
            assert_eq!(min_dead_lines, 10);
            assert!((max_percentage - 15.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected DeadCode command");
        }
    }

    #[test]
    fn test_dag_command_construction() {
        let cmd = AnalyzeCommands::Dag {
            dag_type: DagType::CallGraph,
            project_path: PathBuf::from("."),
            output: None,
            max_depth: Some(5),
            target_nodes: None,
            filter_external: false,
            show_complexity: false,
            include_duplicates: false,
            include_dead_code: false,
            enhanced: false,
        };

        if let AnalyzeCommands::Dag {
            dag_type,
            max_depth,
            ..
        } = cmd
        {
            assert!(matches!(dag_type, DagType::CallGraph));
            assert_eq!(max_depth, Some(5));
        } else {
            panic!("Expected Dag command");
        }
    }

    #[test]
    fn test_satd_command_construction() {
        let cmd = AnalyzeCommands::Satd {
            path: PathBuf::from("."),
            format: SatdOutputFormat::Summary,
            severity: Some(SatdSeverity::High),
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
        };

        if let AnalyzeCommands::Satd {
            days, strict, ..
        } = cmd
        {
            assert_eq!(days, 30);
            assert!(!strict);
        } else {
            panic!("Expected Satd command");
        }
    }

    #[test]
    fn test_deep_context_command_construction() {
        let cmd = AnalyzeCommands::DeepContext {
            project_path: PathBuf::from("."),
            output: None,
            format: DeepContextOutputFormat::Markdown,
            full: false,
            include: vec![],
            exclude: vec![],
            period_days: 30,
            dag_type: DeepContextDagType::CallGraph,
            max_depth: None,
            include_patterns: vec![],
            exclude_patterns: vec![],
            cache_strategy: DeepContextCacheStrategy::Normal,
            parallel: None,
            verbose: false,
            top_files: 10,
        };

        if let AnalyzeCommands::DeepContext {
            period_days,
            verbose,
            ..
        } = cmd
        {
            assert_eq!(period_days, 30);
            assert!(!verbose);
        } else {
            panic!("Expected DeepContext command");
        }
    }

    #[test]
    fn test_tdg_command_construction() {
        let cmd = AnalyzeCommands::Tdg {
            path: PathBuf::from("."),
            threshold: 1.5,
            top_files: 10,
            format: TdgOutputFormat::Table,
            include_components: false,
            output: None,
            critical_only: false,
            verbose: false,
            ml: false,
        };

        if let AnalyzeCommands::Tdg {
            threshold,
            critical_only,
            ..
        } = cmd
        {
            assert!((threshold - 1.5).abs() < f64::EPSILON);
            assert!(!critical_only);
        } else {
            panic!("Expected Tdg command");
        }
    }

    #[test]
    fn test_build_tdg_command_construction() {
        let cmd = AnalyzeCommands::BuildTdg {
            path: PathBuf::from("."),
            release: true,
            threshold: 2.0,
            fail_on_regression: false,
            tdg_only: true,
            top_files: 10,
            format: TdgOutputFormat::Table,
            output: None,
        };

        if let AnalyzeCommands::BuildTdg {
            release,
            tdg_only,
            threshold,
            ..
        } = cmd
        {
            assert!(release);
            assert!(tdg_only);
            assert!((threshold - 2.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected BuildTdg command");
        }
    }

    #[test]
    fn test_lint_hotspot_command_construction() {
        let cmd = AnalyzeCommands::LintHotspot {
            project_path: PathBuf::from("."),
            file: None,
            format: LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.8,
            enforce: false,
            dry_run: true,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: "-W warnings".to_string(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        if let AnalyzeCommands::LintHotspot {
            max_density,
            dry_run,
            ..
        } = cmd
        {
            assert!((max_density - 5.0).abs() < f64::EPSILON);
            assert!(dry_run);
        } else {
            panic!("Expected LintHotspot command");
        }
    }

    #[test]
    fn test_duplicates_command_construction() {
        let cmd = AnalyzeCommands::Duplicates {
            project_path: PathBuf::from("."),
            detection_type: DuplicateType::All,
            threshold: 0.85,
            min_lines: 5,
            max_tokens: 128,
            format: DuplicateOutputFormat::Summary,
            perf: false,
            include: None,
            exclude: None,
            output: None,
            top_files: 10,
        };

        if let AnalyzeCommands::Duplicates {
            threshold,
            min_lines,
            ..
        } = cmd
        {
            assert!((threshold - 0.85).abs() < f32::EPSILON);
            assert_eq!(min_lines, 5);
        } else {
            panic!("Expected Duplicates command");
        }
    }

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
