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

impl CacheConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Override with environment variables if set
        if let Ok(val) = std::env::var("PAIML_CACHE_MAX_MB") {
            if let Ok(mb) = val.parse() {
                config.max_memory_mb = mb;
            }
        }

        if let Ok(val) = std::env::var("PAIML_CACHE_TTL_AST") {
            if let Ok(secs) = val.parse() {
                config.ast_ttl_secs = secs;
            }
        }

        if let Ok(val) = std::env::var("PAIML_CACHE_ENABLE_WATCH") {
            config.enable_watch = val.to_lowercase() == "true" || val == "1";
        }

        if let Ok(val) = std::env::var("PAIML_CACHE_GIT_BRANCH_AWARE") {
            config.git_cache_by_branch = val.to_lowercase() == "true" || val == "1";
        }

        config
    }

    /// Get AST TTL as Duration
    pub fn ast_ttl(&self) -> Duration {
        Duration::from_secs(self.ast_ttl_secs)
    }

    /// Get template TTL as Duration
    pub fn template_ttl(&self) -> Duration {
        Duration::from_secs(self.template_ttl_secs)
    }

    /// Get DAG TTL as Duration
    pub fn dag_ttl(&self) -> Duration {
        Duration::from_secs(self.dag_ttl_secs)
    }

    /// Get churn TTL as Duration
    pub fn churn_ttl(&self) -> Duration {
        Duration::from_secs(self.churn_ttl_secs)
    }

    /// Get git stats TTL as Duration
    pub fn git_stats_ttl(&self) -> Duration {
        Duration::from_secs(self.git_stats_ttl_secs)
    }

    /// Calculate max memory in bytes
    pub fn max_memory_bytes(&self) -> usize {
        self.max_memory_mb * 1024 * 1024
    }
}
