#[derive(Debug, Clone, Serialize, Deserialize)]
/// Request for mcp operation.
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Response from mcp operation.
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Error type for mcp operations.
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
