//! Path Validation Utilities
//!
//! Provides centralized, standardized path validation functions to reduce code duplication
//! and improve maintainability across the PMAT codebase.

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

/// Path validation errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum PathValidationError {
    #[error("Path does not exist: {path}")]
    NotFound { path: PathBuf },

    #[error("Path is not a file: {path}")]
    NotFile { path: PathBuf },

    #[error("Path is not a directory: {path}")]
    NotDirectory { path: PathBuf },

    #[error("Path is not readable: {path}")]
    NotReadable { path: PathBuf },

    #[error("Invalid path: {path}")]
    Invalid { path: PathBuf },
}

/// Centralized path validation utilities
pub struct PathValidator;

impl PathValidator {
    /// Validate that a path exists
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use pmat::utils::path_validator::PathValidator;
    ///
    /// let existing_path = Path::new("Cargo.toml");
    /// assert!(PathValidator::ensure_exists(existing_path).is_ok());
    /// ```
    pub fn ensure_exists(path: &Path) -> Result<(), PathValidationError> {
        if !path.exists() {
            return Err(PathValidationError::NotFound {
                path: path.to_path_buf(),
            });
        }
        Ok(())
    }

    /// Validate that a path exists and is a file
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use pmat::utils::path_validator::PathValidator;
    ///
    /// let file_path = Path::new("Cargo.toml");
    /// assert!(PathValidator::ensure_file(file_path).is_ok());
    /// ```
    pub fn ensure_file(path: &Path) -> Result<(), PathValidationError> {
        Self::ensure_exists(path)?;

        if !path.is_file() {
            return Err(PathValidationError::NotFile {
                path: path.to_path_buf(),
            });
        }
        Ok(())
    }

    /// Validate that a path exists and is a directory
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use pmat::utils::path_validator::PathValidator;
    ///
    /// let dir_path = Path::new("src");
    /// assert!(PathValidator::ensure_directory(dir_path).is_ok());
    /// ```
    pub fn ensure_directory(path: &Path) -> Result<(), PathValidationError> {
        Self::ensure_exists(path)?;

        if !path.is_dir() {
            return Err(PathValidationError::NotDirectory {
                path: path.to_path_buf(),
            });
        }
        Ok(())
    }

    /// Validate that a path exists and is readable
    pub fn ensure_readable(path: &Path) -> Result<(), PathValidationError> {
        Self::ensure_exists(path)?;

        // Check if we can read the file/directory
        match std::fs::metadata(path) {
            Ok(_) => Ok(()),
            Err(_) => Err(PathValidationError::NotReadable {
                path: path.to_path_buf(),
            }),
        }
    }

    /// Get parent directory, validating it exists
    ///
    /// Returns the parent directory if the path is a file, or the path itself if it's a directory
    pub fn get_valid_parent(path: &Path) -> Result<&Path, PathValidationError> {
        Self::ensure_exists(path)?;

        if path.is_file() {
            path.parent().ok_or_else(|| PathValidationError::Invalid {
                path: path.to_path_buf(),
            })
        } else if path.is_dir() {
            Ok(path)
        } else {
            Err(PathValidationError::Invalid {
                path: path.to_path_buf(),
            })
        }
    }

    /// Check if path is a valid source file (with common extensions)
    #[must_use]
    pub fn is_source_file(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| {
                matches!(
                    ext,
                    "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "h" | "hpp"
                )
            })
    }

    /// Validate path exists and return appropriate error for anyhow
    pub fn validate_exists_anyhow(path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(anyhow!("Path does not exist: {}", path.display()));
        }
        Ok(())
    }

    /// Validate path is file and return appropriate error for anyhow
    pub fn validate_file_anyhow(path: &Path) -> Result<()> {
        Self::validate_exists_anyhow(path)?;

        if !path.is_file() {
            return Err(anyhow!("Path is not a file: {}", path.display()));
        }
        Ok(())
    }

    /// Validate path is directory and return appropriate error for anyhow
    pub fn validate_directory_anyhow(path: &Path) -> Result<()> {
        Self::validate_exists_anyhow(path)?;

        if !path.is_dir() {
            return Err(anyhow!("Path is not a directory: {}", path.display()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    #[ignore]
    fn test_ensure_exists_valid_file() {
        // Use Cargo.toml as a file we know exists
        let path = Path::new("Cargo.toml");
        assert!(PathValidator::ensure_exists(path).is_ok());
    }

    #[test]
    fn test_ensure_exists_invalid_file() {
        let path = Path::new("nonexistent_file.txt");
        assert!(PathValidator::ensure_exists(path).is_err());
    }

    #[test]
    #[ignore]
    fn test_ensure_file_valid() {
        let path = Path::new("Cargo.toml");
        assert!(PathValidator::ensure_file(path).is_ok());
    }

    #[test]
    #[ignore]
    fn test_ensure_directory_valid() {
        let path = Path::new("src");
        assert!(PathValidator::ensure_directory(path).is_ok());
    }

    #[test]
    fn test_get_valid_parent() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "test content")?;

        let parent = PathValidator::get_valid_parent(&file_path)?;
        assert_eq!(parent, temp_dir.path());

        Ok(())
    }

    #[test]
    fn test_is_source_file() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Test Rust file
        let rs_file = temp_dir.path().join("test.rs");
        fs::write(&rs_file, "fn main() {}")?;
        assert!(PathValidator::is_source_file(&rs_file));

        // Test non-source file
        let txt_file = temp_dir.path().join("test.txt");
        fs::write(&txt_file, "text content")?;
        assert!(!PathValidator::is_source_file(&txt_file));

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_validate_anyhow_methods() {
        // Test with Cargo.toml (should exist)
        let path = Path::new("Cargo.toml");
        assert!(PathValidator::validate_exists_anyhow(path).is_ok());
        assert!(PathValidator::validate_file_anyhow(path).is_ok());

        // Test with src directory
        let dir_path = Path::new("src");
        assert!(PathValidator::validate_directory_anyhow(dir_path).is_ok());

        // Test with nonexistent path
        let bad_path = Path::new("nonexistent");
        assert!(PathValidator::validate_exists_anyhow(bad_path).is_err());
    }
}
