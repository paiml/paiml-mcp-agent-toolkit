// Unit tests for CacheConfig: defaults, TTL conversions, memory, env vars, serde, clone/debug, edge cases

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // =========================================================================
    // Default Configuration Tests
    // =========================================================================

    #[test]
    fn test_cache_config_default_values() {
        let config = CacheConfig::default();

        // Memory settings
        assert_eq!(config.max_memory_mb, 100);

        // Watch settings
        assert!(config.enable_watch);

        // TTL values in seconds
        assert_eq!(config.ast_ttl_secs, 300); // 5 minutes
        assert_eq!(config.template_ttl_secs, 600); // 10 minutes
        assert_eq!(config.dag_ttl_secs, 180); // 3 minutes
        assert_eq!(config.churn_ttl_secs, 1800); // 30 minutes
        assert_eq!(config.git_stats_ttl_secs, 900); // 15 minutes

        // Warmup settings
        assert!(!config.warmup_on_startup);
        assert_eq!(config.warmup_patterns.len(), 3);
        assert!(config.warmup_patterns.contains(&"src/**/*.rs".to_string()));
        assert!(config.warmup_patterns.contains(&"**/*.ts".to_string()));
        assert!(config.warmup_patterns.contains(&"**/*.py".to_string()));

        // Git settings
        assert!(config.git_cache_by_branch);
        assert_eq!(config.git_cache_max_age_days, 7);

        // Performance tuning
        assert_eq!(config.parallel_warmup_threads, 4);
        assert!(!config.cache_compression);
        assert_eq!(config.eviction_batch_size, 10);
    }

    // =========================================================================
    // TTL Duration Conversion Tests
    // =========================================================================

    #[test]
    fn test_ast_ttl_conversion() {
        let config = CacheConfig::default();
        assert_eq!(config.ast_ttl(), Duration::from_secs(300));

        let custom_config = CacheConfig {
            ast_ttl_secs: 60,
            ..Default::default()
        };
        assert_eq!(custom_config.ast_ttl(), Duration::from_secs(60));
    }

    #[test]
    fn test_template_ttl_conversion() {
        let config = CacheConfig::default();
        assert_eq!(config.template_ttl(), Duration::from_secs(600));

        let custom_config = CacheConfig {
            template_ttl_secs: 1200,
            ..Default::default()
        };
        assert_eq!(custom_config.template_ttl(), Duration::from_secs(1200));
    }

    #[test]
    fn test_dag_ttl_conversion() {
        let config = CacheConfig::default();
        assert_eq!(config.dag_ttl(), Duration::from_secs(180));

        let custom_config = CacheConfig {
            dag_ttl_secs: 360,
            ..Default::default()
        };
        assert_eq!(custom_config.dag_ttl(), Duration::from_secs(360));
    }

    #[test]
    fn test_churn_ttl_conversion() {
        let config = CacheConfig::default();
        assert_eq!(config.churn_ttl(), Duration::from_secs(1800));

        let custom_config = CacheConfig {
            churn_ttl_secs: 3600,
            ..Default::default()
        };
        assert_eq!(custom_config.churn_ttl(), Duration::from_secs(3600));
    }

    #[test]
    fn test_git_stats_ttl_conversion() {
        let config = CacheConfig::default();
        assert_eq!(config.git_stats_ttl(), Duration::from_secs(900));

        let custom_config = CacheConfig {
            git_stats_ttl_secs: 450,
            ..Default::default()
        };
        assert_eq!(custom_config.git_stats_ttl(), Duration::from_secs(450));
    }

    // =========================================================================
    // Memory Calculation Tests
    // =========================================================================

    #[test]
    fn test_max_memory_bytes_calculation() {
        let config = CacheConfig::default();
        // 100 MB = 100 * 1024 * 1024 = 104_857_600 bytes
        assert_eq!(config.max_memory_bytes(), 104_857_600);

        let mut custom_config = CacheConfig {
            max_memory_mb: 256,
            ..Default::default()
        };
        assert_eq!(custom_config.max_memory_bytes(), 256 * 1024 * 1024);

        custom_config.max_memory_mb = 1;
        assert_eq!(custom_config.max_memory_bytes(), 1024 * 1024);

        custom_config.max_memory_mb = 0;
        assert_eq!(custom_config.max_memory_bytes(), 0);
    }

    // =========================================================================
    // Environment Variable Tests
    // =========================================================================

    // =========================================================================
    // Environment variable tests.
    //
    // `from_env` reads process-global state, so these MUST NOT run concurrently
    // with each other: `serial_test` serializes them under a shared key, and
    // `EnvGuard` clears every PAIML_CACHE_* var on entry *and* on drop so a
    // panicking test cannot leak state into the next one. Before this, the
    // suite worked around the race by asserting nothing (`let _ = field`) or by
    // accepting "either the set value or the default", which meant the parsing
    // logic was effectively untested — and `test_from_env_with_no_env_vars`,
    // the one test that did assert, failed intermittently.
    // =========================================================================

    /// Clears all cache env vars on construction and on drop.
    struct EnvGuard;

    impl EnvGuard {
        fn new() -> Self {
            Self::clear();
            Self
        }

        fn clear() {
            for key in [
                "PAIML_CACHE_MAX_MB",
                "PAIML_CACHE_TTL_AST",
                "PAIML_CACHE_ENABLE_WATCH",
                "PAIML_CACHE_GIT_BRANCH_AWARE",
            ] {
                std::env::remove_var(key);
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            Self::clear();
        }
    }

    #[test]
    #[serial_test::serial(paiml_cache_env)]
    fn test_from_env_with_no_env_vars() {
        let _guard = EnvGuard::new();

        let config = CacheConfig::from_env();

        assert_eq!(config.max_memory_mb, 100);
        assert_eq!(config.ast_ttl_secs, 300);
        assert!(config.enable_watch);
        assert!(config.git_cache_by_branch);
    }

    #[test]
    #[serial_test::serial(paiml_cache_env)]
    fn test_from_env_with_max_mb_override() {
        let _guard = EnvGuard::new();
        std::env::set_var("PAIML_CACHE_MAX_MB", "512");

        assert_eq!(CacheConfig::from_env().max_memory_mb, 512);
    }

    #[test]
    #[serial_test::serial(paiml_cache_env)]
    fn test_from_env_with_invalid_max_mb() {
        let _guard = EnvGuard::new();
        std::env::set_var("PAIML_CACHE_MAX_MB", "not_a_number");

        // Unparseable values are ignored, leaving the default intact.
        assert_eq!(CacheConfig::from_env().max_memory_mb, 100);
    }

    #[test]
    #[serial_test::serial(paiml_cache_env)]
    fn test_from_env_with_ast_ttl_override() {
        let _guard = EnvGuard::new();
        std::env::set_var("PAIML_CACHE_TTL_AST", "120");

        assert_eq!(CacheConfig::from_env().ast_ttl_secs, 120);
    }

    #[test]
    #[serial_test::serial(paiml_cache_env)]
    fn test_from_env_with_invalid_ast_ttl() {
        let _guard = EnvGuard::new();
        std::env::set_var("PAIML_CACHE_TTL_AST", "invalid");

        assert_eq!(CacheConfig::from_env().ast_ttl_secs, 300);
    }

    #[test]
    #[serial_test::serial(paiml_cache_env)]
    fn test_from_env_enable_watch_true() {
        let _guard = EnvGuard::new();
        std::env::set_var("PAIML_CACHE_ENABLE_WATCH", "true");

        assert!(CacheConfig::from_env().enable_watch);
    }

    #[test]
    #[serial_test::serial(paiml_cache_env)]
    fn test_from_env_enable_watch_false() {
        let _guard = EnvGuard::new();
        std::env::set_var("PAIML_CACHE_ENABLE_WATCH", "false");

        assert!(!CacheConfig::from_env().enable_watch);
    }

    #[test]
    #[serial_test::serial(paiml_cache_env)]
    fn test_from_env_enable_watch_one() {
        let _guard = EnvGuard::new();
        std::env::set_var("PAIML_CACHE_ENABLE_WATCH", "1");

        assert!(CacheConfig::from_env().enable_watch);
    }

    #[test]
    #[serial_test::serial(paiml_cache_env)]
    fn test_from_env_enable_watch_zero() {
        let _guard = EnvGuard::new();
        std::env::set_var("PAIML_CACHE_ENABLE_WATCH", "0");

        // Only "true" (any case) and "1" enable it; everything else disables.
        assert!(!CacheConfig::from_env().enable_watch);
    }

    #[test]
    #[serial_test::serial(paiml_cache_env)]
    fn test_from_env_enable_watch_uppercase() {
        let _guard = EnvGuard::new();
        std::env::set_var("PAIML_CACHE_ENABLE_WATCH", "TRUE");

        assert!(CacheConfig::from_env().enable_watch);
    }

    #[test]
    #[serial_test::serial(paiml_cache_env)]
    fn test_from_env_git_branch_aware_true() {
        let _guard = EnvGuard::new();
        std::env::set_var("PAIML_CACHE_GIT_BRANCH_AWARE", "true");

        assert!(CacheConfig::from_env().git_cache_by_branch);
    }

    #[test]
    #[serial_test::serial(paiml_cache_env)]
    fn test_from_env_git_branch_aware_false() {
        let _guard = EnvGuard::new();
        std::env::set_var("PAIML_CACHE_GIT_BRANCH_AWARE", "false");

        assert!(!CacheConfig::from_env().git_cache_by_branch);
    }

    #[test]
    #[serial_test::serial(paiml_cache_env)]
    fn test_from_env_git_branch_aware_one() {
        let _guard = EnvGuard::new();
        std::env::set_var("PAIML_CACHE_GIT_BRANCH_AWARE", "1");

        assert!(CacheConfig::from_env().git_cache_by_branch);
    }

    /// The guard must leave the environment clean even when a test panics,
    /// otherwise one failure cascades into unrelated ones.
    #[test]
    #[serial_test::serial(paiml_cache_env)]
    fn test_env_guard_clears_on_unwind() {
        let _outer = EnvGuard::new();

        let result = std::panic::catch_unwind(|| {
            let _guard = EnvGuard::new();
            std::env::set_var("PAIML_CACHE_MAX_MB", "999");
            panic!("simulated test failure");
        });

        assert!(result.is_err());
        assert!(std::env::var("PAIML_CACHE_MAX_MB").is_err());
        assert_eq!(CacheConfig::from_env().max_memory_mb, 100);
    }

    // =========================================================================
    // Serialization/Deserialization Tests
    // =========================================================================

    #[test]
    fn test_cache_config_serialization() {
        let config = CacheConfig::default();
        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("\"max_memory_mb\":100"));
        assert!(json.contains("\"enable_watch\":true"));
        assert!(json.contains("\"ast_ttl_secs\":300"));
    }

    #[test]
    fn test_cache_config_deserialization() {
        let json = r#"{
            "max_memory_mb": 256,
            "enable_watch": false,
            "ast_ttl_secs": 120,
            "template_ttl_secs": 300,
            "dag_ttl_secs": 90,
            "churn_ttl_secs": 900,
            "git_stats_ttl_secs": 450,
            "warmup_on_startup": true,
            "warmup_patterns": ["*.rs"],
            "git_cache_by_branch": false,
            "git_cache_max_age_days": 14,
            "parallel_warmup_threads": 8,
            "cache_compression": true,
            "eviction_batch_size": 20
        }"#;

        let config: CacheConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.max_memory_mb, 256);
        assert!(!config.enable_watch);
        assert_eq!(config.ast_ttl_secs, 120);
        assert_eq!(config.template_ttl_secs, 300);
        assert_eq!(config.dag_ttl_secs, 90);
        assert_eq!(config.churn_ttl_secs, 900);
        assert_eq!(config.git_stats_ttl_secs, 450);
        assert!(config.warmup_on_startup);
        assert_eq!(config.warmup_patterns, vec!["*.rs".to_string()]);
        assert!(!config.git_cache_by_branch);
        assert_eq!(config.git_cache_max_age_days, 14);
        assert_eq!(config.parallel_warmup_threads, 8);
        assert!(config.cache_compression);
        assert_eq!(config.eviction_batch_size, 20);
    }

    #[test]
    fn test_cache_config_roundtrip() {
        let original = CacheConfig {
            max_memory_mb: 512,
            enable_watch: false,
            ast_ttl_secs: 60,
            template_ttl_secs: 120,
            dag_ttl_secs: 30,
            churn_ttl_secs: 600,
            git_stats_ttl_secs: 300,
            warmup_on_startup: true,
            warmup_patterns: vec!["**/*.go".to_string(), "**/*.java".to_string()],
            git_cache_by_branch: false,
            git_cache_max_age_days: 30,
            parallel_warmup_threads: 16,
            cache_compression: true,
            eviction_batch_size: 50,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: CacheConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.max_memory_mb, original.max_memory_mb);
        assert_eq!(deserialized.enable_watch, original.enable_watch);
        assert_eq!(deserialized.ast_ttl_secs, original.ast_ttl_secs);
        assert_eq!(deserialized.template_ttl_secs, original.template_ttl_secs);
        assert_eq!(deserialized.dag_ttl_secs, original.dag_ttl_secs);
        assert_eq!(deserialized.churn_ttl_secs, original.churn_ttl_secs);
        assert_eq!(deserialized.git_stats_ttl_secs, original.git_stats_ttl_secs);
        assert_eq!(deserialized.warmup_on_startup, original.warmup_on_startup);
        assert_eq!(deserialized.warmup_patterns, original.warmup_patterns);
        assert_eq!(
            deserialized.git_cache_by_branch,
            original.git_cache_by_branch
        );
        assert_eq!(
            deserialized.git_cache_max_age_days,
            original.git_cache_max_age_days
        );
        assert_eq!(
            deserialized.parallel_warmup_threads,
            original.parallel_warmup_threads
        );
        assert_eq!(deserialized.cache_compression, original.cache_compression);
        assert_eq!(
            deserialized.eviction_batch_size,
            original.eviction_batch_size
        );
    }

    // =========================================================================
    // Clone and Debug Tests
    // =========================================================================

    #[test]
    fn test_cache_config_clone() {
        let original = CacheConfig::default();
        let cloned = original.clone();

        assert_eq!(cloned.max_memory_mb, original.max_memory_mb);
        assert_eq!(cloned.enable_watch, original.enable_watch);
        assert_eq!(cloned.warmup_patterns, original.warmup_patterns);
    }

    #[test]
    fn test_cache_config_debug() {
        let config = CacheConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("CacheConfig"));
        assert!(debug_str.contains("max_memory_mb"));
        assert!(debug_str.contains("enable_watch"));
    }

    // =========================================================================
    // Edge Case Tests
    // =========================================================================

    #[test]
    fn test_zero_ttl_values() {
        let config = CacheConfig {
            ast_ttl_secs: 0,
            template_ttl_secs: 0,
            dag_ttl_secs: 0,
            churn_ttl_secs: 0,
            git_stats_ttl_secs: 0,
            ..Default::default()
        };

        assert_eq!(config.ast_ttl(), Duration::from_secs(0));
        assert_eq!(config.template_ttl(), Duration::from_secs(0));
        assert_eq!(config.dag_ttl(), Duration::from_secs(0));
        assert_eq!(config.churn_ttl(), Duration::from_secs(0));
        assert_eq!(config.git_stats_ttl(), Duration::from_secs(0));
    }

    #[test]
    fn test_large_ttl_values() {
        let config = CacheConfig {
            ast_ttl_secs: u64::MAX,
            ..Default::default()
        };

        assert_eq!(config.ast_ttl(), Duration::from_secs(u64::MAX));
    }

    #[test]
    fn test_empty_warmup_patterns() {
        let config = CacheConfig {
            warmup_patterns: vec![],
            ..Default::default()
        };

        assert!(config.warmup_patterns.is_empty());
    }

    #[test]
    fn test_large_memory_configuration() {
        let config = CacheConfig {
            max_memory_mb: 16384, // 16 GB
            ..Default::default()
        };

        assert_eq!(config.max_memory_bytes(), 16384 * 1024 * 1024);
    }
}
