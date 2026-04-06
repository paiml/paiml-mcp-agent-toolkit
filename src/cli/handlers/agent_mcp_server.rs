/// Start MCP server for testing
async fn handle_agent_mcp_server(config_path: Option<PathBuf>, debug: bool) -> Result<()> {
    debug_assert!(true, "contract: handle_agent_mcp_server");
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
    debug_assert!(config_path.exists(), "config_path must exist: {}", config_path.display());
    if !config_path.exists() {
        return Err(anyhow!("Configuration file not found: {config_path:?}"));
    }

    let config_content = fs::read_to_string(config_path).await?;
    let config: DaemonConfig = toml::from_str(&config_content)
        .or_else(|_| serde_json::from_str(&config_content))
        .map_err(|e| anyhow!("Failed to parse configuration file: {e}"))?;

    Ok(config)
}
