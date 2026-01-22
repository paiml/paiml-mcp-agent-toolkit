use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;
use axum::body::Body;
use axum::http::Method;
use serde_json::{json, Value};
use tracing::debug;

use crate::cli::commands::{QddCommands, ScaffoldCommands};
use crate::cli::{
    AnalyzeCommands, Commands, ComplexityOutputFormat, ContextFormat, DagType, OutputFormat,
};
use crate::models::churn::ChurnOutputFormat;
use crate::unified_protocol::{
    CliContext, Protocol, ProtocolAdapter, ProtocolError, UnifiedRequest, UnifiedResponse,
};

// Format helpers extracted to cli_helpers.rs for file health (CB-040)
use super::cli_helpers::{
    big_o_format_to_string, churn_format_to_string, complexity_format_to_string,
    dag_type_to_string, dead_code_format_to_string, deep_context_cache_strategy_to_string,
    deep_context_dag_type_to_string, deep_context_format_to_string, format_to_string,
    graph_metric_type_to_string, graph_metrics_format_to_string,
    incremental_coverage_format_to_string, name_similarity_format_to_string,
    proof_annotation_format_to_string, property_type_filter_to_string,
    provability_format_to_string, satd_format_to_string, satd_severity_to_string,
    symbol_table_format_to_string, symbol_type_filter_to_string, tdg_format_to_string,
    verification_method_filter_to_string,
};

/// CLI adapter that converts command line arguments to unified requests
pub struct CliAdapter;

impl CliAdapter {
    #[must_use]
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
            Commands::Scaffold { command } => match command {
                ScaffoldCommands::Project {
                    toolchain,
                    templates,
                    params,
                    parallel,
                } => Self::decode_scaffold(toolchain, templates, params, *parallel),
                _ => Err(ProtocolError::UnsupportedProtocol(
                    "Agent scaffolding not supported via unified protocol".to_string(),
                )),
            },
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
                language: _,
                languages: _,
            } => Self::decode_context(toolchain.as_deref(), project_path, output, format),
            Commands::Analyze(analyze_cmd) => Self::decode_analyze_command(analyze_cmd),
            Commands::Qdd(_) => Self::cli_only_command_error(),
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
            Commands::Serve {
                host,
                port,
                cors,
                transport: _,
            } => Self::decode_serve(host, *port, *cors),
            Commands::Diagnose(_)
            | Commands::QualityGate { .. }
            | Commands::QualityGates { .. } // TICKET-PMAT-5023
            | Commands::Maintain { .. } // TICKET-PMAT-5032
            | Commands::Hooks(_) // TICKET-PMAT-5034
            | Commands::Report { .. }
            | Commands::RepoScore { .. } // Sprint 48: Repository health scoring (CLI-only)
            | Commands::RustProjectScore { .. } // Sprint 3: Rust Project Score v1.1 (CLI-only)
            | Commands::BrickScore { .. } // PMAT-446: ComputeBrick profiling score (CLI-only)
            | Commands::PopperScore { .. } // Popper Falsifiability Score v1.1 (CLI-only)
            | Commands::DemoScore { .. } // GH-109/112: Demo Quality scoring (CLI-only)
            | Commands::Enforce(_)
            | Commands::Refactor(_)
            | Commands::Roadmap(_)
            | Commands::Test { .. }
            | Commands::Memory { .. }
            | Commands::Cache { .. }
            | Commands::Telemetry { .. }
            | Commands::Config { .. }
            | Commands::Agent { .. }
            | Commands::Tdg { .. }
            | Commands::ValidateDocs(_)
            | Commands::ValidateReadme(_) // Sprint 38: Hallucination detection
            | Commands::RedTeam(_) // Red Team Mode: Commit hallucination detection
            | Commands::Org(_) // Phase 4: Organizational intelligence (CLI-only)
            | Commands::Prompt(_) // Phase 4: Prompt generation (CLI-only)
            | Commands::Embed(_) // PMAT-SEARCH-011
            | Commands::Semantic(_) // PMAT-SEARCH-011
            | Commands::Debug { .. } // Sprint 74: Time-travel debugging (CLI-only)
            | Commands::Work { .. } // Issue #75: Unified GitHub/YAML workflow (CLI-only)
            | Commands::Comply { .. } // GH-96: PMAT compliance and migration system (CLI-only)
            | Commands::ProjectDiag { .. } // Project diagnostics - lltop Tab 8 (CLI-only)
            | Commands::TestDiscovery { .. } // GH-98: Systematic test discovery and fixing (CLI-only)
            | Commands::DebugFiveWhys { .. } // Five Whys root cause analysis (CLI-only)
            | Commands::Localize { .. } // GH-103: Tarantula fault localization (CLI-only)
            | Commands::Oracle { .. } // PMAT Oracle - PDCA loop (CLI-only)
            | Commands::ShowMetrics { .. } // Phase 3.1: O(1) Quality Gates CLI (CLI-only)
            | Commands::PredictQuality { .. } // Phase 4.1: Predictive Quality Gates CLI (CLI-only)
            | Commands::RecordMetric { .. } // Phase 3.4: O(1) Quality Gates CI/CD (CLI-only)
            | Commands::QaWork { .. } // GH-102: Toyota Way QA validation (CLI-only)
            | Commands::PerfectionScore { .. } // master-plan-pmat-work-system.md: 200-point score (CLI-only)
            | Commands::Spec { .. } // master-plan-pmat-work-system.md: Spec management (CLI-only)
            | Commands::CudaTdg { .. } // CUDA-SIMD TDG: 100-point Popper falsification (CLI-only)
            | Commands::DepsAudit { .. } // Dependency audit for Sovereign AI stack migration (CLI-only)
            => Self::cli_only_command_error(),

            #[cfg(feature = "mutation-testing")]
            Commands::Mutate(_) => Self::cli_only_command_error(),
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

    /// Toyota Way Extract Method: Focused analyze command dispatch with reduced complexity
    /// Original complexity: 24 -> Target: <10 through categorized dispatch
    fn decode_analyze_command(
        analyze_cmd: &AnalyzeCommands,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        // Toyota Way Extract Method: Determine command category and dispatch accordingly
        let command_category = Self::get_analyze_command_category(analyze_cmd);

        match command_category {
            AnalyzeCommandCategory::Basic => Self::dispatch_basic_analysis(analyze_cmd),
            AnalyzeCommandCategory::Advanced => Self::dispatch_advanced_analysis(analyze_cmd),
            AnalyzeCommandCategory::Structural => Self::dispatch_structural_analysis(analyze_cmd),
            AnalyzeCommandCategory::Specialized => Self::dispatch_specialized_analysis(analyze_cmd),
        }
    }

    /// Toyota Way Extract Method: Basic analysis commands dispatch
    /// Handles core metrics: churn, complexity, dead code, SATD, TDG, lint hotspots
    fn dispatch_basic_analysis(
        analyze_cmd: &AnalyzeCommands,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        match analyze_cmd {
            AnalyzeCommands::Churn {
                project_path,
                days,
                format,
                output,
                top_files,
                include: _,
                exclude: _,
            } => Self::decode_analyze_churn(project_path, *days, format, output, *top_files),
            AnalyzeCommands::Complexity {
                path,
                project_path,
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
                fail_on_violation: _,
                timeout: _,
                ml: _, // GH-97: ML flag
            } => Self::decode_analyze_complexity_with_migration(
                path,
                project_path,
                file,
                files,
                toolchain,
                format,
                output,
                max_cyclomatic,
                max_cognitive,
                include,
                *watch,
                *top_files,
            ),
            AnalyzeCommands::DeadCode {
                path,
                format,
                top_files,
                include_unreachable,
                min_dead_lines,
                include_tests,
                output,
                fail_on_violation: _,
                max_percentage: _,
                timeout: _,
                include: _,
                exclude: _,
                max_depth: _,
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
                strict,
                evolution,
                days,
                metrics,
                output,
                top_files,
                fail_on_violation: _,
                timeout: _,
                include: _,
                exclude: _,
            } => Self::decode_analyze_satd(
                path,
                format,
                severity,
                *critical_only,
                *include_tests,
                *strict,
                *evolution,
                *days,
                *metrics,
                output,
                *top_files,
            ),
            AnalyzeCommands::Tdg {
                path,
                threshold,
                top_files,
                format,
                include_components,
                output,
                critical_only,
                verbose,
                ml: _, // GH-97: ML flag
            } => Self::decode_analyze_tdg(
                path,
                output,
                format,
                *threshold,
                *critical_only,
                *top_files,
                *include_components,
                *verbose,
            ),
            AnalyzeCommands::LintHotspot {
                project_path,
                file,
                format,
                max_density,
                min_confidence,
                enforce,
                dry_run,
                enforcement_metadata,
                output,
                perf,
                clippy_flags,
                top_files,
                include: _,
                exclude: _,
            } => Self::decode_analyze_lint_hotspot(
                project_path,
                file,
                format,
                max_density,
                min_confidence,
                enforce,
                dry_run,
                enforcement_metadata,
                output,
                perf,
                clippy_flags,
                top_files,
            ),
            _ => Err(ProtocolError::UnsupportedProtocol(
                "Command not supported in basic analysis dispatch".to_string(),
            )),
        }
    }

    /// Toyota Way Extract Method: Advanced analysis commands dispatch
    /// Handles comprehensive analysis: deep context, comprehensive, defect prediction, duplicates, `BigO`
    fn dispatch_advanced_analysis(
        analyze_cmd: &AnalyzeCommands,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        match analyze_cmd {
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
                top_files: _,
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
            AnalyzeCommands::Comprehensive {
                project_path,
                file,
                files,
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
                top_files,
            } => Self::decode_analyze_comprehensive(
                project_path,
                file,
                files,
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
                top_files,
            ),
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
                top_files,
            } => Self::decode_analyze_defect_prediction(
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
                top_files,
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
                top_files,
            } => Self::decode_analyze_duplicates(
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
                top_files,
            ),
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
                top_files,
            } => Self::decode_analyze_big_o(
                project_path,
                format,
                confidence_threshold,
                analyze_space,
                include,
                exclude,
                output,
                perf,
                high_complexity_only,
                top_files,
            ),
            _ => Err(ProtocolError::UnsupportedProtocol(
                "Command not supported in advanced analysis dispatch".to_string(),
            )),
        }
    }

    /// Toyota Way Extract Method: Structural analysis commands dispatch
    /// Handles graph and structural analysis: DAG, graph metrics, symbol table, name similarity
    fn dispatch_structural_analysis(
        analyze_cmd: &AnalyzeCommands,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        match analyze_cmd {
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
            } => Self::decode_analyze_graph_metrics(
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
            ),
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
                top_files,
            } => Self::decode_analyze_symbol_table(
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
                top_files,
            ),
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
            } => Self::decode_analyze_name_similarity(
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
            ),
            _ => Err(ProtocolError::UnsupportedProtocol(
                "Command not supported in structural analysis dispatch".to_string(),
            )),
        }
    }

    /// Toyota Way Extract Method: Specialized analysis commands dispatch
    /// Handles specialized analysis: makefile, provability, proof annotations, coverage, WebAssembly
    fn dispatch_specialized_analysis(
        analyze_cmd: &AnalyzeCommands,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        match analyze_cmd {
            AnalyzeCommands::Makefile {
                path,
                rules,
                format,
                fix,
                gnu_version,
                top_files,
            } => Self::decode_analyze_makefile(path, rules, format, fix, gnu_version, top_files),
            AnalyzeCommands::Provability {
                project_path,
                functions,
                analysis_depth,
                format,
                high_confidence_only,
                include_evidence,
                output,
                top_files,
            } => Self::decode_analyze_provability(
                project_path,
                functions,
                *analysis_depth,
                format,
                *high_confidence_only,
                *include_evidence,
                output,
                *top_files,
            ),
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
                top_files,
            } => Self::decode_analyze_proof_annotations(
                project_path,
                format,
                high_confidence_only,
                include_evidence,
                property_type,
                verification_method,
                output,
                perf,
                clear_cache,
                top_files,
            ),
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
                top_files,
            } => Self::decode_analyze_incremental_coverage(
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
                top_files,
            ),
            AnalyzeCommands::AssemblyScript { top_files: _, .. } => {
                Self::decode_analyze_assemblyscript()
            }
            AnalyzeCommands::WebAssembly { top_files: _, .. } => Self::decode_analyze_webassembly(),
            _ => Err(ProtocolError::UnsupportedProtocol(
                "Command not supported in specialized analysis dispatch".to_string(),
            )),
        }
    }

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

    #[allow(clippy::too_many_arguments)]
    fn decode_analyze_provability(
        project_path: &std::path::Path,
        functions: &[String],
        analysis_depth: usize,
        format: &crate::cli::ProvabilityOutputFormat,
        high_confidence_only: bool,
        include_evidence: bool,
        output: &Option<std::path::PathBuf>,
        top_files: usize,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        let body = json!({
            "project_path": project_path.to_string_lossy(),
            "functions": if functions.is_empty() { None } else { Some(functions) },
            "analysis_depth": &analysis_depth,
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

        let mut unified_request = UnifiedRequest::new(method, path.clone())
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
    #[must_use]
    pub fn new(command: Commands, command_name: String, raw_args: Vec<String>) -> Self {
        Self {
            command,
            command_name,
            raw_args,
        }
    }

    /// Create from the parsed CLI arguments
    fn get_analyze_command_name(analyze_cmd: &AnalyzeCommands) -> &'static str {
        // Toyota Way Extract Method: Use categorized dispatch for analyze command names
        let category = CliAdapter::get_analyze_command_category(analyze_cmd);

        match category {
            AnalyzeCommandCategory::Basic => Self::get_basic_analyze_command_name(analyze_cmd),
            AnalyzeCommandCategory::Advanced => {
                Self::get_advanced_analyze_command_name(analyze_cmd)
            }
            AnalyzeCommandCategory::Structural => {
                Self::get_structural_analyze_command_name(analyze_cmd)
            }
            AnalyzeCommandCategory::Specialized => {
                Self::get_specialized_analyze_command_name(analyze_cmd)
            }
        }
    }

    /// Toyota Way Extract Method: Get QDD command name
    fn get_qdd_command_name(qdd_cmd: &QddCommands) -> &'static str {
        match qdd_cmd {
            QddCommands::Create { .. } => "qdd-create",
            QddCommands::Refactor { .. } => "qdd-refactor",
            QddCommands::Validate { .. } => "qdd-validate",
        }
    }

    /// Toyota Way Extract Method: Basic analysis command names
    fn get_basic_analyze_command_name(analyze_cmd: &AnalyzeCommands) -> &'static str {
        match analyze_cmd {
            AnalyzeCommands::Churn { .. } => "analyze-churn",
            AnalyzeCommands::Complexity { .. } => "analyze-complexity",
            AnalyzeCommands::DeadCode { .. } => "analyze-dead-code",
            AnalyzeCommands::Satd { .. } => "analyze-satd",
            AnalyzeCommands::Tdg { .. } => "analyze-tdg",
            AnalyzeCommands::LintHotspot { .. } => "analyze-lint-hotspot",
            _ => unreachable!("Non-basic command passed to basic command name extractor"),
        }
    }

    /// Toyota Way Extract Method: Advanced analysis command names
    fn get_advanced_analyze_command_name(analyze_cmd: &AnalyzeCommands) -> &'static str {
        match analyze_cmd {
            AnalyzeCommands::DeepContext { .. } => "analyze-deep-context",
            AnalyzeCommands::Comprehensive { .. } => "analyze-comprehensive",
            AnalyzeCommands::DefectPrediction { .. } => "analyze-defect-prediction",
            AnalyzeCommands::Duplicates { .. } => "analyze-duplicates",
            AnalyzeCommands::BigO { .. } => "analyze-big-o",
            _ => unreachable!("Non-advanced command passed to advanced command name extractor"),
        }
    }

    /// Toyota Way Extract Method: Structural analysis command names
    fn get_structural_analyze_command_name(analyze_cmd: &AnalyzeCommands) -> &'static str {
        match analyze_cmd {
            AnalyzeCommands::Dag { .. } => "analyze-dag",
            AnalyzeCommands::GraphMetrics { .. } => "analyze-graph-metrics",
            AnalyzeCommands::SymbolTable { .. } => "analyze-symbol-table",
            AnalyzeCommands::NameSimilarity { .. } => "analyze-name-similarity",
            _ => unreachable!("Non-structural command passed to structural command name extractor"),
        }
    }

    /// Toyota Way Extract Method: Specialized analysis command names
    fn get_specialized_analyze_command_name(analyze_cmd: &AnalyzeCommands) -> &'static str {
        match analyze_cmd {
            AnalyzeCommands::Makefile { .. } => "analyze-makefile",
            AnalyzeCommands::Provability { .. } => "analyze-provability",
            AnalyzeCommands::ProofAnnotations { .. } => "analyze-proof-annotations",
            AnalyzeCommands::IncrementalCoverage { .. } => "analyze-incremental-coverage",
            AnalyzeCommands::AssemblyScript { .. } => "analyze-assemblyscript",
            AnalyzeCommands::WebAssembly { .. } => "analyze-webassembly",
            _ => {
                unreachable!("Non-specialized command passed to specialized command name extractor")
            }
        }
    }

    #[must_use]
    pub fn from_commands(command: Commands) -> Self {
        // Toyota Way Extract Method: Get command name using categorized dispatch
        let command_name = Self::get_command_name_by_category(&command);

        Self {
            command,
            command_name,
            raw_args: std::env::args().collect(),
        }
    }

    /// Toyota Way Extract Method: Get command name using categorized dispatch
    /// Reduces complexity from 23 branches to category-based logic
    fn get_command_name_by_category(command: &Commands) -> String {
        match command {
            // Special case: Analyze command needs sub-command delegation
            Commands::Analyze(analyze_cmd) => Self::get_analyze_command_name(analyze_cmd),
            // Special case: QDD command needs sub-command delegation
            Commands::Qdd(qdd_cmd) => Self::get_qdd_command_name(qdd_cmd),
            // All other commands: extract name directly using category dispatch
            _ => Self::get_simple_command_name(command),
        }
        .to_string()
    }

    /// Toyota Way Extract Method: Get simple command name for non-analyze commands
    /// Single responsibility: name extraction using category-based dispatch
    fn get_simple_command_name(command: &Commands) -> &'static str {
        let category = Self::get_command_category(command);

        match category {
            CommandCategory::Generation => Self::get_generation_command_name(command),
            CommandCategory::Analysis => Self::get_analysis_command_name(command),
            CommandCategory::Operations => Self::get_operations_command_name(command),
            CommandCategory::Workflow => Self::get_workflow_command_name(command),
            CommandCategory::System => Self::get_system_command_name(command),
            CommandCategory::Configuration => Self::get_configuration_command_name(command),
            CommandCategory::Demo => "demo",
            CommandCategory::Enforcement => "enforce",
        }
    }

    /// Toyota Way Extract Method: Determine command category
    fn get_command_category(command: &Commands) -> CommandCategory {
        match command {
            Commands::Generate { .. } | Commands::Scaffold { .. } => CommandCategory::Generation,
            Commands::QualityGate { .. } | Commands::QualityGates { .. } | Commands::Report { .. } | Commands::RepoScore { .. } | Commands::RustProjectScore { .. } | Commands::BrickScore { .. } | Commands::PopperScore { .. } | Commands::DemoScore { .. } | Commands::ValidateDocs(_) | Commands::ValidateReadme(_) | Commands::RedTeam(_) | Commands::Org(_) | Commands::Prompt(_) | Commands::Embed(_) | Commands::Semantic(_) | Commands::ShowMetrics { .. } | Commands::PredictQuality { .. } | Commands::RecordMetric { .. } | Commands::DepsAudit { .. } => CommandCategory::Analysis,
            #[cfg(feature = "mutation-testing")]
            Commands::Mutate(_) => CommandCategory::Analysis,
            Commands::Serve { .. }
            | Commands::Cache { .. }
            | Commands::Memory { .. }
            | Commands::Telemetry { .. } => CommandCategory::Operations,
            Commands::Refactor(_)
            | Commands::Test { .. }
            | Commands::Roadmap(_)
            | Commands::Maintain { .. } // TICKET-PMAT-5032
            | Commands::Hooks(_) // TICKET-PMAT-5034
            | Commands::Validate { .. } => CommandCategory::Workflow,
            Commands::List { .. }
            | Commands::Search { .. }
            | Commands::Context { .. }
            | Commands::Diagnose(_)
            | Commands::Debug { .. } => CommandCategory::System,
            Commands::Config { .. } | Commands::Agent { .. } | Commands::Tdg { .. } => {
                CommandCategory::Configuration
            }
            Commands::Demo { .. } => CommandCategory::Demo,
            Commands::Enforce(_) => CommandCategory::Enforcement,
            Commands::Analyze(_) => {
                unreachable!("Analyze commands handled by get_analyze_command_name")
            }
            Commands::Qdd(_) => {
                unreachable!("QDD commands handled by get_qdd_command_name")
            }
            Commands::Work { .. } => {
                CommandCategory::Workflow // Issue #75: Unified GitHub/YAML workflow
            }
            Commands::Comply { .. } => {
                CommandCategory::Analysis // GH-96: PMAT compliance and migration system
            }
            Commands::ProjectDiag { .. } => {
                CommandCategory::Analysis // Project diagnostics - lltop Tab 8 equivalent
            }
            Commands::TestDiscovery { .. } => {
                CommandCategory::Analysis // GH-98: Systematic test discovery and fixing
            }
            Commands::DebugFiveWhys { .. } => {
                CommandCategory::Analysis // Five Whys root cause analysis
            }
            Commands::Localize { .. } => {
                CommandCategory::Analysis // GH-103: Tarantula fault localization
            }
            Commands::Oracle { .. } => {
                CommandCategory::Workflow // PMAT Oracle - PDCA loop for automated quality improvement
            }
            Commands::QaWork { .. } => {
                CommandCategory::Workflow // GH-102: Toyota Way QA validation
            }
            Commands::PerfectionScore { .. } => {
                CommandCategory::Analysis // master-plan-pmat-work-system.md: 200-point unified score
            }
            Commands::Spec { .. } => {
                CommandCategory::Workflow // master-plan-pmat-work-system.md: Spec management
            }
            Commands::CudaTdg { .. } => {
                CommandCategory::Analysis // CUDA-SIMD TDG: 100-point Popper falsification
            }
        }
    }

    /// Toyota Way Extract Method: Generation command names
    fn get_generation_command_name(command: &Commands) -> &'static str {
        match command {
            Commands::Generate { .. } => "generate",
            Commands::Scaffold { .. } => "scaffold",
            _ => unreachable!("Non-generation command passed to generation command name extractor"),
        }
    }

    /// Toyota Way Extract Method: Analysis command names (non-analyze)
    fn get_analysis_command_name(command: &Commands) -> &'static str {
        match command {
            Commands::QualityGate { .. } => "quality-gate",
            Commands::Report { .. } => "report",
            Commands::DebugFiveWhys { .. } => "five-whys",
            _ => unreachable!("Non-analysis command passed to analysis command name extractor"),
        }
    }

    /// Toyota Way Extract Method: Operations command names
    fn get_operations_command_name(command: &Commands) -> &'static str {
        match command {
            Commands::Serve { .. } => "serve",
            Commands::Cache { .. } => "cache",
            Commands::Memory { .. } => "memory",
            Commands::Telemetry { .. } => "telemetry",
            _ => unreachable!("Non-operations command passed to operations command name extractor"),
        }
    }

    /// Toyota Way Extract Method: Workflow command names
    fn get_workflow_command_name(command: &Commands) -> &'static str {
        match command {
            Commands::Refactor(_) => "refactor",
            Commands::Test { .. } => "test",
            Commands::Roadmap(_) => "roadmap",
            Commands::Validate { .. } => "validate",
            Commands::Maintain { .. } => "maintain",
            Commands::Hooks(_) => "hooks",
            Commands::Work { .. } => "work",
            _ => unreachable!("Non-workflow command passed to workflow command name extractor"),
        }
    }

    /// Toyota Way Extract Method: System command names
    fn get_system_command_name(command: &Commands) -> &'static str {
        match command {
            Commands::List { .. } => "list",
            Commands::Search { .. } => "search",
            Commands::Context { .. } => "context",
            Commands::Diagnose(_) => "diagnose",
            Commands::Debug { .. } => "debug",
            _ => unreachable!("Non-system command passed to system command name extractor"),
        }
    }

    /// Toyota Way Extract Method: Configuration command names
    fn get_configuration_command_name(command: &Commands) -> &'static str {
        match command {
            Commands::Config { .. } => "config",
            Commands::Agent { .. } => "agent",
            Commands::Tdg { .. } => "tdg",
            _ => unreachable!(
                "Non-configuration command passed to configuration command name extractor"
            ),
        }
    }
}

/// Toyota Way Extract Method: Categories for analyze command dispatch
/// Reduces complexity from 24 branches to 4 categories
#[derive(Debug, Clone, Copy)]
enum AnalyzeCommandCategory {
    /// Core analysis commands (basic metrics): churn, complexity, dead code, SATD, TDG, lint hotspots
    Basic,
    /// Advanced analysis commands (comprehensive): deep context, comprehensive, defect prediction, duplicates, `BigO`
    Advanced,
    /// Graph and structural analysis: DAG, graph metrics, symbol table, name similarity
    Structural,
    /// Specialized analysis commands: makefile, provability, proof annotations, coverage, WebAssembly
    Specialized,
}

/// Toyota Way Extract Method: Categories for general CLI command dispatch
/// Reduces complexity from 23 branches to logical groups
#[derive(Debug, Clone, Copy)]
enum CommandCategory {
    /// Generation and creation commands: generate, scaffold
    Generation,
    /// Analysis and assessment commands: analyze (delegated), quality-gate, report
    Analysis,
    /// Operations and maintenance commands: serve, cache, memory, telemetry
    Operations,
    /// Development workflow commands: refactor, test, roadmap, validate
    Workflow,
    /// System interaction commands: list, search, context, diagnose
    System,
    /// Configuration and setup commands: config, agent, tdg
    Configuration,
    /// Demo and examples: demo
    Demo,
    /// Runtime enforcement: enforce
    Enforcement,
}

impl CliAdapter {
    /// Toyota Way Extract Method: Categorize analyze command by type
    /// Single responsibility: classification logic only
    fn get_analyze_command_category(analyze_cmd: &AnalyzeCommands) -> AnalyzeCommandCategory {
        match analyze_cmd {
            // Core analysis commands (basic metrics)
            AnalyzeCommands::Churn { .. }
            | AnalyzeCommands::Complexity { .. }
            | AnalyzeCommands::DeadCode { .. }
            | AnalyzeCommands::Defects { .. }
            | AnalyzeCommands::Satd { .. }
            | AnalyzeCommands::Tdg { .. }
            | AnalyzeCommands::BuildTdg { .. }
            | AnalyzeCommands::LintHotspot { .. }
            | AnalyzeCommands::Clippy { .. }
            | AnalyzeCommands::Entropy { .. } => AnalyzeCommandCategory::Basic,

            // Advanced analysis commands (comprehensive)
            AnalyzeCommands::DeepContext { .. }
            | AnalyzeCommands::Comprehensive { .. }
            | AnalyzeCommands::DefectPrediction { .. }
            | AnalyzeCommands::Duplicates { .. }
            | AnalyzeCommands::BigO { .. } => AnalyzeCommandCategory::Advanced,

            // Graph and structural analysis
            AnalyzeCommands::Dag { .. }
            | AnalyzeCommands::GraphMetrics { .. }
            | AnalyzeCommands::SymbolTable { .. }
            | AnalyzeCommands::NameSimilarity { .. } => AnalyzeCommandCategory::Structural,

            // Specialized analysis commands
            AnalyzeCommands::Makefile { .. }
            | AnalyzeCommands::Provability { .. }
            | AnalyzeCommands::ProofAnnotations { .. }
            | AnalyzeCommands::IncrementalCoverage { .. }
            | AnalyzeCommands::CoverageImprove { .. }
            | AnalyzeCommands::AssemblyScript { .. }
            | AnalyzeCommands::WebAssembly { .. }
            | AnalyzeCommands::Wasm { .. }
            | AnalyzeCommands::Cluster { .. } // PMAT-SEARCH-011
            | AnalyzeCommands::Topics { .. } // PMAT-SEARCH-011
            => AnalyzeCommandCategory::Specialized,

            #[cfg(feature = "mutation-testing")]
            AnalyzeCommands::Mutate { .. } => AnalyzeCommandCategory::Specialized,

            #[cfg(feature = "deep-wasm")]
            AnalyzeCommands::DeepWasm { .. } => AnalyzeCommandCategory::Specialized,
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
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            CliOutput::Success { exit_code, .. } => *exit_code,
            CliOutput::Error { exit_code, .. } => *exit_code,
        }
    }

    /// Get the content/message
    #[must_use]
    pub fn content(&self) -> &str {
        match self {
            CliOutput::Success { content, .. } => content,
            CliOutput::Error { message, .. } => message,
        }
    }
}

// Tests extracted to cli_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
