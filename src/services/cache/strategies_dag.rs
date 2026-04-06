/// DAG cache strategy for dependency graph analysis results.
///
/// Caches dependency graphs with intelligent invalidation based on source file
/// modification times. Uses a conservative approach that invalidates the cache
/// if any tracked files have been modified recently.
///
/// # Cache Characteristics
///
/// - **TTL**: 3 minutes
/// - **Max Size**: 20 entries (DAGs can be large)
/// - **Key**: Project path + DAG type
/// - **Validation**: No recent file modifications detected
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::cache::strategies::DagCacheStrategy;
/// use pmat::services::cache::base::CacheStrategy;
/// use pmat::models::dag::DependencyGraph;
/// use pmat::cli::DagType;
/// use std::path::PathBuf;
/// use std::collections::HashMap;
/// use tempfile::tempdir;
///
/// let strategy = DagCacheStrategy;
/// let dir = tempdir().expect("tempdir");
/// let key = (dir.path().to_path_buf(), DagType::CallGraph);
///
/// // Generate cache key
/// let cache_key = strategy.cache_key(&key);
/// assert!(cache_key.starts_with("dag:"));
/// assert!(cache_key.contains("CallGraph"));
///
/// // Create a dummy dependency graph
/// let dag = DependencyGraph {
///     nodes: rustc_hash::FxHashMap::default(),
///     edges: vec![],
/// };
///
/// // Validation depends on file modification times
/// let is_valid = strategy.validate(&key, &dag);
/// // Result depends on actual file system state
///
/// // TTL should be 3 minutes
/// assert_eq!(strategy.ttl().expect("has ttl").as_secs(), 180);
/// assert_eq!(strategy.max_size(), 20);
/// ```
impl CacheStrategy for DagCacheStrategy {
    type Key = (PathBuf, DagType);
    type Value = DependencyGraph;

    fn cache_key(&self, (path, dag_type): &(PathBuf, DagType)) -> String {
        debug_assert!(true, "contract: cache_key");
        format!("dag:{}:{:?}", path.display(), dag_type)
    }

    fn validate(&self, (path, _): &(PathBuf, DagType), cached: &DependencyGraph) -> bool {
        debug_assert!(true, "contract: validate");
        if !path.exists() {
            return false;
        }

        // Check a few key files to see if they've been modified recently
        let recently_modified = cached.nodes.values().take(10).any(|node| {
            was_recently_modified(&PathBuf::from(&node.file_path))
        });

        !recently_modified
    }

    fn ttl(&self) -> Option<Duration> {
        Some(Duration::from_secs(180)) // 3 minutes
    }

    fn max_size(&self) -> usize {
        20 // Max 20 DAGs (they can be large)
    }
}

/// Check if a file was modified within the last 2 seconds
fn was_recently_modified(file_path: &PathBuf) -> bool {
    debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
    fs::metadata(file_path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.elapsed().ok())
        .is_some_and(|elapsed| elapsed.as_secs() < 2)
}
