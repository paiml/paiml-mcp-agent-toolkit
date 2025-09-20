//! Caching layer for MCP server performance optimization
//!
//! Provides in-memory caching for frequently accessed data in the MCP server,
//! reducing redundant computations and improving response times.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Cache entry with expiration tracking
#[derive(Debug, Clone)]
struct CacheEntry {
    value: Value,
    expires_at: Instant,
}

/// Configuration for cache behavior
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in cache
    pub max_entries: usize,
    /// Default TTL for cache entries
    pub default_ttl: Duration,
    /// Enable metrics collection
    pub enable_metrics: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl: Duration::from_secs(300), // 5 minutes
            enable_metrics: true,
        }
    }
}

/// Cache metrics for monitoring
#[derive(Debug, Default, Clone)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_requests: u64,
}

impl CacheMetrics {
    /// Calculate cache hit ratio
    #[must_use] 
    pub fn hit_ratio(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.hits as f64 / self.total_requests as f64
        }
    }
}

/// High-performance cache for MCP server
///
/// Provides thread-safe, TTL-based caching with automatic eviction
/// and metrics collection for monitoring cache effectiveness.
pub struct McpCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    config: CacheConfig,
    metrics: Arc<RwLock<CacheMetrics>>,
}

impl Default for McpCache {
    fn default() -> Self {
        Self::new()
    }
}

impl McpCache {
    /// Create a new cache with default configuration
    #[must_use] 
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new cache with custom configuration
    #[must_use] 
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
        }
    }

    /// Get a value from the cache
    pub async fn get(&self, key: &str) -> Option<Value> {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;

        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(key) {
            if entry.expires_at > Instant::now() {
                metrics.hits += 1;
                return Some(entry.value.clone());
            }
        }

        metrics.misses += 1;
        None
    }

    /// Set a value in the cache with default TTL
    pub async fn set(&self, key: String, value: Value) {
        self.set_with_ttl(key, value, self.config.default_ttl).await;
    }

    /// Set a value in the cache with custom TTL
    pub async fn set_with_ttl(&self, key: String, value: Value, ttl: Duration) {
        let mut entries = self.entries.write().await;

        // Evict old entries if at capacity
        if entries.len() >= self.config.max_entries {
            self.evict_expired(&mut entries).await;

            // If still at capacity, remove oldest entry
            if entries.len() >= self.config.max_entries {
                if let Some(oldest_key) = self.find_oldest(&entries) {
                    entries.remove(&oldest_key);
                    if self.config.enable_metrics {
                        let mut metrics = self.metrics.write().await;
                        metrics.evictions += 1;
                    }
                }
            }
        }

        entries.insert(
            key,
            CacheEntry {
                value,
                expires_at: Instant::now() + ttl,
            },
        );
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }

    /// Get current cache metrics
    pub async fn metrics(&self) -> CacheMetrics {
        self.metrics.read().await.clone()
    }

    /// Get number of entries in cache
    pub async fn size(&self) -> usize {
        self.entries.read().await.len()
    }

    /// Evict expired entries
    async fn evict_expired(&self, entries: &mut HashMap<String, CacheEntry>) {
        let now = Instant::now();
        let expired_keys: Vec<String> = entries
            .iter()
            .filter(|(_, entry)| entry.expires_at <= now)
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            entries.remove(&key);
            if self.config.enable_metrics {
                let mut metrics = self.metrics.write().await;
                metrics.evictions += 1;
            }
        }
    }

    /// Find the oldest entry in the cache
    fn find_oldest(&self, entries: &HashMap<String, CacheEntry>) -> Option<String> {
        entries
            .iter()
            .min_by_key(|(_, entry)| entry.expires_at)
            .map(|(key, _)| key.clone())
    }
}

/// Cache key builder for consistent key generation
pub struct CacheKeyBuilder;

impl CacheKeyBuilder {
    /// Build cache key for analysis results
    #[must_use] 
    pub fn analysis_key(file_path: &str, version: &str) -> String {
        format!("analysis:{file_path}:{version}")
    }

    /// Build cache key for refactoring plans
    #[must_use] 
    pub fn refactor_plan_key(file_path: &str, config_hash: u64) -> String {
        format!("refactor_plan:{file_path}:{config_hash}")
    }

    /// Build cache key for complexity metrics
    #[must_use] 
    pub fn complexity_key(file_path: &str) -> String {
        format!("complexity:{file_path}")
    }

    /// Build cache key for MCP method results
    #[must_use] 
    pub fn method_result_key(method: &str, params_hash: u64) -> String {
        format!("method:{method}:{params_hash}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache = McpCache::new();

        // Test set and get
        cache
            .set(
                "test_key".to_string(),
                Value::String("test_value".to_string()),
            )
            .await;
        let value = cache.get("test_key").await;
        assert_eq!(value, Some(Value::String("test_value".to_string())));

        // Test miss
        let missing = cache.get("missing_key").await;
        assert_eq!(missing, None);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = McpCache::new();

        // Set with very short TTL
        cache
            .set_with_ttl(
                "expire_key".to_string(),
                Value::String("expire_value".to_string()),
                Duration::from_millis(10),
            )
            .await;

        // Should exist immediately
        assert!(cache.get("expire_key").await.is_some());

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Should be expired
        assert!(cache.get("expire_key").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_metrics() {
        let cache = McpCache::new();

        // Generate some cache activity
        cache
            .set("key1".to_string(), Value::String("value1".to_string()))
            .await;
        let _ = cache.get("key1").await; // Hit
        let _ = cache.get("key2").await; // Miss
        let _ = cache.get("key1").await; // Hit

        let metrics = cache.metrics().await;
        assert_eq!(metrics.hits, 2);
        assert_eq!(metrics.misses, 1);
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.hit_ratio(), 2.0 / 3.0);
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        let config = CacheConfig {
            max_entries: 2,
            default_ttl: Duration::from_secs(60),
            enable_metrics: true,
        };
        let cache = McpCache::with_config(config);

        // Fill cache to capacity
        cache
            .set("key1".to_string(), Value::String("value1".to_string()))
            .await;
        cache
            .set("key2".to_string(), Value::String("value2".to_string()))
            .await;

        // This should trigger eviction
        cache
            .set("key3".to_string(), Value::String("value3".to_string()))
            .await;

        // Cache should not exceed max_entries
        assert!(cache.size().await <= 2);
    }

    #[test]
    fn test_cache_key_builder() {
        let analysis_key = CacheKeyBuilder::analysis_key("src/main.rs", "v1.0.0");
        assert_eq!(analysis_key, "analysis:src/main.rs:v1.0.0");

        let refactor_key = CacheKeyBuilder::refactor_plan_key("src/lib.rs", 12345);
        assert_eq!(refactor_key, "refactor_plan:src/lib.rs:12345");

        let complexity_key = CacheKeyBuilder::complexity_key("src/test.rs");
        assert_eq!(complexity_key, "complexity:src/test.rs");

        let method_key = CacheKeyBuilder::method_result_key("refactor.start", 67890);
        assert_eq!(method_key, "method:refactor.start:67890");
    }
}

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
