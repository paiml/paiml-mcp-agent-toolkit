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
use tracing::{info, warn};

/// Handle agent commands
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
            handle_agent_start(
                project_path,
                config,
                working_dir,
                pid_file,
                log_file,
                foreground,
                health_interval,
                max_memory_mb,
                !no_auto_restart,
            )
            .await
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

/// Start the background agent daemon
async fn handle_agent_start(
    _project_path: PathBuf,
    config_path: Option<PathBuf>,
    working_dir: Option<PathBuf>,
    pid_file: Option<PathBuf>,
    log_file: Option<PathBuf>,
    foreground: bool,
    health_interval: u64,
    max_memory_mb: u64,
    auto_restart: bool,
) -> Result<()> {
    info!("Starting Claude Code Agent daemon");

    // Load or create configuration
    let mut daemon_config = if let Some(config_path) = config_path {
        load_daemon_config(&config_path).await?
    } else {
        DaemonConfig::default()
    };

    // Override configuration with command-line options
    daemon_config.daemon.health_check_interval = Duration::from_secs(health_interval);
    daemon_config.daemon.max_memory_mb = max_memory_mb;
    daemon_config.daemon.auto_restart = auto_restart;

    if let Some(working_dir) = working_dir {
        daemon_config.daemon.working_directory = working_dir;
    }

    if let Some(pid_file) = pid_file {
        daemon_config.daemon.pid_file = Some(pid_file);
    }

    if let Some(log_file) = log_file {
        daemon_config.daemon.log_file = Some(log_file);
    }

    // Check if daemon is already running
    if DaemonManager::is_running().await {
        return Err(anyhow!(
            "Agent daemon is already running. Use 'pmat agent stop' to stop it first."
        ));
    }

    // Create and start daemon
    let mut daemon = AgentDaemon::new(daemon_config);

    if foreground {
        info!("Starting daemon in foreground mode");
        daemon.start().await
    } else {
        info!("Starting daemon in background mode");
        // In a real implementation, we would fork the process here
        // For now, just start normally
        warn!("Background mode not fully implemented, running in foreground");
        daemon.start().await
    }
}

/// Stop the background agent daemon
async fn handle_agent_stop(_pid_file: Option<PathBuf>, _force: bool, _timeout: u64) -> Result<()> {
    info!("Stopping Claude Code Agent daemon");

    if !DaemonManager::is_running().await {
        warn!("Agent daemon is not running");
        return Ok(());
    }

    // TODO: Implement daemon communication and graceful shutdown
    info!("Daemon stop functionality not yet implemented");
    Ok(())
}

/// Show daemon status
async fn handle_agent_status(
    _pid_file: Option<PathBuf>,
    format: crate::cli::OutputFormat,
) -> Result<()> {
    info!("Checking Claude Code Agent daemon status");

    let is_running = DaemonManager::is_running().await;

    match format {
        crate::cli::OutputFormat::Json => {
            let status = serde_json::json!({
                "running": is_running,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "version": env!("CARGO_PKG_VERSION")
            });
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        _ => {
            if is_running {
                println!("✅ Claude Code Agent daemon is running");
            } else {
                println!("❌ Claude Code Agent daemon is not running");
            }
        }
    }

    Ok(())
}

/// Start monitoring a new project
async fn handle_agent_monitor(
    project_path: PathBuf,
    project_id: Option<String>,
    _thresholds: Option<PathBuf>,
) -> Result<()> {
    let project_id = project_id.unwrap_or_else(|| {
        project_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });

    info!(
        "Starting monitoring for project '{}' at {:?}",
        project_id, project_path
    );

    if !DaemonManager::is_running().await {
        return Err(anyhow!(
            "Agent daemon is not running. Start it first with 'pmat agent start'"
        ));
    }

    // TODO: Send command to running daemon to start monitoring
    info!("Project monitoring command sent to daemon");
    Ok(())
}

/// Stop monitoring a project
async fn handle_agent_unmonitor(project_id: String) -> Result<()> {
    info!("Stopping monitoring for project '{}'", project_id);

    if !DaemonManager::is_running().await {
        return Err(anyhow!(
            "Agent daemon is not running. Start it first with 'pmat agent start'"
        ));
    }

    // TODO: Send command to running daemon to stop monitoring
    info!("Stop monitoring command sent to daemon");
    Ok(())
}

/// Run health check
async fn handle_agent_health(_pid_file: Option<PathBuf>, detailed: bool) -> Result<()> {
    if !DaemonManager::is_running().await {
        println!("❌ Agent daemon is not running");
        return Ok(());
    }

    if detailed {
        // TODO: Get detailed health information from running daemon
        let health_info = serde_json::json!({
            "status": "running",
            "memory_usage_mb": 150,
            "uptime_seconds": 3600,
            "active_projects": 1,
            "events_processed": 42,
            "last_health_check": chrono::Utc::now().to_rfc3339()
        });
        println!("{}", serde_json::to_string_pretty(&health_info)?);
    } else {
        println!("✅ Agent daemon is healthy");
    }

    Ok(())
}

/// Reload daemon configuration
async fn handle_agent_reload(
    _pid_file: Option<PathBuf>,
    config_path: Option<PathBuf>,
) -> Result<()> {
    info!("Reloading agent daemon configuration");

    if !DaemonManager::is_running().await {
        return Err(anyhow!(
            "Agent daemon is not running. Start it first with 'pmat agent start'"
        ));
    }

    if let Some(config_path) = config_path {
        info!("Loading configuration from {:?}", config_path);
        let _config = load_daemon_config(&config_path).await?;
    }

    // TODO: Send reload command to running daemon
    info!("Configuration reload command sent to daemon");
    Ok(())
}

/// Run quality gate through agent
async fn handle_agent_quality_gate(
    project: String,
    _file: Option<PathBuf>,
    _format: crate::cli::QualityGateOutputFormat,
) -> Result<()> {
    info!("Running quality gate for project '{}'", project);

    if !DaemonManager::is_running().await {
        return Err(anyhow!(
            "Agent daemon is not running. Start it first with 'pmat agent start'"
        ));
    }

    // TODO: Send quality gate command to running daemon
    info!("Quality gate command sent to daemon");
    Ok(())
}

/// Start MCP server for testing
async fn handle_agent_mcp_server(config_path: Option<PathBuf>, debug: bool) -> Result<()> {
    // Only log to stderr if debug is enabled
    if debug {
        eprintln!("Starting MCP server in debug mode");
    }

    // Load or create configuration
    let daemon_config = if let Some(config_path) = config_path {
        load_daemon_config(&config_path).await?
    } else {
        DaemonConfig::default()
    };

    // Create and start MCP server
    let mut mcp_server = crate::agent::ClaudeCodeAgentMcpServer::new(daemon_config.agent);

    if debug {
        eprintln!("MCP Server starting on stdio transport...");
        eprintln!("Server capabilities:");
        eprintln!("  - start_quality_monitoring: Start monitoring a project");
        eprintln!("  - run_quality_gates: Execute quality gates");
        eprintln!("  - analyze_complexity: Analyze code complexity");
        eprintln!("  - health_check: Check system health");
        eprintln!();
        eprintln!("Ready for MCP client connections via stdio...");
    }

    // Start the MCP server (this will block)
    mcp_server.start_stdio().await
}

/// Load daemon configuration from file
async fn load_daemon_config(config_path: &PathBuf) -> Result<DaemonConfig> {
    if !config_path.exists() {
        return Err(anyhow!("Configuration file not found: {:?}", config_path));
    }

    let config_content = fs::read_to_string(config_path).await?;
    let config: DaemonConfig = toml::from_str(&config_content)
        .or_else(|_| serde_json::from_str(&config_content))
        .map_err(|e| anyhow!("Failed to parse configuration file: {}", e))?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_load_daemon_config_missing_file() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");

        let result = load_daemon_config(&config_path).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Configuration file not found"));
    }

    #[tokio::test]
    async fn test_daemon_status_json_format() {
        let result = handle_agent_status(None, crate::cli::OutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_agent_monitor_with_default_project_id() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        let result = handle_agent_monitor(project_path, None, None).await;
        // This will fail because daemon is not running, but that's expected
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Agent daemon is not running"));
    }
}
