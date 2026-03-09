#![cfg_attr(coverage_nightly, coverage(off))]
//! Extended dispatch: infrastructure, tools, and configuration commands

use super::super::CommandExecutor;
use crate::cli::Commands;
use anyhow::Result;

impl CommandExecutor {
    /// Execute infrastructure, tool, and configuration commands
    pub(super) async fn execute_ext_tools(&self, command: Commands) -> Result<()> {
        match command {
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
                record,
                from_stdin,
                dry_run,
            } => {
                if record {
                    crate::cli::command_dispatcher::test_record::execute_test_record(from_stdin, dry_run).await
                } else {
                    crate::cli::handlers::test_handlers::handle_test(
                        suite, iterations, memory, throughput, regression, timeout, output, perf,
                    )
                    .await
                }
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
            } => crate::cli::handlers::handle_telemetry(system, service, reset, test_event).await,
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
            _ => unreachable!("execute_ext_tools called with non-tools command"),
        }
    }
}
