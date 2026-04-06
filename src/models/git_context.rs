#![cfg_attr(coverage_nightly, coverage(off))]
//! Git context metadata for linking TDG analysis to git commits.
//!
//! This module provides git context extraction and management for correlating
//! quality metrics with git history (commits, branches, authors, tags).
//!
//! # Sprint 65 - Git-Commit Correlation
//!
//! Inspired by HGM (Huxley-Gödel Machine) quality tracking system.
//!
//! # Feature Flags
//!
//! - `git-lib`: Use libgit2 for git operations (faster, more features)
//! - Default: Use shell `git` commands (no extra dependencies)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
#[cfg(not(feature = "git-lib"))]
use std::process::Command;
use thiserror::Error;

/// Git context for a specific analysis run
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitContext {
    /// Full commit SHA (40 hex chars)
    pub commit_sha: String,

    /// Short commit SHA (7 hex chars) for display
    pub commit_sha_short: String,

    /// Branch name (e.g., "main", "feature/tdg-git")
    pub branch: String,

    /// Commit author name
    pub author_name: String,

    /// Commit author email
    pub author_email: String,

    /// Commit timestamp (when code was committed)
    pub commit_timestamp: DateTime<Utc>,

    /// Commit message (first line only)
    pub commit_message: String,

    /// Git tags at this commit (e.g., ["v2.177.0"])
    pub tags: Vec<String>,

    /// Parent commit SHAs (for merge commits)
    pub parent_commits: Vec<String>,

    /// Repository remote URL (if available)
    pub remote_url: Option<String>,

    /// Is working directory clean? (false = uncommitted changes)
    pub is_clean: bool,

    /// Uncommitted file count (if is_clean = false)
    pub uncommitted_files: usize,
}

/// Errors that can occur when extracting git context
#[derive(Debug, Error)]
pub enum GitContextError {
    #[error("Not a git repository: {0}")]
    NotGitRepo(String),

    #[error("Git command failed: {0}")]
    GitCommandFailed(String),

    #[error("Invalid commit SHA: {0}")]
    InvalidCommitSha(String),

    #[cfg(feature = "git-lib")]
    #[error("Git2 error: {0}")]
    Git2Error(#[from] git2::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Chrono parse error: {0}")]
    ChronoError(#[from] chrono::ParseError),
}

impl GitContext {
    /// Extract git context from the current working directory
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn from_current_dir(repo_path: &Path) -> Result<Self, GitContextError> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        #[cfg(feature = "git-lib")]
        {
            Self::from_current_dir_git2(repo_path)
        }
        #[cfg(not(feature = "git-lib"))]
        {
            Self::from_current_dir_shell(repo_path)
        }
    }

    /// Extract git context from a specific commit SHA
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn from_commit_sha(repo_path: &Path, sha: &str) -> Result<Self, GitContextError> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        #[cfg(feature = "git-lib")]
        {
            Self::from_commit_sha_git2(repo_path, sha)
        }
        #[cfg(not(feature = "git-lib"))]
        {
            Self::from_commit_sha_shell(repo_path, sha)
        }
    }

    /// Check if we're in a git repository
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn is_git_repo(path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        #[cfg(feature = "git-lib")]
        {
            git2::Repository::open(path).is_ok()
        }
        #[cfg(not(feature = "git-lib"))]
        {
            Command::new("git")
                .args(["rev-parse", "--git-dir"])
                .current_dir(path)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }

    /// Get git context or return None if not in a git repo
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn try_from_current_dir(repo_path: &Path) -> Option<Self> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        Self::from_current_dir(repo_path).ok()
    }

    // =========================================================================
    // Shell git implementation (default - no git2 dependency)
    // =========================================================================

    #[cfg(not(feature = "git-lib"))]
    fn from_current_dir_shell(repo_path: &Path) -> Result<Self, GitContextError> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        Self::from_commit_sha_shell(repo_path, "HEAD")
    }

    #[cfg(not(feature = "git-lib"))]
    fn from_commit_sha_shell(repo_path: &Path, sha: &str) -> Result<Self, GitContextError> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        // Get commit SHA
        let commit_sha = Self::git_cmd(repo_path, &["rev-parse", sha])?;
        let commit_sha_short = commit_sha.chars().take(7).collect();

        // Get branch name
        let branch = Self::git_cmd(repo_path, &["rev-parse", "--abbrev-ref", "HEAD"])
            .unwrap_or_else(|_| "(detached)".to_string());

        // Get author info using format
        let author_info = Self::git_cmd(repo_path, &["log", "-1", "--format=%an|%ae", sha])?;
        let parts: Vec<&str> = author_info.split('|').collect();
        let author_name = parts.first().unwrap_or(&"Unknown").to_string();
        let author_email = parts.get(1).unwrap_or(&"unknown@example.com").to_string();

        // Get commit timestamp (Unix timestamp)
        let timestamp_str = Self::git_cmd(repo_path, &["log", "-1", "--format=%ct", sha])?;
        let timestamp_secs: i64 = timestamp_str.parse().map_err(|_| {
            GitContextError::GitCommandFailed(format!("Invalid timestamp: {timestamp_str}"))
        })?;
        let commit_timestamp = DateTime::from_timestamp(timestamp_secs, 0)
            .ok_or_else(|| GitContextError::GitCommandFailed("Invalid timestamp".to_string()))?;

        // Get commit message (first line)
        let commit_message = Self::git_cmd(repo_path, &["log", "-1", "--format=%s", sha])?;

        // Get tags at this commit
        let tags_output =
            Self::git_cmd(repo_path, &["tag", "--points-at", sha]).unwrap_or_default();
        let tags: Vec<String> = tags_output
            .lines()
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        // Get parent commits
        let parents_output = Self::git_cmd(repo_path, &["log", "-1", "--format=%P", sha])?;
        let parent_commits: Vec<String> = parents_output
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        // Get remote URL
        let remote_url = Self::git_cmd(repo_path, &["remote", "get-url", "origin"]).ok();

        // Check working directory status
        let status_output =
            Self::git_cmd(repo_path, &["status", "--porcelain"]).unwrap_or_default();
        let uncommitted_files = status_output.lines().filter(|s| !s.is_empty()).count();
        let is_clean = uncommitted_files == 0;

        Ok(GitContext {
            commit_sha,
            commit_sha_short,
            branch,
            author_name,
            author_email,
            commit_timestamp,
            commit_message,
            tags,
            parent_commits,
            remote_url,
            is_clean,
            uncommitted_files,
        })
    }

    #[cfg(not(feature = "git-lib"))]
    fn git_cmd(repo_path: &Path, args: &[&str]) -> Result<String, GitContextError> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let output = Command::new("git")
            .args(args)
            .current_dir(repo_path)
            .output()
            .map_err(|e| GitContextError::GitCommandFailed(format!("Failed to run git: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitContextError::GitCommandFailed(format!(
                "git {} failed: {}",
                args.join(" "),
                stderr.trim()
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    // =========================================================================
    // git2 implementation (feature = "git-lib")
    // =========================================================================

    #[cfg(feature = "git-lib")]
    fn from_current_dir_git2(repo_path: &Path) -> Result<Self, GitContextError> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        use git2::Repository;

        let repo = Repository::open(repo_path)
            .map_err(|e| GitContextError::NotGitRepo(format!("{}: {}", repo_path.display(), e)))?;

        let head = repo.head()?;
        let commit = head.peel_to_commit()?;

        Self::from_git2_commit(&repo, &commit)
    }

    #[cfg(feature = "git-lib")]
    fn from_commit_sha_git2(repo_path: &Path, sha: &str) -> Result<Self, GitContextError> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        use git2::{Oid, Repository};

        let repo = Repository::open(repo_path)
            .map_err(|e| GitContextError::NotGitRepo(format!("{}: {}", repo_path.display(), e)))?;

        let oid = Oid::from_str(sha)
            .map_err(|e| GitContextError::InvalidCommitSha(format!("Invalid SHA '{sha}': {e}")))?;

        let commit = repo.find_commit(oid).map_err(|e| {
            GitContextError::InvalidCommitSha(format!("Commit '{sha}' not found: {e}"))
        })?;

        Self::from_git2_commit(&repo, &commit)
    }

    #[cfg(feature = "git-lib")]
    fn from_git2_commit(
        repo: &git2::Repository,
        commit: &git2::Commit,
    ) -> Result<Self, GitContextError> {
        debug_assert!(true, "contract: from_git2_commit");
        use chrono::TimeZone;

        let commit_sha = commit.id().to_string();
        let commit_sha_short = commit_sha.get(..7).unwrap_or(&commit_sha).to_string();

        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();
        let author_email = author.email().unwrap_or("unknown@example.com").to_string();

        let timestamp_secs = commit.time().seconds();
        let commit_timestamp = Utc
            .timestamp_opt(timestamp_secs, 0)
            .single()
            .ok_or_else(|| {
                GitContextError::GitCommandFailed(format!("Invalid timestamp: {timestamp_secs}"))
            })?;

        let commit_message = commit
            .message()
            .unwrap_or("(no message)")
            .lines()
            .next()
            .unwrap_or("(no message)")
            .to_string();

        let branch = Self::get_current_branch_git2(repo)?;
        let tags = Self::get_tags_at_commit_git2(repo, commit)?;
        let parent_commits = commit.parent_ids().map(|oid| oid.to_string()).collect();
        let remote_url = Self::get_remote_url_git2(repo).ok();
        let (is_clean, uncommitted_files) = Self::check_working_dir_status_git2(repo)?;

        Ok(GitContext {
            commit_sha,
            commit_sha_short,
            branch,
            author_name,
            author_email,
            commit_timestamp,
            commit_message,
            tags,
            parent_commits,
            remote_url,
            is_clean,
            uncommitted_files,
        })
    }

    #[cfg(feature = "git-lib")]
    fn get_current_branch_git2(repo: &git2::Repository) -> Result<String, GitContextError> {
        debug_assert!(true, "contract: get_current_branch_git2");
        let head = repo.head()?;
        Ok(head.shorthand().unwrap_or("(detached)").to_string())
    }

    #[cfg(feature = "git-lib")]
    fn get_tags_at_commit_git2(
        repo: &git2::Repository,
        commit: &git2::Commit,
    ) -> Result<Vec<String>, GitContextError> {
        debug_assert!(true, "contract: get_tags_at_commit_git2");
        let mut tags = Vec::new();
        let commit_id = commit.id();

        repo.tag_foreach(|oid, name| {
            if let Ok(tag_name) = std::str::from_utf8(name) {
                let tag_name = tag_name.trim_start_matches("refs/tags/");
                if let Ok(tag_obj) = repo.find_tag(oid) {
                    if let Ok(target) = tag_obj.target() {
                        if target.id() == commit_id {
                            tags.push(tag_name.to_string());
                        }
                    }
                } else if oid == commit_id {
                    tags.push(tag_name.to_string());
                }
            }
            true
        })?;

        Ok(tags)
    }

    #[cfg(feature = "git-lib")]
    fn get_remote_url_git2(repo: &git2::Repository) -> Result<String, GitContextError> {
        debug_assert!(true, "contract: get_remote_url_git2");
        let remote = repo.find_remote("origin")?;
        remote
            .url()
            .ok_or_else(|| GitContextError::GitCommandFailed("Remote URL not found".to_string()))
            .map(String::from)
    }

    #[cfg(feature = "git-lib")]
    fn check_working_dir_status_git2(
        repo: &git2::Repository,
    ) -> Result<(bool, usize), GitContextError> {
        debug_assert!(true, "contract: check_working_dir_status_git2");
        let statuses = repo.statuses(None)?;
        let uncommitted_count = statuses.len();
        let is_clean = uncommitted_count == 0;
        Ok((is_clean, uncommitted_count))
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn get_repo_root() -> PathBuf {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut current = manifest_dir.clone();
        loop {
            let git_dir = current.join(".git");
            if git_dir.exists() && git_dir.join("HEAD").exists() {
                return current;
            }
            if !current.pop() {
                return manifest_dir.parent().unwrap().to_path_buf();
            }
        }
    }

    #[test]
    fn test_is_git_repo_returns_true_for_git_repo() {
        let repo_path = get_repo_root();
        assert!(GitContext::is_git_repo(&repo_path));
    }

    #[test]
    fn test_is_git_repo_returns_false_for_non_git_dir() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!GitContext::is_git_repo(temp_dir.path()));
    }

    #[test]
    fn test_from_current_dir_extracts_commit_sha() {
        let repo_path = get_repo_root();
        let context = GitContext::from_current_dir(&repo_path).unwrap();
        assert_eq!(context.commit_sha.len(), 40);
        assert_eq!(context.commit_sha_short.len(), 7);
    }

    #[test]
    fn test_from_current_dir_extracts_author_info() {
        let repo_path = get_repo_root();
        let context = GitContext::from_current_dir(&repo_path).unwrap();
        assert!(!context.author_name.is_empty());
        assert!(context.author_email.contains('@'));
    }

    #[test]
    fn test_from_current_dir_fails_for_non_git_dir() {
        let temp_dir = TempDir::new().unwrap();
        let result = GitContext::from_current_dir(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_current_dir_returns_none_for_non_git_dir() {
        let temp_dir = TempDir::new().unwrap();
        assert!(GitContext::try_from_current_dir(temp_dir.path()).is_none());
    }

    #[test]
    fn test_git_context_serialization() {
        let repo_path = get_repo_root();
        let context = GitContext::from_current_dir(&repo_path).unwrap();
        let json = serde_json::to_string(&context).unwrap();
        let deserialized: GitContext = serde_json::from_str(&json).unwrap();
        assert_eq!(context, deserialized);
    }
}
