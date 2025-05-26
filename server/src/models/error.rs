use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("S3 operation failed: {operation}")]
    S3Error {
        operation: String,
        #[source]
        source: anyhow::Error,
    },

    #[error("Template not found: {uri}")]
    TemplateNotFound { uri: String },

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid template URI: {uri}")]
    InvalidUri { uri: String },

    #[error("Template rendering failed at line {line}: {message}")]
    RenderError { line: u32, message: String },

    #[error("Parameter validation failed: {parameter} - {reason}")]
    ValidationError { parameter: String, reason: String },

    #[error("Invalid UTF-8 in template content")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("Cache operation failed")]
    CacheError(#[from] anyhow::Error),

    #[error("JSON serialization error")]
    JsonError(#[from] serde_json::Error),
}

impl TemplateError {
    pub fn to_mcp_code(&self) -> i32 {
        match self {
            TemplateError::TemplateNotFound { .. } => -32001,
            TemplateError::InvalidUri { .. } => -32002,
            TemplateError::ValidationError { .. } => -32003,
            TemplateError::RenderError { .. } => -32004,
            _ => -32000, // Generic error
        }
    }
}
