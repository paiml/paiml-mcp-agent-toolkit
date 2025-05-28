use crate::cli::DagType;
use crate::models::churn::CodeChurnAnalysis;
use crate::models::dag::DependencyGraph;
use crate::models::template::TemplateResource;
use crate::services::cache::base::CacheStrategy;
use crate::services::context::FileContext;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};

/// AST cache strategy for file analysis results
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

    fn validate(&self, path: &PathBuf, _cached: &FileContext) -> bool {
        // For now, rely on mtime-based cache key
        // In a full implementation, we'd check content hash
        path.exists()
    }

    fn ttl(&self) -> Option<Duration> {
        Some(Duration::from_secs(300)) // 5 minutes
    }

    fn max_size(&self) -> usize {
        100 // Max 100 AST entries
    }
}

/// Template cache strategy
#[derive(Clone)]
pub struct TemplateCacheStrategy;

impl CacheStrategy for TemplateCacheStrategy {
    type Key = String; // Template URI
    type Value = TemplateResource;

    fn cache_key(&self, uri: &String) -> String {
        format!("template:{}", uri)
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

/// DAG cache strategy
#[derive(Clone)]
pub struct DagCacheStrategy;

impl CacheStrategy for DagCacheStrategy {
    type Key = (PathBuf, DagType);
    type Value = DependencyGraph;

    fn cache_key(&self, (path, dag_type): &(PathBuf, DagType)) -> String {
        format!("dag:{}:{:?}", path.display(), dag_type)
    }

    fn validate(&self, (path, _): &(PathBuf, DagType), _cached: &DependencyGraph) -> bool {
        // Simple validation - check if path still exists
        path.exists()
    }

    fn ttl(&self) -> Option<Duration> {
        Some(Duration::from_secs(180)) // 3 minutes
    }

    fn max_size(&self) -> usize {
        20 // Max 20 DAGs (they can be large)
    }
}

/// Code churn cache strategy
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

/// Git statistics cache strategy
#[derive(Clone)]
pub struct GitStatsCacheStrategy;

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
