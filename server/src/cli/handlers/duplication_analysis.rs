//! Duplication and similarity analysis handlers
//!
//! This module contains the complex duplication detection and name similarity
//! analysis functions extracted from the main CLI module.

use anyhow::Result;
use std::path::PathBuf;

/// Refactored duplicate analysis handler - temporarily delegates to main module
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_duplicates(
    project_path: PathBuf,
    detection_type: crate::cli::DuplicateType,
    threshold: f64,
    min_lines: usize,
    max_tokens: usize,
    format: crate::cli::DuplicateOutputFormat,
    perf: bool,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
) -> Result<()> {
    // Delegate to the refactored implementation
    crate::cli::analysis::handle_analyze_duplicates(
        project_path,
        detection_type,
        threshold as f32,
        min_lines,
        max_tokens,
        format,
        perf,
        include,
        exclude,
        output,
    )
    .await
}
