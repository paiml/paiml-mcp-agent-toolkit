#![cfg_attr(coverage_nightly, coverage(off))]
//! Demo command definitions and quality gate operations

use crate::stateless_server::StatelessTemplateServer;
use anyhow::Result;
use std::sync::Arc;

/// Command group for demo and quality gate operations
pub struct DemoCommandGroup;

impl Default for DemoCommandGroup {
    fn default() -> Self {
        Self
    }
}

#[cfg(feature = "demo")]
impl DemoCommandGroup {
    /// Handle demo command with comprehensive parameter support
    #[allow(clippy::too_many_arguments)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

#[cfg(not(feature = "demo"))]
impl DemoCommandGroup {
    /// Handle demo command - feature not enabled
    #[allow(clippy::too_many_arguments)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn handle_demo(
        &self,
        _server: Arc<StatelessTemplateServer>,
        _path: Option<std::path::PathBuf>,
        _url: Option<String>,
        _repo: Option<String>,
        _format: crate::cli::OutputFormat,
        _protocol: crate::cli::DemoProtocol,
        _show_api: bool,
        _no_browser: bool,
        _port: Option<u16>,
        _cli: bool,
        _target_nodes: usize,
        _centrality_threshold: f64,
        _merge_threshold: usize,
        _debug: bool,
        _debug_output: Option<std::path::PathBuf>,
        _skip_vendor: bool,
        _max_line_length: Option<usize>,
    ) -> Result<()> {
        anyhow::bail!("Demo feature not enabled. Build with --features demo")
    }

    /// Handle quality gate command - feature not enabled
    #[allow(clippy::too_many_arguments)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn handle_quality_gate(
        &self,
        _project_path: std::path::PathBuf,
        _file: Option<std::path::PathBuf>,
        _format: crate::cli::QualityGateOutputFormat,
        _fail_on_violation: bool,
        _checks: Vec<crate::cli::QualityCheckType>,
        _max_dead_code: f64,
        _min_entropy: f64,
        _max_complexity_p99: u32,
        _include_provability: bool,
        _output: Option<std::path::PathBuf>,
        _perf: bool,
    ) -> Result<()> {
        anyhow::bail!("Demo feature not enabled. Build with --features demo")
    }
}

/// Factory for creating command executors
pub struct CommandExecutorFactory;

impl CommandExecutorFactory {
    /// Create a new command executor instance
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn create(server: Arc<StatelessTemplateServer>) -> super::CommandExecutor {
        super::CommandExecutor::new(server)
    }
}
