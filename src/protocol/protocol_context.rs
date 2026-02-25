// RequestContext implementation and protocol-specific helper structures

impl RequestContext {
    #[must_use]
    pub fn new(protocol: &str) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            trace_id: Uuid::new_v4(),
            protocol: protocol.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    #[must_use]
    pub fn from_json_rpc(request: &JsonRpcRequest) -> Self {
        Self {
            request_id: request.id.to_string(),
            trace_id: Uuid::new_v4(),
            protocol: "json-rpc".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    #[must_use]
    pub fn from_http(request: &HttpRequest) -> Self {
        Self {
            request_id: request
                .headers
                .get("x-request-id")
                .cloned()
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            trace_id: Uuid::new_v4(),
            protocol: "http".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

// Helper structures for protocol-specific requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
    pub id: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Value,
}

impl From<ErrorInfo> for JsonRpcError {
    fn from(error: ErrorInfo) -> Self {
        Self {
            code: error.code,
            message: error.message,
            data: error.details,
        }
    }
}
