// Extended analysis decode methods for CliAdapter:
// serve, demo, lint_hotspot, makefile, duplicates, defect_prediction,
// comprehensive, graph_metrics, name_similarity, proof_annotations,
// incremental_coverage, symbol_table, big_o, assemblyscript, webassembly,
// and utility functions (cli_only_command_error, format_to_extension_string)

impl CliAdapter {

    fn decode_serve(
        host: &str,
        port: u16,
        cors: bool,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let body = json!({
            "host": host,
            "port": port,
            "cors": cors
        });
        Ok((Method::POST, "/api/v1/serve".to_string(), body, None))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_demo(
        path: &Option<PathBuf>,
        url: &Option<String>,
        format: &OutputFormat,
        no_browser: bool,
        port: &Option<u16>,
        cli: bool,
        target_nodes: usize,
        centrality_threshold: f64,
        merge_threshold: usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let body = json!({
            "path": path.as_ref().map(|p| p.to_string_lossy().to_string()),
            "url": url,
            "format": format!("{format:?}").to_lowercase(),
            "no_browser": &no_browser,
            "port": port,
            "cli_mode": &cli,
            "target_nodes": &target_nodes,
            "centrality_threshold": &centrality_threshold,
            "merge_threshold": &merge_threshold
        });
        Ok((Method::POST, "/api/v1/demo".to_string(), body, None))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_lint_hotspot(
        project_path: &std::path::Path,
        file: &Option<PathBuf>,
        format: &crate::cli::LintHotspotOutputFormat,
        max_density: &f64,
        min_confidence: &f64,
        enforce: &bool,
        dry_run: &bool,
        enforcement_metadata: &bool,
        output: &Option<PathBuf>,
        perf: &bool,
        clippy_flags: &String,
        top_files: &usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let params = json!({
            "project_path": project_path,
            "file": file,
            "format": format,
            "max_density": max_density,
            "min_confidence": min_confidence,
            "enforce": enforce,
            "dry_run": dry_run,
            "enforcement_metadata": enforcement_metadata,
            "output": output,
            "perf": perf,
            "clippy_flags": clippy_flags,
            "top_files": top_files,
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/lint-hotspot".to_string(),
            params,
            None,
        ))
    }

    fn decode_analyze_makefile(
        path: &std::path::Path,
        rules: &Vec<String>,
        format: &crate::cli::MakefileOutputFormat,
        fix: &bool,
        gnu_version: &String,
        top_files: &usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let params = json!({
            "path": path,
            "rules": rules,
            "fix": fix,
            "gnu_version": gnu_version,
            "format": format,
            "top_files": top_files,
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/makefile".to_string(),
            params,
            None,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_duplicates(
        project_path: &std::path::Path,
        detection_type: &crate::cli::DuplicateType,
        threshold: &f32,
        min_lines: &usize,
        max_tokens: &usize,
        format: &crate::cli::DuplicateOutputFormat,
        perf: &bool,
        include: &Option<String>,
        exclude: &Option<String>,
        output: &Option<PathBuf>,
        top_files: &usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let params = json!({
            "project_path": project_path,
            "detection_type": detection_type,
            "threshold": threshold,
            "min_lines": min_lines,
            "max_tokens": max_tokens,
            "format": format,
            "perf": perf,
            "include": include,
            "exclude": exclude,
            "output": output,
            "top_files": top_files,
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/duplicates".to_string(),
            params,
            None,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_defect_prediction(
        project_path: &std::path::Path,
        confidence_threshold: &f32,
        min_lines: &usize,
        include_low_confidence: &bool,
        format: &crate::cli::DefectPredictionOutputFormat,
        high_risk_only: &bool,
        include_recommendations: &bool,
        include: &Option<String>,
        exclude: &Option<String>,
        output: &Option<PathBuf>,
        perf: &bool,
        top_files: &usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let params = json!({
            "project_path": project_path,
            "confidence_threshold": confidence_threshold,
            "min_lines": min_lines,
            "include_low_confidence": include_low_confidence,
            "format": format,
            "high_risk_only": high_risk_only,
            "include_recommendations": include_recommendations,
            "include": include,
            "exclude": exclude,
            "output": output,
            "perf": perf,
            "top_files": top_files,
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/defect-prediction".to_string(),
            params,
            None,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_comprehensive(
        project_path: &std::path::Path,
        file: &Option<PathBuf>,
        files: &[PathBuf],
        format: &crate::cli::ComprehensiveOutputFormat,
        include_duplicates: &bool,
        include_dead_code: &bool,
        include_defects: &bool,
        include_complexity: &bool,
        include_tdg: &bool,
        confidence_threshold: &f32,
        min_lines: &usize,
        include: &Option<String>,
        exclude: &Option<String>,
        output: &Option<PathBuf>,
        perf: &bool,
        executive_summary: &bool,
        top_files: &usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let params = json!({
            "project_path": project_path,
            "file": file,
            "files": files.iter().map(|f| f.to_string_lossy()).collect::<Vec<_>>(),
            "format": format,
            "include_duplicates": include_duplicates,
            "include_dead_code": include_dead_code,
            "include_defects": include_defects,
            "include_complexity": include_complexity,
            "include_tdg": include_tdg,
            "confidence_threshold": confidence_threshold,
            "min_lines": min_lines,
            "include": include,
            "exclude": exclude,
            "output": output,
            "perf": perf,
            "executive_summary": executive_summary,
            "top_files": top_files,
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/comprehensive".to_string(),
            params,
            None,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_graph_metrics(
        project_path: &std::path::Path,
        metrics: &[crate::cli::GraphMetricType],
        pagerank_seeds: &[String],
        damping_factor: &f32,
        max_iterations: &usize,
        convergence_threshold: &f64,
        format: &crate::cli::GraphMetricsOutputFormat,
        include: &Option<String>,
        exclude: &Option<String>,
        output: &Option<PathBuf>,
        export_graphml: &bool,
        perf: &bool,
        top_k: &usize,
        min_centrality: &f64,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let params = json!({
            "project_path": project_path,
            "metrics": metrics.iter().map(graph_metric_type_to_string).collect::<Vec<_>>(),
            "pagerank_seeds": pagerank_seeds,
            "damping_factor": damping_factor,
            "max_iterations": max_iterations,
            "convergence_threshold": convergence_threshold,
            "format": graph_metrics_format_to_string(format),
            "include": include,
            "exclude": exclude,
            "output": output,
            "export_graphml": export_graphml,
            "perf": perf,
            "top_k": top_k,
            "min_centrality": min_centrality,
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/graph-metrics".to_string(),
            params,
            None,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_name_similarity(
        project_path: &std::path::Path,
        query: &str,
        top_k: &usize,
        phonetic: &bool,
        scope: &crate::cli::SearchScope,
        threshold: &f32,
        format: &crate::cli::NameSimilarityOutputFormat,
        include: &Option<String>,
        exclude: &Option<String>,
        output: &Option<PathBuf>,
        perf: &bool,
        fuzzy: &bool,
        case_sensitive: &bool,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let params = json!({
            "project_path": project_path,
            "query": query,
            "top_k": top_k,
            "phonetic": phonetic,
            "scope": match scope {
                crate::cli::SearchScope::Functions => "functions",
                crate::cli::SearchScope::Types => "types",
                crate::cli::SearchScope::Variables => "variables",
                crate::cli::SearchScope::All => "all",
            },
            "threshold": threshold,
            "format": name_similarity_format_to_string(format),
            "include": include,
            "exclude": exclude,
            "output": output,
            "perf": perf,
            "fuzzy": fuzzy,
            "case_sensitive": case_sensitive,
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/name-similarity".to_string(),
            params,
            None,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_proof_annotations(
        project_path: &std::path::Path,
        format: &crate::cli::ProofAnnotationOutputFormat,
        high_confidence_only: &bool,
        include_evidence: &bool,
        property_type: &Option<crate::cli::PropertyTypeFilter>,
        verification_method: &Option<crate::cli::VerificationMethodFilter>,
        output: &Option<PathBuf>,
        perf: &bool,
        clear_cache: &bool,
        top_files: &usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let params = json!({
            "project_path": project_path,
            "format": proof_annotation_format_to_string(format),
            "high_confidence_only": high_confidence_only,
            "include_evidence": include_evidence,
            "property_type": property_type.as_ref().map(property_type_filter_to_string),
            "verification_method": verification_method.as_ref().map(verification_method_filter_to_string),
            "output": output,
            "perf": perf,
            "clear_cache": clear_cache,
            "top_files": top_files,
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/proof-annotations".to_string(),
            params,
            None,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_incremental_coverage(
        project_path: &std::path::Path,
        base_branch: &String,
        target_branch: &Option<String>,
        format: &crate::cli::IncrementalCoverageOutputFormat,
        coverage_threshold: &f64,
        changed_files_only: &bool,
        detailed: &bool,
        output: &Option<PathBuf>,
        perf: &bool,
        cache_dir: &Option<PathBuf>,
        force_refresh: &bool,
        top_files: &usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let params = json!({
            "project_path": project_path,
            "base_branch": base_branch,
            "target_branch": target_branch,
            "format": incremental_coverage_format_to_string(format),
            "coverage_threshold": coverage_threshold,
            "changed_files_only": changed_files_only,
            "detailed": detailed,
            "output": output,
            "perf": perf,
            "cache_dir": cache_dir,
            "force_refresh": force_refresh,
            "top_files": top_files,
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/incremental-coverage".to_string(),
            params,
            None,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_symbol_table(
        project_path: &std::path::Path,
        format: &crate::cli::SymbolTableOutputFormat,
        query: &Option<String>,
        filter: &Option<crate::cli::SymbolTypeFilter>,
        include: &Vec<String>,
        exclude: &Vec<String>,
        show_unreferenced: &bool,
        show_references: &bool,
        output: &Option<PathBuf>,
        perf: &bool,
        top_files: &usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let params = json!({
            "project_path": project_path,
            "format": symbol_table_format_to_string(format),
            "query": query,
            "filter": filter.as_ref().map(symbol_type_filter_to_string),
            "include": include,
            "exclude": exclude,
            "show_unreferenced": show_unreferenced,
            "show_references": show_references,
            "output": output,
            "perf": perf,
            "top_files": top_files,
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/symbol-table".to_string(),
            params,
            None,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_big_o(
        project_path: &std::path::Path,
        format: &crate::cli::BigOOutputFormat,
        confidence_threshold: &u8,
        analyze_space: &bool,
        include: &Vec<String>,
        exclude: &Vec<String>,
        output: &Option<PathBuf>,
        perf: &bool,
        high_complexity_only: &bool,
        top_files: &usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let params = json!({
            "project_path": project_path,
            "format": big_o_format_to_string(format),
            "confidence_threshold": confidence_threshold,
            "analyze_space": analyze_space,
            "include": include,
            "exclude": exclude,
            "output": output,
            "perf": perf,
            "high_complexity_only": high_complexity_only,
            "top_files": top_files,
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/big-o".to_string(),
            params,
            None,
        ))
    }

    fn decode_analyze_assemblyscript(
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        Ok((
            Method::POST,
            "/api/v1/analyze/assemblyscript".to_string(),
            json!({}),
            None,
        ))
    }

    fn decode_analyze_webassembly(
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        Ok((
            Method::POST,
            "/api/v1/analyze/webassembly".to_string(),
            json!({}),
            None,
        ))
    }

    fn cli_only_command_error(
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        Err(ProtocolError::InvalidFormat(
            "Command should be handled directly by CLI".to_string(),
        ))
    }

    fn format_to_extension_string(format: &OutputFormat) -> &'static str {
        match format {
            OutputFormat::Json => "json",
            OutputFormat::Table => "table",
            OutputFormat::Yaml => "yaml",
            OutputFormat::Markdown => "md",
            OutputFormat::Csv => "csv",
            OutputFormat::Summary => "txt",
            OutputFormat::Text => "txt",
            OutputFormat::Plain => "txt",
            OutputFormat::Junit => "xml",
        }
    }
}
