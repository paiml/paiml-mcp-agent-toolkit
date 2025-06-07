use crate::services::cache::base::CacheStats;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

/// Unified cache trait that all cache implementations must implement
/// This trait consolidates the fragmented cache patterns in the codebase
#[async_trait]
pub trait UnifiedCache: Send + Sync {
    /// The key type for this cache
    type Key: Hash + Eq + Clone + Send + Sync;
    /// The value type for this cache
    type Value: Clone + Send + Sync;

    /// Get a value from the cache
    async fn get(&self, key: &Self::Key) -> Option<Arc<Self::Value>>;

    /// Put a value into the cache
    async fn put(&self, key: Self::Key, value: Self::Value) -> Result<()>;

    /// Get or compute a value with caching
    async fn get_or_compute<F, Fut>(&self, key: Self::Key, compute: F) -> Result<Arc<Self::Value>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<Self::Value>> + Send,
    {
        // Check cache first
        if let Some(cached) = self.get(&key).await {
            return Ok(cached);
        }

        // Compute and cache
        let value = compute().await?;
        self.put(key.clone(), value.clone()).await?;
        Ok(Arc::new(value))
    }

    /// Remove a value from the cache
    async fn remove(&self, key: &Self::Key) -> Option<Arc<Self::Value>>;

    /// Clear all entries from the cache
    async fn clear(&self) -> Result<()>;

    /// Get cache statistics
    fn stats(&self) -> Arc<CacheStats>;

    /// Get the current size of the cache in bytes
    fn size_bytes(&self) -> usize;

    /// Get the number of entries in the cache
    fn len(&self) -> usize;

    /// Check if the cache is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Evict entries if necessary based on cache policy
    async fn evict_if_needed(&self) -> Result<()>;
}

/// SIMD-aware cache key for vectorized operations
#[derive(Hash, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct VectorizedCacheKey {
    /// High 64 bits of the hash
    pub hash_high: u64,
    /// Low 64 bits of the hash
    pub hash_low: u64,
}

impl VectorizedCacheKey {
    /// Create a new vectorized cache key
    pub fn new(hash_high: u64, hash_low: u64) -> Self {
        Self {
            hash_high,
            hash_low,
        }
    }

    /// Create from a byte slice using SIMD-friendly hashing
    pub fn from_bytes(data: &[u8]) -> Self {
        use std::collections::hash_map::DefaultHasher;

        // Split data for parallel hashing
        let mid = data.len() / 2;
        let (left, right) = data.split_at(mid);

        let mut hasher1 = DefaultHasher::new();
        hasher1.write(left);
        let hash_high = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        hasher2.write(right);
        hasher2.write_u64(hash_high); // Mix in the first hash
        let hash_low = hasher2.finish();

        Self {
            hash_high,
            hash_low,
        }
    }

    /// Create from a string
    pub fn from_string(s: &str) -> Self {
        Self::from_bytes(s.as_bytes())
    }
}

/// Configuration for unified cache implementations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnifiedCacheConfig {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// TTL for cache entries
    pub ttl: Option<Duration>,
    /// Maximum number of entries
    pub max_entries: usize,
    /// Enable persistent storage
    pub persistent: bool,
    /// Persistent storage path
    pub storage_path: Option<String>,
    /// Eviction batch size
    pub eviction_batch_size: usize,
}

impl Default for UnifiedCacheConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 512 * 1024 * 1024, // 512MB
            ttl: Some(Duration::from_secs(300)), // 5 minutes
            max_entries: 10000,
            persistent: false,
            storage_path: None,
            eviction_batch_size: 100,
        }
    }
}

/// Layered cache implementation that combines memory and persistent storage
/// Uses generic types instead of trait objects to avoid dyn-compatibility issues
#[derive(Clone)]
pub struct LayeredCache<M, P = M>
where
    M: UnifiedCache,
    P: UnifiedCache<Key = M::Key, Value = M::Value>,
{
    /// In-memory cache layer
    memory: M,
    /// Optional persistent cache layer
    persistent: Option<P>,
    /// Statistics
    stats: Arc<CacheStats>,
}

impl<M, P> LayeredCache<M, P>
where
    M: UnifiedCache,
    P: UnifiedCache<Key = M::Key, Value = M::Value>,
{
    /// Create a new layered cache
    pub fn new(memory: M, persistent: Option<P>) -> Self {
        Self {
            memory,
            persistent,
            stats: Arc::new(CacheStats::new()),
        }
    }
}

#[async_trait]
impl<M, P> UnifiedCache for LayeredCache<M, P>
where
    M: UnifiedCache + Clone,
    P: UnifiedCache<Key = M::Key, Value = M::Value> + Clone,
    M::Key: 'static,
    M::Value: 'static,
{
    type Key = M::Key;
    type Value = M::Value;

    async fn get(&self, key: &Self::Key) -> Option<Arc<Self::Value>> {
        // Try memory first
        if let Some(value) = self.memory.get(key).await {
            self.stats.record_hit();
            return Some(value);
        }

        // Try persistent storage
        if let Some(ref persistent) = self.persistent {
            if let Some(value) = persistent.get(key).await {
                // Promote to memory cache
                let _ = self.memory.put(key.clone(), (*value).clone()).await;
                self.stats.record_hit();
                return Some(value);
            }
        }

        self.stats.record_miss();
        None
    }

    async fn put(&self, key: Self::Key, value: Self::Value) -> Result<()> {
        // Write to memory
        self.memory.put(key.clone(), value.clone()).await?;

        // Write to persistent if available
        if let Some(ref persistent) = self.persistent {
            persistent.put(key, value).await?;
        }

        Ok(())
    }

    async fn remove(&self, key: &Self::Key) -> Option<Arc<Self::Value>> {
        let memory_result = self.memory.remove(key).await;

        if let Some(ref persistent) = self.persistent {
            persistent.remove(key).await;
        }

        memory_result
    }

    async fn clear(&self) -> Result<()> {
        self.memory.clear().await?;

        if let Some(ref persistent) = self.persistent {
            persistent.clear().await?;
        }

        Ok(())
    }

    fn stats(&self) -> Arc<CacheStats> {
        self.stats.clone()
    }

    fn size_bytes(&self) -> usize {
        let mut size = self.memory.size_bytes();

        if let Some(ref persistent) = self.persistent {
            size += persistent.size_bytes();
        }

        size
    }

    fn len(&self) -> usize {
        self.memory.len()
    }

    async fn evict_if_needed(&self) -> Result<()> {
        self.memory.evict_if_needed().await?;

        if let Some(ref persistent) = self.persistent {
            persistent.evict_if_needed().await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vectorized_cache_key() {
        let key1 = VectorizedCacheKey::from_string("test_key");
        let key2 = VectorizedCacheKey::from_string("test_key");
        let key3 = VectorizedCacheKey::from_string("different_key");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_vectorized_cache_key_from_bytes() {
        let data = b"some binary data";
        let key1 = VectorizedCacheKey::from_bytes(data);
        let key2 = VectorizedCacheKey::from_bytes(data);

        assert_eq!(key1, key2);
    }
}
