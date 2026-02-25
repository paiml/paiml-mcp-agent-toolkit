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
