// Property-based tests for CacheConfig: TTL conversions, memory calculation, serialization, clone, relationships

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::time::Duration;

    // =========================================================================
    // Property-Based Tests for TTL Conversions
    // =========================================================================

    proptest! {
        #[test]
        fn ttl_conversion_roundtrip(secs in 0u64..10_000_000) {
            let mut config = CacheConfig::default();
            config.ast_ttl_secs = secs;

            let duration = config.ast_ttl();
            prop_assert_eq!(duration.as_secs(), secs);
        }

        #[test]
        fn all_ttls_are_consistent(
            ast in 0u64..1_000_000,
            template in 0u64..1_000_000,
            dag in 0u64..1_000_000,
            churn in 0u64..1_000_000,
            git_stats in 0u64..1_000_000
        ) {
            let mut config = CacheConfig::default();
            config.ast_ttl_secs = ast;
            config.template_ttl_secs = template;
            config.dag_ttl_secs = dag;
            config.churn_ttl_secs = churn;
            config.git_stats_ttl_secs = git_stats;

            prop_assert_eq!(config.ast_ttl(), Duration::from_secs(ast));
            prop_assert_eq!(config.template_ttl(), Duration::from_secs(template));
            prop_assert_eq!(config.dag_ttl(), Duration::from_secs(dag));
            prop_assert_eq!(config.churn_ttl(), Duration::from_secs(churn));
            prop_assert_eq!(config.git_stats_ttl(), Duration::from_secs(git_stats));
        }
    }

    // =========================================================================
    // Property-Based Tests for Memory Calculation
    // =========================================================================

    proptest! {
        #[test]
        fn memory_bytes_calculation_correct(mb in 0usize..65536) {
            let mut config = CacheConfig::default();
            config.max_memory_mb = mb;

            let expected = mb.saturating_mul(1024).saturating_mul(1024);
            prop_assert_eq!(config.max_memory_bytes(), expected);
        }

        #[test]
        fn memory_bytes_never_overflows_for_reasonable_values(mb in 0usize..1048576) {
            let mut config = CacheConfig::default();
            config.max_memory_mb = mb;

            // Should not panic
            let _ = config.max_memory_bytes();
        }
    }

    // =========================================================================
    // Property-Based Tests for Serialization
    // =========================================================================

    proptest! {
        #[test]
        fn serialization_preserves_memory(mb in 0usize..65536) {
            let mut config = CacheConfig::default();
            config.max_memory_mb = mb;

            let json = serde_json::to_string(&config).unwrap();
            let restored: CacheConfig = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(restored.max_memory_mb, mb);
        }

        #[test]
        fn serialization_preserves_ttls(
            ast in 0u64..1_000_000,
            template in 0u64..1_000_000,
            dag in 0u64..1_000_000
        ) {
            let mut config = CacheConfig::default();
            config.ast_ttl_secs = ast;
            config.template_ttl_secs = template;
            config.dag_ttl_secs = dag;

            let json = serde_json::to_string(&config).unwrap();
            let restored: CacheConfig = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(restored.ast_ttl_secs, ast);
            prop_assert_eq!(restored.template_ttl_secs, template);
            prop_assert_eq!(restored.dag_ttl_secs, dag);
        }

        #[test]
        fn serialization_preserves_booleans(
            enable_watch in proptest::bool::ANY,
            warmup_on_startup in proptest::bool::ANY,
            git_cache_by_branch in proptest::bool::ANY,
            cache_compression in proptest::bool::ANY
        ) {
            let mut config = CacheConfig::default();
            config.enable_watch = enable_watch;
            config.warmup_on_startup = warmup_on_startup;
            config.git_cache_by_branch = git_cache_by_branch;
            config.cache_compression = cache_compression;

            let json = serde_json::to_string(&config).unwrap();
            let restored: CacheConfig = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(restored.enable_watch, enable_watch);
            prop_assert_eq!(restored.warmup_on_startup, warmup_on_startup);
            prop_assert_eq!(restored.git_cache_by_branch, git_cache_by_branch);
            prop_assert_eq!(restored.cache_compression, cache_compression);
        }
    }

    // =========================================================================
    // Property-Based Tests for Clone
    // =========================================================================

    proptest! {
        #[test]
        fn clone_produces_equal_config(
            mb in 0usize..65536,
            enable_watch in proptest::bool::ANY,
            threads in 1usize..128
        ) {
            let mut config = CacheConfig::default();
            config.max_memory_mb = mb;
            config.enable_watch = enable_watch;
            config.parallel_warmup_threads = threads;

            let cloned = config.clone();

            prop_assert_eq!(cloned.max_memory_mb, config.max_memory_mb);
            prop_assert_eq!(cloned.enable_watch, config.enable_watch);
            prop_assert_eq!(cloned.parallel_warmup_threads, config.parallel_warmup_threads);
        }
    }

    // =========================================================================
    // Property-Based Tests for Configuration Relationships
    // =========================================================================

    proptest! {
        #[test]
        fn ttl_ordering_preserved(
            short in 1u64..100,
            medium in 100u64..1000,
            long in 1000u64..10000
        ) {
            let mut config = CacheConfig::default();
            config.ast_ttl_secs = short;
            config.template_ttl_secs = medium;
            config.churn_ttl_secs = long;

            prop_assert!(config.ast_ttl() < config.template_ttl());
            prop_assert!(config.template_ttl() < config.churn_ttl());
        }

        #[test]
        fn warmup_patterns_preserved(count in 0usize..10) {
            let mut config = CacheConfig::default();
            let patterns: Vec<String> = (0..count)
                .map(|i| format!("pattern_{}", i))
                .collect();
            config.warmup_patterns = patterns.clone();

            let json = serde_json::to_string(&config).unwrap();
            let restored: CacheConfig = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(restored.warmup_patterns.len(), count);
            for (i, pattern) in restored.warmup_patterns.iter().enumerate() {
                prop_assert_eq!(pattern, &format!("pattern_{}", i));
            }
        }
    }
}
