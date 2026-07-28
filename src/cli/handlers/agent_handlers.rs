#![cfg_attr(coverage_nightly, coverage(off))]
//! Agent command handlers for Claude Code integration
//!
//! This module implements handlers for the agent subcommands, providing
//! background daemon management and continuous quality monitoring capabilities.

use crate::agent::{AgentDaemon, DaemonConfig, DaemonManager};
use crate::cli::AgentCommands;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tracing::{error, info, warn};

/// Configuration for starting the agent daemon
#[derive(Debug, Clone)]
struct AgentStartConfig {
    _project_path: PathBuf,
    config_path: Option<PathBuf>,
    working_dir: Option<PathBuf>,
    pid_file: Option<PathBuf>,
    log_file: Option<PathBuf>,
    foreground: bool,
    health_interval: u64,
    max_memory_mb: u64,
    auto_restart: bool,
}

/// Handle agent commands
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_agent_command(command: AgentCommands) -> Result<()> {
    match command {
        AgentCommands::Start {
            project_path,
            config,
            working_dir,
            pid_file,
            log_file,
            foreground,
            health_interval,
            max_memory_mb,
            no_auto_restart,
        } => {
            let start_config = AgentStartConfig {
                _project_path: project_path,
                config_path: config,
                working_dir,
                pid_file,
                log_file,
                foreground,
                health_interval,
                max_memory_mb,
                auto_restart: !no_auto_restart,
            };
            handle_agent_start(start_config).await
        }
        AgentCommands::Stop {
            pid_file,
            force,
            timeout,
        } => handle_agent_stop(pid_file, force, timeout).await,
        AgentCommands::Status { pid_file, format } => handle_agent_status(pid_file, format).await,
        AgentCommands::Monitor {
            project_path,
            project_id,
            thresholds,
        } => handle_agent_monitor(project_path, project_id, thresholds).await,
        AgentCommands::Unmonitor { project_id } => handle_agent_unmonitor(project_id).await,
        AgentCommands::Health { pid_file, detailed } => {
            handle_agent_health(pid_file, detailed).await
        }
        AgentCommands::Reload { pid_file, config } => handle_agent_reload(pid_file, config).await,
        AgentCommands::QualityGate {
            project,
            file,
            format,
        } => handle_agent_quality_gate(project, file, format).await,
        AgentCommands::McpServer { config, debug } => handle_agent_mcp_server(config, debug).await,
    }
}

// Daemon start/stop/status lifecycle handlers
include!("agent_daemon_lifecycle.rs");

// Monitoring, health, reload, and quality gate handlers
include!("agent_monitoring_handlers.rs");

// MCP server handler and config loading
include!("agent_mcp_server.rs");

// Unit tests
include!("agent_handlers_tests.rs");
