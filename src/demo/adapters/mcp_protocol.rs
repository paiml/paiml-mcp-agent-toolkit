// MCP demo adapter protocol trait implementation and conversions
// Included from mcp.rs - shares parent module scope

impl Default for McpDemoAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl McpDemoError {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn to_mcp_error(&self) -> McpError {
        match self {
            McpDemoError::InvalidJsonRpc(_) => McpError {
                code: -32600,
                message: "Invalid Request".to_string(),
                data: Some(serde_json::json!({"error": self.to_string()})),
            },
            McpDemoError::UnknownMethod(_) => McpError {
                code: -32601,
                message: "Method not found".to_string(),
                data: Some(serde_json::json!({"error": self.to_string()})),
            },
            McpDemoError::InvalidParams(_) => McpError {
                code: -32602,
                message: "Invalid params".to_string(),
                data: Some(serde_json::json!({"error": self.to_string()})),
            },
            McpDemoError::RequestNotFound(_) => McpError {
                code: -32001,
                message: "Request not found".to_string(),
                data: Some(serde_json::json!({"error": self.to_string()})),
            },
            _ => McpError {
                code: -32603,
                message: "Internal error".to_string(),
                data: Some(serde_json::json!({"error": self.to_string()})),
            },
        }
    }
}

#[async_trait]
impl DemoProtocol for McpDemoAdapter {
    type Request = McpRequest;
    type Response = McpResponse;
    type Error = McpDemoError;

    async fn decode_request(&self, raw: &[u8]) -> Result<Self::Request, Self::Error> {
        debug_assert!(!raw.is_empty(), "raw must not be empty");
        let request: McpRequest = serde_json::from_slice(raw)?;

        // Validate JSON-RPC format
        if request.jsonrpc != "2.0" {
            return Err(McpDemoError::InvalidJsonRpc(format!(
                "Expected jsonrpc: '2.0', got: '{}'",
                request.jsonrpc
            )));
        }

        Ok(request)
    }

    async fn encode_response(&self, resp: Self::Response) -> Result<Vec<u8>, Self::Error> {
        debug_assert!(true, "contract: encode_response");
        let json = serde_json::to_vec(&resp)?;
        Ok(json)
    }

    async fn get_protocol_metadata(&self) -> ProtocolMetadata {
        debug_assert!(true, "contract: get_protocol_metadata");
        ProtocolMetadata {
            name: "mcp",
            version: "2.0",
            description: "Model Context Protocol (JSON-RPC 2.0) for AI integration".to_string(),
            request_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "jsonrpc": {
                        "type": "string",
                        "const": "2.0"
                    },
                    "method": {
                        "type": "string",
                        "enum": ["demo.analyze", "demo.getResults", "demo.getApiTrace"]
                    },
                    "params": {
                        "type": "object",
                        "description": "Method-specific parameters"
                    },
                    "id": {
                        "description": "Request identifier (any JSON type)"
                    }
                },
                "required": ["jsonrpc", "method"]
            }),
            response_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "jsonrpc": {
                        "type": "string",
                        "const": "2.0"
                    },
                    "result": {
                        "description": "Method result (on success)"
                    },
                    "error": {
                        "type": "object",
                        "properties": {
                            "code": {"type": "integer"},
                            "message": {"type": "string"},
                            "data": {}
                        },
                        "required": ["code", "message"]
                    },
                    "id": {
                        "description": "Request identifier (matches request)"
                    }
                },
                "required": ["jsonrpc"]
            }),
            example_requests: vec![
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "demo.analyze",
                    "params": {"path": "/repo", "cache": true, "include_trace": true},
                    "id": 1
                }),
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "demo.getResults",
                    "params": {"request_id": "uuid", "include_metadata": true},
                    "id": 2
                }),
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "demo.getApiTrace",
                    "params": {"limit": 5},
                    "id": 3
                }),
            ],
            capabilities: vec![
                "json_rpc_2.0".to_string(),
                "method_routing".to_string(),
                "parameter_validation".to_string(),
                "error_handling".to_string(),
                "request_tracing".to_string(),
                "async_results".to_string(),
            ],
        }
    }

    async fn execute_demo(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        debug_assert!(true, "contract: execute_demo");
        match request.method.as_str() {
            "demo.analyze" => self.handle_demo_analyze(request.params, request.id).await,
            "demo.getResults" => {
                self.handle_demo_get_results(request.params, request.id)
                    .await
            }
            "demo.getApiTrace" => {
                self.handle_demo_get_api_trace(request.params, request.id)
                    .await
            }
            _ => {
                let error = McpDemoError::UnknownMethod(request.method.clone());
                Ok(McpResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(error.to_mcp_error()),
                    id: request.id,
                })
            }
        }
    }
}

impl From<Value> for McpRequest {
    fn from(value: Value) -> Self {
        serde_json::from_value(value).unwrap_or_else(|_| McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "invalid".to_string(),
            params: None,
            id: None,
        })
    }
}

impl From<McpResponse> for Value {
    fn from(val: McpResponse) -> Self {
        serde_json::to_value(val).unwrap_or(Value::Null)
    }
}
