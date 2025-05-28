use crate::cli::DagType;
use crate::models::churn::CodeChurnAnalysis;
use crate::models::dag::DependencyGraph;
use crate::models::template::TemplateResource;
use crate::services::cache::{
    config::CacheConfig,
    content_cache::ContentCache,
    diagnostics::{CacheDiagnostics, CacheEffectiveness, CacheStatsSnapshot},
    strategies::*,
};
use crate::services::context::FileContext;
use anyhow::Result;
use parking_lot::RwLock;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

/// Session-based cache manager that coordinates multiple cache types
pub struct SessionCacheManager {
    // Different cache types
    ast_cache: Arc<RwLock<ContentCache<AstCacheStrategy>>>,
    template_cache: Arc<RwLock<ContentCache<TemplateCacheStrategy>>>,
    dag_cache: Arc<RwLock<ContentCache<DagCacheStrategy>>>,
    churn_cache: Arc<RwLock<ContentCache<ChurnCacheStrategy>>>,
    git_stats_cache: Arc<RwLock<ContentCache<GitStatsCacheStrategy>>>,

    // Global settings
    config: CacheConfig,
    session_id: Uuid,
    created: Instant,
}

impl SessionCacheManager {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            ast_cache: Arc::new(RwLock::new(ContentCache::new(AstCacheStrategy))),
            template_cache: Arc::new(RwLock::new(ContentCache::new(TemplateCacheStrategy))),
            dag_cache: Arc::new(RwLock::new(ContentCache::new(DagCacheStrategy))),
            churn_cache: Arc::new(RwLock::new(ContentCache::new(ChurnCacheStrategy))),
            git_stats_cache: Arc::new(RwLock::new(ContentCache::new(GitStatsCacheStrategy))),
            config,
            session_id: Uuid::new_v4(),
            created: Instant::now(),
        }
    }

    /// Get or compute AST analysis
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
        if let Some(ast) = self.ast_cache.read().get(&path_buf) {
            return Ok(ast);
        }

        // Compute and cache
        let ast = compute().await?;
        self.ast_cache.write().put(path_buf, ast.clone());
        Ok(Arc::new(ast))
    }

    /// Get or compute template
    pub async fn get_or_compute_template<F, Fut>(
        &self,
        uri: &str,
        compute: F,
    ) -> Result<Arc<TemplateResource>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<TemplateResource>>,
    {
        let uri_string = uri.to_string();

        // Try cache first
        if let Some(template) = self.template_cache.read().get(&uri_string) {
            return Ok(template);
        }

        // Compute and cache
        let template = compute().await?;
        self.template_cache
            .write()
            .put(uri_string, template.clone());
        Ok(Arc::new(template))
    }

    /// Get or compute DAG
    pub async fn get_or_compute_dag<F, Fut>(
        &self,
        path: &Path,
        dag_type: DagType,
        compute: F,
    ) -> Result<Arc<DependencyGraph>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<DependencyGraph>>,
    {
        let key = (path.to_path_buf(), dag_type);

        // Try cache first
        if let Some(dag) = self.dag_cache.read().get(&key) {
            return Ok(dag);
        }

        // Compute and cache
        let dag = compute().await?;
        self.dag_cache.write().put(key, dag.clone());
        Ok(Arc::new(dag))
    }

    /// Get or compute code churn analysis
    pub async fn get_or_compute_churn<F, Fut>(
        &self,
        repo: &Path,
        period_days: u32,
        compute: F,
    ) -> Result<Arc<CodeChurnAnalysis>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<CodeChurnAnalysis>>,
    {
        let key = (repo.to_path_buf(), period_days);

        // Try cache first
        if let Some(churn) = self.churn_cache.read().get(&key) {
            return Ok(churn);
        }

        // Compute and cache
        let churn = compute().await?;
        self.churn_cache.write().put(key, churn.clone());
        Ok(Arc::new(churn))
    }

    /// Get or compute git statistics
    pub async fn get_or_compute_git_stats<F, Fut>(
        &self,
        repo: &Path,
        compute: F,
    ) -> Result<Arc<GitStats>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<GitStats>>,
    {
        let path_buf = repo.to_path_buf();

        // Try cache first
        if let Some(stats) = self.git_stats_cache.read().get(&path_buf) {
            return Ok(stats);
        }

        // Compute and cache
        let stats = compute().await?;
        self.git_stats_cache.write().put(path_buf, stats.clone());
        Ok(Arc::new(stats))
    }

    /// Calculate memory pressure (0.0 to 1.0)
    pub fn memory_pressure(&self) -> f32 {
        let total_bytes = self.get_total_cache_size();
        total_bytes as f32 / self.config.max_memory_bytes() as f32
    }

    /// Get total cache size in bytes
    pub fn get_total_cache_size(&self) -> usize {
        let ast_size = self.ast_cache.read().stats.memory_usage();
        let template_size = self.template_cache.read().stats.memory_usage();
        let dag_size = self.dag_cache.read().stats.memory_usage();
        let churn_size = self.churn_cache.read().stats.memory_usage();
        let git_stats_size = self.git_stats_cache.read().stats.memory_usage();

        ast_size + template_size + dag_size + churn_size + git_stats_size
    }

    /// Evict least recently used entries if memory pressure is high
    pub fn evict_if_needed(&self) {
        if self.memory_pressure() > 0.8 {
            // Evict from each cache type
            for _ in 0..self.config.eviction_batch_size {
                self.ast_cache.write().evict_lru();
                self.template_cache.write().evict_lru();
                self.dag_cache.write().evict_lru();
                self.churn_cache.write().evict_lru();
                self.git_stats_cache.write().evict_lru();

                // Check if we've freed enough memory
                if self.memory_pressure() < 0.7 {
                    break;
                }
            }
        }
    }

    /// Clear all caches
    pub fn clear_all(&self) {
        self.ast_cache.write().clear();
        self.template_cache.write().clear();
        self.dag_cache.write().clear();
        self.churn_cache.write().clear();
        self.git_stats_cache.write().clear();
    }

    /// Invalidate entries for a specific file
    pub fn invalidate_file(&self, path: &Path) {
        let path_str = path.to_string_lossy();

        // Invalidate AST cache for this file
        self.ast_cache
            .write()
            .invalidate_matching(|key| key.contains(&*path_str));

        // Invalidate DAG cache that might include this file
        self.dag_cache
            .write()
            .invalidate_matching(|key| key.contains(&*path_str));
    }

    /// Invalidate entries for a directory
    pub fn invalidate_directory(&self, dir: &Path) {
        let dir_str = dir.to_string_lossy();

        // Invalidate all caches that might reference this directory
        self.ast_cache
            .write()
            .invalidate_matching(|key| key.contains(&*dir_str));

        self.dag_cache
            .write()
            .invalidate_matching(|key| key.contains(&*dir_str));

        self.churn_cache
            .write()
            .invalidate_matching(|key| key.contains(&*dir_str));
    }

    /// Get cache diagnostics
    pub fn get_diagnostics(&self) -> CacheDiagnostics {
        let ast_cache = self.ast_cache.read();
        let template_cache = self.template_cache.read();
        let dag_cache = self.dag_cache.read();
        let churn_cache = self.churn_cache.read();
        let git_stats_cache = self.git_stats_cache.read();

        let cache_stats = vec![
            (
                "ast".to_string(),
                CacheStatsSnapshot::from((&ast_cache.stats, ast_cache.len())),
            ),
            (
                "template".to_string(),
                CacheStatsSnapshot::from((&template_cache.stats, template_cache.len())),
            ),
            (
                "dag".to_string(),
                CacheStatsSnapshot::from((&dag_cache.stats, dag_cache.len())),
            ),
            (
                "churn".to_string(),
                CacheStatsSnapshot::from((&churn_cache.stats, churn_cache.len())),
            ),
            (
                "git_stats".to_string(),
                CacheStatsSnapshot::from((&git_stats_cache.stats, git_stats_cache.len())),
            ),
        ];

        // Collect hot paths from AST cache
        let hot_paths = ast_cache.hot_entries(10);

        let effectiveness = self.calculate_effectiveness(&cache_stats);

        CacheDiagnostics {
            session_id: self.session_id,
            uptime: self.created.elapsed(),
            memory_usage_mb: self.get_total_cache_size() as f64 / (1024.0 * 1024.0),
            memory_pressure: self.memory_pressure(),
            cache_stats,
            hot_paths,
            effectiveness,
        }
    }

    fn calculate_effectiveness(
        &self,
        cache_stats: &[(String, CacheStatsSnapshot)],
    ) -> CacheEffectiveness {
        let total_hits: u64 = cache_stats.iter().map(|(_, s)| s.hits).sum();
        let total_misses: u64 = cache_stats.iter().map(|(_, s)| s.misses).sum();
        let total_requests = total_hits + total_misses;

        let overall_hit_rate = if total_requests > 0 {
            total_hits as f64 / total_requests as f64
        } else {
            0.0
        };

        let memory_efficiency = if self.config.max_memory_bytes() > 0 {
            1.0 - self.memory_pressure() as f64
        } else {
            0.0
        };

        // Estimate time saved (assuming 100ms average computation time)
        let time_saved_ms = total_hits * 100;

        // Find most valuable caches by hit count
        let mut valuable_caches: Vec<(String, f64)> = cache_stats
            .iter()
            .map(|(name, stats)| (name.clone(), stats.hits as f64))
            .collect();
        valuable_caches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        valuable_caches.truncate(3);

        CacheEffectiveness {
            overall_hit_rate,
            memory_efficiency,
            time_saved_ms,
            most_valuable_caches: valuable_caches,
        }
    }
}

// Safe to send between threads
unsafe impl Send for SessionCacheManager {}
unsafe impl Sync for SessionCacheManager {}
