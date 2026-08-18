//! Duplication and similarity analysis handlers
//!
//! This module contains the complex duplication detection and name similarity
//! analysis functions extracted from the main CLI module.

use anyhow::Result;
use std::path::PathBuf;

/// Configuration for duplicate analysis handling
pub struct DuplicateAnalysisConfig {
    pub project_path: PathBuf,
    pub detection_type: crate::cli::DuplicateType,
    pub threshold: f64,
    pub min_lines: usize,
    pub max_tokens: usize,
    pub format: crate::cli::DuplicateOutputFormat,
    pub perf: bool,
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub output: Option<PathBuf>,
    pub top_files: usize,
}

/// Refactored duplicate analysis handler - temporarily delegates to main module
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_analyze_duplicates(config: DuplicateAnalysisConfig) -> Result<()> {
    // Delegate to the refactored implementation
    crate::cli::analysis::handle_analyze_duplicates(
        config.project_path,
        config.detection_type,
        config.threshold as f32,
        config.min_lines,
        config.max_tokens,
        config.format,
        config.perf,
        config.include,
        config.exclude,
        config.output,
        config.top_files,
    )
    .await
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_duplicate_analysis_config_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = DuplicateAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            detection_type: crate::cli::DuplicateType::Exact,
            threshold: 0.8,
            min_lines: 5,
            max_tokens: 100,
            format: crate::cli::DuplicateOutputFormat::Json,
            perf: false,
            include: Some("*.rs".to_string()),
            exclude: Some("test_*.rs".to_string()),
            output: None,
            top_files: 10,
        };

        assert_eq!(config.threshold, 0.8);
        assert_eq!(config.min_lines, 5);
        assert_eq!(config.max_tokens, 100);
        assert!(!config.perf);
    }

    #[test]
    fn test_duplicate_analysis_config_with_output() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("duplicates.json");

        let config = DuplicateAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            detection_type: crate::cli::DuplicateType::Semantic,
            threshold: 0.9,
            min_lines: 10,
            max_tokens: 200,
            format: crate::cli::DuplicateOutputFormat::Detailed,
            perf: true,
            include: None,
            exclude: None,
            output: Some(output_path.clone()),
            top_files: 10,
        };

        assert_eq!(config.output.unwrap(), output_path);
        assert!(config.perf);
    }

    #[test]
    fn test_duplicate_analysis_config_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let config = DuplicateAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            detection_type: crate::cli::DuplicateType::Exact,
            threshold: 0.7,
            min_lines: 3,
            max_tokens: 50,
            format: crate::cli::DuplicateOutputFormat::Json,
            perf: false,
            include: None,
            exclude: None,
            output: None,
            top_files: 10,
        };

        assert!(config.include.is_none());
        assert!(config.exclude.is_none());
        assert!(config.output.is_none());
    }

    /// The delegation reaches the real analyzer, and the real analyzer refuses
    /// an empty tree.
    ///
    /// This body used to end in `assert!(result.is_err() || result.is_ok())` —
    /// a tautology that held for every possible outcome, so it covered the
    /// delegation without checking anything about it. The empty-directory
    /// refusal (#1015) is proof the call actually arrived somewhere that looked
    /// at the path.
    #[tokio::test]
    async fn test_handle_analyze_duplicates_delegates() {
        let temp_dir = TempDir::new().unwrap();
        let config = DuplicateAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            detection_type: crate::cli::DuplicateType::Exact,
            threshold: 0.8,
            min_lines: 5,
            max_tokens: 100,
            format: crate::cli::DuplicateOutputFormat::Json,
            perf: false,
            include: None,
            exclude: None,
            output: None,
            top_files: 10,
        };

        let message = handle_analyze_duplicates(config)
            .await
            .expect_err("0 duplicate lines out of 0 total lines is not a measurement")
            .to_string();
        assert!(
            message.contains("no source files were found")
                && message.contains("This is not a clean result"),
            "the refusal must name what was missing and say it is not clean, got: {message}"
        );
    }
}
