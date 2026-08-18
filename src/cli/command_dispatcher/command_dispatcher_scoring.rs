//! Scoring, infrastructure, and configuration command handlers
//!
//! Extracted from command_dispatcher.rs for cognitive complexity reduction.
#![cfg_attr(coverage_nightly, coverage(off))]

use super::CommandDispatcher;
use crate::cli::commands::Commands;
use crate::cli::handlers;
use std::sync::Arc;

impl CommandDispatcher {
    /// Route scoring and reporting commands (extracted to reduce route_command cognitive complexity)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) async fn route_scoring_command(command: Commands) -> anyhow::Result<()> {
        match command {
            Commands::QualityGate {
                project_path,
                file,
                format,
                report_only,
                // A no-op since 3.32.0 — the gate exits non-zero on blocking
                // violations by default — and its help text says so. Still
                // parsed so existing `--fail-on-violation` callers keep working.
                fail_on_violation: _,
                checks,
                max_dead_code,
                min_entropy,
                max_complexity_p99,
                include_provability,
                output,
                perf,
            } => {
                // Pass QualityGateOutputFormat directly to preserve Junit/Markdown (#230)
                crate::cli::analysis_utilities::handle_quality_gate(
                    project_path,
                    file,
                    format,
                    crate::cli::analysis_utilities::gate_exits_on_violation(report_only),
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
                // Pass ReportOutputFormat straight through. It used to be
                // squeezed into OutputFormat (everything non-JSON -> Table) and
                // then widened back to ReportOutputFormat::Text, so
                // `pmat report --output-format markdown` silently produced a
                // plain-text report — a declared --format that did not do what
                // it says. Verified: `-f markdown` emitted the Text renderer.
                //
                // #706: that fix inlined the 12-argument `handle_generate_report`
                // call here and in dispatch_ext_scoring.rs, which orphaned
                // `execute_report_command` — the wrapper #672's format-fidelity
                // tests exercise. Route through the wrapper so the tested path
                // is the one production runs.
                Self::execute_report_command(
                    Some(project_path),
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
                )
                .await
            }
            Commands::Score {
                path,
                gate,
                format,
                output,
                trend,
                regression_check,
                stack,
            } => {
                handlers::handle_score(
                    &path,
                    gate,
                    &format,
                    output.as_deref(),
                    trend,
                    regression_check,
                    stack,
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
            Commands::InfraScore {
                path,
                format,
                verbose,
                failures_only,
                output,
            } => {
                handlers::handle_infra_score(
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
            _ => unreachable!("route_scoring_command called with non-scoring command"),
        }
    }

    /// Route infrastructure and configuration commands
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) async fn route_infra_command(
        command: Commands,
        _server: Arc<crate::stateless_server::StatelessTemplateServer>,
    ) -> anyhow::Result<()> {
        match command {
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
                ml: _,
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
            Commands::Maintain { command } => Self::route_maintain_command(command).await,
            Commands::Hooks(hooks_cmd) => handlers::handle_hooks_command(&hooks_cmd).await,
            // Delegates instead of bailing inline. The inline `bail!`s surfaced
            // as exit 1 while the sibling executor path (debug_exec.rs) went
            // through the handlers, so the same command exited 1 or 2 depending
            // on which dispatcher served it. One route, one exit code.
            Commands::Debug { command } => {
                use crate::cli::commands::DebugCommands;
                match command {
                    DebugCommands::Serve {
                        port,
                        host,
                        record_dir,
                    } => handlers::debug_handlers::handle_debug_serve(port, host, record_dir).await,
                    DebugCommands::Replay {
                        recording,
                        position,
                        interactive,
                    } => {
                        handlers::debug_handlers::handle_debug_replay(
                            recording,
                            position,
                            interactive,
                        )
                        .await
                    }
                }
            }
            _ => unreachable!("route_infra_command called with non-infra command"),
        }
    }

    /// Route maintain subcommands
    async fn route_maintain_command(
        command: crate::cli::commands::MaintainCommands,
    ) -> anyhow::Result<()> {
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
                let config = handlers::roadmap_handler::RoadmapMaintenanceConfig::new(
                    validate,
                    health,
                    fix,
                    generate_tickets,
                    dry_run,
                );
                handlers::handle_maintain_roadmap(roadmap, tickets_dir, config, format).await
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
                dry_run: _,
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
}
