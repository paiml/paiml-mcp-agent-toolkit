//! Git context metadata for linking TDG analysis to git commits.
//!
//! This module provides git context extraction and management for correlating
//! quality metrics with git history (commits, branches, authors, tags).
//!
//! # Sprint 65 - Git-Commit Correlation
//!
//! Inspired by HGM (Huxley-Gödel Machine) quality tracking system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
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

    #[error("Git2 error: {0}")]
    Git2Error(#[from] git2::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Chrono parse error: {0}")]
    ChronoError(#[from] chrono::ParseError),
}

impl GitContext {
    /// Extract git context from the current working directory
    ///
    /// # Errors
    ///
    /// Returns `GitContextError` if:
    /// - Not in a git repository
    /// - Git command fails
    /// - Cannot parse git output
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use paiml_mcp_agent_toolkit::models::git_context::GitContext;
    /// use std::path::Path;
    ///
    /// let context = GitContext::from_current_dir(Path::new(".")).unwrap();
    /// println!("Current commit: {}", context.commit_sha_short);
    /// ```
    pub fn from_current_dir(repo_path: &Path) -> Result<Self, GitContextError> {
        use git2::Repository;

        // Open repository
        let repo = Repository::open(repo_path).map_err(|e| {
            GitContextError::NotGitRepo(format!("{}: {}", repo_path.display(), e))
        })?;

        // Get HEAD commit
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;

        Self::from_git2_commit(&repo, &commit)
    }

    /// Extract git context from a specific commit SHA
    ///
    /// # Errors
    ///
    /// Returns `GitContextError` if:
    /// - Not in a git repository
    /// - Commit SHA not found
    /// - Cannot parse commit data
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use paiml_mcp_agent_toolkit::models::git_context::GitContext;
    /// use std::path::Path;
    ///
    /// let context = GitContext::from_commit_sha(Path::new("."), "abc123def456").unwrap();
    /// println!("Commit author: {}", context.author_name);
    /// ```
    pub fn from_commit_sha(repo_path: &Path, sha: &str) -> Result<Self, GitContextError> {
        use git2::{Oid, Repository};

        // Open repository
        let repo = Repository::open(repo_path).map_err(|e| {
            GitContextError::NotGitRepo(format!("{}: {}", repo_path.display(), e))
        })?;

        // Parse commit SHA
        let oid = Oid::from_str(sha).map_err(|e| {
            GitContextError::InvalidCommitSha(format!("Invalid SHA '{}': {}", sha, e))
        })?;

        // Get commit
        let commit = repo.find_commit(oid).map_err(|e| {
            GitContextError::InvalidCommitSha(format!("Commit '{}' not found: {}", sha, e))
        })?;

        Self::from_git2_commit(&repo, &commit)
    }

    /// Check if we're in a git repository
    ///
    /// # Examples
    ///
    /// ```rust
    /// use paiml_mcp_agent_toolkit::models::git_context::GitContext;
    /// use std::path::Path;
    ///
    /// if GitContext::is_git_repo(Path::new(".")) {
    ///     println!("This is a git repository");
    /// }
    /// ```
    pub fn is_git_repo(path: &Path) -> bool {
        use git2::Repository;
        Repository::open(path).is_ok()
    }

    /// Get git context or return None if not in a git repo
    ///
    /// This is a convenience wrapper around `from_current_dir` that returns
    /// `None` instead of an error if not in a git repository.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use paiml_mcp_agent_toolkit::models::git_context::GitContext;
    /// use std::path::Path;
    ///
    /// match GitContext::try_from_current_dir(Path::new(".")) {
    ///     Some(context) => println!("Git context: {:?}", context),
    ///     None => println!("Not in a git repository"),
    /// }
    /// ```
    pub fn try_from_current_dir(repo_path: &Path) -> Option<Self> {
        Self::from_current_dir(repo_path).ok()
    }

    /// Internal: Extract git context from a git2::Commit
    fn from_git2_commit(
        repo: &git2::Repository,
        commit: &git2::Commit,
    ) -> Result<Self, GitContextError> {
        use chrono::TimeZone;

        // Extract commit SHA
        let commit_sha = commit.id().to_string();
        let commit_sha_short = commit.id().to_string()[..7].to_string();

        // Extract author info
        let author = commit.author();
        let author_name = author
            .name()
            .unwrap_or("Unknown")
            .to_string();
        let author_email = author
            .email()
            .unwrap_or("unknown@example.com")
            .to_string();

        // Extract commit timestamp
        let timestamp_secs = commit.time().seconds();
        let commit_timestamp = Utc.timestamp_opt(timestamp_secs, 0).single().ok_or_else(|| {
            GitContextError::GitCommandFailed(format!(
                "Invalid timestamp: {}",
                timestamp_secs
            ))
        })?;

        // Extract commit message (first line only)
        let commit_message = commit
            .message()
            .unwrap_or("(no message)")
            .lines()
            .next()
            .unwrap_or("(no message)")
            .to_string();

        // Extract branch name
        let branch = Self::get_current_branch(repo)?;

        // Extract tags at this commit
        let tags = Self::get_tags_at_commit(repo, commit)?;

        // Extract parent commits
        let parent_commits = commit
            .parent_ids()
            .map(|oid| oid.to_string())
            .collect();

        // Extract remote URL (optional)
        let remote_url = Self::get_remote_url(repo).ok();

        // Check if working directory is clean
        let (is_clean, uncommitted_files) = Self::check_working_dir_status(repo)?;

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

    /// Get current branch name
    fn get_current_branch(repo: &git2::Repository) -> Result<String, GitContextError> {
        let head = repo.head()?;

        if let Some(branch_name) = head.shorthand() {
            Ok(branch_name.to_string())
        } else {
            // Detached HEAD - return commit SHA
            Ok("(detached)".to_string())
        }
    }

    /// Get tags at specific commit
    fn get_tags_at_commit(
        repo: &git2::Repository,
        commit: &git2::Commit,
    ) -> Result<Vec<String>, GitContextError> {
        let mut tags = Vec::new();
        let commit_id = commit.id();

        repo.tag_foreach(|oid, name| {
            if let Ok(tag_name) = std::str::from_utf8(name) {
                // Remove "refs/tags/" prefix
                let tag_name = tag_name.trim_start_matches("refs/tags/");

                // Check if tag points to this commit
                if let Ok(tag_obj) = repo.find_tag(oid) {
                    if let Ok(target) = tag_obj.target() {
                        if target.id() == commit_id {
                            tags.push(tag_name.to_string());
                        }
                    }
                } else if oid == commit_id {
                    // Lightweight tag (direct ref to commit)
                    tags.push(tag_name.to_string());
                }
            }
            true // Continue iteration
        })?;

        Ok(tags)
    }

    /// Get remote URL (if available)
    fn get_remote_url(repo: &git2::Repository) -> Result<String, GitContextError> {
        let remote = repo.find_remote("origin")?;
        remote
            .url()
            .ok_or_else(|| {
                GitContextError::GitCommandFailed("Remote URL not found".to_string())
            })
            .map(String::from)
    }

    /// Check working directory status
    fn check_working_dir_status(
        repo: &git2::Repository,
    ) -> Result<(bool, usize), GitContextError> {
        let statuses = repo.statuses(None)?;
        let uncommitted_count = statuses.len();
        let is_clean = uncommitted_count == 0;
        Ok((is_clean, uncommitted_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Helper: Get repository root (tests run in target/debug/deps/)
    fn get_repo_root() -> PathBuf {
        // Start from CARGO_MANIFEST_DIR (server/) and go up to repo root
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        // Walk up from server/ to find actual .git directory (not just hooks)
        let mut current = manifest_dir.clone();
        loop {
            let git_dir = current.join(".git");
            if git_dir.exists() && git_dir.join("HEAD").exists() {
                // Found real .git directory with HEAD file
                return current;
            }
            if !current.pop() {
                // Fallback: go up one level from server/
                return manifest_dir.parent().unwrap().to_path_buf();
            }
        }
    }

    // GREEN TEST 1: Test is_git_repo returns true for git repository
    #[test]
    fn test_is_git_repo_returns_true_for_git_repo() {
        // Arrange: Use repository root (known to be a git repo)
        let repo_path = get_repo_root();
        eprintln!("Testing repo path: {:?}", repo_path);
        eprintln!(".git exists: {}", repo_path.join(".git").exists());

        // Act & Assert
        assert!(
            GitContext::is_git_repo(&repo_path),
            "Repo at {:?} should be detected as git repo", repo_path
        );
    }

    // RED TEST 2: Test is_git_repo returns false for non-git directory
    #[test]
    
    fn test_is_git_repo_returns_false_for_non_git_dir() {
        // Arrange: Create temp directory without .git
        let temp_dir = TempDir::new().unwrap();

        // Act & Assert
        assert!(
            !GitContext::is_git_repo(temp_dir.path()),
            "Temp directory should NOT be detected as git repo"
        );
    }

    // RED TEST 3: Test from_current_dir extracts commit SHA
    #[test]
    
    fn test_from_current_dir_extracts_commit_sha() {
        // Arrange
        let repo_path = get_repo_root();

        // Act
        let context = GitContext::from_current_dir(&repo_path).unwrap();

        // Assert
        assert_eq!(
            context.commit_sha.len(),
            40,
            "Full commit SHA should be 40 hex chars"
        );
        assert_eq!(
            context.commit_sha_short.len(),
            7,
            "Short commit SHA should be 7 hex chars"
        );
        assert!(
            context.commit_sha.starts_with(&context.commit_sha_short),
            "Short SHA should be prefix of full SHA"
        );
    }

    // RED TEST 4: Test from_current_dir extracts branch name
    #[test]
    
    fn test_from_current_dir_extracts_branch_name() {
        // Arrange
        let repo_path = get_repo_root();

        // Act
        let context = GitContext::from_current_dir(&repo_path).unwrap();

        // Assert
        assert!(
            !context.branch.is_empty(),
            "Branch name should not be empty"
        );
        // Current repo is on master (per CLAUDE.md)
        assert_eq!(
            context.branch, "master",
            "Current repo should be on master branch"
        );
    }

    // RED TEST 5: Test from_current_dir extracts author info
    #[test]
    
    fn test_from_current_dir_extracts_author_info() {
        // Arrange
        let repo_path = get_repo_root();

        // Act
        let context = GitContext::from_current_dir(&repo_path).unwrap();

        // Assert
        assert!(
            !context.author_name.is_empty(),
            "Author name should not be empty"
        );
        assert!(
            !context.author_email.is_empty(),
            "Author email should not be empty"
        );
        assert!(
            context.author_email.contains('@'),
            "Author email should contain @"
        );
    }

    // RED TEST 6: Test from_current_dir extracts commit timestamp
    #[test]
    
    fn test_from_current_dir_extracts_commit_timestamp() {
        // Arrange
        let repo_path = get_repo_root();

        // Act
        let context = GitContext::from_current_dir(&repo_path).unwrap();

        // Assert
        let now = Utc::now();
        assert!(
            context.commit_timestamp <= now,
            "Commit timestamp should be in the past or present"
        );
        // Reasonable sanity check (commits are not from before 2020)
        let year_2020 = DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(
            context.commit_timestamp >= year_2020,
            "Commit timestamp should be after 2020"
        );
    }

    // RED TEST 7: Test from_current_dir extracts commit message
    #[test]
    
    fn test_from_current_dir_extracts_commit_message() {
        // Arrange
        let repo_path = get_repo_root();

        // Act
        let context = GitContext::from_current_dir(&repo_path).unwrap();

        // Assert
        assert!(
            !context.commit_message.is_empty(),
            "Commit message should not be empty"
        );
        // Commit message should be first line only (no newlines)
        assert!(
            !context.commit_message.contains('\n'),
            "Commit message should be first line only"
        );
    }

    // RED TEST 8: Test from_current_dir detects clean working directory
    #[test]
    
    fn test_from_current_dir_detects_clean_working_dir() {
        // Arrange
        let repo_path = get_repo_root();

        // Act
        let context = GitContext::from_current_dir(&repo_path).unwrap();

        // Assert
        // Note: This test may fail if working directory is dirty
        // We check both is_clean and uncommitted_files for consistency
        if context.is_clean {
            assert_eq!(
                context.uncommitted_files, 0,
                "Clean repo should have 0 uncommitted files"
            );
        } else {
            assert!(
                context.uncommitted_files > 0,
                "Dirty repo should have >0 uncommitted files"
            );
        }
    }

    // RED TEST 9: Test from_current_dir extracts tags
    #[test]
    
    fn test_from_current_dir_extracts_tags() {
        // Arrange
        let repo_path = get_repo_root();

        // Act
        let context = GitContext::from_current_dir(&repo_path).unwrap();

        // Assert
        // Tags list may be empty (if HEAD is not tagged)
        // Just verify it's a valid Vec<String>
        for tag in &context.tags {
            assert!(
                !tag.is_empty(),
                "Tag name should not be empty if present"
            );
        }
    }

    // RED TEST 10: Test from_current_dir extracts parent commits
    #[test]
    
    fn test_from_current_dir_extracts_parent_commits() {
        // Arrange
        let repo_path = get_repo_root();

        // Act
        let context = GitContext::from_current_dir(&repo_path).unwrap();

        // Assert
        // Normal commits have 1 parent, merge commits have 2+, initial commit has 0
        assert!(
            context.parent_commits.len() <= 2,
            "Most commits have 0-2 parents"
        );
        for parent in &context.parent_commits {
            assert_eq!(
                parent.len(),
                40,
                "Parent commit SHA should be 40 hex chars"
            );
        }
    }

    // RED TEST 11: Test from_current_dir fails for non-git directory
    #[test]
    
    fn test_from_current_dir_fails_for_non_git_dir() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();

        // Act
        let result = GitContext::from_current_dir(temp_dir.path());

        // Assert
        assert!(
            result.is_err(),
            "Should return error for non-git directory"
        );
        match result.unwrap_err() {
            GitContextError::NotGitRepo(_) => {
                // Expected error type
            }
            other => panic!("Expected NotGitRepo error, got: {:?}", other),
        }
    }

    // RED TEST 12: Test try_from_current_dir returns None for non-git directory
    #[test]
    
    fn test_try_from_current_dir_returns_none_for_non_git_dir() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();

        // Act
        let result = GitContext::try_from_current_dir(temp_dir.path());

        // Assert
        assert!(result.is_none(), "Should return None for non-git directory");
    }

    // RED TEST 13: Test try_from_current_dir returns Some for git directory
    #[test]
    
    fn test_try_from_current_dir_returns_some_for_git_dir() {
        // Arrange
        let repo_path = get_repo_root();

        // Act
        let result = GitContext::try_from_current_dir(&repo_path);

        // Assert
        assert!(result.is_some(), "Should return Some for git directory");
    }

    // RED TEST 14: Test from_commit_sha extracts specific commit
    #[test]
    
    fn test_from_commit_sha_extracts_specific_commit() {
        // Arrange: Use a known commit from git history
        let repo_path = get_repo_root();
        // Get HEAD commit SHA first (we'll use this as test data)
        let head_context = GitContext::from_current_dir(&repo_path).unwrap();
        let head_sha = head_context.commit_sha.clone();

        // Act: Query by that specific SHA
        let context = GitContext::from_commit_sha(&repo_path, &head_sha).unwrap();

        // Assert
        assert_eq!(
            context.commit_sha, head_sha,
            "Queried commit should match requested SHA"
        );
        assert_eq!(
            context.author_name, head_context.author_name,
            "Author should match HEAD commit"
        );
        assert_eq!(
            context.commit_message, head_context.commit_message,
            "Message should match HEAD commit"
        );
    }

    // RED TEST 15: Test from_commit_sha fails for invalid SHA
    #[test]
    
    fn test_from_commit_sha_fails_for_invalid_sha() {
        // Arrange
        let repo_path = get_repo_root();
        let invalid_sha = "0000000000000000000000000000000000000000";

        // Act
        let result = GitContext::from_commit_sha(&repo_path, invalid_sha);

        // Assert
        assert!(result.is_err(), "Should return error for invalid SHA");
        match result.unwrap_err() {
            GitContextError::InvalidCommitSha(_) | GitContextError::Git2Error(_) => {
                // Expected error types
            }
            other => panic!("Expected InvalidCommitSha or Git2Error, got: {:?}", other),
        }
    }

    // RED TEST 16: Test GitContext serialization (for storage)
    #[test]
    
    fn test_git_context_serialization() {
        // Arrange
        let repo_path = get_repo_root();
        let context = GitContext::from_current_dir(&repo_path).unwrap();

        // Act: Serialize to JSON
        let json = serde_json::to_string(&context).unwrap();

        // Assert: Deserialize back
        let deserialized: GitContext = serde_json::from_str(&json).unwrap();
        assert_eq!(
            context, deserialized,
            "GitContext should round-trip through JSON"
        );
    }

    // RED TEST 17: Test remote URL extraction
    #[test]
    
    fn test_from_current_dir_extracts_remote_url() {
        // Arrange
        let repo_path = get_repo_root();

        // Act
        let context = GitContext::from_current_dir(&repo_path).unwrap();

        // Assert
        // Remote URL is optional (may be None for local-only repos)
        if let Some(remote_url) = &context.remote_url {
            assert!(
                !remote_url.is_empty(),
                "Remote URL should not be empty if present"
            );
            // Should be a valid URL or git path
            assert!(
                remote_url.contains("://") || remote_url.contains('@'),
                "Remote URL should be valid format"
            );
        }
    }
}
