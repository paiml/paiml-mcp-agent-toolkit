#![cfg_attr(coverage_nightly, coverage(off))]
//! Scorer trait and implementations for Rust Project Score v1.1
//!
//! Defines the common interface for all 6 scoring category analyzers.
//! Each scorer analyzes a Rust project and returns a CategoryScore.

use super::models::{CategoryScore, FileCache, ScoringMode};
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

impl ScorerError {
    /// Returns true if this error indicates a missing tool
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_tool_not_found(&self) -> bool {
        matches!(self, ScorerError::ToolNotFound(_))
    }

    /// Returns true if this error is an IO error
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_io_error(&self) -> bool {
        matches!(self, ScorerError::IoError(_))
    }

    /// Returns true if this error is a command execution error
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_command_error(&self) -> bool {
        matches!(self, ScorerError::CommandError(_))
    }

    /// Returns true if this error is a parse error
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_parse_error(&self) -> bool {
        matches!(self, ScorerError::ParseError(_))
    }

    /// Returns true if this error indicates an invalid project
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_invalid_project(&self) -> bool {
        matches!(self, ScorerError::InvalidProject(_))
    }
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

    /// Analyze a Rust project and return the score for this category (default: Fast mode)
    ///
    /// # Arguments
    /// * `project_path` - Path to the root of the Rust project (contains Cargo.toml)
    ///
    /// # Returns
    /// * `ScorerResult<CategoryScore>` - The score earned and max possible
    fn score(&self, project_path: &Path) -> ScorerResult<CategoryScore> {
        self.score_with_mode(project_path, ScoringMode::default())
    }

    /// Analyze a Rust project with configurable scoring mode
    ///
    /// # Arguments
    /// * `project_path` - Path to the root of the Rust project
    /// * `mode` - Scoring mode (Quick/<10s, Fast/<60s, Full/<5m)
    ///
    /// # Returns
    /// * `ScorerResult<CategoryScore>` - The score earned and max possible
    ///
    /// # Performance
    /// - Quick mode: <10s - Filesystem only, no subprocesses
    /// - Fast mode: <60s - Skip expensive cargo operations (default)
    /// - Full mode: <5m - All checks including mutation testing
    fn score_with_mode(
        &self,
        project_path: &Path,
        mode: ScoringMode,
    ) -> ScorerResult<CategoryScore>;

    /// Analyze a Rust project with configurable scoring mode and optional file cache
    ///
    /// **Kaizen Round 4**: Cache-aware scoring method to eliminate redundant filesystem reads
    ///
    /// # Arguments
    /// * `project_path` - Path to the root of the Rust project
    /// * `mode` - Scoring mode (Quick/<10s, Fast/<60s, Full/<5m)
    /// * `cache` - Optional in-memory file cache (eliminates 22 filesystem walks)
    ///
    /// # Returns
    /// * `ScorerResult<CategoryScore>` - The score earned and max possible
    ///
    /// # Performance Impact (Kaizen Round 4)
    /// - Without cache: 22 filesystem operations, 23,513 syscalls, 180ms (78% of time)
    /// - With cache: Single filesystem walk, ~1,000 syscalls, ~20ms (90% reduction)
    /// - Projected: 230ms → 70ms total time (3x improvement)
    ///
    /// # Default Implementation
    /// Falls back to `score_with_mode()` if not overridden (backward compatible)
    fn score_with_cache(
        &self,
        project_path: &Path,
        mode: ScoringMode,
        _cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        // Default: ignore cache, use direct filesystem reads
        self.score_with_mode(project_path, mode)
    }

    /// Optional: Provide detailed recommendations for improvement
    ///
    /// Default implementation returns empty vec
    fn recommendations(&self, _project_path: &Path) -> Vec<String> {
        Vec::new()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    include!("scorer_test_mocks.rs");
    include!("scorer_test_trait.rs");
}
