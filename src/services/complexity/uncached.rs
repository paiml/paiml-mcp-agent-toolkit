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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_file_complexity_uncached(
    path: &Path,
    content: Option<&str>,
) -> anyhow::Result<FileComplexityMetrics> {
    use anyhow::Context;

    // When the caller hands us the text to measure, measure THAT text: the
    // AST path below reads the file from disk and would silently score
    // something else (the Issue #67 extraction tests analyse content for paths
    // that do not exist on disk at all).
    if let Some(supplied) = content {
        let language = crate::cli::language_analyzer::Language::from_path(path);
        return crate::cli::language_analyzer::analyze_with_heuristics(path, supplied, language)
            .with_context(|| format!("Failed to analyze file complexity: {}", path.display()));
    }

    let file_content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Route through the SAME entry point the project scan uses
    // (`analysis_utilities::analyze_project_files` -> `analyze_file_complexity`),
    // which prefers the AST analyzer and falls back to heuristics only when a
    // file cannot be parsed.
    //
    // This function used to call `analyze_with_heuristics` directly, so the MCP
    // `analyze_complexity` tool (its only production caller besides `--file`)
    // reported different numbers from `pmat analyze complexity` for the same
    // function: on a 10-line `complex()` the heuristic counter said cyclomatic
    // 10 / cognitive 18 and raised a threshold violation, where the AST
    // analyzer — and the CLI — said 6 / 9 with no violation.
    //
    // The Issue #67 concern that motivated the heuristic detour (AST line
    // numbers invented as `i * 50`) no longer applies: extents now come from
    // measured source spans, so line numbers still reflect the CURRENT file.
    crate::cli::language_analyzer::analyze_file_complexity(path, &file_content)
        .await
        .with_context(|| format!("Failed to analyze file complexity: {}", path.display()))
}

#[cfg(test)]
mod uncached_agreement_tests {
    use super::*;

    /// The MCP `analyze_complexity` tool and `pmat analyze complexity` must
    /// report the same numbers for the same function. They did not: the MCP
    /// side came through here and got heuristic counts.
    #[tokio::test]
    async fn test_uncached_agrees_with_project_scan_analyzer() {
        let dir = tempfile::TempDir::new().unwrap();
        let file = dir.path().join("lib.rs");
        let source = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n\
                      \n\
                      pub fn complex(n: u32) -> u32 {\n\
                      \x20   let mut acc = 0;\n\
                      \x20   for i in 0..n {\n\
                      \x20       if i % 2 == 0 { acc += i; }\n\
                      \x20       else if i % 3 == 0 { acc += i * 2; }\n\
                      \x20       else if i % 5 == 0 { acc += i * 3; }\n\
                      \x20       else if i % 7 == 0 { acc += i * 4; }\n\
                      \x20       else { acc += 1; }\n\
                      \x20   }\n\
                      \x20   acc\n\
                      }\n";
        std::fs::write(&file, source).unwrap();

        let uncached = analyze_file_complexity_uncached(&file, None).await.unwrap();
        let project_scan = crate::cli::language_analyzer::analyze_file_complexity(&file, source)
            .await
            .unwrap();

        let pick = |m: &FileComplexityMetrics| {
            m.functions
                .iter()
                .find(|f| f.name == "complex")
                .map(|f| (f.metrics.cyclomatic, f.metrics.cognitive))
        };

        assert_eq!(
            pick(&uncached),
            pick(&project_scan),
            "uncached analysis must not diverge from the project scan"
        );
        assert!(
            pick(&uncached).is_some(),
            "`complex` must be found: {:?}",
            uncached.functions
        );
    }
}
