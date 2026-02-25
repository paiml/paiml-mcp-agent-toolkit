// Handler integration tests and MCP endpoint tests for unified protocol service
// Included via include!() from service_tests.rs

    // === Handler Integration Tests ===

    #[tokio::test]
    async fn test_handler_list_templates() {
        let state = Arc::new(AppState::default());
        let query = Query(ListTemplatesQuery {
            format: Some("json".to_string()),
            category: Some("cli".to_string()),
        });

        let result = handlers::list_templates(Extension(state), query).await;
        assert!(result.is_ok());
        let Json(templates) = result.unwrap();
        assert!(templates.total >= 1);
    }

    #[tokio::test]
    async fn test_handler_get_template_found() {
        let state = Arc::new(AppState::default());
        let path = Path("makefile/rust/cli".to_string());

        let result = handlers::get_template(Extension(state), path).await;
        assert!(result.is_ok());
        let Json(template) = result.unwrap();
        assert_eq!(template.id, "makefile/rust/cli");
    }

    #[tokio::test]
    async fn test_handler_get_template_not_found() {
        let state = Arc::new(AppState::default());
        let path = Path("nonexistent/template".to_string());

        let result = handlers::get_template(Extension(state), path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handler_generate_template() {
        let state = Arc::new(AppState::default());
        let mut params = HashMap::new();
        params.insert(
            "project_name".to_string(),
            Value::String("test-generated".to_string()),
        );

        let json_params = Json(GenerateParams {
            template_uri: "makefile/rust/cli".to_string(),
            parameters: params,
        });

        let result = handlers::generate_template(Extension(state), json_params).await;
        assert!(result.is_ok());
        let Json(generated) = result.unwrap();
        assert!(generated.content.contains("test-generated"));
    }

    #[tokio::test]
    async fn test_handler_analyze_complexity() {
        let state = Arc::new(AppState::default());
        let params = Json(ComplexityParams {
            project_path: ".".to_string(),
            toolchain: "rust".to_string(),
            format: "json".to_string(),
            max_cyclomatic: Some(20),
            max_cognitive: Some(15),
            top_files: Some(10),
        });

        let result = handlers::analyze_complexity(Extension(state), params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_analyze_complexity_get() {
        let state = Arc::new(AppState::default());
        let query = Query(ComplexityQueryParams {
            project_path: Some(".".to_string()),
            toolchain: Some("rust".to_string()),
            format: Some("json".to_string()),
            max_cyclomatic: Some(25),
            max_cognitive: None,
            top_files: None,
        });

        let result = handlers::analyze_complexity_get(Extension(state), query).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_analyze_complexity_get_defaults() {
        let state = Arc::new(AppState::default());
        let query = Query(ComplexityQueryParams {
            project_path: None,
            toolchain: None,
            format: None,
            max_cyclomatic: None,
            max_cognitive: None,
            top_files: None,
        });

        let result = handlers::analyze_complexity_get(Extension(state), query).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_analyze_churn() {
        let state = Arc::new(AppState::default());
        let params = Json(ChurnParams {
            project_path: ".".to_string(),
            period_days: 30,
            format: "json".to_string(),
        });

        let result = handlers::analyze_churn(Extension(state), params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_generate_context() {
        let state = Arc::new(AppState::default());
        let params = Json(ContextParams {
            toolchain: "rust".to_string(),
            project_path: ".".to_string(),
            format: "json".to_string(),
        });

        let result = handlers::generate_context(Extension(state), params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_analyze_dead_code() {
        let state = Arc::new(AppState::default());
        let params = Json(DeadCodeParams {
            project_path: ".".to_string(),
            format: "json".to_string(),
            top_files: Some(5),
            include_unreachable: true,
            min_dead_lines: 1,
            include_tests: false,
        });

        let result = handlers::analyze_dead_code(Extension(state), params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_health_check() {
        use axum::response::IntoResponse;

        let response = handlers::health_check().await;
        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handler_metrics() {
        use axum::response::IntoResponse;

        let state = Arc::new(AppState::default());

        // Add some metrics
        {
            let mut requests = state.metrics.requests_total.lock();
            *requests.entry(Protocol::Http).or_insert(0) += 5;
        }

        let response = handlers::metrics(Extension(state)).await;
        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // === MCP Endpoint Tests ===

    #[tokio::test]
    async fn test_mcp_endpoint_list_templates() {
        let state = Arc::new(AppState::default());
        let method = Path("list_templates".to_string());
        let params = Json(serde_json::json!({}));

        let result = handlers::mcp_endpoint(Extension(state), method, params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_endpoint_generate_template() {
        let state = Arc::new(AppState::default());
        let method = Path("generate_template".to_string());
        let params = Json(serde_json::json!({
            "template_uri": "makefile/rust/cli",
            "parameters": {"project_name": "mcp-test"}
        }));

        let result = handlers::mcp_endpoint(Extension(state), method, params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_endpoint_analyze_complexity() {
        let state = Arc::new(AppState::default());
        let method = Path("analyze_complexity".to_string());
        let params = Json(serde_json::json!({
            "project_path": ".",
            "toolchain": "rust",
            "format": "json"
        }));

        let result = handlers::mcp_endpoint(Extension(state), method, params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_endpoint_analyze_dead_code() {
        let state = Arc::new(AppState::default());
        let method = Path("analyze_dead_code".to_string());
        let params = Json(serde_json::json!({
            "project_path": ".",
            "format": "json"
        }));

        let result = handlers::mcp_endpoint(Extension(state), method, params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_endpoint_unknown_method() {
        let state = Arc::new(AppState::default());
        let method = Path("unknown_method".to_string());
        let params = Json(serde_json::json!({}));

        let result = handlers::mcp_endpoint(Extension(state), method, params).await;
        assert!(result.is_err());
    }
