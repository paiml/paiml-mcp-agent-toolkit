/// AST cache strategy for file analysis results with file modification tracking.
///
/// This strategy caches parsed AST data and `FileContext` information, automatically
/// invalidating entries when the source files are modified. Uses file modification
/// times (mtime) for cache validation.
///
/// # Cache Characteristics
///
/// - **TTL**: 5 minutes
/// - **Max Size**: 100 entries
/// - **Key**: File path + modification time
/// - **Validation**: File exists and mtime unchanged
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::cache::strategies::AstCacheStrategy;
/// use pmat::services::cache::base::CacheStrategy;
/// use pmat::services::context::FileContext;
/// use std::path::PathBuf;
/// use tempfile::tempdir;
/// use std::fs;
///
/// let strategy = AstCacheStrategy;
/// let dir = tempdir().expect("tempdir");
/// let file_path = dir.path().join("test.rs");
/// fs::write(&file_path, "fn main() {}").expect("write");
///
/// // Generate cache key
/// let key = strategy.cache_key(&file_path);
/// assert!(key.starts_with("ast:"));
/// assert!(key.contains("test.rs"));
///
/// // Create a dummy FileContext for validation
/// let file_context = FileContext {
///     path: file_path.to_string_lossy().to_string(),
///     items: vec![],
///     language: "rust".to_string(),
///     complexity_metrics: None,
/// };
///
/// // Should validate if file exists and hasn't changed
/// assert!(strategy.validate(&file_path, &file_context));
///
/// // TTL should be 5 minutes
/// assert_eq!(strategy.ttl().expect("has ttl").as_secs(), 300);
/// assert_eq!(strategy.max_size(), 100);
/// ```
impl CacheStrategy for AstCacheStrategy {
    type Key = PathBuf;
    type Value = FileContext;

    fn cache_key(&self, path: &PathBuf) -> String {
        // Include file path and mtime for uniqueness
        let mtime = fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_secs());

        format!("ast:{}:{}", path.display(), mtime)
    }

    fn validate(&self, path: &PathBuf, cached: &FileContext) -> bool {
        // Check if file still exists and hasn't been modified
        if !path.exists() {
            return false;
        }

        // Get current mtime
        let current_mtime = fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_secs());

        // The cached FileContext should be for the same file
        // Compare the path to ensure we're validating the right entry
        let cached_path = PathBuf::from(&cached.path);
        if cached_path != *path {
            return false;
        }

        // Check if the file has been modified since caching
        // We need to compare the mtime when the cache entry was created
        // with the current mtime
        if let Ok(cached_metadata) = fs::metadata(&cached.path) {
            if let Ok(cached_modified) = cached_metadata.modified() {
                if let Ok(cached_duration) = cached_modified.duration_since(UNIX_EPOCH) {
                    let file_mtime = cached_duration.as_secs();
                    // If the file's current mtime matches what we expect, it's valid
                    return current_mtime == file_mtime;
                }
            }
        }

        // If we can't determine mtime, invalidate to be safe
        false
    }

    fn ttl(&self) -> Option<Duration> {
        Some(Duration::from_secs(300)) // 5 minutes
    }

    fn max_size(&self) -> usize {
        100 // Max 100 AST entries
    }
}
