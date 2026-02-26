// === ClaudeCodeAgentMcpServer Basic Tests ===

#[tokio::test]
async fn test_mcp_server_new_with_custom_config() {
    let config = AgentConfig {
        name: "custom-server".to_string(),
        version: "3.0.0".to_string(),
        complexity_threshold: 30,
        watch_patterns: vec!["**/*.rs".to_string()],
        update_interval: 15,
        max_projects: 20,
    };

    let server = ClaudeCodeAgentMcpServer::new(config);
    assert_eq!(server.config.name, "custom-server");
    assert_eq!(server.config.version, "3.0.0");
    assert_eq!(server.config.complexity_threshold, 30);
}

#[tokio::test]
async fn test_mcp_server_handle_initialize() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let result = server.handle_initialize(json!({})).await.unwrap();
    assert!(result.get("protocolVersion").is_some());
    assert!(result.get("capabilities").is_some());
    assert!(result.get("serverInfo").is_some());

    let server_info = result.get("serverInfo").unwrap();
    assert_eq!(server_info["name"], "pmat-agent");
}

#[tokio::test]
async fn test_mcp_server_handle_initialize_with_params() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    // Initialize can accept various params
    let params = json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {
            "name": "test-client",
            "version": "1.0.0"
        }
    });

    let result = server.handle_initialize(params).await.unwrap();
    assert!(result.get("protocolVersion").is_some());
    assert_eq!(result["protocolVersion"], "2024-11-05");
}

#[tokio::test]
async fn test_mcp_server_handle_tools_list() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let result = server.handle_tools_list().await.unwrap();
    assert!(result.get("tools").is_some());

    let tools = result["tools"].as_array().unwrap();
    assert!(!tools.is_empty());

    // Check specific tools exist
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    assert!(tool_names.contains(&"start_quality_monitoring"));
    assert!(tool_names.contains(&"run_quality_gates"));
    assert!(tool_names.contains(&"analyze_complexity"));
    assert!(tool_names.contains(&"health_check"));
}

#[tokio::test]
async fn test_mcp_server_handle_health_check() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let result = server.handle_health_check().await.unwrap();
    assert_eq!(result["status"], "healthy");
    assert!(result.get("timestamp").is_some());
    assert!(result.get("version").is_some());
}

#[tokio::test]
async fn test_mcp_server_get_tool_capabilities() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let capabilities = server.get_tool_capabilities();
    assert!(capabilities.get("start_quality_monitoring").is_some());
    assert!(capabilities.get("stop_quality_monitoring").is_some());
    assert!(capabilities.get("get_quality_status").is_some());
    assert!(capabilities.get("run_quality_gates").is_some());
    assert!(capabilities.get("analyze_complexity").is_some());
    assert!(capabilities.get("health_check").is_some());
}

#[tokio::test]
async fn test_mcp_server_get_resource_capabilities() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let capabilities = server.get_resource_capabilities();
    assert!(capabilities.get("quality-metrics").is_some());
    assert!(capabilities.get("complexity-heatmap").is_some());
    assert!(capabilities.get("refactor-suggestions").is_some());
    assert!(capabilities.get("quality-reports").is_some());
}

#[tokio::test]
async fn test_mcp_server_get_prompt_capabilities() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let capabilities = server.get_prompt_capabilities();
    assert!(capabilities.get("quality-summary").is_some());
    assert!(capabilities.get("refactoring-guide").is_some());
}

#[tokio::test]
async fn test_mcp_server_clone() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let cloned = server.clone();
    assert_eq!(cloned.config.name, server.config.name);
    assert_eq!(cloned.config.version, server.config.version);
}

#[tokio::test]
async fn test_tool_capabilities_input_schema_structure() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let capabilities = server.get_tool_capabilities();

    // Check that each tool has proper inputSchema
    let start_monitoring = capabilities.get("start_quality_monitoring").unwrap();
    let input_schema = start_monitoring.get("inputSchema").unwrap();
    assert_eq!(input_schema["type"], "object");
    assert!(input_schema.get("properties").is_some());
    assert!(input_schema.get("required").is_some());

    // Check run_quality_gates
    let quality_gates = capabilities.get("run_quality_gates").unwrap();
    let qg_schema = quality_gates.get("inputSchema").unwrap();
    assert!(qg_schema["properties"].get("target_path").is_some());
    assert!(qg_schema["properties"].get("output_format").is_some());
}

#[tokio::test]
async fn test_resource_capabilities_mime_types() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let capabilities = server.get_resource_capabilities();

    // Check mime types are correctly set
    assert_eq!(
        capabilities["quality-metrics"]["mimeType"],
        "application/json"
    );
    assert_eq!(
        capabilities["complexity-heatmap"]["mimeType"],
        "application/json"
    );
}

#[tokio::test]
async fn test_prompt_capabilities_arguments() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let capabilities = server.get_prompt_capabilities();

    // Check quality-summary arguments
    let quality_summary = capabilities.get("quality-summary").unwrap();
    assert!(quality_summary.get("description").is_some());
    let args = quality_summary.get("arguments").unwrap();
    assert!(args.get("project_id").is_some());

    // Check refactoring-guide arguments
    let refactoring = capabilities.get("refactoring-guide").unwrap();
    let refactor_args = refactoring.get("arguments").unwrap();
    assert!(refactor_args.get("file_path").is_some());
    assert!(refactor_args.get("complexity_target").is_some());
}

#[tokio::test]
async fn test_mcp_server_quality_gate_service_is_initialized() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    // Verify the quality gate service is properly initialized
    // We can't directly access the service, but we can call methods that use it
    let temp_dir = TempDir::new().unwrap();
    let path_str = temp_dir.path().to_string_lossy().to_string();

    let params = json!({
        "target_path": path_str
    });

    // This exercises the quality_gate_service
    let result = server.handle_run_quality_gates(&params).await;
    assert!(result.is_ok());
}
