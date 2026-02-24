    // === Serialization Tests ===

    #[test]
    fn test_mcp_request_serialization() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(123),
            method: "test".to_string(),
            params: Some(json!({"key": "value"})),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"test\""));

        let deserialized: McpRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.jsonrpc, "2.0");
        assert_eq!(deserialized.method, "test");
    }

    #[test]
    fn test_mcp_request_serialization_no_params() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "test".to_string(),
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        // params should be serialized as null when None
        let deserialized: McpRequest = serde_json::from_str(&json).unwrap();
        assert!(deserialized.params.is_none());
    }

    #[test]
    fn test_mcp_response_serialization_success() {
        let response = McpResponse::success(json!(42), json!({"result": "success"}));

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));

        let deserialized: McpResponse = serde_json::from_str(&json).unwrap();
        assert!(deserialized.result.is_some());
        assert!(deserialized.error.is_none());
    }

    #[test]
    fn test_mcp_response_serialization_error() {
        let response = McpResponse::error(json!(42), -32600, "Invalid request".to_string());

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"error\""));
        assert!(!json.contains("\"result\":"));

        let deserialized: McpResponse = serde_json::from_str(&json).unwrap();
        assert!(deserialized.error.is_some());
        assert!(deserialized.result.is_none());
    }

    #[test]
    fn test_mcp_error_serialization() {
        let error = McpError {
            code: -32700,
            message: "Parse error".to_string(),
            data: Some(json!({"position": 42})),
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("-32700"));
        assert!(json.contains("Parse error"));

        let deserialized: McpError = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code, error.code);
    }

    #[test]
    fn test_mcp_error_serialization_no_data() {
        let error = McpError {
            code: -32600,
            message: "Invalid request".to_string(),
            data: None,
        };

        let json = serde_json::to_string(&error).unwrap();
        // data should be skipped when None
        assert!(!json.contains("\"data\":null"));
    }

    #[test]
    fn test_tool_call_params_serialization() {
        let params = ToolCallParams {
            name: "analyze".to_string(),
            arguments: json!({"path": "/test"}),
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: ToolCallParams = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, params.name);
        assert_eq!(deserialized.arguments, params.arguments);
    }

    #[test]
    fn test_prompt_serialization() {
        let prompt = Prompt {
            name: "test-prompt".to_string(),
            description: "A test prompt".to_string(),
            arguments: vec![PromptArgument {
                name: "arg1".to_string(),
                description: Some("First arg".to_string()),
                required: true,
            }],
        };

        let json = serde_json::to_string(&prompt).unwrap();
        let deserialized: Prompt = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, prompt.name);
        assert_eq!(deserialized.arguments.len(), 1);
    }

    // === Edge Case Tests ===

    #[test]
    fn test_mcp_request_empty_method() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "".to_string(),
            params: None,
        };

        assert!(request.method.is_empty());
    }

    #[test]
    fn test_mcp_response_null_result() {
        let response = McpResponse::success(json!(1), Value::Null);

        assert!(response.result.is_some());
        assert!(response.result.unwrap().is_null());
    }

    #[test]
    fn test_mcp_error_positive_code() {
        // Non-standard but valid: positive error code
        let error = McpError {
            code: 100,
            message: "Custom error".to_string(),
            data: None,
        };

        assert_eq!(error.code, 100);
    }

    #[test]
    fn test_complex_nested_params() {
        let params = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "value": 42
                    }
                }
            },
            "array": [1, 2, 3, {"nested": true}]
        });

        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "complex".to_string(),
            params: Some(params),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: McpRequest = serde_json::from_str(&json).unwrap();

        let nested_value = &deserialized.params.unwrap()["level1"]["level2"]["level3"]["value"];
        assert_eq!(nested_value, &json!(42));
    }

    #[test]
    fn test_special_characters_in_strings() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("id-with-unicode-\u{1F600}"),
            method: "method/with/slashes".to_string(),
            params: Some(json!({
                "path": "/path/with spaces/and\ttabs",
                "message": "Line1\nLine2"
            })),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: McpRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.method, "method/with/slashes");
    }
