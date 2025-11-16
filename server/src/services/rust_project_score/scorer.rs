//! Scorer trait and implementations for Rust Project Score v1.1
//!
//! Defines the common interface for all 6 scoring category analyzers.
//! Each scorer analyzes a Rust project and returns a CategoryScore.

use super::models::CategoryScore;
use std::path::Path;

/// Result type for scoring operations
pub type ScorerResult<T> = Result<T, ScorerError>;

/// Errors that can occur during scoring
#[derive(Debug, Clone, thiserror::Error)]
pub enum ScorerError {
    #[error("Failed to execute command: {0}")]
    CommandError(String),

    #[error("Failed to parse output: {0}")]
    ParseError(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Invalid project structure: {0}")]
    InvalidProject(String),

    #[error("IO error: {0}")]
    IoError(String),
}

/// Common trait for all scoring category analyzers
///
/// Each scorer implements this trait to analyze a specific category
/// and return a score with recommendations.
pub trait Scorer: Send + Sync {
    /// Name of this scoring category
    fn name(&self) -> &str;

    /// Maximum possible points for this category
    fn max_points(&self) -> f64;

    /// Analyze a Rust project and return the score for this category (fast mode)
    ///
    /// # Arguments
    /// * `project_path` - Path to the root of the Rust project (contains Cargo.toml)
    ///
    /// # Returns
    /// * `ScorerResult<CategoryScore>` - The score earned and max possible
    fn score(&self, project_path: &Path) -> ScorerResult<CategoryScore> {
        self.score_with_mode(project_path, false)
    }

    /// Analyze a Rust project with configurable mode
    ///
    /// # Arguments
    /// * `project_path` - Path to the root of the Rust project
    /// * `full` - If true, run full analysis (slower but comprehensive)
    ///   If false, run fast mode (skip expensive checks like mutation testing)
    ///
    /// # Returns
    /// * `ScorerResult<CategoryScore>` - The score earned and max possible
    ///
    /// # Performance
    /// - Fast mode: Should complete in <10 seconds per scorer
    /// - Full mode: May take up to 60 seconds per scorer
    fn score_with_mode(&self, project_path: &Path, full: bool) -> ScorerResult<CategoryScore>;

    /// Optional: Provide detailed recommendations for improvement
    ///
    /// Default implementation returns empty vec
    fn recommendations(&self, _project_path: &Path) -> Vec<String> {
        Vec::new()
    }
}
