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
}
