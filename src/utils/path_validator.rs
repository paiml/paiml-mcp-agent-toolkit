#![cfg_attr(coverage_nightly, coverage(off))]
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

// Validation methods (ensure_exists, ensure_file, ensure_directory, etc.)
include!("path_validator_checks.rs");

// Tests
include!("path_validator_tests.rs");
