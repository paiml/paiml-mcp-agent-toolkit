//! Demo Command Handlers for CommandDispatcher
//!
//! Extracted from command_dispatcher mod.rs for file health compliance (CB-040).
//! Contains demo command execution and helper functions.

use super::CommandDispatcher;
use crate::cli::{DemoProtocol, OutputFormat};
use std::path::PathBuf;
use std::sync::Arc;

impl CommandDispatcher {
    /// Execute demo command with protocol conversion (reduces complexity)
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn execute_demo_command(
        path: Option<PathBuf>,
        url: Option<String>,
        repo: Option<String>,
        format: Option<OutputFormat>,
        protocol: DemoProtocol,
        show_api: bool,
        no_browser: bool,
        port: u16,
        cli: bool,
        target_nodes: Option<usize>,
        centrality_threshold: Option<f64>,
        merge_threshold: Option<f64>,
        debug: bool,
        debug_output: Option<PathBuf>,
        skip_vendor: bool,
        no_skip_vendor: bool,
        max_line_length: Option<usize>,
        server: Arc<crate::stateless_server::StatelessTemplateServer>,
    ) -> anyhow::Result<()> {
        let demo_protocol = Self::convert_demo_protocol(protocol, cli);
        let demo_args = Self::create_demo_args(
            path,
            url,
            repo,
            format,
            demo_protocol,
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
        );

        crate::demo::run_demo(demo_args, server).await
    }

    /// Convert CLI `DemoProtocol` to demo module Protocol
    pub(crate) fn convert_demo_protocol(
        protocol: DemoProtocol,
        cli: bool,
    ) -> crate::demo::Protocol {
        if cli {
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
        }
    }

    /// Create demo arguments structure
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_demo_args(
        path: Option<PathBuf>,
        url: Option<String>,
        repo: Option<String>,
        format: Option<OutputFormat>,
        protocol: crate::demo::Protocol,
        show_api: bool,
        no_browser: bool,
        port: u16,
        cli: bool,
        target_nodes: Option<usize>,
        centrality_threshold: Option<f64>,
        merge_threshold: Option<f64>,
        debug: bool,
        debug_output: Option<PathBuf>,
        skip_vendor: bool,
        no_skip_vendor: bool,
        max_line_length: Option<usize>,
    ) -> crate::demo::DemoArgs {
        crate::demo::DemoArgs {
            path,
            url,
            repo,
            format: format.unwrap_or(OutputFormat::Table),
            protocol,
            show_api,
            no_browser,
            port: Some(port),
            web: !cli,
            target_nodes: target_nodes.unwrap_or(1000),
            centrality_threshold: centrality_threshold.unwrap_or(0.5),
            merge_threshold: merge_threshold.map_or(100, |t| t as usize),
            debug,
            debug_output,
            skip_vendor: skip_vendor && !no_skip_vendor,
            max_line_length,
        }
    }
}
