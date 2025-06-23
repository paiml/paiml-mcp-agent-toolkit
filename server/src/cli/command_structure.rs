//! Command Structure - Comprehensive CLI decomposition architecture
//!
//! This module provides the complete command structure decomposition to reduce
//! the main CLI module from 145 functions and 10,304 lines to a manageable size.
//!
//! Architecture:
//! - CommandExecutor: Main command execution orchestrator
//! - CommandRegistry: Registry of all available commands  
//! - CommandGroup: Logical grouping of related commands
//! - ModularHandlers: Individual command implementation modules

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
            Commands::Scaffold {
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
            } => {
                self.registry
                    .utility_handlers
                    .handle_context(toolchain, project_path, output, format, include_large_files, skip_expensive_metrics)
                    .await
            }
            Commands::Serve { port, host, cors } => {
                self.registry
                    .utility_handlers
                    .handle_serve(host, port, cors)
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
                crate::cli::handlers::enhanced_reporting_handlers::handle_generate_report(
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
            Commands::Diagnose(args) => self.registry.utility_handlers.handle_diagnose(args).await,
            Commands::Refactor(refactor_cmd) => {
                super::handlers::route_refactor_command(refactor_cmd).await
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
    pub async fn handle_context(
        &self,
        toolchain: Option<String>,
        project_path: std::path::PathBuf,
        output: Option<std::path::PathBuf>,
        format: crate::cli::ContextFormat,
        include_large_files: bool,
        skip_expensive_metrics: bool,
    ) -> Result<()> {
        crate::cli::handlers::utility_handlers::handle_context(
            toolchain,
            project_path,
            output,
            format,
            include_large_files,
            skip_expensive_metrics,
        )
        .await
    }

    /// Handle serve command
    pub async fn handle_serve(&self, host: String, port: u16, cors: bool) -> Result<()> {
        crate::cli::handlers::utility_handlers::handle_serve(host, port, cors).await
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
    pub fn create(server: Arc<StatelessTemplateServer>) -> CommandExecutor {
        CommandExecutor::new(server)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_registry_creation() {
        let _registry = CommandRegistry::default();

        // Verify all command groups are initialized - no assertion needed
    }

    #[test]
    fn test_command_group_defaults() {
        let _generate = GenerateCommandGroup;
        let _analyze = AnalyzeCommandGroup;
        let _utility = UtilityCommandGroup;
        let _demo = DemoCommandGroup;

        // All groups should be creatable - no assertion needed
    }
}
