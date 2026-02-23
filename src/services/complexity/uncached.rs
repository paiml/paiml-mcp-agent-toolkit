#![cfg_attr(coverage_nightly, coverage(off))]
//! Uncached complexity analysis and cache key computation.

use std::path::Path;

use super::types::FileComplexityMetrics;

/// Cache key computation for complexity metrics
/// Computes a cache key for complexity analysis based on file path and content
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::complexity::compute_complexity_cache_key;
/// use std::path::Path;
///
/// let path = Path::new("src/main.rs");
/// let content = b"fn main() { println!(\"Hello\"); }";
///
/// let key = compute_complexity_cache_key(path, content);
/// assert!(key.starts_with("cx:"));
/// assert!(key.len() > 10);
/// ```
#[must_use]
pub fn compute_complexity_cache_key(path: &Path, content: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    path.hash(&mut hasher);
    format!("cx:{:x}", hasher.finish())
}

/// Analyze file complexity WITHOUT using TDG cache (Issue #67 fix)
///
/// This function performs fresh analysis and always reports accurate
/// line numbers from the current file location. Use this for:
/// - `--file` parameter (single file analysis)
/// - `--force-refresh` flag
/// - Pre-commit hooks requiring accurate line numbers
///
/// # Root Cause (Issue #67)
///
/// The TDG cache uses content hash as the primary key. When functions are
/// extracted from one file to another, the content hash remains the same,
/// causing line numbers from the OLD location to be reported for the NEW file.
///
/// # Solution
///
/// This function bypasses the TDG cache entirely and performs fresh AST/heuristic
/// analysis, ensuring line numbers reflect the actual current file location.
///
/// # Arguments
///
/// * `path` - File path to analyze
/// * `content` - Optional file content (reads from disk if None)
///
/// # Returns
///
/// Fresh `FileComplexityMetrics` with accurate line numbers
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::complexity::analyze_file_complexity_uncached;
/// use std::path::Path;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Analyze file with fresh line numbers (bypasses cache)
/// let path = Path::new("src/extracted_functions.rs");
/// let metrics = analyze_file_complexity_uncached(path, None).await?;
///
/// // Line numbers reflect CURRENT file location
/// for func in &metrics.functions {
///     println!("{} at lines {}-{}", func.name, func.line_start, func.line_end);
/// }
/// # Ok(())
/// # }
/// ```
///
/// # See Also
///
/// - Issue #67: https://github.com/paiml/paiml-mcp-agent-toolkit/issues/67
/// - Test suite: `complexity_file_extraction_tests.rs`
pub async fn analyze_file_complexity_uncached(
    path: &Path,
    content: Option<&str>,
) -> anyhow::Result<FileComplexityMetrics> {
    use anyhow::Context;

    // Read file content if not provided
    let file_content;
    let content_ref = if let Some(c) = content {
        c
    } else {
        file_content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        &file_content
    };

    // CRITICAL (Issue #67): Use heuristic analyzer for accurate line numbers
    // The AST analyzer returns approximate line numbers (i * 50), which breaks
    // extracted function scenarios. The heuristic analyzer provides EXACT line
    // numbers by parsing the actual file content.
    //
    // This ensures:
    // - Functions extracted from old_file.rs:500 to new_file.rs:148
    // - Report line 148 (CURRENT location), not line 500 (OLD cached location)
    let language = crate::cli::language_analyzer::Language::from_path(path);
    crate::cli::language_analyzer::analyze_with_heuristics(path, content_ref, language)
        .with_context(|| format!("Failed to analyze file complexity: {}", path.display()))
}
