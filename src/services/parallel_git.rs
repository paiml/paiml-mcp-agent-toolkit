#![cfg_attr(coverage_nightly, coverage(off))]
//! Parallel Git Operations Service
//!
//! This module provides parallelized git operations using connection pooling
//! and concurrent execution for improved performance.
//!
//! ## Module Structure
//! - `parallel_git_operations.rs`: Core executor methods (command execution, batching, parsing)
//! - `parallel_git_tests.rs`: Unit tests and property tests

use crate::models::error::TemplateError;
use anyhow::Result;
use futures::future::join_all;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info};

/// Configuration for parallel git operations
#[derive(Debug, Clone)]
pub struct ParallelGitConfig {
    /// Maximum number of concurrent git processes
    pub max_concurrent_operations: usize,
    /// Enable command caching
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
}

impl Default for ParallelGitConfig {
    fn default() -> Self {
        Self {
            max_concurrent_operations: num_cpus::get().min(8),
            enable_caching: true,
            cache_ttl_seconds: 300, // 5 minutes
        }
    }
}

/// Result cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    result: String,
    timestamp: std::time::Instant,
}

/// Parallel git operations executor
pub struct ParallelGitExecutor {
    config: ParallelGitConfig,
    semaphore: Arc<Semaphore>,
    cache: Arc<RwLock<rustc_hash::FxHashMap<String, CacheEntry>>>,
    project_root: PathBuf,
}

/// Commit information
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

/// Diff statistics
#[derive(Debug, Clone)]
pub struct DiffStats {
    pub file: PathBuf,
    pub additions: usize,
    pub deletions: usize,
}

// --- Submodule includes ---
include!("parallel_git_operations.rs");
include!("parallel_git_tests.rs");
