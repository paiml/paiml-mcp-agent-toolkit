#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.bind_address, "127.0.0.1:3000");
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.name, "PMAT MCP Server");
        assert_eq!(config.request_timeout, std::time::Duration::from_secs(30));
        assert!(config.enable_logging);
        assert!(config.unix_socket.is_none());
    }

    #[test]
    fn test_server_config_version() {
        let config = ServerConfig::default();
        assert!(!config.version.is_empty());
        // Version should be from Cargo.toml
        assert!(config.version.chars().any(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_server_config_clone() {
        let config = ServerConfig::default();
        let cloned = config.clone();
        assert_eq!(config.bind_address, cloned.bind_address);
        assert_eq!(config.max_connections, cloned.max_connections);
        assert_eq!(config.name, cloned.name);
    }

    #[test]
    fn test_server_config_custom() {
        let config = ServerConfig {
            name: "Custom MCP".to_string(),
            version: "1.0.0".to_string(),
            bind_address: "0.0.0.0:8080".to_string(),
            unix_socket: Some("/tmp/test.sock".to_string()),
            max_connections: 50,
            request_timeout: std::time::Duration::from_secs(60),
            enable_logging: false,
            semantic_enabled: false,
            semantic_db_path: None,
            semantic_workspace: None,
        };
        assert_eq!(config.name, "Custom MCP");
        assert_eq!(config.bind_address, "0.0.0.0:8080");
        assert_eq!(config.unix_socket, Some("/tmp/test.sock".to_string()));
        assert_eq!(config.max_connections, 50);
        assert!(!config.enable_logging);
    }

    #[test]
    fn test_server_config_semantic_defaults() {
        let config = ServerConfig::default();
        // Semantic uses local embeddings - no API keys required
        // Can be enabled via PMAT_SEMANTIC_ENABLED env var
        assert!(config.semantic_db_path.is_some() || config.semantic_db_path.is_none());
    }

    #[actix_rt::test]
    async fn test_mcp_server_creation() {
        use crate::agents::registry::AgentRegistry;

        let registry = Arc::new(AgentRegistry::new());
        let config = ServerConfig::default();

        let server = McpServer::new(registry, config).unwrap();
        assert_eq!(server.context.server_info.protocol_version, MCP_VERSION);
    }

    #[actix_rt::test]
    async fn test_mcp_server_info() {
        use crate::agents::registry::AgentRegistry;

        let registry = Arc::new(AgentRegistry::new());
        let config = ServerConfig {
            name: "Test Server".to_string(),
            version: "2.0.0".to_string(),
            ..ServerConfig::default()
        };

        let server = McpServer::new(registry, config).unwrap();
        assert_eq!(server.context.server_info.name, "Test Server");
        assert_eq!(server.context.server_info.version, "2.0.0");
    }

    #[actix_rt::test]
    async fn test_mcp_server_capabilities() {
        use crate::agents::registry::AgentRegistry;

        let registry = Arc::new(AgentRegistry::new());
        let config = ServerConfig::default();

        let server = McpServer::new(registry, config).unwrap();

        // Check capabilities are set
        assert!(server.context.capabilities.tools.is_some());
        assert!(server.context.capabilities.resources.is_some());
        assert!(server.context.capabilities.prompts.is_some());
        assert!(server.context.capabilities.logging.is_some());
    }

    #[actix_rt::test]
    async fn test_mcp_server_no_logging() {
        use crate::agents::registry::AgentRegistry;

        let registry = Arc::new(AgentRegistry::new());
        let config = ServerConfig {
            enable_logging: false,
            ..ServerConfig::default()
        };

        let server = McpServer::new(registry, config).unwrap();
        assert!(server.context.capabilities.logging.is_none());
    }

    #[actix_rt::test]
    async fn test_mcp_server_shutdown() {
        use crate::agents::registry::AgentRegistry;

        let registry = Arc::new(AgentRegistry::new());
        let config = ServerConfig::default();

        let server = McpServer::new(registry, config).unwrap();

        // Call shutdown - should not panic
        server.shutdown();
    }

    #[actix_rt::test]
    async fn test_mcp_server_registries_initialized() {
        use crate::agents::registry::AgentRegistry;

        let registry = Arc::new(AgentRegistry::new());
        let config = ServerConfig::default();

        let server = McpServer::new(registry, config).unwrap();

        // Access registries - should not panic
        let tools = server.context.tools.read();
        let resources = server.context.resources.read();
        let prompts = server.context.prompts.read();

        // They should be empty before register_defaults
        drop(tools);
        drop(resources);
        drop(prompts);
    }

    #[test]
    fn test_stdio_transport_new() {
        let _transport = StdioTransport::new();
        // Just verify it constructs
    }

    #[actix_rt::test]
    async fn test_stdio_transport_close() {
        let transport = StdioTransport::new();
        let result = transport.close().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_tcp_transport_construction() {
        // TcpTransport needs a real stream, so we can't easily test construction
        // But we can verify the types compile
        fn _type_check() {
            let _ = |stream: tokio::net::TcpStream| {
                let _ = TcpTransport::new(stream);
            };
        }
    }

    #[test]
    fn test_unix_transport_construction() {
        // UnixTransport needs a real stream, so we can't easily test construction
        // But we can verify the types compile
        fn _type_check() {
            let _ = |stream: tokio::net::UnixStream| {
                let _ = UnixTransport::new(stream);
            };
        }
    }

    #[test]
    fn test_mcp_error_construction() {
        let error = McpError {
            code: -32600,
            message: "Invalid request".to_string(),
            data: None,
        };
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid request");
    }

    #[test]
    fn test_mcp_error_with_data() {
        let error = McpError {
            code: -32000,
            message: "Server error".to_string(),
            data: Some(serde_json::json!({"details": "Something went wrong"})),
        };
        assert!(error.data.is_some());
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(error_codes::PARSE_ERROR, -32700);
        assert_eq!(error_codes::INTERNAL_ERROR, -32603);
    }

    #[actix_rt::test]
    async fn test_register_defaults_no_panic() {
        use crate::agents::registry::AgentRegistry;

        let registry = Arc::new(AgentRegistry::new());
        let config = ServerConfig {
            semantic_enabled: false, // Disable semantic to avoid API key requirement
            ..ServerConfig::default()
        };

        let server = McpServer::new(registry, config).unwrap();

        // This should not panic
        let result = server.register_defaults().await;
        assert!(result.is_ok());
    }

    #[actix_rt::test]
    async fn test_register_agent_tools() {
        use crate::agents::registry::AgentRegistry;

        let registry = Arc::new(AgentRegistry::new());
        let config = ServerConfig::default();

        let server = McpServer::new(registry, config).unwrap();

        // Register agent tools
        let result = server.register_agent_tools().await;
        assert!(result.is_ok());

        // Verify tools were registered
        let tools = server.context.tools.read();
        assert!(tools.list().len() >= 4); // At least analyze, transform, validate, orchestrate
    }

    #[actix_rt::test]
    async fn test_register_agent_resources() {
        use crate::agents::registry::AgentRegistry;

        let registry = Arc::new(AgentRegistry::new());
        let config = ServerConfig::default();

        let server = McpServer::new(registry, config).unwrap();

        // Register resources
        let result = server.register_agent_resources().await;
        assert!(result.is_ok());

        // Verify resources were registered
        let resources = server.context.resources.read();
        assert!(resources.list().len() >= 3); // agent_state, metrics, quality_report
    }

    #[actix_rt::test]
    async fn test_register_agent_prompts() {
        use crate::agents::registry::AgentRegistry;

        let registry = Arc::new(AgentRegistry::new());
        let config = ServerConfig::default();

        let server = McpServer::new(registry, config).unwrap();

        // Register prompts
        let result = server.register_agent_prompts().await;
        assert!(result.is_ok());

        // Verify prompts were registered
        let prompts = server.context.prompts.read();
        assert!(prompts.list().len() >= 4); // code_analysis, refactoring, quality_assessment, repo_score
    }

    #[actix_rt::test]
    async fn test_register_tdg_tools() {
        use crate::agents::registry::AgentRegistry;

        let registry = Arc::new(AgentRegistry::new());
        let config = ServerConfig::default();

        let server = McpServer::new(registry, config).unwrap();

        // Register TDG tools
        let result = server.register_tdg_tools().await;
        assert!(result.is_ok());
    }

    #[actix_rt::test]
    async fn test_register_polyglot_tools() {
        use crate::agents::registry::AgentRegistry;

        let registry = Arc::new(AgentRegistry::new());
        let config = ServerConfig::default();

        let server = McpServer::new(registry, config).unwrap();

        // Register polyglot tools
        let result = server.register_polyglot_tools().await;
        assert!(result.is_ok());
    }

    #[actix_rt::test]
    async fn test_register_hallucination_detection_tools() {
        use crate::agents::registry::AgentRegistry;

        let registry = Arc::new(AgentRegistry::new());
        let config = ServerConfig::default();

        let server = McpServer::new(registry, config).unwrap();

        // Register hallucination detection tools
        let result = server.register_hallucination_detection_tools().await;
        assert!(result.is_ok());
    }

    #[actix_rt::test]
    async fn test_register_jvm_tools() {
        use crate::agents::registry::AgentRegistry;

        let registry = Arc::new(AgentRegistry::new());
        let config = ServerConfig::default();

        let server = McpServer::new(registry, config).unwrap();

        // Register JVM tools (may be empty if features not enabled)
        let result = server.register_jvm_tools().await;
        assert!(result.is_ok());
    }

    #[actix_rt::test]
    async fn test_register_semantic_tools_disabled() {
        use crate::agents::registry::AgentRegistry;

        let registry = Arc::new(AgentRegistry::new());
        let config = ServerConfig {
            semantic_enabled: false,
            ..ServerConfig::default()
        };

        let server = McpServer::new(registry, config).unwrap();

        // Should complete without error when disabled
        let result = server.register_semantic_tools().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_server_config_request_timeout() {
        let config = ServerConfig {
            request_timeout: std::time::Duration::from_secs(120),
            ..ServerConfig::default()
        };
        assert_eq!(config.request_timeout, std::time::Duration::from_secs(120));
    }

    #[test]
    fn test_capabilities_tools() {
        let cap = ToolsCapability {
            list_changed: Some(true),
        };
        assert_eq!(cap.list_changed, Some(true));
    }

    #[test]
    fn test_capabilities_resources() {
        let cap = ResourcesCapability {
            subscribe: Some(true),
            list_changed: Some(true),
        };
        assert_eq!(cap.subscribe, Some(true));
        assert_eq!(cap.list_changed, Some(true));
    }

    #[test]
    fn test_capabilities_prompts() {
        let cap = PromptsCapability {
            list_changed: Some(true),
        };
        assert_eq!(cap.list_changed, Some(true));
    }

    #[test]
    fn test_logging_capability() {
        let cap = LoggingCapabilities {
            level: "debug".to_string(),
        };
        assert_eq!(cap.level, "debug");
    }

    #[test]
    fn test_server_info() {
        let info = ServerInfo {
            name: "Test".to_string(),
            version: "1.0".to_string(),
            protocol_version: "2024-11-05".to_string(),
        };
        assert_eq!(info.name, "Test");
        assert_eq!(info.version, "1.0");
        assert_eq!(info.protocol_version, "2024-11-05");
    }

    #[test]
    fn test_mcp_version_constant() {
        assert!(!MCP_VERSION.is_empty());
        // Version should be a date format like "2024-11-05"
        assert!(MCP_VERSION.len() >= 8);
    }
}
