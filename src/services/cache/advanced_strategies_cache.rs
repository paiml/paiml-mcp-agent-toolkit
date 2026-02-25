#![cfg_attr(coverage_nightly, coverage(off))]
//! AdaptiveCache implementation
//!
//! Multi-tier adaptive cache with intelligent tier promotion and eviction.
//!
//! Split into submodules via `include!()`:
//! - `advanced_strategies_cache_api.rs` — public API (new, get, put, remove, clear, stats, warming, maintenance)
//! - `advanced_strategies_cache_eviction.rs` — tier helpers, insertion, eviction strategies
//! - `advanced_strategies_cache_maintenance.rs` — stats recording, pattern updates, cleanup

use super::advanced_strategies_types::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

/// Entry in the adaptive cache with rich metadata
#[derive(Debug, Clone)]
pub(crate) struct AdaptiveCacheEntry<T> {
    /// Cached value
    pub value: Arc<T>,
    /// Access pattern analysis
    pub pattern: AccessPattern,
    /// Entry size in bytes
    pub size: usize,
    /// Cache tier this entry belongs to
    pub tier: CacheTier,
    /// Entry creation time
    pub created_at: DateTime<Utc>,
    /// Entry expiration time (if TTL-based)
    pub expires_at: Option<DateTime<Utc>>,
}

impl<T> AdaptiveCacheEntry<T> {}

/// Multi-tier adaptive cache implementation
pub struct AdaptiveCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Cache configuration
    pub(crate) config: AdvancedCacheConfig,
    /// L1 cache (fastest)
    pub(crate) l1_cache: Arc<RwLock<FxHashMap<K, AdaptiveCacheEntry<V>>>>,
    /// L2 cache (compressed)
    pub(crate) l2_cache: Arc<RwLock<FxHashMap<K, AdaptiveCacheEntry<V>>>>,
    /// L3 cache (persistent)
    pub(crate) l3_cache: Arc<RwLock<FxHashMap<K, AdaptiveCacheEntry<V>>>>,
    /// Access pattern tracker
    pub(crate) access_patterns: Arc<RwLock<FxHashMap<K, AccessPattern>>>,
    /// Cache statistics
    pub(crate) stats: Arc<RwLock<AdaptiveCacheStats>>,
    /// Predictive cache warmer
    pub(crate) predictor: Arc<super::advanced_strategies_predictor::CachePredictor<K>>,
}

// Public API: new, get, put, remove, clear, get_stats, warm_cache, background_maintenance
include!("advanced_strategies_cache_api.rs");

// Tier helpers, insertion, and eviction strategies
include!("advanced_strategies_cache_eviction.rs");

// Stats recording, pattern updates, cleanup, and optimization
include!("advanced_strategies_cache_maintenance.rs");
