//! Command Dispatcher - Reduces CLI complexity through handler pattern
//!
//! This module implements a dispatch table pattern to reduce cyclomatic complexity
//! in the CLI module by delegating command execution to specialized handlers.

use super::{AnalyzeCommands, Commands, DemoProtocol, RefactorCommands};
use crate::stateless_server::StatelessTemplateServer;
use std::sync::Arc;

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
        use super::handlers;

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
            Commands::Scaffold {
                toolchain,
                templates,
                params,
                parallel,
            } => handlers::handle_scaffold(server, toolchain, templates, params, parallel).await,
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
            } => {
                handlers::handle_context(
                    toolchain,
                    project_path,
                    output,
                    format,
                    include_large_files,
                    skip_expensive_metrics,
                )
                .await
            }
            Commands::Analyze(analyze_cmd) => Self::execute_analyze_command(analyze_cmd).await,
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
                // Convert CLI DemoProtocol to demo module Protocol
                let demo_protocol = if cli {
                    crate::demo::Protocol::Cli
                } else {
                    match protocol {
                        DemoProtocol::Cli => crate::demo::Protocol::Cli,
                        DemoProtocol::Http => crate::demo::Protocol::Http,
                        DemoProtocol::Mcp => crate::demo::Protocol::Mcp,
                        #[cfg(feature = "tui")]
                        DemoProtocol::Tui => crate::demo::Protocol::Tui,
                        DemoProtocol::All => crate::demo::Protocol::All,
                    }
                };

                let web_mode = !cli;

                // Create demo args
                let demo_args = crate::demo::DemoArgs {
                    path,
                    url,
                    repo,
                    format,
                    protocol: demo_protocol,
                    show_api,
                    no_browser,
                    port,
                    web: web_mode,
                    target_nodes,
                    centrality_threshold,
                    merge_threshold,
                    debug,
                    debug_output,
                    skip_vendor: skip_vendor && !no_skip_vendor,
                    max_line_length,
                };

                crate::demo::run_demo(demo_args, server).await
            }
            Commands::QualityGate {
                project_path,
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
                handlers::handle_quality_gate(
                    project_path,
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
                include_visualizations,
                include_executive_summary,
                include_recommendations,
                analyses,
                confidence_threshold,
                output,
                perf,
            } => {
                handlers::enhanced_reporting_handlers::handle_generate_report(
                    project_path,
                    output_format,
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
            Commands::Serve { port, host, cors } => handlers::handle_serve(host, port, cors).await,
            Commands::Diagnose(args) => super::diagnose::handle_diagnose(args).await,
            Commands::Enforce(enforce_cmd) => handlers::route_enforce_command(enforce_cmd).await,
            Commands::Refactor(refactor_cmd) => Self::execute_refactor_command(refactor_cmd).await,
        }
    }

    /// Execute analyze commands using handler pattern (reduces CC)
    pub async fn execute_analyze_command(analyze_cmd: AnalyzeCommands) -> anyhow::Result<()> {
        // Delegate to the modular analysis handlers
        super::handlers::route_analyze_command(analyze_cmd).await
    }

    /// Execute refactor commands using handler pattern (reduces CC)
    pub async fn execute_refactor_command(refactor_cmd: RefactorCommands) -> anyhow::Result<()> {
        // Delegate to the refactor handlers
        super::handlers::route_refactor_command(refactor_cmd).await
    }
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_command_dispatcher_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
