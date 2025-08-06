//! Error types for agent scaffolding operations.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during agent scaffolding.
#[derive(Debug, Error)]
pub enum ScaffoldError {
    /// Template not found.
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    /// Invalid agent configuration.
    #[error("Invalid agent configuration: {0}")]
    InvalidConfiguration(String),

    /// Directory already exists.
    #[error("Directory already exists: {}", .0.display())]
    DirectoryExists(PathBuf),

    /// Template generation failed.
    #[error("Template generation failed")]
    GenerationFailed(#[source] anyhow::Error),

    /// Post-generation hook failed.
    #[error("Post-generation hook failed: {0}")]
    HookFailed(String),

    /// Template validation failed.
    #[error("Template validation failed")]
    ValidationFailed(#[source] anyhow::Error),

    /// Incompatible template version.
    #[error("Incompatible template version: requires PMAT {required}, but running {current}")]
    IncompatibleVersion {
        /// Required version.
        required: String,
        /// Current version.
        current: String,
    },

    /// Missing required feature.
    #[error("Missing required feature: {0}")]
    MissingFeature(String),

    /// I/O error.
    #[error("I/O error")]
    IoError(#[from] std::io::Error),

    /// Template rendering error.
    #[error("Template rendering failed: {0}")]
    RenderError(String),

    /// Invalid template syntax.
    #[error("Invalid template syntax: {0}")]
    InvalidTemplate(String),

    /// User cancelled operation.
    #[error("Operation cancelled by user")]
    UserCancelled,

    /// Network error (for remote templates).
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Parse error.
    #[error("Parse error: {0}")]
    ParseError(String),
}

// Note: anyhow already provides a blanket From implementation for Error types

/// Result type for scaffold operations.
pub type ScaffoldResult<T> = Result<T, ScaffoldError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ScaffoldError::TemplateNotFound("my-template".to_string());
        assert_eq!(err.to_string(), "Template not found: my-template");

        let err = ScaffoldError::DirectoryExists(PathBuf::from("/tmp/test"));
        assert_eq!(err.to_string(), "Directory already exists: /tmp/test");

        let err = ScaffoldError::IncompatibleVersion {
            required: "0.30.0".to_string(),
            current: "0.29.0".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Incompatible template version: requires PMAT 0.30.0, but running 0.29.0"
        );
    }

    #[test]
    fn test_error_conversion() {
        let err = ScaffoldError::UserCancelled;
        let anyhow_err: anyhow::Error = err.into();
        assert_eq!(anyhow_err.to_string(), "Operation cancelled by user");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let scaffold_err = ScaffoldError::from(io_err);
        assert!(matches!(scaffold_err, ScaffoldError::IoError(_)));
    }
}