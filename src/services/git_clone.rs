#![cfg_attr(coverage_nightly, coverage(off))]
//! Git repository cloning and caching service
//!
//! This module provides efficient Git repository cloning with caching,
//! progress tracking, and automatic cleanup. It supports both HTTPS and SSH
//! URLs, handles authentication, and prevents redundant clones through
//! intelligent caching strategies.
//!
//! # Feature Flag
//!
//! This module requires the `git-lib` feature for full functionality.
//! Without it, only basic shell `git clone` is available.

#![cfg(feature = "git-lib")]

//! # Features
//!
//! - **URL Normalization**: Handles various GitHub URL formats
//! - **Smart Caching**: Avoids re-cloning already cached repositories
//! - **Progress Tracking**: Real-time clone progress reporting
//! - **Automatic Cleanup**: Removes old clones to save disk space
//! - **Concurrent Cloning**: Thread-safe operations with proper locking
//!
//! # Example
//!
//! ```no_run
//! use pmat::services::git_clone::{GitCloner, ClonedRepo};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let cloner = GitCloner::new(PathBuf::from(".cache"));
//!
//! // Clone a repository
//! let result = cloner.clone_or_update("https://github.com/rust-lang/rust").await?;
//!
//! println!("Cloned to: {}", result.path.display());
//! println!("From cache: {}", result.cached);
//!
//! // Subsequent calls use cache
//! let cached = cloner.clone_or_update("https://github.com/rust-lang/rust").await?;
//! assert!(cached.cached);
//! # Ok(())
//! # }
//! ```

use anyhow::Result;
use git2::{build::RepoBuilder, FetchOptions, Progress, RemoteCallbacks, Repository};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;

lazy_static! {
    // Pre-compiled regex patterns for GitHub URL parsing
    // Name pattern: alphanumeric at start/end, can contain dash, underscore, dot in middle
    // Single char names are also valid
    static ref NAME_PATTERN: &'static str = r"[a-zA-Z0-9](?:[a-zA-Z0-9\-_\.]*[a-zA-Z0-9])?";

    static ref GITHUB_HTTPS_REGEX: Regex = {
        Regex::new(&format!(
            r"^https://github\.com/({name})/({name})(?:\.git)?/?$",
            name = *NAME_PATTERN
        ))
        .expect("Invalid HTTPS regex pattern")
    };

    static ref GITHUB_SSH_REGEX: Regex = {
        Regex::new(&format!(
            r"^git@github\.com:({name})/({name})(?:\.git)?$",
            name = *NAME_PATTERN
        ))
        .expect("Invalid SSH regex pattern")
    };

    static ref GITHUB_SHORT_REGEX: Regex = {
        Regex::new(&format!(
            r"^({name})/({name})$",
            name = *NAME_PATTERN
        ))
        .expect("Invalid short format regex pattern")
    };
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Clone progress.
pub struct CloneProgress {
    pub stage: String,
    pub current: usize,
    pub total: usize,
    pub bytes_transferred: usize,
}

#[derive(Clone, Debug)]
/// Cloned repo.
pub struct ClonedRepo {
    pub path: PathBuf,
    pub url: String,
    pub cached: bool,
}

#[derive(Debug, thiserror::Error)]
/// Error variants for clone operations.
pub enum CloneError {
    #[error("Git error: {0}")]
    GitError(#[from] git2::Error),

    #[error("Repository too large: {size_mb}MB exceeds limit")]
    TooLarge { size_mb: u64 },

    #[error("Clone operation timed out")]
    Timeout,

    #[error("Invalid GitHub URL: {0}")]
    InvalidUrl(String),

    #[error("GitHub API error: {0}")]
    ApiError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Clone)]
/// Git cloner.
pub struct GitCloner {
    cache_dir: PathBuf,
    progress: Arc<Mutex<CloneProgress>>,
    timeout: Duration,
    max_size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq)]
/// Parsed git hub url.
pub struct ParsedGitHubUrl {
    pub owner: String,
    pub repo: String,
}

// --- Included implementation files ---

include!("git_clone_operations.rs");
include!("git_clone_url_parsing.rs");
include!("git_clone_tests.rs");
