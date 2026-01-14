use crate::unified_protocol::adapters::http::{HttpAdapter, HttpOutput};
use crate::unified_protocol::{HttpContext, Protocol, ProtocolAdapter};
use axum::body::Body;
use axum::http::{Response, StatusCode};
use serde_json::json;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[tokio::test]
async fn test_http_adapter_creation() {
    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
    let adapter = HttpAdapter::new(bind_addr);

    assert_eq!(adapter.protocol(), Protocol::Http);
}

#[tokio::test]
async fn test_http_adapter_bind() {
    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
    let mut adapter = HttpAdapter::new(bind_addr);

    let result = adapter.bind().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_http_output_creation() {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(json!({"status": "success"}).to_string()))
        .unwrap();

    let output = HttpOutput::Response(response);

    // Test that output was created successfully
    match output {
        HttpOutput::Response(resp) => {
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }
}

#[tokio::test]
async fn test_http_context_creation() {
    let context = HttpContext {
        remote_addr: Some("192.168.1.100:8080".to_string()),
        user_agent: Some(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
        ),
    };

    assert_eq!(context.remote_addr, Some("192.168.1.100:8080".to_string()));
    assert!(context.user_agent.is_some());
    assert!(context.user_agent.unwrap().contains("Mozilla"));
}

#[tokio::test]
async fn test_http_context_with_no_remote_addr() {
    let context = HttpContext {
        remote_addr: None,
        user_agent: Some("curl/7.68.0".to_string()),
    };

    assert!(context.remote_addr.is_none());
    assert_eq!(context.user_agent, Some("curl/7.68.0".to_string()));
}

#[tokio::test]
async fn test_http_context_with_no_user_agent() {
    let context = HttpContext {
        remote_addr: Some("10.0.0.1:3000".to_string()),
        user_agent: None,
    };

    assert_eq!(context.remote_addr, Some("10.0.0.1:3000".to_string()));
    assert!(context.user_agent.is_none());
}

#[tokio::test]
async fn test_http_context_empty() {
    let context = HttpContext {
        remote_addr: None,
        user_agent: None,
    };

    assert!(context.remote_addr.is_none());
    assert!(context.user_agent.is_none());
}

#[test]
fn test_protocol_adapter_trait() {
    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
    let adapter = HttpAdapter::new(bind_addr);

    // Test trait method
    assert_eq!(adapter.protocol(), Protocol::Http);
}

#[tokio::test]
async fn test_http_status_code_variations() {
    for status in [
        StatusCode::OK,
        StatusCode::CREATED,
        StatusCode::BAD_REQUEST,
        StatusCode::NOT_FOUND,
        StatusCode::INTERNAL_SERVER_ERROR,
    ] {
        let response = Response::builder()
            .status(status)
            .body(Body::empty())
            .unwrap();

        let output = HttpOutput::Response(response);

        // Test that output was created successfully for all status codes
        match output {
            HttpOutput::Response(resp) => {
                assert_eq!(resp.status(), status);
            }
        }
    }
}

#[tokio::test]
async fn test_http_response_with_json() {
    let response_data = json!({
        "status": "success",
        "data": {
            "total_files": 42,
            "complexity_scores": [1.2, 2.5, 3.8],
            "recommendations": ["Reduce complexity in src/main.rs", "Add unit tests"]
        },
        "metadata": {
            "analysis_duration_ms": 1250,
            "timestamp": "2024-01-15T10:30:00Z"
        }
    });

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .header("x-response-time", "1250ms")
        .body(Body::from(response_data.to_string()))
        .unwrap();

    let output = HttpOutput::Response(response);

    // Test that JSON response output was created successfully
    match output {
        HttpOutput::Response(resp) => {
            assert_eq!(resp.status(), StatusCode::OK);
            assert!(resp.headers().contains_key("content-type"));
            assert!(resp.headers().contains_key("x-response-time"));
        }
    }
}

#[tokio::test]
async fn test_http_error_responses() {
    let error_responses = [
        (StatusCode::BAD_REQUEST, "Invalid parameters"),
        (StatusCode::NOT_FOUND, "Resource not found"),
        (StatusCode::INTERNAL_SERVER_ERROR, "Analysis failed"),
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Service temporarily unavailable",
        ),
    ];

    for (status, message) in error_responses {
        let error_body = json!({
            "error": message,
            "timestamp": "2024-01-15T10:30:00Z"
        });

        let response = Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(Body::from(error_body.to_string()))
            .unwrap();

        let output = HttpOutput::Response(response);

        match output {
            HttpOutput::Response(resp) => {
                assert_eq!(resp.status(), status);
            }
        }
    }
}
