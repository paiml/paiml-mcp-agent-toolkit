use crate::services::cache::{
    config::CacheConfig,
    diagnostics::{CacheDiagnostics, CacheEffectiveness, CacheStatsSnapshot},
    persistent::PersistentCache,
    strategies::*,
};
use crate::services::context::FileContext;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

/// Persistent cache manager that stores cache data on disk
pub struct PersistentCacheManager {
    // Different cache types
    ast_cache: Arc<PersistentCache<AstCacheStrategy>>,

    // Global settings
    config: CacheConfig,
    session_id: Uuid,
    created: Instant,
    #[allow(dead_code)]
    cache_dir: PathBuf,
}

impl PersistentCacheManager {
    pub fn new(config: CacheConfig, cache_dir: PathBuf) -> Result<Self> {
        // Create individual cache directories
        let ast_cache_dir = cache_dir.join("ast");

        Ok(Self {
            ast_cache: Arc::new(PersistentCache::new(AstCacheStrategy, ast_cache_dir)?),
            config,
            session_id: Uuid::new_v4(),
            created: Instant::now(),
            cache_dir,
        })
    }

    /// Create with default cache directory
    pub fn with_default_dir(config: CacheConfig) -> Result<Self> {
        let cache_dir = Self::default_cache_dir()?;
        Self::new(config, cache_dir)
    }

    /// Get default cache directory
    pub fn default_cache_dir() -> Result<PathBuf> {
        if let Some(cache_dir) = dirs::cache_dir() {
            Ok(cache_dir.join("paiml-mcp-agent-toolkit"))
        } else if let Some(home_dir) = dirs::home_dir() {
            Ok(home_dir.join(".cache").join("paiml-mcp-agent-toolkit"))
        } else {
            // Fallback to /tmp
            Ok(PathBuf::from("/tmp/paiml-mcp-agent-toolkit-cache"))
        }
    }

    /// Get or compute AST with caching
    pub async fn get_or_compute_ast<F, Fut>(
        &self,
        path: &Path,
        compute: F,
    ) -> Result<Arc<FileContext>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<FileContext>>,
    {
        let path_buf = path.to_path_buf();

        // Try cache first
        if let Some(ast) = self.ast_cache.get(&path_buf) {
            return Ok(ast);
        }

        // Compute and cache
        let ast = compute().await?;
        let _ = self.ast_cache.put(path_buf, ast.clone());
        Ok(Arc::new(ast))
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) {
        self.ast_cache.cleanup_expired();
    }

    /// Clear all caches
    pub fn clear(&self) {
        let _ = self.ast_cache.clear();
    }

    /// Get cache diagnostics
    pub fn get_diagnostics(&self) -> CacheDiagnostics {
        let uptime = self.created.elapsed();
        let ast_size = self.ast_cache.stats.memory_usage();

        let memory_usage_mb = ast_size as f64 / (1024.0 * 1024.0);
        let memory_pressure = if self.config.max_memory_mb > 0 {
            (memory_usage_mb / self.config.max_memory_mb as f64).min(1.0) as f32
        } else {
            0.0
        };

        // Trigger cleanup if memory pressure is high
        if memory_pressure > 0.8 {
            self.ast_cache.cleanup_expired();
        }

        let cache_stats = vec![(
            "ast".to_string(),
            CacheStatsSnapshot::from((&self.ast_cache.stats, self.ast_cache.len())),
        )];

        // Calculate effectiveness
        let total_operations = cache_stats
            .iter()
            .map(|(_, stats)| stats.hits + stats.misses)
            .sum::<u64>();

        let total_hits = cache_stats.iter().map(|(_, stats)| stats.hits).sum::<u64>();

        let overall_hit_rate = if total_operations > 0 {
            total_hits as f64 / total_operations as f64
        } else {
            0.0
        };

        let memory_efficiency = 1.0 - memory_pressure as f64;

        // Estimate time saved (simplified calculation)
        let time_saved_ms = total_hits * 100; // Assume 100ms saved per cache hit

        let most_valuable_caches = vec![("ast".to_string(), total_hits as f64)];

        let effectiveness = CacheEffectiveness {
            overall_hit_rate,
            memory_efficiency,
            time_saved_ms,
            most_valuable_caches,
        };

        CacheDiagnostics {
            session_id: self.session_id,
            uptime,
            memory_usage_mb,
            memory_pressure,
            cache_stats,
            hot_paths: Vec::new(), // TODO: Implement hot path tracking
            effectiveness,
        }
    }
}
