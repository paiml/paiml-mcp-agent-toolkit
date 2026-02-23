#![cfg_attr(coverage_nightly, coverage(off))]
//! Type definitions for the enhanced parser module

use crate::unified_quality::metrics::Metrics;
use std::time::SystemTime;

/// Cached syntax tree with metadata
pub struct CachedSyntax {
    /// Serialized syntax tree (to avoid Send issues)
    pub syntax_str: String,

    /// Source code content
    pub content: String,

    /// Last modified time
    pub last_modified: SystemTime,

    /// Content hash for validation
    pub content_hash: u64,

    /// Computed metrics
    pub metrics: Option<Metrics>,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub memory_usage_estimate: usize,
}
