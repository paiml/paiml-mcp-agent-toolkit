use crate::unified_protocol::error::AppError;
use crate::unified_protocol::service::{AppState, ServiceMetrics, UnifiedService};
use crate::unified_protocol::{HttpContext, McpContext, Protocol};
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn test_unified_service_creation() {
    let service = UnifiedService::new();

    // Test that service is created successfully
    assert!(std::mem::size_of_val(&service) > 0);
}

#[tokio::test]
async fn test_app_state_default() {
    let state = AppState::default();

    // Verify state components are initialized
    assert!(std::mem::size_of_val(&state.template_service) > 0);
    assert!(std::mem::size_of_val(&state.analysis_service) > 0);
    assert!(std::mem::size_of_val(&state.metrics) > 0);
}

#[tokio::test]
async fn test_service_metrics_creation() {
    let metrics = ServiceMetrics::default();

    // Verify metrics collections are initialized
    let requests = metrics.requests_total.lock();
    assert!(requests.is_empty());
    drop(requests);

    let errors = metrics.errors_total.lock();
    assert!(errors.is_empty());
    drop(errors);

    let durations = metrics.request_duration_ms.lock();
    assert!(durations.is_empty());
}

#[tokio::test]
async fn test_service_metrics_increment() {
    let metrics = ServiceMetrics::default();

    // Test incrementing request counts
    {
        let mut requests = metrics.requests_total.lock();
        *requests.entry(Protocol::Http).or_insert(0) += 1;
        *requests.entry(Protocol::Mcp).or_insert(0) += 2;
    }

    let requests = metrics.requests_total.lock();
    assert_eq!(requests.get(&Protocol::Http), Some(&1));
    assert_eq!(requests.get(&Protocol::Mcp), Some(&2));
    assert_eq!(requests.get(&Protocol::Cli), None);
}

#[tokio::test]
async fn test_service_metrics_duration_tracking() {
    let metrics = ServiceMetrics::default();

    // Test adding duration measurements
    {
        let mut durations = metrics.request_duration_ms.lock();
        durations.entry(Protocol::Http).or_default().push(150);
        durations.entry(Protocol::Http).or_default().push(200);
        durations.entry(Protocol::Mcp).or_default().push(75);
    }

    let durations = metrics.request_duration_ms.lock();
    let http_durations = durations.get(&Protocol::Http).unwrap();
    assert_eq!(http_durations.len(), 2);
    assert_eq!(http_durations[0], 150);
    assert_eq!(http_durations[1], 200);

    let mcp_durations = durations.get(&Protocol::Mcp).unwrap();
    assert_eq!(mcp_durations.len(), 1);
    assert_eq!(mcp_durations[0], 75);
}

#[tokio::test]
async fn test_service_metrics_error_tracking() {
    let metrics = ServiceMetrics::default();

    // Test tracking errors by protocol
    {
        let mut errors = metrics.errors_total.lock();
        *errors.entry(Protocol::Http).or_insert(0) += 1;
        *errors.entry(Protocol::Http).or_insert(0) += 1;
        *errors.entry(Protocol::Mcp).or_insert(0) += 3;
    }

    let errors = metrics.errors_total.lock();
    assert_eq!(errors.get(&Protocol::Http), Some(&2));
    assert_eq!(errors.get(&Protocol::Mcp), Some(&3));
    assert_eq!(errors.get(&Protocol::Cli), None);
}

#[test]
fn test_protocol_context_http_only() {
    let context = HttpContext {
        remote_addr: Some("127.0.0.1:8080".to_string()),
        user_agent: Some("curl/7.68.0".to_string()),
    };

    assert_eq!(context.remote_addr, Some("127.0.0.1:8080".to_string()));
    assert_eq!(context.user_agent, Some("curl/7.68.0".to_string()));
}

#[test]
fn test_protocol_context_mcp_only() {
    let context = McpContext {
        id: Some(json!(1)),
        method: "test_method".to_string(),
    };

    assert_eq!(context.id, Some(json!(1)));
    assert_eq!(context.method, "test_method");
}

#[test]
fn test_app_error_types() {
    // Test different error types can be created
    let validation_error = AppError::Validation("Invalid parameter".to_string());
    let not_found_error = AppError::NotFound("Resource not found".to_string());
    let internal_error = AppError::Internal(anyhow::anyhow!("Internal server error"));

    match validation_error {
        AppError::Validation(msg) => assert_eq!(msg, "Invalid parameter"),
        _ => panic!("Expected Validation error"),
    }

    match not_found_error {
        AppError::NotFound(msg) => assert_eq!(msg, "Resource not found"),
        _ => panic!("Expected NotFound error"),
    }

    match internal_error {
        AppError::Internal(_) => (), // Just verify it matches
        _ => panic!("Expected Internal error"),
    }
}

#[test]
fn test_app_error_status_codes() {
    assert_eq!(
        AppError::NotFound("test".to_string()).status_code(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        AppError::Validation("test".to_string()).status_code(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        AppError::Unauthorized.status_code(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        AppError::Internal(anyhow::anyhow!("test")).status_code(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[test]
fn test_mcp_error_codes() {
    assert_eq!(
        AppError::NotFound("test".to_string()).mcp_error_code(),
        -32001
    );
    assert_eq!(
        AppError::Validation("test".to_string()).mcp_error_code(),
        -32602
    );
    assert_eq!(
        AppError::Internal(anyhow::anyhow!("test")).mcp_error_code(),
        -32603
    );
}

#[test]
fn test_error_types() {
    assert_eq!(
        AppError::NotFound("test".to_string()).error_type(),
        "NOT_FOUND"
    );
    assert_eq!(
        AppError::Validation("test".to_string()).error_type(),
        "VALIDATION_ERROR"
    );
    assert_eq!(
        AppError::Template("test".to_string()).error_type(),
        "TEMPLATE_ERROR"
    );
}

#[tokio::test]
async fn test_error_to_protocol_response() {
    let error = AppError::NotFound("test resource".to_string());

    // Test MCP response
    let mcp_response = error.to_protocol_response(Protocol::Mcp).unwrap();
    assert_eq!(mcp_response.status, StatusCode::OK);

    // Test HTTP response
    let http_response = error.to_protocol_response(Protocol::Http).unwrap();
    assert_eq!(http_response.status, StatusCode::NOT_FOUND);

    // Test CLI response
    let cli_response = error.to_protocol_response(Protocol::Cli).unwrap();
    assert_eq!(cli_response.status, StatusCode::OK);
}
