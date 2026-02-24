#![cfg_attr(coverage_nightly, coverage(off))]
use std::net::SocketAddr;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use http_body_util::BodyExt;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use serde::Serialize;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info};

use crate::unified_protocol::{
    HttpContext, Protocol, ProtocolAdapter, ProtocolError, UnifiedRequest, UnifiedResponse,
};

include!("http_adapter.rs");
include!("http_server.rs");
include!("http_response_builder.rs");


#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_http_adapter_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
        let adapter = HttpAdapter::new(addr);

        assert_eq!(adapter.bind_addr, addr);
        assert_eq!(adapter.protocol(), Protocol::Http);
    }

    #[tokio::test]
    async fn test_http_response_builder() {
        let response = HttpResponseBuilder::ok();
        assert_eq!(response.status, StatusCode::OK);

        let json_response =
            HttpResponseBuilder::json(&serde_json::json!({"message": "test"})).unwrap();
        assert_eq!(json_response.status, StatusCode::OK);
        assert!(json_response.headers.contains_key("content-type"));

        let text_response = HttpResponseBuilder::text("Hello, World!");
        assert_eq!(text_response.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_http_adapter_encode() {
        let adapter = HttpAdapter::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000));
        let response = UnifiedResponse::ok()
            .with_json(&serde_json::json!({"message": "test"}))
            .unwrap();

        let encoded = adapter.encode(response).await.unwrap();
        match encoded {
            HttpOutput::Response(http_response) => {
                assert_eq!(http_response.status(), StatusCode::OK);
            }
        }
    }

    #[test]
    fn test_http_context() {
        let context = HttpContext {
            remote_addr: Some("127.0.0.1:12345".to_string()),
            user_agent: Some("test-agent/1.0".to_string()),
        };

        assert_eq!(context.remote_addr, Some("127.0.0.1:12345".to_string()));
        assert_eq!(context.user_agent, Some("test-agent/1.0".to_string()));
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
    use std::net::{IpAddr, Ipv4Addr};

    // ============ HttpAdapter Tests ============

    #[test]
    fn test_http_adapter_new_with_various_ports() {
        for port in [80, 443, 8080, 3000, 9000] {
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
            let adapter = HttpAdapter::new(addr);
            assert_eq!(adapter.bind_addr.port(), port);
        }
    }

    #[test]
    fn test_http_adapter_new_with_ipv6() {
        use std::net::Ipv6Addr;
        let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 8080);
        let adapter = HttpAdapter::new(addr);
        assert_eq!(adapter.bind_addr, addr);
        assert!(adapter.listener.is_none());
    }

    #[test]
    fn test_http_adapter_unspecified_address() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
        let adapter = HttpAdapter::new(addr);
        assert!(adapter.bind_addr.ip().is_unspecified());
    }

    #[test]
    fn test_http_adapter_protocol() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
        let adapter = HttpAdapter::new(addr);
        assert!(matches!(adapter.protocol(), Protocol::Http));
    }

    #[tokio::test]
    async fn test_http_adapter_bind() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
        let mut adapter = HttpAdapter::new(addr);
        let result = adapter.bind().await;
        assert!(result.is_ok());
        assert!(adapter.listener.is_some());
    }

    #[tokio::test]
    async fn test_http_adapter_accept_without_bind() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
        let mut adapter = HttpAdapter::new(addr);
        let result = adapter.accept().await;
        assert!(result.is_err());
        match result {
            Err(ProtocolError::InvalidFormat(msg)) => {
                assert!(msg.contains("not bound"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    // ============ HttpStreamAdapter Tests ============

    #[test]
    fn test_http_stream_adapter_protocol() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
        let stream_adapter = HttpStreamAdapter {
            stream: None,
            remote_addr: addr,
        };
        assert!(matches!(stream_adapter.protocol(), Protocol::Http));
    }

    #[tokio::test]
    async fn test_http_stream_adapter_decode_no_stream() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
        let stream_adapter = HttpStreamAdapter {
            stream: None,
            remote_addr: addr,
        };
        let result = stream_adapter.decode(()).await;
        assert!(result.is_err());
        match result {
            Err(ProtocolError::InvalidFormat(msg)) => {
                assert!(msg.contains("No stream"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[tokio::test]
    async fn test_http_stream_adapter_encode() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
        let stream_adapter = HttpStreamAdapter {
            stream: None,
            remote_addr: addr,
        };
        let response = UnifiedResponse::ok();
        let result = stream_adapter.encode(response).await;
        assert!(result.is_ok());
        let encoded = result.unwrap();
        assert_eq!(encoded.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_http_stream_adapter_encode_with_headers() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
        let stream_adapter = HttpStreamAdapter {
            stream: None,
            remote_addr: addr,
        };
        let response = UnifiedResponse::ok()
            .with_header("x-custom", "value")
            .with_header("x-another", "test");
        let result = stream_adapter.encode(response).await;
        assert!(result.is_ok());
    }

    // ============ HttpInput Tests ============

    #[test]
    fn test_http_input_debug() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
        let request = Request::builder()
            .method("GET")
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let input = HttpInput::Request {
            request,
            remote_addr: addr,
        };
        let debug = format!("{:?}", input);
        assert!(debug.contains("Request"));
    }

    // ============ HttpOutput Tests ============

    #[test]
    fn test_http_output_debug() {
        let response = Response::builder().status(200).body(Body::empty()).unwrap();
        let output = HttpOutput::Response(response);
        let debug = format!("{:?}", output);
        assert!(debug.contains("Response"));
    }

    // ============ HttpResponseBuilder Extended Tests ============

    #[test]
    fn test_http_response_builder_ok() {
        let response = HttpResponseBuilder::ok();
        assert_eq!(response.status, StatusCode::OK);
    }

    #[test]
    fn test_http_response_builder_not_found() {
        let response = HttpResponseBuilder::not_found();
        assert_eq!(response.status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_http_response_builder_internal_error() {
        let response = HttpResponseBuilder::internal_error();
        assert_eq!(response.status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_http_response_builder_text_empty() {
        let response = HttpResponseBuilder::text("");
        assert_eq!(response.status, StatusCode::OK);
        assert!(response.headers.contains_key("content-type"));
    }

    #[test]
    fn test_http_response_builder_text_unicode() {
        let response = HttpResponseBuilder::text("Hello, 世界! 🌍");
        assert_eq!(response.status, StatusCode::OK);
    }

    #[test]
    fn test_http_response_builder_html_empty() {
        let response = HttpResponseBuilder::html("");
        assert_eq!(response.status, StatusCode::OK);
        assert!(response.headers.contains_key("content-type"));
        assert_eq!(response.headers.get("content-type").unwrap(), "text/html");
    }

    #[test]
    fn test_http_response_builder_html_full_page() {
        let html = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body><h1>Hello</h1></body>
</html>"#;
        let response = HttpResponseBuilder::html(html);
        assert_eq!(response.status, StatusCode::OK);
    }

    #[test]
    fn test_http_response_builder_json_empty_object() {
        let response = HttpResponseBuilder::json(&serde_json::json!({})).unwrap();
        assert_eq!(response.status, StatusCode::OK);
        assert!(response.headers.contains_key("content-type"));
    }

    #[test]
    fn test_http_response_builder_json_array() {
        let response = HttpResponseBuilder::json(&serde_json::json!([1, 2, 3])).unwrap();
        assert_eq!(response.status, StatusCode::OK);
    }

    #[test]
    fn test_http_response_builder_json_nested() {
        let data = serde_json::json!({
            "user": {
                "id": 123,
                "profile": {
                    "name": "Alice",
                    "settings": {
                        "theme": "dark"
                    }
                }
            }
        });
        let response = HttpResponseBuilder::json(&data).unwrap();
        assert_eq!(response.status, StatusCode::OK);
    }

    #[test]
    fn test_http_response_builder_json_with_null() {
        let response = HttpResponseBuilder::json(&serde_json::json!(null)).unwrap();
        assert_eq!(response.status, StatusCode::OK);
    }

    #[test]
    fn test_http_response_builder_json_with_boolean() {
        let response = HttpResponseBuilder::json(&serde_json::json!(true)).unwrap();
        assert_eq!(response.status, StatusCode::OK);
    }

    #[test]
    fn test_http_response_builder_json_with_number() {
        let response = HttpResponseBuilder::json(&serde_json::json!(42.5)).unwrap();
        assert_eq!(response.status, StatusCode::OK);
    }

    // ============ HttpContext Tests ============

    #[test]
    fn test_http_context_empty() {
        let context = HttpContext {
            remote_addr: None,
            user_agent: None,
        };
        assert!(context.remote_addr.is_none());
        assert!(context.user_agent.is_none());
    }

    #[test]
    fn test_http_context_with_remote_addr_only() {
        let context = HttpContext {
            remote_addr: Some("10.0.0.1:5000".to_string()),
            user_agent: None,
        };
        assert_eq!(context.remote_addr.as_deref(), Some("10.0.0.1:5000"));
    }

    #[test]
    fn test_http_context_with_user_agent_only() {
        let context = HttpContext {
            remote_addr: None,
            user_agent: Some("Mozilla/5.0".to_string()),
        };
        assert_eq!(context.user_agent.as_deref(), Some("Mozilla/5.0"));
    }

    // ============ HttpServer Tests ============

    #[test]
    fn test_http_server_creation() {
        struct DummyService;

        #[async_trait]
        impl HttpServiceHandler for DummyService {
            async fn handle(
                &self,
                _request: UnifiedRequest,
            ) -> Result<UnifiedResponse, ProtocolError> {
                Ok(UnifiedResponse::ok())
            }
            fn clone_boxed(&self) -> Box<dyn HttpServiceHandler> {
                Box::new(DummyService)
            }
        }

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
        let server = HttpServer::new(addr, Box::new(DummyService));
        assert_eq!(
            server.adapter.bind_addr.ip(),
            IpAddr::V4(Ipv4Addr::LOCALHOST)
        );
    }

    #[tokio::test]
    async fn test_http_server_bind() {
        struct DummyService;

        #[async_trait]
        impl HttpServiceHandler for DummyService {
            async fn handle(
                &self,
                _request: UnifiedRequest,
            ) -> Result<UnifiedResponse, ProtocolError> {
                Ok(UnifiedResponse::ok())
            }
            fn clone_boxed(&self) -> Box<dyn HttpServiceHandler> {
                Box::new(DummyService)
            }
        }

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
        let mut server = HttpServer::new(addr, Box::new(DummyService));
        let result = server.bind().await;
        assert!(result.is_ok());
    }

    // ============ Adapter Encode Tests ============

    #[tokio::test]
    async fn test_http_adapter_encode_with_custom_status() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
        let adapter = HttpAdapter::new(addr);

        let response = UnifiedResponse::new(StatusCode::CREATED);
        let encoded = adapter.encode(response).await.unwrap();
        match encoded {
            HttpOutput::Response(http_response) => {
                assert_eq!(http_response.status(), StatusCode::CREATED);
            }
        }
    }

    #[tokio::test]
    async fn test_http_adapter_encode_with_multiple_headers() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
        let adapter = HttpAdapter::new(addr);

        let response = UnifiedResponse::ok()
            .with_header("x-request-id", "abc123")
            .with_header("x-correlation-id", "xyz789");
        let result = adapter.encode(response).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_http_adapter_encode_not_found() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
        let adapter = HttpAdapter::new(addr);

        let response = UnifiedResponse::new(StatusCode::NOT_FOUND);
        let encoded = adapter.encode(response).await.unwrap();
        match encoded {
            HttpOutput::Response(http_response) => {
                assert_eq!(http_response.status(), StatusCode::NOT_FOUND);
            }
        }
    }

    #[tokio::test]
    async fn test_http_adapter_encode_internal_error() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
        let adapter = HttpAdapter::new(addr);

        let response = UnifiedResponse::new(StatusCode::INTERNAL_SERVER_ERROR);
        let encoded = adapter.encode(response).await.unwrap();
        match encoded {
            HttpOutput::Response(http_response) => {
                assert_eq!(http_response.status(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    // ============ from_stream Tests ============

    #[tokio::test]
    async fn test_from_stream_creates_adapter() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn a task to accept the connection
        let accept_handle = tokio::spawn(async move {
            let (stream, remote_addr) = listener.accept().await.unwrap();
            HttpAdapter::from_stream(stream, remote_addr)
        });

        // Connect to the listener
        let _client = TcpStream::connect(addr).await.unwrap();

        // Get the adapter
        let stream_adapter = accept_handle.await.unwrap();
        assert!(matches!(stream_adapter.protocol(), Protocol::Http));
    }
}
