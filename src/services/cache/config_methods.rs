// CacheConfig method implementations: environment loading, TTL conversions, memory calculation

impl CacheConfig {
    /// Load configuration from environment variables
    #[must_use]
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
    #[must_use]
    pub fn ast_ttl(&self) -> Duration {
        Duration::from_secs(self.ast_ttl_secs)
    }

    /// Get template TTL as Duration
    #[must_use]
    pub fn template_ttl(&self) -> Duration {
        Duration::from_secs(self.template_ttl_secs)
    }

    /// Get DAG TTL as Duration
    #[must_use]
    pub fn dag_ttl(&self) -> Duration {
        Duration::from_secs(self.dag_ttl_secs)
    }

    /// Get churn TTL as Duration
    #[must_use]
    pub fn churn_ttl(&self) -> Duration {
        Duration::from_secs(self.churn_ttl_secs)
    }

    /// Get git stats TTL as Duration
    #[must_use]
    pub fn git_stats_ttl(&self) -> Duration {
        Duration::from_secs(self.git_stats_ttl_secs)
    }

    /// Calculate max memory in bytes
    #[must_use]
    pub fn max_memory_bytes(&self) -> usize {
        self.max_memory_mb * 1024 * 1024
    }
}
