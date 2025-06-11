//! Tests for unified protocol service

use super::*;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct MockHandler;

#[async_trait::async_trait]
impl RequestHandler for MockHandler {
    async fn handle(&self, method: &str, params: Value) -> Result<Value, AppError> {
        match method {
            "test_method" => Ok(json!({"result": "success", "params": params})),
            "error_method" => Err(AppError::NotFound("test resource".to_string())),
            "invalid_method" => Err(AppError::BadRequest("invalid params".to_string())),
            _ => Err(AppError::NotFound(format!("Method not found: {}", method))),
        }
    }
}

#[tokio::test]
async fn test_unified_service_creation() {
    let handler = Arc::new(MockHandler);
    let service = UnifiedService::new(handler.clone());
    
    // Test that service is created with handler
    let req = UnifiedRequest::new(Method::POST, "/test")
        .with_json(&json!({
            "method": "test_method",
            "params": {"key": "value"}
        }))
        .unwrap();
    
    let response = service.process(req).await;
    assert!(response.is_ok());
}

#[tokio::test]
async fn test_process_json_request() {
    let handler = Arc::new(MockHandler);
    let service = UnifiedService::new(handler);
    
    let req = UnifiedRequest::new(Method::POST, "/api/test")
        .with_json(&json!({
            "method": "test_method",
            "params": {"test": 123}
        }))
        .unwrap();
    
    let response = service.process(req).await.unwrap();
    assert_eq!(response.status, StatusCode::OK);
}

#[tokio::test]
async fn test_process_error_response() {
    let handler = Arc::new(MockHandler);
    let service = UnifiedService::new(handler);
    
    let req = UnifiedRequest::new(Method::POST, "/api/test")
        .with_json(&json!({
            "method": "error_method",
            "params": {}
        }))
        .unwrap();
    
    let response = service.process(req).await.unwrap();
    // Error responses may have different status codes based on error type
    assert_ne!(response.status, StatusCode::OK);
}

#[tokio::test]
async fn test_process_invalid_json() {
    let handler = Arc::new(MockHandler);
    let service = UnifiedService::new(handler);
    
    let req = UnifiedRequest::new(Method::POST, "/api/test")
        .with_body(Body::from("invalid json"));
    
    let response = service.process(req).await.unwrap();
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_process_with_trace_id() {
    let handler = Arc::new(MockHandler);
    let service = UnifiedService::new(handler);
    
    let req = UnifiedRequest::new(Method::POST, "/api/test")
        .with_json(&json!({
            "method": "test_method",
            "params": {"trace": true}
        }))
        .unwrap();
    
    let original_trace_id = req.trace_id;
    let response = service.process(req).await.unwrap();
    
    // Response should have the same trace ID
    assert_eq!(response.trace_id, original_trace_id);
}

#[test]
fn test_request_handler_trait() {
    // Just verify the trait can be implemented
    struct TestHandler;
    
    #[async_trait::async_trait]
    impl RequestHandler for TestHandler {
        async fn handle(&self, _method: &str, _params: Value) -> Result<Value, AppError> {
            Ok(json!({"status": "ok"}))
        }
    }
    
    let _handler = TestHandler;
}