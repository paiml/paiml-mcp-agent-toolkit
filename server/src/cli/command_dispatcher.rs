//! Command Dispatcher - Reduces CLI complexity through handler pattern
//!
//! This module implements a dispatch table pattern to reduce cyclomatic complexity
//! in the CLI module by delegating command execution to specialized handlers.

use super::{AnalyzeCommands, Commands};
use crate::stateless_server::StatelessTemplateServer;
use std::sync::Arc;

/// Trait for command handlers to reduce complexity through delegation
#[allow(dead_code)]
pub trait CommandHandler: Send + Sync {
    async fn execute(&self, server: Arc<StatelessTemplateServer>) -> anyhow::Result<()>;
}

/// Trait for analyze command handlers
#[allow(dead_code)]
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
        match command {
            Commands::Generate {
                category,
                template,
                params,
                output,
                create_dirs,
            } => {
                super::handlers::handle_generate(
                    server,
                    category,
                    template,
                    params,
                    output,
                    create_dirs,
                )
                .await
            }
            Commands::Scaffold {
                toolchain,
                templates,
                params,
                parallel,
            } => {
                super::handlers::handle_scaffold(server, toolchain, templates, params, parallel)
                    .await
            }
            Commands::List {
                toolchain,
                category,
                format,
            } => super::handlers::handle_list(server, toolchain, category, format).await,
            Commands::Search {
                query,
                toolchain,
                limit,
            } => super::handlers::handle_search(server, query, toolchain, limit).await,
            Commands::Validate { uri, params } => {
                super::handlers::handle_validate(server, uri, params).await
            }
            Commands::Context {
                toolchain,
                project_path,
                output,
                format,
            } => super::handlers::handle_context(toolchain, project_path, output, format).await,
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
                super::execute_demo_command(
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
                    server,
                )
                .await
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
                super::handle_quality_gate(
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
                super::handlers::enhanced_reporting_handlers::handle_generate_report(
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
            Commands::Serve { port, host, cors } => super::handle_serve(host, port, cors).await,
            Commands::Diagnose(args) => super::diagnose::handle_diagnose(args).await,
        }
    }

    /// Execute analyze commands using handler pattern (reduces CC)
    pub async fn execute_analyze_command(analyze_cmd: AnalyzeCommands) -> anyhow::Result<()> {
        // Delegate to the modular analysis handlers
        super::handlers::route_analyze_command(analyze_cmd).await
    }
}
