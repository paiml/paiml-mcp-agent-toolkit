#![cfg_attr(coverage_nightly, coverage(off))]
// Error types for pmat repo-score

use thiserror::Error;

#[derive(Error, Debug)]
/// Error variants for repo score operations.
pub enum RepoScoreError {
    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),

    #[error("Not a git repository: {0}")]
    NotGitRepository(String),

    #[error("Scorer '{0}' timed out after {1}s")]
    ScorerTimeout(String, u64),

    #[error("Scorer '{0}' failed: {1}")]
    ScorerFailed(String, String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Git error: {0}")]
    GitError(String),

    #[error("External command failed: {0}")]
    CommandFailed(String),
}

pub type Result<T> = std::result::Result<T, RepoScoreError>;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_not_found_error() {
        let err = RepoScoreError::RepositoryNotFound("/path/to/repo".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Repository not found"));
        assert!(msg.contains("/path/to/repo"));
    }

    #[test]
    fn test_not_git_repository_error() {
        let err = RepoScoreError::NotGitRepository("/some/path".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Not a git repository"));
    }

    #[test]
    fn test_scorer_timeout_error() {
        let err = RepoScoreError::ScorerTimeout("complexity".to_string(), 30);
        let msg = err.to_string();
        assert!(msg.contains("complexity"));
        assert!(msg.contains("timed out"));
        assert!(msg.contains("30"));
    }

    #[test]
    fn test_scorer_failed_error() {
        let err = RepoScoreError::ScorerFailed("coverage".to_string(), "no tests".to_string());
        let msg = err.to_string();
        assert!(msg.contains("coverage"));
        assert!(msg.contains("failed"));
    }

    #[test]
    fn test_invalid_config_error() {
        let err = RepoScoreError::InvalidConfig("missing threshold".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Invalid configuration"));
    }

    #[test]
    fn test_git_error() {
        let err = RepoScoreError::GitError("could not clone".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Git error"));
    }

    #[test]
    fn test_command_failed_error() {
        let err = RepoScoreError::CommandFailed("cargo build failed".to_string());
        let msg = err.to_string();
        assert!(msg.contains("External command failed"));
    }

    #[test]
    fn test_error_debug_format() {
        let err = RepoScoreError::RepositoryNotFound("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("RepositoryNotFound"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let repo_err: RepoScoreError = io_err.into();
        let msg = repo_err.to_string();
        assert!(msg.contains("IO error"));
    }

    #[test]
    #[allow(clippy::unnecessary_literal_unwrap)]
    fn test_result_type_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_err() {
        let result: Result<i32> = Err(RepoScoreError::InvalidConfig("test".to_string()));
        assert!(result.is_err());
    }
}
