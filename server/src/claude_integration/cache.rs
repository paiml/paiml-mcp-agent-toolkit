// Two-tier cache implementation for Claude integration
// L1: In-memory cache with short TTL, L2: Persistent cache

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

/// Analysis result that can be cached
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub complexity: u32,
    pub cognitive_complexity: u32,
    pub satd_count: usize,
    pub timestamp: SystemTime,
    pub content_hash: u64,
}

impl Default for AnalysisResult {
    fn default() -> Self {
        Self {
            complexity: 0,
            cognitive_complexity: 0,
            satd_count: 0,
            timestamp: SystemTime::now(),
            content_hash: 0,
        }
    }
}

/// Cache entry with TTL
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: Arc<T>,
    expires_at: SystemTime,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value: Arc::new(value),
            expires_at: SystemTime::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }
}

/// Two-tier cache with L1 (memory) and L2 (persistent)
pub struct TwoTierCache {
    /// L1: In-process cache with 10ms TTL
    l1: Arc<RwLock<HashMap<u64, CacheEntry<AnalysisResult>>>>,

    /// L2: Memory-mapped cache with 60s TTL
    l2: Arc<RwLock<HashMap<u64, CacheEntry<AnalysisResult>>>>,

    /// L1 cache TTL
    l1_ttl: Duration,

    /// L2 cache TTL
    l2_ttl: Duration,

    /// Metrics
    l1_hits: AtomicU64,
    l1_misses: AtomicU64,
    l2_hits: AtomicU64,
    l2_misses: AtomicU64,
}

impl Default for TwoTierCache {
    fn default() -> Self {
        Self::new()
    }
}

impl TwoTierCache {
    pub fn new() -> Self {
        Self {
            l1: Arc::new(RwLock::new(HashMap::new())),
            l2: Arc::new(RwLock::new(HashMap::new())),
            l1_ttl: Duration::from_millis(10),
            l2_ttl: Duration::from_secs(60),
            l1_hits: AtomicU64::new(0),
            l1_misses: AtomicU64::new(0),
            l2_hits: AtomicU64::new(0),
            l2_misses: AtomicU64::new(0),
        }
    }

    /// Get value from cache or load it
    pub async fn get_with_loader<F, Fut>(&self, key: &str, loader: F) -> Arc<AnalysisResult>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = AnalysisResult>,
    {
        let hash = self.hash_key(key);

        // L1 lookup - ~100ns
        {
            let l1_guard = self.l1.read().await;
            if let Some(entry) = l1_guard.get(&hash) {
                if !entry.is_expired() {
                    self.l1_hits.fetch_add(1, Ordering::Relaxed);
                    return Arc::clone(&entry.value);
                }
            }
        }
        self.l1_misses.fetch_add(1, Ordering::Relaxed);

        // L2 lookup - ~1μs
        {
            let l2_guard = self.l2.read().await;
            if let Some(entry) = l2_guard.get(&hash) {
                if !entry.is_expired() {
                    self.l2_hits.fetch_add(1, Ordering::Relaxed);
                    let value = Arc::clone(&entry.value);

                    // Promote to L1
                    drop(l2_guard);
                    let mut l1_guard = self.l1.write().await;
                    l1_guard.insert(hash, CacheEntry::new((*value).clone(), self.l1_ttl));

                    return value;
                }
            }
        }
        self.l2_misses.fetch_add(1, Ordering::Relaxed);

        // Load from source - ~15ms
        let result = Arc::new(loader().await);

        // Populate both caches
        {
            let mut l1_guard = self.l1.write().await;
            l1_guard.insert(hash, CacheEntry::new((*result).clone(), self.l1_ttl));
        }

        {
            let mut l2_guard = self.l2.write().await;
            l2_guard.insert(hash, CacheEntry::new((*result).clone(), self.l2_ttl));
        }

        result
    }

    /// Hash key using FNV-1a
    #[inline(always)]
    pub fn hash_key(&self, key: &str) -> u64 {
        let mut hasher = FnvHasher::default();
        hasher.write(key.as_bytes());
        hasher.finish()
    }

    /// Get cache hit rate metrics
    pub fn hit_rate(&self) -> CacheMetrics {
        let l1_total =
            self.l1_hits.load(Ordering::Relaxed) + self.l1_misses.load(Ordering::Relaxed);
        let l2_total =
            self.l2_hits.load(Ordering::Relaxed) + self.l2_misses.load(Ordering::Relaxed);

        let l1_hit_rate = if l1_total > 0 {
            self.l1_hits.load(Ordering::Relaxed) as f64 / l1_total as f64
        } else {
            0.0
        };

        let l2_hit_rate = if l2_total > 0 {
            self.l2_hits.load(Ordering::Relaxed) as f64 / l2_total as f64
        } else {
            0.0
        };

        let effective_total = l1_total;
        let effective_hits =
            self.l1_hits.load(Ordering::Relaxed) + self.l2_hits.load(Ordering::Relaxed);
        let effective_hit_rate = if effective_total > 0 {
            effective_hits as f64 / effective_total as f64
        } else {
            0.0
        };

        CacheMetrics {
            l1_hit_rate,
            l2_hit_rate,
            effective_hit_rate,
            l1_size: 0, // Would need to track size
            l2_size: 0,
        }
    }

    /// Clear all caches
    pub async fn clear(&self) {
        self.l1.write().await.clear();
        self.l2.write().await.clear();
    }

    /// Evict expired entries
    pub async fn evict_expired(&self) {
        {
            let mut l1_guard = self.l1.write().await;
            l1_guard.retain(|_, entry| !entry.is_expired());
        }

        {
            let mut l2_guard = self.l2.write().await;
            l2_guard.retain(|_, entry| !entry.is_expired());
        }
    }
}

/// FNV-1a hasher for fast hashing
#[derive(Default)]
struct FnvHasher {
    state: u64,
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        const FNV_PRIME: u64 = 0x100000001b3;
        const FNV_OFFSET: u64 = 0xcbf29ce484222325;

        self.state = FNV_OFFSET;
        for &byte in bytes {
            self.state ^= u64::from(byte);
            self.state = self.state.wrapping_mul(FNV_PRIME);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CacheMetrics {
    pub l1_hit_rate: f64,
    pub l2_hit_rate: f64,
    pub effective_hit_rate: f64,
    pub l1_size: usize,
    pub l2_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_creation() {
        let cache = TwoTierCache::new();
        let metrics = cache.hit_rate();
        assert_eq!(metrics.l1_hit_rate, 0.0);
        assert_eq!(metrics.l2_hit_rate, 0.0);
    }

    #[tokio::test]
    async fn test_cache_l1_hit() {
        let cache = TwoTierCache::new();

        // First access - miss
        let result = cache
            .get_with_loader("test_key", || async {
                AnalysisResult {
                    complexity: 10,
                    ..Default::default()
                }
            })
            .await;

        assert_eq!(result.complexity, 10);

        // Second access - L1 hit
        let result2 = cache
            .get_with_loader("test_key", || async {
                AnalysisResult {
                    complexity: 20,
                    ..Default::default()
                }
            })
            .await;

        // Should get cached value
        assert_eq!(result2.complexity, 10);

        let metrics = cache.hit_rate();
        assert!(metrics.l1_hit_rate > 0.0);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = TwoTierCache::new();

        // First load - miss
        cache
            .get_with_loader("test_key", || async { AnalysisResult::default() })
            .await;

        // Second access - hit
        cache
            .get_with_loader("test_key", || async { AnalysisResult::default() })
            .await;

        let metrics_before = cache.hit_rate();
        assert!(metrics_before.effective_hit_rate > 0.0);

        // Clear cache
        cache.clear().await;

        // After clear, metrics are not reset (they track historical performance)
        // But cache entries are gone
        let metrics_after = cache.hit_rate();
        assert_eq!(
            metrics_after.effective_hit_rate,
            metrics_before.effective_hit_rate
        );
    }

    #[test]
    fn test_fnv_hasher() {
        let mut hasher = FnvHasher::default();
        hasher.write(b"test");
        let hash1 = hasher.finish();

        let mut hasher2 = FnvHasher::default();
        hasher2.write(b"test");
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }
}
