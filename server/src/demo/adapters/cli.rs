use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::demo::protocol_harness::{DemoProtocol, ProtocolMetadata};

/// CLI protocol adapter for demo harness
pub struct CliDemoAdapter;

/// CLI-specific request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliRequest {
    pub path: String,
    pub command: String,
    pub args: Vec<String>,
    pub show_api: bool,
}

/// CLI-specific response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliResponse {
    pub command: String,
    pub execution_time_ms: u64,
    pub output_format: String,
    pub cache_key: String,
    pub result: Value,
    pub api_trace: Option<CliApiTrace>,
}

/// CLI API trace information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliApiTrace {
    pub command_line: String,
    pub working_directory: String,
    pub environment: Vec<(String, String)>,
    pub exit_code: i32,
    pub stdout_lines: usize,
    pub stderr_lines: usize,
}

/// CLI-specific errors
#[derive(Debug, Error)]
pub enum CliDemoError {
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl CliDemoAdapter {
    pub fn new() -> Self {
        Self
    }

    async fn execute_context_analysis(&self, path: &str) -> Result<Value, CliDemoError> {
        use crate::services::deep_context::{AnalysisType, DeepContextAnalyzer, DeepContextConfig};
        use std::path::PathBuf;

        let path_buf = PathBuf::from(path);

        // Validate path exists
        if !path_buf.exists() {
            return Err(CliDemoError::InvalidPath(format!(
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
        let deep_context = analyzer.analyze_project(&path_buf).await.map_err(|e| {
            CliDemoError::ExecutionFailed(format!("Deep context analysis failed: {e}"))
        })?;

        // Add CLI metadata
        let mut result = serde_json::to_value(&deep_context)?;
        if let Some(obj) = result.as_object_mut() {
            obj.insert(
                "cli_metadata".to_string(),
                serde_json::json!({
                    "command": "paiml-mcp-agent-toolkit analyze context --format json",
                    "version": env!("CARGO_PKG_VERSION"),
                    "protocol": "cli"
                }),
            );
        }

        Ok(result)
    }

    fn generate_cache_key(&self, path: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);

        format!("sha256:{:x}", hasher.finish())
    }

    fn create_api_trace(
        &self,
        request: &CliRequest,
        _execution_time_ms: u64,
        result: &Value,
    ) -> CliApiTrace {
        let command_line = format!(
            "paiml-mcp-agent-toolkit {} {}",
            request.command,
            request.args.join(" ")
        );

        // Count lines in result for trace info
        let result_str = serde_json::to_string_pretty(result).unwrap_or_default();
        let stdout_lines = result_str.lines().count();

        CliApiTrace {
            command_line,
            working_directory: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            environment: vec![
                (
                    "RUST_LOG".to_string(),
                    std::env::var("RUST_LOG").unwrap_or_default(),
                ),
                (
                    "PATH".to_string(),
                    std::env::var("PATH").unwrap_or_default(),
                ),
            ],
            exit_code: 0,
            stdout_lines,
            stderr_lines: 0,
        }
    }
}

impl Default for CliDemoAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DemoProtocol for CliDemoAdapter {
    type Request = CliRequest;
    type Response = CliResponse;
    type Error = CliDemoError;

    async fn decode_request(&self, raw: &[u8]) -> Result<Self::Request, Self::Error> {
        let value: Value = serde_json::from_slice(raw)?;

        // Extract CLI request from JSON
        let path = value
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".")
            .to_string();

        let show_api = value
            .get("show_api")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(CliRequest {
            path,
            command: "analyze context".to_string(),
            args: vec!["--format".to_string(), "json".to_string()],
            show_api,
        })
    }

    async fn encode_response(&self, resp: Self::Response) -> Result<Vec<u8>, Self::Error> {
        let json = serde_json::to_vec_pretty(&resp)?;
        Ok(json)
    }

    async fn get_protocol_metadata(&self) -> ProtocolMetadata {
        ProtocolMetadata {
            name: "cli",
            version: "1.0.0",
            description: "Command-line interface protocol for direct execution".to_string(),
            request_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to analyze",
                        "default": "."
                    },
                    "show_api": {
                        "type": "boolean",
                        "description": "Show API introspection information",
                        "default": false
                    }
                },
                "required": []
            }),
            response_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Executed command"
                    },
                    "execution_time_ms": {
                        "type": "integer",
                        "description": "Execution time in milliseconds"
                    },
                    "output_format": {
                        "type": "string",
                        "description": "Output format used"
                    },
                    "cache_key": {
                        "type": "string",
                        "description": "Cache key for this result"
                    },
                    "result": {
                        "type": "object",
                        "description": "Analysis result data"
                    },
                    "api_trace": {
                        "type": "object",
                        "description": "API execution trace (optional)"
                    }
                },
                "required": ["command", "execution_time_ms", "result"]
            }),
            example_requests: vec![
                serde_json::json!({
                    "path": "/path/to/repo",
                    "show_api": false
                }),
                serde_json::json!({
                    "path": ".",
                    "show_api": true
                }),
            ],
            capabilities: vec![
                "direct_execution".to_string(),
                "filesystem_access".to_string(),
                "binary_invocation".to_string(),
                "api_introspection".to_string(),
            ],
        }
    }

    async fn execute_demo(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let start_time = std::time::Instant::now();

        // Execute the analysis
        let result = self.execute_context_analysis(&request.path).await?;

        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        let cache_key = self.generate_cache_key(&request.path);

        // Create API trace if requested
        let api_trace = if request.show_api {
            Some(self.create_api_trace(&request, execution_time_ms, &result))
        } else {
            None
        };

        Ok(CliResponse {
            command: format!("{} {}", request.command, request.args.join(" ")),
            execution_time_ms,
            output_format: "JSON".to_string(),
            cache_key,
            result,
            api_trace,
        })
    }
}

impl From<Value> for CliRequest {
    fn from(value: Value) -> Self {
        let path = value
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".")
            .to_string();

        let show_api = value
            .get("show_api")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        CliRequest {
            path,
            command: "analyze context".to_string(),
            args: vec!["--format".to_string(), "json".to_string()],
            show_api,
        }
    }
}

impl From<CliResponse> for Value {
    fn from(val: CliResponse) -> Self {
        serde_json::to_value(val).unwrap_or(Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cli_adapter_metadata() {
        let adapter = CliDemoAdapter::new();
        let metadata = adapter.get_protocol_metadata().await;

        assert_eq!(metadata.name, "cli");
        assert_eq!(metadata.version, "1.0.0");
        assert!(!metadata.capabilities.is_empty());
        assert!(!metadata.example_requests.is_empty());
    }

    #[test]
    fn test_cli_request_from_value() {
        let value = serde_json::json!({
            "path": "/test/path",
            "show_api": true
        });

        let request = CliRequest::from(value);
        assert_eq!(request.path, "/test/path");
        assert!(request.show_api);
        assert_eq!(request.command, "analyze context");
    }

    #[test]
    fn test_cache_key_generation() {
        let adapter = CliDemoAdapter::new();
        let key1 = adapter.generate_cache_key("/test/path");
        let key2 = adapter.generate_cache_key("/test/path");

        // Keys should be different due to timestamp (using nanosecond resolution)
        assert_ne!(key1, key2);
        assert!(key1.starts_with("sha256:"));
        assert!(key2.starts_with("sha256:"));
    }

    #[test]
    fn test_api_trace_creation() {
        let adapter = CliDemoAdapter::new();
        let request = CliRequest {
            path: "/test".to_string(),
            command: "analyze context".to_string(),
            args: vec!["--format".to_string(), "json".to_string()],
            show_api: true,
        };

        let result = serde_json::json!({"test": "data"});
        let trace = adapter.create_api_trace(&request, 1000, &result);

        assert!(trace.command_line.contains("analyze context"));
        assert_eq!(trace.exit_code, 0);
        assert!(trace.stdout_lines > 0);
    }
}
