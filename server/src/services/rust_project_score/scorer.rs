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
    pub fn is_tool_not_found(&self) -> bool {
        matches!(self, ScorerError::ToolNotFound(_))
    }

    /// Returns true if this error is an IO error
    pub fn is_io_error(&self) -> bool {
        matches!(self, ScorerError::IoError(_))
    }

    /// Returns true if this error is a command execution error
    pub fn is_command_error(&self) -> bool {
        matches!(self, ScorerError::CommandError(_))
    }

    /// Returns true if this error is a parse error
    pub fn is_parse_error(&self) -> bool {
        matches!(self, ScorerError::ParseError(_))
    }

    /// Returns true if this error indicates an invalid project
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ==========================================================================
    // ScorerError Tests
    // ==========================================================================

    mod scorer_error_tests {
        use super::*;

        #[test]
        fn test_command_error_display() {
            let error = ScorerError::CommandError("cargo clippy failed".to_string());
            assert_eq!(
                error.to_string(),
                "Failed to execute command: cargo clippy failed"
            );
        }

        #[test]
        fn test_parse_error_display() {
            let error = ScorerError::ParseError("invalid JSON".to_string());
            assert_eq!(error.to_string(), "Failed to parse output: invalid JSON");
        }

        #[test]
        fn test_tool_not_found_display() {
            let error = ScorerError::ToolNotFound("cargo-audit".to_string());
            assert_eq!(error.to_string(), "Tool not found: cargo-audit");
        }

        #[test]
        fn test_invalid_project_display() {
            let error = ScorerError::InvalidProject("missing Cargo.toml".to_string());
            assert_eq!(
                error.to_string(),
                "Invalid project structure: missing Cargo.toml"
            );
        }

        #[test]
        fn test_io_error_display() {
            let error = ScorerError::IoError("permission denied".to_string());
            assert_eq!(error.to_string(), "IO error: permission denied");
        }

        #[test]
        fn test_error_clone() {
            let error = ScorerError::CommandError("test".to_string());
            let cloned = error.clone();
            assert_eq!(error.to_string(), cloned.to_string());
        }

        #[test]
        fn test_error_debug() {
            let error = ScorerError::ParseError("invalid".to_string());
            let debug_str = format!("{:?}", error);
            assert!(debug_str.contains("ParseError"));
            assert!(debug_str.contains("invalid"));
        }

        #[test]
        fn test_is_tool_not_found() {
            assert!(ScorerError::ToolNotFound("test".to_string()).is_tool_not_found());
            assert!(!ScorerError::IoError("test".to_string()).is_tool_not_found());
            assert!(!ScorerError::CommandError("test".to_string()).is_tool_not_found());
            assert!(!ScorerError::ParseError("test".to_string()).is_tool_not_found());
            assert!(!ScorerError::InvalidProject("test".to_string()).is_tool_not_found());
        }

        #[test]
        fn test_is_io_error() {
            assert!(ScorerError::IoError("test".to_string()).is_io_error());
            assert!(!ScorerError::ToolNotFound("test".to_string()).is_io_error());
            assert!(!ScorerError::CommandError("test".to_string()).is_io_error());
            assert!(!ScorerError::ParseError("test".to_string()).is_io_error());
            assert!(!ScorerError::InvalidProject("test".to_string()).is_io_error());
        }

        #[test]
        fn test_is_command_error() {
            assert!(ScorerError::CommandError("test".to_string()).is_command_error());
            assert!(!ScorerError::IoError("test".to_string()).is_command_error());
            assert!(!ScorerError::ToolNotFound("test".to_string()).is_command_error());
            assert!(!ScorerError::ParseError("test".to_string()).is_command_error());
            assert!(!ScorerError::InvalidProject("test".to_string()).is_command_error());
        }

        #[test]
        fn test_is_parse_error() {
            assert!(ScorerError::ParseError("test".to_string()).is_parse_error());
            assert!(!ScorerError::IoError("test".to_string()).is_parse_error());
            assert!(!ScorerError::ToolNotFound("test".to_string()).is_parse_error());
            assert!(!ScorerError::CommandError("test".to_string()).is_parse_error());
            assert!(!ScorerError::InvalidProject("test".to_string()).is_parse_error());
        }

        #[test]
        fn test_is_invalid_project() {
            assert!(ScorerError::InvalidProject("test".to_string()).is_invalid_project());
            assert!(!ScorerError::IoError("test".to_string()).is_invalid_project());
            assert!(!ScorerError::ToolNotFound("test".to_string()).is_invalid_project());
            assert!(!ScorerError::CommandError("test".to_string()).is_invalid_project());
            assert!(!ScorerError::ParseError("test".to_string()).is_invalid_project());
        }

        #[test]
        fn test_error_with_empty_message() {
            let error = ScorerError::CommandError(String::new());
            assert_eq!(error.to_string(), "Failed to execute command: ");
        }

        #[test]
        fn test_error_with_unicode_message() {
            let error = ScorerError::ParseError("invalid UTF-8: \u{1F600}".to_string());
            assert!(error.to_string().contains("\u{1F600}"));
        }

        #[test]
        fn test_error_with_long_message() {
            let long_msg = "a".repeat(10000);
            let error = ScorerError::IoError(long_msg.clone());
            assert!(error.to_string().contains(&long_msg));
        }
    }

    // ==========================================================================
    // ScorerResult Tests
    // ==========================================================================

    mod scorer_result_tests {
        use super::*;

        #[test]
        fn test_scorer_result_ok() {
            let result: ScorerResult<i32> = Ok(42);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 42);
        }

        #[test]
        fn test_scorer_result_err() {
            let result: ScorerResult<i32> = Err(ScorerError::IoError("test".to_string()));
            assert!(result.is_err());
        }

        #[test]
        fn test_scorer_result_with_category_score() {
            let result: ScorerResult<CategoryScore> = Ok(CategoryScore::new(10.0, 25.0));
            assert!(result.is_ok());
            let score = result.unwrap();
            assert_eq!(score.earned, 10.0);
            assert_eq!(score.max, 25.0);
        }

        #[test]
        fn test_scorer_result_map() {
            let result: ScorerResult<i32> = Ok(10);
            let mapped = result.map(|x| x * 2);
            assert_eq!(mapped.unwrap(), 20);
        }

        #[test]
        fn test_scorer_result_and_then() {
            let result: ScorerResult<i32> = Ok(10);
            let chained = result.and_then(|x| Ok(x * 2));
            assert_eq!(chained.unwrap(), 20);
        }
    }

    // ==========================================================================
    // Mock Scorer for Testing Trait Default Implementations
    // ==========================================================================

    /// Simple mock scorer for testing the Scorer trait
    struct MockScorer {
        name: String,
        max_points: f64,
        earned_points: f64,
    }

    impl MockScorer {
        fn new(name: &str, max_points: f64, earned_points: f64) -> Self {
            Self {
                name: name.to_string(),
                max_points,
                earned_points,
            }
        }
    }

    impl Scorer for MockScorer {
        fn name(&self) -> &str {
            &self.name
        }

        fn max_points(&self) -> f64 {
            self.max_points
        }

        fn score_with_mode(
            &self,
            _project_path: &Path,
            _mode: ScoringMode,
        ) -> ScorerResult<CategoryScore> {
            Ok(CategoryScore::new(self.earned_points, self.max_points))
        }
    }

    /// Mock scorer that returns errors
    struct FailingScorer {
        error_type: String,
    }

    impl FailingScorer {
        fn new(error_type: &str) -> Self {
            Self {
                error_type: error_type.to_string(),
            }
        }
    }

    impl Scorer for FailingScorer {
        fn name(&self) -> &str {
            "FailingScorer"
        }

        fn max_points(&self) -> f64 {
            10.0
        }

        fn score_with_mode(
            &self,
            _project_path: &Path,
            _mode: ScoringMode,
        ) -> ScorerResult<CategoryScore> {
            match self.error_type.as_str() {
                "command" => Err(ScorerError::CommandError("command failed".to_string())),
                "parse" => Err(ScorerError::ParseError("parse failed".to_string())),
                "tool" => Err(ScorerError::ToolNotFound("tool missing".to_string())),
                "project" => Err(ScorerError::InvalidProject("bad project".to_string())),
                "io" => Err(ScorerError::IoError("io failed".to_string())),
                _ => Err(ScorerError::IoError("unknown".to_string())),
            }
        }
    }

    /// Mock scorer with custom recommendations
    struct RecommendingScorer {
        recommendations: Vec<String>,
    }

    impl RecommendingScorer {
        fn new(recommendations: Vec<String>) -> Self {
            Self { recommendations }
        }
    }

    impl Scorer for RecommendingScorer {
        fn name(&self) -> &str {
            "RecommendingScorer"
        }

        fn max_points(&self) -> f64 {
            15.0
        }

        fn score_with_mode(
            &self,
            _project_path: &Path,
            _mode: ScoringMode,
        ) -> ScorerResult<CategoryScore> {
            Ok(CategoryScore::new(10.0, 15.0))
        }

        fn recommendations(&self, _project_path: &Path) -> Vec<String> {
            self.recommendations.clone()
        }
    }

    /// Mock scorer with custom cache behavior
    struct CacheAwareScorer {
        use_cache: bool,
    }

    impl CacheAwareScorer {
        fn new(use_cache: bool) -> Self {
            Self { use_cache }
        }
    }

    impl Scorer for CacheAwareScorer {
        fn name(&self) -> &str {
            "CacheAwareScorer"
        }

        fn max_points(&self) -> f64 {
            20.0
        }

        fn score_with_mode(
            &self,
            _project_path: &Path,
            _mode: ScoringMode,
        ) -> ScorerResult<CategoryScore> {
            // Without cache, return lower score
            Ok(CategoryScore::new(10.0, 20.0))
        }

        fn score_with_cache(
            &self,
            _project_path: &Path,
            _mode: ScoringMode,
            cache: Option<&FileCache>,
        ) -> ScorerResult<CategoryScore> {
            if self.use_cache && cache.is_some() {
                // With cache, return higher score (simulating cache benefit)
                Ok(CategoryScore::new(15.0, 20.0))
            } else {
                Ok(CategoryScore::new(10.0, 20.0))
            }
        }
    }

    // ==========================================================================
    // Scorer Trait Tests
    // ==========================================================================

    mod scorer_trait_tests {
        use super::*;

        #[test]
        fn test_scorer_name() {
            let scorer = MockScorer::new("RustTooling", 25.0, 20.0);
            assert_eq!(scorer.name(), "RustTooling");
        }

        #[test]
        fn test_scorer_max_points() {
            let scorer = MockScorer::new("Testing", 20.0, 15.0);
            assert_eq!(scorer.max_points(), 20.0);
        }

        #[test]
        fn test_scorer_score_uses_default_mode() {
            let scorer = MockScorer::new("CodeQuality", 26.0, 22.0);
            let path = PathBuf::from("/tmp/test");
            let result = scorer.score(&path);
            assert!(result.is_ok());
            let score = result.unwrap();
            assert_eq!(score.earned, 22.0);
            assert_eq!(score.max, 26.0);
        }

        #[test]
        fn test_scorer_score_with_mode_quick() {
            let scorer = MockScorer::new("Performance", 10.0, 8.0);
            let path = PathBuf::from("/tmp/test");
            let result = scorer.score_with_mode(&path, ScoringMode::Quick);
            assert!(result.is_ok());
        }

        #[test]
        fn test_scorer_score_with_mode_fast() {
            let scorer = MockScorer::new("Dependencies", 12.0, 10.0);
            let path = PathBuf::from("/tmp/test");
            let result = scorer.score_with_mode(&path, ScoringMode::Fast);
            assert!(result.is_ok());
        }

        #[test]
        fn test_scorer_score_with_mode_full() {
            let scorer = MockScorer::new("Documentation", 15.0, 12.0);
            let path = PathBuf::from("/tmp/test");
            let result = scorer.score_with_mode(&path, ScoringMode::Full);
            assert!(result.is_ok());
        }

        #[test]
        fn test_scorer_default_recommendations() {
            let scorer = MockScorer::new("Test", 10.0, 5.0);
            let path = PathBuf::from("/tmp/test");
            let recommendations = scorer.recommendations(&path);
            assert!(recommendations.is_empty());
        }

        #[test]
        fn test_scorer_custom_recommendations() {
            let scorer = RecommendingScorer::new(vec![
                "Add more tests".to_string(),
                "Improve documentation".to_string(),
            ]);
            let path = PathBuf::from("/tmp/test");
            let recommendations = scorer.recommendations(&path);
            assert_eq!(recommendations.len(), 2);
            assert!(recommendations.contains(&"Add more tests".to_string()));
            assert!(recommendations.contains(&"Improve documentation".to_string()));
        }

        #[test]
        fn test_scorer_empty_recommendations() {
            let scorer = RecommendingScorer::new(vec![]);
            let path = PathBuf::from("/tmp/test");
            let recommendations = scorer.recommendations(&path);
            assert!(recommendations.is_empty());
        }

        #[test]
        fn test_scorer_default_score_with_cache() {
            let scorer = MockScorer::new("Test", 10.0, 8.0);
            let path = PathBuf::from("/tmp/test");
            let cache = FileCache::new();

            // Default implementation ignores cache
            let result = scorer.score_with_cache(&path, ScoringMode::Fast, Some(&cache));
            assert!(result.is_ok());
            let score = result.unwrap();
            assert_eq!(score.earned, 8.0);
        }

        #[test]
        fn test_scorer_custom_score_with_cache() {
            let scorer = CacheAwareScorer::new(true);
            let path = PathBuf::from("/tmp/test");
            let cache = FileCache::new();

            // With cache, should return higher score
            let result = scorer.score_with_cache(&path, ScoringMode::Fast, Some(&cache));
            assert!(result.is_ok());
            let score = result.unwrap();
            assert_eq!(score.earned, 15.0);

            // Without cache, should return lower score
            let result = scorer.score_with_cache(&path, ScoringMode::Fast, None);
            assert!(result.is_ok());
            let score = result.unwrap();
            assert_eq!(score.earned, 10.0);
        }

        #[test]
        fn test_scorer_with_cache_none() {
            let scorer = MockScorer::new("Test", 10.0, 8.0);
            let path = PathBuf::from("/tmp/test");

            let result = scorer.score_with_cache(&path, ScoringMode::Fast, None);
            assert!(result.is_ok());
        }
    }

    // ==========================================================================
    // Scorer Error Handling Tests
    // ==========================================================================

    mod scorer_error_handling_tests {
        use super::*;

        #[test]
        fn test_scorer_command_error() {
            let scorer = FailingScorer::new("command");
            let path = PathBuf::from("/tmp/test");
            let result = scorer.score(&path);
            assert!(result.is_err());
            assert!(result.unwrap_err().is_command_error());
        }

        #[test]
        fn test_scorer_parse_error() {
            let scorer = FailingScorer::new("parse");
            let path = PathBuf::from("/tmp/test");
            let result = scorer.score(&path);
            assert!(result.is_err());
            assert!(result.unwrap_err().is_parse_error());
        }

        #[test]
        fn test_scorer_tool_not_found() {
            let scorer = FailingScorer::new("tool");
            let path = PathBuf::from("/tmp/test");
            let result = scorer.score(&path);
            assert!(result.is_err());
            assert!(result.unwrap_err().is_tool_not_found());
        }

        #[test]
        fn test_scorer_invalid_project() {
            let scorer = FailingScorer::new("project");
            let path = PathBuf::from("/tmp/test");
            let result = scorer.score(&path);
            assert!(result.is_err());
            assert!(result.unwrap_err().is_invalid_project());
        }

        #[test]
        fn test_scorer_io_error() {
            let scorer = FailingScorer::new("io");
            let path = PathBuf::from("/tmp/test");
            let result = scorer.score(&path);
            assert!(result.is_err());
            assert!(result.unwrap_err().is_io_error());
        }
    }

    // ==========================================================================
    // Scorer Thread Safety Tests
    // ==========================================================================

    mod scorer_thread_safety_tests {
        use super::*;
        use std::sync::Arc;
        use std::thread;

        #[test]
        fn test_scorer_is_send() {
            fn assert_send<T: Send>() {}
            assert_send::<MockScorer>();
        }

        #[test]
        fn test_scorer_is_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<MockScorer>();
        }

        #[test]
        fn test_scorer_can_be_shared_across_threads() {
            let scorer = Arc::new(MockScorer::new("ThreadSafe", 10.0, 8.0));
            let path = PathBuf::from("/tmp/test");

            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let scorer = Arc::clone(&scorer);
                    let path = path.clone();
                    thread::spawn(move || scorer.score(&path))
                })
                .collect();

            for handle in handles {
                let result = handle.join().unwrap();
                assert!(result.is_ok());
            }
        }

        #[test]
        fn test_scorer_trait_object() {
            let scorer: Box<dyn Scorer> = Box::new(MockScorer::new("TraitObject", 10.0, 7.0));
            let path = PathBuf::from("/tmp/test");
            let result = scorer.score(&path);
            assert!(result.is_ok());
            assert_eq!(scorer.name(), "TraitObject");
            assert_eq!(scorer.max_points(), 10.0);
        }
    }

    // ==========================================================================
    // Edge Case Tests
    // ==========================================================================

    mod edge_case_tests {
        use super::*;

        #[test]
        fn test_scorer_with_zero_max_points() {
            let scorer = MockScorer::new("ZeroMax", 0.0, 0.0);
            assert_eq!(scorer.max_points(), 0.0);
            let path = PathBuf::from("/tmp/test");
            let result = scorer.score(&path);
            assert!(result.is_ok());
            let score = result.unwrap();
            assert_eq!(score.max, 0.0);
        }

        #[test]
        fn test_scorer_with_negative_points() {
            // Edge case: negative points (should be avoided in practice)
            let scorer = MockScorer::new("Negative", 10.0, -5.0);
            let path = PathBuf::from("/tmp/test");
            let result = scorer.score(&path);
            assert!(result.is_ok());
            let score = result.unwrap();
            assert_eq!(score.earned, -5.0);
        }

        #[test]
        fn test_scorer_with_fractional_points() {
            let scorer = MockScorer::new("Fractional", 25.5, 17.25);
            let path = PathBuf::from("/tmp/test");
            let result = scorer.score(&path);
            assert!(result.is_ok());
            let score = result.unwrap();
            assert!((score.earned - 17.25).abs() < 0.001);
            assert!((score.max - 25.5).abs() < 0.001);
        }

        #[test]
        fn test_scorer_with_large_points() {
            let scorer = MockScorer::new("Large", 1_000_000.0, 999_999.0);
            let path = PathBuf::from("/tmp/test");
            let result = scorer.score(&path);
            assert!(result.is_ok());
            let score = result.unwrap();
            assert_eq!(score.earned, 999_999.0);
        }

        #[test]
        fn test_scorer_with_empty_name() {
            let scorer = MockScorer::new("", 10.0, 5.0);
            assert_eq!(scorer.name(), "");
        }

        #[test]
        fn test_scorer_with_unicode_name() {
            let scorer = MockScorer::new("测试スコア", 10.0, 5.0);
            assert_eq!(scorer.name(), "测试スコア");
        }

        #[test]
        fn test_scorer_with_special_path_characters() {
            let scorer = MockScorer::new("Test", 10.0, 5.0);
            let path = PathBuf::from("/tmp/path with spaces/and-dashes/under_scores");
            let result = scorer.score(&path);
            assert!(result.is_ok());
        }

        #[test]
        fn test_scorer_perfect_score() {
            let scorer = MockScorer::new("Perfect", 25.0, 25.0);
            let path = PathBuf::from("/tmp/test");
            let result = scorer.score(&path);
            assert!(result.is_ok());
            let score = result.unwrap();
            assert!(score.is_perfect());
        }

        #[test]
        fn test_scorer_over_max_score() {
            // Edge case: earned > max (should be avoided in practice)
            let scorer = MockScorer::new("OverMax", 10.0, 15.0);
            let path = PathBuf::from("/tmp/test");
            let result = scorer.score(&path);
            assert!(result.is_ok());
            let score = result.unwrap();
            assert!(score.earned > score.max);
        }
    }

    // ==========================================================================
    // Integration Tests with FileCache
    // ==========================================================================

    mod file_cache_integration_tests {
        use super::*;

        #[test]
        fn test_scorer_with_populated_cache() {
            let scorer = CacheAwareScorer::new(true);
            let path = PathBuf::from("/tmp/test");

            let mut cache = FileCache::new();
            cache.insert(
                PathBuf::from("/tmp/test/Cargo.toml"),
                "[package]\nname = \"test\"".to_string(),
            );
            cache.insert(
                PathBuf::from("/tmp/test/src/lib.rs"),
                "fn main() {}".to_string(),
            );

            let result = scorer.score_with_cache(&path, ScoringMode::Fast, Some(&cache));
            assert!(result.is_ok());
        }

        #[test]
        fn test_scorer_with_empty_cache() {
            let scorer = CacheAwareScorer::new(true);
            let path = PathBuf::from("/tmp/test");
            let cache = FileCache::new();

            let result = scorer.score_with_cache(&path, ScoringMode::Fast, Some(&cache));
            assert!(result.is_ok());
        }

        #[test]
        fn test_scorer_cache_stats() {
            let mut cache = FileCache::new();
            cache.insert(PathBuf::from("/test/file1.rs"), "content1".to_string());
            cache.insert(PathBuf::from("/test/file2.rs"), "content2".to_string());

            let (file_count, total_bytes) = cache.stats();
            assert_eq!(file_count, 2);
            assert_eq!(total_bytes, 16); // "content1" + "content2"
        }

        #[test]
        fn test_scorer_cache_get() {
            let mut cache = FileCache::new();
            let path = PathBuf::from("/test/file.rs");
            cache.insert(path.clone(), "fn test() {}".to_string());

            assert!(cache.get(&path).is_some());
            assert_eq!(cache.get(&path).unwrap(), "fn test() {}");
            assert!(cache.get(&PathBuf::from("/nonexistent")).is_none());
        }
    }
}
