// Request processing, metrics, and state tests for unified protocol service
// Included via include!() into mod tests in service_tests.rs

    #[tokio::test]
    async fn test_service_metrics_initialization() {
        let metrics = ServiceMetrics::default();

        let requests = metrics.requests_total.lock();
        assert_eq!(requests.len(), 0);

        let errors = metrics.errors_total.lock();
        assert_eq!(errors.len(), 0);

        let durations = metrics.request_duration_ms.lock();
        assert_eq!(durations.len(), 0);
    }

    #[tokio::test]
    async fn test_app_state_default() {
        let state = AppState::default();

        assert!(Arc::strong_count(&state.template_service) >= 1);
        assert!(Arc::strong_count(&state.analysis_service) >= 1);
        assert!(Arc::strong_count(&state.metrics) >= 1);
    }

    #[tokio::test]
    async fn test_unified_request_creation() {
        let request = UnifiedRequest::new(axum::http::Method::GET, "/api/v1/templates".to_string());

        assert_eq!(request.method, axum::http::Method::GET);
        assert_eq!(request.path, "/api/v1/templates");
        assert!(request.extensions.is_empty());
    }

    #[tokio::test]
    async fn test_process_request_health_check() {
        let service = UnifiedService::new();
        let request = UnifiedRequest::new(axum::http::Method::GET, "/health".to_string());

        let response = service.process_request(request).await.unwrap();
        assert_eq!(response.status, axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_process_request_metrics_endpoint() {
        let service = UnifiedService::new();
        let request = UnifiedRequest::new(axum::http::Method::GET, "/metrics".to_string());

        let response = service.process_request(request).await.unwrap();
        assert_eq!(response.status, axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_record_request_metrics_by_data() {
        let service = UnifiedService::new();
        let response = UnifiedResponse {
            status: axum::http::StatusCode::OK,
            headers: Default::default(),
            body: Default::default(),
            trace_id: uuid::Uuid::new_v4(),
        };

        service.record_request_metrics_by_data(
            "GET",
            "/api/v1/templates",
            &HashMap::new(),
            &response,
            100,
        );

        let requests = service.state.metrics.requests_total.lock();
        assert!(requests.contains_key(&Protocol::Http));
    }

    #[tokio::test]
    async fn test_protocol_extraction_from_path() {
        let service = UnifiedService::new();

        // Test MCP protocol detection
        let protocol = service.extract_protocol_from_path("/mcp/call_tool");
        assert_eq!(protocol, Protocol::Mcp);

        // Test HTTP protocol default
        let protocol = service.extract_protocol_from_path("/api/v1/templates");
        assert_eq!(protocol, Protocol::Http);
    }

    #[tokio::test]
    async fn test_error_metrics_recording() {
        let service = UnifiedService::new();
        let response = UnifiedResponse {
            status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            headers: Default::default(),
            body: Default::default(),
            trace_id: uuid::Uuid::new_v4(),
        };

        service.record_request_metrics_by_data(
            "GET",
            "/api/v1/templates",
            &HashMap::new(),
            &response,
            50,
        );

        let errors = service.state.metrics.errors_total.lock();
        assert!(errors.contains_key(&Protocol::Http));
        assert_eq!(*errors.get(&Protocol::Http).unwrap(), 1);
    }

    #[tokio::test]
    async fn test_duration_metrics_recording() {
        let service = UnifiedService::new();
        let response = UnifiedResponse {
            status: axum::http::StatusCode::OK,
            headers: Default::default(),
            body: Default::default(),
            trace_id: uuid::Uuid::new_v4(),
        };

        service.record_request_metrics_by_data(
            "GET",
            "/api/v1/templates",
            &HashMap::new(),
            &response,
            250,
        );

        let durations = service.state.metrics.request_duration_ms.lock();
        assert!(durations.contains_key(&Protocol::Http));
        assert_eq!(durations.get(&Protocol::Http).unwrap()[0], 250);
    }

    #[tokio::test]
    async fn test_router_cloning() {
        let service = UnifiedService::new();
        let router1 = service.router();
        let router2 = service.router();

        // Both should be valid router instances
        // This test verifies the router can be cloned for multi-threaded usage
        assert!(format!("{:?}", router1).contains("Router"));
        assert!(format!("{:?}", router2).contains("Router"));
    }

    #[tokio::test]
    async fn test_invalid_request_path() {
        let service = UnifiedService::new();
        let request =
            UnifiedRequest::new(axum::http::Method::GET, "/nonexistent/endpoint".to_string());

        let response = service.process_request(request).await.unwrap();
        assert_eq!(response.status, axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_complexity_analysis_params() {
        let params = ComplexityParams {
            project_path: "/test/path".to_string(),
            toolchain: "stable".to_string(),
            format: "json".to_string(),
            max_cyclomatic: Some(20),
            max_cognitive: Some(15),
            top_files: Some(10),
        };

        assert_eq!(params.project_path, "/test/path");
        assert_eq!(params.toolchain, "stable");
        assert_eq!(params.max_cyclomatic, Some(20));
        assert_eq!(params.max_cognitive, Some(15));
    }

    #[tokio::test]
    async fn test_satd_analysis_structure() {
        let analysis = SatdAnalysis {
            project_path: "/test/project".to_string(),
            total_debt_items: 5,
            debt_density: 0.02,
            critical_items: 2,
            categories: HashMap::from([("TODO".to_string(), 3), ("FIXME".to_string(), 2)]),
            files: vec![SatdFile {
                path: "test.rs".to_string(),
                debt_count: 1,
                items: vec![SatdItem {
                    line: 42,
                    category: "TODO".to_string(),
                    severity: "Medium".to_string(),
                    text: "Implement this feature".to_string(),
                    context: None,
                }],
            }],
        };

        assert_eq!(analysis.total_debt_items, 5);
        assert_eq!(analysis.categories.get("TODO"), Some(&3));
        assert_eq!(analysis.files[0].items[0].line, 42);
    }
