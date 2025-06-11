use super::*;
use crate::services::cache::base::{HashCacheStrategy, CacheKey};
use crate::services::cache::config::CacheConfig;
use tempfile::TempDir;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct TestKey(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestValue(String);

impl CacheKey for TestKey {
    fn size_bytes(&self) -> usize {
        self.0.len()
    }
}

#[tokio::test]
async fn test_content_cache_adapter_creation() {
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let config = CacheConfig::default();
    let cache = ContentCache::new(strategy, config);
    let adapter = ContentCacheAdapter::new(cache);
    
    assert_eq!(adapter.len(), 0);
    assert_eq!(adapter.size_bytes(), 0);
}

#[tokio::test]
async fn test_content_cache_adapter_get_put() {
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let config = CacheConfig::default();
    let cache = ContentCache::new(strategy, config);
    let adapter = ContentCacheAdapter::new(cache);
    
    // Put a value
    let key = TestKey("key1".to_string());
    let value = TestValue("value1".to_string());
    adapter.put(key.clone(), value.clone()).await.unwrap();
    
    // Get the value
    let retrieved = adapter.get(&key).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().0, "value1");
    
    // Check stats
    assert_eq!(adapter.len(), 1);
    assert!(adapter.size_bytes() > 0);
}

#[tokio::test]
async fn test_content_cache_adapter_remove() {
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let config = CacheConfig::default();
    let cache = ContentCache::new(strategy, config);
    let adapter = ContentCacheAdapter::new(cache);
    
    // Put and remove a value
    let key = TestKey("key1".to_string());
    let value = TestValue("value1".to_string());
    adapter.put(key.clone(), value).await.unwrap();
    
    assert_eq!(adapter.len(), 1);
    
    let removed = adapter.remove(&key).await;
    assert!(removed.is_some());
    assert_eq!(adapter.len(), 0);
}

#[tokio::test]
async fn test_content_cache_adapter_clear() {
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let config = CacheConfig::default();
    let cache = ContentCache::new(strategy, config);
    let adapter = ContentCacheAdapter::new(cache);
    
    // Put multiple values
    for i in 0..5 {
        let key = TestKey(format!("key{}", i));
        let value = TestValue(format!("value{}", i));
        adapter.put(key, value).await.unwrap();
    }
    
    assert_eq!(adapter.len(), 5);
    
    // Clear cache
    adapter.clear().await.unwrap();
    assert_eq!(adapter.len(), 0);
}

#[tokio::test]
async fn test_content_cache_adapter_stats() {
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let config = CacheConfig::default();
    let cache = ContentCache::new(strategy, config);
    let adapter = ContentCacheAdapter::new(cache);
    
    // Initial stats
    let stats = adapter.stats();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    
    // Miss
    let key = TestKey("key1".to_string());
    assert!(adapter.get(&key).await.is_none());
    
    let stats = adapter.stats();
    assert_eq!(stats.misses, 1);
    
    // Hit
    let value = TestValue("value1".to_string());
    adapter.put(key.clone(), value).await.unwrap();
    assert!(adapter.get(&key).await.is_some());
    
    let stats = adapter.stats();
    assert_eq!(stats.hits, 1);
}

#[tokio::test]
async fn test_content_cache_adapter_evict_if_needed() {
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let config = CacheConfig::default();
    let cache = ContentCache::new(strategy, config);
    let adapter = ContentCacheAdapter::new(cache);
    
    // evict_if_needed should always succeed for ContentCache
    adapter.evict_if_needed().await.unwrap();
}

#[tokio::test]
async fn test_persistent_cache_adapter_creation() {
    let temp_dir = TempDir::new().unwrap();
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let config = CacheConfig::default();
    let cache = PersistentCache::new(strategy, config, temp_dir.path()).unwrap();
    let adapter = PersistentCacheAdapter::new(cache);
    
    assert_eq!(adapter.len(), 0);
}

#[tokio::test]
async fn test_persistent_cache_adapter_get_put() {
    let temp_dir = TempDir::new().unwrap();
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let config = CacheConfig::default();
    let cache = PersistentCache::new(strategy, config, temp_dir.path()).unwrap();
    let adapter = PersistentCacheAdapter::new(cache);
    
    // Put a value
    let key = TestKey("key1".to_string());
    let value = TestValue("value1".to_string());
    adapter.put(key.clone(), value.clone()).await.unwrap();
    
    // Get the value
    let retrieved = adapter.get(&key).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().0, "value1");
}

#[tokio::test]
async fn test_persistent_cache_adapter_remove() {
    let temp_dir = TempDir::new().unwrap();
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let config = CacheConfig::default();
    let cache = PersistentCache::new(strategy, config, temp_dir.path()).unwrap();
    let adapter = PersistentCacheAdapter::new(cache);
    
    // Put and remove a value
    let key = TestKey("key1".to_string());
    let value = TestValue("value1".to_string());
    adapter.put(key.clone(), value).await.unwrap();
    
    let removed = adapter.remove(&key).await;
    assert!(removed.is_some());
    
    // Verify it's gone
    assert!(adapter.get(&key).await.is_none());
}

#[tokio::test]
async fn test_persistent_cache_adapter_clear() {
    let temp_dir = TempDir::new().unwrap();
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let config = CacheConfig::default();
    let cache = PersistentCache::new(strategy, config, temp_dir.path()).unwrap();
    let adapter = PersistentCacheAdapter::new(cache);
    
    // Put multiple values
    for i in 0..5 {
        let key = TestKey(format!("key{}", i));
        let value = TestValue(format!("value{}", i));
        adapter.put(key, value).await.unwrap();
    }
    
    // Clear cache
    adapter.clear().await.unwrap();
    assert_eq!(adapter.len(), 0);
}

#[tokio::test]
async fn test_persistent_cache_adapter_stats() {
    let temp_dir = TempDir::new().unwrap();
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let config = CacheConfig::default();
    let cache = PersistentCache::new(strategy, config, temp_dir.path()).unwrap();
    let adapter = PersistentCacheAdapter::new(cache);
    
    let stats = adapter.stats();
    assert_eq!(stats.items, 0);
}

#[tokio::test]
async fn test_persistent_cache_adapter_evict_if_needed() {
    let temp_dir = TempDir::new().unwrap();
    let strategy = HashCacheStrategy::<TestKey, TestValue>::new();
    let mut config = CacheConfig::default();
    config.max_memory_bytes = 100; // Small limit to trigger eviction
    
    let cache = PersistentCache::new(strategy, config, temp_dir.path()).unwrap();
    let adapter = PersistentCacheAdapter::new(cache);
    
    // Add items until we need eviction
    for i in 0..10 {
        let key = TestKey(format!("key{}", i));
        let value = TestValue(format!("value{}", i));
        adapter.put(key, value).await.unwrap();
    }
    
    // Evict if needed should succeed
    adapter.evict_if_needed().await.unwrap();
}