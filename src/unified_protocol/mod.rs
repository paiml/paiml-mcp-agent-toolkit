#![cfg_attr(coverage_nightly, coverage(off))]
//! Unified protocol implementation for PMAT.
//!
//! This module provides a protocol-agnostic service layer that supports multiple
//! transport mechanisms (HTTP, MCP, CLI) through a unified interface. This design
//! follows the Toyota Way principle of standardization and flexibility.
//!
//! # Architecture
//!
//! The unified protocol consists of:
//! - **Service**: Core protocol-agnostic business logic
//! - **Adapters**: Transport-specific implementations (HTTP, MCP)
//! - **Error**: Unified error handling across all transports
//!
//! # Example
//!
//! ```no_run
//! use pmat::unified_protocol::{UnifiedRequest, UnifiedResponse, Protocol};
//! use axum::http::{Method, StatusCode};
//!
//! # fn example() {
//! // Create a unified request
//! let request = UnifiedRequest::new(Method::GET, "/analyze/complexity".to_string())
//!     .with_header("content-type", "application/json")
//!     .with_extension("protocol", Protocol::Http);
//!
//! // Create a unified response
//! let response = UnifiedResponse::ok()
//!     .with_json(&serde_json::json!({
//!         "status": "success",
//!         "files": []
//!     }))
//!     .expect("internal error");
//! # }
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

pub mod adapters;
pub mod error;
pub mod service;
// pub mod test_harness; // TRACKED: Fix Future type issues

// -----------------------------------------------------------------------------
// Core types
// -----------------------------------------------------------------------------

/// Core unified request abstraction that can represent any protocol
#[derive(Debug)]
pub struct UnifiedRequest {
    pub method: Method,
    pub path: String,
    pub headers: HeaderMap,
    pub body: Body,
    pub extensions: HashMap<String, Value>,
    pub trace_id: Uuid,
}

/// Core unified response abstraction
#[derive(Debug)]
pub struct UnifiedResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Body,
    pub trace_id: Uuid,
}

/// Protocol enumeration for type-safe protocol handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    Mcp,
    Http,
    Cli,
    WebSocket,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Mcp => write!(f, "mcp"),
            Protocol::Http => write!(f, "http"),
            Protocol::Cli => write!(f, "cli"),
            Protocol::WebSocket => write!(f, "websocket"),
        }
    }
}

/// Core trait for protocol adapters
#[async_trait]
pub trait ProtocolAdapter: Send + Sync + 'static {
    type Input: Send + 'static;
    type Output: Send + 'static;

    /// Convert protocol-specific input to unified request
    async fn decode(&self, input: Self::Input) -> Result<UnifiedRequest, ProtocolError>;

    /// Convert unified response to protocol-specific output
    async fn encode(&self, response: UnifiedResponse) -> Result<Self::Output, ProtocolError>;

    /// Get the protocol type this adapter handles
    fn protocol(&self) -> Protocol;
}

/// Protocol-specific errors
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Failed to decode request: {0}")]
    DecodeError(String),

    #[error("Failed to encode response: {0}")]
    EncodeError(String),

    #[error("Protocol not supported: {0}")]
    UnsupportedProtocol(String),

    #[error("Invalid request format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    HttpError(String),
}

/// Context information for MCP requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContext {
    pub id: Option<Value>,
    pub method: String,
}

/// Context information for CLI requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliContext {
    pub command: String,
    pub args: Vec<String>,
}

/// Context information for HTTP requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpContext {
    pub remote_addr: Option<String>,
    pub user_agent: Option<String>,
}

// -----------------------------------------------------------------------------
// Implementation includes (semantic split for maintainability)
// -----------------------------------------------------------------------------

include!("protocol_request.rs");
include!("protocol_response.rs");
include!("protocol_registry.rs");
include!("protocol_tests.rs");
include!("protocol_property_tests.rs");
