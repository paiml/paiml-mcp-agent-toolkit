// Cache tests - test module for TwoTierCache
// Included by cache.rs via include!()

#[cfg_attr(coverage_nightly, coverage(off))]
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

    #[test]
    fn test_analysis_result_default() {
        let result = AnalysisResult::default();
        assert_eq!(result.complexity, 0);
        assert_eq!(result.cognitive_complexity, 0);
        assert_eq!(result.satd_count, 0);
        assert_eq!(result.content_hash, 0);
    }

    #[test]
    fn test_analysis_result_creation() {
        let result = AnalysisResult {
            complexity: 15,
            cognitive_complexity: 20,
            satd_count: 3,
            timestamp: SystemTime::now(),
            content_hash: 12345,
        };
        assert_eq!(result.complexity, 15);
        assert_eq!(result.cognitive_complexity, 20);
        assert_eq!(result.satd_count, 3);
        assert_eq!(result.content_hash, 12345);
    }

    #[test]
    fn test_cache_entry_creation() {
        let result = AnalysisResult::default();
        let entry = CacheEntry::new(result, Duration::from_secs(60));
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_expiration() {
        let result = AnalysisResult::default();
        // Entry with 0 duration should be expired immediately
        let entry = CacheEntry::new(result, Duration::from_nanos(1));
        std::thread::sleep(Duration::from_millis(1));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_default() {
        let cache = TwoTierCache::default();
        let metrics = cache.hit_rate();
        assert_eq!(metrics.l1_hit_rate, 0.0);
        assert_eq!(metrics.effective_hit_rate, 0.0);
    }

    #[test]
    fn test_hash_key_consistency() {
        let cache = TwoTierCache::new();
        let hash1 = cache.hash_key("test_key");
        let hash2 = cache.hash_key("test_key");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_key_difference() {
        let cache = TwoTierCache::new();
        let hash1 = cache.hash_key("key1");
        let hash2 = cache.hash_key("key2");
        assert_ne!(hash1, hash2);
    }

    #[tokio::test]
    async fn test_evict_expired() {
        let cache = TwoTierCache::new();

        // Add an entry
        cache
            .get_with_loader("test_key", || async { AnalysisResult::default() })
            .await;

        // Evict expired - should not remove non-expired entries
        cache.evict_expired().await;

        // Entry should still be accessible via L1 or L2
        let result = cache
            .get_with_loader("test_key", || async {
                AnalysisResult {
                    complexity: 99,
                    ..Default::default()
                }
            })
            .await;

        // Should get cached value, not the new one (99)
        assert_eq!(result.complexity, 0);
    }

    #[test]
    fn test_cache_metrics_struct() {
        let metrics = CacheMetrics {
            l1_hit_rate: 0.75,
            l2_hit_rate: 0.5,
            effective_hit_rate: 0.85,
            l1_size: 100,
            l2_size: 500,
        };
        assert_eq!(metrics.l1_hit_rate, 0.75);
        assert_eq!(metrics.l2_hit_rate, 0.5);
        assert_eq!(metrics.effective_hit_rate, 0.85);
        assert_eq!(metrics.l1_size, 100);
        assert_eq!(metrics.l2_size, 500);
    }

    #[tokio::test]
    async fn test_multiple_keys() {
        let cache = TwoTierCache::new();

        // Add multiple entries with different keys
        cache
            .get_with_loader("key1", || async {
                AnalysisResult {
                    complexity: 10,
                    ..Default::default()
                }
            })
            .await;

        cache
            .get_with_loader("key2", || async {
                AnalysisResult {
                    complexity: 20,
                    ..Default::default()
                }
            })
            .await;

        // Retrieve both - should get cached values
        let result1 = cache
            .get_with_loader("key1", || async {
                AnalysisResult {
                    complexity: 100,
                    ..Default::default()
                }
            })
            .await;

        let result2 = cache
            .get_with_loader("key2", || async {
                AnalysisResult {
                    complexity: 200,
                    ..Default::default()
                }
            })
            .await;

        assert_eq!(result1.complexity, 10);
        assert_eq!(result2.complexity, 20);
    }

    #[test]
    fn test_fnv_hasher_different_inputs() {
        let mut hasher1 = FnvHasher::default();
        hasher1.write(b"hello");
        let hash1 = hasher1.finish();

        let mut hasher2 = FnvHasher::default();
        hasher2.write(b"world");
        let hash2 = hasher2.finish();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_fnv_hasher_empty() {
        let mut hasher = FnvHasher::default();
        hasher.write(b"");
        let hash = hasher.finish();
        // FNV-1a hash of empty string should be the offset basis
        assert_ne!(hash, 0);
    }
}
