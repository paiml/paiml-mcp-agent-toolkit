// MCP extended tests - comprehensive coverage for all types
// Included from mcp.rs - NO use imports or #! inner attributes allowed

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
