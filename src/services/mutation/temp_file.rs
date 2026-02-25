#![cfg_attr(coverage_nightly, coverage(off))]
//! Temporary file management for distributed mutation testing
//!
//! Provides RAII-based temporary file handling for distributed mutation
//! testing to ensure proper cleanup, even in error cases.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

/// RAII wrapper for temporary files in distributed mutation testing
///
/// Ensures temporary files are cleaned up when they go out of scope,
/// even in the case of errors or panics.
pub struct WorkerTempFile {
    /// Path to the temporary file
    path: PathBuf,

    /// Whether the file has been manually cleaned up
    cleaned_up: bool,

    /// Whether to use sync cleanup (for Drop)
    use_sync_cleanup: bool,
}

/// RAII wrapper for temporary directories
///
/// Ensures temporary directories are cleaned up when they go out of scope,
/// even in the case of errors or panics.
pub struct TempDir {
    /// Path to the temporary directory
    path: PathBuf,

    /// Whether the directory has been manually cleaned up
    cleaned_up: bool,
}

// WorkerTempFile methods, Drop impl, and create_temp_dir free function
include!("temp_file_operations.rs");

// TempDir methods and Drop impl
include!("temp_file_dir.rs");

// Tests
include!("temp_file_tests.rs");
