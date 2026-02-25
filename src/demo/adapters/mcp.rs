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

// Handler methods (handle_demo_analyze, handle_demo_get_results,
// handle_demo_get_api_trace, execute_context_analysis)
include!("mcp_handlers.rs");

// DemoProtocol trait impl, McpDemoError::to_mcp_error, Default, From conversions
include!("mcp_protocol.rs");

// Unit tests and property tests
include!("mcp_tests.rs");
