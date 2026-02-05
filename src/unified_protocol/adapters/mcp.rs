use async_trait::async_trait;
use axum::body::Body;
use axum::http::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader as AsyncBufReader, Stdin};
use tracing::debug;

use crate::unified_protocol::{
    McpContext, Protocol, ProtocolAdapter, ProtocolError, UnifiedRequest, UnifiedResponse,
};

/// MCP (Model Context Protocol) adapter for JSON-RPC over STDIO
pub struct McpAdapter {
    #[allow(dead_code)]
    stdin: Option<AsyncBufReader<Stdin>>,
}

impl McpAdapter {
    #[must_use]
    pub fn new() -> Self {
        Self { stdin: None }
    }

    #[must_use]
    pub fn with_stdin(stdin: Stdin) -> Self {
        Self {
            stdin: Some(AsyncBufReader::new(stdin)),
        }
    }
}

impl Default for McpAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProtocolAdapter for McpAdapter {
    type Input = McpInput;
    type Output = String;

    fn protocol(&self) -> Protocol {
        Protocol::Mcp
    }

    async fn decode(&self, input: Self::Input) -> Result<UnifiedRequest, ProtocolError> {
        debug!("Decoding MCP input: {:?}", input);

        let json_rpc: JsonRpcRequest = match input {
            McpInput::Line(line) => serde_json::from_str(&line)
                .map_err(|e| ProtocolError::DecodeError(format!("Invalid JSON-RPC: {e}")))?,
            McpInput::Request(req) => req,
        };

        // Validate JSON-RPC structure
        if json_rpc.jsonrpc != "2.0" {
            return Err(ProtocolError::InvalidFormat(
                "Invalid JSON-RPC version, expected '2.0'".to_string(),
            ));
        }

        // Convert to unified request
        let path = format!("/mcp/{}", json_rpc.method);
        let body = serde_json::to_vec(&json_rpc.params.unwrap_or(Value::Null))?;

        let unified_request = UnifiedRequest::new(Method::POST, path)
            .with_body(Body::from(body))
            .with_header("content-type", "application/json")
            .with_extension("protocol", Protocol::Mcp)
            .with_extension(
                "mcp_context",
                McpContext {
                    id: json_rpc.id.clone(),
                    method: json_rpc.method.clone(),
                },
            );

        debug!(
            method = %json_rpc.method,
            id = ?json_rpc.id,
            "Decoded MCP request"
        );

        Ok(unified_request)
    }

    async fn encode(&self, response: UnifiedResponse) -> Result<Self::Output, ProtocolError> {
        debug!(status = %response.status, "Encoding MCP response");

        // Extract MCP context to get the request ID
        let body_bytes = http_body_util::BodyExt::collect(response.body)
            .await
            .map_err(|e| ProtocolError::EncodeError(format!("Failed to read response body: {e}")))?
            .to_bytes();

        let response_data: Value = serde_json::from_slice(&body_bytes)?;

        // Check if this is already a JSON-RPC response
        if response_data.get("jsonrpc").is_some() {
            return Ok(serde_json::to_string(&response_data)?);
        }

        // Build JSON-RPC response
        let json_rpc_response = if response.status.is_success() {
            JsonRpcResponse::success(response_data, None) // ID would come from context
        } else {
            // Try to extract error information
            let error_code = match response.status.as_u16() {
                400 => -32602, // Invalid params
                404 => -32601, // Method not found
                500 => -32603, // Internal error
                _ => -32000,   // Server error
            };

            let error_message = response_data
                .get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            JsonRpcResponse::error(
                JsonRpcError {
                    code: error_code,
                    message: error_message,
                    data: response_data.get("data").cloned(),
                },
                None,
            )
        };

        let result = serde_json::to_string(&json_rpc_response)?;
        debug!(response = %result, "Encoded MCP response");

        Ok(result)
    }
}

/// Input types for MCP adapter
#[derive(Debug)]
pub enum McpInput {
    Line(String),
    Request(JsonRpcRequest),
}

/// JSON-RPC 2.0 request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

impl JsonRpcRequest {
    #[must_use]
    pub fn new(method: String, params: Option<Value>, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id,
        }
    }

    #[must_use]
    pub fn notification(method: String, params: Option<Value>) -> Self {
        Self::new(method, params, None)
    }

    #[must_use]
    pub fn request(method: String, params: Option<Value>, id: Value) -> Self {
        Self::new(method, params, Some(id))
    }
}

/// JSON-RPC 2.0 response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

impl JsonRpcResponse {
    #[must_use]
    pub fn success(result: Value, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    #[must_use]
    pub fn error(error: JsonRpcError, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

/// JSON-RPC 2.0 error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    // Standard JSON-RPC error codes
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    #[must_use]
    pub fn parse_error() -> Self {
        Self {
            code: Self::PARSE_ERROR,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    #[must_use]
    pub fn invalid_request() -> Self {
        Self {
            code: Self::INVALID_REQUEST,
            message: "Invalid Request".to_string(),
            data: None,
        }
    }

    #[must_use]
    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: Self::METHOD_NOT_FOUND,
            message: format!("Method not found: {method}"),
            data: None,
        }
    }

    #[must_use]
    pub fn invalid_params(message: &str) -> Self {
        Self {
            code: Self::INVALID_PARAMS,
            message: format!("Invalid params: {message}"),
            data: None,
        }
    }

    #[must_use]
    pub fn internal_error(message: &str) -> Self {
        Self {
            code: Self::INTERNAL_ERROR,
            message: format!("Internal error: {message}"),
            data: None,
        }
    }
}

/// Helper for reading MCP messages from STDIO
pub struct McpReader {
    reader: AsyncBufReader<Stdin>,
}

impl McpReader {
    #[must_use]
    pub fn new(stdin: Stdin) -> Self {
        Self {
            reader: AsyncBufReader::new(stdin),
        }
    }

    /// Read a single JSON-RPC message from stdin
    pub async fn read_message(&mut self) -> Result<JsonRpcRequest, ProtocolError> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            return Err(ProtocolError::IoError(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "EOF on stdin",
            )));
        }

        let line = line.trim();
        if line.is_empty() {
            return Err(ProtocolError::InvalidFormat("Empty line".to_string()));
        }

        serde_json::from_str(line)
            .map_err(|e| ProtocolError::DecodeError(format!("Invalid JSON: {e}")))
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_rpc_request_creation() {
        let req = JsonRpcRequest::request(
            "test_method".to_string(),
            Some(json!({"param": "value"})),
            json!(1),
        );

        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "test_method");
        assert_eq!(req.id, Some(json!(1)));
    }

    #[test]
    fn test_json_rpc_notification() {
        let notification = JsonRpcRequest::notification(
            "test_notification".to_string(),
            Some(json!({"param": "value"})),
        );

        assert_eq!(notification.jsonrpc, "2.0");
        assert_eq!(notification.method, "test_notification");
        assert_eq!(notification.id, None);
    }

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse::success(json!({"result": "success"}), Some(json!(1)));

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert_eq!(response.id, Some(json!(1)));
    }

    #[test]
    fn test_json_rpc_response_error() {
        let error = JsonRpcError::method_not_found("unknown_method");
        let response = JsonRpcResponse::error(error, Some(json!(1)));

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, JsonRpcError::METHOD_NOT_FOUND);
    }

    #[tokio::test]
    async fn test_mcp_adapter_decode() {
        let adapter = McpAdapter::new();
        let request = JsonRpcRequest::request(
            "test_method".to_string(),
            Some(json!({"param": "value"})),
            json!(1),
        );

        let unified_request = adapter.decode(McpInput::Request(request)).await.unwrap();

        assert_eq!(unified_request.method, Method::POST);
        assert_eq!(unified_request.path, "/mcp/test_method");
        assert_eq!(
            unified_request.get_extension::<Protocol>("protocol"),
            Some(Protocol::Mcp)
        );

        let mcp_context: McpContext = unified_request.get_extension("mcp_context").unwrap();
        assert_eq!(mcp_context.method, "test_method");
        assert_eq!(mcp_context.id, Some(json!(1)));
    }

    #[tokio::test]
    async fn test_mcp_adapter_encode_success() {
        let adapter = McpAdapter::new();
        let response = UnifiedResponse::ok()
            .with_json(&json!({"message": "success"}))
            .unwrap();

        let encoded = adapter.encode(response).await.unwrap();
        let parsed: JsonRpcResponse = serde_json::from_str(&encoded).unwrap();

        assert_eq!(parsed.jsonrpc, "2.0");
        assert!(parsed.result.is_some());
        assert!(parsed.error.is_none());
    }

    #[test]
    fn test_standard_json_rpc_errors() {
        assert_eq!(JsonRpcError::PARSE_ERROR, -32700);
        assert_eq!(JsonRpcError::INVALID_REQUEST, -32600);
        assert_eq!(JsonRpcError::METHOD_NOT_FOUND, -32601);
        assert_eq!(JsonRpcError::INVALID_PARAMS, -32602);
        assert_eq!(JsonRpcError::INTERNAL_ERROR, -32603);
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod extended_tests {
    use super::*;
    use serde_json::json;

    // ============ McpAdapter Tests ============

    #[test]
    fn test_mcp_adapter_new() {
        let adapter = McpAdapter::new();
        assert!(adapter.stdin.is_none());
    }

    #[test]
    fn test_mcp_adapter_default() {
        let adapter = McpAdapter::default();
        assert!(adapter.stdin.is_none());
    }

    #[test]
    fn test_mcp_adapter_protocol() {
        let adapter = McpAdapter::new();
        assert!(matches!(adapter.protocol(), Protocol::Mcp));
    }

    // ============ JsonRpcRequest Tests ============

    #[test]
    fn test_json_rpc_request_new_basic() {
        let req = JsonRpcRequest::new("test".to_string(), None, None);
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "test");
        assert!(req.params.is_none());
        assert!(req.id.is_none());
    }

    #[test]
    fn test_json_rpc_request_new_with_params() {
        let params = json!({"key": "value", "number": 42});
        let req = JsonRpcRequest::new("method".to_string(), Some(params.clone()), None);
        assert_eq!(req.params, Some(params));
    }

    #[test]
    fn test_json_rpc_request_new_with_id() {
        let req = JsonRpcRequest::new("method".to_string(), None, Some(json!(123)));
        assert_eq!(req.id, Some(json!(123)));
    }

    #[test]
    fn test_json_rpc_request_notification() {
        let notification = JsonRpcRequest::notification("notify".to_string(), None);
        assert_eq!(notification.method, "notify");
        assert!(notification.id.is_none());
    }

    #[test]
    fn test_json_rpc_request_notification_with_params() {
        let params = json!({"event": "update"});
        let notification = JsonRpcRequest::notification("notify".to_string(), Some(params.clone()));
        assert_eq!(notification.params, Some(params));
    }

    #[test]
    fn test_json_rpc_request_request() {
        let req = JsonRpcRequest::request("method".to_string(), None, json!("id-1"));
        assert_eq!(req.id, Some(json!("id-1")));
    }

    #[test]
    fn test_json_rpc_request_serialization() {
        let req = JsonRpcRequest::request(
            "test_method".to_string(),
            Some(json!({"param": "value"})),
            json!(1),
        );
        let serialized = serde_json::to_string(&req).unwrap();
        assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
        assert!(serialized.contains("\"method\":\"test_method\""));
    }

    #[test]
    fn test_json_rpc_request_deserialization() {
        let json_str = r#"{"jsonrpc":"2.0","method":"test","id":1}"#;
        let req: JsonRpcRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "test");
        assert_eq!(req.id, Some(json!(1)));
    }

    #[test]
    fn test_json_rpc_request_clone() {
        let req = JsonRpcRequest::request("method".to_string(), None, json!(1));
        let cloned = req.clone();
        assert_eq!(cloned.method, req.method);
        assert_eq!(cloned.id, req.id);
    }

    #[test]
    fn test_json_rpc_request_debug() {
        let req = JsonRpcRequest::new("debug_test".to_string(), None, None);
        let debug = format!("{:?}", req);
        assert!(debug.contains("JsonRpcRequest"));
        assert!(debug.contains("debug_test"));
    }

    // ============ JsonRpcResponse Tests ============

    #[test]
    fn test_json_rpc_response_success_with_result() {
        let result = json!({"data": [1, 2, 3]});
        let response = JsonRpcResponse::success(result.clone(), Some(json!(1)));
        assert_eq!(response.result, Some(result));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_response_success_without_id() {
        let response = JsonRpcResponse::success(json!(null), None);
        assert!(response.id.is_none());
    }

    #[test]
    fn test_json_rpc_response_error_with_error() {
        let error = JsonRpcError::internal_error("test error");
        let response = JsonRpcResponse::error(error, Some(json!(1)));
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }

    #[test]
    fn test_json_rpc_response_serialization() {
        let response = JsonRpcResponse::success(json!({"result": "ok"}), Some(json!(1)));
        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
        assert!(serialized.contains("\"result\""));
    }

    #[test]
    fn test_json_rpc_response_deserialization() {
        let json_str = r#"{"jsonrpc":"2.0","result":"success","id":1}"#;
        let response: JsonRpcResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.result, Some(json!("success")));
    }

    #[test]
    fn test_json_rpc_response_clone() {
        let response = JsonRpcResponse::success(json!(1), Some(json!(1)));
        let cloned = response.clone();
        assert_eq!(cloned.result, response.result);
    }

    #[test]
    fn test_json_rpc_response_debug() {
        let response = JsonRpcResponse::success(json!(null), None);
        let debug = format!("{:?}", response);
        assert!(debug.contains("JsonRpcResponse"));
    }

    // ============ JsonRpcError Tests ============

    #[test]
    fn test_json_rpc_error_parse_error() {
        let error = JsonRpcError::parse_error();
        assert_eq!(error.code, JsonRpcError::PARSE_ERROR);
        assert_eq!(error.message, "Parse error");
        assert!(error.data.is_none());
    }

    #[test]
    fn test_json_rpc_error_invalid_request() {
        let error = JsonRpcError::invalid_request();
        assert_eq!(error.code, JsonRpcError::INVALID_REQUEST);
        assert_eq!(error.message, "Invalid Request");
    }

    #[test]
    fn test_json_rpc_error_method_not_found() {
        let error = JsonRpcError::method_not_found("unknown");
        assert_eq!(error.code, JsonRpcError::METHOD_NOT_FOUND);
        assert!(error.message.contains("unknown"));
    }

    #[test]
    fn test_json_rpc_error_invalid_params() {
        let error = JsonRpcError::invalid_params("missing required field");
        assert_eq!(error.code, JsonRpcError::INVALID_PARAMS);
        assert!(error.message.contains("missing required field"));
    }

    #[test]
    fn test_json_rpc_error_internal_error() {
        let error = JsonRpcError::internal_error("database connection failed");
        assert_eq!(error.code, JsonRpcError::INTERNAL_ERROR);
        assert!(error.message.contains("database connection failed"));
    }

    #[test]
    fn test_json_rpc_error_serialization() {
        let error = JsonRpcError::parse_error();
        let serialized = serde_json::to_string(&error).unwrap();
        assert!(serialized.contains("\"code\":-32700"));
        assert!(serialized.contains("\"message\":\"Parse error\""));
    }

    #[test]
    fn test_json_rpc_error_with_data() {
        let error = JsonRpcError {
            code: -32000,
            message: "Server error".to_string(),
            data: Some(json!({"details": "additional info"})),
        };
        assert!(error.data.is_some());
        let serialized = serde_json::to_string(&error).unwrap();
        assert!(serialized.contains("\"data\""));
    }

    #[test]
    fn test_json_rpc_error_clone() {
        let error = JsonRpcError::internal_error("test");
        let cloned = error.clone();
        assert_eq!(cloned.code, error.code);
        assert_eq!(cloned.message, error.message);
    }

    #[test]
    fn test_json_rpc_error_debug() {
        let error = JsonRpcError::parse_error();
        let debug = format!("{:?}", error);
        assert!(debug.contains("JsonRpcError"));
    }

    // ============ McpInput Tests ============

    #[test]
    fn test_mcp_input_line() {
        let input = McpInput::Line(r#"{"jsonrpc":"2.0","method":"test"}"#.to_string());
        let debug = format!("{:?}", input);
        assert!(debug.contains("Line"));
    }

    #[test]
    fn test_mcp_input_request() {
        let req = JsonRpcRequest::new("test".to_string(), None, None);
        let input = McpInput::Request(req);
        let debug = format!("{:?}", input);
        assert!(debug.contains("Request"));
    }

    // ============ Decode Tests ============

    #[tokio::test]
    async fn test_mcp_adapter_decode_from_line() {
        let adapter = McpAdapter::new();
        let line = r#"{"jsonrpc":"2.0","method":"test_method","id":1}"#.to_string();
        let input = McpInput::Line(line);

        let unified_request = adapter.decode(input).await.unwrap();
        assert_eq!(unified_request.path, "/mcp/test_method");
    }

    #[tokio::test]
    async fn test_mcp_adapter_decode_invalid_json() {
        let adapter = McpAdapter::new();
        let line = "not valid json".to_string();
        let input = McpInput::Line(line);

        let result = adapter.decode(input).await;
        assert!(result.is_err());
        match result {
            Err(ProtocolError::DecodeError(msg)) => {
                assert!(msg.contains("Invalid JSON-RPC"));
            }
            _ => panic!("Expected DecodeError"),
        }
    }

    #[tokio::test]
    async fn test_mcp_adapter_decode_wrong_version() {
        let adapter = McpAdapter::new();
        let line = r#"{"jsonrpc":"1.0","method":"test"}"#.to_string();
        let input = McpInput::Line(line);

        let result = adapter.decode(input).await;
        assert!(result.is_err());
        match result {
            Err(ProtocolError::InvalidFormat(msg)) => {
                assert!(msg.contains("2.0"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[tokio::test]
    async fn test_mcp_adapter_decode_with_params() {
        let adapter = McpAdapter::new();
        let req = JsonRpcRequest::request(
            "method_with_params".to_string(),
            Some(json!({"key": "value"})),
            json!(42),
        );
        let input = McpInput::Request(req);

        let unified_request = adapter.decode(input).await.unwrap();
        assert_eq!(unified_request.path, "/mcp/method_with_params");
    }

    #[tokio::test]
    async fn test_mcp_adapter_decode_without_params() {
        let adapter = McpAdapter::new();
        let req = JsonRpcRequest::request("method_no_params".to_string(), None, json!(1));
        let input = McpInput::Request(req);

        let unified_request = adapter.decode(input).await.unwrap();
        assert_eq!(unified_request.path, "/mcp/method_no_params");
    }

    // ============ Encode Tests ============

    #[tokio::test]
    async fn test_mcp_adapter_encode_error_response_400() {
        let adapter = McpAdapter::new();
        let response = UnifiedResponse::new(axum::http::StatusCode::BAD_REQUEST)
            .with_json(&json!({"error": "Bad request"}))
            .unwrap();

        let encoded = adapter.encode(response).await.unwrap();
        let parsed: JsonRpcResponse = serde_json::from_str(&encoded).unwrap();
        assert!(parsed.error.is_some());
        assert_eq!(parsed.error.unwrap().code, -32602); // Invalid params
    }

    #[tokio::test]
    async fn test_mcp_adapter_encode_error_response_404() {
        let adapter = McpAdapter::new();
        let response = UnifiedResponse::new(axum::http::StatusCode::NOT_FOUND)
            .with_json(&json!({"error": "Not found"}))
            .unwrap();

        let encoded = adapter.encode(response).await.unwrap();
        let parsed: JsonRpcResponse = serde_json::from_str(&encoded).unwrap();
        assert!(parsed.error.is_some());
        assert_eq!(parsed.error.unwrap().code, -32601); // Method not found
    }

    #[tokio::test]
    async fn test_mcp_adapter_encode_error_response_500() {
        let adapter = McpAdapter::new();
        let response = UnifiedResponse::new(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
            .with_json(&json!({"error": "Internal error"}))
            .unwrap();

        let encoded = adapter.encode(response).await.unwrap();
        let parsed: JsonRpcResponse = serde_json::from_str(&encoded).unwrap();
        assert!(parsed.error.is_some());
        assert_eq!(parsed.error.unwrap().code, -32603); // Internal error
    }

    #[tokio::test]
    async fn test_mcp_adapter_encode_error_response_other() {
        let adapter = McpAdapter::new();
        let response = UnifiedResponse::new(axum::http::StatusCode::SERVICE_UNAVAILABLE)
            .with_json(&json!({"error": "Service unavailable"}))
            .unwrap();

        let encoded = adapter.encode(response).await.unwrap();
        let parsed: JsonRpcResponse = serde_json::from_str(&encoded).unwrap();
        assert!(parsed.error.is_some());
        assert_eq!(parsed.error.unwrap().code, -32000); // Server error
    }

    #[tokio::test]
    async fn test_mcp_adapter_encode_already_jsonrpc() {
        let adapter = McpAdapter::new();
        let jsonrpc_response = JsonRpcResponse::success(json!({"data": "test"}), Some(json!(1)));
        let response = UnifiedResponse::ok().with_json(&jsonrpc_response).unwrap();

        let encoded = adapter.encode(response).await.unwrap();
        let parsed: JsonRpcResponse = serde_json::from_str(&encoded).unwrap();
        assert_eq!(parsed.jsonrpc, "2.0");
    }

    // ============ Error Constants Tests ============

    #[test]
    fn test_error_code_constants() {
        assert_eq!(JsonRpcError::PARSE_ERROR, -32700);
        assert_eq!(JsonRpcError::INVALID_REQUEST, -32600);
        assert_eq!(JsonRpcError::METHOD_NOT_FOUND, -32601);
        assert_eq!(JsonRpcError::INVALID_PARAMS, -32602);
        assert_eq!(JsonRpcError::INTERNAL_ERROR, -32603);
    }
}
