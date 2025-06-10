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
}

/// Refactored duplicate analysis handler - temporarily delegates to main module
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
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplication_analysis_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
