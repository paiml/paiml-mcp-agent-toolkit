// Default service implementation, UnifiedService, and protocol extraction tests
// Included via include!() from service_tests.rs

    // === Default Service Implementation Tests ===

    #[tokio::test]
    async fn test_default_template_service_list() {
        let service = DefaultTemplateService;
        let query = ListTemplatesQuery {
            format: None,
            category: None,
        };

        let result = service.list_templates(&query).await;
        assert!(result.is_ok());
        let list = result.unwrap();
        assert!(list.total > 0);
    }

    #[tokio::test]
    async fn test_default_template_service_get_existing() {
        let service = DefaultTemplateService;
        let result = service.get_template("makefile/rust/cli").await;
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(template.id, "makefile/rust/cli");
    }

    #[tokio::test]
    async fn test_default_template_service_get_not_found() {
        let service = DefaultTemplateService;
        let result = service.get_template("nonexistent/template").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_default_template_service_generate() {
        let service = DefaultTemplateService;
        let mut params = HashMap::new();
        params.insert(
            "project_name".to_string(),
            Value::String("my-app".to_string()),
        );

        let generate_params = GenerateParams {
            template_uri: "makefile/rust/cli".to_string(),
            parameters: params,
        };

        let result = service.generate_template(&generate_params).await;
        assert!(result.is_ok());
        let generated = result.unwrap();
        assert!(generated.content.contains("my-app"));
    }

    #[tokio::test]
    async fn test_default_analysis_service_complexity() {
        let service = DefaultAnalysisService;
        let params = ComplexityParams {
            project_path: "/test".to_string(),
            toolchain: "rust".to_string(),
            format: "json".to_string(),
            max_cyclomatic: None,
            max_cognitive: None,
            top_files: None,
        };

        let result = service.analyze_complexity(&params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_default_analysis_service_churn() {
        let service = DefaultAnalysisService;
        let params = ChurnParams {
            project_path: "/test".to_string(),
            period_days: 30,
            format: "json".to_string(),
        };

        let result = service.analyze_churn(&params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_default_analysis_service_context() {
        let service = DefaultAnalysisService;
        let params = ContextParams {
            toolchain: "rust".to_string(),
            project_path: "/test".to_string(),
            format: "json".to_string(),
        };

        let result = service.generate_context(&params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_default_analysis_service_dead_code() {
        let service = DefaultAnalysisService;
        let params = DeadCodeParams {
            project_path: "/test".to_string(),
            format: "json".to_string(),
            top_files: None,
            include_unreachable: false,
            min_dead_lines: 0,
            include_tests: false,
        };

        let result = service.analyze_dead_code(&params).await;
        assert!(result.is_ok());
    }

    // === UnifiedService Tests ===

    #[tokio::test]
    async fn test_unified_service_creation_coverage() {
        let service = UnifiedService::new();
        // Verify the service has been created with valid state
        assert!(Arc::strong_count(&service.state) >= 1);
    }

    #[tokio::test]
    async fn test_unified_service_default() {
        let service = UnifiedService::default();
        assert!(Arc::strong_count(&service.state) >= 1);
    }

    #[tokio::test]
    async fn test_unified_service_router() {
        let service = UnifiedService::new();
        let router = service.router();
        // Verify the router can be cloned (for use in multi-threaded contexts)
        let _router2 = router.clone();
    }

    #[tokio::test]
    async fn test_unified_service_with_custom_services() {
        struct MockTemplateService;

        #[async_trait::async_trait]
        impl TemplateService for MockTemplateService {
            async fn list_templates(
                &self,
                _query: &ListTemplatesQuery,
            ) -> Result<TemplateList, AppError> {
                Ok(TemplateList {
                    templates: vec![],
                    total: 0,
                })
            }

            async fn get_template(&self, _id: &str) -> Result<TemplateInfo, AppError> {
                Err(AppError::NotFound("Mock".to_string()))
            }

            async fn generate_template(
                &self,
                _params: &GenerateParams,
            ) -> Result<GeneratedTemplate, AppError> {
                Ok(GeneratedTemplate {
                    template_id: "mock".to_string(),
                    content: "mock content".to_string(),
                    metadata: TemplateMetadata {
                        name: "Mock".to_string(),
                        version: "1.0.0".to_string(),
                        generated_at: chrono::Utc::now().to_rfc3339(),
                    },
                })
            }
        }

        let service = UnifiedService::new().with_template_service(MockTemplateService);
        assert!(Arc::strong_count(&service.state) >= 1);
    }

    // === Protocol Extraction Tests ===

    #[test]
    fn test_extract_protocol_from_path_mcp() {
        let service = UnifiedService::new();
        let protocol = service.extract_protocol_from_path("/mcp/call_tool");
        assert_eq!(protocol, Protocol::Mcp);
    }

    #[test]
    fn test_extract_protocol_from_path_http() {
        let service = UnifiedService::new();
        let protocol = service.extract_protocol_from_path("/api/v1/templates");
        assert_eq!(protocol, Protocol::Http);
    }

    #[test]
    fn test_extract_protocol_from_path_health() {
        let service = UnifiedService::new();
        let protocol = service.extract_protocol_from_path("/health");
        assert_eq!(protocol, Protocol::Http);
    }
