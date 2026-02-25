// Included from mod.rs — do NOT add `use` imports or `#!` inner attributes

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::atomic::Ordering;

    // =========================================================================
    // Property-Based Tests for CacheStats
    // =========================================================================

    proptest! {
        #[test]
        fn hit_rate_is_bounded(hits in 0u64..10000, misses in 0u64..10000) {
            let stats = CacheStats::new();

            for _ in 0..hits {
                stats.record_hit();
            }
            for _ in 0..misses {
                stats.record_miss();
            }

            let rate = stats.hit_rate();
            prop_assert!(rate >= 0.0, "Hit rate should be >= 0");
            prop_assert!(rate <= 1.0, "Hit rate should be <= 1");
        }

        #[test]
        fn total_requests_equals_sum(hits in 0u64..1000, misses in 0u64..1000) {
            let stats = CacheStats::new();

            for _ in 0..hits {
                stats.record_hit();
            }
            for _ in 0..misses {
                stats.record_miss();
            }

            prop_assert_eq!(stats.total_requests(), hits + misses);
        }

        #[test]
        fn bytes_operations_are_consistent(adds in proptest::collection::vec(1usize..1000, 0..10)) {
            let stats = CacheStats::new();

            let total: usize = adds.iter().sum();
            for add in adds {
                stats.add_bytes(add);
            }

            prop_assert_eq!(stats.memory_usage(), total);
        }

        #[test]
        fn hit_rate_calculation_is_correct(hits in 1u64..1000, misses in 1u64..1000) {
            let stats = CacheStats::new();

            for _ in 0..hits {
                stats.record_hit();
            }
            for _ in 0..misses {
                stats.record_miss();
            }

            let expected = hits as f64 / (hits + misses) as f64;
            let actual = stats.hit_rate();

            prop_assert!((actual - expected).abs() < 1e-10,
                "Expected hit rate {} but got {}", expected, actual);
        }
    }

    // =========================================================================
    // Property-Based Tests for CacheEntry
    // =========================================================================

    proptest! {
        #[test]
        fn cache_entry_preserves_value(value in ".*") {
            let entry = CacheEntry::new(value.clone(), 100);
            prop_assert_eq!(entry.value.as_ref(), &value);
        }

        #[test]
        fn cache_entry_preserves_size(size in 0usize..1_000_000) {
            let entry = CacheEntry::new("test".to_string(), size);
            prop_assert_eq!(entry.size_bytes, size);
        }

        #[test]
        fn access_count_increases_monotonically(access_count in 0u32..100) {
            let entry = CacheEntry::new("test".to_string(), 100);

            for _ in 0..access_count {
                entry.access();
            }

            prop_assert_eq!(
                entry.access_count.load(Ordering::Relaxed),
                access_count
            );
        }

        #[test]
        fn age_is_non_negative(_dummy in 0..100) {
            let entry = CacheEntry::new("test".to_string(), 100);
            let age = entry.age();
            // Duration::as_nanos() returns u128, always non-negative; just verify it completes
            let _ = age.as_nanos();
        }
    }

    // =========================================================================
    // Property-Based Tests for GitStats
    // =========================================================================

    proptest! {
        #[test]
        fn git_stats_clone_preserves_data(
            commits in 0usize..10000,
            author_count in 0usize..100
        ) {
            let authors: Vec<String> = (0..author_count)
                .map(|i| format!("author_{}", i))
                .collect();

            let stats = GitStats {
                total_commits: commits,
                authors: authors.clone(),
                branch: "main".to_string(),
                head_commit: "abc123".to_string(),
            };

            let cloned = stats.clone();

            prop_assert_eq!(cloned.total_commits, commits);
            prop_assert_eq!(cloned.authors.len(), author_count);
            prop_assert_eq!(cloned.branch, "main");
            prop_assert_eq!(cloned.head_commit, "abc123");
        }
    }

    // =========================================================================
    // Property-Based Tests for Strategy TTLs
    // =========================================================================

    proptest! {
        #[test]
        fn ttl_durations_are_consistent(_dummy in 0..10) {
            // All TTLs should be positive and consistent across calls
            let ast_ttl1 = AstCacheStrategy.ttl().unwrap();
            let ast_ttl2 = AstCacheStrategy.ttl().unwrap();
            prop_assert_eq!(ast_ttl1, ast_ttl2);

            let template_ttl1 = TemplateCacheStrategy.ttl().unwrap();
            let template_ttl2 = TemplateCacheStrategy.ttl().unwrap();
            prop_assert_eq!(template_ttl1, template_ttl2);
        }

        #[test]
        fn max_sizes_are_positive(_dummy in 0..10) {
            prop_assert!(AstCacheStrategy.max_size() > 0);
            prop_assert!(TemplateCacheStrategy.max_size() > 0);
            prop_assert!(DagCacheStrategy.max_size() > 0);
            prop_assert!(ChurnCacheStrategy.max_size() > 0);
            prop_assert!(GitStatsCacheStrategy.max_size() > 0);
        }
    }
}
