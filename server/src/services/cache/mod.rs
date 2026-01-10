pub mod adapters;
pub mod advanced_strategies;
pub mod base;
#[cfg(test)]
pub mod cache_property_tests;
#[cfg(test)]
pub mod cache_property_tests_fast;
pub mod cache_trait;
pub mod config;
pub mod content_cache;
pub mod diagnostics;
pub mod manager;
pub mod orchestrator;
pub mod persistent;
pub mod persistent_manager;
pub mod strategies;
pub mod unified; // Compatibility stub
pub mod unified_manager; // Compatibility stub

pub use advanced_strategies::{
    AdaptiveCache, AdaptiveCacheStats, AdvancedCacheConfig, CacheTier, CacheWarmingConfig,
    EvictionPolicy, PerformanceConfig,
};
pub use base::{CacheEntry, CacheStats, CacheStrategy};
pub use cache_trait::AstCacheManager;
pub use config::CacheConfig;
pub use content_cache::ContentCache;
pub use diagnostics::{CacheDiagnostics, CacheEffectiveness};
pub use manager::SessionCacheManager;
pub use orchestrator::{
    CacheOrchestrator, OrchestratorConfig, OrchestratorStats, PerformanceMetrics,
    StrategyRecommendation, WorkloadProfile,
};
pub use persistent_manager::PersistentCacheManager;
pub use strategies::{
    AstCacheStrategy, ChurnCacheStrategy, DagCacheStrategy, GitStats, GitStatsCacheStrategy,
    TemplateCacheStrategy,
};

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    // =========================================================================
    // Module Re-export Tests
    // =========================================================================

    #[test]
    fn test_cache_config_reexport() {
        // Verify CacheConfig is re-exported and usable
        let config = CacheConfig::default();
        assert_eq!(config.max_memory_mb, 100);
    }

    #[test]
    fn test_cache_entry_reexport() {
        // Verify CacheEntry is re-exported
        let entry = CacheEntry::new("test_value".to_string(), 100);
        assert_eq!(entry.size_bytes, 100);
        assert_eq!(*entry.value, "test_value");
    }

    #[test]
    fn test_cache_stats_reexport() {
        // Verify CacheStats is re-exported
        let stats = CacheStats::new();
        assert_eq!(stats.hits.load(Ordering::Relaxed), 0);
        assert_eq!(stats.misses.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_cache_strategy_reexport() {
        // Verify CacheStrategy types are re-exported
        let ast_strategy = AstCacheStrategy;
        assert_eq!(ast_strategy.ttl(), Some(Duration::from_secs(300)));

        let template_strategy = TemplateCacheStrategy;
        assert_eq!(template_strategy.ttl(), Some(Duration::from_secs(600)));

        let dag_strategy = DagCacheStrategy;
        assert_eq!(dag_strategy.ttl(), Some(Duration::from_secs(180)));

        let churn_strategy = ChurnCacheStrategy;
        assert_eq!(churn_strategy.ttl(), Some(Duration::from_secs(1800)));

        let git_stats_strategy = GitStatsCacheStrategy;
        assert_eq!(git_stats_strategy.ttl(), Some(Duration::from_secs(900)));
    }

    #[test]
    fn test_git_stats_struct_creation() {
        let stats = GitStats {
            total_commits: 42,
            authors: vec!["alice".to_string(), "bob".to_string()],
            branch: "main".to_string(),
            head_commit: "abc123def456".to_string(),
        };

        assert_eq!(stats.total_commits, 42);
        assert_eq!(stats.authors.len(), 2);
        assert_eq!(stats.branch, "main");
        assert_eq!(stats.head_commit, "abc123def456");
    }

    // =========================================================================
    // Advanced Strategies Re-export Tests
    // =========================================================================

    #[test]
    fn test_advanced_cache_config_reexport() {
        let config = AdvancedCacheConfig::default();
        // Verify it has the expected eviction policy field
        let _ = config.eviction_policy;
    }

    #[test]
    fn test_eviction_policy_reexport() {
        // Verify EvictionPolicy enum variants (uppercase)
        let lru = EvictionPolicy::LRU;
        let lfu = EvictionPolicy::LFU;
        let adaptive = EvictionPolicy::Adaptive;

        // Just verify they exist and can be matched
        match lru {
            EvictionPolicy::LRU => {}
            _ => panic!("Expected LRU"),
        }
        match lfu {
            EvictionPolicy::LFU => {}
            _ => panic!("Expected LFU"),
        }
        match adaptive {
            EvictionPolicy::Adaptive => {}
            _ => panic!("Expected Adaptive"),
        }
    }

    #[test]
    fn test_cache_tier_reexport() {
        // Actual variants are L1, L2, L3
        let l1 = CacheTier::L1;
        let l2 = CacheTier::L2;
        let l3 = CacheTier::L3;

        match l1 {
            CacheTier::L1 => {}
            _ => panic!("Expected L1"),
        }
        match l2 {
            CacheTier::L2 => {}
            _ => panic!("Expected L2"),
        }
        match l3 {
            CacheTier::L3 => {}
            _ => panic!("Expected L3"),
        }
    }

    // =========================================================================
    // Orchestrator Re-export Tests
    // =========================================================================

    #[test]
    fn test_orchestrator_config_reexport() {
        let config = OrchestratorConfig::default();
        // Just verify it can be created and has expected field
        assert!(config.evaluation_interval.as_secs() > 0 || config.evaluation_interval.as_millis() > 0);
    }

    #[test]
    fn test_workload_profile_reexport() {
        // WorkloadProfile is a struct with numeric fields
        let profile = WorkloadProfile {
            request_rate: 100.0,
            working_set_size: 1024,
            temporal_locality: 0.8,
            spatial_locality: 0.6,
            read_write_ratio: 0.9,
            target_hit_rate: 0.95,
            latency_sensitivity: 0.7,
        };

        assert_eq!(profile.request_rate, 100.0);
        assert_eq!(profile.working_set_size, 1024);
        assert_eq!(profile.read_write_ratio, 0.9);
    }

    // =========================================================================
    // Diagnostics Re-export Tests
    // =========================================================================

    #[test]
    fn test_cache_diagnostics_reexport() {
        // Create CacheDiagnostics with required fields
        let effectiveness = CacheEffectiveness {
            overall_hit_rate: 0.85,
            memory_efficiency: 0.9,
            time_saved_ms: 1000,
            most_valuable_caches: vec![("ast".to_string(), 0.95)],
        };
        let diagnostics = CacheDiagnostics {
            session_id: uuid::Uuid::new_v4(),
            uptime: Duration::from_secs(3600),
            memory_usage_mb: 256.0,
            memory_pressure: 0.3,
            cache_stats: vec![],
            hot_paths: vec![],
            effectiveness,
        };
        // Just verify it exists and can be created
        let _ = format!("{:?}", diagnostics);
    }

    #[test]
    fn test_cache_effectiveness_reexport() {
        let effectiveness = CacheEffectiveness {
            overall_hit_rate: 0.85,
            memory_efficiency: 0.9,
            time_saved_ms: 1000,
            most_valuable_caches: vec![],
        };
        // Verify it exists
        let _ = format!("{:?}", effectiveness);
        assert_eq!(effectiveness.overall_hit_rate, 0.85);
    }

    // =========================================================================
    // Content Cache Re-export Tests
    // =========================================================================

    #[test]
    fn test_content_cache_reexport() {
        // Verify ContentCache type is re-exported
        let _cache_type: std::any::TypeId = std::any::TypeId::of::<ContentCache<AstCacheStrategy>>();
    }

    // =========================================================================
    // Session Cache Manager Re-export Tests
    // =========================================================================

    #[test]
    fn test_session_cache_manager_reexport() {
        // Verify SessionCacheManager type is re-exported
        let _type_id = std::any::TypeId::of::<SessionCacheManager>();
    }

    // =========================================================================
    // Persistent Cache Manager Re-export Tests
    // =========================================================================

    #[test]
    fn test_persistent_cache_manager_creation() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config = CacheConfig::default();
        let manager = PersistentCacheManager::new(config, temp_dir.path().to_path_buf());

        // Verify it can be created
        let _ = manager;
    }

    // =========================================================================
    // Cache Entry Comprehensive Tests
    // =========================================================================

    #[test]
    fn test_cache_entry_access_increments_count() {
        let entry = CacheEntry::new("value".to_string(), 50);

        assert_eq!(entry.access_count.load(Ordering::Relaxed), 0);

        entry.access();
        assert_eq!(entry.access_count.load(Ordering::Relaxed), 1);

        entry.access();
        entry.access();
        assert_eq!(entry.access_count.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn test_cache_entry_age() {
        let entry = CacheEntry::new("value".to_string(), 50);

        let age = entry.age();
        // Age should be very small right after creation
        assert!(age < Duration::from_secs(1));
    }

    #[test]
    fn test_cache_entry_last_accessed_duration() {
        let entry = CacheEntry::new("value".to_string(), 50);

        let last_accessed = entry.last_accessed_duration();
        assert!(last_accessed < Duration::from_secs(1));

        entry.access();
        let after_access = entry.last_accessed_duration();
        assert!(after_access < Duration::from_secs(1));
    }

    #[test]
    fn test_cache_entry_clone() {
        let entry = CacheEntry::new("value".to_string(), 50);
        entry.access();
        entry.access();

        let cloned = entry.clone();

        assert_eq!(*cloned.value, "value");
        assert_eq!(cloned.size_bytes, 50);
        // Access count is shared via Arc
        assert_eq!(cloned.access_count.load(Ordering::Relaxed), 2);
    }

    // =========================================================================
    // Cache Stats Comprehensive Tests
    // =========================================================================

    #[test]
    fn test_cache_stats_default() {
        let stats = CacheStats::default();
        assert_eq!(stats.hits.load(Ordering::Relaxed), 0);
        assert_eq!(stats.misses.load(Ordering::Relaxed), 0);
        assert_eq!(stats.evictions.load(Ordering::Relaxed), 0);
        assert_eq!(stats.total_bytes.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_cache_stats_hit_rate_empty() {
        let stats = CacheStats::new();
        // No hits or misses
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_stats_hit_rate_all_hits() {
        let stats = CacheStats::new();
        stats.record_hit();
        stats.record_hit();
        stats.record_hit();

        assert_eq!(stats.hit_rate(), 1.0);
    }

    #[test]
    fn test_cache_stats_hit_rate_all_misses() {
        let stats = CacheStats::new();
        stats.record_miss();
        stats.record_miss();

        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_stats_hit_rate_mixed() {
        let stats = CacheStats::new();
        stats.record_hit();
        stats.record_hit();
        stats.record_miss();
        stats.record_miss();

        // 2 hits out of 4 total = 0.5
        assert!((stats.hit_rate() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_stats_total_requests() {
        let stats = CacheStats::new();
        assert_eq!(stats.total_requests(), 0);

        stats.record_hit();
        assert_eq!(stats.total_requests(), 1);

        stats.record_miss();
        assert_eq!(stats.total_requests(), 2);

        stats.record_hit();
        stats.record_miss();
        assert_eq!(stats.total_requests(), 4);
    }

    #[test]
    fn test_cache_stats_memory_usage() {
        let stats = CacheStats::new();
        assert_eq!(stats.memory_usage(), 0);

        stats.add_bytes(100);
        assert_eq!(stats.memory_usage(), 100);

        stats.add_bytes(200);
        assert_eq!(stats.memory_usage(), 300);

        stats.remove_bytes(50);
        assert_eq!(stats.memory_usage(), 250);
    }

    #[test]
    fn test_cache_stats_eviction_tracking() {
        let stats = CacheStats::new();
        assert_eq!(stats.evictions.load(Ordering::Relaxed), 0);

        stats.record_eviction();
        assert_eq!(stats.evictions.load(Ordering::Relaxed), 1);

        stats.record_eviction();
        stats.record_eviction();
        assert_eq!(stats.evictions.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn test_cache_stats_clone() {
        let stats = CacheStats::new();
        stats.record_hit();
        stats.add_bytes(100);

        let cloned = stats.clone();

        // Both share the same underlying atomics via Arc
        assert_eq!(cloned.hits.load(Ordering::Relaxed), 1);
        assert_eq!(cloned.total_bytes.load(Ordering::Relaxed), 100);

        // Modifying one affects the other
        stats.record_hit();
        assert_eq!(cloned.hits.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_cache_stats_debug() {
        let stats = CacheStats::new();
        let debug_str = format!("{:?}", stats);

        assert!(debug_str.contains("CacheStats"));
        assert!(debug_str.contains("hits"));
        assert!(debug_str.contains("misses"));
    }

    // =========================================================================
    // Strategy Max Size Tests
    // =========================================================================

    #[test]
    fn test_all_strategies_have_positive_max_size() {
        assert!(AstCacheStrategy.max_size() > 0);
        assert!(TemplateCacheStrategy.max_size() > 0);
        assert!(DagCacheStrategy.max_size() > 0);
        assert!(ChurnCacheStrategy.max_size() > 0);
        assert!(GitStatsCacheStrategy.max_size() > 0);
    }

    #[test]
    fn test_all_strategies_have_ttl() {
        assert!(AstCacheStrategy.ttl().is_some());
        assert!(TemplateCacheStrategy.ttl().is_some());
        assert!(DagCacheStrategy.ttl().is_some());
        assert!(ChurnCacheStrategy.ttl().is_some());
        assert!(GitStatsCacheStrategy.ttl().is_some());
    }

    // =========================================================================
    // AstCacheManager Trait Tests
    // =========================================================================

    #[test]
    fn test_ast_cache_manager_trait_exists() {
        // Just verify the trait is accessible
        fn _accepts_ast_cache_manager<T: AstCacheManager>(_: &T) {}
    }
}

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
            prop_assert!(age.as_nanos() >= 0);
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
