// Error types for pmat repo-score

use thiserror::Error;

#[derive(Error, Debug)]
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
