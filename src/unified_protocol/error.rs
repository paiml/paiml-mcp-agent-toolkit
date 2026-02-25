#![cfg_attr(coverage_nightly, coverage(off))]
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

use super::{Protocol, UnifiedResponse};

/// Unified application error type with protocol-aware serialization
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Authentication required")]
    Unauthorized,

    #[error("Access forbidden: {0}")]
    Forbidden(String),

    #[error("Request payload too large")]
    PayloadTooLarge,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Service temporarily unavailable")]
    ServiceUnavailable,

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Analysis error: {0}")]
    Analysis(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Protocol error: {0}")]
    Protocol(#[from] super::ProtocolError),
}

/// MCP-specific error structure
#[derive(Debug, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// HTTP-specific error response
#[derive(Debug, Serialize, Deserialize)]
pub struct HttpErrorResponse {
    pub error: String,
    pub error_type: String,
    pub timestamp: String,
}

/// CLI-specific error response
#[derive(Debug, Serialize, Deserialize)]
pub struct CliErrorResponse {
    pub message: String,
    pub error_type: String,
    pub exit_code: i32,
}

/// Extract the current protocol from request context
/// This would typically be set by middleware or the protocol adapter
fn extract_protocol_from_context() -> Option<Protocol> {
    // In a real implementation, this would extract from request extensions
    // For now, we'll use a thread-local or similar mechanism
    CURRENT_PROTOCOL.with(std::cell::Cell::get)
}

thread_local! {
    static CURRENT_PROTOCOL: std::cell::Cell<Option<Protocol>> = const { std::cell::Cell::new(None) };
}

/// Set the current protocol context (used by middleware)
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::unified_protocol::{Protocol, error::{set_protocol_context, clear_protocol_context}};
///
/// set_protocol_context(Protocol::Http);
/// // Protocol context is now set to HTTP
/// clear_protocol_context();
/// // Protocol context is now cleared
/// ```
pub fn set_protocol_context(protocol: Protocol) {
    CURRENT_PROTOCOL.with(|p| p.set(Some(protocol)));
}

/// Clear the protocol context
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::unified_protocol::{Protocol, error::{set_protocol_context, clear_protocol_context}};
///
/// set_protocol_context(Protocol::Mcp);
/// clear_protocol_context();
/// // Protocol context is now None
/// ```
pub fn clear_protocol_context() {
    CURRENT_PROTOCOL.with(|p| p.set(None));
}

// AppError method implementations and IntoResponse trait impl
include!("error_methods.rs");

// Unit tests and property tests
include!("error_tests.rs");
