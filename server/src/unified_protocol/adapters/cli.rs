use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;
use axum::body::Body;
use axum::http::Method;
use serde_json::{json, Value};
use tracing::debug;

use crate::cli::{
    AnalyzeCommands, Commands, ComplexityOutputFormat, ContextFormat, DagType, OutputFormat,
};
use crate::models::churn::ChurnOutputFormat;
use crate::unified_protocol::{
    CliContext, Protocol, ProtocolAdapter, ProtocolError, UnifiedRequest, UnifiedResponse,
};

/// CLI adapter that converts command line arguments to unified requests
pub struct CliAdapter;

impl CliAdapter {
    pub fn new() -> Self {
        Self
    }

    fn decode_command(
        &self,
        command: &Commands,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        match command {
            Commands::Generate {
                category,
                template,
                params,
                output,
                create_dirs,
            } => Self::decode_generate(category, template, params, output, create_dirs),
            Commands::Scaffold {
                toolchain,
                templates,
                params,
                parallel,
            } => Self::decode_scaffold(toolchain, templates, params, *parallel),
            Commands::List {
                toolchain,
                category,
                format,
            } => Self::decode_list(toolchain, category, format),
            Commands::Search {
                query,
                toolchain,
                limit,
            } => Self::decode_search(query, toolchain, *limit),
            Commands::Validate { uri, params } => Self::decode_validate(uri, params),
            Commands::Context {
                toolchain,
                project_path,
                output,
                format,
                include_large_files: _,
                skip_expensive_metrics: _,
            } => Self::decode_context(toolchain.as_deref(), project_path, output, format),
            Commands::Analyze(analyze_cmd) => Self::decode_analyze_command(analyze_cmd),
            Commands::Demo {
                path,
                url,
                format,
                no_browser,
                port,
                cli,
                target_nodes,
                centrality_threshold,
                merge_threshold,
                ..
            } => Self::decode_demo(
                path,
                url,
                format,
                *no_browser,
                port,
                *cli,
                *target_nodes,
                *centrality_threshold,
                *merge_threshold,
            ),
            Commands::Serve { host, port, cors } => Self::decode_serve(host, *port, *cors),
            Commands::Diagnose(_) => {
                // Diagnose command is handled directly in the CLI, not through the unified protocol
                Err(ProtocolError::InvalidFormat(
                    "Diagnose command should be handled directly by CLI".to_string(),
                ))
            }
            Commands::QualityGate { .. } => {
                // QualityGate command is handled directly in the CLI, not through the unified protocol
                Err(ProtocolError::InvalidFormat(
                    "QualityGate command should be handled directly by CLI".to_string(),
                ))
            }
            Commands::Report { .. } => {
                // Report command is handled directly in the CLI, not through the unified protocol
                Err(ProtocolError::InvalidFormat(
                    "Report command should be handled directly by CLI".to_string(),
                ))
            }
            Commands::Enforce(_) => {
                // Enforce command is handled directly in the CLI, not through the unified protocol
                Err(ProtocolError::InvalidFormat(
                    "Enforce command should be handled directly by CLI".to_string(),
                ))
            }
            Commands::Refactor(_) => {
                // Refactor command is handled directly in the CLI, not through the unified protocol
                Err(ProtocolError::InvalidFormat(
                    "Refactor command should be handled directly by CLI".to_string(),
                ))
            }
        }
    }

    fn decode_generate(
        category: &str,
        template: &str,
        params: &[(String, Value)],
        output: &Option<std::path::PathBuf>,
        create_dirs: &bool,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let params_map: HashMap<String, Value> = params.iter().cloned().collect();
        let body = json!({
            "template_uri": format!("template://{}/{}", category, template),
            "parameters": params_map,
            "output_path": output,
            "create_dirs": create_dirs
        });
        Ok((Method::POST, "/api/v1/generate".to_string(), body, None))
    }

    fn decode_scaffold(
        toolchain: &str,
        templates: &[String],
        params: &[(String, Value)],
        parallel: usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let params_map: HashMap<String, Value> = params.iter().cloned().collect();
        let body = json!({
            "toolchain": toolchain,
            "templates": templates,
            "parameters": params_map,
            "parallel": &parallel
        });
        Ok((Method::POST, "/api/v1/scaffold".to_string(), body, None))
    }

    fn decode_list(
        toolchain: &Option<String>,
        category: &Option<String>,
        format: &OutputFormat,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let mut query_params = Vec::new();
        if let Some(tc) = toolchain {
            query_params.push(format!("toolchain={tc}"));
        }
        if let Some(cat) = category {
            query_params.push(format!("category={cat}"));
        }
        if !query_params.is_empty() {
            query_params.push(format!("format={format:?}").to_lowercase());
        }

        let query_string = if query_params.is_empty() {
            String::new()
        } else {
            format!("?{}", query_params.join("&"))
        };

        Ok((
            Method::GET,
            format!("/api/v1/templates{query_string}"),
            json!({}),
            Some(format.clone()),
        ))
    }

    fn decode_search(
        query: &str,
        toolchain: &Option<String>,
        limit: usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let body = json!({
            "query": query,
            "toolchain": toolchain,
            "limit": &limit
        });
        Ok((Method::POST, "/api/v1/search".to_string(), body, None))
    }

    fn decode_validate(
        uri: &str,
        params: &[(String, Value)],
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let params_map: HashMap<String, Value> = params.iter().cloned().collect();
        let body = json!({
            "template_uri": uri,
            "parameters": params_map
        });
        Ok((Method::POST, "/api/v1/validate".to_string(), body, None))
    }

    fn decode_context(
        toolchain: Option<&str>,
        project_path: &std::path::Path,
        output: &Option<std::path::PathBuf>,
        format: &ContextFormat,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let body = json!({
            "toolchain": toolchain,
            "project_path": project_path.to_string_lossy(),
            "output_path": output,
            "format": format_to_string(format)
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/context".to_string(),
            body,
            Some(OutputFormat::Json),
        ))
    }

    fn decode_analyze_command(
        analyze_cmd: &AnalyzeCommands,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        match analyze_cmd {
            AnalyzeCommands::Churn {
                project_path,
                days,
                format,
                output,
            } => Self::decode_analyze_churn(project_path, *days, format, output),
            AnalyzeCommands::Complexity {
                project_path,
                toolchain,
                format,
                output,
                max_cyclomatic,
                max_cognitive,
                include,
                watch,
                top_files,
            } => Self::decode_analyze_complexity(
                project_path,
                toolchain,
                format,
                output,
                max_cyclomatic,
                max_cognitive,
                include,
                *watch,
                *top_files,
            ),
            AnalyzeCommands::Dag {
                dag_type,
                project_path,
                output,
                max_depth,
                target_nodes,
                filter_external,
                show_complexity,
                include_duplicates,
                include_dead_code,
                enhanced,
            } => Self::decode_analyze_dag(
                dag_type,
                project_path,
                output,
                max_depth,
                target_nodes,
                *filter_external,
                *show_complexity,
                *include_duplicates,
                *include_dead_code,
                *enhanced,
            ),
            AnalyzeCommands::DeadCode {
                path,
                format,
                top_files,
                include_unreachable,
                min_dead_lines,
                include_tests,
                output,
            } => Self::decode_analyze_dead_code(
                path,
                format,
                top_files,
                *include_unreachable,
                *min_dead_lines,
                *include_tests,
                output,
            ),
            AnalyzeCommands::Satd {
                path,
                format,
                severity,
                critical_only,
                include_tests,
                evolution,
                days,
                metrics,
                output,
            } => Self::decode_analyze_satd(
                path,
                format,
                severity,
                *critical_only,
                *include_tests,
                *evolution,
                *days,
                *metrics,
                output,
            ),
            AnalyzeCommands::DeepContext {
                project_path,
                output,
                format,
                full,
                include,
                exclude,
                period_days,
                dag_type,
                max_depth,
                include_patterns,
                exclude_patterns,
                cache_strategy,
                parallel,
                verbose,
            } => Self::decode_analyze_deep_context(
                project_path,
                output,
                format,
                *full,
                include,
                exclude,
                *period_days,
                dag_type,
                max_depth,
                include_patterns,
                exclude_patterns,
                cache_strategy,
                parallel,
                *verbose,
            ),
            AnalyzeCommands::Tdg {
                path,
                threshold,
                top,
                format,
                include_components,
                output,
                critical_only,
                verbose,
            } => Self::decode_analyze_tdg(
                path,
                output,
                format,
                *threshold,
                *critical_only,
                *top,
                *include_components,
                *verbose,
            ),
            AnalyzeCommands::LintHotspot {
                project_path,
                format,
                max_density,
                min_confidence,
                enforce,
                dry_run,
                enforcement_metadata,
                output,
                perf,
                clippy_flags,
            } => {
                // Convert LintHotspot command to generic analyze method
                let params = json!({
                    "project_path": project_path,
                    "format": format,
                    "max_density": max_density,
                    "min_confidence": min_confidence,
                    "enforce": enforce,
                    "dry_run": dry_run,
                    "enforcement_metadata": enforcement_metadata,
                    "output": output,
                    "perf": perf,
                    "clippy_flags": clippy_flags,
                });
                Ok((
                    Method::POST,
                    "/api/v1/analyze/lint-hotspot".to_string(),
                    params,
                    None,
                ))
            }
            AnalyzeCommands::Makefile {
                path,
                rules,
                format,
                fix,
                gnu_version,
            } => {
                // Convert Makefile command to generic analyze method
                let params = json!({
                    "path": path,
                    "rules": rules,
                    "fix": fix,
                    "gnu_version": gnu_version,
                    "format": format,
                });
                Ok((
                    Method::POST,
                    "/api/v1/analyze/makefile".to_string(),
                    params,
                    None,
                ))
            }
            AnalyzeCommands::Provability {
                project_path,
                functions,
                analysis_depth,
                format,
                high_confidence_only,
                include_evidence,
                output,
            } => Self::decode_analyze_provability(
                project_path,
                functions,
                *analysis_depth,
                format,
                *high_confidence_only,
                *include_evidence,
                output,
            ),
            AnalyzeCommands::Duplicates {
                project_path,
                detection_type,
                threshold,
                min_lines,
                max_tokens,
                format,
                perf,
                include,
                exclude,
                output,
            } => {
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
                });
                Ok((
                    Method::POST,
                    "/api/v1/analyze/duplicates".to_string(),
                    params,
                    None,
                ))
            }
            AnalyzeCommands::DefectPrediction {
                project_path,
                confidence_threshold,
                min_lines,
                include_low_confidence,
                format,
                high_risk_only,
                include_recommendations,
                include,
                exclude,
                output,
                perf,
            } => {
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
                });
                Ok((
                    Method::POST,
                    "/api/v1/analyze/defect-prediction".to_string(),
                    params,
                    None,
                ))
            }
            AnalyzeCommands::Comprehensive {
                project_path,
                format,
                include_duplicates,
                include_dead_code,
                include_defects,
                include_complexity,
                include_tdg,
                confidence_threshold,
                min_lines,
                include,
                exclude,
                output,
                perf,
                executive_summary,
            } => {
                let params = json!({
                    "project_path": project_path,
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
                });
                Ok((
                    Method::POST,
                    "/api/v1/analyze/comprehensive".to_string(),
                    params,
                    None,
                ))
            }
            AnalyzeCommands::GraphMetrics {
                project_path,
                metrics,
                pagerank_seeds,
                damping_factor,
                max_iterations,
                convergence_threshold,
                format,
                include,
                exclude,
                output,
                export_graphml,
                perf,
                top_k,
                min_centrality,
            } => {
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
            AnalyzeCommands::NameSimilarity {
                project_path,
                query,
                top_k,
                phonetic,
                scope,
                threshold,
                format,
                include,
                exclude,
                output,
                perf,
                fuzzy,
                case_sensitive,
            } => {
                use crate::cli::SearchScope;
                let params = json!({
                    "project_path": project_path,
                    "query": query,
                    "top_k": top_k,
                    "phonetic": phonetic,
                    "scope": match scope {
                        SearchScope::Functions => "functions",
                        SearchScope::Types => "types",
                        SearchScope::Variables => "variables",
                        SearchScope::All => "all",
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
            AnalyzeCommands::ProofAnnotations {
                project_path,
                format,
                high_confidence_only,
                include_evidence,
                property_type,
                verification_method,
                output,
                perf,
                clear_cache,
            } => {
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
                });
                Ok((
                    Method::POST,
                    "/api/v1/analyze/proof-annotations".to_string(),
                    params,
                    None,
                ))
            }
            AnalyzeCommands::IncrementalCoverage {
                project_path,
                base_branch,
                target_branch,
                format,
                coverage_threshold,
                changed_files_only,
                detailed,
                output,
                perf,
                cache_dir,
                force_refresh,
            } => {
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
                });
                Ok((
                    Method::POST,
                    "/api/v1/analyze/incremental-coverage".to_string(),
                    params,
                    None,
                ))
            }
            AnalyzeCommands::SymbolTable {
                project_path,
                format,
                query,
                filter,
                include,
                exclude,
                show_unreferenced,
                show_references,
                output,
                perf,
            } => {
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
                });
                Ok((
                    Method::POST,
                    "/api/v1/analyze/symbol-table".to_string(),
                    params,
                    None,
                ))
            }
            AnalyzeCommands::BigO {
                project_path,
                format,
                confidence_threshold,
                analyze_space,
                include,
                exclude,
                output,
                perf,
                high_complexity_only,
            } => {
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
                });
                Ok((
                    Method::POST,
                    "/api/v1/analyze/big-o".to_string(),
                    params,
                    None,
                ))
            }
            AnalyzeCommands::AssemblyScript { .. } => {
                Ok((
                    Method::POST,
                    "/api/v1/analyze/assemblyscript".to_string(),
                    json!({}),
                    None,
                ))
            }
            AnalyzeCommands::WebAssembly { .. } => {
                Ok((
                    Method::POST,
                    "/api/v1/analyze/webassembly".to_string(),
                    json!({}),
                    None,
                ))
            }
        }
    }

    fn decode_analyze_churn(
        project_path: &std::path::Path,
        days: u32,
        format: &ChurnOutputFormat,
        output: &Option<std::path::PathBuf>,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let body = json!({
            "project_path": project_path.to_string_lossy(),
            "period_days": &days,
            "format": churn_format_to_string(format),
            "output_path": output
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
        evolution: bool,
        days: u32,
        metrics: bool,
        output: &Option<std::path::PathBuf>,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let body = json!({
            "project_path": path.to_string_lossy(),
            "format": satd_format_to_string(format),
            "severity": severity.as_ref().map(satd_severity_to_string),
            "critical_only": &critical_only,
            "include_tests": &include_tests,
            "evolution": &evolution,
            "days": &days,
            "metrics": &metrics,
            "output_path": output
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
        dag_type: &crate::cli::DeepContextDagType,
        max_depth: &Option<usize>,
        include_patterns: &[String],
        exclude_patterns: &[String],
        cache_strategy: &crate::cli::DeepContextCacheStrategy,
        parallel: &Option<usize>,
        verbose: bool,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let body = json!({
            "project_path": project_path.to_string_lossy(),
            "output_path": output,
            "format": deep_context_format_to_string(format),
            "full": &full,
            "include": include,
            "exclude": exclude,
            "period_days": &period_days,
            "dag_type": deep_context_dag_type_to_string(dag_type),
            "max_depth": max_depth,
            "include_patterns": include_patterns,
            "exclude_patterns": exclude_patterns,
            "cache_strategy": deep_context_cache_strategy_to_string(cache_strategy),
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

    fn decode_analyze_provability(
        project_path: &std::path::Path,
        functions: &[String],
        analysis_depth: usize,
        format: &crate::cli::ProvabilityOutputFormat,
        high_confidence_only: bool,
        include_evidence: bool,
        output: &Option<std::path::PathBuf>,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let body = json!({
            "project_path": project_path.to_string_lossy(),
            "functions": if functions.is_empty() { None } else { Some(functions) },
            "analysis_depth": &analysis_depth,
            "format": provability_format_to_string(format),
            "high_confidence_only": &high_confidence_only,
            "include_evidence": &include_evidence,
            "output_path": output
        });
        Ok((
            Method::POST,
            "/api/v1/analyze/provability".to_string(),
            body,
            Some(OutputFormat::Json),
        ))
    }

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

    fn format_to_extension_string(format: &OutputFormat) -> &'static str {
        match format {
            OutputFormat::Json => "json",
            OutputFormat::Table => "table",
            OutputFormat::Yaml => "yaml",
        }
    }
}

impl Default for CliAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProtocolAdapter for CliAdapter {
    type Input = CliInput;
    type Output = CliOutput;

    fn protocol(&self) -> Protocol {
        Protocol::Cli
    }

    async fn decode(&self, input: Self::Input) -> Result<UnifiedRequest, ProtocolError> {
        debug!("Decoding CLI input: {:?}", input.command_name);

        let (method, path, body, output_format) = self.decode_command(&input.command)?;

        let cli_context = CliContext {
            command: input.command_name.clone(),
            args: input.raw_args.clone(),
        };

        let mut unified_request = UnifiedRequest::new(method, path.to_string())
            .with_body(Body::from(serde_json::to_vec(&body)?))
            .with_header("content-type", "application/json")
            .with_extension("protocol", Protocol::Cli)
            .with_extension("cli_context", cli_context);

        // Add output format if specified
        if let Some(format) = output_format {
            let format_string = Self::format_to_extension_string(&format);
            unified_request = unified_request.with_extension("output_format", format_string);
        }

        debug!(
            command = %input.command_name,
            path = %path,
            "Decoded CLI request"
        );

        Ok(unified_request)
    }

    async fn encode(&self, response: UnifiedResponse) -> Result<Self::Output, ProtocolError> {
        debug!(status = %response.status, "Encoding CLI response");

        let body_bytes = axum::body::to_bytes(response.body, usize::MAX)
            .await
            .map_err(|e| {
                ProtocolError::EncodeError(format!("Failed to read response body: {e}"))
            })?;

        // For CLI, we typically want to output to stdout/stderr
        if response.status.is_success() {
            let content = String::from_utf8(body_bytes.to_vec()).map_err(|e| {
                ProtocolError::EncodeError(format!("Invalid UTF-8 in response: {e}"))
            })?;

            Ok(CliOutput::Success {
                content,
                exit_code: 0,
            })
        } else {
            // Try to parse error information
            let error_data: Result<Value, _> = serde_json::from_slice(&body_bytes);
            let error_message = match error_data {
                Ok(json) => json
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Unknown error")
                    .to_string(),
                Err(_) => String::from_utf8_lossy(&body_bytes).to_string(),
            };

            let exit_code = match response.status.as_u16() {
                400..=499 => 1, // Client errors
                500..=599 => 2, // Server errors
                _ => 1,
            };

            Ok(CliOutput::Error {
                message: error_message,
                exit_code,
            })
        }
    }
}

/// Input for CLI adapter
// Note: Debug omitted because Commands doesn't implement Debug in non-test builds
pub struct CliInput {
    pub command: Commands,
    pub command_name: String,
    pub raw_args: Vec<String>,
}

impl CliInput {
    pub fn new(command: Commands, command_name: String, raw_args: Vec<String>) -> Self {
        Self {
            command,
            command_name,
            raw_args,
        }
    }

    /// Create from the parsed CLI arguments
    fn get_analyze_command_name(analyze_cmd: &AnalyzeCommands) -> &'static str {
        match analyze_cmd {
            AnalyzeCommands::Churn { .. } => "analyze-churn",
            AnalyzeCommands::Complexity { .. } => "analyze-complexity",
            AnalyzeCommands::Dag { .. } => "analyze-dag",
            AnalyzeCommands::DeadCode { .. } => "analyze-dead-code",
            AnalyzeCommands::Satd { .. } => "analyze-satd",
            AnalyzeCommands::DeepContext { .. } => "analyze-deep-context",
            AnalyzeCommands::Tdg { .. } => "analyze-tdg",
            AnalyzeCommands::LintHotspot { .. } => "analyze-lint-hotspot",
            AnalyzeCommands::Makefile { .. } => "analyze-makefile",
            AnalyzeCommands::Provability { .. } => "analyze-provability",
            AnalyzeCommands::Duplicates { .. } => "analyze-duplicates",
            AnalyzeCommands::DefectPrediction { .. } => "analyze-defect-prediction",
            AnalyzeCommands::Comprehensive { .. } => "analyze-comprehensive",
            AnalyzeCommands::GraphMetrics { .. } => "analyze-graph-metrics",
            AnalyzeCommands::NameSimilarity { .. } => "analyze-name-similarity",
            AnalyzeCommands::ProofAnnotations { .. } => "analyze-proof-annotations",
            AnalyzeCommands::IncrementalCoverage { .. } => "analyze-incremental-coverage",
            AnalyzeCommands::SymbolTable { .. } => "analyze-symbol-table",
            AnalyzeCommands::BigO { .. } => "analyze-big-o",
            AnalyzeCommands::AssemblyScript { .. } => "analyze-assemblyscript",
            AnalyzeCommands::WebAssembly { .. } => "analyze-webassembly",
        }
    }

    pub fn from_commands(command: Commands) -> Self {
        let command_name = match &command {
            Commands::Generate { .. } => "generate",
            Commands::Scaffold { .. } => "scaffold",
            Commands::List { .. } => "list",
            Commands::Search { .. } => "search",
            Commands::Validate { .. } => "validate",
            Commands::Context { .. } => "context",
            Commands::Analyze(analyze_cmd) => Self::get_analyze_command_name(analyze_cmd),
            Commands::Demo { .. } => "demo",
            Commands::Serve { .. } => "serve",
            Commands::Diagnose(_) => "diagnose",
            Commands::QualityGate { .. } => "quality-gate",
            Commands::Report { .. } => "report",
            Commands::Enforce(_) => "enforce",
            Commands::Refactor(_) => "refactor",
        }
        .to_string();

        Self {
            command,
            command_name,
            raw_args: std::env::args().collect(),
        }
    }
}

/// Output for CLI adapter
#[derive(Debug)]
pub enum CliOutput {
    Success { content: String, exit_code: i32 },
    Error { message: String, exit_code: i32 },
}

impl CliOutput {
    /// Write the output to stdout/stderr and exit with appropriate code
    pub fn write_and_exit(self) -> ! {
        match self {
            CliOutput::Success { content, exit_code } => {
                print!("{content}");
                std::process::exit(exit_code);
            }
            CliOutput::Error { message, exit_code } => {
                eprintln!("Error: {message}");
                std::process::exit(exit_code);
            }
        }
    }

    /// Get the exit code without exiting
    pub fn exit_code(&self) -> i32 {
        match self {
            CliOutput::Success { exit_code, .. } => *exit_code,
            CliOutput::Error { exit_code, .. } => *exit_code,
        }
    }

    /// Get the content/message
    pub fn content(&self) -> &str {
        match self {
            CliOutput::Success { content, .. } => content,
            CliOutput::Error { message, .. } => message,
        }
    }
}

// Helper functions for format conversion

fn format_to_string(format: &ContextFormat) -> String {
    match format {
        ContextFormat::Markdown => "markdown".to_string(),
        ContextFormat::Json => "json".to_string(),
        ContextFormat::Sarif => "sarif".to_string(),
        ContextFormat::LlmOptimized => "llm-optimized".to_string(),
    }
}

fn churn_format_to_string(format: &ChurnOutputFormat) -> String {
    match format {
        ChurnOutputFormat::Summary => "summary".to_string(),
        ChurnOutputFormat::Markdown => "markdown".to_string(),
        ChurnOutputFormat::Json => "json".to_string(),
        ChurnOutputFormat::Csv => "csv".to_string(),
    }
}

fn complexity_format_to_string(format: &ComplexityOutputFormat) -> String {
    match format {
        ComplexityOutputFormat::Summary => "summary".to_string(),
        ComplexityOutputFormat::Full => "full".to_string(),
        ComplexityOutputFormat::Json => "json".to_string(),
        ComplexityOutputFormat::Sarif => "sarif".to_string(),
    }
}

fn dag_type_to_string(dag_type: &DagType) -> String {
    match dag_type {
        DagType::CallGraph => "call-graph".to_string(),
        DagType::ImportGraph => "import-graph".to_string(),
        DagType::Inheritance => "inheritance".to_string(),
        DagType::FullDependency => "full-dependency".to_string(),
    }
}

fn dead_code_format_to_string(format: &crate::cli::DeadCodeOutputFormat) -> String {
    match format {
        crate::cli::DeadCodeOutputFormat::Summary => "summary".to_string(),
        crate::cli::DeadCodeOutputFormat::Json => "json".to_string(),
        crate::cli::DeadCodeOutputFormat::Sarif => "sarif".to_string(),
        crate::cli::DeadCodeOutputFormat::Markdown => "markdown".to_string(),
    }
}

fn satd_format_to_string(format: &crate::cli::SatdOutputFormat) -> String {
    match format {
        crate::cli::SatdOutputFormat::Summary => "summary".to_string(),
        crate::cli::SatdOutputFormat::Json => "json".to_string(),
        crate::cli::SatdOutputFormat::Sarif => "sarif".to_string(),
        crate::cli::SatdOutputFormat::Markdown => "markdown".to_string(),
    }
}

fn satd_severity_to_string(severity: &crate::cli::SatdSeverity) -> String {
    match severity {
        crate::cli::SatdSeverity::Critical => "critical".to_string(),
        crate::cli::SatdSeverity::High => "high".to_string(),
        crate::cli::SatdSeverity::Medium => "medium".to_string(),
        crate::cli::SatdSeverity::Low => "low".to_string(),
    }
}

fn graph_metric_type_to_string(metric: &crate::cli::GraphMetricType) -> String {
    match metric {
        crate::cli::GraphMetricType::All => "all".to_string(),
        crate::cli::GraphMetricType::Centrality => "centrality".to_string(),
        crate::cli::GraphMetricType::Betweenness => "betweenness".to_string(),
        crate::cli::GraphMetricType::Closeness => "closeness".to_string(),
        crate::cli::GraphMetricType::PageRank => "pagerank".to_string(),
        crate::cli::GraphMetricType::Clustering => "clustering".to_string(),
        crate::cli::GraphMetricType::Components => "components".to_string(),
    }
}

fn graph_metrics_format_to_string(format: &crate::cli::GraphMetricsOutputFormat) -> String {
    match format {
        crate::cli::GraphMetricsOutputFormat::Summary => "summary".to_string(),
        crate::cli::GraphMetricsOutputFormat::Detailed => "detailed".to_string(),
        crate::cli::GraphMetricsOutputFormat::Human => "human".to_string(),
        crate::cli::GraphMetricsOutputFormat::Json => "json".to_string(),
        crate::cli::GraphMetricsOutputFormat::Csv => "csv".to_string(),
        crate::cli::GraphMetricsOutputFormat::GraphML => "graphml".to_string(),
        crate::cli::GraphMetricsOutputFormat::Markdown => "markdown".to_string(),
    }
}

fn name_similarity_format_to_string(format: &crate::cli::NameSimilarityOutputFormat) -> String {
    match format {
        crate::cli::NameSimilarityOutputFormat::Summary => "summary".to_string(),
        crate::cli::NameSimilarityOutputFormat::Detailed => "detailed".to_string(),
        crate::cli::NameSimilarityOutputFormat::Human => "human".to_string(),
        crate::cli::NameSimilarityOutputFormat::Json => "json".to_string(),
        crate::cli::NameSimilarityOutputFormat::Csv => "csv".to_string(),
        crate::cli::NameSimilarityOutputFormat::Markdown => "markdown".to_string(),
    }
}

fn property_type_filter_to_string(filter: &crate::cli::PropertyTypeFilter) -> String {
    match filter {
        crate::cli::PropertyTypeFilter::All => "all".to_string(),
        crate::cli::PropertyTypeFilter::MemorySafety => "memory-safety".to_string(),
        crate::cli::PropertyTypeFilter::ThreadSafety => "thread-safety".to_string(),
        crate::cli::PropertyTypeFilter::DataRaceFreeze => "data-race-freeze".to_string(),
        crate::cli::PropertyTypeFilter::Termination => "termination".to_string(),
        crate::cli::PropertyTypeFilter::FunctionalCorrectness => {
            "functional-correctness".to_string()
        }
        crate::cli::PropertyTypeFilter::ResourceBounds => "resource-bounds".to_string(),
    }
}

fn verification_method_filter_to_string(method: &crate::cli::VerificationMethodFilter) -> String {
    match method {
        crate::cli::VerificationMethodFilter::All => "all".to_string(),
        crate::cli::VerificationMethodFilter::FormalProof => "formal-proof".to_string(),
        crate::cli::VerificationMethodFilter::ModelChecking => "model-checking".to_string(),
        crate::cli::VerificationMethodFilter::StaticAnalysis => "static-analysis".to_string(),
        crate::cli::VerificationMethodFilter::AbstractInterpretation => {
            "abstract-interpretation".to_string()
        }
        crate::cli::VerificationMethodFilter::BorrowChecker => "borrow-checker".to_string(),
    }
}

fn proof_annotation_format_to_string(format: &crate::cli::ProofAnnotationOutputFormat) -> String {
    match format {
        crate::cli::ProofAnnotationOutputFormat::Summary => "summary".to_string(),
        crate::cli::ProofAnnotationOutputFormat::Full => "full".to_string(),
        crate::cli::ProofAnnotationOutputFormat::Json => "json".to_string(),
        crate::cli::ProofAnnotationOutputFormat::Markdown => "markdown".to_string(),
        crate::cli::ProofAnnotationOutputFormat::Sarif => "sarif".to_string(),
    }
}

fn incremental_coverage_format_to_string(
    format: &crate::cli::IncrementalCoverageOutputFormat,
) -> String {
    match format {
        crate::cli::IncrementalCoverageOutputFormat::Summary => "summary".to_string(),
        crate::cli::IncrementalCoverageOutputFormat::Detailed => "detailed".to_string(),
        crate::cli::IncrementalCoverageOutputFormat::Json => "json".to_string(),
        crate::cli::IncrementalCoverageOutputFormat::Markdown => "markdown".to_string(),
        crate::cli::IncrementalCoverageOutputFormat::Lcov => "lcov".to_string(),
        crate::cli::IncrementalCoverageOutputFormat::Delta => "delta".to_string(),
        crate::cli::IncrementalCoverageOutputFormat::Sarif => "sarif".to_string(),
    }
}

fn symbol_type_filter_to_string(filter: &crate::cli::SymbolTypeFilter) -> String {
    match filter {
        crate::cli::SymbolTypeFilter::All => "all".to_string(),
        crate::cli::SymbolTypeFilter::Functions => "functions".to_string(),
        crate::cli::SymbolTypeFilter::Classes => "classes".to_string(),
        crate::cli::SymbolTypeFilter::Types => "types".to_string(),
        crate::cli::SymbolTypeFilter::Variables => "variables".to_string(),
        crate::cli::SymbolTypeFilter::Modules => "modules".to_string(),
    }
}

fn symbol_table_format_to_string(format: &crate::cli::SymbolTableOutputFormat) -> String {
    match format {
        crate::cli::SymbolTableOutputFormat::Summary => "summary".to_string(),
        crate::cli::SymbolTableOutputFormat::Detailed => "detailed".to_string(),
        crate::cli::SymbolTableOutputFormat::Human => "human".to_string(),
        crate::cli::SymbolTableOutputFormat::Json => "json".to_string(),
        crate::cli::SymbolTableOutputFormat::Csv => "csv".to_string(),
    }
}

fn big_o_format_to_string(format: &crate::cli::BigOOutputFormat) -> String {
    match format {
        crate::cli::BigOOutputFormat::Summary => "summary".to_string(),
        crate::cli::BigOOutputFormat::Json => "json".to_string(),
        crate::cli::BigOOutputFormat::Markdown => "markdown".to_string(),
        crate::cli::BigOOutputFormat::Detailed => "detailed".to_string(),
    }
}

/// CLI runner that integrates with the unified protocol system
pub struct CliRunner {
    adapter: CliAdapter,
}

impl CliRunner {
    pub fn new() -> Self {
        Self {
            adapter: CliAdapter::new(),
        }
    }

    /// Run a CLI command through the unified protocol system
    pub async fn run_command<H>(
        &self,
        command: Commands,
        handler: H,
    ) -> Result<CliOutput, ProtocolError>
    where
        H: Fn(
            UnifiedRequest,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<UnifiedResponse, ProtocolError>> + Send>,
        >,
    {
        let input = CliInput::from_commands(command);
        let unified_request = self.adapter.decode(input).await?;
        let unified_response = handler(unified_request).await?;
        self.adapter.encode(unified_response).await
    }
}

impl Default for CliRunner {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for deep context format conversion

fn deep_context_format_to_string(format: &crate::cli::DeepContextOutputFormat) -> String {
    match format {
        crate::cli::DeepContextOutputFormat::Markdown => "markdown".to_string(),
        crate::cli::DeepContextOutputFormat::Json => "json".to_string(),
        crate::cli::DeepContextOutputFormat::Sarif => "sarif".to_string(),
    }
}

fn deep_context_dag_type_to_string(dag_type: &crate::cli::DeepContextDagType) -> String {
    match dag_type {
        crate::cli::DeepContextDagType::CallGraph => "call-graph".to_string(),
        crate::cli::DeepContextDagType::ImportGraph => "import-graph".to_string(),
        crate::cli::DeepContextDagType::Inheritance => "inheritance".to_string(),
        crate::cli::DeepContextDagType::FullDependency => "full-dependency".to_string(),
    }
}

fn deep_context_cache_strategy_to_string(
    strategy: &crate::cli::DeepContextCacheStrategy,
) -> String {
    match strategy {
        crate::cli::DeepContextCacheStrategy::Normal => "normal".to_string(),
        crate::cli::DeepContextCacheStrategy::ForceRefresh => "force-refresh".to_string(),
        crate::cli::DeepContextCacheStrategy::Offline => "offline".to_string(),
    }
}

fn tdg_format_to_string(format: &crate::cli::TdgOutputFormat) -> String {
    match format {
        crate::cli::TdgOutputFormat::Table => "table".to_string(),
        crate::cli::TdgOutputFormat::Json => "json".to_string(),
        crate::cli::TdgOutputFormat::Markdown => "markdown".to_string(),
        crate::cli::TdgOutputFormat::Sarif => "sarif".to_string(),
    }
}

fn provability_format_to_string(format: &crate::cli::ProvabilityOutputFormat) -> String {
    match format {
        crate::cli::ProvabilityOutputFormat::Summary => "summary".to_string(),
        crate::cli::ProvabilityOutputFormat::Full => "full".to_string(),
        crate::cli::ProvabilityOutputFormat::Json => "json".to_string(),
        crate::cli::ProvabilityOutputFormat::Sarif => "sarif".to_string(),
        crate::cli::ProvabilityOutputFormat::Markdown => "markdown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{AnalyzeCommands, Commands, ComplexityOutputFormat, DagType, OutputFormat};
    use crate::models::churn::ChurnOutputFormat;
    use serde_json::{json, Value};
    use std::path::PathBuf;

    #[test]
    fn test_cli_input_creation() {
        let params = vec![
            (
                "project_name".to_string(),
                Value::String("test".to_string()),
            ),
            ("version".to_string(), Value::String("1.0.0".to_string())),
        ];

        let command = Commands::Generate {
            category: "makefile".to_string(),
            template: "rust/cli".to_string(),
            params,
            output: Some(PathBuf::from("Makefile")),
            create_dirs: true,
        };

        let input = CliInput::from_commands(command);
        assert_eq!(input.command_name, "generate");
    }

    #[tokio::test]
    async fn test_cli_adapter_decode_generate() {
        let adapter = CliAdapter::new();
        let params = vec![(
            "project_name".to_string(),
            Value::String("test".to_string()),
        )];

        let command = Commands::Generate {
            category: "makefile".to_string(),
            template: "rust/cli".to_string(),
            params,
            output: None,
            create_dirs: false,
        };

        let input = CliInput::from_commands(command);
        let unified_request = adapter.decode(input).await.unwrap();

        assert_eq!(unified_request.method, Method::POST);
        assert_eq!(unified_request.path, "/api/v1/generate");
        assert_eq!(
            unified_request.get_extension::<Protocol>("protocol"),
            Some(Protocol::Cli)
        );

        let cli_context: CliContext = unified_request.get_extension("cli_context").unwrap();
        assert_eq!(cli_context.command, "generate");
    }

    #[tokio::test]
    async fn test_cli_adapter_decode_list() {
        let adapter = CliAdapter::new();
        let command = Commands::List {
            toolchain: Some("rust".to_string()),
            category: None,
            format: OutputFormat::Json,
        };

        let input = CliInput::from_commands(command);
        let unified_request = adapter.decode(input).await.unwrap();

        assert_eq!(unified_request.method, Method::GET);
        assert!(unified_request.path.starts_with("/api/v1/templates"));
        assert!(unified_request.path.contains("toolchain=rust"));
    }

    #[tokio::test]
    async fn test_cli_adapter_decode_analyze_complexity() {
        let adapter = CliAdapter::new();
        let command = Commands::Analyze(AnalyzeCommands::Complexity {
            project_path: PathBuf::from("."),
            toolchain: Some("rust".to_string()),
            format: ComplexityOutputFormat::Json,
            output: None,
            max_cyclomatic: Some(10),
            max_cognitive: Some(15),
            include: vec!["**/*.rs".to_string()],
            watch: false,
            top_files: 0,
        });

        let input = CliInput::from_commands(command);
        let unified_request = adapter.decode(input).await.unwrap();

        assert_eq!(unified_request.method, Method::POST);
        assert_eq!(unified_request.path, "/api/v1/analyze/complexity");

        // Verify body contains expected fields
        let body_bytes = axum::body::to_bytes(unified_request.body, usize::MAX)
            .await
            .unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["toolchain"], "rust");
        assert_eq!(body_json["max_cyclomatic"], 10);
        assert_eq!(body_json["max_cognitive"], 15);
    }

    #[tokio::test]
    async fn test_cli_adapter_encode_success() {
        let adapter = CliAdapter::new();
        let response = UnifiedResponse::ok()
            .with_json(&json!({"message": "success"}))
            .unwrap();

        let output = adapter.encode(response).await.unwrap();
        match output {
            CliOutput::Success { content, exit_code } => {
                assert_eq!(exit_code, 0);
                assert!(content.contains("success"));
            }
            _ => panic!("Expected success output"),
        }
    }

    #[tokio::test]
    async fn test_cli_adapter_encode_error() {
        let adapter = CliAdapter::new();
        let response = UnifiedResponse::new(axum::http::StatusCode::BAD_REQUEST)
            .with_json(&json!({"error": "Invalid request"}))
            .unwrap();

        let output = adapter.encode(response).await.unwrap();
        match output {
            CliOutput::Error { message, exit_code } => {
                assert_eq!(exit_code, 1);
                assert!(message.contains("Invalid request"));
            }
            _ => panic!("Expected error output"),
        }
    }

    #[test]
    fn test_format_conversions() {
        assert_eq!(format_to_string(&ContextFormat::Markdown), "markdown");
        assert_eq!(format_to_string(&ContextFormat::Json), "json");

        assert_eq!(
            churn_format_to_string(&ChurnOutputFormat::Summary),
            "summary"
        );
        assert_eq!(churn_format_to_string(&ChurnOutputFormat::Json), "json");

        assert_eq!(
            complexity_format_to_string(&ComplexityOutputFormat::Sarif),
            "sarif"
        );

        assert_eq!(dag_type_to_string(&DagType::CallGraph), "call-graph");
        assert_eq!(
            dag_type_to_string(&DagType::FullDependency),
            "full-dependency"
        );
    }

    #[test]
    fn test_cli_output_methods() {
        let success = CliOutput::Success {
            content: "test content".to_string(),
            exit_code: 0,
        };
        assert_eq!(success.exit_code(), 0);
        assert_eq!(success.content(), "test content");

        let error = CliOutput::Error {
            message: "test error".to_string(),
            exit_code: 1,
        };
        assert_eq!(error.exit_code(), 1);
        assert_eq!(error.content(), "test error");
    }
}
