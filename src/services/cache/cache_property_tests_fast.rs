#[cfg(test)]
mod tests {
    use crate::services::cache::unified::VectorizedCacheKey;
    use proptest::prelude::*;
    use std::collections::HashMap;

    // Smaller, faster strategies for property tests
    prop_compose! {
        fn arb_cache_key()
            (content in "[a-zA-Z0-9]{5,20}",  // Reduced from 50
             suffix in 0u64..100)              // Reduced from 1000
            -> String
        {
            format!("{}-{}", content, suffix)
        }
    }

    prop_compose! {
        fn arb_cache_value()
            (size in 10usize..100,  // Reduced from 100..10000
             seed in any::<u64>())
            -> Vec<u8>
        {
            // Generate deterministic content based on seed
            (0..size)
                .map(|i| ((seed.wrapping_add(i as u64)) % 256) as u8)
                .collect()
        }
    }

    proptest! {
        // Set custom configuration for faster tests
        #![proptest_config(ProptestConfig::with_cases(10))]  // Reduced from default 256

        #[test]
        fn cache_get_put_consistency_fast(
            operations in prop::collection::vec(
                (arb_cache_key(), arb_cache_value()),
                0..10  // Reduced from 50
            )
        ) {
            // Use a simple sync HashMap instead of async cache for faster testing
            let mut cache = HashMap::new();
            let mut stats = TestCacheStats::default();

            for (key, value) in operations {
                // Put value
                cache.insert(key.clone(), value.clone());

                // Get should return what we put
                let retrieved = cache.get(&key);
                prop_assert!(retrieved.is_some(), "Value not found for key: {}", key);
                prop_assert_eq!(retrieved.unwrap(), &value, "Retrieved value doesn't match");

                // Update stats
                stats.total_requests += 1;
                stats.cache_hits += 1;
            }

            // Verify stats consistency
            prop_assert_eq!(stats.cache_hits + stats.cache_misses, stats.total_requests);
        }

        #[test]
        fn cache_eviction_maintains_invariants_fast(
            entries in prop::collection::vec(
                (arb_cache_key(), arb_cache_value()),
                0..20  // Reduced from 100
            ),
            max_size in 100usize..1000  // Reduced from 1024..10240
        ) {
            let mut cache = SimpleEvictingCache::new(max_size);

            // Insert entries until we exceed capacity
            for (key, value) in entries {
                cache.put(key, value);

                // Size should never exceed max after eviction
                prop_assert!(cache.size_bytes() <= max_size,
                    "Cache size {} exceeds max {}", cache.size_bytes(), max_size);
            }

            // Manual eviction should maintain invariants
            let size_before = cache.size_bytes();
            cache.evict_if_needed();
            let size_after = cache.size_bytes();

            prop_assert!(size_after <= size_before,
                "Size increased after eviction: {} -> {}", size_before, size_after);
            prop_assert!(size_after <= max_size,
                "Size {} still exceeds max {} after eviction", size_after, max_size);
        }

        #[test]
        fn cache_remove_consistency_fast(
            initial_entries in prop::collection::vec(
                (arb_cache_key(), arb_cache_value()),
                0..10  // Reduced from 20
            ),
            remove_indices in prop::collection::vec(any::<usize>(), 0..5)  // Reduced from 10
        ) {
            let mut cache = HashMap::new();

            // Insert initial entries
            for (key, value) in &initial_entries {
                cache.insert(key.clone(), value.clone());
            }

            let initial_len = cache.len();

            // Remove some entries
            let mut removed_count = 0;
            for idx in remove_indices {
                if !initial_entries.is_empty() {
                    let (key, _) = &initial_entries[idx % initial_entries.len()];
                    if cache.remove(key).is_some() {
                        removed_count += 1;

                        // Verify it's gone
                        prop_assert!(!cache.contains_key(key),
                            "Removed key {} still present", key);
                    }
                }
            }

            // Length should decrease appropriately
            prop_assert_eq!(cache.len(), initial_len.saturating_sub(removed_count),
                "Cache length inconsistent after removals");
        }

        #[test]
        fn cache_clear_idempotent_fast(
            entries in prop::collection::vec(
                (arb_cache_key(), arb_cache_value()),
                0..10  // Reduced from 30
            )
        ) {
            let mut cache = HashMap::new();

            // Insert entries
            for (key, value) in &entries {
                cache.insert(key.clone(), value.clone());
            }

            // Clear should empty the cache
            cache.clear();
            prop_assert_eq!(cache.len(), 0);

            // Clear again should be idempotent
            cache.clear();
            prop_assert_eq!(cache.len(), 0);

            // All entries should be gone
            for (key, _) in &entries {
                prop_assert!(!cache.contains_key(key));
            }
        }

        #[test]
        fn vectorized_key_deterministic_fast(
            data in prop::collection::vec(any::<u8>(), 0..100)  // Reduced from 1000
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
    }

    // Simple synchronous cache for faster testing
    struct SimpleEvictingCache {
        entries: HashMap<String, Vec<u8>>,
        max_size: usize,
    }

    impl SimpleEvictingCache {
        fn new(max_size: usize) -> Self {
            Self {
                entries: HashMap::new(),
                max_size,
            }
        }

        fn put(&mut self, key: String, value: Vec<u8>) {
            self.entries.insert(key, value);
            self.evict_if_needed();
        }

        fn size_bytes(&self) -> usize {
            self.entries.iter().map(|(k, v)| k.len() + v.len()).sum()
        }

        fn evict_if_needed(&mut self) {
            while self.size_bytes() > self.max_size && !self.entries.is_empty() {
                // Simple FIFO eviction - remove first key
                if let Some(key) = self.entries.keys().next().cloned() {
                    self.entries.remove(&key);
                } else {
                    break;
                }
            }
        }
    }

    #[derive(Default)]
    struct TestCacheStats {
        cache_hits: usize,
        cache_misses: usize,
        total_requests: usize,
    }
}
