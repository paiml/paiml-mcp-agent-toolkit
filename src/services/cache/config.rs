#![cfg_attr(coverage_nightly, coverage(off))]
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,

    /// Enable file watching for invalidation
    pub enable_watch: bool,

    /// Cache TTLs
    pub ast_ttl_secs: u64,
    pub template_ttl_secs: u64,
    pub dag_ttl_secs: u64,
    pub churn_ttl_secs: u64,
    pub git_stats_ttl_secs: u64,

    /// Warmup settings
    pub warmup_on_startup: bool,
    pub warmup_patterns: Vec<String>,

    /// Git-specific settings
    pub git_cache_by_branch: bool,
    pub git_cache_max_age_days: u32,

    /// Performance tuning
    pub parallel_warmup_threads: usize,
    pub cache_compression: bool,
    pub eviction_batch_size: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 100,
            enable_watch: true,

            // TTLs in seconds
            ast_ttl_secs: 300,       // 5 minutes
            template_ttl_secs: 600,  // 10 minutes
            dag_ttl_secs: 180,       // 3 minutes
            churn_ttl_secs: 1800,    // 30 minutes
            git_stats_ttl_secs: 900, // 15 minutes

            warmup_on_startup: false,
            warmup_patterns: vec![
                "src/**/*.rs".to_string(),
                "**/*.ts".to_string(),
                "**/*.py".to_string(),
            ],

            git_cache_by_branch: true,
            git_cache_max_age_days: 7,

            parallel_warmup_threads: 4,
            cache_compression: false,
            eviction_batch_size: 10,
        }
    }
}

// CacheConfig methods: from_env(), TTL duration conversions, max_memory_bytes()
include!("config_methods.rs");

// Unit tests: defaults, TTL conversions, memory, env vars, serde, clone/debug, edge cases
include!("config_tests.rs");

// Property-based tests: TTL roundtrips, memory calculation, serialization, clone, relationships
include!("config_property_tests.rs");
