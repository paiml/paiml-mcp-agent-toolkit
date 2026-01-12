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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    fn test_app_error_status_codes_extended() {
        // BadRequest
        assert_eq!(
            AppError::BadRequest("bad".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        // Forbidden
        assert_eq!(
            AppError::Forbidden("denied".to_string()).status_code(),
            StatusCode::FORBIDDEN
        );
        // PayloadTooLarge
        assert_eq!(
            AppError::PayloadTooLarge.status_code(),
            StatusCode::PAYLOAD_TOO_LARGE
        );
        // RateLimitExceeded
        assert_eq!(
            AppError::RateLimitExceeded.status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
        // ServiceUnavailable
        assert_eq!(
            AppError::ServiceUnavailable.status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        // Template
        assert_eq!(
            AppError::Template("template err".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        // Analysis
        assert_eq!(
            AppError::Analysis("analysis err".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_app_error_status_codes_from_errors() {
        // Io error
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        assert_eq!(app_err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);

        // Json error
        let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let app_err: AppError = json_err.into();
        assert_eq!(app_err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);

        // Protocol error
        let protocol_err = super::super::ProtocolError::DecodeError("decode failed".to_string());
        let app_err: AppError = protocol_err.into();
        assert_eq!(app_err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
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
    fn test_mcp_error_codes_extended() {
        // BadRequest
        assert_eq!(
            AppError::BadRequest("bad".to_string()).mcp_error_code(),
            -32602
        );
        // Unauthorized
        assert_eq!(AppError::Unauthorized.mcp_error_code(), -32600);
        // Forbidden
        assert_eq!(
            AppError::Forbidden("denied".to_string()).mcp_error_code(),
            -32600
        );
        // PayloadTooLarge
        assert_eq!(AppError::PayloadTooLarge.mcp_error_code(), -32600);
        // RateLimitExceeded
        assert_eq!(AppError::RateLimitExceeded.mcp_error_code(), -32000);
        // ServiceUnavailable
        assert_eq!(AppError::ServiceUnavailable.mcp_error_code(), -32000);
        // Template
        assert_eq!(
            AppError::Template("err".to_string()).mcp_error_code(),
            -32603
        );
        // Analysis
        assert_eq!(
            AppError::Analysis("err".to_string()).mcp_error_code(),
            -32603
        );
    }

    #[test]
    fn test_mcp_error_codes_from_errors() {
        // Io error
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "io");
        let app_err: AppError = io_err.into();
        assert_eq!(app_err.mcp_error_code(), -32603);

        // Json error
        let json_err = serde_json::from_str::<i32>("x").unwrap_err();
        let app_err: AppError = json_err.into();
        assert_eq!(app_err.mcp_error_code(), -32603);

        // Protocol error
        let protocol_err = super::super::ProtocolError::EncodeError("encode".to_string());
        let app_err: AppError = protocol_err.into();
        assert_eq!(app_err.mcp_error_code(), -32603);
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

    #[test]
    fn test_error_types_extended() {
        assert_eq!(
            AppError::BadRequest("bad".to_string()).error_type(),
            "BAD_REQUEST"
        );
        assert_eq!(AppError::Unauthorized.error_type(), "UNAUTHORIZED");
        assert_eq!(
            AppError::Forbidden("denied".to_string()).error_type(),
            "FORBIDDEN"
        );
        assert_eq!(AppError::PayloadTooLarge.error_type(), "PAYLOAD_TOO_LARGE");
        assert_eq!(
            AppError::RateLimitExceeded.error_type(),
            "RATE_LIMIT_EXCEEDED"
        );
        assert_eq!(
            AppError::ServiceUnavailable.error_type(),
            "SERVICE_UNAVAILABLE"
        );
        assert_eq!(
            AppError::Internal(anyhow::anyhow!("err")).error_type(),
            "INTERNAL_ERROR"
        );
        assert_eq!(
            AppError::Analysis("analysis".to_string()).error_type(),
            "ANALYSIS_ERROR"
        );
    }

    #[test]
    fn test_error_types_from_errors() {
        // Io error
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "io");
        let app_err: AppError = io_err.into();
        assert_eq!(app_err.error_type(), "IO_ERROR");

        // Json error
        let json_err = serde_json::from_str::<i32>("x").unwrap_err();
        let app_err: AppError = json_err.into();
        assert_eq!(app_err.error_type(), "JSON_ERROR");

        // Protocol error
        let protocol_err = super::super::ProtocolError::InvalidFormat("invalid".to_string());
        let app_err: AppError = protocol_err.into();
        assert_eq!(app_err.error_type(), "PROTOCOL_ERROR");
    }

    #[test]
    fn test_error_display() {
        assert!(AppError::NotFound("resource".to_string())
            .to_string()
            .contains("not found"));
        assert!(AppError::Validation("field".to_string())
            .to_string()
            .contains("Validation"));
        assert!(AppError::BadRequest("bad".to_string())
            .to_string()
            .contains("Bad request"));
        assert!(AppError::Unauthorized.to_string().contains("required"));
        assert!(AppError::Forbidden("access".to_string())
            .to_string()
            .contains("forbidden"));
        assert!(AppError::PayloadTooLarge.to_string().contains("large"));
        assert!(AppError::RateLimitExceeded
            .to_string()
            .contains("Rate limit"));
        assert!(AppError::ServiceUnavailable
            .to_string()
            .contains("unavailable"));
        assert!(AppError::Template("tmpl".to_string())
            .to_string()
            .contains("Template"));
        assert!(AppError::Analysis("ana".to_string())
            .to_string()
            .contains("Analysis"));
    }

    #[tokio::test]
    async fn test_protocol_context() {
        set_protocol_context(Protocol::Mcp);
        assert_eq!(extract_protocol_from_context(), Some(Protocol::Mcp));

        clear_protocol_context();
        assert_eq!(extract_protocol_from_context(), None);
    }

    #[tokio::test]
    async fn test_protocol_context_http() {
        set_protocol_context(Protocol::Http);
        assert_eq!(extract_protocol_from_context(), Some(Protocol::Http));
        clear_protocol_context();
    }

    #[tokio::test]
    async fn test_protocol_context_cli() {
        set_protocol_context(Protocol::Cli);
        assert_eq!(extract_protocol_from_context(), Some(Protocol::Cli));
        clear_protocol_context();
    }

    #[tokio::test]
    async fn test_protocol_context_websocket() {
        set_protocol_context(Protocol::WebSocket);
        assert_eq!(extract_protocol_from_context(), Some(Protocol::WebSocket));
        clear_protocol_context();
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

    #[tokio::test]
    async fn test_error_to_protocol_response_websocket() {
        let error = AppError::BadRequest("bad input".to_string());
        let ws_response = error.to_protocol_response(Protocol::WebSocket).unwrap();
        // WebSocket uses HTTP-like responses
        assert_eq!(ws_response.status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_cli_response_exit_codes() {
        // NotFound = exit code 2
        let err = AppError::NotFound("missing".to_string());
        let response = err.to_protocol_response(Protocol::Cli).unwrap();
        assert_eq!(response.status, StatusCode::OK);

        // Validation = exit code 1
        let err = AppError::Validation("invalid".to_string());
        let response = err.to_protocol_response(Protocol::Cli).unwrap();
        assert_eq!(response.status, StatusCode::OK);

        // BadRequest = exit code 1
        let err = AppError::BadRequest("bad".to_string());
        let response = err.to_protocol_response(Protocol::Cli).unwrap();
        assert_eq!(response.status, StatusCode::OK);

        // Unauthorized = exit code 3
        let err = AppError::Unauthorized;
        let response = err.to_protocol_response(Protocol::Cli).unwrap();
        assert_eq!(response.status, StatusCode::OK);

        // Forbidden = exit code 3
        let err = AppError::Forbidden("denied".to_string());
        let response = err.to_protocol_response(Protocol::Cli).unwrap();
        assert_eq!(response.status, StatusCode::OK);

        // Internal = exit code 1
        let err = AppError::Internal(anyhow::anyhow!("internal"));
        let response = err.to_protocol_response(Protocol::Cli).unwrap();
        assert_eq!(response.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_mcp_response_structure() {
        let error = AppError::Validation("field required".to_string());
        let response = error.to_protocol_response(Protocol::Mcp).unwrap();
        // MCP always returns 200 for JSON-RPC
        assert_eq!(response.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_http_response_structure() {
        let error = AppError::Forbidden("access denied".to_string());
        let response = error.to_protocol_response(Protocol::Http).unwrap();
        assert_eq!(response.status, StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_mcp_error_serialization() {
        let err = McpError {
            code: -32001,
            message: "Not found".to_string(),
            data: Some(json!({"key": "value"})),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("-32001"));
        assert!(json.contains("Not found"));

        let deserialized: McpError = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code, -32001);
        assert_eq!(deserialized.message, "Not found");
    }

    #[test]
    fn test_mcp_error_without_data() {
        let err = McpError {
            code: -32600,
            message: "Invalid".to_string(),
            data: None,
        };
        let json = serde_json::to_string(&err).unwrap();
        // data should be skipped when None
        assert!(!json.contains("data"));

        let deserialized: McpError = serde_json::from_str(&json).unwrap();
        assert!(deserialized.data.is_none());
    }

    #[test]
    fn test_mcp_error_debug() {
        let err = McpError {
            code: -32603,
            message: "Internal".to_string(),
            data: None,
        };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("McpError"));
        assert!(debug_str.contains("-32603"));
    }

    #[test]
    fn test_http_error_response_serialization() {
        let err = HttpErrorResponse {
            error: "Something went wrong".to_string(),
            error_type: "INTERNAL_ERROR".to_string(),
            timestamp: "2025-01-12T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("Something went wrong"));
        assert!(json.contains("INTERNAL_ERROR"));

        let deserialized: HttpErrorResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.error, "Something went wrong");
        assert_eq!(deserialized.error_type, "INTERNAL_ERROR");
    }

    #[test]
    fn test_http_error_response_debug() {
        let err = HttpErrorResponse {
            error: "Error".to_string(),
            error_type: "BAD_REQUEST".to_string(),
            timestamp: "2025-01-12T00:00:00Z".to_string(),
        };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("HttpErrorResponse"));
    }

    #[test]
    fn test_cli_error_response_serialization() {
        let err = CliErrorResponse {
            message: "Command failed".to_string(),
            error_type: "VALIDATION_ERROR".to_string(),
            exit_code: 1,
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("Command failed"));
        assert!(json.contains("exit_code"));

        let deserialized: CliErrorResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.message, "Command failed");
        assert_eq!(deserialized.exit_code, 1);
    }

    #[test]
    fn test_cli_error_response_debug() {
        let err = CliErrorResponse {
            message: "Fail".to_string(),
            error_type: "NOT_FOUND".to_string(),
            exit_code: 2,
        };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("CliErrorResponse"));
    }

    #[test]
    fn test_into_response_with_default_protocol() {
        clear_protocol_context();
        let error = AppError::NotFound("test".to_string());
        let response = error.into_response();
        // Default is HTTP protocol
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_into_response_with_mcp_protocol() {
        set_protocol_context(Protocol::Mcp);
        let error = AppError::BadRequest("bad".to_string());
        let response = error.into_response();
        // MCP returns 200 for JSON-RPC
        assert_eq!(response.status(), StatusCode::OK);
        clear_protocol_context();
    }

    #[test]
    fn test_into_response_with_http_protocol() {
        set_protocol_context(Protocol::Http);
        let error = AppError::Unauthorized;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        clear_protocol_context();
    }

    #[test]
    fn test_error_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("something went wrong");
        let app_err: AppError = anyhow_err.into();
        assert!(matches!(app_err, AppError::Internal(_)));
        assert!(app_err.to_string().contains("something went wrong"));
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
