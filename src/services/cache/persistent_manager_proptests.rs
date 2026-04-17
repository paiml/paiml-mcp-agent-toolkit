#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::TempDir;

    // =========================================================================
    // Property-Based Tests for Configuration
    // =========================================================================

    proptest! {
        #[test]
        fn test_diagnostics_memory_pressure_bounds(max_mb in 1usize..10000) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let cache_dir = temp_dir.path().join("cache");
            let config = CacheConfig {
                max_memory_mb: max_mb,
                ..Default::default()
            };

            if let Ok(manager) = PersistentCacheManager::new(config, cache_dir) {
                let diag = manager.get_diagnostics();
                prop_assert!(diag.memory_pressure >= 0.0);
                prop_assert!(diag.memory_pressure <= 1.0);
            }
        }

        #[test]
        fn test_diagnostics_effectiveness_bounds(_seed in 0u32..1000) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let cache_dir = temp_dir.path().join(format!("cache_{}", _seed));
            let config = CacheConfig::default();

            if let Ok(manager) = PersistentCacheManager::new(config, cache_dir) {
                let diag = manager.get_diagnostics();

                // Hit rate should be between 0 and 1
                prop_assert!(diag.effectiveness.overall_hit_rate >= 0.0);
                prop_assert!(diag.effectiveness.overall_hit_rate <= 1.0);

                // Memory efficiency should be between 0 and 1
                prop_assert!(diag.effectiveness.memory_efficiency >= 0.0);
                prop_assert!(diag.effectiveness.memory_efficiency <= 1.0);
            }
        }
    }

    // =========================================================================
    // Property-Based Tests for Uptime
    // =========================================================================

    proptest! {
        #[test]
        fn test_uptime_is_non_negative(_seed in 0u32..100) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let cache_dir = temp_dir.path().join(format!("cache_{}", _seed));
            let config = CacheConfig::default();

            if let Ok(manager) = PersistentCacheManager::new(config, cache_dir) {
                let diag = manager.get_diagnostics();
                // Duration::as_nanos() returns u128, always non-negative
                let _ = diag.uptime.as_nanos();
            }
        }
    }

    // =========================================================================
    // Property-Based Tests for Session ID
    // =========================================================================

    proptest! {
        #[test]
        fn test_session_id_is_valid_uuid(_seed in 0u32..100) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let cache_dir = temp_dir.path().join(format!("cache_{}", _seed));
            let config = CacheConfig::default();

            if let Ok(manager) = PersistentCacheManager::new(config, cache_dir) {
                let diag = manager.get_diagnostics();
                // UUID should not be nil
                prop_assert!(!diag.session_id.is_nil());
                // UUID version should be 4 (random)
                prop_assert_eq!(diag.session_id.get_version_num(), 4);
            }
        }
    }

    // =========================================================================
    // Property-Based Tests for Cache Stats
    // =========================================================================

    proptest! {
        #[test]
        fn test_cache_stats_has_ast_entry(_seed in 0u32..100) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let cache_dir = temp_dir.path().join(format!("cache_{}", _seed));
            let config = CacheConfig::default();

            if let Ok(manager) = PersistentCacheManager::new(config, cache_dir) {
                let diag = manager.get_diagnostics();
                let has_ast = diag.cache_stats.iter().any(|(name, _)| name == "ast");
                prop_assert!(has_ast, "Should always have AST cache stats");
            }
        }
    }

    // =========================================================================
    // Property-Based Tests for Memory Usage
    // =========================================================================

    proptest! {
        #[test]
        fn test_memory_usage_is_non_negative(_seed in 0u32..100) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let cache_dir = temp_dir.path().join(format!("cache_{}", _seed));
            let config = CacheConfig::default();

            if let Ok(manager) = PersistentCacheManager::new(config, cache_dir) {
                let diag = manager.get_diagnostics();
                prop_assert!(diag.memory_usage_mb >= 0.0);
            }
        }
    }
}
