#![cfg_attr(coverage_nightly, coverage(off))]
//! Data types for the O(1) Hooks Cache Manager.
//!
//! Contains all structs and enums used for cache entries, gate definitions,
//! gate results, and cache checking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Level 0: Repo-wide cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeHashCache {
    /// Git tree hash of HEAD
    pub tree_hash: String,
    /// Overall result (pass/fail)
    pub result: CacheResult,
    /// Per-gate results
    pub gates: HashMap<String, GateCacheEntry>,
    /// Timestamp of cache creation
    pub timestamp: DateTime<Utc>,
    /// PMAT version that created this cache
    pub pmat_version: String,
    /// Config hash (invalidate on config change)
    pub config_hash: String,
}

/// Level 1: Per-gate cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCacheEntry {
    /// Hash of files relevant to this gate
    pub files_hash: String,
    /// Gate result
    pub result: CacheResult,
    /// Execution time in milliseconds
    pub duration_ms: u64,
    /// Number of files checked
    pub files_checked: usize,
    /// Warnings (if any)
    pub warnings: Vec<String>,
}

/// Cache result enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheResult {
    Pass,
    Fail,
    Warn,
}

/// Metrics for cache health monitoring (CB-021)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HooksCacheMetrics {
    pub total_runs: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_cache_hit_time_ms: f64,
    pub avg_cache_miss_time_ms: f64,
    pub last_full_rebuild: Option<DateTime<Utc>>,
    pub cache_size_bytes: u64,
}

/// Result of cache check
#[derive(Debug)]
pub enum CacheCheckResult {
    /// Cache hit - can skip all gates
    Hit {
        result: CacheResult,
        cached_at: DateTime<Utc>,
    },
    /// Cache miss - need to run gates
    Miss { reason: CacheMissReason },
    /// Partial hit - some gates can be skipped
    Partial {
        cached_gates: Vec<String>,
        uncached_gates: Vec<String>,
    },
}

/// Reason for cache miss
#[derive(Debug, Clone)]
pub enum CacheMissReason {
    /// No cache file exists
    NoCacheFile,
    /// Tree hash changed
    TreeHashChanged { old: String, new: String },
    /// Config hash changed
    ConfigHashChanged,
    /// Cache is stale (too old)
    CacheStale { age_hours: i64 },
    /// PMAT version changed
    VersionChanged { old: String, new: String },
    /// Cache file corrupted
    CacheCorrupted(String),
}

impl std::error::Error for CacheMissReason {}

impl std::fmt::Display for CacheMissReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheMissReason::NoCacheFile => write!(f, "No cache file exists"),
            CacheMissReason::TreeHashChanged { old, new } => {
                write!(
                    f,
                    "Tree hash changed: {} → {}",
                    old.get(..8).unwrap_or(old),
                    new.get(..8).unwrap_or(new)
                )
            }
            CacheMissReason::ConfigHashChanged => write!(f, "Config file changed"),
            CacheMissReason::CacheStale { age_hours } => {
                write!(
                    f,
                    "Cache stale ({}h old, max {}h)",
                    age_hours,
                    super::MAX_CACHE_AGE_HOURS
                )
            }
            CacheMissReason::VersionChanged { old, new } => {
                write!(f, "PMAT version changed: {} → {}", old, new)
            }
            CacheMissReason::CacheCorrupted(msg) => write!(f, "Cache corrupted: {}", msg),
        }
    }
}

// =========================================================================
// Phase 2 & 3: Gate Definition and Results
// =========================================================================

/// Definition of a quality gate for Phase 2/3
#[derive(Debug, Clone)]
pub struct GateDefinition {
    /// Gate name (e.g., "complexity", "satd", "format")
    pub name: String,
    /// Files this gate operates on
    pub files: Vec<PathBuf>,
    /// File patterns (e.g., "*.rs", "*.py")
    pub patterns: Vec<String>,
}

impl GateDefinition {
    /// Create a new gate definition
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(name: impl Into<String>, files: Vec<PathBuf>) -> Self {
        Self {
            name: name.into(),
            files,
            patterns: vec![],
        }
    }

    /// Create with file patterns
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_patterns(name: impl Into<String>, patterns: Vec<String>) -> Self {
        Self {
            name: name.into(),
            files: vec![],
            patterns,
        }
    }
}

/// Result of running a single gate
#[derive(Debug, Clone)]
pub struct GateRunResult {
    /// Gate result
    pub result: CacheResult,
    /// Execution time in milliseconds
    pub duration_ms: u64,
    /// Warnings generated
    pub warnings: Vec<String>,
    /// Whether result came from cache
    pub from_cache: bool,
}

/// Result of checking which gates are cached (Phase 2)
#[derive(Debug)]
pub struct GateCheckResult {
    /// Gates with valid cache entries
    pub cached: Vec<(String, GateCacheEntry)>,
    /// Gates that need to run
    pub uncached: Vec<GateDefinition>,
}

/// Results of parallel gate execution (Phase 3)
#[derive(Debug)]
pub struct ParallelGateResults {
    /// Overall result
    pub overall: CacheResult,
    /// Individual gate results
    pub results: Vec<(String, GateRunResult)>,
    /// Errors from failed gates
    pub errors: Vec<(String, String)>,
    /// Total execution time
    pub total_duration_ms: u64,
}

/// Results of smart gate execution (Phase 2 + 3 combined)
#[derive(Debug)]
pub struct SmartGateResults {
    /// Overall result
    pub overall: CacheResult,
    /// Individual gate results (cached + freshly run)
    pub results: Vec<(String, GateRunResult)>,
    /// Errors from failed gates
    pub errors: Vec<(String, String)>,
    /// Number of gates that used cache
    pub gates_cached: usize,
    /// Number of gates that had to run
    pub gates_run: usize,
    /// Total execution time
    pub total_duration_ms: u64,
}
