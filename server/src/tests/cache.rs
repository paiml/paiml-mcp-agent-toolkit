use crate::services::cache::{get_content, get_metadata, put_content, put_metadata};
use lru::LruCache;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, PartialEq)]
struct TestMetadata {
    name: String,
    version: u32,
}

#[tokio::test]
async fn test_metadata_cache_hit() {
    let cache = Arc::new(RwLock::new(LruCache::<String, Arc<TestMetadata>>::new(
        std::num::NonZeroUsize::new(10).unwrap(),
    )));

    let metadata = Arc::new(TestMetadata {
        name: "test".to_string(),
        version: 1,
    });

    // Put metadata in cache
    put_metadata(&cache, "test_key".to_string(), metadata.clone()).await;

    // Get metadata from cache
    let result = get_metadata(&cache, "test_key").await;

    assert!(result.is_some());
    assert_eq!(*result.unwrap(), *metadata);
}

#[tokio::test]
async fn test_metadata_cache_miss() {
    let cache = Arc::new(RwLock::new(LruCache::<String, Arc<TestMetadata>>::new(
        std::num::NonZeroUsize::new(10).unwrap(),
    )));

    // Try to get non-existent metadata
    let result = get_metadata(&cache, "non_existent").await;

    assert!(result.is_none());
}

#[tokio::test]
async fn test_content_cache_hit() {
    let cache = Arc::new(RwLock::new(LruCache::<String, Arc<str>>::new(
        std::num::NonZeroUsize::new(10).unwrap(),
    )));

    let content: Arc<str> = Arc::from("test content");

    // Put content in cache
    put_content(&cache, "content_key".to_string(), content.clone()).await;

    // Get content from cache
    let result = get_content(&cache, "content_key").await;

    assert!(result.is_some());
    assert_eq!(result.unwrap(), content);
}

#[tokio::test]
async fn test_content_cache_miss() {
    let cache = Arc::new(RwLock::new(LruCache::<String, Arc<str>>::new(
        std::num::NonZeroUsize::new(10).unwrap(),
    )));

    // Try to get non-existent content
    let result = get_content(&cache, "non_existent").await;

    assert!(result.is_none());
}

#[tokio::test]
async fn test_cache_lru_eviction() {
    let cache = Arc::new(RwLock::new(LruCache::<String, Arc<str>>::new(
        std::num::NonZeroUsize::new(2).unwrap(), // Small cache size of 2
    )));

    let content1: Arc<str> = Arc::from("content1");
    let content2: Arc<str> = Arc::from("content2");
    let content3: Arc<str> = Arc::from("content3");

    // Fill cache
    put_content(&cache, "key1".to_string(), content1.clone()).await;
    put_content(&cache, "key2".to_string(), content2.clone()).await;

    // Both should be in cache
    assert!(get_content(&cache, "key1").await.is_some());
    assert!(get_content(&cache, "key2").await.is_some());

    // Add third item, should evict first
    put_content(&cache, "key3".to_string(), content3.clone()).await;

    // First should be evicted, second and third should remain
    assert!(get_content(&cache, "key1").await.is_none());
    assert!(get_content(&cache, "key2").await.is_some());
    assert!(get_content(&cache, "key3").await.is_some());
}

#[tokio::test]
async fn test_cache_update_existing() {
    let cache = Arc::new(RwLock::new(LruCache::<String, Arc<str>>::new(
        std::num::NonZeroUsize::new(10).unwrap(),
    )));

    let content1: Arc<str> = Arc::from("original content");
    let content2: Arc<str> = Arc::from("updated content");

    // Put original content
    put_content(&cache, "update_key".to_string(), content1).await;

    // Update with new content
    put_content(&cache, "update_key".to_string(), content2.clone()).await;

    // Should get updated content
    let result = get_content(&cache, "update_key").await;
    assert!(result.is_some());
    assert_eq!(result.unwrap(), content2);
}

#[tokio::test]
async fn test_concurrent_cache_access() {
    let cache = Arc::new(RwLock::new(LruCache::<String, Arc<str>>::new(
        std::num::NonZeroUsize::new(10).unwrap(),
    )));

    let content: Arc<str> = Arc::from("concurrent content");

    // Put content
    put_content(&cache, "concurrent_key".to_string(), content.clone()).await;

    // Spawn multiple tasks to read concurrently
    let mut handles = vec![];
    for _ in 0..5 {
        let cache_clone = cache.clone();
        let handle = tokio::spawn(async move { get_content(&cache_clone, "concurrent_key").await });
        handles.push(handle);
    }

    // All reads should succeed
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), content);
    }
}
