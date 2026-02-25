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
/// use pmat::services::cache::ChurnCacheStrategy;
/// use pmat::services::cache::CacheStrategy;
/// use pmat::models::churn::CodeChurnAnalysis;
/// use std::path::PathBuf;
/// use tempfile::tempdir;
///
/// let strategy = ChurnCacheStrategy;
/// let repo_dir = tempdir().expect("tempdir");
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
/// assert_eq!(strategy.ttl().expect("has ttl").as_secs(), 1800);
/// assert_eq!(strategy.max_size(), 20);
/// ```
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
/// ```rust,no_run
/// use pmat::services::cache::{GitStatsCacheStrategy, GitStats, CacheStrategy};
/// use std::path::PathBuf;
/// use tempfile::tempdir;
///
/// let strategy = GitStatsCacheStrategy;
/// let repo_dir = tempdir().expect("tempdir");
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
/// assert_eq!(strategy.ttl().expect("has ttl").as_secs(), 900);
/// assert_eq!(strategy.max_size(), 10);
/// ```
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
            .is_some_and(|head| head == cached.head_commit)
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
