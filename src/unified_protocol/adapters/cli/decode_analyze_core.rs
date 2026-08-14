// Core analysis decode methods for CliAdapter:
// complexity (with migration), churn, dag, dead_code, satd, deep_context, tdg, provability

impl CliAdapter {
    /// Extract Method (Toyota Way): Handle parameter migration for complexity analysis
    /// Complexity reduction: Extracted complex parameter logic from main dispatch
    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_complexity_with_migration(
        path: &std::path::Path,
        project_path: &Option<std::path::PathBuf>,
        file: &Option<std::path::PathBuf>,
        files: &[std::path::PathBuf],
        toolchain: &Option<String>,
        format: &ComplexityOutputFormat,
        output: &Option<std::path::PathBuf>,
        max_cyclomatic: &Option<u16>,
        max_cognitive: &Option<u16>,
        include: &[String],
        watch: bool,
        top_files: usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        // Handle parameter migration: use new 'path' or deprecated 'project_path'
        let analysis_path = if let Some(deprecated_path) = project_path {
            deprecated_path.as_ref()
        } else {
            path
        };
        Self::decode_analyze_complexity(
            analysis_path,
            file,
            files,
            toolchain,
            format,
            output,
            max_cyclomatic,
            max_cognitive,
            include,
            watch,
            top_files,
        )
    }

    fn decode_analyze_churn(
        project_path: &std::path::Path,
        days: u32,
        format: &ChurnOutputFormat,
        output: &Option<std::path::PathBuf>,
        top_files: usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let body = json!({
            "project_path": project_path.to_string_lossy(),
            "period_days": &days,
            "format": churn_format_to_string(format),
            "output_path": output,
            "top_files": top_files
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/churn".to_string(),
            body,
            Some(OutputFormat::Json),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_complexity(
        project_path: &std::path::Path,
        file: &Option<std::path::PathBuf>,
        files: &[std::path::PathBuf],
        toolchain: &Option<String>,
        format: &ComplexityOutputFormat,
        output: &Option<std::path::PathBuf>,
        max_cyclomatic: &Option<u16>,
        max_cognitive: &Option<u16>,
        include: &[String],
        watch: bool,
        top_files: usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let body = json!({
            "project_path": project_path.to_string_lossy(),
            "file": file.as_ref().map(|f| f.to_string_lossy()),
            "files": files.iter().map(|f| f.to_string_lossy()).collect::<Vec<_>>(),
            "toolchain": toolchain,
            "format": complexity_format_to_string(format),
            "output_path": output,
            "max_cyclomatic": max_cyclomatic,
            "max_cognitive": max_cognitive,
            "include_patterns": include,
            "watch": &watch,
            "top_files": &top_files
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/complexity".to_string(),
            body,
            Some(OutputFormat::Json),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_dag(
        dag_type: &DagType,
        project_path: &std::path::Path,
        output: &Option<std::path::PathBuf>,
        max_depth: &Option<usize>,
        target_nodes: &Option<usize>,
        filter_external: bool,
        show_complexity: bool,
        include_duplicates: bool,
        include_dead_code: bool,
        enhanced: bool,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let body = json!({
            "project_path": project_path.to_string_lossy(),
            "dag_type": dag_type_to_string(dag_type),
            "output_path": output,
            "max_depth": max_depth,
            "target_nodes": target_nodes,
            "filter_external": &filter_external,
            "show_complexity": &show_complexity,
            "include_duplicates": &include_duplicates,
            "include_dead_code": &include_dead_code,
            "enhanced": &enhanced
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/dag".to_string(),
            body,
            Some(OutputFormat::Json),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_dead_code(
        path: &std::path::Path,
        format: &crate::cli::DeadCodeOutputFormat,
        top_files: &Option<usize>,
        include_unreachable: bool,
        min_dead_lines: usize,
        include_tests: bool,
        output: &Option<std::path::PathBuf>,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let body = json!({
            "project_path": path.to_string_lossy(),
            "format": dead_code_format_to_string(format),
            "top_files": top_files,
            "include_unreachable": &include_unreachable,
            "min_dead_lines": &min_dead_lines,
            "include_tests": &include_tests,
            "output_path": output
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/dead-code".to_string(),
            body,
            Some(OutputFormat::Json),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_satd(
        path: &std::path::Path,
        format: &crate::cli::SatdOutputFormat,
        severity: &Option<crate::cli::SatdSeverity>,
        critical_only: bool,
        include_tests: bool,
        strict: bool,
        evolution: bool,
        days: u32,
        metrics: bool,
        output: &Option<std::path::PathBuf>,
        top_files: usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let body = json!({
            "project_path": path.to_string_lossy(),
            "format": satd_format_to_string(format),
            "severity": severity.as_ref().map(satd_severity_to_string),
            "critical_only": &critical_only,
            "include_tests": &include_tests,
            "strict": &strict,
            "evolution": &evolution,
            "days": &days,
            "metrics": &metrics,
            "output_path": output,
            "top_files": &top_files
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/satd".to_string(),
            body,
            Some(OutputFormat::Json),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_deep_context(
        project_path: &std::path::Path,
        output: &Option<std::path::PathBuf>,
        format: &crate::cli::DeepContextOutputFormat,
        full: bool,
        include: &[String],
        exclude: &[String],
        period_days: u32,
        dag_type: &Option<crate::cli::DeepContextDagType>,
        max_depth: &Option<usize>,
        include_patterns: &[String],
        exclude_patterns: &[String],
        cache_strategy: &Option<crate::cli::DeepContextCacheStrategy>,
        parallel: &Option<usize>,
        verbose: bool,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        // These two are `Option` because deep-context implements NEITHER: it
        // builds no DAG (all four `--dag-type` values produced one identical
        // report) and consults no cache. The CLI route refuses them rather than
        // accept a knob wired to nothing (#915, `reject_unimplemented_deep_
        // context_flags`), and this route must give the same answer — a flag
        // that errors on one protocol and is silently dropped on another is the
        // same defect in a new place. `None` means the user did not pass it.
        let mut unsupported = Vec::new();
        if dag_type.is_some() {
            unsupported.push("--dag-type");
        }
        if cache_strategy.is_some() {
            unsupported.push("--cache-strategy");
        }
        if !unsupported.is_empty() {
            return Err(ProtocolError::InvalidFormat(format!(
                "analyze deep-context does not implement {}; the flag(s) would be accepted and ignored. \
                 Use --include-pattern / --exclude-pattern to select files.",
                unsupported.join(", ")
            )));
        }
        let body = json!({
            "project_path": project_path.to_string_lossy(),
            "output_path": output,
            "format": deep_context_format_to_string(format),
            "full": &full,
            "include": include,
            "exclude": exclude,
            "period_days": &period_days,
            // Refused above when present, so these are always absent here. They
            // stay in the body as null rather than carrying a fabricated
            // default, which is what made the flags look implemented.
            "dag_type": dag_type.as_ref().map(deep_context_dag_type_to_string),
            "max_depth": max_depth,
            "include_patterns": include_patterns,
            "exclude_patterns": exclude_patterns,
            "cache_strategy": cache_strategy.as_ref().map(deep_context_cache_strategy_to_string),
            "parallel": parallel,
            "verbose": &verbose
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/deep-context".to_string(),
            body,
            Some(OutputFormat::Json),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_tdg(
        path: &std::path::Path,
        output: &Option<std::path::PathBuf>,
        format: &crate::cli::TdgOutputFormat,
        threshold: f64,
        critical_only: bool,
        top: usize,
        include_components: bool,
        verbose: bool,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let body = json!({
            "project_path": path.to_string_lossy(),
            "output_path": output,
            "format": tdg_format_to_string(format),
            "threshold": &threshold,
            "critical_only": &critical_only,
            "top": &top,
            "include_components": &include_components,
            "verbose": &verbose
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/tdg".to_string(),
            body,
            Some(OutputFormat::Json),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_provability(
        project_path: &std::path::Path,
        functions: &[String],
        analysis_depth: Option<usize>,
        format: &crate::cli::ProvabilityOutputFormat,
        high_confidence_only: bool,
        include_evidence: bool,
        output: &Option<std::path::PathBuf>,
        top_files: usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        // `--analysis-depth` bounds an iteration that does not exist:
        // `LightweightProvabilityAnalyzer` scores each function once from source
        // patterns, and `AbstractInterpreter::analyze_iteration` has no caller.
        // Depths 0, 1, 10, 50 and 1000 produced one identical report. Refused
        // here for the same reason the CLI refuses it (#915).
        if analysis_depth.is_some() {
            return Err(ProtocolError::InvalidFormat(
                "analyze provability does not implement --analysis-depth; there is no iteration \
                 to bound, so the flag would be accepted and ignored."
                    .to_string(),
            ));
        }
        let body = json!({
            "project_path": project_path.to_string_lossy(),
            "functions": if functions.is_empty() { None } else { Some(functions) },
            "analysis_depth": analysis_depth,
            "format": provability_format_to_string(format),
            "high_confidence_only": &high_confidence_only,
            "include_evidence": &include_evidence,
            "output_path": output,
            "top_files": &top_files
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/provability".to_string(),
            body,
            Some(OutputFormat::Json),
        ))
    }
}
