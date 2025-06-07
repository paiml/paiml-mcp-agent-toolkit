use crate::cli::DagType;
use crate::models::churn::CodeChurnAnalysis;
use crate::models::dag::DependencyGraph;
use crate::models::template::TemplateResource;
use crate::services::cache::{
    adapters::{ContentCacheAdapter, PersistentCacheAdapter},
    content_cache::ContentCache,
    persistent::PersistentCache,
    strategies::{
        AstCacheStrategy, ChurnCacheStrategy, DagCacheStrategy, GitStats, GitStatsCacheStrategy,
        TemplateCacheStrategy,
    },
    unified::{LayeredCache, UnifiedCache, UnifiedCacheConfig},
};
use crate::services::context::FileContext;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

/// Unified cache manager that uses the new UnifiedCache trait
/// This replaces SessionCacheManager and PersistentCacheManager
#[derive(Clone)]
pub struct UnifiedCacheManager {
    // AST cache with layered storage
    ast_cache: LayeredCache<
        ContentCacheAdapter<AstCacheStrategy>,
        PersistentCacheAdapter<AstCacheStrategy>,
    >,

    // Template cache (memory only)
    template_cache: ContentCacheAdapter<TemplateCacheStrategy>,

    // DAG cache with layered storage
    dag_cache: LayeredCache<
        ContentCacheAdapter<DagCacheStrategy>,
        PersistentCacheAdapter<DagCacheStrategy>,
    >,

    // Churn cache (memory only due to git dependency)
    churn_cache: ContentCacheAdapter<ChurnCacheStrategy>,

    // Git stats cache (memory only)
    git_stats_cache: ContentCacheAdapter<GitStatsCacheStrategy>,

    // Configuration
    config: UnifiedCacheConfig,
}

impl UnifiedCacheManager {
    /// Create a new unified cache manager
    pub fn new(config: UnifiedCacheConfig) -> Result<Self> {
        let cache_dir = config
            .storage_path
            .clone()
            .unwrap_or_else(|| ".pmat_cache".to_string());
        let cache_dir = std::path::PathBuf::from(&cache_dir);

        // Create AST cache with layered storage
        let ast_memory = ContentCacheAdapter::new(ContentCache::new(AstCacheStrategy));
        let ast_persistent = if config.persistent {
            Some(PersistentCacheAdapter::new(PersistentCache::new(
                AstCacheStrategy,
                cache_dir.join("ast"),
            )?))
        } else {
            None
        };
        let ast_cache = LayeredCache::new(ast_memory, ast_persistent);

        // Create template cache (memory only)
        let template_cache = ContentCacheAdapter::new(ContentCache::new(TemplateCacheStrategy));

        // Create DAG cache with layered storage
        let dag_memory = ContentCacheAdapter::new(ContentCache::new(DagCacheStrategy));
        let dag_persistent = if config.persistent {
            Some(PersistentCacheAdapter::new(PersistentCache::new(
                DagCacheStrategy,
                cache_dir.join("dag"),
            )?))
        } else {
            None
        };
        let dag_cache = LayeredCache::new(dag_memory, dag_persistent);

        // Create churn cache (memory only)
        let churn_cache = ContentCacheAdapter::new(ContentCache::new(ChurnCacheStrategy));

        // Create git stats cache (memory only)
        let git_stats_cache = ContentCacheAdapter::new(ContentCache::new(GitStatsCacheStrategy));

        Ok(Self {
            ast_cache,
            template_cache,
            dag_cache,
            churn_cache,
            git_stats_cache,
            config,
        })
    }

    /// Get or compute AST analysis
    pub async fn get_or_compute_ast<F, Fut>(
        &self,
        path: &Path,
        compute: F,
    ) -> Result<Arc<FileContext>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<FileContext>> + Send,
    {
        self.ast_cache
            .get_or_compute(path.to_path_buf(), compute)
            .await
    }

    /// Get or compute template
    pub async fn get_or_compute_template<F, Fut>(
        &self,
        uri: &str,
        compute: F,
    ) -> Result<Arc<TemplateResource>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<TemplateResource>> + Send,
    {
        self.template_cache
            .get_or_compute(uri.to_string(), compute)
            .await
    }

    /// Get or compute DAG
    pub async fn get_or_compute_dag<F, Fut>(
        &self,
        path: &Path,
        dag_type: DagType,
        compute: F,
    ) -> Result<Arc<DependencyGraph>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<DependencyGraph>> + Send,
    {
        let key = (path.to_path_buf(), dag_type);
        self.dag_cache.get_or_compute(key, compute).await
    }

    /// Get or compute code churn analysis
    pub async fn get_or_compute_churn<F, Fut>(
        &self,
        repo: &Path,
        period_days: u32,
        compute: F,
    ) -> Result<Arc<CodeChurnAnalysis>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<CodeChurnAnalysis>> + Send,
    {
        let key = (repo.to_path_buf(), period_days);
        self.churn_cache.get_or_compute(key, compute).await
    }

    /// Get or compute git statistics
    pub async fn get_or_compute_git_stats<F, Fut>(
        &self,
        repo: &Path,
        compute: F,
    ) -> Result<Arc<GitStats>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<GitStats>> + Send,
    {
        self.git_stats_cache
            .get_or_compute(repo.to_path_buf(), compute)
            .await
    }

    /// Calculate total memory usage across all caches
    pub fn total_memory_usage(&self) -> usize {
        self.ast_cache.size_bytes()
            + self.template_cache.size_bytes()
            + self.dag_cache.size_bytes()
            + self.churn_cache.size_bytes()
            + self.git_stats_cache.size_bytes()
    }

    /// Calculate memory pressure (0.0 to 1.0)
    pub fn memory_pressure(&self) -> f32 {
        let total_bytes = self.total_memory_usage();
        total_bytes as f32 / self.config.max_memory_bytes as f32
    }

    /// Evict entries if memory pressure is high
    pub async fn evict_if_needed(&self) -> Result<()> {
        if self.memory_pressure() > 0.8 {
            // Evict from all caches
            self.ast_cache.evict_if_needed().await?;
            self.template_cache.evict_if_needed().await?;
            self.dag_cache.evict_if_needed().await?;
            self.churn_cache.evict_if_needed().await?;
            self.git_stats_cache.evict_if_needed().await?;
        }
        Ok(())
    }

    /// Clear all caches
    pub async fn clear_all(&self) -> Result<()> {
        self.ast_cache.clear().await?;
        self.template_cache.clear().await?;
        self.dag_cache.clear().await?;
        self.churn_cache.clear().await?;
        self.git_stats_cache.clear().await?;
        Ok(())
    }

    /// Get diagnostics for all caches
    pub fn diagnostics(&self) -> UnifiedCacheDiagnostics {
        UnifiedCacheDiagnostics {
            ast_stats: self.ast_cache.stats(),
            template_stats: self.template_cache.stats(),
            dag_stats: self.dag_cache.stats(),
            churn_stats: self.churn_cache.stats(),
            git_stats_stats: self.git_stats_cache.stats(),
            total_memory_bytes: self.total_memory_usage(),
            memory_pressure: self.memory_pressure(),
        }
    }
}

/// Diagnostics for the unified cache system
#[derive(Debug, Clone)]
pub struct UnifiedCacheDiagnostics {
    pub ast_stats: Arc<crate::services::cache::base::CacheStats>,
    pub template_stats: Arc<crate::services::cache::base::CacheStats>,
    pub dag_stats: Arc<crate::services::cache::base::CacheStats>,
    pub churn_stats: Arc<crate::services::cache::base::CacheStats>,
    pub git_stats_stats: Arc<crate::services::cache::base::CacheStats>,
    pub total_memory_bytes: usize,
    pub memory_pressure: f32,
}

impl UnifiedCacheDiagnostics {
    /// Get total hit rate across all caches
    pub fn overall_hit_rate(&self) -> f64 {
        let total_hits = self
            .ast_stats
            .hits
            .load(std::sync::atomic::Ordering::Relaxed)
            + self
                .template_stats
                .hits
                .load(std::sync::atomic::Ordering::Relaxed)
            + self
                .dag_stats
                .hits
                .load(std::sync::atomic::Ordering::Relaxed)
            + self
                .churn_stats
                .hits
                .load(std::sync::atomic::Ordering::Relaxed)
            + self
                .git_stats_stats
                .hits
                .load(std::sync::atomic::Ordering::Relaxed);

        let total_misses = self
            .ast_stats
            .misses
            .load(std::sync::atomic::Ordering::Relaxed)
            + self
                .template_stats
                .misses
                .load(std::sync::atomic::Ordering::Relaxed)
            + self
                .dag_stats
                .misses
                .load(std::sync::atomic::Ordering::Relaxed)
            + self
                .churn_stats
                .misses
                .load(std::sync::atomic::Ordering::Relaxed)
            + self
                .git_stats_stats
                .misses
                .load(std::sync::atomic::Ordering::Relaxed);

        let total_requests = total_hits + total_misses;
        if total_requests > 0 {
            total_hits as f64 / total_requests as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_cache_manager() {
        let config = UnifiedCacheConfig {
            persistent: false,
            ..Default::default()
        };

        let manager = UnifiedCacheManager::new(config).unwrap();

        // Basic functionality test
        assert_eq!(manager.memory_pressure(), 0.0);
        assert_eq!(manager.total_memory_usage(), 0);
    }

    #[tokio::test]
    async fn test_ast_cache_functionality() {
        use crate::services::context::FileContext;
        use tempfile::NamedTempFile;

        let config = UnifiedCacheConfig {
            persistent: false,
            ..Default::default()
        };

        // Create a real temporary file for testing
        let temp_file = NamedTempFile::new().unwrap();
        let test_path = temp_file.path();

        let manager = UnifiedCacheManager::new(config).unwrap();

        // First call should compute
        let mut compute_called = false;
        let result = manager
            .get_or_compute_ast(test_path, || {
                compute_called = true;
                async {
                    Ok(FileContext {
                        path: test_path.to_string_lossy().to_string(),
                        language: "rust".to_string(),
                        items: vec![],
                        complexity_metrics: None,
                    })
                }
            })
            .await
            .unwrap();

        assert!(compute_called);
        assert_eq!(result.path, test_path.to_string_lossy());

        // Second call should use cache
        compute_called = false;
        let cached_result = manager
            .get_or_compute_ast(test_path, || {
                compute_called = true;
                async {
                    Ok(FileContext {
                        path: test_path.to_string_lossy().to_string(),
                        language: "rust".to_string(),
                        items: vec![],
                        complexity_metrics: None,
                    })
                }
            })
            .await
            .unwrap();

        assert!(!compute_called); // Should not compute again
        assert_eq!(cached_result.path, test_path.to_string_lossy());

        // Verify cache stats
        let diagnostics = manager.diagnostics();
        assert_eq!(
            diagnostics
                .ast_stats
                .hits
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
        assert_eq!(
            diagnostics
                .ast_stats
                .misses
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
        assert_eq!(diagnostics.overall_hit_rate(), 0.5); // 1 hit / 2 requests
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        let config = UnifiedCacheConfig {
            persistent: false,
            max_memory_bytes: 1024, // Very small limit to trigger eviction
            ..Default::default()
        };

        let manager = UnifiedCacheManager::new(config).unwrap();

        // Clear caches and test eviction
        manager.clear_all().await.unwrap();
        assert_eq!(manager.total_memory_usage(), 0);

        // Eviction should work without error even with empty cache
        manager.evict_if_needed().await.unwrap();
    }
}
