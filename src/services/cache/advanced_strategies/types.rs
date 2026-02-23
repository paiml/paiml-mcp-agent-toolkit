#![cfg_attr(coverage_nightly, coverage(off))]
//! Type definitions for advanced caching strategies
//!
//! Contains enums, configuration structs, statistics types, and cache entry definitions.

use chrono::{DateTime, Utc};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicUsize};
use std::sync::Arc;
use std::time::Duration;

/// Cache eviction policies for different use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least Recently Used - good for general purpose
    LRU,
    /// Least Frequently Used - good for long-running processes
    LFU,
    /// Time-To-Live based - good for time-sensitive data
    TTL,
    /// First In First Out - good for streaming data
    FIFO,
    /// Random eviction - good for cache poisoning resistance
    Random,
    /// Adaptive policy based on access patterns
    Adaptive,
}

/// Cache tier levels with different characteristics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheTier {
    /// L1: In-memory, fastest access (< 1ms)
    L1,
    /// L2: Compressed memory, fast access (< 10ms)
    L2,
    /// L3: Persistent storage, slower access (< 100ms)
    L3,
}

/// Access pattern analysis for intelligent caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPattern {
    /// Frequency of access
    pub frequency: f64,
    /// Temporal locality score
    pub temporal_locality: f64,
    /// Spatial locality score (related files)
    pub spatial_locality: f64,
    /// Access sequence entropy
    pub entropy: f64,
    /// Last access time
    pub last_access: DateTime<Utc>,
    /// Access count
    pub access_count: u64,
}

/// Configuration for advanced caching strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedCacheConfig {
    /// Primary eviction policy
    pub eviction_policy: EvictionPolicy,
    /// Enable multi-tier caching
    pub enable_multi_tier: bool,
    /// Enable predictive caching
    pub enable_predictive: bool,
    /// Enable collaborative caching
    pub enable_collaborative: bool,
    /// Maximum memory per tier (bytes)
    pub tier_memory_limits: FxHashMap<CacheTier, usize>,
    /// Cache warming configuration
    pub warming_config: CacheWarmingConfig,
    /// Performance tuning parameters
    pub performance_config: PerformanceConfig,
}

/// Configuration for cache warming strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheWarmingConfig {
    /// Enable automatic cache warming on startup
    pub auto_warm: bool,
    /// Maximum time to spend warming cache
    pub max_warm_time: Duration,
    /// Files to pre-load based on patterns
    pub warm_patterns: Vec<String>,
    /// Dependency-based warming (warm related files)
    pub dependency_warming: bool,
}

/// Performance tuning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable compression for L2/L3 tiers
    pub compression_enabled: bool,
    /// Compression level (1-9)
    pub compression_level: u32,
    /// Enable background cleanup
    pub background_cleanup: bool,
    /// Cleanup interval
    pub cleanup_interval: Duration,
    /// Enable cache statistics collection
    pub stats_enabled: bool,
}

impl Default for AdvancedCacheConfig {
    fn default() -> Self {
        let mut tier_limits = FxHashMap::default();
        tier_limits.insert(CacheTier::L1, 64 * 1024 * 1024); // 64MB
        tier_limits.insert(CacheTier::L2, 256 * 1024 * 1024); // 256MB
        tier_limits.insert(CacheTier::L3, 1024 * 1024 * 1024); // 1GB

        Self {
            eviction_policy: EvictionPolicy::Adaptive,
            enable_multi_tier: true,
            enable_predictive: true,
            enable_collaborative: false, // Disabled by default for security
            tier_memory_limits: tier_limits,
            warming_config: CacheWarmingConfig {
                auto_warm: true,
                max_warm_time: Duration::from_secs(30),
                warm_patterns: vec![
                    "**/*.rs".to_string(),
                    "**/Cargo.toml".to_string(),
                    "**/*.md".to_string(),
                ],
                dependency_warming: true,
            },
            performance_config: PerformanceConfig {
                compression_enabled: true,
                compression_level: 6,
                background_cleanup: true,
                cleanup_interval: Duration::from_secs(60),
                stats_enabled: true,
            },
        }
    }
}

/// Entry in the adaptive cache with rich metadata
#[derive(Debug, Clone)]
pub(crate) struct AdaptiveCacheEntry<T> {
    /// Cached value
    pub(crate) value: Arc<T>,
    /// Access pattern analysis
    pub(crate) pattern: AccessPattern,
    /// Entry size in bytes
    pub(crate) size: usize,
    /// Cache tier this entry belongs to
    pub(crate) tier: CacheTier,
    /// Entry creation time
    pub(crate) created_at: DateTime<Utc>,
    /// Entry expiration time (if TTL-based)
    pub(crate) expires_at: Option<DateTime<Utc>>,
}

impl<T> AdaptiveCacheEntry<T> {}

/// Advanced cache statistics
#[derive(Debug, Default)]
pub struct AdaptiveCacheStats {
    /// Per-tier statistics
    pub tier_stats: FxHashMap<CacheTier, TierStats>,
    /// Access pattern statistics
    pub pattern_stats: PatternStats,
    /// Performance metrics
    pub performance: PerformanceStats,
    /// Cache warming statistics
    pub warming_stats: WarmingStats,
}

/// Statistics for a specific cache tier
#[derive(Debug, Default)]
pub struct TierStats {
    /// Number of entries
    pub entry_count: usize,
    /// Total memory usage
    pub memory_usage: usize,
    /// Hit count
    pub hits: AtomicU64,
    /// Miss count
    pub misses: AtomicU64,
    /// Eviction count
    pub evictions: AtomicU64,
    /// Average access time
    pub avg_access_time: Duration,
}

/// Access pattern analysis statistics
#[derive(Debug, Default)]
pub struct PatternStats {
    /// Average frequency across all entries
    pub avg_frequency: f64,
    /// Average temporal locality
    pub avg_temporal_locality: f64,
    /// Average spatial locality
    pub avg_spatial_locality: f64,
    /// Pattern adaptation count
    pub adaptations: AtomicU64,
}

/// Performance metrics
#[derive(Debug, Default)]
pub struct PerformanceStats {
    /// Average cache lookup time
    pub avg_lookup_time: Duration,
    /// Average cache insert time
    pub avg_insert_time: Duration,
    /// Cache warming time
    pub warming_time: Duration,
    /// Compression efficiency
    pub compression_efficiency: f32,
    /// Background cleanup operations
    pub cleanup_operations: AtomicU64,
}

/// Cache warming statistics
#[derive(Debug, Default)]
pub struct WarmingStats {
    /// Files warmed during startup
    pub files_warmed: AtomicUsize,
    /// Warming success rate
    pub warming_success_rate: f64,
    /// Time spent warming
    pub total_warming_time: Duration,
    /// Predictive hits
    pub predictive_hits: AtomicU64,
}
