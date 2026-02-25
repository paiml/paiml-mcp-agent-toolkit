//! Unified Protocol Design implementation per SPECIFICATION.md Section 3
//!
//! This module provides the protocol abstraction layer that unifies all
//! interfaces (CLI, MCP, HTTP) through a single operation model.

#![cfg_attr(coverage_nightly, coverage(off))]
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

// --- Submodule includes ---

// Parameter structures for each operation
include!("protocol_params.rs");

// RequestContext impl, JSON-RPC/HTTP helper structs, conversions
include!("protocol_context.rs");

// Tests (property tests + coverage tests)
include!("protocol_tests.rs");
