// MCP demo adapter tests
// Included from mcp.rs - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_adapter_metadata() {
        let adapter = McpDemoAdapter::new();
        let metadata = adapter.get_protocol_metadata().await;

        assert_eq!(metadata.name, "mcp");
        assert_eq!(metadata.version, "2.0");
        assert!(!metadata.capabilities.is_empty());
        assert!(!metadata.example_requests.is_empty());
    }

    #[test]
    fn test_mcp_request_from_value() {
        let value = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "demo.analyze",
            "params": {"path": "/test"},
            "id": 1
        });

        let request = McpRequest::from(value);
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "demo.analyze");
        assert_eq!(request.id, Some(serde_json::json!(1)));
    }

    #[tokio::test]
    async fn test_demo_analyze() {
        use std::fs;
        use tempfile::TempDir;

        // Create a small test directory instead of analyzing the entire project
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a simple test file
        fs::write(
            temp_path.join("test.rs"),
            "fn main() { println!(\"Hello\"); }",
        )
        .unwrap();

        let adapter = McpDemoAdapter::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "demo.analyze".to_string(),
            params: Some(serde_json::json!({
                "path": temp_path.to_string_lossy(),
                "cache": false,
                "include_trace": true
            })),
            id: Some(serde_json::json!(1)),
        };

        let response = adapter.execute_demo(request).await.unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let adapter = McpDemoAdapter::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "unknown.method".to_string(),
            params: None,
            id: Some(serde_json::json!(1)),
        };

        let response = adapter.execute_demo(request).await.unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32601);
    }

    #[test]
    fn test_error_conversion() {
        let error = McpDemoError::UnknownMethod("test".to_string());
        let mcp_error = error.to_mcp_error();

        assert_eq!(mcp_error.code, -32601);
        assert_eq!(mcp_error.message, "Method not found");
    }
}

