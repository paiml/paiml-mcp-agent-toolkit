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
    pub fn new() -> Self {
        Self { stdin: None }
    }

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
    pub fn new(method: String, params: Option<Value>, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id,
        }
    }

    pub fn notification(method: String, params: Option<Value>) -> Self {
        Self::new(method, params, None)
    }

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
    pub fn success(result: Value, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

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

    pub fn parse_error() -> Self {
        Self {
            code: Self::PARSE_ERROR,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    pub fn invalid_request() -> Self {
        Self {
            code: Self::INVALID_REQUEST,
            message: "Invalid Request".to_string(),
            data: None,
        }
    }

    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: Self::METHOD_NOT_FOUND,
            message: format!("Method not found: {method}"),
            data: None,
        }
    }

    pub fn invalid_params(message: &str) -> Self {
        Self {
            code: Self::INVALID_PARAMS,
            message: format!("Invalid params: {message}"),
            data: None,
        }
    }

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
