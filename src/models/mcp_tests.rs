    #[test]
    fn test_mcp_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }

    // === McpRequest Tests ===

    #[test]
    fn test_mcp_request_creation() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "tools/list");
        assert!(request.params.is_none());
    }

    #[test]
    fn test_mcp_request_with_params() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("request-123"),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "analyze",
                "arguments": {"path": "/test"}
            })),
        };

        assert!(request.params.is_some());
        let params = request.params.unwrap();
        assert_eq!(params["name"], "analyze");
    }

    #[test]
    fn test_mcp_request_with_numeric_id() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(42),
            method: "test".to_string(),
            params: None,
        };

        assert_eq!(request.id, json!(42));
    }

    #[test]
    fn test_mcp_request_with_string_id() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("uuid-12345"),
            method: "test".to_string(),
            params: None,
        };

        assert_eq!(request.id, json!("uuid-12345"));
    }

    #[test]
    fn test_mcp_request_with_null_id() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::Null,
            method: "notification".to_string(),
            params: None,
        };

        assert!(request.id.is_null());
    }

    #[test]
    fn test_mcp_request_clone() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "test".to_string(),
            params: Some(json!({"key": "value"})),
        };

        let cloned = request.clone();
        assert_eq!(cloned.jsonrpc, request.jsonrpc);
        assert_eq!(cloned.method, request.method);
        assert_eq!(cloned.params, request.params);
    }

    // === McpResponse Tests ===

    #[test]
    fn test_mcp_response_success() {
        let response = McpResponse::success(json!(1), json!({"status": "ok"}));

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, json!(1));
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        assert_eq!(result["status"], "ok");
    }

    #[test]
    fn test_mcp_response_error() {
        let response = McpResponse::error(json!(2), -32601, "Method not found".to_string());

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, json!(2));
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Method not found");
        assert!(error.data.is_none());
    }

    #[test]
    fn test_mcp_response_success_with_complex_result() {
        let complex_result = json!({
            "tools": [
                {"name": "tool1", "description": "First tool"},
                {"name": "tool2", "description": "Second tool"}
            ],
            "count": 2
        });

        let response = McpResponse::success(json!(99), complex_result);

        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result["tools"].is_array());
        assert_eq!(result["count"], 2);
    }

    #[test]
    fn test_mcp_response_error_with_string_id() {
        let response = McpResponse::error(json!("request-abc"), -32700, "Parse error".to_string());

        assert_eq!(response.id, json!("request-abc"));
        assert_eq!(response.error.unwrap().code, -32700);
    }

    #[test]
    fn test_mcp_response_clone() {
        let response = McpResponse::success(json!(1), json!({"data": "test"}));
        let cloned = response.clone();

        assert_eq!(cloned.jsonrpc, response.jsonrpc);
        assert_eq!(cloned.id, response.id);
        assert_eq!(cloned.result, response.result);
    }

    // === McpError Tests ===

    #[test]
    fn test_mcp_error_creation() {
        let error = McpError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        };

        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid Request");
        assert!(error.data.is_none());
    }

    #[test]
    fn test_mcp_error_with_data() {
        let error = McpError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(json!({
                "param": "path",
                "expected": "string",
                "got": "number"
            })),
        };

        assert!(error.data.is_some());
        let data = error.data.unwrap();
        assert_eq!(data["param"], "path");
    }

    #[test]
    fn test_mcp_error_clone() {
        let error = McpError {
            code: -32000,
            message: "Server error".to_string(),
            data: Some(json!({"details": "Something went wrong"})),
        };

        let cloned = error.clone();
        assert_eq!(cloned.code, error.code);
        assert_eq!(cloned.message, error.message);
        assert_eq!(cloned.data, error.data);
    }

    // === ToolCallParams Tests ===

    #[test]
    fn test_tool_call_params_creation() {
        let params = ToolCallParams {
            name: "analyze".to_string(),
            arguments: json!({"path": "/src", "format": "json"}),
        };

        assert_eq!(params.name, "analyze");
        assert_eq!(params.arguments["path"], "/src");
        assert_eq!(params.arguments["format"], "json");
    }

    #[test]
    fn test_tool_call_params_empty_arguments() {
        let params = ToolCallParams {
            name: "list".to_string(),
            arguments: json!({}),
        };

        assert!(params.arguments.as_object().unwrap().is_empty());
    }

    // === GenerateTemplateArgs Tests ===

    #[test]
    fn test_generate_template_args_creation() {
        let mut parameters = serde_json::Map::new();
        parameters.insert("project_name".to_string(), json!("my-project"));
        parameters.insert("version".to_string(), json!("1.0.0"));

        let args = GenerateTemplateArgs {
            resource_uri: "template://makefile/rust".to_string(),
            parameters,
        };

        assert_eq!(args.resource_uri, "template://makefile/rust");
        assert_eq!(args.parameters.len(), 2);
        assert_eq!(args.parameters["project_name"], "my-project");
    }

    #[test]
    fn test_generate_template_args_empty_parameters() {
        let args = GenerateTemplateArgs {
            resource_uri: "template://readme".to_string(),
            parameters: serde_json::Map::new(),
        };

        assert!(args.parameters.is_empty());
    }

    // === ListTemplatesArgs Tests ===

    #[test]
    fn test_list_templates_args_all_none() {
        let args = ListTemplatesArgs {
            toolchain: None,
            category: None,
        };

        assert!(args.toolchain.is_none());
        assert!(args.category.is_none());
    }

    #[test]
    fn test_list_templates_args_with_filters() {
        let args = ListTemplatesArgs {
            toolchain: Some("rust".to_string()),
            category: Some("makefile".to_string()),
        };

        assert_eq!(args.toolchain, Some("rust".to_string()));
        assert_eq!(args.category, Some("makefile".to_string()));
    }

    // === ResourceReadParams Tests ===

    #[test]
    fn test_resource_read_params_creation() {
        let params = ResourceReadParams {
            uri: "template://context/python".to_string(),
        };

        assert_eq!(params.uri, "template://context/python");
    }

    // === ValidateTemplateArgs Tests ===

    #[test]
    fn test_validate_template_args_creation() {
        let mut parameters = serde_json::Map::new();
        parameters.insert("name".to_string(), json!("test-project"));

        let args = ValidateTemplateArgs {
            resource_uri: "template://gitignore/node".to_string(),
            parameters,
        };

        assert_eq!(args.resource_uri, "template://gitignore/node");
        assert!(args.parameters.contains_key("name"));
    }

    // === ScaffoldProjectArgs Tests ===

    #[test]
    fn test_scaffold_project_args_creation() {
        let mut parameters = serde_json::Map::new();
        parameters.insert("project_name".to_string(), json!("new-app"));
        parameters.insert("author".to_string(), json!("developer"));

        let args = ScaffoldProjectArgs {
            toolchain: "rust".to_string(),
            templates: vec![
                "makefile".to_string(),
                "readme".to_string(),
                "gitignore".to_string(),
            ],
            parameters,
        };

        assert_eq!(args.toolchain, "rust");
        assert_eq!(args.templates.len(), 3);
        assert!(args.templates.contains(&"makefile".to_string()));
    }

    #[test]
    fn test_scaffold_project_args_single_template() {
        let args = ScaffoldProjectArgs {
            toolchain: "deno".to_string(),
            templates: vec!["context".to_string()],
            parameters: serde_json::Map::new(),
        };

        assert_eq!(args.templates.len(), 1);
    }

    // === SearchTemplatesArgs Tests ===

    #[test]
    fn test_search_templates_args_query_only() {
        let args = SearchTemplatesArgs {
            query: "makefile".to_string(),
            toolchain: None,
        };

        assert_eq!(args.query, "makefile");
        assert!(args.toolchain.is_none());
    }

    #[test]
    fn test_search_templates_args_with_toolchain() {
        let args = SearchTemplatesArgs {
            query: "readme".to_string(),
            toolchain: Some("python".to_string()),
        };

        assert_eq!(args.query, "readme");
        assert_eq!(args.toolchain, Some("python".to_string()));
    }

    // === PromptGetParams Tests ===

    #[test]
    fn test_prompt_get_params_creation() {
        let params = PromptGetParams {
            name: "analyze-code".to_string(),
        };

        assert_eq!(params.name, "analyze-code");
    }

    // === Prompt Tests ===

    #[test]
    fn test_prompt_creation() {
        let prompt = Prompt {
            name: "code-review".to_string(),
            description: "Review code for quality and best practices".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "file_path".to_string(),
                    description: Some("Path to the file to review".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "focus".to_string(),
                    description: Some("Area to focus on".to_string()),
                    required: false,
                },
            ],
        };

        assert_eq!(prompt.name, "code-review");
        assert_eq!(prompt.arguments.len(), 2);
        assert!(prompt.arguments[0].required);
        assert!(!prompt.arguments[1].required);
    }

    #[test]
    fn test_prompt_no_arguments() {
        let prompt = Prompt {
            name: "simple-prompt".to_string(),
            description: "A simple prompt with no arguments".to_string(),
            arguments: vec![],
        };

        assert!(prompt.arguments.is_empty());
    }

    // === PromptArgument Tests ===

    #[test]
    fn test_prompt_argument_required() {
        let arg = PromptArgument {
            name: "path".to_string(),
            description: Some("The file path".to_string()),
            required: true,
        };

        assert!(arg.required);
        assert!(arg.description.is_some());
    }

    #[test]
    fn test_prompt_argument_optional_no_description() {
        let arg = PromptArgument {
            name: "verbose".to_string(),
            description: None,
            required: false,
        };

        assert!(!arg.required);
        assert!(arg.description.is_none());
    }
