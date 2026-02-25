// AppError method implementations and IntoResponse trait impl.
// Included by error.rs - shares its module scope (no `use` imports here).

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
