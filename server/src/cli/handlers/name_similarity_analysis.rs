//! Name similarity analysis handler
//!
//! This module contains the complex name similarity analysis function
//! extracted from the main CLI module to reduce complexity.

use anyhow::Result;
use std::path::PathBuf;

/// Refactored name similarity handler - temporarily delegates to main module
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_name_similarity(
    project_path: PathBuf,
    query: String,
    top_k: usize,
    phonetic: bool,
    scope: crate::cli::SearchScope,
    threshold: f64,
    format: crate::cli::NameSimilarityOutputFormat,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
    fuzzy: bool,
    case_sensitive: bool,
) -> Result<()> {
    // Temporarily delegate to the original implementation
    // This will be refactored once the module structure is in place
    crate::cli::handle_analyze_name_similarity(
        project_path,
        query,
        top_k,
        phonetic,
        scope,
        threshold as f32,
        format,
        include,
        exclude,
        output,
        perf,
        fuzzy,
        case_sensitive,
    )
    .await
}
