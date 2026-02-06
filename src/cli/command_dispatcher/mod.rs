//! Command Dispatcher - Reduces CLI complexity through handler pattern
//!
//! This module implements a dispatch table pattern to reduce cyclomatic complexity
//! in the CLI module by delegating command execution to specialized handlers.
#![cfg_attr(coverage_nightly, coverage(off))]

use super::commands::QddCommands;
use super::{AnalyzeCommands, Commands, OutputFormat, RefactorCommands};
use crate::cli::handlers;
use crate::cli::handlers::cache::CacheCommand;
use crate::cli::handlers::memory::MemoryCommand;
use crate::stateless_server::StatelessTemplateServer;
use std::sync::Arc;

// Extracted modules for CB-040 file health compliance
mod config_commands;
#[cfg(feature = "demo")]
mod demo_commands;
mod metrics_commands;
mod quality_commands;
mod roadmap_commands;
mod scaffold_commands;
mod semantic_commands;
mod test_commands;

// Work and spec handlers extracted for file health compliance (CB-040)
#[path = "command_dispatcher_work.rs"]
mod command_dispatcher_work;

/// Trait for command handlers to reduce complexity through delegation
#[allow(dead_code)]
#[allow(async_fn_in_trait)]
pub trait CommandHandler: Send + Sync {
    async fn execute(&self, server: Arc<StatelessTemplateServer>) -> anyhow::Result<()>;
}

/// Trait for analyze command handlers
#[allow(dead_code)]
#[allow(async_fn_in_trait)]
pub trait AnalyzeCommandHandler: Send + Sync {
    async fn execute(&self) -> anyhow::Result<()>;
}

/// Command dispatcher that reduces complexity by delegating to handlers
pub struct CommandDispatcher;

impl CommandDispatcher {
    /// Execute a command using the handler pattern (reduces CC from dispatch match)
    pub async fn execute_command(
        command: Commands,
        server: Arc<StatelessTemplateServer>,
    ) -> anyhow::Result<()> {
        Self::route_command(command, server).await
    }

    /// Route commands to appropriate handlers (reduces complexity)
    async fn route_command(
        command: Commands,
        server: Arc<StatelessTemplateServer>,
    ) -> anyhow::Result<()> {
        match command {
            Commands::Generate {
                category,
                template,
                params,
                output,
                create_dirs,
            } => {
                handlers::handle_generate(server, category, template, params, output, create_dirs)
                    .await
            }
            Commands::Scaffold { command } => Self::execute_scaffold_command(command, server).await,
            Commands::List {
                toolchain,
                category,
                format,
            } => handlers::handle_list(server, toolchain, category, format).await,
            Commands::Search {
                query,
                toolchain,
                limit,
            } => handlers::handle_search(server, query, toolchain, limit).await,
            Commands::Validate { uri, params } => {
                handlers::handle_validate(server, uri, params).await
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
                handlers::handle_context(
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
                definition_type,
                summary,
                git_history,
                regex,
                literal,
                case_sensitive,
                ignore_case,
                exclude,
                exclude_file,
                files_with_matches,
                count,
                after_context,
                before_context,
                context_lines,
            } => {
                // Default is to show code; --summary disables it
                let show_code = !summary;
                handlers::handle_query(
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
                    definition_type,
                    show_code,
                    git_history,
                    regex,
                    literal,
                    case_sensitive,
                    ignore_case,
                    exclude,
                    exclude_file,
                    files_with_matches,
                    count,
                    after_context,
                    before_context,
                    context_lines,
                )
                .await
            }
            Commands::Analyze(analyze_cmd) => Self::execute_analyze_command(analyze_cmd).await,
            Commands::Qdd(qdd_cmd) => Self::execute_qdd_command(qdd_cmd).await,
            Commands::Embed(embed_cmd) => Self::execute_embed_command(embed_cmd).await,
            Commands::Semantic(semantic_cmd) => Self::execute_semantic_command(semantic_cmd).await,
            #[cfg(feature = "demo")]
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
                Self::execute_demo_command(
                    path,
                    url,
                    repo,
                    Some(format),
                    protocol,
                    show_api,
                    no_browser,
                    port.unwrap_or(8080),
                    cli,
                    Some(target_nodes),
                    Some(centrality_threshold),
                    Some(merge_threshold as f64),
                    debug,
                    debug_output,
                    skip_vendor,
                    no_skip_vendor,
                    max_line_length,
                    server,
                )
                .await
            }
            #[cfg(not(feature = "demo"))]
            Commands::Demo { .. } => {
                anyhow::bail!("Demo feature not enabled. Build with --features demo")
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
                    handlers::handle_org_command(_org_cmd).await
                }
                #[cfg(not(feature = "org-intelligence"))]
                {
                    anyhow::bail!("Organizational intelligence feature is not enabled. Rebuild with --features org-intelligence")
                }
            }
            Commands::Prompt(prompt_cmd) => handlers::handle_prompt_command(prompt_cmd).await,
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
                // Convert QualityGateOutputFormat to OutputFormat for the internal method
                let output_format = match format {
                    crate::cli::enums::QualityGateOutputFormat::Json => OutputFormat::Json,
                    _ => OutputFormat::Table,
                };

                // Convert QualityCheckType vec to String vec
                let check_strings: Vec<String> = checks
                    .iter()
                    .map(|c| format!("{c:?}").to_lowercase())
                    .collect();

                Self::execute_quality_gate_command(
                    Some(project_path),
                    file,
                    output_format,
                    fail_on_violation,
                    check_strings,
                    Some(max_dead_code),
                    Some(min_entropy),
                    Some(max_complexity_p99 as usize),
                    include_provability,
                    output,
                    perf,
                )
                .await
            }
            Commands::Report {
                project_path,
                output_format,
                include_visualizations,
                include_executive_summary,
                include_recommendations,
                analyses,
                confidence_threshold,
                output,
                perf,
                text,
                markdown,
                csv,
            } => {
                // Convert ReportOutputFormat to OutputFormat for the internal method
                let internal_format = match output_format {
                    crate::cli::enums::ReportOutputFormat::Json => OutputFormat::Json,
                    _ => OutputFormat::Table,
                };

                // Convert AnalysisType vec to String vec
                let analysis_strings: Vec<String> = analyses
                    .iter()
                    .map(|a| format!("{a:?}").to_lowercase())
                    .collect();

                Self::execute_report_command(
                    Some(project_path),
                    internal_format,
                    include_visualizations,
                    include_executive_summary,
                    include_recommendations,
                    analysis_strings,
                    Some(f64::from(confidence_threshold) / 100.0),
                    output,
                    perf,
                    text,
                    markdown,
                    csv,
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
                handlers::handle_repo_score(
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
                handlers::handle_rust_project_score(
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
                handlers::handle_brick_score(
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
                handlers::handle_popper_score(
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
                handlers::handle_demo_score(
                    &path,
                    &format,
                    verbose,
                    failures_only,
                    output.as_deref(),
                )
                .await
            }
            Commands::Serve {
                port,
                host,
                cors,
                transport,
            } => handlers::handle_serve(host, port, cors, transport).await,
            Commands::Diagnose(args) => super::diagnose::handle_diagnose(args).await,
            Commands::Enforce(enforce_cmd) => handlers::route_enforce_command(enforce_cmd).await,
            Commands::Refactor(refactor_cmd) => Self::execute_refactor_command(refactor_cmd).await,
            Commands::Roadmap(roadmap_cmd) => Self::execute_roadmap_command(roadmap_cmd).await,
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
                Self::execute_test_command(
                    suite, iterations, memory, throughput, regression, timeout, output, perf,
                )
                .await
            }
            Commands::Memory { command } => Self::execute_memory_command(command).await,
            Commands::Cache { command } => Self::execute_cache_command(command).await,
            Commands::Telemetry {
                system,
                service,
                reset,
                test_event,
            } => {
                handlers::telemetry_handlers::handle_telemetry(system, service, reset, test_event)
                    .await
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
                Self::execute_config_command(
                    show,
                    edit,
                    validate,
                    reset,
                    section,
                    if set.is_empty() { None } else { Some(set) },
                    config_path,
                )
                .await
            }

            Commands::ShowMetrics {
                trend,
                days,
                metric,
                format,
                failures_only,
            } => {
                Self::execute_show_metrics_command(trend, days, metric, format, failures_only).await
            }

            Commands::PredictQuality {
                metric,
                threshold,
                days,
                format,
                all,
                failures_only,
            } => {
                handlers::predict_quality_handlers::handle_predict_quality(
                    metric,
                    threshold,
                    days,
                    format,
                    all,
                    failures_only,
                )
                .await
            }

            Commands::RecordMetric {
                metric,
                value,
                timestamp,
            } => Self::execute_record_metric_command(metric, value, timestamp).await,

            #[cfg(feature = "agent-daemon")]
            Commands::Agent { command } => handlers::handle_agent_command(command).await,
            #[cfg(not(feature = "agent-daemon"))]
            Commands::Agent { .. } => {
                anyhow::bail!("Agent daemon feature not enabled. Build with --features agent-daemon")
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
                let tdg_config = handlers::tdg_handlers::TdgCommandConfig {
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
                handlers::handle_tdg_command(tdg_config).await
            }

            Commands::QualityGates {
                command,
                config,
                report,
                json,
                project_dir,
            } => {
                handlers::handle_quality_gates_command(command, config, report, json, project_dir)
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
                        let config = handlers::roadmap_handler::RoadmapMaintenanceConfig::new(
                            validate,
                            health,
                            fix,
                            generate_tickets,
                            dry_run,
                        );
                        handlers::handle_maintain_roadmap(roadmap, tickets_dir, config, format)
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
                        let config = handlers::health_handler::HealthCheckConfig::new(
                            quick,
                            all,
                            check_build,
                            check_tests,
                            check_coverage,
                            check_complexity,
                            check_satd,
                        );
                        handlers::handle_maintain_health(project_dir, format, config).await
                    }
                    MaintainCommands::BugReport {
                        title,
                        dry_run,
                        interactive,
                        clear,
                    } => {
                        handlers::bug_report_handler::handle_bug_report(
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
                        handlers::cleanup_resources_handler::handle_cleanup_resources(
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

            Commands::Hooks(hooks_cmd) => handlers::handle_hooks_command(&hooks_cmd).await,

            #[cfg(feature = "mutation-testing")]
            Commands::Mutate(args) => handlers::mutate::handle(args, server).await,

            Commands::Debug { command } => {
                // Sprint 74: Time-travel debugging commands
                // TODO: Implement debug handlers in DEBUG-002 and DEBUG-003
                use crate::cli::commands::DebugCommands;
                match command {
                    DebugCommands::Serve {
                        port,
                        host,
                        record_dir,
                    } => {
                        anyhow::bail!("Debug serve command not yet implemented (DEBUG-002). Port: {}, Host: {}, Record Dir: {:?}", port, host, record_dir)
                    }
                    DebugCommands::Replay {
                        recording,
                        position,
                        interactive,
                    } => {
                        anyhow::bail!("Debug replay command not yet implemented (DEBUG-003). Recording: {:?}, Position: {:?}, Interactive: {}", recording, position, interactive)
                    }
                }
            }
            Commands::Work { command } => {
                // Issue #75: Unified GitHub/YAML workflow
                Self::execute_work_command(&command).await
            }
            Commands::QaWork { command } => {
                // GH-102: Toyota Way quality validation
                handlers::qa_work_handler::handle_qa_work_command(command).await
            }
            Commands::Comply { command } => {
                // GH-96: PMAT compliance and migration system
                handlers::comply_handlers::handle_comply_command(command).await
            }

            Commands::ProjectDiag {
                path,
                format,
                category,
                failures_only,
                output,
                quiet,
            } => {
                // Project diagnostics (lltop Tab 8 equivalent)
                let config = handlers::project_diag_handlers::ProjectDiagConfig {
                    path,
                    format,
                    category,
                    failures_only,
                    output,
                    quiet,
                };
                handlers::project_diag_handlers::handle_project_diag(config).await
            }

            Commands::TestDiscovery { command } => {
                // GH-98: Systematic test discovery and fixing
                handlers::test_discovery_handlers::handle_test_discovery_command(command).await
            }

            Commands::DebugFiveWhys {
                issue,
                depth,
                format,
                output,
                path,
                context,
                auto_analyze,
            } => {
                // Five Whys root cause analysis (Toyota Way)
                crate::cli::handlers::five_whys_handlers::handle_debug(
                    &issue,
                    depth,
                    format,
                    output.as_deref(),
                    &path,
                    context.as_deref(),
                    auto_analyze,
                )
                .await
            }

            Commands::Oracle { command } => {
                // PMAT Oracle - PDCA loop for automated quality improvement (Toyota Way)
                crate::cli::handlers::oracle_handlers::handle_oracle_command(command).await
            }

            Commands::PerfectionScore {
                path,
                breakdown,
                target,
                format,
                output,
                fast,
            } => {
                // Unified 200-point Perfection Score (master-plan-pmat-work-system.md)
                handlers::perfection_score_handlers::handle_perfection_score(
                    &path,
                    breakdown,
                    target,
                    format,
                    output.as_deref(),
                    fast,
                )
                .await
            }

            Commands::Spec { command } => {
                // Specification management (master-plan-pmat-work-system.md)
                Self::handle_spec_command(command).await
            }

            Commands::Localize {
                passed_coverage,
                failed_coverage,
                passed_count,
                failed_count,
                formula,
                top_n,
                output,
                format,
            } => {
                // Fault localization using Tarantula SBFL (GH-103)
                crate::cli::handlers::localize_handlers::handle_localize(
                    &passed_coverage,
                    &failed_coverage,
                    passed_count,
                    failed_count,
                    &formula,
                    top_n,
                    output.as_deref(),
                    &format,
                )
                .await
            }

            Commands::CudaTdg {
                path,
                command,
                format,
                min_score,
                fail_on_p0,
                simd,
                wgpu,
                output,
                quiet,
            } => {
                // CUDA-SIMD TDG: 100-point Popper falsification scoring
                let config = handlers::CudaTdgCommandConfig {
                    path,
                    command,
                    format,
                    min_score,
                    fail_on_p0,
                    simd,
                    wgpu,
                    output,
                    quiet,
                };
                handlers::handle_cuda_tdg_command(config).await
            }

            Commands::DepsAudit {
                path,
                format,
                all,
                pareto,
                sort_by,
            } => {
                // Dependency audit for Sovereign AI stack migration
                handlers::deps_audit_handlers::handle_deps_audit(
                    &path, &format, all, pareto, &sort_by,
                )
            }
        }
    }

    /// Execute analyze commands using handler pattern (reduces CC)
    pub async fn execute_analyze_command(analyze_cmd: AnalyzeCommands) -> anyhow::Result<()> {
        // Delegate to the modular analysis handlers
        super::handlers::route_analyze_command(analyze_cmd).await
    }

    /// Execute QDD commands using handler pattern (reduces CC)
    pub async fn execute_qdd_command(qdd_cmd: QddCommands) -> anyhow::Result<()> {
        // Delegate to the QDD handlers
        super::handlers::qdd_handlers::handle_qdd_command(qdd_cmd).await
    }

    /// Execute refactor commands using handler pattern (reduces CC)
    pub async fn execute_refactor_command(refactor_cmd: RefactorCommands) -> anyhow::Result<()> {
        // Delegate to the refactor handlers
        super::handlers::route_refactor_command(refactor_cmd).await
    }

    /// Execute memory management commands using handler pattern (reduces CC)
    pub async fn execute_memory_command(memory_cmd: MemoryCommand) -> anyhow::Result<()> {
        // Delegate to the memory handler
        super::handlers::handle_memory_command(&memory_cmd).await
    }

    /// Execute cache management commands using handler pattern (reduces CC)
    pub async fn execute_cache_command(cache_cmd: CacheCommand) -> anyhow::Result<()> {
        // Delegate to the cache handler
        super::handlers::handle_cache_command(&cache_cmd).await
    }
}

// Tests extracted for file health compliance (CB-040)
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
