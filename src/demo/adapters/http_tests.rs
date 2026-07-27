#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_adapter_metadata() {
        let adapter = HttpDemoAdapter::new();
        let metadata = adapter.get_protocol_metadata().await;

        assert_eq!(metadata.name, "http");
        assert_eq!(metadata.version, "1.1");
        assert!(!metadata.capabilities.is_empty());
        assert!(!metadata.example_requests.is_empty());
    }

    #[test]
    fn test_http_request_from_value() {
        let value = serde_json::json!({
            "method": "GET",
            "path": "/demo/analyze",
            "query": {"path": "/test"},
            "headers": {"Accept": "application/json"}
        });

        let request = HttpRequest::from(value);
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/demo/analyze");
        assert_eq!(request.query_params.get("path"), Some(&"/test".to_string()));
    }

    #[tokio::test]
    async fn test_api_introspection() {
        let adapter = HttpDemoAdapter::new();
        let body = adapter.handle_api_introspection().await.unwrap();

        match body {
            HttpResponseBody::Introspection { endpoints, .. } => {
                assert!(!endpoints.is_empty());
                assert!(endpoints.iter().any(|e| e.path == "/demo/analyze"));
                assert!(endpoints.iter().any(|e| e.path == "/demo/api"));
            }
            _ => panic!("Expected introspection response"),
        }
    }

    #[tokio::test]
    async fn test_context_analysis() {
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

        let adapter = HttpDemoAdapter::new();
        let result = adapter
            .execute_context_analysis(&temp_path.to_string_lossy())
            .await
            .unwrap();

        // Deep context analysis returns specific fields from the DeepContext struct
        assert!(result.get("metadata").is_some());
        assert!(result.get("quality_scorecard").is_some());
        assert!(result.get("analyses").is_some());
        assert!(result.get("file_tree").is_some());
    }
}

