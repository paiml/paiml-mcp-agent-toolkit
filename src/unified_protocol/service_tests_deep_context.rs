// Deep context parameter parsing and process request tests for unified protocol service
// Included via include!() from service_tests.rs

    // === Deep Context Parameter Parsing Tests ===

    #[test]
    fn test_parse_deep_context_params_minimal() {
        let params = serde_json::json!({
            "project_path": "/test/path"
        });

        let result = handlers::parse_deep_context_params(&params);
        assert!(result.is_ok());
        let (path, config) = result.unwrap();
        assert_eq!(path.to_string_lossy(), "/test/path");
        assert_eq!(config.period_days, 30); // default
    }

    #[test]
    fn test_parse_deep_context_params_full() {
        let params = serde_json::json!({
            "project_path": "/test/project",
            "period_days": 60,
            "parallel": 4,
            "include": ["ast", "complexity", "churn"]
        });

        let result = handlers::parse_deep_context_params(&params);
        assert!(result.is_ok());
        let (path, config) = result.unwrap();
        assert_eq!(path.to_string_lossy(), "/test/project");
        assert_eq!(config.period_days, 60);
        assert_eq!(config.parallel, 4);
        assert!(!config.include_analyses.is_empty());
    }

    #[test]
    fn test_parse_deep_context_params_all_analysis_types() {
        let params = serde_json::json!({
            "project_path": ".",
            "include": ["ast", "complexity", "churn", "dag", "dead-code", "satd", "tdg"]
        });

        let result = handlers::parse_deep_context_params(&params);
        assert!(result.is_ok());
        let (_, config) = result.unwrap();
        assert_eq!(config.include_analyses.len(), 7);
    }

    #[test]
    fn test_parse_deep_context_params_unknown_analysis_type() {
        let params = serde_json::json!({
            "project_path": ".",
            "include": ["unknown", "invalid"]
        });

        let result = handlers::parse_deep_context_params(&params);
        assert!(result.is_ok());
        let (_, config) = result.unwrap();
        assert!(config.include_analyses.is_empty());
    }

    // === Process Request Tests ===

    #[tokio::test]
    async fn test_process_request_health() {
        let service = UnifiedService::new();
        let request = UnifiedRequest::new(axum::http::Method::GET, "/health".to_string());

        let response = service.process_request(request).await;
        assert!(response.is_ok());
        assert_eq!(response.unwrap().status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_process_request_metrics() {
        let service = UnifiedService::new();
        let request = UnifiedRequest::new(axum::http::Method::GET, "/metrics".to_string());

        let response = service.process_request(request).await;
        assert!(response.is_ok());
        assert_eq!(response.unwrap().status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_process_request_not_found() {
        let service = UnifiedService::new();
        let request = UnifiedRequest::new(axum::http::Method::GET, "/nonexistent/path".to_string());

        let response = service.process_request(request).await;
        assert!(response.is_ok());
        assert_eq!(response.unwrap().status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_process_request_with_extensions() {
        let service = UnifiedService::new();
        let request = UnifiedRequest::new(axum::http::Method::GET, "/health".to_string())
            .with_extension("protocol", Protocol::Http)
            .with_extension("request_id", "test-123");

        let response = service.process_request(request).await;
        assert!(response.is_ok());
    }
