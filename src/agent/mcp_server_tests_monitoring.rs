// === Start/Stop/Get-Status Monitoring Tests ===

#[tokio::test]
async fn test_mcp_server_handle_start_monitoring() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let temp_dir = TempDir::new().unwrap();
    let path_str = temp_dir.path().to_string_lossy().to_string();

    let params = json!({
        "project_path": path_str,
        "watch_patterns": ["**/*.rs", "**/*.py"],
        "complexity_threshold": 25
    });

    let result = server.handle_start_monitoring(&params).await.unwrap();
    assert!(result.get("text").is_some());
    let text = result["text"].as_str().unwrap();
    assert!(text.contains("Started quality monitoring"));
}

#[tokio::test]
async fn test_mcp_server_handle_start_monitoring_missing_path() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let params = json!({});

    let result = server.handle_start_monitoring(&params).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("project_path"));
}

#[tokio::test]
async fn test_mcp_server_handle_start_monitoring_nonexistent_path() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let params = json!({
        "project_path": "/nonexistent/path/that/does/not/exist"
    });

    let result = server.handle_start_monitoring(&params).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
}

#[tokio::test]
async fn test_mcp_server_handle_start_monitoring_default_watch_patterns() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config.clone());

    let temp_dir = TempDir::new().unwrap();
    let path_str = temp_dir.path().to_string_lossy().to_string();

    // Only provide project_path, use default watch_patterns
    let params = json!({
        "project_path": path_str
    });

    let result = server.handle_start_monitoring(&params).await.unwrap();
    let text = result["text"].as_str().unwrap();
    // Should use default patterns from config
    assert!(text.contains("Started quality monitoring"));
}

#[tokio::test]
async fn test_mcp_server_handle_stop_monitoring() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let params = json!({
        "project_id": "test_project"
    });

    let result = server.handle_stop_monitoring(&params).await.unwrap();
    assert!(result.get("text").is_some());
    let text = result["text"].as_str().unwrap();
    assert!(text.contains("Stopped quality monitoring"));
    assert!(text.contains("test_project"));
}

#[tokio::test]
async fn test_mcp_server_handle_stop_monitoring_missing_id() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let params = json!({});

    let result = server.handle_stop_monitoring(&params).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("project_id"));
}

#[tokio::test]
async fn test_mcp_server_handle_get_status_not_monitored() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let params = json!({
        "project_id": "unknown_project"
    });

    let result = server.handle_get_status(&params).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("not being monitored"));
}

#[tokio::test]
async fn test_mcp_server_handle_get_status_missing_id() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let params = json!({});

    let result = server.handle_get_status(&params).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("project_id"));
}

#[tokio::test]
async fn test_mcp_server_handle_get_status_with_last_analysis() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    // Manually add a monitored project with last_analysis
    let analysis = ProjectAnalysisResult {
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        quality_score: 0.95,
        files_analyzed: 20,
        functions_analyzed: 80,
        avg_complexity: 7.0,
        hotspot_functions: 1,
        satd_issues: 0,
        quality_gate_status: "PASSED".to_string(),
        recommendations: vec!["Keep up the good work".to_string()],
    };

    let project = MonitoredProject {
        path: PathBuf::from("/test/project"),
        name: "test_with_analysis".to_string(),
        watch_patterns: vec!["**/*.rs".to_string()],
        complexity_threshold: 20,
        last_analysis: Some(analysis),
        started_at: std::time::SystemTime::now(),
    };

    server
        .monitored_projects
        .insert("test_with_analysis".to_string(), project);

    let params = json!({
        "project_id": "test_with_analysis"
    });

    let result = server.handle_get_status(&params).await.unwrap();
    let text = result["text"].as_str().unwrap();
    assert!(text.contains("Quality Status"));
    assert!(text.contains("quality_score"));
}

#[tokio::test]
async fn test_mcp_server_handle_get_status_without_last_analysis() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    // Manually add a monitored project without last_analysis
    let project = MonitoredProject {
        path: PathBuf::from("/test/project"),
        name: "test_no_analysis".to_string(),
        watch_patterns: vec!["**/*.rs".to_string()],
        complexity_threshold: 20,
        last_analysis: None,
        started_at: std::time::SystemTime::now(),
    };

    server
        .monitored_projects
        .insert("test_no_analysis".to_string(), project);

    let params = json!({
        "project_id": "test_no_analysis"
    });

    let result = server.handle_get_status(&params).await.unwrap();
    let text = result["text"].as_str().unwrap();
    assert!(text.contains("Quality Status"));
    assert!(text.contains("PENDING"));
}

#[tokio::test]
async fn test_mcp_server_start_and_stop_monitoring_sequence() {
    let config = AgentConfig::default();
    let mut server = ClaudeCodeAgentMcpServer::new(config);

    let temp_dir = TempDir::new().unwrap();
    let path_str = temp_dir.path().to_string_lossy().to_string();
    let project_name = temp_dir
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    // Start monitoring
    let start_params = json!({
        "project_path": path_str,
        "complexity_threshold": 15
    });
    let start_result = server.handle_start_monitoring(&start_params).await.unwrap();
    assert!(start_result["text"].as_str().unwrap().contains("Started"));

    // Verify project is in monitored_projects
    assert!(server.monitored_projects.contains_key(&project_name));

    // Stop monitoring
    let stop_params = json!({
        "project_id": project_name
    });
    let stop_result = server.handle_stop_monitoring(&stop_params).await.unwrap();
    assert!(stop_result["text"].as_str().unwrap().contains("Stopped"));

    // Verify project is removed from monitored_projects
    assert!(!server.monitored_projects.contains_key(&project_name));
}
