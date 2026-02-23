#![cfg_attr(coverage_nightly, coverage(off))]
//! O(1) Hooks Cache Manager (PMAT-453)
//!
//! Implements the 3-level hash hierarchy for O(1) pre-commit hooks:
//! - Level 0: Git tree hash (repo-wide cache)
//! - Level 1: Per-gate hash (staged files by type)
//! - Level 2: Per-file hash (individual file results)
//!
//! Phase 2: Per-gate caching for partial cache hits
//! Phase 3: Parallel gate execution with rayon
//!
//! Spec: server/docs/specifications/pmat-hooks-v2-spec.md

pub mod types;

mod gates;
mod helpers;
mod manager;

use std::path::{Path, PathBuf};

pub use types::{
    CacheCheckResult, CacheMissReason, CacheResult, GateCacheEntry, GateCheckResult,
    GateDefinition, GateRunResult, HooksCacheMetrics, ParallelGateResults, SmartGateResults,
    TreeHashCache,
};

/// Cache directory relative to project root
const CACHE_DIR: &str = ".pmat/hooks-cache";

/// Maximum cache age before forced re-run (hours)
const MAX_CACHE_AGE_HOURS: i64 = 24;

/// O(1) Hooks Cache Manager
#[derive(Debug)]
pub struct HooksCacheManager {
    project_path: PathBuf,
    cache_dir: PathBuf,
}

impl HooksCacheManager {
    /// Create new cache manager for project
    pub fn new(project_path: &Path) -> Self {
        let cache_dir = project_path.join(CACHE_DIR);
        Self {
            project_path: project_path.to_path_buf(),
            cache_dir,
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests;
