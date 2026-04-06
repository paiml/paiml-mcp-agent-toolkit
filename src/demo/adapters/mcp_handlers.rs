// MCP demo adapter handler methods
// Included from mcp.rs - shares parent module scope

impl McpDemoAdapter {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }

    async fn handle_demo_analyze(
        &self,
        params: Option<Value>,
        id: Option<Value>,
    ) -> Result<McpResponse, McpDemoError> {
        let start_time = std::time::Instant::now();

        let params: DemoAnalyzeParams = match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                McpDemoError::InvalidParams(format!("Failed to parse demo.analyze params: {e}"))
            })?,
            None => {
                return Err(McpDemoError::InvalidParams(
                    "Missing params for demo.analyze".to_string(),
                ))
            }
        };

        let request_id = Uuid::new_v4().to_string();

        // Execute context analysis
        let result = self.execute_context_analysis(&params.path).await?;

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // Create trace if requested
        let trace = if params.include_trace {
            Some(McpTrace {
                method: "demo.analyze".to_string(),
                params: serde_json::to_value(&params)?,
                internal_command: vec![
                    "paiml-mcp-agent-toolkit".to_string(),
                    "analyze".to_string(),
                    "context".to_string(),
                    "--format".to_string(),
                    "json".to_string(),
                    "--path".to_string(),
                    params.path.clone(),
                ],
                translation_time_ns: 1_500, // Time to translate JSON-RPC to internal format
                validation_time_ns: 800,    // Time to validate parameters
                protocol_overhead_ns: 2_300, // Total protocol overhead
            })
        } else {
            None
        };

        let analyze_result = DemoAnalyzeResult {
            request_id: request_id.clone(),
            status: "completed".to_string(),
            base_command: format!(
                "paiml-mcp-agent-toolkit analyze context --format json --path {}",
                params.path
            ),
            execution_time_ms,
            cache_hit: params.cache, // TRACKED: Implement actual cache checking
            result,
            trace,
        };

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::to_value(analyze_result)?),
            error: None,
            id,
        })
    }

    async fn handle_demo_get_results(
        &self,
        params: Option<Value>,
        id: Option<Value>,
    ) -> Result<McpResponse, McpDemoError> {
        let params: DemoGetResultsParams = match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                McpDemoError::InvalidParams(format!("Failed to parse demo.getResults params: {e}"))
            })?,
            None => {
                return Err(McpDemoError::InvalidParams(
                    "Missing params for demo.getResults".to_string(),
                ))
            }
        };

        // For demo purposes, return cached result
        let cached_result = serde_json::json!({
            "request_id": params.request_id,
            "status": "completed",
            "cached_at": chrono::Utc::now().to_rfc3339(),
            "result": {
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
            }
        });

        if params.include_metadata {
            let mut result = cached_result;
            result["metadata"] = serde_json::json!({
                "cache_hit": true,
                "retrieval_time_ms": 5,
                "original_execution_time_ms": 2847,
                "cache_age_seconds": 180
            });

            Ok(McpResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result),
                error: None,
                id,
            })
        } else {
            Ok(McpResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(cached_result),
                error: None,
                id,
            })
        }
    }

    async fn handle_demo_get_api_trace(
        &self,
        params: Option<Value>,
        id: Option<Value>,
    ) -> Result<McpResponse, McpDemoError> {
        let params: DemoGetApiTraceParams = match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                McpDemoError::InvalidParams(format!("Failed to parse demo.getApiTrace params: {e}"))
            })?,
            None => DemoGetApiTraceParams {
                request_id: None,
                limit: Some(10),
            },
        };

        let traces = if let Some(ref request_id) = params.request_id {
            // Return specific trace
            vec![serde_json::json!({
                "request_id": request_id,
                "method": "demo.analyze",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "protocol": "mcp",
                "translation": {
                    "from": {"jsonrpc": "2.0", "method": "demo.analyze"},
                    "to": ["paiml-mcp-agent-toolkit", "analyze", "context"],
                    "translation_time_ns": 1500
                },
                "execution": {
                    "command": "paiml-mcp-agent-toolkit analyze context --format json --path /repo",
                    "execution_time_ms": 2847,
                    "cache_hit": false
                },
                "response": {
                    "encoding_time_ns": 850,
                    "size_bytes": 15420
                }
            })]
        } else {
            // Return recent traces (limited)
            let limit = params.limit.unwrap_or(10).min(100); // Cap at 100
            (0..limit)
                .map(|i| serde_json::json!({
                    "request_id": format!("trace_{}", i),
                    "method": if i % 3 == 0 { "demo.analyze" } else if i % 3 == 1 { "demo.getResults" } else { "demo.getApiTrace" },
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "execution_time_ms": 1000 + (i * 100) as u64
                }))
                .collect()
        };

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({
                "traces": traces,
                "total_count": traces.len(),
                "filtered": params.request_id.is_some()
            })),
            error: None,
            id,
        })
    }

    async fn execute_context_analysis(&self, path: &str) -> Result<Value, McpDemoError> {
        use crate::services::deep_context::{AnalysisType, DeepContextAnalyzer, DeepContextConfig};
        use std::path::PathBuf;

        // Parse path
        let project_path = PathBuf::from(path);

        // Validate path exists
        if !project_path.exists() {
            return Err(McpDemoError::AnalysisFailed(format!(
                "Path does not exist: {path}"
            )));
        }

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
            McpDemoError::AnalysisFailed(format!("Deep context analysis failed: {e}"))
        })?;

        // Add MCP metadata
        let mut result = serde_json::to_value(&deep_context)?;
        if let Some(obj) = result.as_object_mut() {
            obj.insert(
                "mcp_metadata".to_string(),
                serde_json::json!({
                    "protocol_version": "2.0",
                    "method_mapping": {
                        "analyze": "demo.analyze",
                        "get_results": "demo.getResults",
                        "get_trace": "demo.getApiTrace"
                    },
                    "response_format": "json-rpc"
                }),
            );
        }

        Ok(result)
    }
}
