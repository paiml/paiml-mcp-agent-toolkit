//! Enhanced error context for better CLI error messages
//!
//! Provides rich error context with file paths, suggestions, and actionable advice.

use std::path::{Path, PathBuf};
use thiserror::Error;

/// Enhanced error with context and suggestions
#[derive(Debug, Error)]
pub enum ContextError {
    #[error("Failed to read {file_path}")]
    FileNotFound {
        file_path: PathBuf,
        suggestions: Vec<String>,
    },

    #[error("Failed to write {file_path}")]
    FileWriteError {
        file_path: PathBuf,
        reason: String,
        suggestions: Vec<String>,
    },

    #[error("Failed to parse {file_path}")]
    ParseError {
        file_path: PathBuf,
        reason: String,
        suggestions: Vec<String>,
    },

    #[error("Invalid configuration in {file_path}")]
    ConfigError {
        file_path: PathBuf,
        field: String,
        reason: String,
        suggestions: Vec<String>,
    },
}

impl ContextError {
    /// Format error with full context and suggestions
    ///
    /// CC=4: Match on enum variants
    pub fn format_detailed(&self) -> String {
        match self {
            ContextError::FileNotFound {
                file_path,
                suggestions,
            } => format_file_not_found(file_path, suggestions),

            ContextError::FileWriteError {
                file_path,
                reason,
                suggestions,
            } => format_file_write_error(file_path, reason, suggestions),

            ContextError::ParseError {
                file_path,
                reason,
                suggestions,
            } => format_parse_error(file_path, reason, suggestions),

            ContextError::ConfigError {
                file_path,
                field,
                reason,
                suggestions,
            } => format_config_error(file_path, field, reason, suggestions),
        }
    }
}

/// Format file not found error
///
/// CC=2: Conditional suggestions
fn format_file_not_found(file_path: &Path, suggestions: &[String]) -> String {
    let mut msg = format!(
        "ERROR: Failed to read {}\n  Location: {}\n  Reason: File not found",
        file_path.file_name().unwrap_or_default().to_string_lossy(),
        file_path.display()
    );

    if !suggestions.is_empty() {
        msg.push_str("\n\n  Suggestions:");
        for suggestion in suggestions {
            msg.push_str(&format!("\n  - {}", suggestion));
        }
    }

    msg
}

/// Format file write error
///
/// CC=2: Conditional suggestions
fn format_file_write_error(file_path: &Path, reason: &str, suggestions: &[String]) -> String {
    let mut msg = format!(
        "ERROR: Failed to write {}\n  Location: {}\n  Reason: {}",
        file_path.file_name().unwrap_or_default().to_string_lossy(),
        file_path.display(),
        reason
    );

    if !suggestions.is_empty() {
        msg.push_str("\n\n  Suggestions:");
        for suggestion in suggestions {
            msg.push_str(&format!("\n  - {}", suggestion));
        }
    }

    msg
}

/// Format parse error
///
/// CC=2: Conditional suggestions
fn format_parse_error(file_path: &Path, reason: &str, suggestions: &[String]) -> String {
    let mut msg = format!(
        "ERROR: Failed to parse {}\n  Location: {}\n  Reason: {}",
        file_path.file_name().unwrap_or_default().to_string_lossy(),
        file_path.display(),
        reason
    );

    if !suggestions.is_empty() {
        msg.push_str("\n\n  Suggestions:");
        for suggestion in suggestions {
            msg.push_str(&format!("\n  - {}", suggestion));
        }
    }

    msg
}

/// Format configuration error
///
/// CC=2: Conditional suggestions
fn format_config_error(
    file_path: &Path,
    field: &str,
    reason: &str,
    suggestions: &[String],
) -> String {
    let mut msg = format!(
        "ERROR: Invalid configuration in {}\n  Location: {}\n  Field: {}\n  Reason: {}",
        file_path.file_name().unwrap_or_default().to_string_lossy(),
        file_path.display(),
        field,
        reason
    );

    if !suggestions.is_empty() {
        msg.push_str("\n\n  Suggestions:");
        for suggestion in suggestions {
            msg.push_str(&format!("\n  - {}", suggestion));
        }
    }

    msg
}

/// Helper to create file not found error with roadmap suggestions
///
/// CC=1: Simple constructor
pub fn roadmap_not_found(path: &Path) -> ContextError {
    ContextError::FileNotFound {
        file_path: path.to_path_buf(),
        suggestions: vec![
            "Run 'pmat maintain roadmap' from project root".to_string(),
            "Ensure you're in the correct directory".to_string(),
            "Check if ROADMAP.md exists in your project".to_string(),
        ],
    }
}

/// Helper to create file not found error with Cargo.toml suggestions
///
/// CC=1: Simple constructor
pub fn cargo_toml_not_found(path: &Path) -> ContextError {
    ContextError::FileNotFound {
        file_path: path.to_path_buf(),
        suggestions: vec![
            "Ensure you're in a Rust project directory".to_string(),
            "Run 'cargo init' to create a new Rust project".to_string(),
            "Check if you're in the correct directory".to_string(),
        ],
    }
}

/// Helper to create file not found error with generic suggestions
///
/// CC=1: Simple constructor
pub fn file_not_found(path: &Path) -> ContextError {
    ContextError::FileNotFound {
        file_path: path.to_path_buf(),
        suggestions: vec![
            format!("Ensure {} exists", path.display()),
            "Check if you're in the correct directory".to_string(),
            "Verify the file path is correct".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roadmap_not_found_error() {
        let path = PathBuf::from("/home/user/project/ROADMAP.md");
        let error = roadmap_not_found(&path);
        let formatted = error.format_detailed();

        assert!(formatted.contains("ROADMAP.md"));
        assert!(formatted.contains("/home/user/project/ROADMAP.md"));
        assert!(formatted.contains("File not found"));
        assert!(formatted.contains("Suggestions:"));
        assert!(formatted.contains("pmat maintain roadmap"));
    }

    #[test]
    fn test_cargo_toml_not_found_error() {
        let path = PathBuf::from("/project/Cargo.toml");
        let error = cargo_toml_not_found(&path);
        let formatted = error.format_detailed();

        assert!(formatted.contains("Cargo.toml"));
        assert!(formatted.contains("Rust project"));
        assert!(formatted.contains("cargo init"));
    }

    #[test]
    fn test_file_write_error_formatting() {
        let error = ContextError::FileWriteError {
            file_path: PathBuf::from("/tmp/output.txt"),
            reason: "Permission denied".to_string(),
            suggestions: vec!["Check file permissions".to_string()],
        };

        let formatted = error.format_detailed();
        assert!(formatted.contains("output.txt"));
        assert!(formatted.contains("Permission denied"));
        assert!(formatted.contains("Check file permissions"));
    }

    #[test]
    fn test_parse_error_formatting() {
        let error = ContextError::ParseError {
            file_path: PathBuf::from("/config/settings.toml"),
            reason: "Invalid TOML syntax".to_string(),
            suggestions: vec!["Validate TOML syntax online".to_string()],
        };

        let formatted = error.format_detailed();
        assert!(formatted.contains("settings.toml"));
        assert!(formatted.contains("Invalid TOML syntax"));
    }

    #[test]
    fn test_config_error_formatting() {
        let error = ContextError::ConfigError {
            file_path: PathBuf::from("/project/.pmat.toml"),
            field: "max_complexity".to_string(),
            reason: "Must be between 1 and 20".to_string(),
            suggestions: vec!["Set max_complexity to a value between 1 and 20".to_string()],
        };

        let formatted = error.format_detailed();
        assert!(formatted.contains(".pmat.toml"));
        assert!(formatted.contains("max_complexity"));
        assert!(formatted.contains("between 1 and 20"));
    }
}
