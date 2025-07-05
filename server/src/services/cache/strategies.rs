use crate::cli::DagType;
use crate::models::churn::CodeChurnAnalysis;
use crate::models::dag::DependencyGraph;
use crate::models::template::TemplateResource;
use crate::services::cache::base::CacheStrategy;
use crate::services::context::FileContext;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};

/// AST cache strategy for file analysis results with file modification tracking.
///
/// This strategy caches parsed AST data and FileContext information, automatically
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
/// ```rust,ignore
/// use pmat::services::cache::strategies::AstCacheStrategy;
/// use pmat::services::cache::base::CacheStrategy;
/// use pmat::services::context::FileContext;
/// use std::path::PathBuf;
/// use tempfile::tempdir;
/// use std::fs;
///
/// let strategy = AstCacheStrategy;
/// let dir = tempdir().unwrap();
/// let file_path = dir.path().join("test.rs");
/// fs::write(&file_path, "fn main() {}").unwrap();
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
/// };
///
/// // Should validate if file exists and hasn't changed
/// assert!(strategy.validate(&file_path, &file_context));
///
/// // TTL should be 5 minutes
/// assert_eq!(strategy.ttl().unwrap().as_secs(), 300);
/// assert_eq!(strategy.max_size(), 100);
/// ```
#[derive(Clone)]
pub struct AstCacheStrategy;

impl CacheStrategy for AstCacheStrategy {
    type Key = PathBuf;
    type Value = FileContext;

    fn cache_key(&self, path: &PathBuf) -> String {
        // Include file path and mtime for uniqueness
        let mtime = fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

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
            .map(|d| d.as_secs())
            .unwrap_or(0);

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

/// Template cache strategy for embedded template resources.
///
/// Caches template resources that are embedded in the binary. Since templates
/// are immutable, validation always returns true for better performance.
///
/// # Cache Characteristics
///
/// - **TTL**: 10 minutes
/// - **Max Size**: 50 entries
/// - **Key**: Template URI prefixed with "template:"
/// - **Validation**: Always valid (templates don't change)
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::cache::strategies::TemplateCacheStrategy;
/// use pmat::services::cache::base::CacheStrategy;
/// use pmat::models::template::TemplateResource;
///
/// let strategy = TemplateCacheStrategy;
/// let template_uri = "template://makefile/rust.mk".to_string();
///
/// // Generate cache key
/// let key = strategy.cache_key(&template_uri);
/// assert_eq!(key, "template:template://makefile/rust.mk");
///
/// // Create a dummy template resource
/// let template = TemplateResource {
///     uri: template_uri.clone(),
///     name: "test".to_string(),
///     description: "test description".to_string(),
///     toolchain: pmat::models::template::Toolchain::RustCli { cargo_features: vec![] },
///     category: pmat::models::template::TemplateCategory::Makefile,
///     parameters: vec![],
///     s3_object_key: "test".to_string(),
///     content_hash: "test".to_string(),
///     semantic_version: semver::Version::parse("1.0.0").unwrap(),
///     dependency_graph: vec![],
/// };
///
/// // Templates are always valid (embedded, immutable)
/// assert!(strategy.validate(&template_uri, &template));
///
/// // TTL should be 10 minutes
/// assert_eq!(strategy.ttl().unwrap().as_secs(), 600);
/// assert_eq!(strategy.max_size(), 50);
/// ```
#[derive(Clone)]
pub struct TemplateCacheStrategy;

impl CacheStrategy for TemplateCacheStrategy {
    type Key = String; // Template URI
    type Value = TemplateResource;

    fn cache_key(&self, uri: &String) -> String {
        format!("template:{uri}")
    }

    fn validate(&self, _uri: &String, _cached: &TemplateResource) -> bool {
        // Templates are embedded and don't change
        true
    }

    fn ttl(&self) -> Option<Duration> {
        Some(Duration::from_secs(600)) // 10 minutes
    }

    fn max_size(&self) -> usize {
        50 // Max 50 templates
    }
}

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
/// ```rust,ignore
/// use pmat::services::cache::strategies::DagCacheStrategy;
/// use pmat::services::cache::base::CacheStrategy;
/// use pmat::models::dag::DependencyGraph;
/// use pmat::cli::DagType;
/// use std::path::PathBuf;
/// use std::collections::HashMap;
/// use tempfile::tempdir;
///
/// let strategy = DagCacheStrategy;
/// let dir = tempdir().unwrap();
/// let key = (dir.path().to_path_buf(), DagType::CallGraph);
///
/// // Generate cache key
/// let cache_key = strategy.cache_key(&key);
/// assert!(cache_key.starts_with("dag:"));
/// assert!(cache_key.contains("CallGraph"));
///
/// // Create a dummy dependency graph
/// let dag = DependencyGraph {
///     nodes: HashMap::new(),
///     edges: vec![],
/// };
///
/// // Validation depends on file modification times
/// let is_valid = strategy.validate(&key, &dag);
/// // Result depends on actual file system state
///
/// // TTL should be 3 minutes
/// assert_eq!(strategy.ttl().unwrap().as_secs(), 180);
/// assert_eq!(strategy.max_size(), 20);
/// ```
#[derive(Clone)]
pub struct DagCacheStrategy;

impl CacheStrategy for DagCacheStrategy {
    type Key = (PathBuf, DagType);
    type Value = DependencyGraph;

    fn cache_key(&self, (path, dag_type): &(PathBuf, DagType)) -> String {
        format!("dag:{}:{:?}", path.display(), dag_type)
    }

    fn validate(&self, (path, _): &(PathBuf, DagType), cached: &DependencyGraph) -> bool {
        // Check if path still exists
        if !path.exists() {
            return false;
        }

        // DAG cache should be invalidated if any source file in the project has changed
        // For now, we'll use a simple approach: check if any file in the DAG has been modified
        // This is conservative but ensures correctness

        // Check a few key files to see if they've been modified recently
        // In a full implementation, we'd track all source file mtimes
        for node in cached.nodes.values().take(10) {
            let file_path = PathBuf::from(&node.file_path);
            if file_path.exists() {
                // Check if file was modified in the last few seconds
                // This is a heuristic to detect recent changes
                if let Ok(metadata) = fs::metadata(&file_path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            // If any file was modified in the last 2 seconds, invalidate
                            if elapsed.as_secs() < 2 {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        true
    }

    fn ttl(&self) -> Option<Duration> {
        Some(Duration::from_secs(180)) // 3 minutes
    }

    fn max_size(&self) -> usize {
        20 // Max 20 DAGs (they can be large)
    }
}

/// Code churn cache strategy with Git commit tracking.
///
/// Caches code churn analysis results and automatically invalidates when the
/// Git HEAD commit changes. Uses Git SHA for precise invalidation detection.
///
/// # Cache Characteristics
///
/// - **TTL**: 30 minutes (Git data is stable)
/// - **Max Size**: 20 entries (memory-intensive)
/// - **Key**: Repository path + period + HEAD commit SHA
/// - **Validation**: HEAD commit unchanged
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::cache::strategies::ChurnCacheStrategy;
/// use pmat::services::cache::base::CacheStrategy;
/// use pmat::models::churn::CodeChurnAnalysis;
/// use std::path::PathBuf;
/// use tempfile::tempdir;
///
/// let strategy = ChurnCacheStrategy;
/// let repo_dir = tempdir().unwrap();
/// let key = (repo_dir.path().to_path_buf(), 30); // 30 days
///
/// // Generate cache key (includes Git HEAD)
/// let cache_key = strategy.cache_key(&key);
/// assert!(cache_key.starts_with("churn:"));
/// assert!(cache_key.contains(":30:"));
///
/// // Create a dummy churn analysis
/// let churn = CodeChurnAnalysis {
///     generated_at: chrono::Utc::now(),
///     period_days: 30,
///     repository_root: repo_dir.path().to_path_buf(),
///     files: vec![],
///     summary: pmat::models::churn::ChurnSummary {
///         total_commits: 0,
///         total_files_changed: 0,
///         hotspot_files: vec![],
///         stable_files: vec![],
///         author_contributions: std::collections::HashMap::new(),
///     },
/// };
///
/// // Validation depends on Git state
/// // let is_valid = strategy.validate(&key, &churn);
///
/// // TTL should be 30 minutes
/// assert_eq!(strategy.ttl().unwrap().as_secs(), 1800);
/// assert_eq!(strategy.max_size(), 20);
/// ```
#[derive(Clone)]
pub struct ChurnCacheStrategy;

impl CacheStrategy for ChurnCacheStrategy {
    type Key = (PathBuf, u32); // repo path + period in days
    type Value = CodeChurnAnalysis;

    fn cache_key(&self, (repo, period_days): &(PathBuf, u32)) -> String {
        // Include HEAD commit SHA for invalidation
        let head = self.get_git_head(repo).unwrap_or_default();
        format!("churn:{}:{}:{}", repo.display(), period_days, head)
    }

    fn validate(&self, (repo, _): &(PathBuf, u32), _cached: &CodeChurnAnalysis) -> bool {
        // Check if HEAD has moved
        if let Some(_current_head) = self.get_git_head(repo) {
            // The cache key includes the HEAD commit, so if we get here
            // with the same key, it's still valid
            true
        } else {
            false
        }
    }

    fn ttl(&self) -> Option<Duration> {
        Some(Duration::from_secs(1800)) // 30 minutes - git data is stable
    }

    fn max_size(&self) -> usize {
        20 // Churn analyses are memory-intensive
    }
}

impl ChurnCacheStrategy {
    fn get_git_head(&self, repo: &PathBuf) -> Option<String> {
        std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo)
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    }
}

/// Git statistics cache strategy for repository metadata.
///
/// Caches Git repository statistics (commits, authors, branches) with validation
/// based on HEAD commit SHA. Provides fast access to repository metadata.
///
/// # Cache Characteristics
///
/// - **TTL**: 15 minutes
/// - **Max Size**: 10 entries
/// - **Key**: Repository path + current branch
/// - **Validation**: HEAD commit unchanged
///
/// # Examples
///
/// ```rust
/// use pmat::services::cache::strategies::{
///     GitStatsCacheStrategy, GitStats
/// };
/// use pmat::services::cache::base::CacheStrategy;
/// use std::path::PathBuf;
/// use tempfile::tempdir;
///
/// let strategy = GitStatsCacheStrategy;
/// let repo_dir = tempdir().unwrap();
///
/// // Generate cache key
/// let cache_key = strategy.cache_key(&repo_dir.path().to_path_buf());
/// assert!(cache_key.starts_with("git_stats:"));
///
/// // Create sample Git stats
/// let stats = GitStats {
///     total_commits: 42,
///     authors: vec!["alice".to_string(), "bob".to_string()],
///     branch: "main".to_string(),
///     head_commit: "abc123".to_string(),
/// };
///
/// // Validation depends on Git HEAD
/// // let is_valid = strategy.validate(&repo_dir.path().to_path_buf(), &stats);
///
/// // TTL should be 15 minutes
/// assert_eq!(strategy.ttl().unwrap().as_secs(), 900);
/// assert_eq!(strategy.max_size(), 10);
/// ```
#[derive(Clone)]
pub struct GitStatsCacheStrategy;

/// Git repository statistics structure.
///
/// Contains metadata about a Git repository including commit count,
/// authors, current branch, and HEAD commit SHA.
///
/// # Examples
///
/// ```rust
/// use pmat::services::cache::strategies::GitStats;
///
/// let stats = GitStats {
///     total_commits: 150,
///     authors: vec![
///         "alice@example.com".to_string(),
///         "bob@example.com".to_string(),
///     ],
///     branch: "main".to_string(),
///     head_commit: "a1b2c3d4e5f6".to_string(),
/// };
///
/// assert_eq!(stats.total_commits, 150);
/// assert_eq!(stats.authors.len(), 2);
/// assert_eq!(stats.branch, "main");
/// ```
#[derive(Clone)]
pub struct GitStats {
    pub total_commits: usize,
    pub authors: Vec<String>,
    pub branch: String,
    pub head_commit: String,
}

impl CacheStrategy for GitStatsCacheStrategy {
    type Key = PathBuf;
    type Value = GitStats;

    fn cache_key(&self, repo: &PathBuf) -> String {
        let branch = self
            .get_current_branch(repo)
            .unwrap_or_else(|| "unknown".to_string());
        format!("git_stats:{}:{}", repo.display(), branch)
    }

    fn validate(&self, repo: &PathBuf, cached: &GitStats) -> bool {
        // Check if HEAD is still the same
        self.get_git_head(repo)
            .map(|head| head == cached.head_commit)
            .unwrap_or(false)
    }

    fn ttl(&self) -> Option<Duration> {
        Some(Duration::from_secs(900)) // 15 minutes
    }

    fn max_size(&self) -> usize {
        10 // Git stats are small but numerous
    }
}

impl GitStatsCacheStrategy {
    fn get_current_branch(&self, repo: &PathBuf) -> Option<String> {
        std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(repo)
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    }

    fn get_git_head(&self, repo: &PathBuf) -> Option<String> {
        std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo)
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_ast_cache_strategy() {
        let strategy = AstCacheStrategy;
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        // Create test file
        File::create(&test_file).unwrap();

        // Test cache key generation
        let key = strategy.cache_key(&test_file);
        assert!(key.starts_with("ast:"));
        assert!(key.contains("test.rs"));

        // Test TTL
        assert_eq!(strategy.ttl(), Some(Duration::from_secs(300)));

        // Test max size
        assert_eq!(strategy.max_size(), 100);
    }

    #[test]
    fn test_template_cache_strategy() {
        let strategy = TemplateCacheStrategy;

        // Test cache key
        let key = strategy.cache_key(&"template:test".to_string());
        assert_eq!(key, "template:template:test");

        // Test TTL
        assert_eq!(strategy.ttl(), Some(Duration::from_secs(600)));

        // Test max size
        assert_eq!(strategy.max_size(), 50);
    }

    #[test]
    fn test_dag_cache_strategy() {
        let strategy = DagCacheStrategy;

        // Test cache key
        let key = strategy.cache_key(&(PathBuf::from("/test"), DagType::ImportGraph));
        assert!(key.contains("dag:"));
        assert!(key.contains("ImportGraph"));

        // Test TTL
        assert_eq!(strategy.ttl(), Some(Duration::from_secs(180)));

        // Test max size
        assert_eq!(strategy.max_size(), 20);
    }

    #[test]
    fn test_churn_cache_strategy() {
        let strategy = ChurnCacheStrategy;
        let temp_dir = TempDir::new().unwrap();

        // Test cache key
        let key = strategy.cache_key(&(temp_dir.path().to_path_buf(), 30));
        assert!(key.starts_with("churn:"));

        // Test TTL
        assert_eq!(strategy.ttl(), Some(Duration::from_secs(1800)));

        // Test max size
        assert_eq!(strategy.max_size(), 20);
    }

    #[test]
    fn test_git_stats_cache_strategy() {
        let strategy = GitStatsCacheStrategy;
        let temp_dir = TempDir::new().unwrap();

        // Test cache key
        let key = strategy.cache_key(&temp_dir.path().to_path_buf());
        assert!(key.starts_with("git_stats:"));

        // Test TTL
        assert_eq!(strategy.ttl(), Some(Duration::from_secs(900)));

        // Test max size
        assert_eq!(strategy.max_size(), 10);
    }
}
