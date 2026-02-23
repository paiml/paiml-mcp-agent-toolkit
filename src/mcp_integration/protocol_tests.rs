#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(test)]
mod tests {
    use crate::mcp_integration::registry::{
        PromptRegistry, ResourceContent, ResourceContentType, ResourceRegistry, ResourceTemplate,
        ToolRegistry,
    };
    use crate::mcp_integration::types::{
        error_codes, McpError, ServerInfo, MCP_VERSION,
    };

    #[test]
    fn test_server_info() {
        let info = ServerInfo::default();
        assert_eq!(info.protocol_version, MCP_VERSION);
        assert_eq!(info.name, "PMAT Agent Server");
    }

    #[test]
    fn test_tool_registry() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_resource_registry() {
        let registry = ResourceRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_prompt_registry() {
        let registry = PromptRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use crate::mcp_integration::registry::{
        ContentPart, McpPrompt, McpResource, McpTool, PromptArgument, PromptContent,
        PromptMessage, PromptMetadata, PromptRegistry, ResourceContent, ResourceContentType,
        ResourceRegistry, ResourceTemplate, ToolMetadata, ToolRegistry,
    };
    use crate::mcp_integration::types::{
        error_codes, McpError, McpNotification, McpRequest, McpResponse, RequestId,
        ServerCapabilities, ServerInfo, ToolsCapability, PromptsCapability, ResourcesCapability,
        LoggingCapabilities, MCP_VERSION,
    };
    use async_trait::async_trait;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    // ============================================================
    // Test Fixtures
    // ============================================================

    /// A mock MCP tool for testing
    struct MockTool {
        name: String,
        description: String,
        execute_count: AtomicUsize,
    }

    impl MockTool {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                description: format!("Mock tool: {}", name),
                execute_count: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl McpTool for MockTool {
        fn metadata(&self) -> ToolMetadata {
            ToolMetadata {
                name: self.name.clone(),
                description: self.description.clone(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": { "type": "string" }
                    }
                }),
            }
        }

        async fn execute(&self, params: Value) -> Result<Value, McpError> {
            self.execute_count.fetch_add(1, Ordering::SeqCst);
            Ok(serde_json::json!({
                "result": "executed",
                "params": params
            }))
        }
    }

    /// A mock MCP resource for testing
    struct MockResource {
        uri_template: String,
        name: String,
    }

    impl MockResource {
        fn new(uri_template: &str, name: &str) -> Self {
            Self {
                uri_template: uri_template.to_string(),
                name: name.to_string(),
            }
        }
    }

    #[async_trait]
    impl McpResource for MockResource {
        fn template(&self) -> ResourceTemplate {
            ResourceTemplate {
                uri_template: self.uri_template.clone(),
                name: self.name.clone(),
                description: Some(format!("Mock resource: {}", self.name)),
                mime_type: Some("text/plain".to_string()),
            }
        }

        async fn read(&self, uri: &str) -> Result<ResourceContent, McpError> {
            Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: Some("text/plain".to_string()),
                content: ResourceContentType::Text {
                    text: format!("Content for {}", uri),
                },
            })
        }

        fn subscribe(&self, _uri: &str) -> Option<tokio::sync::watch::Receiver<ResourceContent>> {
            None
        }
    }

    /// A mock MCP prompt for testing
    struct MockPrompt {
        name: String,
        description: String,
    }

    impl MockPrompt {
        fn new(name: &str, description: &str) -> Self {
            Self {
                name: name.to_string(),
                description: description.to_string(),
            }
        }
    }

    #[async_trait]
    impl McpPrompt for MockPrompt {
        fn metadata(&self) -> PromptMetadata {
            PromptMetadata {
                name: self.name.clone(),
                description: Some(self.description.clone()),
                arguments: Some(vec![PromptArgument {
                    name: "context".to_string(),
                    description: Some("Context for the prompt".to_string()),
                    required: Some(true),
                }]),
            }
        }

        async fn get(
            &self,
            arguments: Option<HashMap<String, String>>,
        ) -> Result<Vec<PromptMessage>, McpError> {
            let context = arguments
                .as_ref()
                .and_then(|a| a.get("context"))
                .map(|s| s.as_str())
                .unwrap_or("default");

            Ok(vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text(format!("Prompt with context: {}", context)),
            }])
        }
    }

    // ============================================================
    // ServerInfo Tests
    // ============================================================

    #[test]
    fn test_server_info_default_values() {
        let info = ServerInfo::default();
        assert_eq!(info.name, "PMAT Agent Server");
        assert_eq!(info.protocol_version, MCP_VERSION);
        assert!(!info.version.is_empty());
    }

    #[test]
    fn test_server_info_clone() {
        let info = ServerInfo::default();
        let cloned = info.clone();
        assert_eq!(info.name, cloned.name);
        assert_eq!(info.version, cloned.version);
        assert_eq!(info.protocol_version, cloned.protocol_version);
    }

    #[test]
    fn test_server_info_debug() {
        let info = ServerInfo::default();
        let debug = format!("{:?}", info);
        assert!(debug.contains("ServerInfo"));
        assert!(debug.contains("PMAT"));
    }

    // ============================================================
    // ToolRegistry Tests
    // ============================================================

    #[test]
    fn test_tool_registry_new_is_empty() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_tool_registry_default_is_empty() {
        let registry = ToolRegistry::default();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_tool_registry_register_single() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(MockTool::new("test_tool"));

        registry.register(tool);

        assert_eq!(registry.list().len(), 1);
        assert!(registry.get("test_tool").is_some());
    }

    #[test]
    fn test_tool_registry_register_multiple() {
        let mut registry = ToolRegistry::new();

        for i in 0..5 {
            let tool = Arc::new(MockTool::new(&format!("tool_{}", i)));
            registry.register(tool);
        }

        assert_eq!(registry.list().len(), 5);

        for i in 0..5 {
            assert!(registry.get(&format!("tool_{}", i)).is_some());
        }
    }

    #[test]
    fn test_tool_registry_get_nonexistent() {
        let registry = ToolRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_tool_registry_list_returns_metadata() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(MockTool::new("my_tool"));
        registry.register(tool);

        let list = registry.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "my_tool");
        assert!(list[0].description.contains("Mock tool"));
    }

    // ============================================================
    // ResourceRegistry Tests
    // ============================================================

    #[test]
    fn test_resource_registry_new_is_empty() {
        let registry = ResourceRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_resource_registry_default_is_empty() {
        let registry = ResourceRegistry::default();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_resource_registry_register_single() {
        let mut registry = ResourceRegistry::new();
        let resource = Arc::new(MockResource::new("file:///{}", "files"));

        registry.register(resource);

        assert_eq!(registry.list().len(), 1);
        assert!(registry.get("file:///{}").is_some());
    }

    #[test]
    fn test_resource_registry_register_multiple() {
        let mut registry = ResourceRegistry::new();

        registry.register(Arc::new(MockResource::new("file:///{}", "files")));
        registry.register(Arc::new(MockResource::new("git:///{}", "git")));
        registry.register(Arc::new(MockResource::new("http:///{}", "http")));

        assert_eq!(registry.list().len(), 3);
    }

    #[test]
    fn test_resource_registry_get_nonexistent() {
        let registry = ResourceRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_resource_registry_find_matching() {
        let mut registry = ResourceRegistry::new();
        registry.register(Arc::new(MockResource::new("file:///{}", "files")));

        // Should find matching
        let found = registry.find_matching("file:///path/to/file");
        assert!(found.is_some());

        // Should not find non-matching
        let not_found = registry.find_matching("git:///repo");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_resource_registry_list_returns_templates() {
        let mut registry = ResourceRegistry::new();
        registry.register(Arc::new(MockResource::new("file:///{}", "files")));

        let list = registry.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "files");
        assert_eq!(list[0].uri_template, "file:///{}");
    }

    // ============================================================
    // PromptRegistry Tests
    // ============================================================

    #[test]
    fn test_prompt_registry_new_is_empty() {
        let registry = PromptRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_prompt_registry_default_is_empty() {
        let registry = PromptRegistry::default();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_prompt_registry_register_single() {
        let mut registry = PromptRegistry::new();
        let prompt = Arc::new(MockPrompt::new("test_prompt", "A test prompt"));

        registry.register(prompt);

        assert_eq!(registry.list().len(), 1);
        assert!(registry.get("test_prompt").is_some());
    }

    #[test]
    fn test_prompt_registry_register_multiple() {
        let mut registry = PromptRegistry::new();

        for i in 0..5 {
            let prompt = Arc::new(MockPrompt::new(
                &format!("prompt_{}", i),
                &format!("Description {}", i),
            ));
            registry.register(prompt);
        }

        assert_eq!(registry.list().len(), 5);
    }

    #[test]
    fn test_prompt_registry_get_nonexistent() {
        let registry = PromptRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_prompt_registry_list_returns_metadata() {
        let mut registry = PromptRegistry::new();
        registry.register(Arc::new(MockPrompt::new("my_prompt", "My description")));

        let list = registry.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "my_prompt");
        assert_eq!(list[0].description, Some("My description".to_string()));
    }

    // ============================================================
    // RequestId Tests
    // ============================================================

    #[test]
    fn test_request_id_string() {
        let id = RequestId::String("abc-123".to_string());
        let json = serde_json::to_string(&id).unwrap();
        assert!(json.contains("abc-123"));
    }

    #[test]
    fn test_request_id_number() {
        let id = RequestId::Number(42);
        let json = serde_json::to_string(&id).unwrap();
        assert!(json.contains("42"));
    }

    #[test]
    fn test_request_id_deserialize_string() {
        let json = r#""request-id""#;
        let id: RequestId = serde_json::from_str(json).unwrap();
        match id {
            RequestId::String(s) => assert_eq!(s, "request-id"),
            _ => panic!("Expected string id"),
        }
    }

    #[test]
    fn test_request_id_deserialize_number() {
        let json = "123";
        let id: RequestId = serde_json::from_str(json).unwrap();
        match id {
            RequestId::Number(n) => assert_eq!(n, 123),
            _ => panic!("Expected number id"),
        }
    }

    // ============================================================
    // McpError Tests
    // ============================================================

    #[test]
    fn test_mcp_error_display() {
        let error = McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing parameter".to_string(),
            data: None,
        };

        let display = format!("{}", error);
        assert!(display.contains("MCP Error"));
        assert!(display.contains(&error_codes::INVALID_PARAMS.to_string()));
        assert!(display.contains("Missing parameter"));
    }

    #[test]
    fn test_mcp_error_with_data() {
        let error = McpError {
            code: error_codes::INTERNAL_ERROR,
            message: "Something went wrong".to_string(),
            data: Some(serde_json::json!({ "details": "extra info" })),
        };

        assert_eq!(error.code, error_codes::INTERNAL_ERROR);
        assert!(error.data.is_some());
    }

    #[test]
    fn test_mcp_error_clone() {
        let error = McpError {
            code: error_codes::METHOD_NOT_FOUND,
            message: "Not found".to_string(),
            data: None,
        };

        let cloned = error.clone();
        assert_eq!(error.code, cloned.code);
        assert_eq!(error.message, cloned.message);
    }

    // ============================================================
    // Error Codes Tests
    // ============================================================

    #[test]
    fn test_error_codes_values() {
        assert_eq!(error_codes::PARSE_ERROR, -32700);
        assert_eq!(error_codes::INVALID_REQUEST, -32600);
        assert_eq!(error_codes::METHOD_NOT_FOUND, -32601);
        assert_eq!(error_codes::INVALID_PARAMS, -32602);
        assert_eq!(error_codes::INTERNAL_ERROR, -32603);
    }

    // ============================================================
    // McpRequest Tests
    // ============================================================

    #[test]
    fn test_mcp_request_serialize() {
        let request = McpRequest {
            id: RequestId::Number(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("tools/list"));
    }

    #[test]
    fn test_mcp_request_with_params() {
        let request = McpRequest {
            id: RequestId::String("req-1".to_string()),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "analyze",
                "arguments": { "path": "/src" }
            })),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("tools/call"));
        assert!(json.contains("analyze"));
    }

    #[test]
    fn test_mcp_request_deserialize() {
        let json = r#"{"id":1,"method":"initialize","params":null}"#;
        let request: McpRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.method, "initialize");
        match request.id {
            RequestId::Number(n) => assert_eq!(n, 1),
            _ => panic!("Expected number id"),
        }
    }

    // ============================================================
    // McpResponse Tests
    // ============================================================

    #[test]
    fn test_mcp_response_success() {
        let response = McpResponse {
            id: RequestId::Number(1),
            result: Some(serde_json::json!({ "success": true })),
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("success"));
        assert!(!json.contains("error"));
    }

    #[test]
    fn test_mcp_response_error() {
        let response = McpResponse {
            id: RequestId::Number(1),
            result: None,
            error: Some(McpError {
                code: error_codes::INTERNAL_ERROR,
                message: "Failed".to_string(),
                data: None,
            }),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("Failed"));
    }

    // ============================================================
    // McpNotification Tests
    // ============================================================

    #[test]
    fn test_mcp_notification_serialize() {
        let notification = McpNotification {
            method: "notifications/resources/updated".to_string(),
            params: Some(serde_json::json!({ "uri": "file:///test" })),
        };

        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("notifications/resources/updated"));
        assert!(json.contains("file:///test"));
    }

    #[test]
    fn test_mcp_notification_without_params() {
        let notification = McpNotification {
            method: "initialized".to_string(),
            params: None,
        };

        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("initialized"));
    }

    // ============================================================
    // ServerCapabilities Tests
    // ============================================================

    #[test]
    fn test_server_capabilities_serialize() {
        let capabilities = ServerCapabilities {
            experimental: None,
            logging: Some(LoggingCapabilities {
                level: "info".to_string(),
            }),
            prompts: Some(PromptsCapability {
                list_changed: Some(true),
            }),
            resources: Some(ResourcesCapability {
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
        };

        let json = serde_json::to_string(&capabilities).unwrap();
        assert!(json.contains("logging"));
        assert!(json.contains("prompts"));
        assert!(json.contains("resources"));
        assert!(json.contains("tools"));
    }

    #[test]
    fn test_server_capabilities_minimal() {
        let capabilities = ServerCapabilities {
            experimental: None,
            logging: None,
            prompts: None,
            resources: None,
            tools: None,
        };

        let json = serde_json::to_string(&capabilities).unwrap();
        // All None fields should be serializable
        assert!(!json.is_empty());
    }

    // ============================================================
    // ResourceContent Tests
    // ============================================================

    #[test]
    fn test_resource_content_text() {
        let content = ResourceContent {
            uri: "file:///test.txt".to_string(),
            mime_type: Some("text/plain".to_string()),
            content: ResourceContentType::Text {
                text: "Hello, World!".to_string(),
            },
        };

        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("file:///test.txt"));
        assert!(json.contains("Hello, World!"));
    }

    #[test]
    fn test_resource_content_blob() {
        let content = ResourceContent {
            uri: "file:///image.png".to_string(),
            mime_type: Some("image/png".to_string()),
            content: ResourceContentType::Blob {
                blob: "aGVsbG8=".to_string(), // base64 "hello"
            },
        };

        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("image/png"));
        assert!(json.contains("aGVsbG8="));
    }

    // ============================================================
    // ResourceTemplate Tests
    // ============================================================

    #[test]
    fn test_resource_template_serialize() {
        let template = ResourceTemplate {
            uri_template: "file:///{path}".to_string(),
            name: "files".to_string(),
            description: Some("File system access".to_string()),
            mime_type: Some("text/plain".to_string()),
        };

        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("file:///{path}"));
        assert!(json.contains("files"));
    }

    #[test]
    fn test_resource_template_minimal() {
        let template = ResourceTemplate {
            uri_template: "test://".to_string(),
            name: "test".to_string(),
            description: None,
            mime_type: None,
        };

        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("test://"));
    }

    // ============================================================
    // PromptMetadata Tests
    // ============================================================

    #[test]
    fn test_prompt_metadata_full() {
        let metadata = PromptMetadata {
            name: "code_review".to_string(),
            description: Some("Review code for issues".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "file".to_string(),
                    description: Some("File to review".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "depth".to_string(),
                    description: Some("Review depth".to_string()),
                    required: Some(false),
                },
            ]),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("code_review"));
        assert!(json.contains("file"));
        assert!(json.contains("depth"));
    }

    #[test]
    fn test_prompt_metadata_minimal() {
        let metadata = PromptMetadata {
            name: "simple".to_string(),
            description: None,
            arguments: None,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("simple"));
    }

    // ============================================================
    // PromptMessage Tests
    // ============================================================

    #[test]
    fn test_prompt_message_text() {
        let message = PromptMessage {
            role: "user".to_string(),
            content: PromptContent::Text("Hello".to_string()),
        };

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_prompt_message_parts() {
        let message = PromptMessage {
            role: "assistant".to_string(),
            content: PromptContent::Parts(vec![
                ContentPart::Text {
                    text: "Here's the code:".to_string(),
                },
                ContentPart::Resource {
                    uri: "file:///code.rs".to_string(),
                },
            ]),
        };

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("assistant"));
        assert!(json.contains("Here's the code"));
        assert!(json.contains("file:///code.rs"));
    }

    // ============================================================
    // ContentPart Tests
    // ============================================================

    #[test]
    fn test_content_part_text() {
        let part = ContentPart::Text {
            text: "Some text".to_string(),
        };

        let json = serde_json::to_string(&part).unwrap();
        assert!(json.contains("text"));
        assert!(json.contains("Some text"));
    }

    #[test]
    fn test_content_part_image() {
        let part = ContentPart::Image {
            data: "base64data".to_string(),
            mime_type: "image/png".to_string(),
        };

        let json = serde_json::to_string(&part).unwrap();
        assert!(json.contains("image"));
        assert!(json.contains("base64data"));
        assert!(json.contains("image/png"));
    }

    #[test]
    fn test_content_part_resource() {
        let part = ContentPart::Resource {
            uri: "file:///path".to_string(),
        };

        let json = serde_json::to_string(&part).unwrap();
        assert!(json.contains("resource"));
        assert!(json.contains("file:///path"));
    }

    // ============================================================
    // ToolMetadata Tests
    // ============================================================

    #[test]
    fn test_tool_metadata_serialize() {
        let metadata = ToolMetadata {
            name: "analyze".to_string(),
            description: "Analyze code".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                }
            }),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("analyze"));
        assert!(json.contains("Analyze code"));
        assert!(json.contains("properties"));
    }

    #[test]
    fn test_tool_metadata_clone() {
        let metadata = ToolMetadata {
            name: "test".to_string(),
            description: "desc".to_string(),
            input_schema: serde_json::json!({}),
        };

        let cloned = metadata.clone();
        assert_eq!(metadata.name, cloned.name);
        assert_eq!(metadata.description, cloned.description);
    }

    // ============================================================
    // MCP_VERSION Constant Tests
    // ============================================================

    #[test]
    fn test_mcp_version_format() {
        // Version should be in date format
        assert!(MCP_VERSION.contains("-"));
        let parts: Vec<&str> = MCP_VERSION.split('-').collect();
        assert_eq!(parts.len(), 3);

        // Year should be 2024+
        let year: i32 = parts[0].parse().unwrap();
        assert!(year >= 2024);
    }

    // ============================================================
    // Property-Based Tests
    // ============================================================

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_tool_registry_preserves_tools(count in 1usize..20) {
                let mut registry = ToolRegistry::new();

                for i in 0..count {
                    let tool = Arc::new(MockTool::new(&format!("tool_{}", i)));
                    registry.register(tool);
                }

                prop_assert_eq!(registry.list().len(), count);
            }

            #[test]
            fn test_resource_registry_preserves_resources(count in 1usize..20) {
                let mut registry = ResourceRegistry::new();

                for i in 0..count {
                    let resource = Arc::new(MockResource::new(
                        &format!("scheme_{}:///{{}}", i),
                        &format!("resource_{}", i)
                    ));
                    registry.register(resource);
                }

                prop_assert_eq!(registry.list().len(), count);
            }

            #[test]
            fn test_prompt_registry_preserves_prompts(count in 1usize..20) {
                let mut registry = PromptRegistry::new();

                for i in 0..count {
                    let prompt = Arc::new(MockPrompt::new(
                        &format!("prompt_{}", i),
                        &format!("Description {}", i)
                    ));
                    registry.register(prompt);
                }

                prop_assert_eq!(registry.list().len(), count);
            }

            #[test]
            fn test_request_id_roundtrip_number(n in 0i64..10000) {
                let id = RequestId::Number(n);
                let json = serde_json::to_string(&id).unwrap();
                let parsed: RequestId = serde_json::from_str(&json).unwrap();

                match parsed {
                    RequestId::Number(parsed_n) => prop_assert_eq!(n, parsed_n),
                    _ => prop_assert!(false, "Expected number"),
                }
            }

            #[test]
            fn test_request_id_roundtrip_string(s in "[a-z0-9-]{1,20}") {
                let id = RequestId::String(s.clone());
                let json = serde_json::to_string(&id).unwrap();
                let parsed: RequestId = serde_json::from_str(&json).unwrap();

                match parsed {
                    RequestId::String(parsed_s) => prop_assert_eq!(s, parsed_s),
                    _ => prop_assert!(false, "Expected string"),
                }
            }

            #[test]
            fn test_mcp_error_preserves_code(code in -50000i32..0) {
                let error = McpError {
                    code,
                    message: "test".to_string(),
                    data: None,
                };

                let json = serde_json::to_string(&error).unwrap();
                let parsed: McpError = serde_json::from_str(&json).unwrap();

                prop_assert_eq!(error.code, parsed.code);
            }

            #[test]
            fn test_tool_metadata_name_preserved(name in "[a-z_]{1,30}") {
                let metadata = ToolMetadata {
                    name: name.clone(),
                    description: "test".to_string(),
                    input_schema: serde_json::json!({}),
                };

                let json = serde_json::to_string(&metadata).unwrap();
                let parsed: ToolMetadata = serde_json::from_str(&json).unwrap();

                prop_assert_eq!(name, parsed.name);
            }
        }
    }

    // ============================================================
    // Edge Cases
    // ============================================================

    #[test]
    fn test_empty_method_name() {
        let request = McpRequest {
            id: RequestId::Number(1),
            method: "".to_string(),
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: McpRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.method, "");
    }

    #[test]
    fn test_very_long_error_message() {
        let long_message = "x".repeat(10000);
        let error = McpError {
            code: error_codes::INTERNAL_ERROR,
            message: long_message.clone(),
            data: None,
        };

        let display = format!("{}", error);
        assert!(display.contains(&long_message[..100])); // Contains at least start
    }

    #[test]
    fn test_special_characters_in_strings() {
        let notification = McpNotification {
            method: "test\n\t\"special".to_string(),
            params: Some(serde_json::json!({
                "unicode": "\u{1F600}",
                "newline": "line1\nline2"
            })),
        };

        let json = serde_json::to_string(&notification).unwrap();
        let parsed: McpNotification = serde_json::from_str(&json).unwrap();
        assert!(parsed.method.contains("special"));
    }

    #[test]
    fn test_nested_json_in_params() {
        let request = McpRequest {
            id: RequestId::Number(1),
            method: "complex".to_string(),
            params: Some(serde_json::json!({
                "level1": {
                    "level2": {
                        "level3": {
                            "level4": [1, 2, 3]
                        }
                    }
                }
            })),
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: McpRequest = serde_json::from_str(&json).unwrap();
        assert!(parsed.params.is_some());
    }

    #[test]
    fn test_null_values_in_params() {
        let request = McpRequest {
            id: RequestId::Number(1),
            method: "test".to_string(),
            params: Some(serde_json::json!({
                "key": null,
                "array": [null, null],
                "nested": { "nullval": null }
            })),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("null"));
    }
}
