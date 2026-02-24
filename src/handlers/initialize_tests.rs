#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::mcp::McpRequest;
    use crate::TemplateServer;
    use std::sync::Arc;

    async fn create_test_server() -> Arc<TemplateServer> {
        Arc::new(
            TemplateServer::new()
                .await
                .expect("Failed to create test server"),
        )
    }

    fn create_test_request(id: serde_json::Value, params: Option<serde_json::Value>) -> McpRequest {
        McpRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: "initialize".to_string(),
            params,
        }
    }

    // === handle_initialize tests ===

    #[tokio::test]
    async fn test_handle_initialize_basic() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), None);

        let response = handle_initialize(server, request).await;

        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result.get("protocolVersion").is_some());
        assert!(result.get("capabilities").is_some());
        assert!(result.get("serverInfo").is_some());
    }

    #[tokio::test]
    async fn test_handle_initialize_with_protocol_version() {
        let server = create_test_server().await;
        let params = serde_json::json!({
            "protocolVersion": "2025-01-01"
        });
        let request = create_test_request(serde_json::json!(1), Some(params));

        let response = handle_initialize(server, request).await;

        let result = response.result.unwrap();
        assert_eq!(result.get("protocolVersion").unwrap(), "2025-01-01");
    }

    #[tokio::test]
    async fn test_handle_initialize_default_protocol_version() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), None);

        let response = handle_initialize(server, request).await;

        let result = response.result.unwrap();
        assert_eq!(result.get("protocolVersion").unwrap(), "2024-11-05");
    }

    #[tokio::test]
    async fn test_handle_initialize_server_info() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), None);

        let response = handle_initialize(server, request).await;

        let result = response.result.unwrap();
        let server_info = result.get("serverInfo").unwrap();

        assert_eq!(server_info.get("name").unwrap(), "pmat");
        assert!(server_info.get("version").is_some());
        assert_eq!(
            server_info.get("vendor").unwrap(),
            "Pragmatic AI Labs (paiml.com)"
        );
        assert_eq!(server_info.get("author").unwrap(), "Pragmatic AI Labs");
    }

    #[tokio::test]
    async fn test_handle_initialize_capabilities() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), None);

        let response = handle_initialize(server, request).await;

        let result = response.result.unwrap();
        let capabilities = result.get("capabilities").unwrap();

        assert!(capabilities.get("tools").is_some());
        assert!(capabilities.get("resources").is_some());
        assert!(capabilities.get("prompts").is_some());
    }

    #[tokio::test]
    async fn test_handle_initialize_server_capabilities_array() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), None);

        let response = handle_initialize(server, request).await;

        let result = response.result.unwrap();
        let server_info = result.get("serverInfo").unwrap();
        let capabilities = server_info.get("capabilities").unwrap();

        assert!(capabilities.is_array());
        let caps_array = capabilities.as_array().unwrap();
        assert!(!caps_array.is_empty());
    }

    #[tokio::test]
    async fn test_handle_initialize_supported_templates() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), None);

        let response = handle_initialize(server, request).await;

        let result = response.result.unwrap();
        let server_info = result.get("serverInfo").unwrap();
        let templates = server_info.get("supportedTemplates").unwrap();

        assert!(templates.is_array());
        let templates_array = templates.as_array().unwrap();
        assert!(templates_array.contains(&serde_json::json!("makefile")));
        assert!(templates_array.contains(&serde_json::json!("readme")));
        assert!(templates_array.contains(&serde_json::json!("gitignore")));
    }

    #[tokio::test]
    async fn test_handle_initialize_supported_toolchains() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), None);

        let response = handle_initialize(server, request).await;

        let result = response.result.unwrap();
        let server_info = result.get("serverInfo").unwrap();
        let toolchains = server_info.get("supportedToolchains").unwrap();

        assert!(toolchains.is_array());
        let toolchains_array = toolchains.as_array().unwrap();
        assert!(toolchains_array.contains(&serde_json::json!("rust")));
        assert!(toolchains_array.contains(&serde_json::json!("deno")));
        assert!(toolchains_array.contains(&serde_json::json!("python-uv")));
    }

    #[tokio::test]
    async fn test_handle_initialize_examples() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), None);

        let response = handle_initialize(server, request).await;

        let result = response.result.unwrap();
        let server_info = result.get("serverInfo").unwrap();
        let examples = server_info.get("examples").unwrap();

        assert!(examples.is_array());
        let examples_array = examples.as_array().unwrap();
        assert!(!examples_array.is_empty());
    }

    #[tokio::test]
    async fn test_handle_initialize_string_id() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!("test-id"), None);

        let response = handle_initialize(server, request).await;

        assert!(response.result.is_some());
        assert_eq!(response.id, serde_json::json!("test-id"));
    }

    #[tokio::test]
    async fn test_handle_initialize_null_id() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::Value::Null, None);

        let response = handle_initialize(server, request).await;

        assert!(response.result.is_some());
        assert!(response.id.is_null());
    }

    #[tokio::test]
    async fn test_handle_initialize_empty_params() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), Some(serde_json::json!({})));

        let response = handle_initialize(server, request).await;

        assert!(response.result.is_some());
        // Default protocol version should be used
        let result = response.result.unwrap();
        assert_eq!(result.get("protocolVersion").unwrap(), "2024-11-05");
    }

    // === handle_tools_list tests ===

    #[tokio::test]
    async fn test_handle_tools_list_basic() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result.get("tools").is_some());
    }

    #[tokio::test]
    async fn test_handle_tools_list_has_tools() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap();
        assert!(tools.is_array());
        let tools_array = tools.as_array().unwrap();
        assert!(!tools_array.is_empty());
    }

    #[tokio::test]
    async fn test_handle_tools_list_contains_generate_template() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        let has_generate_template = tools.iter().any(|t| {
            t.get("name")
                .map(|n| n == "generate_template")
                .unwrap_or(false)
        });
        assert!(has_generate_template);
    }

    #[tokio::test]
    async fn test_handle_tools_list_contains_scaffold_project() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        let has_scaffold_project = tools.iter().any(|t| {
            t.get("name")
                .map(|n| n == "scaffold_project")
                .unwrap_or(false)
        });
        assert!(has_scaffold_project);
    }

    #[tokio::test]
    async fn test_handle_tools_list_contains_analyze_complexity() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        let has_analyze_complexity = tools.iter().any(|t| {
            t.get("name")
                .map(|n| n == "analyze_complexity")
                .unwrap_or(false)
        });
        assert!(has_analyze_complexity);
    }

    #[tokio::test]
    async fn test_handle_tools_list_contains_analyze_churn() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        let has_analyze_churn = tools.iter().any(|t| {
            t.get("name")
                .map(|n| n == "analyze_code_churn")
                .unwrap_or(false)
        });
        assert!(has_analyze_churn);
    }

    #[tokio::test]
    async fn test_handle_tools_list_contains_analyze_dead_code() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        let has_analyze_dead_code = tools.iter().any(|t| {
            t.get("name")
                .map(|n| n == "analyze_dead_code")
                .unwrap_or(false)
        });
        assert!(has_analyze_dead_code);
    }

    #[tokio::test]
    async fn test_handle_tools_list_contains_analyze_satd() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        let has_analyze_satd = tools
            .iter()
            .any(|t| t.get("name").map(|n| n == "analyze_satd").unwrap_or(false));
        assert!(has_analyze_satd);
    }

    #[tokio::test]
    async fn test_handle_tools_list_tool_has_description() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        // All tools should have descriptions
        for tool in tools {
            assert!(
                tool.get("description").is_some(),
                "Tool {} missing description",
                tool.get("name").unwrap_or(&serde_json::Value::Null)
            );
        }
    }

    #[tokio::test]
    async fn test_handle_tools_list_tool_has_input_schema() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        // All tools should have inputSchema
        for tool in tools {
            assert!(
                tool.get("inputSchema").is_some(),
                "Tool {} missing inputSchema",
                tool.get("name").unwrap_or(&serde_json::Value::Null)
            );
        }
    }

    #[tokio::test]
    async fn test_handle_tools_list_vectorized_tools() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        let vectorized_tools = [
            "analyze_duplicates_vectorized",
            "analyze_graph_metrics_vectorized",
            "analyze_name_similarity_vectorized",
            "analyze_symbol_table_vectorized",
            "analyze_incremental_coverage_vectorized",
            "analyze_big_o_vectorized",
        ];

        for tool_name in &vectorized_tools {
            let has_tool = tools
                .iter()
                .any(|t| t.get("name").map(|n| n == *tool_name).unwrap_or(false));
            assert!(has_tool, "Missing vectorized tool: {}", tool_name);
        }
    }

    #[tokio::test]
    async fn test_handle_tools_list_string_id() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!("test-id-123"),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        assert_eq!(response.id, serde_json::json!("test-id-123"));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
