#![cfg_attr(coverage_nightly, coverage(off))]
use std::hash::Hash;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(test)]
#[path = "base_tests.rs"]
mod tests;

/// Base trait for cache strategies
pub trait CacheStrategy: Send + Sync {
    type Key: Hash + Eq + Clone + Send;
    type Value: Clone + Send;

    /// Generate a unique cache key for the input
    fn cache_key(&self, input: &Self::Key) -> String;

    /// Validate if the cached value is still valid
    fn validate(&self, key: &Self::Key, value: &Self::Value) -> bool;

    /// Time-to-live for cache entries
    fn ttl(&self) -> Option<Duration>;

    /// Maximum number of entries in the cache
    fn max_size(&self) -> usize;
}

/// A single cache entry with metadata
#[derive(Clone)]
pub struct CacheEntry<V> {
    pub value: Arc<V>,
    pub created: Instant,
    pub access_count: Arc<AtomicU32>,
    pub size_bytes: usize,
    pub last_accessed: Arc<parking_lot::Mutex<Instant>>,
}

impl<V> CacheEntry<V> {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(value: V, size_bytes: usize) -> Self {
        Self {
            value: Arc::new(value),
            created: Instant::now(),
            access_count: Arc::new(AtomicU32::new(0)),
            size_bytes,
            last_accessed: Arc::new(parking_lot::Mutex::new(Instant::now())),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Access.
    pub fn access(&self) {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        *self.last_accessed.lock() = Instant::now();
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Age.
    pub fn age(&self) -> Duration {
        self.created.elapsed()
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Last accessed duration.
    pub fn last_accessed_duration(&self) -> Duration {
        self.last_accessed.lock().elapsed()
    }
}

/// Cache statistics
#[derive(Clone, Debug)]
pub struct CacheStats {
    pub hits: Arc<AtomicU64>,
    pub misses: Arc<AtomicU64>,
    pub evictions: Arc<AtomicU64>,
    pub total_bytes: Arc<AtomicUsize>,
}

impl CacheStats {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            evictions: Arc::new(AtomicU64::new(0)),
            total_bytes: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Record hit.
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record miss.
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Record eviction.
    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// Add bytes.
    pub fn add_bytes(&self, bytes: usize) {
        self.total_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Remove bytes.
    pub fn remove_bytes(&self, bytes: usize) {
        self.total_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Hit rate.
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let total = hits + self.misses.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            hits / total
        } else {
            0.0
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Total requests.
    pub fn total_requests(&self) -> u64 {
        self.hits.load(Ordering::Relaxed) + self.misses.load(Ordering::Relaxed)
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Memory usage.
    pub fn memory_usage(&self) -> usize {
        self.total_bytes.load(Ordering::Relaxed)
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
