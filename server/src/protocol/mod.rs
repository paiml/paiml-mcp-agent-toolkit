//! Unified Protocol Design implementation per SPECIFICATION.md Section 3
//!
//! This module provides the protocol abstraction layer that unifies all
//! interfaces (CLI, MCP, HTTP) through a single operation model.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

pub mod adapters;
pub mod operations;

/// Single protocol implementation used by all interfaces
#[async_trait]
pub trait ProtocolAdapter: Send + Sync {
    type Request: for<'de> Deserialize<'de>;
    type Response: Serialize;

    fn decode(&self, raw: &[u8]) -> Result<UnifiedRequest, ProtocolError>;
    fn encode(&self, response: UnifiedResponse) -> Result<Vec<u8>, ProtocolError>;

    async fn handle(&self, request: Self::Request) -> Self::Response;
}

/// Unified request for all protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRequest {
    pub operation: Operation,
    pub params: Value,
    pub context: RequestContext,
}

/// Unified response for all protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedResponse {
    pub result: Option<Value>,
    pub error: Option<ErrorInfo>,
    pub metadata: ResponseMetadata,
}

/// Request context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub request_id: String,
    pub trace_id: Uuid,
    pub protocol: String,
    pub timestamp: i64,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub request_id: String,
    pub duration_ms: u64,
    pub version: String,
}

/// Error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: i32,
    pub message: String,
    pub details: Option<Value>,
}

/// Protocol errors
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Unknown method: {0}")]
    UnknownMethod(String),

    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// All operations go through this enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Operation {
    // Analysis
    AnalyzeComplexity(ComplexityParams),
    AnalyzeSatd(SatdParams),
    AnalyzeDeadCode(DeadCodeParams),
    GenerateContext(ContextParams),

    // Quality
    QualityGate(QualityGateParams),
    QualityProxy(QualityProxyParams),

    // Refactoring
    RefactorStart(RefactorStartParams),
    RefactorNext(RefactorNextParams),
    RefactorStop(RefactorStopParams),

    // Scaffolding
    ScaffoldProject(ProjectParams),
    ScaffoldAgent(AgentParams),

    // PDMT
    PdmtTodos(PdmtParams),
}

// Parameter structures for each operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityParams {
    pub file_path: Option<String>,
    pub max_cyclomatic: Option<u32>,
    pub max_cognitive: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatdParams {
    pub file_path: Option<String>,
    pub strict: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeParams {
    pub file_path: Option<String>,
    pub include_tests: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextParams {
    pub file_path: Option<String>,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateParams {
    pub file_path: Option<String>,
    pub fail_on_violation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityProxyParams {
    pub file_path: String,
    pub content: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorStartParams {
    pub file_path: String,
    pub target_complexity: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorNextParams {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorStopParams {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectParams {
    pub name: String,
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentParams {
    pub name: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdmtParams {
    pub requirement: String,
    pub granularity: String,
    pub seed: Option<u64>,
}

impl RequestContext {
    pub fn new(protocol: &str) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            trace_id: Uuid::new_v4(),
            protocol: protocol.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn from_json_rpc(request: &JsonRpcRequest) -> Self {
        Self {
            request_id: request.id.to_string(),
            trace_id: Uuid::new_v4(),
            protocol: "json-rpc".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn from_http(request: &HttpRequest) -> Self {
        Self {
            request_id: request
                .headers
                .get("x-request-id")
                .cloned()
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            trace_id: Uuid::new_v4(),
            protocol: "http".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

// Helper structures for protocol-specific requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
    pub id: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Value,
}

impl From<ErrorInfo> for JsonRpcError {
    fn from(error: ErrorInfo) -> Self {
        Self {
            code: error.code,
            message: error.message,
            data: error.details,
        }
    }
}
