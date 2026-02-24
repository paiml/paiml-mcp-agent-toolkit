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
