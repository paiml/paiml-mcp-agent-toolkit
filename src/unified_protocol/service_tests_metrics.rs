// Metrics recording, service metrics, and AppState tests for unified protocol service
// Included via include!() from service_tests.rs

    // === Record Metrics Tests ===

    #[test]
    fn test_record_request_metrics_success() {
        let service = UnifiedService::new();
        let response = UnifiedResponse {
            status: StatusCode::OK,
            headers: Default::default(),
            body: Body::empty(),
            trace_id: uuid::Uuid::new_v4(),
        };

        service.record_request_metrics_by_data(
            "GET",
            "/api/v1/templates",
            &HashMap::new(),
            &response,
            150,
        );

        let requests = service.state.metrics.requests_total.lock();
        assert!(requests.get(&Protocol::Http).is_some());
        assert_eq!(*requests.get(&Protocol::Http).unwrap(), 1);
    }

    #[test]
    fn test_record_request_metrics_error_4xx() {
        let service = UnifiedService::new();
        let response = UnifiedResponse {
            status: StatusCode::BAD_REQUEST,
            headers: Default::default(),
            body: Body::empty(),
            trace_id: uuid::Uuid::new_v4(),
        };

        service.record_request_metrics_by_data(
            "POST",
            "/api/v1/generate",
            &HashMap::new(),
            &response,
            50,
        );

        let errors = service.state.metrics.errors_total.lock();
        assert!(errors.get(&Protocol::Http).is_some());
        assert_eq!(*errors.get(&Protocol::Http).unwrap(), 1);
    }

    #[test]
    fn test_record_request_metrics_error_5xx() {
        let service = UnifiedService::new();
        let response = UnifiedResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            headers: Default::default(),
            body: Body::empty(),
            trace_id: uuid::Uuid::new_v4(),
        };

        service.record_request_metrics_by_data(
            "GET",
            "/api/v1/analyze/complexity",
            &HashMap::new(),
            &response,
            200,
        );

        let errors = service.state.metrics.errors_total.lock();
        assert!(errors.get(&Protocol::Http).is_some());
    }

    #[test]
    fn test_record_request_metrics_with_mcp_protocol() {
        let service = UnifiedService::new();
        let response = UnifiedResponse {
            status: StatusCode::OK,
            headers: Default::default(),
            body: Body::empty(),
            trace_id: uuid::Uuid::new_v4(),
        };

        let mut extensions = HashMap::new();
        extensions.insert(
            "protocol".to_string(),
            serde_json::to_value(Protocol::Mcp).unwrap(),
        );

        service.record_request_metrics_by_data(
            "POST",
            "/mcp/call_tool",
            &extensions,
            &response,
            75,
        );

        let requests = service.state.metrics.requests_total.lock();
        assert!(requests.get(&Protocol::Mcp).is_some());
    }

    #[test]
    fn test_record_request_metrics_multiple_calls() {
        let service = UnifiedService::new();
        let response = UnifiedResponse {
            status: StatusCode::OK,
            headers: Default::default(),
            body: Body::empty(),
            trace_id: uuid::Uuid::new_v4(),
        };

        for i in 0..5 {
            service.record_request_metrics_by_data(
                "GET",
                "/health",
                &HashMap::new(),
                &response,
                10 * (i + 1),
            );
        }

        let requests = service.state.metrics.requests_total.lock();
        assert_eq!(*requests.get(&Protocol::Http).unwrap(), 5);

        let durations = service.state.metrics.request_duration_ms.lock();
        assert_eq!(durations.get(&Protocol::Http).unwrap().len(), 5);
    }

    // === Service Metrics Tests ===

    #[test]
    fn test_service_metrics_default() {
        let metrics = ServiceMetrics::default();
        let requests = metrics.requests_total.lock();
        assert!(requests.is_empty());
    }

    #[test]
    fn test_service_metrics_increment() {
        let metrics = ServiceMetrics::default();

        {
            let mut requests = metrics.requests_total.lock();
            *requests.entry(Protocol::Http).or_insert(0) += 1;
            *requests.entry(Protocol::Http).or_insert(0) += 1;
            *requests.entry(Protocol::Mcp).or_insert(0) += 1;
        }

        let requests = metrics.requests_total.lock();
        assert_eq!(*requests.get(&Protocol::Http).unwrap(), 2);
        assert_eq!(*requests.get(&Protocol::Mcp).unwrap(), 1);
    }

    #[test]
    fn test_service_metrics_errors() {
        let metrics = ServiceMetrics::default();

        {
            let mut errors = metrics.errors_total.lock();
            *errors.entry(Protocol::Http).or_insert(0) += 3;
        }

        let errors = metrics.errors_total.lock();
        assert_eq!(*errors.get(&Protocol::Http).unwrap(), 3);
    }

    #[test]
    fn test_service_metrics_durations() {
        let metrics = ServiceMetrics::default();

        {
            let mut durations = metrics.request_duration_ms.lock();
            durations.entry(Protocol::Http).or_default().push(100);
            durations.entry(Protocol::Http).or_default().push(200);
            durations.entry(Protocol::Http).or_default().push(150);
        }

        let durations = metrics.request_duration_ms.lock();
        let http_durations = durations.get(&Protocol::Http).unwrap();
        assert_eq!(http_durations.len(), 3);
        assert_eq!(http_durations[0], 100);
    }

    // === AppState Tests ===

    #[test]
    fn test_app_state_creation() {
        let state = AppState::default();
        // Verify all services are initialized
        assert!(Arc::strong_count(&state.template_service) >= 1);
        assert!(Arc::strong_count(&state.analysis_service) >= 1);
        assert!(Arc::strong_count(&state.metrics) >= 1);
    }
