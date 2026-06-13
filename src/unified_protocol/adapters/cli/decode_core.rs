impl CliAdapter {
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
            | Commands::Query { .. } // Semantic code search (CLI-only)
            | Commands::Falsify { .. } // Falsification testing (CLI-only)
            | Commands::Kaizen { .. } // Kaizen continuous improvement (CLI-only)
            | Commands::Extract { .. } // Extract refactoring (CLI-only)
            | Commands::Split { .. } // File splitting (CLI-only)
            | Commands::Sql { .. } // SQL analytics (CLI-only)
            | Commands::Score { .. } // Unified quality score (CLI-only)
            | Commands::InfraScore { .. } // Infrastructure CI/CD quality score (CLI-only)
            | Commands::Verify(_) // CI-faithful pre-flight gate (CLI-only)
            | Commands::Explain { .. } // Check/metric explanation lookup (CLI-only)
            | Commands::CiLocal { .. } // Local CI simulation (CLI-only)
            | Commands::TestStability { .. } // Flaky/timeout test detection (CLI-only)
            | Commands::Stack { .. } // Cross-repo dependency coordination (CLI-only)
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
            Some(*format),
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
}
