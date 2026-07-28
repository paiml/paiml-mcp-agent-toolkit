// MCP unit tests and property tests
// Included from mcp.rs - NO use imports or #! inner attributes allowed

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

