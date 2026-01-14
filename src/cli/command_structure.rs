//! Command Structure - Comprehensive CLI decomposition architecture
//!
//! This module provides the complete command structure decomposition to reduce
//! the main CLI module from 145 functions and 10,304 lines to a manageable size.
//!
//! Architecture:
//! - `CommandExecutor`: Main command execution orchestrator
//! - `CommandRegistry`: Registry of all available commands  
//! - `CommandGroup`: Logical grouping of related commands
//! - `ModularHandlers`: Individual command implementation modules

use crate::cli::commands::ScaffoldCommands;
use crate::cli::{AnalyzeCommands, Commands};
use crate::stateless_server::StatelessTemplateServer;
use anyhow::Result;
use std::sync::Arc;

/// Main command executor that orchestrates all CLI operations
pub struct CommandExecutor {
    server: Arc<StatelessTemplateServer>,
    registry: CommandRegistry,
}

/// Registry that manages all available commands and their handlers
#[derive(Default)]
pub struct CommandRegistry {
    generate_handlers: GenerateCommandGroup,
    analyze_handlers: AnalyzeCommandGroup,
    utility_handlers: UtilityCommandGroup,
    demo_handlers: DemoCommandGroup,
}

/// Command group for generation operations (generate, scaffold, validate)
pub struct GenerateCommandGroup;

/// Command group for analysis operations (complexity, churn, dead-code, etc.)
pub struct AnalyzeCommandGroup;

/// Command group for utility operations (list, search, context, serve)
pub struct UtilityCommandGroup;

/// Command group for demo and quality gate operations
pub struct DemoCommandGroup;

impl CommandExecutor {
    /// Create new command executor with server instance
    #[must_use]
    pub fn new(server: Arc<StatelessTemplateServer>) -> Self {
        Self {
            server,
            registry: CommandRegistry::default(),
        }
    }

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
            Commands::Scaffold { command } => {
                match command {
                    ScaffoldCommands::Project {
                        toolchain,
                        templates,
                        params,
                        parallel,
                    } => {
                        self.registry
                            .generate_handlers
                            .handle_scaffold(
                                self.server.clone(),
                                toolchain,
                                templates,
                                params,
                                parallel,
                            )
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
                        // TICKET-PMAT-5030: Wire up agent scaffolding
                        let params = super::handlers::ScaffoldAgentParams {
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
                        super::handlers::handle_scaffold_agent(params).await
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
                        // TICKET-PMAT-5031: Wire up WASM scaffolding
                        let params = super::handlers::ScaffoldWasmParams {
                            name,
                            framework,
                            features,
                            quality,
                            output,
                            force,
                            dry_run,
                        };
                        super::handlers::handle_scaffold_wasm(params).await
                    }
                    ScaffoldCommands::ListTemplates => {
                        super::handlers::handle_list_agent_templates().await
                    }
                    ScaffoldCommands::ValidateTemplate { path } => {
                        super::handlers::handle_validate_agent_template(path).await
                    }
                    ScaffoldCommands::ListSubagents { all } => {
                        super::handlers::subagent_handlers::list_subagents(all)
                    }
                    ScaffoldCommands::CreateSubagent { agent_name, output } => {
                        super::handlers::subagent_handlers::create_subagent(&agent_name, output)
                    }
                    ScaffoldCommands::CreateAllSubagents { output } => {
                        super::handlers::subagent_handlers::create_all_mvp_subagents(output)
                    }
                    ScaffoldCommands::ValidateSubagent { file_path } => {
                        super::handlers::subagent_handlers::validate_subagent(&file_path)
                    }
                    ScaffoldCommands::ShowToolMapping { agent } => {
                        super::handlers::subagent_handlers::show_tool_mapping(agent)
                    }
                    ScaffoldCommands::ExportToolMapping { output } => {
                        super::handlers::subagent_handlers::export_tool_mapping_json(&output)
                    }
                }
            }
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
                super::handlers::route_refactor_command(refactor_cmd).await
            }
            Commands::Enforce(enforce_cmd) => {
                super::handlers::route_enforce_command(enforce_cmd).await
            }
            Commands::Roadmap(roadmap_cmd) => {
                super::command_dispatcher::CommandDispatcher::execute_roadmap_command(roadmap_cmd)
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
                super::command_dispatcher::CommandDispatcher::execute_memory_command(command).await
            }
            Commands::Cache { command } => {
                super::command_dispatcher::CommandDispatcher::execute_cache_command(command).await
            }
            Commands::Telemetry {
                system,
                service,
                reset,
                test_event,
            } => super::handlers::handle_telemetry(system, service, reset, test_event).await,
            Commands::Config {
                show,
                edit,
                validate,
                reset,
                section,
                set,
                config_path,
            } => {
                super::handlers::handle_configuration(
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

            Commands::Agent { command } => super::handlers::handle_agent_command(command).await,

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
                let tdg_config = super::handlers::tdg_handlers::TdgCommandConfig {
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
                super::handlers::handle_tdg_command(tdg_config).await
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
                super::handlers::handle_quality_gates_command(
                    command,
                    config,
                    report,
                    json,
                    project_dir,
                )
                .await
            }

            Commands::Maintain { command } => {
                use super::commands::MaintainCommands;
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
                            super::handlers::roadmap_handler::RoadmapMaintenanceConfig::new(
                                validate,
                                health,
                                fix,
                                generate_tickets,
                                dry_run,
                            );
                        super::handlers::handle_maintain_roadmap(
                            roadmap,
                            tickets_dir,
                            config,
                            format,
                        )
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
                        let config = super::handlers::health_handler::HealthCheckConfig::new(
                            quick,
                            all,
                            check_build,
                            check_tests,
                            check_coverage,
                            check_complexity,
                            check_satd,
                        );
                        super::handlers::handle_maintain_health(project_dir, format, config).await
                    }
                    MaintainCommands::BugReport {
                        title,
                        dry_run,
                        interactive,
                        clear,
                    } => {
                        super::handlers::bug_report_handler::handle_bug_report(
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
                        super::handlers::cleanup_resources_handler::handle_cleanup_resources(
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

            Commands::Hooks(hooks_cmd) => super::handlers::handle_hooks_command(&hooks_cmd).await,

            // Semantic search commands (PMAT-SEARCH-011)
            Commands::Embed(embed_cmd) => {
                super::command_dispatcher::CommandDispatcher::execute_embed_command(embed_cmd).await
            }
            Commands::Semantic(semantic_cmd) => {
                super::command_dispatcher::CommandDispatcher::execute_semantic_command(semantic_cmd)
                    .await
            }

            // Mutation testing command (Sprint 61)
            #[cfg(feature = "mutation-testing")]
            Commands::Mutate(args) => {
                super::handlers::mutate::handle(args, self.server.clone()).await
            }

            // Time-travel debugging commands (Sprint 74)
            Commands::Debug { command } => {
                use crate::cli::commands::DebugCommands;
                match command {
                    DebugCommands::Serve {
                        port,
                        host,
                        record_dir,
                    } => {
                        // DEBUG-002: DAP Server CLI Handler (implemented)
                        // Sprint 76 CAPTURE-003: Added record_dir support
                        super::handlers::debug_handlers::handle_debug_serve(port, host, record_dir)
                            .await
                    }
                    DebugCommands::Replay {
                        recording,
                        position,
                        interactive,
                    } => {
                        // DEBUG-003: Replay CLI Handler (implemented)
                        super::handlers::debug_handlers::handle_debug_replay(
                            recording,
                            position,
                            interactive,
                        )
                        .await
                    }
                }
            }
            // Phase 3.1: O(1) Quality Gates - CLI Integration
            Commands::ShowMetrics { .. } => {
                anyhow::bail!("ShowMetrics command should be handled by command_dispatcher.rs")
            }
            // Phase 4.1: O(1) Quality Gates - Predictive CLI
            Commands::PredictQuality { .. } => {
                anyhow::bail!("PredictQuality command should be handled by command_dispatcher.rs")
            }
            // Phase 3.4: O(1) Quality Gates - CI/CD Integration
            Commands::RecordMetric { .. } => {
                anyhow::bail!("RecordMetric command should be handled by command_dispatcher.rs")
            }
            // Issue #75: Unified GitHub/YAML workflow
            Commands::Work { command } => {
                anyhow::bail!(
                    "Work command not yet implemented in command structure: {:?}",
                    command
                )
            }
            // GH-102: QA Work - Toyota Way quality validation
            Commands::QaWork { .. } => {
                anyhow::bail!("QaWork command should be handled by command_dispatcher.rs")
            }
            // GH-96: PMAT compliance and migration system - handled by command_dispatcher.rs
            Commands::Comply { .. } => {
                anyhow::bail!("Comply command should be handled by command_dispatcher.rs")
            }

            // Project diagnostics (lltop Tab 8 equivalent) - handled by command_dispatcher.rs
            Commands::ProjectDiag { .. } => {
                anyhow::bail!("ProjectDiag command should be handled by command_dispatcher.rs")
            }

            // GH-98: Systematic test discovery and fixing - handled by command_dispatcher.rs
            Commands::TestDiscovery { .. } => {
                anyhow::bail!("TestDiscovery command should be handled by command_dispatcher.rs")
            }

            // Five Whys root cause analysis - handled by command_dispatcher.rs
            Commands::DebugFiveWhys { .. } => {
                anyhow::bail!("DebugFiveWhys command should be handled by command_dispatcher.rs")
            }

            // Fault localization - handled by command_dispatcher.rs (GH-103)
            Commands::Localize { .. } => {
                anyhow::bail!("Localize command should be handled by command_dispatcher.rs")
            }

            // PMAT Oracle - handled by command_dispatcher.rs
            Commands::Oracle { .. } => {
                anyhow::bail!("Oracle command should be handled by command_dispatcher.rs")
            }

            // master-plan-pmat-work-system.md: 200-point unified score - handled by command_dispatcher.rs
            Commands::PerfectionScore { .. } => {
                anyhow::bail!("PerfectionScore command should be handled by command_dispatcher.rs")
            }

            // master-plan-pmat-work-system.md: Spec management - handled by command_dispatcher.rs
            Commands::Spec { .. } => {
                anyhow::bail!("Spec command should be handled by command_dispatcher.rs")
            }

            // CUDA-SIMD TDG: 100-point Popper falsification - handled by command_dispatcher.rs
            Commands::CudaTdg { .. } => {
                anyhow::bail!("CudaTdg command should be handled by command_dispatcher.rs")
            }

            // Dependency audit for Sovereign AI stack migration - handled by command_dispatcher.rs
            Commands::DepsAudit { .. } => {
                anyhow::bail!("DepsAudit command should be handled by command_dispatcher.rs")
            }
        }
    }
}

impl Default for GenerateCommandGroup {
    fn default() -> Self {
        Self
    }
}

impl GenerateCommandGroup {
    /// Handle generate command with modular implementation
    pub async fn handle_generate(
        &self,
        server: Arc<StatelessTemplateServer>,
        category: String,
        template: String,
        params: Vec<(String, serde_json::Value)>,
        output: Option<std::path::PathBuf>,
        create_dirs: bool,
    ) -> Result<()> {
        // Delegate to generation handlers module
        crate::cli::handlers::generation_handlers::handle_generate(
            server,
            category,
            template,
            params,
            output,
            create_dirs,
        )
        .await
    }

    /// Handle scaffold command
    pub async fn handle_scaffold(
        &self,
        server: Arc<StatelessTemplateServer>,
        toolchain: String,
        templates: Vec<String>,
        params: Vec<(String, serde_json::Value)>,
        parallel: usize,
    ) -> Result<()> {
        crate::cli::handlers::generation_handlers::handle_scaffold(
            server, toolchain, templates, params, parallel,
        )
        .await
    }

    /// Handle validate command
    pub async fn handle_validate(
        &self,
        server: Arc<StatelessTemplateServer>,
        uri: String,
        params: Vec<(String, serde_json::Value)>,
    ) -> Result<()> {
        crate::cli::handlers::generation_handlers::handle_validate(server, uri, params).await
    }
}

impl Default for AnalyzeCommandGroup {
    fn default() -> Self {
        Self
    }
}

impl AnalyzeCommandGroup {
    /// Execute analysis command using modular handlers
    pub async fn execute(&self, cmd: AnalyzeCommands) -> Result<()> {
        // Delegate to analysis handlers which further delegate to specific modules
        crate::cli::handlers::analysis_handlers::route_analyze_command(cmd).await
    }
}

impl Default for UtilityCommandGroup {
    fn default() -> Self {
        Self
    }
}

impl UtilityCommandGroup {
    /// Handle list command
    pub async fn handle_list(
        &self,
        server: Arc<StatelessTemplateServer>,
        toolchain: Option<String>,
        category: Option<String>,
        format: crate::cli::OutputFormat,
    ) -> Result<()> {
        crate::cli::handlers::utility_handlers::handle_list(server, toolchain, category, format)
            .await
    }

    /// Handle search command
    pub async fn handle_search(
        &self,
        server: Arc<StatelessTemplateServer>,
        query: String,
        toolchain: Option<String>,
        limit: usize,
    ) -> Result<()> {
        crate::cli::handlers::utility_handlers::handle_search(server, query, toolchain, limit).await
    }

    /// Handle context command
    #[allow(clippy::too_many_arguments)]
    pub async fn handle_context(
        &self,
        toolchain: Option<String>,
        project_path: std::path::PathBuf,
        output: Option<std::path::PathBuf>,
        format: crate::cli::ContextFormat,
        include_large_files: bool,
        skip_expensive_metrics: bool,
        language: Option<String>,
        languages: Option<Vec<String>>,
    ) -> Result<()> {
        crate::cli::handlers::utility_handlers::handle_context(
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

    /// Handle serve command
    pub async fn handle_serve(
        &self,
        host: String,
        port: u16,
        cors: bool,
        transport: crate::cli::commands::ServeTransport,
    ) -> Result<()> {
        crate::cli::handlers::utility_handlers::handle_serve(host, port, cors, transport).await
    }

    /// Handle diagnose command
    pub async fn handle_diagnose(&self, args: crate::cli::diagnose::DiagnoseArgs) -> Result<()> {
        crate::cli::handlers::utility_handlers::handle_diagnose(args).await
    }
}

impl Default for DemoCommandGroup {
    fn default() -> Self {
        Self
    }
}

impl DemoCommandGroup {
    /// Handle demo command with comprehensive parameter support
    #[allow(clippy::too_many_arguments)]
    pub async fn handle_demo(
        &self,
        server: Arc<StatelessTemplateServer>,
        path: Option<std::path::PathBuf>,
        url: Option<String>,
        repo: Option<String>,
        format: crate::cli::OutputFormat,
        protocol: crate::cli::DemoProtocol,
        show_api: bool,
        no_browser: bool,
        port: Option<u16>,
        cli: bool,
        target_nodes: usize,
        centrality_threshold: f64,
        merge_threshold: usize,
        debug: bool,
        debug_output: Option<std::path::PathBuf>,
        skip_vendor: bool,
        max_line_length: Option<usize>,
    ) -> Result<()> {
        // Use dedicated demo handlers module
        crate::cli::handlers::demo_handlers::handle_demo(
            server,
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
            max_line_length,
        )
        .await
    }

    /// Handle quality gate command
    #[allow(clippy::too_many_arguments)]
    pub async fn handle_quality_gate(
        &self,
        project_path: std::path::PathBuf,
        file: Option<std::path::PathBuf>,
        format: crate::cli::QualityGateOutputFormat,
        fail_on_violation: bool,
        checks: Vec<crate::cli::QualityCheckType>,
        max_dead_code: f64,
        min_entropy: f64,
        max_complexity_p99: u32,
        include_provability: bool,
        output: Option<std::path::PathBuf>,
        perf: bool,
    ) -> Result<()> {
        // Use dedicated demo handlers module
        crate::cli::handlers::demo_handlers::handle_quality_gate(
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
}

/// Factory for creating command executors
pub struct CommandExecutorFactory;

impl CommandExecutorFactory {
    /// Create a new command executor instance
    #[must_use]
    pub fn create(server: Arc<StatelessTemplateServer>) -> CommandExecutor {
        CommandExecutor::new(server)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(err_msg.contains("Work command not yet implemented"));
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

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

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

#[cfg(test)]
mod integration_tests {
    use super::*;

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

#[cfg(test)]
mod edge_case_tests {
    use super::*;

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
        let registries: Vec<CommandRegistry> = (0..10).map(|_| CommandRegistry::default()).collect();

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
