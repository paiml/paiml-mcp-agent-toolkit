//! Parallel Git Operations Service
//!
//! This module provides parallelized git operations using connection pooling
//! and concurrent execution for improved performance.

use crate::models::error::TemplateError;
use anyhow::Result;
use futures::future::join_all;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info};

/// Configuration for parallel git operations
#[derive(Debug, Clone)]
pub struct ParallelGitConfig {
    /// Maximum number of concurrent git processes
    pub max_concurrent_operations: usize,
    /// Enable command caching
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
}

impl Default for ParallelGitConfig {
    fn default() -> Self {
        Self {
            max_concurrent_operations: num_cpus::get().min(8),
            enable_caching: true,
            cache_ttl_seconds: 300, // 5 minutes
        }
    }
}

/// Result cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    result: String,
    timestamp: std::time::Instant,
}

/// Parallel git operations executor
pub struct ParallelGitExecutor {
    config: ParallelGitConfig,
    semaphore: Arc<Semaphore>,
    cache: Arc<RwLock<rustc_hash::FxHashMap<String, CacheEntry>>>,
    project_root: PathBuf,
}

impl ParallelGitExecutor {
    pub fn new(project_root: PathBuf) -> Self {
        Self::with_config(project_root, ParallelGitConfig::default())
    }

    pub fn with_config(project_root: PathBuf, config: ParallelGitConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_operations));
        let cache = Arc::new(RwLock::new(rustc_hash::FxHashMap::default()));

        Self {
            config,
            semaphore,
            cache,
            project_root,
        }
    }

    /// Execute a single git command with caching
    pub async fn execute_command(&self, args: Vec<&str>) -> Result<String> {
        // Generate cache key
        let cache_key = format!("git_{}", args.join("_"));

        // Check cache if enabled
        if self.config.enable_caching {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                if entry.timestamp.elapsed().as_secs() < self.config.cache_ttl_seconds {
                    debug!("Cache hit for git command: {:?}", args);
                    return Ok(entry.result.clone());
                }
            }
        }

        // Acquire semaphore permit
        let _permit = self.semaphore.acquire().await?;

        // Execute git command
        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.project_root)
            .output()
            .await
            .map_err(TemplateError::Io)?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git command failed: {}", error_msg));
        }

        let result = String::from_utf8_lossy(&output.stdout).to_string();

        // Cache result if enabled
        if self.config.enable_caching {
            let mut cache = self.cache.write().await;
            cache.insert(
                cache_key,
                CacheEntry {
                    result: result.clone(),
                    timestamp: std::time::Instant::now(),
                },
            );
        }

        Ok(result)
    }

    /// Execute multiple git commands in parallel
    pub async fn execute_batch(&self, commands: Vec<Vec<&str>>) -> Result<Vec<String>> {
        let futures: Vec<_> = commands
            .into_iter()
            .map(|args| {
                let executor = self.clone();
                async move { executor.execute_command(args).await }
            })
            .collect();

        let results = join_all(futures).await;

        // Collect results, propagating first error
        let mut outputs = Vec::new();
        for result in results {
            outputs.push(result?);
        }

        Ok(outputs)
    }

    /// Get file history for multiple files in parallel
    pub async fn get_file_histories(
        &self,
        files: Vec<PathBuf>,
        max_commits: usize,
    ) -> Result<Vec<(PathBuf, Vec<CommitInfo>)>> {
        let commands: Vec<Vec<String>> = files
            .iter()
            .map(|file| {
                vec![
                    "log".to_string(),
                    "--follow".to_string(),
                    format!("-{}", max_commits),
                    "--pretty=format:%H|%an|%aI|%s".to_string(),
                    "--".to_string(),
                    file.to_str().unwrap_or("").to_string(),
                ]
            })
            .collect();

        let results = self.execute_batch_owned(commands).await?;

        Ok(files
            .into_iter()
            .zip(results)
            .map(|(file, output)| {
                let commits = Self::parse_commit_log(&output);
                (file, commits)
            })
            .collect())
    }

    /// Execute batch with owned strings (helper for complex commands)
    async fn execute_batch_owned(&self, commands: Vec<Vec<String>>) -> Result<Vec<String>> {
        let futures: Vec<_> = commands
            .into_iter()
            .map(|args| {
                let executor = self.clone();
                async move {
                    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                    executor.execute_command(args_refs).await
                }
            })
            .collect();

        let results = join_all(futures).await;

        let mut outputs = Vec::new();
        for result in results {
            outputs.push(result?);
        }

        Ok(outputs)
    }

    /// Get blame information for multiple files in parallel
    pub async fn get_file_blames(&self, files: Vec<PathBuf>) -> Result<Vec<(PathBuf, String)>> {
        let commands: Vec<Vec<&str>> = files
            .iter()
            .map(|file| vec!["blame", "--line-porcelain", file.to_str().unwrap_or("")])
            .collect();

        let results = self.execute_batch(commands).await?;

        Ok(files.into_iter().zip(results).collect())
    }

    /// Get diff statistics for multiple file pairs in parallel
    pub async fn get_diff_stats(
        &self,
        file_pairs: Vec<(PathBuf, String, String)>, // (file, from_commit, to_commit)
    ) -> Result<Vec<DiffStats>> {
        let mut owned_args: Vec<Vec<String>> = Vec::new();

        for (file, from, to) in &file_pairs {
            let args = vec![
                "diff".to_string(),
                "--numstat".to_string(),
                format!("{}..{}", from, to),
                "--".to_string(),
                file.to_string_lossy().to_string(),
            ];
            owned_args.push(args);
        }

        let results = self.execute_batch_owned(owned_args).await?;

        Ok(results
            .into_iter()
            .zip(file_pairs)
            .map(|(output, (file, _, _))| Self::parse_diff_stats(&file, &output))
            .collect())
    }

    /// Parse commit log output
    fn parse_commit_log(output: &str) -> Vec<CommitInfo> {
        output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 4 {
                    Some(CommitInfo {
                        hash: parts[0].to_string(),
                        author: parts[1].to_string(),
                        date: parts[2].to_string(),
                        message: parts[3].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Parse diff stats output
    fn parse_diff_stats(file: &Path, output: &str) -> DiffStats {
        let mut additions = 0;
        let mut deletions = 0;

        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                if let (Ok(add), Ok(del)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                    additions += add;
                    deletions += del;
                }
            }
        }

        DiffStats {
            file: file.to_path_buf(),
            additions,
            deletions,
        }
    }

    /// Clear the command cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Git command cache cleared");
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read().await;
        let size = cache.len();
        let memory = size * std::mem::size_of::<(String, CacheEntry)>();
        (size, memory)
    }
}

impl Clone for ParallelGitExecutor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            semaphore: Arc::clone(&self.semaphore),
            cache: Arc::clone(&self.cache),
            project_root: self.project_root.clone(),
        }
    }
}

/// Commit information
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

/// Diff statistics
#[derive(Debug, Clone)]
pub struct DiffStats {
    pub file: PathBuf,
    pub additions: usize,
    pub deletions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_parallel_git_executor() {
        let executor = ParallelGitExecutor::new(PathBuf::from("."));
        let (size, _) = executor.cache_stats().await;
        assert_eq!(size, 0);
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let executor = ParallelGitExecutor::new(PathBuf::from("."));

        // Execute a simple command
        let result = executor.execute_command(vec!["--version"]).await;
        assert!(result.is_ok());

        // Check cache was populated
        let (size, _) = executor.cache_stats().await;
        assert_eq!(size, 1);
    }

    #[tokio::test]
    async fn test_config_defaults() {
        let config = ParallelGitConfig::default();
        assert!(config.max_concurrent_operations > 0);
        assert!(config.max_concurrent_operations <= 8);
        assert!(config.enable_caching);
        assert_eq!(config.cache_ttl_seconds, 300);
    }

    #[tokio::test]
    async fn test_custom_config() {
        let config = ParallelGitConfig {
            max_concurrent_operations: 4,
            enable_caching: false,
            cache_ttl_seconds: 60,
        };

        let temp_dir = TempDir::new().unwrap();
        let executor = ParallelGitExecutor::with_config(temp_dir.path().to_path_buf(), config);

        // Cache should be empty and stay empty with caching disabled
        let (size, _) = executor.cache_stats().await;
        assert_eq!(size, 0);
    }

    #[tokio::test]
    async fn test_execute_batch() {
        let executor = ParallelGitExecutor::new(PathBuf::from("."));

        let commands = vec![vec!["--version"], vec!["--help"]];

        let results = executor.execute_batch(commands).await;
        assert!(results.is_ok());

        let outputs = results.unwrap();
        assert_eq!(outputs.len(), 2);

        // Both commands should succeed
        for output in outputs {
            assert!(!output.is_empty());
        }
    }

    #[tokio::test]
    async fn test_execute_command_failure() {
        let temp_dir = TempDir::new().unwrap();
        let executor = ParallelGitExecutor::new(temp_dir.path().to_path_buf());

        // Try to run a git command that will fail (no git repo)
        let result = executor.execute_command(vec!["log", "-1"]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let executor = ParallelGitExecutor::new(PathBuf::from("."));

        // Populate cache
        let _ = executor.execute_command(vec!["--version"]).await;
        let (size, _) = executor.cache_stats().await;
        assert!(size > 0);

        // Clear cache
        executor.clear_cache().await;
        let (size, _) = executor.cache_stats().await;
        assert_eq!(size, 0);
    }

    #[tokio::test]
    async fn test_parse_commit_log() {
        let output = r#"abc123|John Doe|2024-01-01T12:00:00Z|Initial commit
def456|Jane Smith|2024-01-02T12:00:00Z|Add feature"#;

        let commits = ParallelGitExecutor::parse_commit_log(output);
        assert_eq!(commits.len(), 2);

        assert_eq!(commits[0].hash, "abc123");
        assert_eq!(commits[0].author, "John Doe");
        assert_eq!(commits[0].message, "Initial commit");

        assert_eq!(commits[1].hash, "def456");
        assert_eq!(commits[1].author, "Jane Smith");
        assert_eq!(commits[1].message, "Add feature");
    }

    #[tokio::test]
    async fn test_parse_commit_log_empty() {
        let commits = ParallelGitExecutor::parse_commit_log("");
        assert_eq!(commits.len(), 0);
    }

    #[tokio::test]
    async fn test_parse_commit_log_invalid() {
        let output = "invalid line\nanother invalid line";
        let commits = ParallelGitExecutor::parse_commit_log(output);
        assert_eq!(commits.len(), 0);
    }

    #[tokio::test]
    async fn test_parse_diff_stats() {
        let output = "10\t20\tsrc/main.rs\n5\t15\tsrc/lib.rs";
        let file = PathBuf::from("test.rs");

        let stats = ParallelGitExecutor::parse_diff_stats(&file, output);
        assert_eq!(stats.file, file);
        assert_eq!(stats.additions, 15); // 10 + 5
        assert_eq!(stats.deletions, 35); // 20 + 15
    }

    #[tokio::test]
    async fn test_parse_diff_stats_empty() {
        let file = PathBuf::from("test.rs");
        let stats = ParallelGitExecutor::parse_diff_stats(&file, "");

        assert_eq!(stats.file, file);
        assert_eq!(stats.additions, 0);
        assert_eq!(stats.deletions, 0);
    }

    #[tokio::test]
    async fn test_get_file_histories_no_git() {
        let temp_dir = TempDir::new().unwrap();
        let executor = ParallelGitExecutor::new(temp_dir.path().to_path_buf());

        let files = vec![
            temp_dir.path().join("file1.rs"),
            temp_dir.path().join("file2.rs"),
        ];

        let result = executor.get_file_histories(files, 10).await;
        assert!(result.is_err()); // Should fail without git
    }

    #[tokio::test]
    async fn test_get_file_blames_no_git() {
        let temp_dir = TempDir::new().unwrap();
        let executor = ParallelGitExecutor::new(temp_dir.path().to_path_buf());

        let files = vec![temp_dir.path().join("file1.rs")];

        let result = executor.get_file_blames(files).await;
        assert!(result.is_err()); // Should fail without git
    }

    #[tokio::test]
    async fn test_get_diff_stats_no_git() {
        let temp_dir = TempDir::new().unwrap();
        let executor = ParallelGitExecutor::new(temp_dir.path().to_path_buf());

        let file_pairs = vec![(
            temp_dir.path().join("file1.rs"),
            "abc123".to_string(),
            "def456".to_string(),
        )];

        let result = executor.get_diff_stats(file_pairs).await;
        assert!(result.is_err()); // Should fail without git
    }

    #[tokio::test]
    async fn test_clone_executor() {
        let executor = ParallelGitExecutor::new(PathBuf::from("."));
        let cloned = executor.clone();

        // Both should share the same cache
        let _ = executor.execute_command(vec!["--version"]).await;

        let (size1, _) = executor.cache_stats().await;
        let (size2, _) = cloned.cache_stats().await;
        assert_eq!(size1, size2);
    }

    #[tokio::test]
    async fn test_cache_ttl() {
        let config = ParallelGitConfig {
            max_concurrent_operations: 4,
            enable_caching: true,
            cache_ttl_seconds: 0, // Immediate expiry
        };

        let executor = ParallelGitExecutor::with_config(PathBuf::from("."), config);

        // Execute command twice
        let result1 = executor.execute_command(vec!["--version"]).await;
        assert!(result1.is_ok());

        // Second execution should not hit cache due to TTL
        let result2 = executor.execute_command(vec!["--version"]).await;
        assert!(result2.is_ok());

        // Cache should have only one entry (the second one)
        let (size, _) = executor.cache_stats().await;
        assert_eq!(size, 1);
    }

    #[tokio::test]
    async fn test_concurrent_execution() {
        let executor = ParallelGitExecutor::new(PathBuf::from("."));

        // Execute multiple commands concurrently
        let mut handles = vec![];
        for _ in 0..5 {
            let exec = executor.clone();
            let handle = tokio::spawn(async move { exec.execute_command(vec!["--version"]).await });
            handles.push(handle);
        }

        // All should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_execute_batch_owned() {
        let executor = ParallelGitExecutor::new(PathBuf::from("."));

        let commands = vec![vec!["--version".to_string()], vec!["--help".to_string()]];

        let results = executor.execute_batch_owned(commands).await;
        assert!(results.is_ok());

        let outputs = results.unwrap();
        assert_eq!(outputs.len(), 2);

        // Both commands should succeed
        for output in outputs {
            assert!(!output.is_empty());
        }
    }
}
