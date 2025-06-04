use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

use crate::demo::protocol_harness::{DemoProtocol, ProtocolMetadata};

/// HTTP/REST protocol adapter for demo harness
pub struct HttpDemoAdapter;

/// HTTP-specific request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Option<Value>,
    pub remote_addr: Option<String>,
}

/// HTTP-specific response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: HttpResponseBody,
    pub request_id: String,
}

/// HTTP response body variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HttpResponseBody {
    /// Immediate response with analysis results
    Analysis {
        protocol: String,
        base_command: String,
        request: HttpRequestInfo,
        response_time_ms: u64,
        cache_hit: bool,
        result: Value,
    },
    /// Asynchronous response with request tracking
    Async {
        request_id: String,
        status: String,
        message: String,
        poll_url: String,
    },
    /// Status check response
    Status {
        request_id: String,
        status: String,
        progress: Option<f32>,
        result: Option<Value>,
        error: Option<String>,
    },
    /// API introspection response
    Introspection {
        protocol: String,
        version: String,
        endpoints: Vec<HttpEndpoint>,
        schemas: HashMap<String, Value>,
        examples: HashMap<String, Value>,
    },
}

/// HTTP request information for introspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequestInfo {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
}

/// HTTP endpoint description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpEndpoint {
    pub method: String,
    pub path: String,
    pub description: String,
    pub parameters: Vec<HttpParameter>,
    pub responses: HashMap<String, String>,
}

/// HTTP parameter description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpParameter {
    pub name: String,
    pub location: String, // "query", "path", "header", "body"
    pub required: bool,
    pub param_type: String,
    pub description: String,
}

/// HTTP-specific errors
#[derive(Debug, Error)]
pub enum HttpDemoError {
    #[error("Invalid HTTP method: {0}")]
    InvalidMethod(String),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("Invalid path parameter: {0}")]
    InvalidPath(String),

    #[error("Analysis execution failed: {0}")]
    AnalysisFailed(String),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl HttpDemoAdapter {
    pub fn new() -> Self {
        Self
    }

    async fn handle_analyze_request(
        &self,
        request: &HttpRequest,
    ) -> Result<HttpResponseBody, HttpDemoError> {
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
                "paiml-mcp-agent-toolkit analyze context --format json --path {}",
                analysis_path
            ),
            request: HttpRequestInfo {
                method: request.method.clone(),
                path: request.path.clone(),
                query: request.query_params.clone(),
                headers: request.headers.clone(),
            },
            response_time_ms,
            cache_hit: false, // TODO: Implement proper cache checking
            result,
        })
    }

    async fn handle_status_request(
        &self,
        request: &HttpRequest,
    ) -> Result<HttpResponseBody, HttpDemoError> {
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
            HttpDemoError::AnalysisFailed(format!("Deep context analysis failed: {}", e))
        })?;

        // Convert to JSON value
        serde_json::to_value(&deep_context).map_err(HttpDemoError::JsonError)
    }
}

impl Default for HttpDemoAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DemoProtocol for HttpDemoAdapter {
    type Request = HttpRequest;
    type Response = HttpResponse;
    type Error = HttpDemoError;

    async fn decode_request(&self, raw: &[u8]) -> Result<Self::Request, Self::Error> {
        let value: Value = serde_json::from_slice(raw)?;

        let method = value
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET")
            .to_string();

        let path = value
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("/demo/analyze")
            .to_string();

        let query_params = value
            .get("query")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        let headers = value
            .get("headers")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_else(|| {
                [("Accept".to_string(), "application/json".to_string())]
                    .into_iter()
                    .collect()
            });

        let body = value.get("body").cloned();
        let remote_addr = value
            .get("remote_addr")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(HttpRequest {
            method,
            path,
            query_params,
            headers,
            body,
            remote_addr,
        })
    }

    async fn encode_response(&self, resp: Self::Response) -> Result<Vec<u8>, Self::Error> {
        let json = serde_json::to_vec_pretty(&resp)?;
        Ok(json)
    }

    async fn get_protocol_metadata(&self) -> ProtocolMetadata {
        ProtocolMetadata {
            name: "http",
            version: "1.1",
            description: "HTTP/REST protocol for web-based access".to_string(),
            request_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "method": {
                        "type": "string",
                        "enum": ["GET", "POST"],
                        "default": "GET"
                    },
                    "path": {
                        "type": "string",
                        "description": "Request path",
                        "default": "/demo/analyze"
                    },
                    "query": {
                        "type": "object",
                        "description": "Query parameters"
                    },
                    "headers": {
                        "type": "object",
                        "description": "HTTP headers"
                    },
                    "body": {
                        "description": "Request body (for POST requests)"
                    }
                }
            }),
            response_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "status": {
                        "type": "integer",
                        "description": "HTTP status code"
                    },
                    "headers": {
                        "type": "object",
                        "description": "Response headers"
                    },
                    "body": {
                        "description": "Response body (varies by endpoint)"
                    }
                },
                "required": ["status", "body"]
            }),
            example_requests: vec![
                serde_json::json!({
                    "method": "GET",
                    "path": "/demo/analyze",
                    "query": {"path": "/repo"},
                    "headers": {"Accept": "application/json"}
                }),
                serde_json::json!({
                    "method": "GET",
                    "path": "/demo/api"
                }),
            ],
            capabilities: vec![
                "rest_api".to_string(),
                "async_requests".to_string(),
                "status_polling".to_string(),
                "api_introspection".to_string(),
                "json_responses".to_string(),
            ],
        }
    }

    async fn execute_demo(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let request_id = Uuid::new_v4().to_string();

        let body = match request.path.as_str() {
            path if path.starts_with("/demo/analyze") => {
                self.handle_analyze_request(&request).await?
            }
            path if path.starts_with("/demo/status/") => {
                self.handle_status_request(&request).await?
            }
            path if path.starts_with("/demo/results/") => {
                self.handle_results_request(&request).await?
            }
            "/demo/api" => self.handle_api_introspection().await?,
            _ => {
                return Err(HttpDemoError::InvalidPath(format!(
                    "Unknown endpoint: {}",
                    request.path
                )));
            }
        };

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("X-Request-ID".to_string(), request_id.clone());
        headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());

        Ok(HttpResponse {
            status: 200,
            headers,
            body,
            request_id,
        })
    }
}

impl From<Value> for HttpRequest {
    fn from(value: Value) -> Self {
        let method = value
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET")
            .to_string();

        let path = value
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("/demo/analyze")
            .to_string();

        let query_params = value
            .get("query")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        let headers = value
            .get("headers")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_else(|| {
                [("Accept".to_string(), "application/json".to_string())]
                    .into_iter()
                    .collect()
            });

        HttpRequest {
            method,
            path,
            query_params,
            headers,
            body: value.get("body").cloned(),
            remote_addr: value
                .get("remote_addr")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }
    }
}

impl From<HttpResponse> for Value {
    fn from(val: HttpResponse) -> Self {
        serde_json::to_value(val).unwrap_or(Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_adapter_metadata() {
        let adapter = HttpDemoAdapter::new();
        let metadata = adapter.get_protocol_metadata().await;

        assert_eq!(metadata.name, "http");
        assert_eq!(metadata.version, "1.1");
        assert!(!metadata.capabilities.is_empty());
        assert!(!metadata.example_requests.is_empty());
    }

    #[test]
    fn test_http_request_from_value() {
        let value = serde_json::json!({
            "method": "GET",
            "path": "/demo/analyze",
            "query": {"path": "/test"},
            "headers": {"Accept": "application/json"}
        });

        let request = HttpRequest::from(value);
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/demo/analyze");
        assert_eq!(request.query_params.get("path"), Some(&"/test".to_string()));
    }

    #[tokio::test]
    async fn test_api_introspection() {
        let adapter = HttpDemoAdapter::new();
        let body = adapter.handle_api_introspection().await.unwrap();

        match body {
            HttpResponseBody::Introspection { endpoints, .. } => {
                assert!(!endpoints.is_empty());
                assert!(endpoints.iter().any(|e| e.path == "/demo/analyze"));
                assert!(endpoints.iter().any(|e| e.path == "/demo/api"));
            }
            _ => panic!("Expected introspection response"),
        }
    }

    #[tokio::test]
    async fn test_context_analysis() {
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

        let adapter = HttpDemoAdapter::new();
        let result = adapter
            .execute_context_analysis(&temp_path.to_string_lossy())
            .await
            .unwrap();

        // Deep context analysis returns specific fields from the DeepContext struct
        assert!(result.get("metadata").is_some());
        assert!(result.get("quality_scorecard").is_some());
        assert!(result.get("analyses").is_some());
        assert!(result.get("file_tree").is_some());
    }
}
