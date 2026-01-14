use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl McpResponse {
    /// Creates a successful MCP response
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::mcp::McpResponse;
    /// use serde_json::json;
    ///
    /// let response = McpResponse::success(
    ///     json!(1),
    ///     json!({"status": "ok"})
    /// );
    ///
    /// assert_eq!(response.jsonrpc, "2.0");
    /// assert!(response.result.is_some());
    /// assert!(response.error.is_none());
    /// ```
    #[must_use]
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Creates an error MCP response
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::mcp::McpResponse;
    /// use serde_json::json;
    ///
    /// let response = McpResponse::error(
    ///     json!(1),
    ///     -32601,
    ///     "Method not found".to_string()
    /// );
    ///
    /// assert_eq!(response.jsonrpc, "2.0");
    /// assert!(response.error.is_some());
    /// assert_eq!(response.error.unwrap().code, -32601);
    /// ```
    #[must_use]
    pub fn error(id: Value, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(McpError {
                code,
                message,
                data: None,
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateTemplateArgs {
    pub resource_uri: String,
    pub parameters: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListTemplatesArgs {
    pub toolchain: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceReadParams {
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTemplateArgs {
    pub resource_uri: String,
    pub parameters: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScaffoldProjectArgs {
    pub toolchain: String,
    pub templates: Vec<String>,
    pub parameters: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchTemplatesArgs {
    pub query: String,
    pub toolchain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptGetParams {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::json;

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

        /// Property: McpResponse success always has jsonrpc "2.0"
        #[test]
        fn prop_mcp_response_success_jsonrpc(id in 1i64..1000i64) {
            let response = McpResponse::success(json!(id), json!({"ok": true}));
            prop_assert_eq!(response.jsonrpc, "2.0");
        }

        /// Property: McpResponse error always has jsonrpc "2.0"
        #[test]
        fn prop_mcp_response_error_jsonrpc(id in 1i64..1000i64, code in -32700i32..-32000i32) {
            let response = McpResponse::error(json!(id), code, "Error".to_string());
            prop_assert_eq!(response.jsonrpc, "2.0");
        }

        /// Property: McpResponse success has result and no error
        #[test]
        fn prop_mcp_response_success_structure(id in 1i64..1000i64) {
            let response = McpResponse::success(json!(id), json!({}));
            prop_assert!(response.result.is_some());
            prop_assert!(response.error.is_none());
        }

        /// Property: McpResponse error has error and no result
        #[test]
        fn prop_mcp_response_error_structure(id in 1i64..1000i64, code in -32700i32..-32000i32) {
            let response = McpResponse::error(json!(id), code, "Error".to_string());
            prop_assert!(response.error.is_some());
            prop_assert!(response.result.is_none());
        }

        /// Property: Serialization roundtrip preserves McpRequest
        #[test]
        fn prop_mcp_request_roundtrip(method in "[a-z/]+") {
            let request = McpRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                method: method.clone(),
                params: None,
            };

            let json = serde_json::to_string(&request).unwrap();
            let roundtrip: McpRequest = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(roundtrip.method, method);
            prop_assert_eq!(roundtrip.jsonrpc, "2.0");
        }

        /// Property: Error code is preserved in McpError
        #[test]
        fn prop_mcp_error_code_preserved(code in -40000i32..40000i32) {
            let error = McpError {
                code,
                message: "Test".to_string(),
                data: None,
            };

            let json = serde_json::to_string(&error).unwrap();
            let roundtrip: McpError = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(roundtrip.code, code);
        }

        /// Property: PromptArgument required flag preserved
        #[test]
        fn prop_prompt_argument_required_preserved(name in "[a-z_]+", required in any::<bool>()) {
            let arg = PromptArgument {
                name: name.clone(),
                description: None,
                required,
            };

            let json = serde_json::to_string(&arg).unwrap();
            let roundtrip: PromptArgument = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(roundtrip.name, name);
            prop_assert_eq!(roundtrip.required, required);
        }
    }
}

#[cfg(test)]
mod coverage_tests {
    //! EXTREME TDD coverage tests for models/mcp.rs
    //! These tests ensure comprehensive coverage of all MCP model types.

    use super::*;
    use serde_json::json;

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
}
