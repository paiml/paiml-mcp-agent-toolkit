#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_request_creation() {
        let req = UnifiedRequest::new(Method::GET, "/test".to_string())
            .with_header("content-type", "application/json")
            .with_extension("test_key", "test_value");

        assert_eq!(req.method, Method::GET);
        assert_eq!(req.path, "/test");
        assert!(req.headers.contains_key("content-type"));
        assert_eq!(
            req.get_extension::<String>("test_key"),
            Some("test_value".to_string())
        );
    }

    #[test]
    fn test_unified_response_creation() {
        let test_data = serde_json::json!({"message": "test"});
        let response = UnifiedResponse::ok()
            .with_json(&test_data)
            .expect("internal error");

        assert_eq!(response.status, StatusCode::OK);
        assert!(response.headers.contains_key("content-type"));
    }

    #[test]
    fn test_protocol_display() {
        assert_eq!(Protocol::Mcp.to_string(), "mcp");
        assert_eq!(Protocol::Http.to_string(), "http");
        assert_eq!(Protocol::Cli.to_string(), "cli");
        assert_eq!(Protocol::WebSocket.to_string(), "websocket");
    }

    #[test]
    fn test_unified_request_with_body() {
        let body = Body::from("test body");
        let req = UnifiedRequest::new(Method::POST, "/api".to_string()).with_body(body);
        assert_eq!(req.method, Method::POST);
        assert_eq!(req.path, "/api");
    }

    #[test]
    fn test_unified_request_invalid_header() {
        // Invalid header should not crash, just skip
        let req = UnifiedRequest::new(Method::GET, "/test".to_string()).with_header("", ""); // Invalid
        assert!(!req.headers.contains_key(""));
    }

    #[test]
    fn test_unified_request_get_missing_extension() {
        let req = UnifiedRequest::new(Method::GET, "/test".to_string());
        assert_eq!(req.get_extension::<String>("nonexistent"), None);
    }

    #[test]
    fn test_unified_response_new() {
        let response = UnifiedResponse::new(StatusCode::NOT_FOUND);
        assert_eq!(response.status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_unified_response_with_body() {
        let response = UnifiedResponse::ok().with_body(Body::from("raw body"));
        assert_eq!(response.status, StatusCode::OK);
    }

    #[test]
    fn test_unified_response_invalid_header() {
        // Invalid header should not crash
        let response = UnifiedResponse::ok().with_header("", "");
        assert!(!response.headers.contains_key(""));
    }

    #[test]
    fn test_unified_response_into_response() {
        let response = UnifiedResponse::ok()
            .with_header("x-custom", "value")
            .with_body(Body::from("test"));
        let axum_response = response.into_response();
        assert_eq!(axum_response.status(), StatusCode::OK);
    }

    #[test]
    fn test_protocol_clone_copy() {
        let p1 = Protocol::Http;
        let p2 = p1;
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_protocol_serialization() {
        let p = Protocol::Mcp;
        let json = serde_json::to_string(&p).unwrap();
        let deserialized: Protocol = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Protocol::Mcp);
    }

    #[test]
    fn test_protocol_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Protocol::Http);
        set.insert(Protocol::Mcp);
        assert!(set.contains(&Protocol::Http));
        assert!(set.contains(&Protocol::Mcp));
    }

    #[test]
    fn test_protocol_error_decode() {
        let err = ProtocolError::DecodeError("test error".to_string());
        assert!(err.to_string().contains("decode"));
    }

    #[test]
    fn test_protocol_error_encode() {
        let err = ProtocolError::EncodeError("test error".to_string());
        assert!(err.to_string().contains("encode"));
    }

    #[test]
    fn test_protocol_error_unsupported() {
        let err = ProtocolError::UnsupportedProtocol("test".to_string());
        assert!(err.to_string().contains("not supported"));
    }

    #[test]
    fn test_protocol_error_invalid_format() {
        let err = ProtocolError::InvalidFormat("bad format".to_string());
        assert!(err.to_string().contains("Invalid"));
    }

    #[test]
    fn test_protocol_error_http() {
        let err = ProtocolError::HttpError("http failed".to_string());
        assert!(err.to_string().contains("HTTP"));
    }

    #[test]
    fn test_protocol_error_json() {
        let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let err: ProtocolError = json_err.into();
        assert!(err.to_string().contains("JSON"));
    }

    #[test]
    fn test_mcp_context() {
        let ctx = McpContext {
            id: Some(serde_json::json!(123)),
            method: "tools/call".to_string(),
        };
        assert_eq!(ctx.method, "tools/call");
        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: McpContext = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.method, "tools/call");
    }

    #[test]
    fn test_cli_context() {
        let ctx = CliContext {
            command: "analyze".to_string(),
            args: vec!["--path".to_string(), ".".to_string()],
        };
        assert_eq!(ctx.command, "analyze");
        assert_eq!(ctx.args.len(), 2);
    }

    #[test]
    fn test_http_context() {
        let ctx = HttpContext {
            remote_addr: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent/1.0".to_string()),
        };
        assert!(ctx.remote_addr.is_some());
        assert!(ctx.user_agent.is_some());
    }

    #[test]
    fn test_adapter_registry_new() {
        let registry = AdapterRegistry::new();
        assert!(registry.get(Protocol::Http).is_none());
    }

    #[test]
    fn test_adapter_registry_default() {
        let registry = AdapterRegistry::default();
        assert!(registry.get(Protocol::Mcp).is_none());
    }
}
