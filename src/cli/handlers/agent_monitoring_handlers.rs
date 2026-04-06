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

    // Send command to running daemon to start monitoring
    match DaemonManager::start_monitoring(&project_path, &project_id).await {
        Ok(()) => {
            info!("Project monitoring command sent to daemon successfully");
            println!("✅ Started monitoring project '{project_id}' at {project_path:?}");
        }
        Err(e) => {
            error!("Failed to start monitoring: {}", e);
            return Err(anyhow::anyhow!("Failed to start monitoring: {e}"));
        }
    }
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

    // Send command to running daemon to stop monitoring
    match DaemonManager::stop_monitoring(&project_id).await {
        Ok(()) => {
            info!("Stop monitoring command sent to daemon successfully");
            println!("✅ Stopped monitoring project '{project_id}'");
        }
        Err(e) => {
            error!("Failed to stop monitoring: {}", e);
            return Err(anyhow::anyhow!("Failed to stop monitoring: {e}"));
        }
    }
    Ok(())
}

/// Run health check
async fn handle_agent_health(_pid_file: Option<PathBuf>, detailed: bool) -> Result<()> {
    if !DaemonManager::is_running().await {
        println!("❌ Agent daemon is not running");
        return Ok(());
    }

    if detailed {
        // Get detailed health information from running daemon
        match DaemonManager::get_health_info().await {
            Ok(health_info) => {
                println!("{}", serde_json::to_string_pretty(&health_info)?);
            }
            Err(e) => {
                error!("Failed to get detailed health info: {}", e);
                // Fallback to basic status
                let basic_health = serde_json::json!({
                    "status": "running",
                    "error": format!("Could not get detailed info: {}", e),
                    "last_check": chrono::Utc::now().to_rfc3339()
                });
                println!("{}", serde_json::to_string_pretty(&basic_health)?);
            }
        }
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

    if let Some(ref path) = config_path {
        info!("Loading configuration from {:?}", path);
        let _config = load_daemon_config(path).await?;
    }

    // Send reload command to running daemon
    match DaemonManager::reload_config(config_path.as_ref()).await {
        Ok(()) => {
            info!("Configuration reload command sent to daemon successfully");
            println!("✅ Configuration reloaded successfully");
        }
        Err(e) => {
            error!("Failed to reload configuration: {}", e);
            return Err(anyhow::anyhow!("Failed to reload configuration: {e}"));
        }
    }
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

    // Send quality gate command to running daemon
    match DaemonManager::run_quality_gate(&project).await {
        Ok(result) => {
            info!("Quality gate command sent to daemon successfully");
            println!("✅ Quality gate completed");
            if let Some(violations) = result.violations {
                if violations > 0 {
                    println!("⚠️  Found {violations} quality violations");
                }
            }
        }
        Err(e) => {
            error!("Failed to run quality gate: {}", e);
            return Err(anyhow::anyhow!("Failed to run quality gate: {e}"));
        }
    }
    Ok(())
}
