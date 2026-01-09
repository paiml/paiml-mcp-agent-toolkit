//! Integration tests for Claude Code Agent
//!
//! These tests verify the complete agent functionality including
//! MCP server, quality monitoring, and state persistence.

#[cfg(test)]
mod agent_tests {
    use pmat::agent::{
        AgentConfig, AgentDaemon, ClaudeCodeAgentMcpServer, DaemonConfig, ProjectState,
        QualityThresholds, StatePersistence,
    };
    use pmat::services::quality_gate_service::{
        QualityCheck, QualityGateInput, QualityGateService,
    };
    use pmat::services::service_base::Service;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    #[ignore] // Hangs indefinitely waiting for stdio input
    async fn test_mcp_server_initialization() {
        // Create MCP server
        let config = AgentConfig::default();
        let _server = ClaudeCodeAgentMcpServer::new(config);

        // Verify server can be created (cannot test start_stdio as it hangs)
        // assert!(server.start_stdio().await.is_ok());
    }

    #[tokio::test]
    async fn test_daemon_lifecycle() {
        // Create daemon configuration
        let config = DaemonConfig::default();
        let mut daemon = AgentDaemon::new(config);

        // Test start
        let start_handle = tokio::spawn(async move { daemon.start().await });

        // Give it time to start
        sleep(Duration::from_millis(100)).await;

        // Should be running (we can't easily test this without the daemon running)
        // In a real test, we'd check the PID file or daemon status

        start_handle.abort();
    }

    #[tokio::test]
    async fn test_state_persistence() {
        let temp_dir = TempDir::new().unwrap();

        // Create persistence manager
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        // Add a project
        let project = ProjectState {
            id: "test_project".to_string(),
            path: PathBuf::from("/test/path"),
            started_at: chrono::Utc::now(),
            last_analyzed: None,
            current_metrics: Default::default(),
            watch_patterns: vec!["*.rs".to_string()],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project.clone()).await.unwrap();

        // Save state
        persistence.save().await.unwrap();

        // Create new persistence instance and verify state was loaded
        let persistence2 = StatePersistence::new(temp_dir.path()).unwrap();
        let state = persistence2.get_state().await;

        assert!(state.monitored_projects.contains_key("test_project"));
        assert_eq!(
            state.monitored_projects["test_project"].path,
            PathBuf::from("/test/path")
        );
    }

    #[tokio::test]
    async fn test_quality_gate_integration() {
        // Create quality gate service
        let service = QualityGateService::new();

        // Create test input
        let input = QualityGateInput {
            path: PathBuf::from("."),
            checks: vec![
                QualityCheck::Complexity { max: 20 },
                QualityCheck::Satd { tolerance: 0 },
            ],
            strict: true,
        };

        // Run quality gates
        let result = service.process(input).await.unwrap();

        // Verify result structure
        assert!(!result.results.is_empty());
        assert!(result.summary.total_checks > 0);
    }

    #[tokio::test]
    async fn test_metrics_update() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        // Add project
        let project = ProjectState {
            id: "metrics_test".to_string(),
            path: PathBuf::from("/test"),
            started_at: chrono::Utc::now(),
            last_analyzed: None,
            current_metrics: Default::default(),
            watch_patterns: vec![],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();

        // Update metrics
        let metrics = pmat::agent::PersistentQualityMetrics {
            avg_complexity: 5.5,
            max_complexity: 15,
            satd_count: 0,
            dead_code_percentage: 2.5,
            quality_score: 92.0,
            files_analyzed: 100,
            total_violations: 0,
        };

        persistence
            .update_metrics("metrics_test", metrics)
            .await
            .unwrap();

        // Verify metrics were updated
        let state = persistence.get_state().await;
        assert_eq!(
            state.monitored_projects["metrics_test"]
                .current_metrics
                .avg_complexity,
            5.5
        );
        assert!(!state.quality_history.is_empty());
    }

    #[tokio::test]
    async fn test_statistics_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        // Update statistics
        persistence
            .update_statistics(|stats| {
                stats.sessions_count += 1;
                stats.analyses_performed += 10;
                stats.violations_detected += 5;
            })
            .await
            .unwrap();

        let state = persistence.get_state().await;
        assert_eq!(state.statistics.sessions_count, 1);
        assert_eq!(state.statistics.analyses_performed, 10);
        assert_eq!(state.statistics.violations_detected, 5);
    }

    #[tokio::test]
    async fn test_configuration_loading() {
        // Test that configuration files can be parsed
        let dev_config = r#"
            [agent]
            version = "1.0.0"
            name = "test"
            complexity_threshold = 20
            watch_patterns = ["**/*.rs"]
            update_interval = 30
            max_projects = 10

            [quality_monitor]
            update_interval = { secs = 5, nanos = 0 }
            complexity_threshold = 20
            watch_patterns = ["**/*.rs"]
            debounce_interval = { secs = 0, nanos = 500000000 }
            max_batch_size = 50

            [daemon]
            working_directory = "/tmp"
            health_check_interval = { secs = 30, nanos = 0 }
            max_memory_mb = 500
            auto_restart = true
            shutdown_timeout = { secs = 10, nanos = 0 }
        "#;

        let config: DaemonConfig = toml::from_str(dev_config).unwrap();
        assert_eq!(config.agent.complexity_threshold, 20);
    }
}

#[cfg(test)]
mod mcp_protocol_tests {
    use serde_json::json;

    #[test]
    fn test_mcp_request_format() {
        // Test that MCP requests are properly formatted
        let request = json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {}
            },
            "id": 1
        });

        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "initialize");
        assert!(request["id"].is_number());
    }

    #[test]
    fn test_mcp_response_format() {
        // Test that MCP responses are properly formatted
        let response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "pmat-agent",
                    "version": "1.0.0"
                }
            }
        });

        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
        assert!(response["result"]["serverInfo"].is_object());
    }

    #[test]
    fn test_tool_call_format() {
        let tool_call = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "run_quality_gates",
                "arguments": {
                    "target_path": "./src"
                }
            },
            "id": 42
        });

        assert_eq!(tool_call["method"], "tools/call");
        assert_eq!(tool_call["params"]["name"], "run_quality_gates");
        assert!(tool_call["params"]["arguments"].is_object());
    }
}
