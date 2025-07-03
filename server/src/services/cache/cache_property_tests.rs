#[cfg(test)]
mod tests {
    use crate::services::cache::{
        unified::{UnifiedCacheConfig, VectorizedCacheKey},
        unified_manager::UnifiedCacheManager,
    };
    use proptest::prelude::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::runtime::Runtime;

    // Strategy for generating cache keys
    prop_compose! {
        fn arb_cache_key()
            (content in "[a-zA-Z0-9]{5,50}",
             suffix in 0u64..1000)
            -> String
        {
            format!("{}-{}", content, suffix)
        }
    }

    // Strategy for generating cache values
    prop_compose! {
        fn arb_cache_value()
            (size in 100usize..10000,
             seed in any::<u64>())
            -> Vec<u8>
        {
            // Generate deterministic content based on seed
            (0..size)
                .map(|i| {
                    let val = seed.wrapping_add(i as u64)
                        .wrapping_mul(1664525)
                        .wrapping_add(1013904223);
                    (val % 256) as u8
                })
                .collect()
        }
    }


    proptest! {
        #[test]
        fn cache_get_put_consistency(
            operations in prop::collection::vec(
                (arb_cache_key(), arb_cache_value()),
                0..50
            )
        ) {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let cache = TestMemoryCache::new(1024 * 1024); // 1MB cache
                
                // Track expected state
                let mut expected = HashMap::new();
                
                for (key, value) in operations {
                    // Put value
                    cache.put(key.clone(), value.clone()).await.unwrap();
                    expected.insert(key.clone(), value.clone());
                    
                    // Get should return what we put
                    let retrieved = cache.get(&key).await;
                    prop_assert!(retrieved.is_some(), "Value not found for key: {}", key);
                    prop_assert_eq!(&*retrieved.unwrap(), &value, "Retrieved value doesn't match");
                    
                    // Stats should be consistent
                    let stats = cache.stats();
                    prop_assert!(stats.cache_hits > 0 || stats.cache_misses > 0);
                    prop_assert_eq!(stats.cache_hits + stats.cache_misses, stats.total_requests);
                }
                
                // All expected values should be retrievable
                for (key, expected_value) in &expected {
                    let retrieved = cache.get(key).await;
                    if cache.len() < expected.len() {
                        // Some eviction might have occurred
                        continue;
                    }
                    prop_assert!(retrieved.is_some(), "Expected value missing for key: {}", key);
                    prop_assert_eq!(&*retrieved.unwrap(), expected_value);
                }
                
                Ok(())
            })?;
        }

        #[test]
        fn cache_remove_consistency(
            initial_entries in prop::collection::vec(
                (arb_cache_key(), arb_cache_value()),
                0..20
            ),
            remove_keys in prop::collection::vec(any::<usize>(), 0..10)
        ) {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let cache = TestMemoryCache::new(1024 * 1024);
                
                // Insert initial entries
                for (key, value) in &initial_entries {
                    cache.put(key.clone(), value.clone()).await.unwrap();
                }
                
                let initial_len = cache.len();
                
                // Remove some entries
                let mut removed_count = 0;
                for idx in remove_keys {
                    if !initial_entries.is_empty() {
                        let (key, _) = &initial_entries[idx % initial_entries.len()];
                        if cache.remove(key).await.is_some() {
                            removed_count += 1;
                            
                            // Verify it's gone
                            prop_assert!(cache.get(key).await.is_none(), 
                                "Removed key {} still present", key);
                        }
                    }
                }
                
                // Length should decrease appropriately
                prop_assert_eq!(cache.len(), initial_len.saturating_sub(removed_count),
                    "Cache length inconsistent after removals");
                
                Ok(())
            })?;
        }

        #[test]
        fn cache_clear_idempotent(
            entries in prop::collection::vec(
                (arb_cache_key(), arb_cache_value()),
                0..30
            )
        ) {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let cache = TestMemoryCache::new(1024 * 1024);
                
                // Insert entries
                for (key, value) in &entries {
                    cache.put(key.clone(), value.clone()).await.unwrap();
                }
                
                // Clear should empty the cache
                cache.clear().await.unwrap();
                prop_assert_eq!(cache.len(), 0);
                prop_assert!(cache.is_empty());
                
                // Clear again should be idempotent
                cache.clear().await.unwrap();
                prop_assert_eq!(cache.len(), 0);
                
                // All entries should be gone
                for (key, _) in &entries {
                    prop_assert!(cache.get(key).await.is_none());
                }
                
                Ok(())
            })?;
        }

        #[test]
        fn cache_eviction_maintains_invariants(
            entries in prop::collection::vec(
                (arb_cache_key(), arb_cache_value()),
                0..100
            ),
            max_size in 1024usize..10240 // 1KB to 10KB
        ) {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let cache = TestMemoryCache::new(max_size);
                
                // Insert entries until we exceed capacity
                for (key, value) in entries {
                    cache.put(key, value).await.unwrap();
                    
                    // Size should never exceed max after eviction
                    cache.evict_if_needed().await.unwrap();
                    prop_assert!(cache.size_bytes() <= max_size,
                        "Cache size {} exceeds max {}", cache.size_bytes(), max_size);
                }
                
                // Manual eviction should maintain invariants
                let size_before = cache.size_bytes();
                cache.evict_if_needed().await.unwrap();
                let size_after = cache.size_bytes();
                
                prop_assert!(size_after <= size_before, 
                    "Size increased after eviction: {} -> {}", size_before, size_after);
                prop_assert!(size_after <= max_size,
                    "Size {} still exceeds max {} after eviction", size_after, max_size);
                
                Ok(())
            })?;
        }

        #[test]
        fn vectorized_key_deterministic(
            data in prop::collection::vec(any::<u8>(), 0..1000)
        ) {
            // Same data should produce same key
            let key1 = VectorizedCacheKey::from_bytes(&data);
            let key2 = VectorizedCacheKey::from_bytes(&data);
            
            prop_assert_eq!(&key1, &key2, "Keys not deterministic");
            prop_assert_eq!(key1.hash_high, key2.hash_high);
            prop_assert_eq!(key1.hash_low, key2.hash_low);
            
            // Different data should (usually) produce different keys
            if !data.is_empty() {
                let mut modified = data.clone();
                modified[0] = modified[0].wrapping_add(1);
                let key3 = VectorizedCacheKey::from_bytes(&modified);
                
                // Very high probability of difference
                prop_assert!(key1 != key3 || data.len() == 1,
                    "Different data produced same key");
            }
        }

        #[test]
        fn cache_operation_sequence_consistency(
            key_pool in prop::collection::vec(arb_cache_key(), 5..10),
            ops in prop::collection::vec(any::<usize>(), 20..50)
        ) {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let cache = TestMemoryCache::new(1024 * 1024);
                let mut shadow = HashMap::new();
                
                for (i, op_seed) in ops.into_iter().enumerate() {
                    let key = &key_pool[op_seed % key_pool.len()];
                    
                    match op_seed % 4 {
                        0 => {
                            // Get
                            let cached = cache.get(key).await;
                            let shadowed = shadow.get(key);
                            
                            match (cached, shadowed) {
                                (Some(c), Some(s)) => prop_assert_eq!(&*c, s),
                                (None, None) => {},
                                _ => prop_assert!(false, "Cache/shadow mismatch on get"),
                            }
                        },
                        1 => {
                            // Put
                            let value = vec![i as u8; 100];
                            cache.put(key.clone(), value.clone()).await.unwrap();
                            shadow.insert(key.clone(), value);
                        },
                        2 => {
                            // Remove
                            let cached = cache.remove(key).await;
                            let shadowed = shadow.remove(key);
                            
                            match (cached, shadowed) {
                                (Some(c), Some(s)) => prop_assert_eq!(&*c, &s),
                                (None, None) => {},
                                _ => prop_assert!(false, "Cache/shadow mismatch on remove"),
                            }
                        },
                        _ => {
                            // Clear
                            if shadow.len() > 20 {
                                cache.clear().await.unwrap();
                                shadow.clear();
                                prop_assert_eq!(cache.len(), 0);
                            }
                        }
                    }
                }
                
                Ok(())
            })?;
        }

        #[test]
        fn unified_cache_manager_consistency(
            operations in prop::collection::vec(
                (arb_cache_key(), arb_cache_value()),
                0..20
            )
        ) {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let config = UnifiedCacheConfig {
                    max_memory_bytes: 1024 * 1024,
                    persistent: true,
                    storage_path: Some(temp_dir.path().to_string_lossy().to_string()),
                    ttl: Some(Duration::from_secs(3600)),
                    max_entries: 1000,
                    eviction_batch_size: 10,
                };
                
                let manager = UnifiedCacheManager::new(config).unwrap();
                
                // Test AST cache operations using get_or_compute
                for (key, _value) in operations {
                    let path = std::path::Path::new(&key);
                    let file_context = crate::services::context::FileContext {
                        path: path.to_string_lossy().to_string(),
                        language: "rust".to_string(),
                        items: vec![],
                        complexity_metrics: None,
                    };
                    
                    // Store via get_or_compute
                    let context_clone = file_context.clone();
                    let result = manager.get_or_compute_ast(path, || async move {
                        Ok(context_clone)
                    }).await;
                    
                    prop_assert!(result.is_ok(), "Failed to store AST cache for {}", key);
                    
                    // Retrieve again - should hit cache
                    let cached_hit = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                    let cached_hit_clone = cached_hit.clone();
                    let context_clone2 = file_context.clone();
                    let retrieved = manager.get_or_compute_ast(path, || async move {
                        // This shouldn't be called if cache hit
                        cached_hit_clone.store(true, std::sync::atomic::Ordering::Relaxed);
                        Ok(context_clone2)
                    }).await;
                    
                    prop_assert!(retrieved.is_ok(), "Failed to retrieve AST cache for {}", key);
                    prop_assert!(!cached_hit.load(std::sync::atomic::Ordering::Relaxed),
                        "Compute function was called on what should be a cache hit");
                    
                    // Verify content matches
                    if let Ok(cached_context) = retrieved {
                        prop_assert_eq!(&cached_context.path, &file_context.path);
                        prop_assert_eq!(&cached_context.language, &file_context.language);
                    }
                }
                
                // Clear and verify
                manager.clear_all().await.unwrap();
                
                Ok(())
            })?;
        }

        #[test]
        fn cache_stats_accuracy(
            operations in prop::collection::vec(
                (prop::sample::select(vec!["get", "put", "hit", "miss"]), arb_cache_key()),
                0..50
            )
        ) {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let cache = TestMemoryCache::new(1024 * 1024);
                let mut expected_hits = 0;
                let mut expected_misses = 0;
                let mut total_requests = 0;
                
                for (op, key) in operations {
                    match op {
                        "put" => {
                            cache.put(key, vec![1, 2, 3]).await.unwrap();
                        },
                        "get" | "hit" => {
                            total_requests += 1;
                            let result = cache.get(&key).await;
                            if result.is_some() {
                                expected_hits += 1;
                            } else {
                                expected_misses += 1;
                            }
                        },
                        "miss" => {
                            total_requests += 1;
                            // Force a miss by getting non-existent key
                            let miss_key = format!("{}-miss", key);
                            let result = cache.get(&miss_key).await;
                            prop_assert!(result.is_none());
                            expected_misses += 1;
                        },
                        _ => unreachable!(),
                    }
                }
                
                let stats = cache.stats();
                prop_assert_eq!(stats.cache_hits, expected_hits,
                    "Hit count mismatch: {} vs {}", stats.cache_hits, expected_hits);
                prop_assert_eq!(stats.cache_misses, expected_misses,
                    "Miss count mismatch: {} vs {}", stats.cache_misses, expected_misses);
                prop_assert_eq!(stats.total_requests, total_requests,
                    "Total requests mismatch: {} vs {}", stats.total_requests, total_requests);
                
                Ok(())
            })?;
        }
    }

    // Test implementation of a simple memory cache for property testing
    struct TestMemoryCache {
        entries: Arc<tokio::sync::RwLock<HashMap<String, Arc<Vec<u8>>>>>,
        max_size: usize,
        stats: Arc<tokio::sync::RwLock<TestCacheStats>>,
    }

    #[derive(Default)]
    struct TestCacheStats {
        cache_hits: usize,
        cache_misses: usize,
        total_requests: usize,
    }

    impl TestMemoryCache {
        fn new(max_size: usize) -> Self {
            Self {
                entries: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
                max_size,
                stats: Arc::new(tokio::sync::RwLock::new(TestCacheStats::default())),
            }
        }

        async fn get(&self, key: &str) -> Option<Arc<Vec<u8>>> {
            let entries = self.entries.read().await;
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            
            if let Some(value) = entries.get(key) {
                stats.cache_hits += 1;
                Some(value.clone())
            } else {
                stats.cache_misses += 1;
                None
            }
        }

        async fn put(&self, key: String, value: Vec<u8>) -> anyhow::Result<()> {
            let mut entries = self.entries.write().await;
            entries.insert(key, Arc::new(value));
            Ok(())
        }

        async fn remove(&self, key: &str) -> Option<Arc<Vec<u8>>> {
            let mut entries = self.entries.write().await;
            entries.remove(key)
        }

        async fn clear(&self) -> anyhow::Result<()> {
            let mut entries = self.entries.write().await;
            entries.clear();
            Ok(())
        }

        async fn evict_if_needed(&self) -> anyhow::Result<()> {
            let mut entries = self.entries.write().await;
            let current_size = self.size_bytes_internal(&entries);
            
            if current_size > self.max_size {
                // Simple FIFO eviction
                while self.size_bytes_internal(&entries) > self.max_size && !entries.is_empty() {
                    if let Some(key) = entries.keys().next().cloned() {
                        entries.remove(&key);
                    }
                }
            }
            Ok(())
        }

        fn len(&self) -> usize {
            // This is a simplified sync version for testing
            futures::executor::block_on(async {
                self.entries.read().await.len()
            })
        }

        fn is_empty(&self) -> bool {
            self.len() == 0
        }

        fn size_bytes(&self) -> usize {
            futures::executor::block_on(async {
                let entries = self.entries.read().await;
                self.size_bytes_internal(&entries)
            })
        }

        fn size_bytes_internal(&self, entries: &HashMap<String, Arc<Vec<u8>>>) -> usize {
            entries.iter()
                .map(|(k, v)| k.len() + v.len())
                .sum()
        }

        fn stats(&self) -> TestCacheStats {
            futures::executor::block_on(async {
                let stats = self.stats.read().await;
                TestCacheStats {
                    cache_hits: stats.cache_hits,
                    cache_misses: stats.cache_misses,
                    total_requests: stats.total_requests,
                }
            })
        }
    }
}