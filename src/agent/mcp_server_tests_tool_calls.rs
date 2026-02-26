// === Tool Call Dispatch and MCP Request Handling Tests ===

#[tokio::test]
async fn test_mcp_server_handle_tool_call_health_check() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let params = json!({
        "name": "health_check",
        "arguments": {}
    });

    let result = server.handle_tool_call(params).await.unwrap();
    assert!(result.get("status").is_some());
}

#[tokio::test]
async fn test_mcp_server_handle_tool_call_unknown() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let params = json!({
        "name": "unknown_tool",
        "arguments": {}
    });

    let result = server.handle_tool_call(params).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unknown tool"));
}

#[tokio::test]
async fn test_mcp_server_handle_tool_call_missing_name() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let params = json!({
        "arguments": {}
    });

    let result = server.handle_tool_call(params).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("tool name"));
}

#[tokio::test]
async fn test_mcp_server_handle_tool_call_start_quality_monitoring() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let temp_dir = TempDir::new().unwrap();
    let path_str = temp_dir.path().to_string_lossy().to_string();

    let params = json!({
        "name": "start_quality_monitoring",
        "arguments": {
            "project_path": path_str
        }
    });

    let result = server.handle_tool_call(params).await.unwrap();
    assert!(result.get("text").is_some());
}

#[tokio::test]
async fn test_mcp_server_handle_tool_call_stop_quality_monitoring() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let params = json!({
        "name": "stop_quality_monitoring",
        "arguments": {
            "project_id": "test_project"
        }
    });

    let result = server.handle_tool_call(params).await.unwrap();
    assert!(result.get("text").is_some());
}

#[tokio::test]
async fn test_mcp_server_handle_tool_call_get_monitoring_status() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    // First add a project to monitor
    let temp_dir = TempDir::new().unwrap();
    let path_str = temp_dir.path().to_string_lossy().to_string();

    // Start monitoring
    let start_params = json!({
        "project_path": path_str
    });
    let _ = server.handle_start_monitoring(&start_params).await.unwrap();

    // Get the project name from path
    let project_name = temp_dir
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let params = json!({
        "name": "get_monitoring_status",
        "arguments": {
            "project_id": project_name
        }
    });

    let result = server.handle_tool_call(params).await.unwrap();
    assert!(result.get("text").is_some());
}

#[tokio::test]
async fn test_mcp_server_handle_tool_call_run_quality_gates() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let temp_dir = TempDir::new().unwrap();
    let path_str = temp_dir.path().to_string_lossy().to_string();

    let params = json!({
        "name": "run_quality_gates",
        "arguments": {
            "target_path": path_str
        }
    });

    let result = server.handle_tool_call(params).await.unwrap();
    assert!(result.get("content").is_some());
}

#[tokio::test]
async fn test_mcp_server_handle_tool_call_analyze_complexity() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let params = json!({
        "name": "analyze_complexity",
        "arguments": {
            "file_path": "/some/path"
        }
    });

    let result = server.handle_tool_call(params).await.unwrap();
    assert!(result.get("content").is_some());
}

#[tokio::test]
async fn test_handle_tool_call_with_null_arguments() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    // When arguments is not provided, it should use Value::Null
    let params = json!({
        "name": "health_check"
        // "arguments" is missing
    });

    let result = server.handle_tool_call(params).await.unwrap();
    assert!(result.get("status").is_some());
}

// === MCP Request Protocol Tests ===

#[tokio::test]
async fn test_mcp_server_handle_mcp_request_initialize() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });

    let result = server
        .handle_mcp_request(&request.to_string())
        .await
        .unwrap();
    assert!(result.is_some());
    let response = result.unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response.get("result").is_some());
}

#[tokio::test]
async fn test_mcp_server_handle_mcp_request_tools_list() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let result = server
        .handle_mcp_request(&request.to_string())
        .await
        .unwrap();
    assert!(result.is_some());
    let response = result.unwrap();
    assert!(response["result"]["tools"].is_array());
}

#[tokio::test]
async fn test_mcp_server_handle_mcp_request_health_check() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "health_check",
        "params": {}
    });

    let result = server
        .handle_mcp_request(&request.to_string())
        .await
        .unwrap();
    assert!(result.is_some());
    let response = result.unwrap();
    assert_eq!(response["result"]["status"], "healthy");
}

#[tokio::test]
async fn test_mcp_server_handle_mcp_request_unknown_method() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "unknown_method",
        "params": {}
    });

    let result = server
        .handle_mcp_request(&request.to_string())
        .await
        .unwrap();
    assert!(result.is_some());
    let response = result.unwrap();
    assert!(response.get("error").is_some());
    assert_eq!(response["error"]["code"], -32601);
}

#[tokio::test]
async fn test_mcp_server_handle_mcp_request_notification() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    // Notification (no id)
    let request = json!({
        "jsonrpc": "2.0",
        "method": "health_check",
        "params": {}
    });

    let result = server
        .handle_mcp_request(&request.to_string())
        .await
        .unwrap();
    // Notifications should return None
    assert!(result.is_none());
}

#[tokio::test]
async fn test_mcp_server_handle_mcp_request_invalid_json() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let result = server.handle_mcp_request("not valid json").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mcp_server_handle_mcp_request_missing_method() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "params": {}
    });

    let result = server.handle_mcp_request(&request.to_string()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("method"));
}

#[tokio::test]
async fn test_mcp_server_handle_mcp_request_tools_call() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "health_check",
            "arguments": {}
        }
    });

    let result = server
        .handle_mcp_request(&request.to_string())
        .await
        .unwrap();
    assert!(result.is_some());
    let response = result.unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 5);
    assert!(response.get("result").is_some());
}

#[tokio::test]
async fn test_mcp_server_handle_mcp_request_with_null_params() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "health_check"
        // params is missing - should default to null
    });

    let result = server
        .handle_mcp_request(&request.to_string())
        .await
        .unwrap();
    assert!(result.is_some());
    let response = result.unwrap();
    assert_eq!(response["result"]["status"], "healthy");
}
