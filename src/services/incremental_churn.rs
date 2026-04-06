#![cfg_attr(coverage_nightly, coverage(off))]
//! Incremental/Lazy Churn Analysis Service
//!
//! This module provides an incremental churn analysis implementation that
//! only analyzes files that have changed since the last analysis run.

use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
use crate::models::error::TemplateError;
use crate::services::git_analysis::GitAnalysisService;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Cache entry for file churn metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChurnCacheEntry {
    pub metrics: FileChurnMetrics,
    pub last_modified: DateTime<Utc>,
    pub git_commit_hash: String,
}

/// Incremental churn analyzer with lazy evaluation
pub struct IncrementalChurnAnalyzer {
    /// Cache of file metrics keyed by path
    cache: Arc<DashMap<PathBuf, ChurnCacheEntry>>,
    /// Project root path
    project_root: PathBuf,
}

impl IncrementalChurnAnalyzer {
    #[must_use]
    pub fn new(project_root: PathBuf) -> Self {
        debug_assert!(
            project_root.exists(),
            "project_root must exist: {}",
            project_root.display()
        );
        Self {
            cache: Arc::new(DashMap::new()),
            project_root,
        }
    }

    /// Clear the cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    #[must_use]
    pub fn cache_stats(&self) -> (usize, usize) {
        let cache_size = self.cache.len();
        let cache_memory = cache_size * std::mem::size_of::<(PathBuf, ChurnCacheEntry)>();
        (cache_size, cache_memory)
    }
}

// Analysis methods: get_file_churn, analyze_incremental, is_cache_valid
include!("incremental_churn_analysis.rs");

// Compute methods: compute_file_churn, batch_compute_churn, git helpers, parsing, summary
include!("incremental_churn_compute.rs");

// Statistics functions: scalar and SIMD churn statistics
include!("incremental_churn_statistics.rs");

// Tests
include!("incremental_churn_tests.rs");
