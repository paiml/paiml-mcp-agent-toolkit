#![cfg_attr(coverage_nightly, coverage(off))]
//! HTTP error types for uniform contract endpoints

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Error type for HTTP endpoints
#[derive(Debug)]
pub(super) enum AppError {
    BadRequest(String),
    Internal(anyhow::Error),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        let err_str = err.to_string();
        // Check if this is a contract validation error (should be 400, not 500)
        if err_str.contains("Path not found")
            || err_str.contains("Invalid timeout")
            || err_str.contains("Too many files")
            || err_str.contains("Invalid parameter")
            || err_str.contains("Missing required")
            || err_str.contains("must be")
        {
            AppError::BadRequest(err_str)
        } else {
            AppError::Internal(err)
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
    }
}
