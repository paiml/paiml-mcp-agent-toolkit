#![cfg_attr(coverage_nightly, coverage(off))]
//! Extended command dispatch - demo, quality, scoring, and other commands

use super::super::CommandExecutor;
use crate::cli::Commands;
use anyhow::Result;

impl CommandExecutor {
    /// Execute extended commands (demo, quality, scoring, maintain, debug, etc.)
    pub(super) async fn execute_extended(&self, command: Commands) -> Result<()> {
        match command {
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
}
