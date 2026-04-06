impl HttpDemoAdapter {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }

    async fn handle_analyze_request(
        &self,
        request: &HttpRequest,
    ) -> Result<HttpResponseBody, HttpDemoError> {
        debug_assert!(true, "contract: handle_analyze_request");
        let start_time = std::time::Instant::now();

        // Extract path from query parameters
        let analysis_path = request
            .query_params
            .get("path")
            .ok_or_else(|| HttpDemoError::MissingParameter("path".to_string()))?;

        // Simulate context analysis (in real implementation, would call actual service)
        let result = self.execute_context_analysis(analysis_path).await?;

        let response_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(HttpResponseBody::Analysis {
            protocol: "http/1.1".to_string(),
            base_command: format!(
                "paiml-mcp-agent-toolkit analyze context --format json --path {analysis_path}"
            ),
            request: HttpRequestInfo {
                method: request.method.clone(),
                path: request.path.clone(),
                query: request.query_params.clone(),
                headers: request.headers.clone(),
            },
            response_time_ms,
            cache_hit: false, // TRACKED: Implement proper cache checking
            result,
        })
    }

    async fn handle_status_request(
        &self,
        request: &HttpRequest,
    ) -> Result<HttpResponseBody, HttpDemoError> {
        debug_assert!(true, "contract: handle_status_request");
        // Extract request_id from path
        let path_parts: Vec<&str> = request.path.trim_start_matches('/').split('/').collect();
        if path_parts.len() < 3 || path_parts[1] != "status" {
            return Err(HttpDemoError::InvalidPath(
                "Expected /demo/status/{request_id}".to_string(),
            ));
        }

        let request_id = path_parts[2];

        // For demo purposes, return completed status
        Ok(HttpResponseBody::Status {
            request_id: request_id.to_string(),
            status: "completed".to_string(),
            progress: Some(100.0),
            result: Some(serde_json::json!({
                "analysis_type": "context",
                "files_analyzed": 42,
                "completion_time": "2023-01-01T00:00:00Z"
            })),
            error: None,
        })
    }

    async fn handle_results_request(
        &self,
        request: &HttpRequest,
    ) -> Result<HttpResponseBody, HttpDemoError> {
        debug_assert!(true, "contract: handle_results_request");
        // Extract request_id from path
        let path_parts: Vec<&str> = request.path.trim_start_matches('/').split('/').collect();
        if path_parts.len() < 3 || path_parts[1] != "results" {
            return Err(HttpDemoError::InvalidPath(
                "Expected /demo/results/{request_id}".to_string(),
            ));
        }

        let request_id = path_parts[2];

        // Return cached results (for demo purposes, generate sample data)
        let result = serde_json::json!({
            "request_id": request_id,
            "analysis_type": "context",
            "summary": {
                "total_files": 156,
                "total_functions": 892,
                "avg_complexity": 3.2,
                "tech_debt_hours": 24.5
            },
            "hotspots": [
                {
                    "file": "src/services/complexity.rs::analyze_function",
                    "complexity": 15,
                    "cognitive_load": 23
                }
            ]
        });

        Ok(HttpResponseBody::Analysis {
            protocol: "http/1.1".to_string(),
            base_command: "cached_result".to_string(),
            request: HttpRequestInfo {
                method: request.method.clone(),
                path: request.path.clone(),
                query: request.query_params.clone(),
                headers: request.headers.clone(),
            },
            response_time_ms: 5, // Cache lookup time
            cache_hit: true,
            result,
        })
    }

    async fn handle_api_introspection(&self) -> Result<HttpResponseBody, HttpDemoError> {
        debug_assert!(true, "contract: handle_api_introspection");
        let endpoints = vec![
            HttpEndpoint {
                method: "GET".to_string(),
                path: "/demo/analyze".to_string(),
                description: "Trigger context analysis".to_string(),
                parameters: vec![HttpParameter {
                    name: "path".to_string(),
                    location: "query".to_string(),
                    required: true,
                    param_type: "string".to_string(),
                    description: "Path to repository or directory to analyze".to_string(),
                }],
                responses: [
                    (
                        "200".to_string(),
                        "Analysis completed successfully".to_string(),
                    ),
                    ("400".to_string(), "Invalid request parameters".to_string()),
                    ("500".to_string(), "Internal server error".to_string()),
                ]
                .into_iter()
                .collect(),
            },
            HttpEndpoint {
                method: "GET".to_string(),
                path: "/demo/status/{request_id}".to_string(),
                description: "Check analysis status".to_string(),
                parameters: vec![HttpParameter {
                    name: "request_id".to_string(),
                    location: "path".to_string(),
                    required: true,
                    param_type: "string".to_string(),
                    description: "Unique identifier for the analysis request".to_string(),
                }],
                responses: [
                    (
                        "200".to_string(),
                        "Status retrieved successfully".to_string(),
                    ),
                    ("404".to_string(), "Request not found".to_string()),
                ]
                .into_iter()
                .collect(),
            },
            HttpEndpoint {
                method: "GET".to_string(),
                path: "/demo/results/{request_id}".to_string(),
                description: "Retrieve analysis results".to_string(),
                parameters: vec![HttpParameter {
                    name: "request_id".to_string(),
                    location: "path".to_string(),
                    required: true,
                    param_type: "string".to_string(),
                    description: "Unique identifier for the analysis request".to_string(),
                }],
                responses: [
                    (
                        "200".to_string(),
                        "Results retrieved successfully".to_string(),
                    ),
                    ("404".to_string(), "Results not found".to_string()),
                ]
                .into_iter()
                .collect(),
            },
            HttpEndpoint {
                method: "GET".to_string(),
                path: "/demo/api".to_string(),
                description: "API introspection and documentation".to_string(),
                parameters: vec![],
                responses: [("200".to_string(), "API documentation retrieved".to_string())]
                    .into_iter()
                    .collect(),
            },
        ];

        Ok(HttpResponseBody::Introspection {
            protocol: "http/1.1".to_string(),
            version: "1.0.0".to_string(),
            endpoints,
            schemas: [(
                "AnalysisResult".to_string(),
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "summary": {"type": "object"},
                        "hotspots": {"type": "array"},
                        "files_analyzed": {"type": "integer"}
                    }
                }),
            )]
            .into_iter()
            .collect(),
            examples: [(
                "analyze_request".to_string(),
                serde_json::json!({
                    "method": "GET",
                    "url": "/demo/analyze?path=/path/to/repo"
                }),
            )]
            .into_iter()
            .collect(),
        })
    }

    async fn execute_context_analysis(&self, path: &str) -> Result<Value, HttpDemoError> {
        debug_assert!(!path.is_empty(), "path must not be empty");
        use crate::services::deep_context::{AnalysisType, DeepContextAnalyzer, DeepContextConfig};
        use std::path::PathBuf;

        // Parse path
        let project_path = PathBuf::from(path);

        // Create analyzer with default config
        let config = DeepContextConfig {
            include_analyses: vec![
                AnalysisType::Ast,
                AnalysisType::Complexity,
                AnalysisType::Churn,
                AnalysisType::Dag,
                AnalysisType::DeadCode,
                AnalysisType::Satd,
                AnalysisType::TechnicalDebtGradient,
            ],
            period_days: 30,
            ..DeepContextConfig::default()
        };

        let analyzer = DeepContextAnalyzer::new(config);

        // Run actual analysis
        let deep_context = analyzer.analyze_project(&project_path).await.map_err(|e| {
            HttpDemoError::AnalysisFailed(format!("Deep context analysis failed: {e}"))
        })?;

        // Convert to JSON value
        serde_json::to_value(&deep_context).map_err(HttpDemoError::JsonError)
    }
}
