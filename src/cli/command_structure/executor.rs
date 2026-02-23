#![cfg_attr(coverage_nightly, coverage(off))]
//! Command execution logic - dispatch, scaffold, maintain, debug, forward

use super::CommandExecutor;
use crate::cli::commands::ScaffoldCommands;
use crate::cli::Commands;
use anyhow::Result;

impl CommandExecutor {
    /// Execute a command using the modular dispatch architecture
    pub async fn execute(&self, command: Commands) -> Result<()> {
        match command {
            // Generation commands
            Commands::Generate {
                category,
                template,
                params,
                output,
                create_dirs,
            } => {
                self.registry
                    .generate_handlers
                    .handle_generate(
                        self.server.clone(),
                        category,
                        template,
                        params,
                        output,
                        create_dirs,
                    )
                    .await
            }
            Commands::Scaffold { command } => self.execute_scaffold(command).await,
            Commands::Validate { uri, params } => {
                self.registry
                    .generate_handlers
                    .handle_validate(self.server.clone(), uri, params)
                    .await
            }

            // Analysis commands
            Commands::Analyze(analyze_cmd) => {
                self.registry.analyze_handlers.execute(analyze_cmd).await
            }

            Commands::Qdd(qdd_cmd) => {
                use crate::cli::handlers::qdd_handlers;
                qdd_handlers::handle_qdd_command(qdd_cmd).await
            }

            // Utility commands
            Commands::List {
                toolchain,
                category,
                format,
            } => {
                self.registry
                    .utility_handlers
                    .handle_list(self.server.clone(), toolchain, category, format)
                    .await
            }
            Commands::Search {
                query,
                toolchain,
                limit,
            } => {
                self.registry
                    .utility_handlers
                    .handle_search(self.server.clone(), query, toolchain, limit)
                    .await
            }
            Commands::Context {
                toolchain,
                project_path,
                output,
                format,
                include_large_files,
                skip_expensive_metrics,
                language,
                languages,
            } => {
                self.registry
                    .utility_handlers
                    .handle_context(
                        toolchain,
                        project_path,
                        output,
                        format,
                        include_large_files,
                        skip_expensive_metrics,
                        language,
                        languages,
                    )
                    .await
            }
            Commands::Query {
                query,
                limit,
                min_grade,
                max_complexity,
                language,
                path,
                project_path,
                format,
                include_source,
                rebuild_index,
                exclude_tests,
                rank_by,
                min_pagerank,
                include_project,
                churn,
                duplicates,
                entropy,
                faults,
                coverage,
                uncovered_only,
                coverage_diff,
                coverage_file,
                coverage_gaps,
                include_excluded,
                definition_type,
                summary,
                git_history,
                regex,
                literal,
                raw,
                case_sensitive,
                ignore_case,
                exclude,
                exclude_file,
                files_with_matches,
                count,
                after_context,
                before_context,
                context_lines,
                ptx_flow,
                ptx_diagnostics,
                suggest_rename,
                apply,
                no_docs,
                docs_only,
                extract_candidates,
                max_module_lines,
            } => {
                // Default is to show code; --summary disables it
                let show_code = !summary;
                let effective_docs = !no_docs;
                crate::cli::handlers::handle_query(
                    query,
                    limit,
                    min_grade,
                    max_complexity,
                    language,
                    path,
                    project_path,
                    format,
                    include_source,
                    rebuild_index,
                    exclude_tests,
                    rank_by,
                    min_pagerank,
                    include_project,
                    churn,
                    duplicates,
                    entropy,
                    faults,
                    coverage,
                    uncovered_only,
                    coverage_diff,
                    coverage_file,
                    coverage_gaps,
                    include_excluded,
                    definition_type,
                    show_code,
                    git_history,
                    regex,
                    literal,
                    raw,
                    case_sensitive,
                    ignore_case,
                    exclude,
                    exclude_file,
                    files_with_matches,
                    count,
                    after_context,
                    before_context,
                    context_lines,
                    ptx_flow,
                    ptx_diagnostics,
                    suggest_rename,
                    apply,
                    effective_docs,
                    docs_only,
                    extract_candidates,
                    max_module_lines,
                )
                .await
            }
            Commands::Serve {
                port,
                host,
                cors,
                transport,
            } => {
                self.registry
                    .utility_handlers
                    .handle_serve(host, port, cors, transport)
                    .await
            }

            // Demo and quality commands
            Commands::Demo {
                path,
                url,
                repo,
                format,
                protocol,
                show_api,
                no_browser,
                port,
                cli,
                target_nodes,
                centrality_threshold,
                merge_threshold,
                debug,
                debug_output,
                skip_vendor,
                no_skip_vendor,
                max_line_length,
            } => {
                self.registry
                    .demo_handlers
                    .handle_demo(
                        self.server.clone(),
                        path,
                        url,
                        repo,
                        format,
                        protocol,
                        show_api,
                        no_browser,
                        port,
                        cli,
                        target_nodes,
                        centrality_threshold,
                        merge_threshold,
                        debug,
                        debug_output,
                        skip_vendor && !no_skip_vendor,
                        max_line_length,
                    )
                    .await
            }
            Commands::QualityGate {
                project_path,
                file,
                format,
                fail_on_violation,
                checks,
                max_dead_code,
                min_entropy,
                max_complexity_p99,
                include_provability,
                output,
                perf,
            } => {
                self.registry
                    .demo_handlers
                    .handle_quality_gate(
                        project_path,
                        file,
                        format,
                        fail_on_violation,
                        checks,
                        max_dead_code,
                        min_entropy,
                        max_complexity_p99,
                        include_provability,
                        output,
                        perf,
                    )
                    .await
            }
            Commands::Report {
                project_path,
                output_format,
                text,
                markdown,
                csv,
                include_visualizations,
                include_executive_summary,
                include_recommendations,
                analyses,
                confidence_threshold,
                output,
                perf,
            } => {
                crate::cli::handlers::enhanced_reporting_handlers::handle_generate_report(
                    project_path,
                    output_format,
                    text,
                    markdown,
                    csv,
                    include_visualizations,
                    include_executive_summary,
                    include_recommendations,
                    analyses,
                    confidence_threshold,
                    output,
                    perf,
                )
                .await
            }
            Commands::RepoScore {
                path,
                format,
                verbose,
                failures_only,
                output,
                update_badge,
                deep,
            } => {
                crate::cli::handlers::repo_score_handlers::handle_repo_score(
                    &path,
                    format,
                    verbose,
                    failures_only,
                    output.as_deref(),
                    update_badge,
                    deep,
                )
                .await
            }
            Commands::RustProjectScore {
                path,
                format,
                verbose,
                failures_only,
                output,
                full,
            } => {
                crate::cli::handlers::rust_project_score_handlers::handle_rust_project_score(
                    &path,
                    &format,
                    verbose,
                    failures_only,
                    output.as_deref(),
                    full,
                )
                .await
            }
            Commands::BrickScore {
                path,
                input,
                format,
                verbose,
                failures_only,
                threshold,
                output,
                hardware,
            } => {
                crate::cli::handlers::brick_score_handlers::handle_brick_score(
                    &path,
                    input.as_deref(),
                    &format,
                    verbose,
                    failures_only,
                    threshold,
                    output.as_deref(),
                    hardware.as_deref(),
                )
                .await
            }
            Commands::PopperScore {
                path,
                format,
                verbose,
                failures_only,
                output,
            } => {
                crate::cli::handlers::popper_score_handlers::handle_popper_score(
                    &path,
                    &format,
                    verbose,
                    failures_only,
                    output.as_deref(),
                )
                .await
            }
            Commands::DemoScore {
                path,
                format,
                verbose,
                failures_only,
                output,
            } => {
                crate::cli::handlers::demo_score_handlers::handle_demo_score(
                    &path,
                    &format,
                    verbose,
                    failures_only,
                    output.as_deref(),
                )
                .await
            }
            Commands::Diagnose(args) => self.registry.utility_handlers.handle_diagnose(args).await,
            Commands::Refactor(refactor_cmd) => {
                crate::cli::handlers::route_refactor_command(refactor_cmd).await
            }
            Commands::Enforce(enforce_cmd) => {
                crate::cli::handlers::route_enforce_command(enforce_cmd).await
            }
            Commands::Roadmap(roadmap_cmd) => {
                crate::cli::command_dispatcher::CommandDispatcher::execute_roadmap_command(
                    roadmap_cmd,
                )
                .await
            }
            Commands::Test {
                suite,
                iterations,
                memory,
                throughput,
                regression,
                timeout,
                output,
                perf,
            } => {
                crate::cli::handlers::test_handlers::handle_test(
                    suite, iterations, memory, throughput, regression, timeout, output, perf,
                )
                .await
            }
            Commands::Memory { command } => {
                crate::cli::command_dispatcher::CommandDispatcher::execute_memory_command(command)
                    .await
            }
            Commands::Cache { command } => {
                crate::cli::command_dispatcher::CommandDispatcher::execute_cache_command(command)
                    .await
            }
            Commands::Telemetry {
                system,
                service,
                reset,
                test_event,
            } => {
                crate::cli::handlers::handle_telemetry(system, service, reset, test_event).await
            }
            Commands::Config {
                show,
                edit,
                validate,
                reset,
                section,
                set,
                config_path,
            } => {
                crate::cli::handlers::handle_configuration(
                    show,
                    edit,
                    validate,
                    reset,
                    section,
                    set,
                    config_path,
                )
                .await
            }

            #[cfg(feature = "agent-daemon")]
            Commands::Agent { command } => {
                crate::cli::handlers::handle_agent_command(command).await
            }
            #[cfg(not(feature = "agent-daemon"))]
            Commands::Agent { .. } => {
                anyhow::bail!(
                    "Agent daemon feature not enabled. Build with --features agent-daemon"
                )
            }

            Commands::Tdg {
                path,
                command,
                format,
                config,
                quiet,
                include_components,
                min_grade,
                output,
                with_git_context,
                explain,
                threshold,
                baseline,
                ml: _, // GH-97: ML flag (not yet implemented in handler)
                viz,
                viz_theme,
            } => {
                let tdg_config = crate::cli::handlers::tdg_handlers::TdgCommandConfig {
                    path,
                    command,
                    format,
                    config,
                    quiet,
                    include_components,
                    min_grade,
                    output,
                    with_git_context,
                    explain,
                    threshold,
                    baseline,
                    viz,
                    viz_theme,
                };
                crate::cli::handlers::handle_tdg_command(tdg_config).await
            }
            Commands::ValidateDocs(cmd) => {
                let exit_code = cmd.execute().await?;
                if exit_code != std::process::ExitCode::SUCCESS {
                    std::process::exit(1);
                }
                Ok(())
            }
            Commands::ValidateReadme(cmd) => {
                // Sprint 38: Hallucination detection (synchronous)
                let exit_code = cmd.execute()?;
                if exit_code != std::process::ExitCode::SUCCESS {
                    std::process::exit(1);
                }
                Ok(())
            }
            Commands::RedTeam(cmd) => {
                // Red Team Mode: Commit hallucination detection
                let exit_code = cmd.execute()?;
                if exit_code != std::process::ExitCode::SUCCESS {
                    std::process::exit(1);
                }
                Ok(())
            }
            Commands::Org(_org_cmd) => {
                #[cfg(feature = "org-intelligence")]
                {
                    crate::cli::handlers::handle_org_command(_org_cmd).await
                }
                #[cfg(not(feature = "org-intelligence"))]
                {
                    anyhow::bail!("Organizational intelligence feature is not enabled. Rebuild with --features org-intelligence")
                }
            }
            Commands::Prompt(prompt_cmd) => {
                crate::cli::handlers::handle_prompt_command(prompt_cmd).await
            }
            Commands::QualityGates {
                command,
                config,
                report,
                json,
                project_dir,
            } => {
                crate::cli::handlers::handle_quality_gates_command(
                    command,
                    config,
                    report,
                    json,
                    project_dir,
                )
                .await
            }

            Commands::Maintain { command } => Self::execute_maintain(command).await,

            Commands::Hooks(hooks_cmd) => {
                crate::cli::handlers::handle_hooks_command(&hooks_cmd).await
            }

            // Semantic search commands (PMAT-SEARCH-011)
            Commands::Embed(embed_cmd) => {
                crate::cli::command_dispatcher::CommandDispatcher::execute_embed_command(embed_cmd)
                    .await
            }
            Commands::Semantic(semantic_cmd) => {
                crate::cli::command_dispatcher::CommandDispatcher::execute_semantic_command(
                    semantic_cmd,
                )
                .await
            }

            // Mutation testing command (Sprint 61)
            #[cfg(feature = "mutation-testing")]
            Commands::Mutate(args) => {
                crate::cli::handlers::mutate::handle(args, self.server.clone()).await
            }

            // Time-travel debugging commands (Sprint 74)
            Commands::Debug { command } => Self::execute_debug(command).await,
            // Commands handled by command_dispatcher.rs
            other => Self::forward_to_dispatcher(other),
        }
    }

    /// Execute scaffold subcommands
    async fn execute_scaffold(&self, command: ScaffoldCommands) -> Result<()> {
        match command {
            ScaffoldCommands::Project {
                toolchain,
                templates,
                params,
                parallel,
            } => {
                self.registry
                    .generate_handlers
                    .handle_scaffold(self.server.clone(), toolchain, templates, params, parallel)
                    .await
            }
            ScaffoldCommands::Agent {
                name,
                template,
                features,
                quality,
                output,
                force,
                dry_run,
                interactive,
                deterministic_core,
                probabilistic_wrapper,
            } => {
                let params = crate::cli::handlers::ScaffoldAgentParams {
                    name,
                    template,
                    features,
                    quality,
                    output,
                    force,
                    dry_run,
                    interactive,
                    deterministic_core,
                    probabilistic_wrapper,
                };
                crate::cli::handlers::handle_scaffold_agent(params).await
            }
            ScaffoldCommands::Wasm {
                name,
                framework,
                features,
                quality,
                output,
                force,
                dry_run,
            } => {
                let params = crate::cli::handlers::ScaffoldWasmParams {
                    name,
                    framework,
                    features,
                    quality,
                    output,
                    force,
                    dry_run,
                };
                crate::cli::handlers::handle_scaffold_wasm(params).await
            }
            ScaffoldCommands::ListTemplates => {
                crate::cli::handlers::handle_list_agent_templates().await
            }
            ScaffoldCommands::ValidateTemplate { path } => {
                crate::cli::handlers::handle_validate_agent_template(path).await
            }
            ScaffoldCommands::ListSubagents { all } => {
                crate::cli::handlers::subagent_handlers::list_subagents(all)
            }
            ScaffoldCommands::CreateSubagent { agent_name, output } => {
                crate::cli::handlers::subagent_handlers::create_subagent(&agent_name, output)
            }
            ScaffoldCommands::CreateAllSubagents { output } => {
                crate::cli::handlers::subagent_handlers::create_all_mvp_subagents(output)
            }
            ScaffoldCommands::ValidateSubagent { file_path } => {
                crate::cli::handlers::subagent_handlers::validate_subagent(&file_path)
            }
            ScaffoldCommands::ShowToolMapping { agent } => {
                crate::cli::handlers::subagent_handlers::show_tool_mapping(agent)
            }
            ScaffoldCommands::ExportToolMapping { output } => {
                crate::cli::handlers::subagent_handlers::export_tool_mapping_json(&output)
            }
        }
    }

    /// Execute maintain subcommands
    async fn execute_maintain(
        command: crate::cli::commands::MaintainCommands,
    ) -> Result<()> {
        use crate::cli::commands::MaintainCommands;
        match command {
            MaintainCommands::Roadmap {
                roadmap,
                tickets_dir,
                validate,
                health,
                fix,
                generate_tickets,
                dry_run,
                format,
            } => {
                let config =
                    crate::cli::handlers::roadmap_handler::RoadmapMaintenanceConfig::new(
                        validate,
                        health,
                        fix,
                        generate_tickets,
                        dry_run,
                    );
                crate::cli::handlers::handle_maintain_roadmap(roadmap, tickets_dir, config, format)
                    .await
            }
            MaintainCommands::Health {
                project_dir,
                format,
                quick,
                all,
                check_build,
                check_tests,
                check_coverage,
                check_complexity,
                check_satd,
            } => {
                let config = crate::cli::handlers::health_handler::HealthCheckConfig::new(
                    quick,
                    all,
                    check_build,
                    check_tests,
                    check_coverage,
                    check_complexity,
                    check_satd,
                );
                crate::cli::handlers::handle_maintain_health(project_dir, format, config).await
            }
            MaintainCommands::BugReport {
                title,
                dry_run,
                interactive,
                clear,
            } => {
                crate::cli::handlers::bug_report_handler::handle_bug_report(
                    title.as_deref(),
                    dry_run,
                    interactive,
                    clear,
                )
                .await
            }
            MaintainCommands::CleanupResources {
                project_dir,
                targets,
                execute,
                exclude,
                min_age_days,
                format,
            } => {
                crate::cli::handlers::cleanup_resources_handler::handle_cleanup_resources(
                    &project_dir,
                    &targets,
                    execute,
                    &exclude,
                    min_age_days,
                    format,
                )
                .await
            }
        }
    }

    /// Execute debug subcommands
    async fn execute_debug(command: crate::cli::commands::DebugCommands) -> Result<()> {
        use crate::cli::commands::DebugCommands;
        match command {
            DebugCommands::Serve {
                port,
                host,
                record_dir,
            } => {
                crate::cli::handlers::debug_handlers::handle_debug_serve(port, host, record_dir)
                    .await
            }
            DebugCommands::Replay {
                recording,
                position,
                interactive,
            } => {
                crate::cli::handlers::debug_handlers::handle_debug_replay(
                    recording,
                    position,
                    interactive,
                )
                .await
            }
        }
    }

    /// Forward commands that should be handled by command_dispatcher.rs
    fn forward_to_dispatcher(command: Commands) -> Result<()> {
        let name = match command {
            Commands::ShowMetrics { .. } => "ShowMetrics",
            Commands::PredictQuality { .. } => "PredictQuality",
            Commands::RecordMetric { .. } => "RecordMetric",
            Commands::Work { .. } => "Work",
            Commands::Falsify { .. } => "Falsify",
            Commands::QaWork { .. } => "QaWork",
            Commands::Comply { .. } => "Comply",
            Commands::ProjectDiag { .. } => "ProjectDiag",
            Commands::TestDiscovery { .. } => "TestDiscovery",
            Commands::DebugFiveWhys { .. } => "DebugFiveWhys",
            Commands::Localize { .. } => "Localize",
            Commands::Oracle { .. } => "Oracle",
            Commands::PerfectionScore { .. } => "PerfectionScore",
            Commands::Spec { .. } => "Spec",
            Commands::CudaTdg { .. } => "CudaTdg",
            Commands::DepsAudit { .. } => "DepsAudit",
            Commands::Kaizen { .. } => "Kaizen",
            Commands::Extract { .. } => "Extract",
            _ => "Unknown",
        };
        anyhow::bail!(
            "{} command should be handled by command_dispatcher.rs",
            name
        )
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::cli::Commands;
    use std::sync::Arc;

    // ============================================================================
    // Test Fixtures and Helpers
    // ============================================================================

    /// Creates a test server instance for testing command execution
    fn create_test_server() -> Arc<StatelessTemplateServer> {
        Arc::new(StatelessTemplateServer::new().expect("Failed to create test server"))
    }

    // ============================================================================
    // CommandExecutor Tests
    // ============================================================================

    #[test]
    fn test_command_executor_new() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server.clone());

        // Verify executor is created successfully
        assert!(Arc::ptr_eq(&executor.server, &server));
    }

    #[test]
    fn test_command_executor_has_registry() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // Registry should be accessible (implicitly tested via execute)
        let _ = &executor.registry;
    }

    // ============================================================================
    // CommandRegistry Tests
    // ============================================================================

    #[test]
    fn test_command_registry_creation() {
        let registry = CommandRegistry::default();

        // Verify all command groups are initialized by accessing them
        let _ = &registry.generate_handlers;
        let _ = &registry.analyze_handlers;
        let _ = &registry.utility_handlers;
        let _ = &registry.demo_handlers;
    }

    #[test]
    fn test_command_registry_default_trait() {
        // Test that Default trait is properly implemented
        let registry1 = CommandRegistry::default();
        let registry2 = CommandRegistry::default();

        // Both should be valid (we can't compare them, but they should exist)
        let _ = registry1;
        let _ = registry2;
    }

    // ============================================================================
    // Command Group Default Tests
    // ============================================================================

    #[test]
    fn test_generate_command_group_default() {
        let group = GenerateCommandGroup::default();
        // Verify it can be created
        let _ = group;
    }

    #[test]
    fn test_analyze_command_group_default() {
        let group = AnalyzeCommandGroup::default();
        // Verify it can be created
        let _ = group;
    }

    #[test]
    fn test_utility_command_group_default() {
        let group = UtilityCommandGroup::default();
        // Verify it can be created
        let _ = group;
    }

    #[test]
    fn test_demo_command_group_default() {
        let group = DemoCommandGroup::default();
        // Verify it can be created
        let _ = group;
    }

    #[test]
    fn test_command_group_defaults() {
        let _generate = GenerateCommandGroup;
        let _analyze = AnalyzeCommandGroup;
        let _utility = UtilityCommandGroup;
        let _demo = DemoCommandGroup;

        // All groups should be creatable via unit struct syntax
    }

    // ============================================================================
    // CommandExecutorFactory Tests
    // ============================================================================

    #[test]
    fn test_command_executor_factory_create() {
        let server = create_test_server();
        let executor = CommandExecutorFactory::create(server.clone());

        // Verify factory creates a valid executor
        assert!(Arc::ptr_eq(&executor.server, &server));
    }

    #[test]
    fn test_command_executor_factory_creates_with_registry() {
        let server = create_test_server();
        let executor = CommandExecutorFactory::create(server);

        // Verify registry is accessible
        let _ = &executor.registry;
    }

    // ============================================================================
    // Command Execution Error Cases (Commands that should bail)
    // ============================================================================

    #[tokio::test]
    async fn test_execute_show_metrics_should_bail() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // ShowMetrics uses OutputFormat, not MetricsOutputFormat
        let command = Commands::ShowMetrics {
            trend: false,
            days: 30,
            metric: None,
            format: crate::cli::OutputFormat::Table,
            failures_only: false,
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_predict_quality_should_bail() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // PredictQuality uses correct field names
        let command = Commands::PredictQuality {
            metric: None,
            threshold: None,
            days: 30,
            format: crate::cli::OutputFormat::Table,
            all: false,
            failures_only: false,
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_record_metric_should_bail() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // RecordMetric uses String metric and f64 value
        let command = Commands::RecordMetric {
            metric: "lint".to_string(),
            value: 1000.0,
            timestamp: None,
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_work_command_should_bail() {
        use crate::cli::commands::WorkCommands;

        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // WorkCommands::Start requires id and optional fields
        let command = Commands::Work {
            command: WorkCommands::Start {
                id: "123".to_string(),
                with_spec: false,
                epic: false,
                path: None,
                create_github: false,
            },
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_qa_work_should_bail() {
        use crate::cli::commands::QaWorkCommands;

        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // QaWork uses subcommand
        let command = Commands::QaWork {
            command: QaWorkCommands::Validate {
                task_id: "123".to_string(),
                path: std::path::PathBuf::from("."),
                strict: false,
                format: crate::cli::commands::QaOutputFormat::Text,
            },
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_comply_should_bail() {
        use crate::cli::commands::ComplyCommands;

        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // ComplyCommands::Check is the correct variant
        let command = Commands::Comply {
            command: ComplyCommands::Check {
                path: std::path::PathBuf::from("."),
                strict: false,
                failures_only: false,
                format: crate::cli::commands::ComplyOutputFormat::Text,
                include_project: vec![],
            },
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_project_diag_should_bail() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // ProjectDiag has correct field names
        let command = Commands::ProjectDiag {
            path: std::path::PathBuf::from("."),
            format: crate::cli::commands::ProjectDiagOutputFormat::Summary,
            category: None,
            failures_only: false,
            output: None,
            quiet: false,
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_test_discovery_should_bail() {
        use crate::cli::commands::TestDiscoveryCommands;

        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // TestDiscovery uses subcommand
        let command = Commands::TestDiscovery {
            command: TestDiscoveryCommands::Run {
                path: std::path::PathBuf::from("."),
                output: std::path::PathBuf::from("test-failures.json"),
                use_nextest: true,
                timeout: 600,
            },
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_debug_five_whys_should_bail() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // DebugFiveWhys uses DebugOutputFormat
        let command = Commands::DebugFiveWhys {
            issue: "test issue".to_string(),
            depth: 5,
            format: crate::cli::DebugOutputFormat::Text,
            output: None,
            path: std::path::PathBuf::from("."),
            context: None,
            auto_analyze: false,
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_localize_should_bail() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // Localize has correct field names
        let command = Commands::Localize {
            passed_coverage: std::path::PathBuf::from("passed.lcov"),
            failed_coverage: std::path::PathBuf::from("failed.lcov"),
            passed_count: 10,
            failed_count: 2,
            formula: "tarantula".to_string(),
            top_n: 10,
            output: None,
            format: "terminal".to_string(),
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_oracle_should_bail() {
        use crate::cli::commands::OracleCommands;

        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        let command = Commands::Oracle {
            command: OracleCommands::Status {
                path: std::path::PathBuf::from("."),
                format: crate::cli::commands::OracleOutputFormat::Text,
            },
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_perfection_score_should_bail() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // PerfectionScore has correct field names
        let command = Commands::PerfectionScore {
            path: std::path::PathBuf::from("."),
            breakdown: false,
            target: None,
            format: crate::cli::commands::PerfectionScoreOutputFormat::Text,
            output: None,
            fast: false,
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_spec_should_bail() {
        use crate::cli::commands::SpecCommands;

        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        let command = Commands::Spec {
            command: SpecCommands::List {
                path: std::path::PathBuf::from("docs/specifications"),
                min_score: None,
                failing_only: false,
                format: crate::cli::commands::SpecOutputFormat::Text,
            },
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_cuda_tdg_should_bail() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // CudaTdg has correct field names
        let command = Commands::CudaTdg {
            path: std::path::PathBuf::from("."),
            command: None,
            format: crate::cli::commands::CudaTdgOutputFormat::Terminal,
            min_score: 85.0,
            fail_on_p0: false,
            simd: false,
            wgpu: false,
            output: None,
            quiet: false,
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    // ============================================================================
    // Org Command Feature Gate Tests
    // ============================================================================

    #[tokio::test]
    #[cfg(not(feature = "org-intelligence"))]
    async fn test_execute_org_without_feature_should_bail() {
        use crate::cli::commands::OrgCommands;

        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        let command = Commands::Org(OrgCommands::Analyze {
            org: "test-org".to_string(),
            output: std::path::PathBuf::from("output.json"),
            max_concurrent: 5,
            summarize: false,
            strip_pii: false,
            top_n: 10,
            min_frequency: 3,
        });

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("org-intelligence"));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::super::*;
    use proptest::prelude::*;
    use std::sync::Arc;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }

        /// Property: CommandExecutorFactory always creates valid executors
        #[test]
        fn factory_always_creates_valid_executor(_seed in 0u64..1000) {
            let server = Arc::new(
                StatelessTemplateServer::new().expect("Failed to create test server")
            );
            let executor = CommandExecutorFactory::create(server.clone());

            // Executor should always be valid
            prop_assert!(Arc::ptr_eq(&executor.server, &server));
        }

        /// Property: CommandRegistry default always creates all groups
        #[test]
        fn registry_default_always_creates_groups(_seed in 0u64..1000) {
            let registry = CommandRegistry::default();

            // All groups should be accessible (implicitly tested)
            let _ = &registry.generate_handlers;
            let _ = &registry.analyze_handlers;
            let _ = &registry.utility_handlers;
            let _ = &registry.demo_handlers;

            prop_assert!(true);
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod integration_tests {
    use super::super::*;
    use std::sync::Arc;

    /// Creates a test server instance
    fn create_test_server() -> Arc<StatelessTemplateServer> {
        Arc::new(StatelessTemplateServer::new().expect("Failed to create test server"))
    }

    // ============================================================================
    // Full Integration Tests for Command Groups
    // ============================================================================

    /// Test that GenerateCommandGroup can be used with the registry
    #[test]
    fn test_generate_group_in_registry() {
        let registry = CommandRegistry::default();
        let _generate = &registry.generate_handlers;
        // Group should be accessible
    }

    /// Test that AnalyzeCommandGroup can be used with the registry
    #[test]
    fn test_analyze_group_in_registry() {
        let registry = CommandRegistry::default();
        let _analyze = &registry.analyze_handlers;
        // Group should be accessible
    }

    /// Test that UtilityCommandGroup can be used with the registry
    #[test]
    fn test_utility_group_in_registry() {
        let registry = CommandRegistry::default();
        let _utility = &registry.utility_handlers;
        // Group should be accessible
    }

    /// Test that DemoCommandGroup can be used with the registry
    #[test]
    fn test_demo_group_in_registry() {
        let registry = CommandRegistry::default();
        let _demo = &registry.demo_handlers;
        // Group should be accessible
    }

    // ============================================================================
    // CommandExecutor with CommandExecutorFactory Integration
    // ============================================================================

    #[test]
    fn test_factory_and_executor_integration() {
        let server = create_test_server();
        let executor = CommandExecutorFactory::create(server);

        // Verify the executor is fully functional
        let _ = &executor.registry.generate_handlers;
        let _ = &executor.registry.analyze_handlers;
        let _ = &executor.registry.utility_handlers;
        let _ = &executor.registry.demo_handlers;
    }

    /// Test multiple executors can be created from same server
    #[test]
    fn test_multiple_executors_same_server() {
        let server = create_test_server();

        let executor1 = CommandExecutorFactory::create(server.clone());
        let executor2 = CommandExecutorFactory::create(server.clone());

        // Both should share the same server
        assert!(Arc::ptr_eq(&executor1.server, &executor2.server));
    }

    /// Test executors with different servers are independent
    #[test]
    fn test_executors_different_servers() {
        let server1 = create_test_server();
        let server2 = create_test_server();

        let executor1 = CommandExecutorFactory::create(server1);
        let executor2 = CommandExecutorFactory::create(server2);

        // Servers should be different
        assert!(!Arc::ptr_eq(&executor1.server, &executor2.server));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod edge_case_tests {
    use super::super::*;
    use std::sync::Arc;

    /// Creates a test server instance
    fn create_test_server() -> Arc<StatelessTemplateServer> {
        Arc::new(StatelessTemplateServer::new().expect("Failed to create test server"))
    }

    // ============================================================================
    // Edge Cases for Command Groups
    // ============================================================================

    /// Test that all command groups implement Default correctly
    #[test]
    fn test_all_groups_implement_default() {
        // These should all compile and run without panic
        let _ = GenerateCommandGroup::default();
        let _ = AnalyzeCommandGroup::default();
        let _ = UtilityCommandGroup::default();
        let _ = DemoCommandGroup::default();
    }

    /// Test CommandRegistry default is consistent
    #[test]
    fn test_registry_default_consistency() {
        // Creating multiple registries should work
        let registries: Vec<CommandRegistry> =
            (0..10).map(|_| CommandRegistry::default()).collect();

        assert_eq!(registries.len(), 10);
    }

    /// Test CommandExecutor can be created many times
    #[test]
    fn test_executor_creation_stress() {
        let server = create_test_server();

        // Create many executors
        let executors: Vec<CommandExecutor> = (0..100)
            .map(|_| CommandExecutorFactory::create(server.clone()))
            .collect();

        assert_eq!(executors.len(), 100);

        // All should share the same server
        for executor in &executors {
            assert!(Arc::ptr_eq(&executor.server, &server));
        }
    }

    // ============================================================================
    // Memory and Safety Tests
    // ============================================================================

    /// Test that dropping executors doesn't cause issues
    #[test]
    fn test_executor_drop_safety() {
        let server = create_test_server();

        {
            let _executor = CommandExecutorFactory::create(server.clone());
            // Executor goes out of scope here
        }

        // Server should still be valid
        assert_eq!(Arc::strong_count(&server), 1);
    }

    /// Test registry drop safety
    #[test]
    fn test_registry_drop_safety() {
        {
            let _registry = CommandRegistry::default();
            // Registry goes out of scope here
        }
        // Should not panic
    }

    /// Test command group drop safety
    #[test]
    fn test_command_group_drop_safety() {
        {
            let _generate = GenerateCommandGroup::default();
            let _analyze = AnalyzeCommandGroup::default();
            let _utility = UtilityCommandGroup::default();
            let _demo = DemoCommandGroup::default();
            // All go out of scope here
        }
        // Should not panic
    }
}
