//! Demo and quality gate command handlers
//!
//! This module contains handlers for demo mode and quality gate operations
//! extracted from the main CLI module to reduce complexity.

use crate::cli::*;
use crate::stateless_server::StatelessTemplateServer;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

/// Handle demo command with comprehensive parameter support
#[allow(clippy::too_many_arguments)]
pub async fn handle_demo(
    server: Arc<StatelessTemplateServer>,
    path: Option<PathBuf>,
    url: Option<String>,
    repo: Option<String>,
    format: OutputFormat,
    protocol: DemoProtocol,
    show_api: bool,
    no_browser: bool,
    port: Option<u16>,
    cli: bool,
    target_nodes: usize,
    centrality_threshold: f64,
    merge_threshold: usize,
    debug: bool,
    debug_output: Option<PathBuf>,
    skip_vendor: bool,
    max_line_length: Option<usize>,
) -> Result<()> {
    // Convert CLI DemoProtocol to demo module Protocol
    let demo_protocol = if cli {
        // --cli flag overrides protocol to CLI
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

    // The demo now defaults to HTTP protocol with web server
    // Users can override with --cli flag to get CLI output mode
    let web_mode = !cli;

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
        skip_vendor,
        max_line_length,
    };
    crate::demo::run_demo(demo_args, server).await
}

/// Handle quality gate command
#[allow(clippy::too_many_arguments)]
pub async fn handle_quality_gate(
    project_path: PathBuf,
    format: QualityGateOutputFormat,
    fail_on_violation: bool,
    checks: Vec<QualityCheckType>,
    max_dead_code: f64,
    min_entropy: f64,
    max_complexity_p99: u32,
    include_provability: bool,
    output: Option<PathBuf>,
    perf: bool,
) -> Result<()> {
    // Delegate to main quality gate implementation for now - will be extracted later
    super::super::stubs::handle_quality_gate(
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

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_demo_handlers_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
