    /// Test Debug implementation for McpRequest
    #[test]
    fn test_mcp_request_debug() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "test".to_string(),
            params: None,
        };

        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("McpRequest"));
        assert!(debug_str.contains("test"));
    }

    /// Test Debug implementation for McpResponse
    #[test]
    fn test_mcp_response_debug() {
        let response = McpResponse::success(json!(1), json!({"ok": true}));

        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("McpResponse"));
    }

    /// Test Debug implementation for McpError
    #[test]
    fn test_mcp_error_debug() {
        let error = McpError {
            code: -32600,
            message: "Invalid request".to_string(),
            data: None,
        };

        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("McpError"));
        assert!(debug_str.contains("-32600"));
    }

    /// Test Debug implementation for ToolCallParams
    #[test]
    fn test_tool_call_params_debug() {
        let params = ToolCallParams {
            name: "analyze".to_string(),
            arguments: json!({}),
        };

        let debug_str = format!("{:?}", params);
        assert!(debug_str.contains("analyze"));
    }

    /// Test Debug implementation for GenerateTemplateArgs
    #[test]
    fn test_generate_template_args_debug() {
        let args = GenerateTemplateArgs {
            resource_uri: "test://uri".to_string(),
            parameters: serde_json::Map::new(),
        };

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("test://uri"));
    }

    /// Test Debug implementation for ListTemplatesArgs
    #[test]
    fn test_list_templates_args_debug() {
        let args = ListTemplatesArgs {
            toolchain: Some("rust".to_string()),
            category: None,
        };

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("rust"));
    }

    /// Test Debug implementation for ResourceReadParams
    #[test]
    fn test_resource_read_params_debug() {
        let params = ResourceReadParams {
            uri: "template://test".to_string(),
        };

        let debug_str = format!("{:?}", params);
        assert!(debug_str.contains("template://test"));
    }

    /// Test Debug implementation for ValidateTemplateArgs
    #[test]
    fn test_validate_template_args_debug() {
        let args = ValidateTemplateArgs {
            resource_uri: "validate://test".to_string(),
            parameters: serde_json::Map::new(),
        };

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("validate://test"));
    }

    /// Test Debug implementation for ScaffoldProjectArgs
    #[test]
    fn test_scaffold_project_args_debug() {
        let args = ScaffoldProjectArgs {
            toolchain: "rust".to_string(),
            templates: vec!["makefile".to_string()],
            parameters: serde_json::Map::new(),
        };

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("rust"));
        assert!(debug_str.contains("makefile"));
    }

    /// Test Debug implementation for SearchTemplatesArgs
    #[test]
    fn test_search_templates_args_debug() {
        let args = SearchTemplatesArgs {
            query: "search query".to_string(),
            toolchain: None,
        };

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("search query"));
    }

    /// Test Debug implementation for PromptGetParams
    #[test]
    fn test_prompt_get_params_debug() {
        let params = PromptGetParams {
            name: "prompt-name".to_string(),
        };

        let debug_str = format!("{:?}", params);
        assert!(debug_str.contains("prompt-name"));
    }

    /// Test Debug implementation for Prompt
    #[test]
    fn test_prompt_debug() {
        let prompt = Prompt {
            name: "my-prompt".to_string(),
            description: "Description".to_string(),
            arguments: vec![],
        };

        let debug_str = format!("{:?}", prompt);
        assert!(debug_str.contains("my-prompt"));
    }

    /// Test Debug implementation for PromptArgument
    #[test]
    fn test_prompt_argument_debug() {
        let arg = PromptArgument {
            name: "arg-name".to_string(),
            description: Some("desc".to_string()),
            required: true,
        };

        let debug_str = format!("{:?}", arg);
        assert!(debug_str.contains("arg-name"));
    }

    /// Test deserialization from malformed JSON
    #[test]
    fn test_mcp_request_invalid_json() {
        let invalid = "not valid json";
        let result: Result<McpRequest, _> = serde_json::from_str(invalid);
        assert!(result.is_err());
    }

    /// Test deserialization with missing required fields
    #[test]
    fn test_mcp_request_missing_fields() {
        let incomplete = r#"{"jsonrpc": "2.0"}"#;
        let result: Result<McpRequest, _> = serde_json::from_str(incomplete);
        assert!(result.is_err());
    }

    /// Test MCP standard error codes
    #[test]
    fn test_mcp_standard_error_codes() {
        let error_codes = [
            (-32700, "Parse error"),
            (-32600, "Invalid Request"),
            (-32601, "Method not found"),
            (-32602, "Invalid params"),
            (-32603, "Internal error"),
        ];

        for (code, message) in error_codes {
            let response = McpResponse::error(json!(1), code, message.to_string());
            let error = response.error.unwrap();
            assert_eq!(error.code, code);
            assert_eq!(error.message, message);
        }
    }

    /// Test serialization with skip_serializing_if
    #[test]
    fn test_skip_serializing_if_behavior() {
        // McpResponse with None result should skip result field
        let error_response = McpResponse::error(json!(1), -32600, "Error".to_string());
        let json = serde_json::to_string(&error_response).unwrap();

        // Result should not appear in JSON when None (using skip_serializing_if)
        assert!(!json.contains("\"result\":null") || !json.contains("\"result\""));

        // Similarly, error should not appear in success response
        let success_response = McpResponse::success(json!(1), json!({}));
        let json = serde_json::to_string(&success_response).unwrap();
        assert!(!json.contains("\"error\":null") || !json.contains("\"error\""));
    }

    /// Test all struct types can be serialized and deserialized
    #[test]
    fn test_all_types_roundtrip() {
        // McpRequest
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "test".to_string(),
            params: Some(json!({})),
        };
        let json = serde_json::to_string(&request).unwrap();
        let _: McpRequest = serde_json::from_str(&json).unwrap();

        // ToolCallParams
        let params = ToolCallParams {
            name: "tool".to_string(),
            arguments: json!({}),
        };
        let json = serde_json::to_string(&params).unwrap();
        let _: ToolCallParams = serde_json::from_str(&json).unwrap();

        // GenerateTemplateArgs
        let args = GenerateTemplateArgs {
            resource_uri: "uri".to_string(),
            parameters: serde_json::Map::new(),
        };
        let json = serde_json::to_string(&args).unwrap();
        let _: GenerateTemplateArgs = serde_json::from_str(&json).unwrap();

        // ListTemplatesArgs
        let args = ListTemplatesArgs {
            toolchain: None,
            category: None,
        };
        let json = serde_json::to_string(&args).unwrap();
        let _: ListTemplatesArgs = serde_json::from_str(&json).unwrap();

        // ResourceReadParams
        let params = ResourceReadParams {
            uri: "uri".to_string(),
        };
        let json = serde_json::to_string(&params).unwrap();
        let _: ResourceReadParams = serde_json::from_str(&json).unwrap();

        // ValidateTemplateArgs
        let args = ValidateTemplateArgs {
            resource_uri: "uri".to_string(),
            parameters: serde_json::Map::new(),
        };
        let json = serde_json::to_string(&args).unwrap();
        let _: ValidateTemplateArgs = serde_json::from_str(&json).unwrap();

        // ScaffoldProjectArgs
        let args = ScaffoldProjectArgs {
            toolchain: "rust".to_string(),
            templates: vec![],
            parameters: serde_json::Map::new(),
        };
        let json = serde_json::to_string(&args).unwrap();
        let _: ScaffoldProjectArgs = serde_json::from_str(&json).unwrap();

        // SearchTemplatesArgs
        let args = SearchTemplatesArgs {
            query: "query".to_string(),
            toolchain: None,
        };
        let json = serde_json::to_string(&args).unwrap();
        let _: SearchTemplatesArgs = serde_json::from_str(&json).unwrap();

        // PromptGetParams
        let params = PromptGetParams {
            name: "name".to_string(),
        };
        let json = serde_json::to_string(&params).unwrap();
        let _: PromptGetParams = serde_json::from_str(&json).unwrap();

        // Prompt
        let prompt = Prompt {
            name: "name".to_string(),
            description: "desc".to_string(),
            arguments: vec![],
        };
        let json = serde_json::to_string(&prompt).unwrap();
        let _: Prompt = serde_json::from_str(&json).unwrap();

        // PromptArgument
        let arg = PromptArgument {
            name: "name".to_string(),
            description: None,
            required: false,
        };
        let json = serde_json::to_string(&arg).unwrap();
        let _: PromptArgument = serde_json::from_str(&json).unwrap();
    }
