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

impl AppError {
    /// Get the appropriate HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Validation(_) | AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            AppError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            AppError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Internal(_)
            | AppError::Template(_)
            | AppError::Analysis(_)
            | AppError::Io(_)
            | AppError::Json(_)
            | AppError::Protocol(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get the MCP error code for this error
    pub fn mcp_error_code(&self) -> i32 {
        match self {
            AppError::NotFound(_) => -32001,
            AppError::Validation(_) | AppError::BadRequest(_) => -32602,
            AppError::Unauthorized => -32600,
            AppError::Forbidden(_) => -32600,
            AppError::PayloadTooLarge => -32600,
            AppError::RateLimitExceeded => -32000,
            AppError::ServiceUnavailable => -32000,
            AppError::Internal(_)
            | AppError::Template(_)
            | AppError::Analysis(_)
            | AppError::Io(_)
            | AppError::Json(_)
            | AppError::Protocol(_) => -32603,
        }
    }

    /// Get a categorized error type string
    pub fn error_type(&self) -> &'static str {
        match self {
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::Unauthorized => "UNAUTHORIZED",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::PayloadTooLarge => "PAYLOAD_TOO_LARGE",
            AppError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            AppError::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::Template(_) => "TEMPLATE_ERROR",
            AppError::Analysis(_) => "ANALYSIS_ERROR",
            AppError::Io(_) => "IO_ERROR",
            AppError::Json(_) => "JSON_ERROR",
            AppError::Protocol(_) => "PROTOCOL_ERROR",
        }
    }

    /// Convert to protocol-specific response
    pub fn to_protocol_response(
        &self,
        protocol: Protocol,
    ) -> Result<UnifiedResponse, serde_json::Error> {
        match protocol {
            Protocol::Mcp => self.to_mcp_response(),
            Protocol::Http => self.to_http_response(),
            Protocol::Cli => self.to_cli_response(),
            Protocol::WebSocket => self.to_http_response(), // WebSocket uses HTTP-like responses
        }
    }

    fn to_mcp_response(&self) -> Result<UnifiedResponse, serde_json::Error> {
        let mcp_error = McpError {
            code: self.mcp_error_code(),
            message: self.to_string(),
            data: Some(json!({
                "type": self.error_type(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
            })),
        };

        UnifiedResponse::new(StatusCode::OK) // MCP always returns 200 for JSON-RPC
            .with_json(&json!({
                "jsonrpc": "2.0",
                "error": mcp_error,
                "id": null
            }))
    }

    fn to_http_response(&self) -> Result<UnifiedResponse, serde_json::Error> {
        let error_response = HttpErrorResponse {
            error: self.to_string(),
            error_type: self.error_type().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        UnifiedResponse::new(self.status_code()).with_json(&error_response)
    }

    fn to_cli_response(&self) -> Result<UnifiedResponse, serde_json::Error> {
        let cli_error = CliErrorResponse {
            message: self.to_string(),
            error_type: self.error_type().to_string(),
            exit_code: match self {
                AppError::NotFound(_) => 2,
                AppError::Validation(_) | AppError::BadRequest(_) => 1,
                AppError::Unauthorized | AppError::Forbidden(_) => 3,
                _ => 1,
            },
        };

        UnifiedResponse::new(StatusCode::OK) // CLI doesn't use HTTP status codes
            .with_json(&cli_error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Default to HTTP protocol if no context is available
        let protocol = extract_protocol_from_context().unwrap_or(Protocol::Http);

        match self.to_protocol_response(protocol) {
            Ok(unified_response) => unified_response.into_response(),
            Err(_) => {
                // Fallback error response
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Failed to serialize error response",
                        "original_error": self.to_string()
                    })),
                )
                    .into_response()
            }
        }
    }
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
    CURRENT_PROTOCOL.with(|p| p.get())
}

thread_local! {
    static CURRENT_PROTOCOL: std::cell::Cell<Option<Protocol>> = const { std::cell::Cell::new(None) };
}

/// Set the current protocol context (used by middleware)
pub fn set_protocol_context(protocol: Protocol) {
    CURRENT_PROTOCOL.with(|p| p.set(Some(protocol)));
}

/// Clear the protocol context
pub fn clear_protocol_context() {
    CURRENT_PROTOCOL.with(|p| p.set(None));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_status_codes() {
        assert_eq!(
            AppError::NotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::Validation("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::Unauthorized.status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::Internal(anyhow::anyhow!("test")).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_mcp_error_codes() {
        assert_eq!(
            AppError::NotFound("test".to_string()).mcp_error_code(),
            -32001
        );
        assert_eq!(
            AppError::Validation("test".to_string()).mcp_error_code(),
            -32602
        );
        assert_eq!(
            AppError::Internal(anyhow::anyhow!("test")).mcp_error_code(),
            -32603
        );
    }

    #[test]
    fn test_error_types() {
        assert_eq!(
            AppError::NotFound("test".to_string()).error_type(),
            "NOT_FOUND"
        );
        assert_eq!(
            AppError::Validation("test".to_string()).error_type(),
            "VALIDATION_ERROR"
        );
        assert_eq!(
            AppError::Template("test".to_string()).error_type(),
            "TEMPLATE_ERROR"
        );
    }

    #[tokio::test]
    async fn test_protocol_context() {
        set_protocol_context(Protocol::Mcp);
        assert_eq!(extract_protocol_from_context(), Some(Protocol::Mcp));

        clear_protocol_context();
        assert_eq!(extract_protocol_from_context(), None);
    }

    #[tokio::test]
    async fn test_error_to_protocol_response() {
        let error = AppError::NotFound("test resource".to_string());

        // Test MCP response
        let mcp_response = error.to_protocol_response(Protocol::Mcp).unwrap();
        assert_eq!(mcp_response.status, StatusCode::OK);

        // Test HTTP response
        let http_response = error.to_protocol_response(Protocol::Http).unwrap();
        assert_eq!(http_response.status, StatusCode::NOT_FOUND);

        // Test CLI response
        let cli_response = error.to_protocol_response(Protocol::Cli).unwrap();
        assert_eq!(cli_response.status, StatusCode::OK);
    }
}
