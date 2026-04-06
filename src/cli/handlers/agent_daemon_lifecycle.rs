/// Start the background agent daemon
async fn handle_agent_start(config: AgentStartConfig) -> Result<()> {
    info!("Starting Claude Code Agent daemon");

    check_daemon_not_running().await?;
    let foreground = config.foreground;
    let daemon_config = prepare_daemon_config(config).await?;
    let daemon = AgentDaemon::new(daemon_config);

    start_daemon_with_mode(daemon, foreground).await
}

async fn check_daemon_not_running() -> Result<()> {
    if DaemonManager::is_running().await {
        Err(anyhow!(
            "Agent daemon is already running. Use 'pmat agent stop' to stop it first."
        ))
    } else {
        Ok(())
    }
}

async fn prepare_daemon_config(config: AgentStartConfig) -> Result<DaemonConfig> {
    let mut daemon_config = load_or_default_config(&config.config_path).await?;
    apply_config_overrides(&mut daemon_config, &config);
    Ok(daemon_config)
}

async fn load_or_default_config(config_path: &Option<PathBuf>) -> Result<DaemonConfig> {
    if let Some(path) = config_path {
        load_daemon_config(path).await
    } else {
        Ok(DaemonConfig::default())
    }
}

fn apply_config_overrides(daemon_config: &mut DaemonConfig, config: &AgentStartConfig) {
    daemon_config.daemon.health_check_interval = Duration::from_secs(config.health_interval);
    daemon_config.daemon.max_memory_mb = config.max_memory_mb;
    daemon_config.daemon.auto_restart = config.auto_restart;

    apply_optional_overrides(daemon_config, config);
}

fn apply_optional_overrides(daemon_config: &mut DaemonConfig, config: &AgentStartConfig) {
    if let Some(working_dir) = &config.working_dir {
        daemon_config.daemon.working_directory = working_dir.clone();
    }

    if let Some(pid_file) = &config.pid_file {
        daemon_config.daemon.pid_file = Some(pid_file.clone());
    }

    if let Some(log_file) = &config.log_file {
        daemon_config.daemon.log_file = Some(log_file.clone());
    }
}

async fn start_daemon_with_mode(mut daemon: AgentDaemon, foreground: bool) -> Result<()> {
    if foreground {
        info!("Starting daemon in foreground mode");
        daemon.start().await
    } else {
        start_background_mode(daemon).await
    }
}

async fn start_background_mode(mut daemon: AgentDaemon) -> Result<()> {
    info!("Starting daemon in background mode");
    // In a real implementation, we would fork the process here
    // For now, just start normally
    warn!("Background mode not fully implemented, running in foreground");
    daemon.start().await
}

/// Stop the background agent daemon
async fn handle_agent_stop(_pid_file: Option<PathBuf>, _force: bool, _timeout: u64) -> Result<()> {
    info!("Stopping Claude Code Agent daemon");

    if !DaemonManager::is_running().await {
        warn!("Agent daemon is not running");
        return Ok(());
    }

    // Implement daemon communication and graceful shutdown
    match DaemonManager::shutdown().await {
        Ok(()) => {
            info!("Agent daemon shut down successfully");
        }
        Err(e) => {
            error!("Failed to shut down daemon: {}", e);
            return Err(anyhow::anyhow!("Failed to shut down daemon: {e}"));
        }
    }
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
