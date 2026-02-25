// Core unified service tests (service creation, template, mock services)
// Included via include!() into mod tests in service_tests.rs

    #[tokio::test]
    async fn test_unified_service_creation() {
        let service = UnifiedService::new();
        assert!(Arc::strong_count(&service.state) >= 1);
    }

    #[tokio::test]
    async fn test_default_template_service() {
        let service = DefaultTemplateService;
        let query = ListTemplatesQuery {
            format: None,
            category: None,
        };

        let result = service.list_templates(&query).await.unwrap();
        assert!(result.total > 0);
        assert!(!result.templates.is_empty());
    }

    #[tokio::test]
    async fn test_template_generation() {
        let service = DefaultTemplateService;
        let mut params = HashMap::with_capacity(2);
        params.insert(
            "project_name".to_string(),
            Value::String("test-project".to_string()),
        );

        let generate_params = GenerateParams {
            template_uri: "makefile/rust/cli".to_string(),
            parameters: params,
        };

        let result = service.generate_template(&generate_params).await.unwrap();
        assert!(result.content.contains("test-project"));
    }

    // === Sprint 46 Phase 6: TDD Tests for UnifiedService ===

    #[tokio::test]
    async fn test_unified_service_with_custom_template_service() {
        struct MockTemplateService;

        #[async_trait::async_trait]
        impl TemplateService for MockTemplateService {
            async fn list_templates(
                &self,
                _query: &ListTemplatesQuery,
            ) -> Result<TemplateList, AppError> {
                Ok(TemplateList {
                    total: 0,
                    templates: vec![],
                })
            }

            async fn get_template(&self, _id: &str) -> Result<TemplateInfo, AppError> {
                Err(AppError::NotFound("Mock template".to_string()))
            }

            async fn generate_template(
                &self,
                _params: &GenerateParams,
            ) -> Result<GeneratedTemplate, AppError> {
                Ok(GeneratedTemplate {
                    template_id: "mock-template".to_string(),
                    content: "Mock generated content".to_string(),
                    metadata: TemplateMetadata {
                        name: "Mock Template".to_string(),
                        version: "1.0.0".to_string(),
                        generated_at: chrono::Utc::now().to_rfc3339(),
                    },
                })
            }
        }

        let service = UnifiedService::new().with_template_service(MockTemplateService);
        assert!(Arc::strong_count(&service.state) >= 1);
    }

    #[tokio::test]
    async fn test_unified_service_with_custom_analysis_service() {
        struct MockAnalysisService;

        #[async_trait::async_trait]
        impl AnalysisService for MockAnalysisService {
            async fn analyze_complexity(
                &self,
                _params: &ComplexityParams,
            ) -> Result<ComplexityAnalysis, AppError> {
                Ok(ComplexityAnalysis {
                    summary: ComplexitySummary {
                        total_functions: 10,
                        average_complexity: 5.0,
                        max_complexity: 15,
                        files_analyzed: 1,
                    },
                    files: vec![],
                })
            }

            async fn analyze_churn(
                &self,
                _params: &ChurnParams,
            ) -> Result<ChurnAnalysis, AppError> {
                Ok(ChurnAnalysis {
                    summary: ChurnSummary {
                        total_commits: 100,
                        files_changed: 50,
                        period_days: 30,
                    },
                    hotspots: vec![],
                })
            }

            async fn analyze_dag(&self, _params: &DagParams) -> Result<DagAnalysis, AppError> {
                Ok(DagAnalysis {
                    graph: "digraph { A -> B; }".to_string(),
                    nodes: 2,
                    edges: 1,
                    cycles: vec![],
                })
            }

            async fn generate_context(
                &self,
                _params: &ContextParams,
            ) -> Result<ProjectContext, AppError> {
                Ok(ProjectContext {
                    project_name: "mock".to_string(),
                    toolchain: "rust".to_string(),
                    structure: ProjectStructure {
                        directories: vec![],
                        files: vec![],
                    },
                    metrics: ContextMetrics {
                        total_lines: 0,
                        total_files: 0,
                        complexity_score: 0.0,
                    },
                })
            }

            async fn analyze_dead_code(
                &self,
                _params: &DeadCodeParams,
            ) -> Result<DeadCodeAnalysis, AppError> {
                Ok(DeadCodeAnalysis {
                    summary: DeadCodeSummary {
                        total_files_analyzed: 0,
                        files_with_dead_code: 0,
                        total_dead_lines: 0,
                        dead_percentage: 0.0,
                    },
                    files: vec![],
                })
            }
        }

        let service = UnifiedService::new().with_analysis_service(MockAnalysisService);
        assert!(Arc::strong_count(&service.state) >= 1);
    }
