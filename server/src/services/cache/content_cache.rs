use crate::services::cache::base::{CacheEntry, CacheStats, CacheStrategy};
use lru::LruCache;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;

/// Content-based cache with LRU eviction
pub struct ContentCache<T: CacheStrategy> {
    /// Primary storage with LRU eviction
    cache: Arc<RwLock<LruCache<String, CacheEntry<T::Value>>>>,

    /// Content validation hashes
    hashes: Arc<RwLock<HashMap<String, u64>>>,

    /// Statistics
    pub stats: CacheStats,

    /// Strategy
    strategy: Arc<T>,
}

impl<T: CacheStrategy> ContentCache<T> {
    pub fn new(strategy: T) -> Self {
        let max_size =
            NonZeroUsize::new(strategy.max_size()).unwrap_or(NonZeroUsize::new(100).unwrap());

        Self {
            cache: Arc::new(RwLock::new(LruCache::new(max_size))),
            hashes: Arc::new(RwLock::new(HashMap::new())),
            stats: CacheStats::new(),
            strategy: Arc::new(strategy),
        }
    }

    /// Get a value from the cache
    pub fn get(&self, key: &T::Key) -> Option<Arc<T::Value>> {
        let cache_key = self.strategy.cache_key(key);

        let mut cache = self.cache.write();

        if let Some(entry) = cache.get_mut(&cache_key) {
            // Check if TTL is expired
            if let Some(ttl) = self.strategy.ttl() {
                if entry.age() > ttl {
                    // Expired, remove it
                    self.stats.remove_bytes(entry.size_bytes);
                    cache.pop(&cache_key);
                    self.stats.record_miss();
                    return None;
                }
            }

            // Validate the entry
            if self.strategy.validate(key, &entry.value) {
                entry.access();
                self.stats.record_hit();
                Some(entry.value.clone())
            } else {
                // Invalid, remove it
                self.stats.remove_bytes(entry.size_bytes);
                cache.pop(&cache_key);
                self.stats.record_miss();
                None
            }
        } else {
            self.stats.record_miss();
            None
        }
    }

    /// Put a value into the cache
    pub fn put(&self, key: T::Key, value: T::Value) {
        let cache_key = self.strategy.cache_key(&key);
        let size_bytes = self.estimate_size(&value);

        let entry = CacheEntry::new(value, size_bytes);

        let mut cache = self.cache.write();

        // Handle eviction if we're at capacity
        if let Some((_key, evicted)) = cache.push(cache_key.clone(), entry) {
            self.stats.remove_bytes(evicted.size_bytes);
            self.stats.record_eviction();
        }

        self.stats.add_bytes(size_bytes);

        // Store content hash for validation
        let mut hasher = DefaultHasher::new();
        cache_key.hash(&mut hasher);
        let hash = hasher.finish();
        self.hashes.write().insert(cache_key, hash);
    }

    /// Remove a specific entry from the cache
    pub fn remove(&self, key: &T::Key) -> Option<Arc<T::Value>> {
        let cache_key = self.strategy.cache_key(key);
        let mut cache = self.cache.write();

        if let Some(entry) = cache.pop(&cache_key) {
            self.stats.remove_bytes(entry.size_bytes);
            self.hashes.write().remove(&cache_key);
            Some(entry.value.clone())
        } else {
            None
        }
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        let mut cache = self.cache.write();

        // Update stats
        for (_, entry) in cache.iter() {
            self.stats.remove_bytes(entry.size_bytes);
        }

        cache.clear();
        self.hashes.write().clear();
    }

    /// Evict the least recently used entry
    pub fn evict_lru(&self) {
        let mut cache = self.cache.write();

        if let Some((_, entry)) = cache.pop_lru() {
            self.stats.remove_bytes(entry.size_bytes);
            self.stats.record_eviction();
        }
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        self.cache.read().len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.read().is_empty()
    }

    /// Estimate the size of a value in bytes
    fn estimate_size(&self, value: &T::Value) -> usize {
        // This is a rough estimate. In a real implementation,
        // we'd use a more accurate size calculation
        std::mem::size_of_val(value) * 2
    }

    /// Get cache metrics
    pub fn metrics(&self) -> CacheMetrics {
        let cache = self.cache.read();

        CacheMetrics {
            entries: cache.len(),
            memory_bytes: self.stats.memory_usage(),
            hit_rate: self.stats.hit_rate(),
            total_requests: self.stats.total_requests(),
            evictions: self
                .stats
                .evictions
                .load(std::sync::atomic::Ordering::Relaxed),
        }
    }

    /// Get hot entries (most frequently accessed)
    pub fn hot_entries(&self, limit: usize) -> Vec<(String, u32)> {
        let cache = self.cache.read();

        let mut entries: Vec<(String, u32)> = cache
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.access_count.load(std::sync::atomic::Ordering::Relaxed),
                )
            })
            .collect();

        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.truncate(limit);
        entries
    }

    /// Invalidate entries matching a predicate
    pub fn invalidate_matching<F>(&self, predicate: F)
    where
        F: Fn(&str) -> bool,
    {
        let mut cache = self.cache.write();
        let keys_to_remove: Vec<String> = cache
            .iter()
            .filter(|(k, _)| predicate(k))
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_remove {
            if let Some(entry) = cache.pop(&key) {
                self.stats.remove_bytes(entry.size_bytes);
                self.stats.record_eviction();
            }
        }
    }
}

/// Cache metrics for monitoring
#[derive(Debug, Clone)]
pub struct CacheMetrics {
    pub entries: usize,
    pub memory_bytes: usize,
    pub hit_rate: f64,
    pub total_requests: u64,
    pub evictions: u64,
}

impl CacheMetrics {
    pub fn memory_mb(&self) -> f64 {
        self.memory_bytes as f64 / (1024.0 * 1024.0)
    }
}

// Implement Clone for ContentCache
impl<T: CacheStrategy> Clone for ContentCache<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        // Note: This creates a new empty cache with the same strategy
        // Actual cache contents are not cloned
        Self::new((*self.strategy).clone())
    }
}
