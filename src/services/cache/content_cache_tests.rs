use super::*;
use crate::services::cache::base::{CacheKey, HashCacheStrategy};
use crate::services::cache::config::CacheConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct TestKey(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestValue(String);

impl CacheKey for TestKey {
    fn size_bytes(&self) -> usize {
        self.0.len()
    }
}

#[test]
fn test_content_cache_creation() {
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let cache = ContentCache::new(strategy);
    
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.stats.hits, 0);
    assert_eq!(cache.stats.misses, 0);
}

#[test]
fn test_content_cache_get_put() {
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let cache = ContentCache::new(strategy);
    
    let key = TestKey("key1".to_string());
    let value = TestValue("value1".to_string());
    
    // Initially empty
    assert!(cache.get(&key).is_none());
    assert_eq!(cache.stats.misses, 1);
    
    // Put a value
    cache.put(key.clone(), value.clone());
    assert_eq!(cache.len(), 1);
    
    // Get the value
    let retrieved = cache.get(&key);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().0, "value1");
    assert_eq!(cache.stats.hits, 1);
}

#[test]
fn test_content_cache_remove() {
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let cache = ContentCache::new(strategy);
    
    let key = TestKey("key1".to_string());
    let value = TestValue("value1".to_string());
    
    cache.put(key.clone(), value);
    assert_eq!(cache.len(), 1);
    
    // Remove the value
    let removed = cache.remove(&key);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().0, "value1");
    assert_eq!(cache.len(), 0);
    
    // Try to get removed value
    assert!(cache.get(&key).is_none());
}

#[test]
fn test_content_cache_clear() {
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let cache = ContentCache::new(strategy);
    
    // Add multiple values
    for i in 0..5 {
        let key = TestKey(format!("key{}", i));
        let value = TestValue(format!("value{}", i));
        cache.put(key, value);
    }
    
    assert_eq!(cache.len(), 5);
    
    // Clear the cache
    cache.clear();
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.stats.memory_usage(), 0);
}

#[test]
fn test_content_cache_lru_eviction() {
    let mut strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    strategy.config.max_items = 3;
    let cache = ContentCache::new(strategy);
    
    // Fill cache to capacity
    for i in 0..3 {
        let key = TestKey(format!("key{}", i));
        let value = TestValue(format!("value{}", i));
        cache.put(key, value);
    }
    
    assert_eq!(cache.len(), 3);
    
    // Add one more item, should evict oldest
    let key = TestKey("key3".to_string());
    let value = TestValue("value3".to_string());
    cache.put(key, value);
    
    // Still have 3 items
    assert_eq!(cache.len(), 3);
    
    // First item should be evicted
    assert!(cache.get(&TestKey("key0".to_string())).is_none());
    assert!(cache.get(&TestKey("key3".to_string())).is_some());
}

#[test]
fn test_content_cache_ttl_expiration() {
    let mut strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    strategy.config.ttl_seconds = Some(0); // Expire immediately
    let cache = ContentCache::new(strategy);
    
    let key = TestKey("key1".to_string());
    let value = TestValue("value1".to_string());
    
    cache.put(key.clone(), value);
    
    // Sleep to ensure TTL expires
    std::thread::sleep(Duration::from_millis(1));
    
    // Should be expired
    assert!(cache.get(&key).is_none());
    assert_eq!(cache.stats.misses, 1);
}

#[test]
fn test_content_cache_update_existing() {
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let cache = ContentCache::new(strategy);
    
    let key = TestKey("key1".to_string());
    let value1 = TestValue("value1".to_string());
    let value2 = TestValue("value2".to_string());
    
    // Put initial value
    cache.put(key.clone(), value1);
    assert_eq!(cache.get(&key).unwrap().0, "value1");
    
    // Update with new value
    cache.put(key.clone(), value2);
    assert_eq!(cache.get(&key).unwrap().0, "value2");
    assert_eq!(cache.len(), 1); // Still only one entry
}

#[test]
fn test_content_cache_metrics() {
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let cache = ContentCache::new(strategy);
    
    // Add some items
    for i in 0..3 {
        let key = TestKey(format!("key{}", i));
        let value = TestValue(format!("value{}", i));
        cache.put(key, value);
    }
    
    let metrics = cache.get_metrics();
    assert_eq!(metrics.entries, 3);
    assert!(metrics.memory_bytes > 0);
    assert_eq!(metrics.total_requests, 0); // No gets yet
    assert_eq!(metrics.evictions, 0);
    
    // Make some requests
    cache.get(&TestKey("key0".to_string()));
    cache.get(&TestKey("key_missing".to_string()));
    
    let metrics = cache.get_metrics();
    assert_eq!(metrics.total_requests, 2);
    assert_eq!(metrics.hit_rate, 0.5); // 1 hit, 1 miss
}

#[test]
fn test_content_cache_invalidate_matching() {
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let cache = ContentCache::new(strategy);
    
    // Add items with pattern
    cache.put(TestKey("user:1".to_string()), TestValue("Alice".to_string()));
    cache.put(TestKey("user:2".to_string()), TestValue("Bob".to_string()));
    cache.put(TestKey("product:1".to_string()), TestValue("Widget".to_string()));
    
    assert_eq!(cache.len(), 3);
    
    // Invalidate all user entries
    cache.invalidate_matching(|k| k.starts_with("user:"));
    
    assert_eq!(cache.len(), 1);
    assert!(cache.get(&TestKey("user:1".to_string())).is_none());
    assert!(cache.get(&TestKey("user:2".to_string())).is_none());
    assert!(cache.get(&TestKey("product:1".to_string())).is_some());
}

#[test]
fn test_content_cache_memory_estimation() {
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let cache = ContentCache::new(strategy);
    
    let key = TestKey("key1".to_string());
    let value = TestValue("a".repeat(1000)); // 1KB value
    
    cache.put(key, value);
    
    let metrics = cache.get_metrics();
    assert!(metrics.memory_bytes >= 1000); // At least the value size
    assert!(metrics.memory_mb() > 0.0);
}

#[test]
fn test_content_cache_clone() {
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let cache = ContentCache::new(strategy);
    
    // Add some data
    cache.put(TestKey("key1".to_string()), TestValue("value1".to_string()));
    
    // Clone creates empty cache with same strategy
    let cloned = cache.clone();
    assert_eq!(cloned.len(), 0);
    assert_eq!(cloned.stats.hits, 0);
}

#[test]
fn test_cache_metrics_creation() {
    let metrics = CacheMetrics {
        entries: 100,
        memory_bytes: 1048576, // 1MB
        hit_rate: 0.85,
        total_requests: 1000,
        evictions: 50,
    };
    
    assert_eq!(metrics.entries, 100);
    assert_eq!(metrics.memory_mb(), 1.0);
    assert_eq!(metrics.hit_rate, 0.85);
}