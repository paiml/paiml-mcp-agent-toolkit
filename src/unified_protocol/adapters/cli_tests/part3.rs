            "summary"
        );
        assert_eq!(
            satd_format_to_string(&crate::cli::SatdOutputFormat::Json),
            "json"
        );
        assert_eq!(
            satd_format_to_string(&crate::cli::SatdOutputFormat::Sarif),
            "sarif"
        );
        assert_eq!(
            satd_format_to_string(&crate::cli::SatdOutputFormat::Markdown),
            "markdown"
        );
    }

    #[test]
    fn test_satd_severity_all_variants() {
        assert_eq!(
            satd_severity_to_string(&crate::cli::SatdSeverity::Critical),
            "critical"
        );
        assert_eq!(
            satd_severity_to_string(&crate::cli::SatdSeverity::High),
            "high"
        );
        assert_eq!(
            satd_severity_to_string(&crate::cli::SatdSeverity::Medium),
            "medium"
        );
        assert_eq!(
            satd_severity_to_string(&crate::cli::SatdSeverity::Low),
            "low"
        );
    }

    #[test]
    fn test_deep_context_format_all_variants() {
        assert_eq!(
            deep_context_format_to_string(&crate::cli::DeepContextOutputFormat::Markdown),
            "markdown"
        );
        assert_eq!(
            deep_context_format_to_string(&crate::cli::DeepContextOutputFormat::Json),
            "json"
        );
        assert_eq!(
            deep_context_format_to_string(&crate::cli::DeepContextOutputFormat::Sarif),
            "sarif"
        );
    }

    #[test]
    fn test_deep_context_dag_type_all_variants() {
        assert_eq!(
            deep_context_dag_type_to_string(&crate::cli::DeepContextDagType::CallGraph),
            "call-graph"
        );
        assert_eq!(
            deep_context_dag_type_to_string(&crate::cli::DeepContextDagType::ImportGraph),
            "import-graph"
        );
        assert_eq!(
            deep_context_dag_type_to_string(&crate::cli::DeepContextDagType::Inheritance),
            "inheritance"
        );
        assert_eq!(
            deep_context_dag_type_to_string(&crate::cli::DeepContextDagType::FullDependency),
            "full-dependency"
        );
    }

    #[test]
    fn test_deep_context_cache_strategy_all_variants() {
        assert_eq!(
            deep_context_cache_strategy_to_string(&crate::cli::DeepContextCacheStrategy::Normal),
            "normal"
        );
        assert_eq!(
            deep_context_cache_strategy_to_string(
                &crate::cli::DeepContextCacheStrategy::ForceRefresh
            ),
            "force-refresh"
        );
        assert_eq!(
            deep_context_cache_strategy_to_string(&crate::cli::DeepContextCacheStrategy::Offline),
            "offline"
        );
    }

    #[test]
    fn test_tdg_format_all_variants() {
        assert_eq!(tdg_format_to_string(&TdgOutputFormat::Table), "table");
        assert_eq!(tdg_format_to_string(&TdgOutputFormat::Json), "json");
        assert_eq!(tdg_format_to_string(&TdgOutputFormat::Markdown), "markdown");
        assert_eq!(tdg_format_to_string(&TdgOutputFormat::Sarif), "sarif");
    }

    #[test]
    fn test_provability_format_all_variants() {
        assert_eq!(
            provability_format_to_string(&ProvabilityOutputFormat::Summary),
            "summary"
        );
        assert_eq!(
            provability_format_to_string(&ProvabilityOutputFormat::Full),
            "full"
        );
        assert_eq!(
            provability_format_to_string(&ProvabilityOutputFormat::Json),
            "json"
        );
        assert_eq!(
            provability_format_to_string(&ProvabilityOutputFormat::Sarif),
            "sarif"
        );
        assert_eq!(
            provability_format_to_string(&ProvabilityOutputFormat::Markdown),
            "markdown"
        );
    }

    #[test]
    fn test_graph_metric_type_all_variants() {
        assert_eq!(graph_metric_type_to_string(&GraphMetricType::All), "all");
        assert_eq!(
            graph_metric_type_to_string(&GraphMetricType::Centrality),
            "centrality"
        );
        assert_eq!(
            graph_metric_type_to_string(&GraphMetricType::Betweenness),
            "betweenness"
        );
        assert_eq!(
            graph_metric_type_to_string(&GraphMetricType::Closeness),
            "closeness"
        );
        assert_eq!(
            graph_metric_type_to_string(&GraphMetricType::PageRank),
            "pagerank"
        );
        assert_eq!(
            graph_metric_type_to_string(&GraphMetricType::Clustering),
            "clustering"
        );
        assert_eq!(
            graph_metric_type_to_string(&GraphMetricType::Components),
            "components"
        );
    }

    #[test]
    fn test_graph_metrics_format_all_variants() {
        assert_eq!(
            graph_metrics_format_to_string(&GraphMetricsOutputFormat::Summary),
            "summary"
        );
        assert_eq!(
            graph_metrics_format_to_string(&GraphMetricsOutputFormat::Detailed),
            "detailed"
        );
        assert_eq!(
            graph_metrics_format_to_string(&GraphMetricsOutputFormat::Human),
            "human"
        );
        assert_eq!(
            graph_metrics_format_to_string(&GraphMetricsOutputFormat::Json),
            "json"
        );
        assert_eq!(
            graph_metrics_format_to_string(&GraphMetricsOutputFormat::Csv),
            "csv"
        );
        assert_eq!(
            graph_metrics_format_to_string(&GraphMetricsOutputFormat::GraphML),
            "graphml"
        );
        assert_eq!(
            graph_metrics_format_to_string(&GraphMetricsOutputFormat::Markdown),
            "markdown"
        );
    }

    #[test]
    fn test_name_similarity_format_all_variants() {
        assert_eq!(
            name_similarity_format_to_string(&NameSimilarityOutputFormat::Summary),
            "summary"
        );
        assert_eq!(
            name_similarity_format_to_string(&NameSimilarityOutputFormat::Detailed),
            "detailed"
        );
        assert_eq!(
            name_similarity_format_to_string(&NameSimilarityOutputFormat::Human),
            "human"
        );
        assert_eq!(
            name_similarity_format_to_string(&NameSimilarityOutputFormat::Json),
            "json"
        );
        assert_eq!(
            name_similarity_format_to_string(&NameSimilarityOutputFormat::Csv),
            "csv"
        );
        assert_eq!(
            name_similarity_format_to_string(&NameSimilarityOutputFormat::Markdown),
            "markdown"
        );
    }

    #[test]
    fn test_property_type_filter_all_variants() {
        assert_eq!(
            property_type_filter_to_string(&PropertyTypeFilter::All),
            "all"
        );
        assert_eq!(
            property_type_filter_to_string(&PropertyTypeFilter::MemorySafety),
            "memory-safety"
        );
        assert_eq!(
            property_type_filter_to_string(&PropertyTypeFilter::ThreadSafety),
            "thread-safety"
        );
        assert_eq!(
            property_type_filter_to_string(&PropertyTypeFilter::DataRaceFreeze),
            "data-race-freeze"
        );
        assert_eq!(
            property_type_filter_to_string(&PropertyTypeFilter::Termination),
            "termination"
        );
        assert_eq!(
            property_type_filter_to_string(&PropertyTypeFilter::FunctionalCorrectness),
            "functional-correctness"
        );
        assert_eq!(
            property_type_filter_to_string(&PropertyTypeFilter::ResourceBounds),
            "resource-bounds"
        );
    }

    #[test]
    fn test_verification_method_filter_all_variants() {
        assert_eq!(
            verification_method_filter_to_string(&VerificationMethodFilter::All),
            "all"
        );
        assert_eq!(
            verification_method_filter_to_string(&VerificationMethodFilter::FormalProof),
            "formal-proof"
        );
        assert_eq!(
            verification_method_filter_to_string(&VerificationMethodFilter::ModelChecking),
            "model-checking"
        );
        assert_eq!(
            verification_method_filter_to_string(&VerificationMethodFilter::StaticAnalysis),
            "static-analysis"
        );
        assert_eq!(
            verification_method_filter_to_string(&VerificationMethodFilter::AbstractInterpretation),
            "abstract-interpretation"
        );
        assert_eq!(
            verification_method_filter_to_string(&VerificationMethodFilter::BorrowChecker),
            "borrow-checker"
        );
    }

    #[test]
    fn test_proof_annotation_format_all_variants() {
        assert_eq!(
            proof_annotation_format_to_string(&ProofAnnotationOutputFormat::Summary),
            "summary"
        );
        assert_eq!(
            proof_annotation_format_to_string(&ProofAnnotationOutputFormat::Full),
            "full"
        );
        assert_eq!(
            proof_annotation_format_to_string(&ProofAnnotationOutputFormat::Json),
            "json"
        );
        assert_eq!(
            proof_annotation_format_to_string(&ProofAnnotationOutputFormat::Markdown),
            "markdown"
        );
        assert_eq!(
            proof_annotation_format_to_string(&ProofAnnotationOutputFormat::Sarif),
            "sarif"
        );
    }

    #[test]
    fn test_incremental_coverage_format_all_variants() {
        assert_eq!(
            incremental_coverage_format_to_string(&IncrementalCoverageOutputFormat::Summary),
            "summary"
        );
        assert_eq!(
            incremental_coverage_format_to_string(&IncrementalCoverageOutputFormat::Detailed),
            "detailed"
        );
        assert_eq!(
            incremental_coverage_format_to_string(&IncrementalCoverageOutputFormat::Json),
            "json"
        );
        assert_eq!(
            incremental_coverage_format_to_string(&IncrementalCoverageOutputFormat::Markdown),
            "markdown"
        );
        assert_eq!(
            incremental_coverage_format_to_string(&IncrementalCoverageOutputFormat::Lcov),
            "lcov"
        );
        assert_eq!(
            incremental_coverage_format_to_string(&IncrementalCoverageOutputFormat::Delta),
            "delta"
        );
        assert_eq!(
            incremental_coverage_format_to_string(&IncrementalCoverageOutputFormat::Sarif),
            "sarif"
        );
    }

    #[test]
    fn test_symbol_type_filter_all_variants() {
        assert_eq!(symbol_type_filter_to_string(&SymbolTypeFilter::All), "all");
        assert_eq!(
            symbol_type_filter_to_string(&SymbolTypeFilter::Functions),
            "functions"
        );
        assert_eq!(
            symbol_type_filter_to_string(&SymbolTypeFilter::Classes),
            "classes"
        );
        assert_eq!(
            symbol_type_filter_to_string(&SymbolTypeFilter::Types),
            "types"
        );
        assert_eq!(
            symbol_type_filter_to_string(&SymbolTypeFilter::Variables),
            "variables"
        );
        assert_eq!(
            symbol_type_filter_to_string(&SymbolTypeFilter::Modules),
            "modules"
        );
    }

    #[test]
    fn test_symbol_table_format_all_variants() {
        assert_eq!(
            symbol_table_format_to_string(&SymbolTableOutputFormat::Summary),
            "summary"
        );
        assert_eq!(
            symbol_table_format_to_string(&SymbolTableOutputFormat::Detailed),
            "detailed"
        );
        assert_eq!(
            symbol_table_format_to_string(&SymbolTableOutputFormat::Human),
            "human"
        );
        assert_eq!(
            symbol_table_format_to_string(&SymbolTableOutputFormat::Json),
            "json"
        );
        assert_eq!(
            symbol_table_format_to_string(&SymbolTableOutputFormat::Csv),
            "csv"
        );
    }

    #[test]
    fn test_big_o_format_all_variants() {
        assert_eq!(
            big_o_format_to_string(&BigOOutputFormat::Summary),
            "summary"
        );
        assert_eq!(big_o_format_to_string(&BigOOutputFormat::Json), "json");
        assert_eq!(
            big_o_format_to_string(&BigOOutputFormat::Markdown),
            "markdown"
        );
        assert_eq!(
            big_o_format_to_string(&BigOOutputFormat::Detailed),
            "detailed"
        );
    }

    #[test]
    fn test_format_to_extension_string() {
        assert_eq!(
            CliAdapter::format_to_extension_string(&OutputFormat::Json),
            "json"
        );
        assert_eq!(
            CliAdapter::format_to_extension_string(&OutputFormat::Table),
            "table"
        );
        assert_eq!(
            CliAdapter::format_to_extension_string(&OutputFormat::Yaml),
            "yaml"
        );
    }

    // === Decode Function Tests ===

    #[test]
    fn test_decode_analyze_duplicates() {
        let result = CliAdapter::decode_analyze_duplicates(
            &PathBuf::from("."),
            &DuplicateType::Exact,
            &0.8,
            &10,
            &1000,
            &DuplicateOutputFormat::Json,
            &false,
            &Some("*.rs".to_string()),
            &Some("test_*".to_string()),
            &None,
            &10,
        );

        assert!(result.is_ok());
        let (method, path, _, _) = result.unwrap();
        assert_eq!(method, Method::POST);
        assert_eq!(path, "/api/v1/analyze/duplicates");
    }

    #[test]
    fn test_decode_analyze_defect_prediction() {
        let result = CliAdapter::decode_analyze_defect_prediction(
            &PathBuf::from("."),
            &0.75,
            &50,
            &true,
            &DefectPredictionOutputFormat::Json,
            &true,
            &true,
            &None,
            &None,
            &None,
            &false,
            &10,
        );

