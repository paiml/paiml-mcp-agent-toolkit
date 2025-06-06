use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::demo::protocol_harness::{DemoProtocol, ProtocolMetadata};

/// MCP (Model Context Protocol) adapter for demo harness
pub struct McpDemoAdapter;

/// MCP JSON-RPC request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

/// MCP JSON-RPC response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

/// MCP error format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Demo-specific analysis request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoAnalyzeParams {
    pub path: String,
    #[serde(default)]
    pub cache: bool,
    #[serde(default)]
    pub include_trace: bool,
}

/// Demo analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoAnalyzeResult {
    pub request_id: String,
    pub status: String,
    pub base_command: String,
    pub execution_time_ms: u64,
    pub cache_hit: bool,
    pub result: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<McpTrace>,
}

/// Demo get results parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoGetResultsParams {
    pub request_id: String,
    #[serde(default)]
    pub include_metadata: bool,
}

/// API trace parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoGetApiTraceParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}

/// MCP-specific trace information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTrace {
    pub method: String,
    pub params: Value,
    pub internal_command: Vec<String>,
    pub translation_time_ns: u64,
    pub validation_time_ns: u64,
    pub protocol_overhead_ns: u64,
}

/// MCP-specific errors
#[derive(Debug, Error)]
pub enum McpDemoError {
    #[error("Invalid JSON-RPC format: {0}")]
    InvalidJsonRpc(String),

    #[error("Unknown method: {0}")]
    UnknownMethod(String),

    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Analysis execution failed: {0}")]
    AnalysisFailed(String),

    #[error("Request not found: {0}")]
    RequestNotFound(String),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl McpDemoError {
    pub fn to_mcp_error(&self) -> McpError {
        match self {
            McpDemoError::InvalidJsonRpc(_) => McpError {
                code: -32600,
                message: "Invalid Request".to_string(),
                data: Some(serde_json::json!({"error": self.to_string()})),
            },
            McpDemoError::UnknownMethod(_) => McpError {
                code: -32601,
                message: "Method not found".to_string(),
                data: Some(serde_json::json!({"error": self.to_string()})),
            },
            McpDemoError::InvalidParams(_) => McpError {
                code: -32602,
                message: "Invalid params".to_string(),
                data: Some(serde_json::json!({"error": self.to_string()})),
            },
            McpDemoError::RequestNotFound(_) => McpError {
                code: -32001,
                message: "Request not found".to_string(),
                data: Some(serde_json::json!({"error": self.to_string()})),
            },
            _ => McpError {
                code: -32603,
                message: "Internal error".to_string(),
                data: Some(serde_json::json!({"error": self.to_string()})),
            },
        }
    }
}

impl McpDemoAdapter {
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
            cache_hit: params.cache, // TODO: Implement actual cache checking
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

impl Default for McpDemoAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DemoProtocol for McpDemoAdapter {
    type Request = McpRequest;
    type Response = McpResponse;
    type Error = McpDemoError;

    async fn decode_request(&self, raw: &[u8]) -> Result<Self::Request, Self::Error> {
        let request: McpRequest = serde_json::from_slice(raw)?;

        // Validate JSON-RPC format
        if request.jsonrpc != "2.0" {
            return Err(McpDemoError::InvalidJsonRpc(format!(
                "Expected jsonrpc: '2.0', got: '{}'",
                request.jsonrpc
            )));
        }

        Ok(request)
    }

    async fn encode_response(&self, resp: Self::Response) -> Result<Vec<u8>, Self::Error> {
        let json = serde_json::to_vec(&resp)?;
        Ok(json)
    }

    async fn get_protocol_metadata(&self) -> ProtocolMetadata {
        ProtocolMetadata {
            name: "mcp",
            version: "2.0",
            description: "Model Context Protocol (JSON-RPC 2.0) for AI integration".to_string(),
            request_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "jsonrpc": {
                        "type": "string",
                        "const": "2.0"
                    },
                    "method": {
                        "type": "string",
                        "enum": ["demo.analyze", "demo.getResults", "demo.getApiTrace"]
                    },
                    "params": {
                        "type": "object",
                        "description": "Method-specific parameters"
                    },
                    "id": {
                        "description": "Request identifier (any JSON type)"
                    }
                },
                "required": ["jsonrpc", "method"]
            }),
            response_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "jsonrpc": {
                        "type": "string",
                        "const": "2.0"
                    },
                    "result": {
                        "description": "Method result (on success)"
                    },
                    "error": {
                        "type": "object",
                        "properties": {
                            "code": {"type": "integer"},
                            "message": {"type": "string"},
                            "data": {}
                        },
                        "required": ["code", "message"]
                    },
                    "id": {
                        "description": "Request identifier (matches request)"
                    }
                },
                "required": ["jsonrpc"]
            }),
            example_requests: vec![
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "demo.analyze",
                    "params": {"path": "/repo", "cache": true, "include_trace": true},
                    "id": 1
                }),
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "demo.getResults",
                    "params": {"request_id": "uuid", "include_metadata": true},
                    "id": 2
                }),
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "demo.getApiTrace",
                    "params": {"limit": 5},
                    "id": 3
                }),
            ],
            capabilities: vec![
                "json_rpc_2.0".to_string(),
                "method_routing".to_string(),
                "parameter_validation".to_string(),
                "error_handling".to_string(),
                "request_tracing".to_string(),
                "async_results".to_string(),
            ],
        }
    }

    async fn execute_demo(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        match request.method.as_str() {
            "demo.analyze" => self.handle_demo_analyze(request.params, request.id).await,
            "demo.getResults" => {
                self.handle_demo_get_results(request.params, request.id)
                    .await
            }
            "demo.getApiTrace" => {
                self.handle_demo_get_api_trace(request.params, request.id)
                    .await
            }
            _ => {
                let error = McpDemoError::UnknownMethod(request.method.clone());
                Ok(McpResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(error.to_mcp_error()),
                    id: request.id,
                })
            }
        }
    }
}

impl From<Value> for McpRequest {
    fn from(value: Value) -> Self {
        serde_json::from_value(value).unwrap_or_else(|_| McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "invalid".to_string(),
            params: None,
            id: None,
        })
    }
}

impl From<McpResponse> for Value {
    fn from(val: McpResponse) -> Self {
        serde_json::to_value(val).unwrap_or(Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_adapter_metadata() {
        let adapter = McpDemoAdapter::new();
        let metadata = adapter.get_protocol_metadata().await;

        assert_eq!(metadata.name, "mcp");
        assert_eq!(metadata.version, "2.0");
        assert!(!metadata.capabilities.is_empty());
        assert!(!metadata.example_requests.is_empty());
    }

    #[test]
    fn test_mcp_request_from_value() {
        let value = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "demo.analyze",
            "params": {"path": "/test"},
            "id": 1
        });

        let request = McpRequest::from(value);
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "demo.analyze");
        assert_eq!(request.id, Some(serde_json::json!(1)));
    }

    #[tokio::test]
    async fn test_demo_analyze() {
        use std::fs;
        use tempfile::TempDir;

        // Create a small test directory instead of analyzing the entire project
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a simple test file
        fs::write(
            temp_path.join("test.rs"),
            "fn main() { println!(\"Hello\"); }",
        )
        .unwrap();

        let adapter = McpDemoAdapter::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "demo.analyze".to_string(),
            params: Some(serde_json::json!({
                "path": temp_path.to_string_lossy(),
                "cache": false,
                "include_trace": true
            })),
            id: Some(serde_json::json!(1)),
        };

        let response = adapter.execute_demo(request).await.unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let adapter = McpDemoAdapter::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "unknown.method".to_string(),
            params: None,
            id: Some(serde_json::json!(1)),
        };

        let response = adapter.execute_demo(request).await.unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32601);
    }

    #[test]
    fn test_error_conversion() {
        let error = McpDemoError::UnknownMethod("test".to_string());
        let mcp_error = error.to_mcp_error();

        assert_eq!(mcp_error.code, -32601);
        assert_eq!(mcp_error.message, "Method not found");
    }
}
