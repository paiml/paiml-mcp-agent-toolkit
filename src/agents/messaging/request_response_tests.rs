#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_request_response() {
        let router = Arc::new(MessageRouter::new());
        let broker = RequestResponseBroker::new(router.clone());

        let from = Uuid::new_v4();
        let to = Uuid::new_v4();

        // Simulate response handler
        let broker_clone = Arc::new(broker);
        let broker_handle = broker_clone.clone();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;

            let response = AgentMessage::new(to, from, "response")
                .unwrap()
                .with_correlation(Uuid::new_v4()); // Would need actual correlation ID

            broker_handle.handle_response(response);
        });

        // Test timeout
        let result = broker_clone
            .request(from, to, "test", Duration::from_millis(5))
            .await;

        assert!(matches!(result, Err(RequestError::Timeout)));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // RequestResponseBroker tests
    #[test]
    fn test_broker_creation() {
        let router = Arc::new(MessageRouter::new());
        let broker = RequestResponseBroker::new(router);

        // Verify broker is created with empty pending requests
        assert!(broker.pending_requests.read().is_empty());
    }

    #[test]
    fn test_handle_response_no_correlation_id() {
        let router = Arc::new(MessageRouter::new());
        let broker = RequestResponseBroker::new(router);

        // Create a message without correlation ID
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "response").unwrap();
        // correlation_id is None by default

        // Should not panic
        broker.handle_response(msg);
    }

    #[test]
    fn test_handle_response_unknown_correlation_id() {
        let router = Arc::new(MessageRouter::new());
        let broker = RequestResponseBroker::new(router);

        // Create a message with unknown correlation ID
        let unknown_correlation = Uuid::new_v4();
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "response")
            .unwrap()
            .with_correlation(unknown_correlation);

        // Should not panic when correlation ID is not found
        broker.handle_response(msg);
    }

    #[actix_rt::test]
    async fn test_request_timeout_cleanup() {
        let router = Arc::new(MessageRouter::new());
        let broker = RequestResponseBroker::new(router);

        let from = Uuid::new_v4();
        let to = Uuid::new_v4();

        // Request with very short timeout
        let result = broker
            .request(from, to, "test", Duration::from_millis(1))
            .await;

        assert!(matches!(result, Err(RequestError::Timeout)));

        // Verify pending request was cleaned up
        assert!(broker.pending_requests.read().is_empty());
    }

    // RequestError tests
    #[test]
    fn test_request_error_timeout() {
        let err = RequestError::Timeout;
        let display_str = format!("{}", err);
        assert!(display_str.contains("timeout") || display_str.contains("Timeout"));

        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Timeout"));
    }

    #[test]
    fn test_request_error_cancelled() {
        let err = RequestError::Cancelled;
        let display_str = format!("{}", err);
        assert!(display_str.contains("cancelled") || display_str.contains("Cancelled"));

        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Cancelled"));
    }

    #[test]
    fn test_request_error_from_router_error() {
        let router_err = RouterError::Expired;
        let request_err: RequestError = router_err.into();

        let display_str = format!("{}", request_err);
        assert!(display_str.contains("Router error") || display_str.contains("expired"));
    }

    // AnalyzeRequest/Response tests
    #[test]
    fn test_analyze_request_serialize() {
        let req = AnalyzeRequest {
            code: "fn main() {}".to_string(),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("fn main()"));

        let deserialized: AnalyzeRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code, req.code);
    }

    #[test]
    fn test_analyze_response_serialize() {
        let resp = AnalyzeResponse {
            complexity: 5,
            lines: 100,
        };

        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: AnalyzeResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.complexity, resp.complexity);
        assert_eq!(deserialized.lines, resp.lines);
    }

    #[test]
    fn test_analyze_request_debug() {
        let req = AnalyzeRequest {
            code: "test code".to_string(),
        };

        let debug_str = format!("{:?}", req);
        assert!(debug_str.contains("AnalyzeRequest"));
        assert!(debug_str.contains("test code"));
    }

    // TransformRequest/Response tests
    #[test]
    fn test_transform_request_serialize() {
        let req = TransformRequest {
            code: "let x = 1".to_string(),
            transform_type: "minify".to_string(),
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: TransformRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.code, req.code);
        assert_eq!(deserialized.transform_type, req.transform_type);
    }

    #[test]
    fn test_transform_response_serialize() {
        let resp = TransformResponse {
            transformed_code: "let x=1".to_string(),
        };

        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: TransformResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.transformed_code, resp.transformed_code);
    }

    #[test]
    fn test_transform_request_debug() {
        let req = TransformRequest {
            code: "code".to_string(),
            transform_type: "format".to_string(),
        };

        let debug_str = format!("{:?}", req);
        assert!(debug_str.contains("TransformRequest"));
    }

    #[test]
    fn test_transform_response_debug() {
        let resp = TransformResponse {
            transformed_code: "transformed".to_string(),
        };

        let debug_str = format!("{:?}", resp);
        assert!(debug_str.contains("TransformResponse"));
    }

    // ValidateRequest/Response tests
    #[test]
    fn test_validate_request_serialize() {
        let req = ValidateRequest {
            code: "function foo() {}".to_string(),
            rules: vec!["no-unused-vars".to_string(), "semi".to_string()],
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: ValidateRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.code, req.code);
        assert_eq!(deserialized.rules, req.rules);
    }

    #[test]
    fn test_validate_response_serialize() {
        let resp = ValidateResponse {
            valid: false,
            errors: vec![
                "Missing semicolon".to_string(),
                "Unused variable".to_string(),
            ],
        };

        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: ValidateResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.valid, resp.valid);
        assert_eq!(deserialized.errors, resp.errors);
    }

    #[test]
    fn test_validate_request_debug() {
        let req = ValidateRequest {
            code: "code".to_string(),
            rules: vec!["rule1".to_string()],
        };

        let debug_str = format!("{:?}", req);
        assert!(debug_str.contains("ValidateRequest"));
    }

    #[test]
    fn test_validate_response_debug() {
        let resp = ValidateResponse {
            valid: true,
            errors: vec![],
        };

        let debug_str = format!("{:?}", resp);
        assert!(debug_str.contains("ValidateResponse"));
    }

    // Request trait default priority tests
    #[test]
    fn test_analyze_request_default_priority() {
        let req = AnalyzeRequest {
            code: "test".to_string(),
        };
        assert_eq!(req.priority(), Priority::Normal);
    }

    #[test]
    fn test_transform_request_default_priority() {
        let req = TransformRequest {
            code: "test".to_string(),
            transform_type: "test".to_string(),
        };
        assert_eq!(req.priority(), Priority::Normal);
    }

    #[test]
    fn test_validate_request_default_priority() {
        let req = ValidateRequest {
            code: "test".to_string(),
            rules: vec![],
        };
        assert_eq!(req.priority(), Priority::Normal);
    }

    // Test successful request-response flow with proper correlation
    #[actix_rt::test]
    async fn test_successful_request_response_flow() {
        let router = Arc::new(MessageRouter::new());
        let broker = Arc::new(RequestResponseBroker::new(router.clone()));

        let from = Uuid::new_v4();
        let to = Uuid::new_v4();

        // Clone broker for the response handler
        let broker_handle = broker.clone();

        // Spawn a task that will insert the correlation_id into pending_requests
        // and then handle the response
        let _broker_for_request = broker.clone();

        // First, manually create a correlation ID and register it
        let correlation_id = Uuid::new_v4();
        let (tx, rx) = tokio::sync::oneshot::channel();
        broker_handle
            .pending_requests
            .write()
            .insert(correlation_id, tx);

        // Now create and handle a response with this correlation ID
        let response = AgentMessage::new(to, from, "response data")
            .unwrap()
            .with_correlation(correlation_id);

        broker_handle.handle_response(response);

        // The receiver should get the response
        let result = rx.await;
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.header.correlation_id, Some(correlation_id));
    }

    // Test RequestError serialization error variant
    #[test]
    fn test_request_error_serialization() {
        // Create a bincode deserialization error with invalid data
        let bad_data = vec![0xFFu8; 16]; // Invalid serialized data
        let bincode_result: Result<String, _> = bincode::deserialize(&bad_data);

        if let Err(e) = bincode_result {
            let request_err = RequestError::Serialization(e);
            let display_str = format!("{}", request_err);
            assert!(display_str.contains("Serialization error"));
        }
    }

    // Test empty rules validation
    #[test]
    fn test_validate_request_empty_rules() {
        let req = ValidateRequest {
            code: "const x = 1;".to_string(),
            rules: vec![],
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: ValidateRequest = serde_json::from_str(&json).unwrap();

        assert!(deserialized.rules.is_empty());
    }

    // Test valid response with no errors
    #[test]
    fn test_validate_response_valid_no_errors() {
        let resp = ValidateResponse {
            valid: true,
            errors: vec![],
        };

        assert!(resp.valid);
        assert!(resp.errors.is_empty());
    }

    // Test AnalyzeResponse debug
    #[test]
    fn test_analyze_response_debug() {
        let resp = AnalyzeResponse {
            complexity: 10,
            lines: 50,
        };

        let debug_str = format!("{:?}", resp);
        assert!(debug_str.contains("AnalyzeResponse"));
        assert!(debug_str.contains("10"));
        assert!(debug_str.contains("50"));
    }
}
