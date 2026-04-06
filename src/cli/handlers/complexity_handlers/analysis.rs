//! Complexity analysis logic: single file, multi-file, project, filtering, and violation checks.

use crate::services::complexity::FileComplexityMetrics;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use super::ComplexityConfig;

/// Analyze a single file and return its complexity metrics
///
/// This helper function handles single file analysis with proper error handling
/// and maintains consistency with the Issue #42 fix for multi-language support.
///
/// **Issue #67 Fix**: When analyzing a single file with `--file` parameter,
/// we ALWAYS use uncached analysis to ensure line numbers reflect the CURRENT
/// file location, not stale cached data from when the function was in a different file.
pub(crate) async fn analyze_single_file(
    file_path: &Path,
    config: &ComplexityConfig,
) -> Result<Vec<FileComplexityMetrics>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    eprintln!("🔍 Analyzing complexity of file: {}", file_path.display());

    // Ensure file exists and resolve absolute path
    let full_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        config.project_path.join(file_path)
    };

    if !full_path.exists() {
        anyhow::bail!("File not found: {}", full_path.display());
    }

    // Issue #67 Fix: Use UNCACHED analysis for single file operations
    // This ensures line numbers are accurate for extracted/moved functions
    // When functions are extracted from one file to another, the TDG cache
    // (keyed by content hash) returns stale line numbers from the old location.
    // By using uncached analysis, we always report line numbers from the CURRENT file.
    let metrics = crate::services::complexity::analyze_file_complexity_uncached(&full_path, None)
        .await
        .context(format!(
            "Failed to analyze file complexity: {}",
            full_path.display()
        ))?;

    Ok(vec![metrics])
}

/// Analyze multiple files and return aggregated complexity metrics
///
/// This helper function processes a list of files, maintaining consistency
/// with single file analysis and proper error handling for missing files.
pub(crate) async fn analyze_multiple_files(
    files: &[PathBuf],
    config: &ComplexityConfig,
) -> Result<Vec<FileComplexityMetrics>> {
    eprintln!("🔍 Analyzing complexity of {} files...", files.len());

    let mut all_metrics = Vec::new();
    for file_path in files {
        let full_path = if file_path.is_absolute() {
            file_path.clone()
        } else {
            config.project_path.join(file_path)
        };

        if !full_path.exists() {
            eprintln!("⚠️  Skipping missing file: {}", full_path.display());
            continue;
        }

        // Use same analyzer as single file mode (Issue #42 consistency)
        let file_content = std::fs::read_to_string(&full_path)
            .context(format!("Failed to read file: {}", full_path.display()))?;

        let metrics =
            crate::cli::language_analyzer::analyze_file_complexity(&full_path, &file_content)
                .await?;
        all_metrics.push(metrics);
    }

    Ok(all_metrics)
}

/// Analyze entire project directory based on toolchain detection
///
/// This helper function handles project-wide analysis with proper toolchain
/// detection and maintains the Issue #42 fix for multi-language projects.
pub(super) async fn analyze_project(
    detected_toolchain: Option<String>,
    config: &ComplexityConfig,
) -> Result<Vec<FileComplexityMetrics>> {
    if let Some(ref toolchain) = detected_toolchain {
        eprintln!("🔍 Analyzing {toolchain} project complexity...");
        crate::cli::analysis_utilities::analyze_project_files(
            &config.project_path,
            Some(toolchain),
            &config.include,
            config.max_cyclomatic,
            config.max_cognitive,
        )
        .await
    } else {
        // No specific toolchain detected - analyze all supported file types
        eprintln!("🔍 Analyzing project complexity (multi-language)...");
        crate::cli::analysis_utilities::analyze_project_files(
            &config.project_path,
            None, // This will trigger analysis of all supported languages
            &config.include,
            config.max_cyclomatic,
            config.max_cognitive,
        )
        .await
    }
}

/// Apply complexity threshold filtering to metrics
///
/// Filters files to only include those with functions exceeding the specified
/// cyclomatic or cognitive complexity thresholds.
///
/// Returns the count of files that were filtered out for better UX reporting.
pub(super) fn apply_complexity_filters(
    file_metrics: &mut Vec<FileComplexityMetrics>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) -> usize {
    if max_cyclomatic.is_none() && max_cognitive.is_none() {
        return 0;
    }

    let original_count = file_metrics.len();

    file_metrics.retain(|file| {
        file.functions.iter().any(|func| {
            let exceeds_cyclomatic =
                max_cyclomatic.is_some_and(|threshold| func.metrics.cyclomatic > threshold);
            let exceeds_cognitive =
                max_cognitive.is_some_and(|threshold| func.metrics.cognitive > threshold);
            exceeds_cyclomatic || exceeds_cognitive
        })
    });

    let filtered_count = original_count - file_metrics.len();

    if filtered_count > 0 {
        let cyc_threshold = max_cyclomatic.unwrap_or(u16::MAX);
        let cog_threshold = max_cognitive.unwrap_or(u16::MAX);
        eprintln!(
            "ℹ️  Filtered {} file(s) with no functions exceeding thresholds (cyclomatic > {}, cognitive > {})",
            filtered_count, cyc_threshold, cog_threshold
        );
    }

    filtered_count
}

/// Apply top files limit by sorting and truncating results
///
/// Sorts files by total complexity (cyclomatic + cognitive) in descending order
/// and keeps only the top N most complex files.
pub(super) fn apply_top_files_limit(
    file_metrics: &mut Vec<FileComplexityMetrics>,
    top_files: usize,
) {
    if top_files > 0 && !file_metrics.is_empty() {
        // Sort files by complexity (descending)
        file_metrics.sort_by(|a, b| {
            let a_complexity =
                f64::from(a.total_complexity.cyclomatic) + f64::from(a.total_complexity.cognitive);
            let b_complexity =
                f64::from(b.total_complexity.cyclomatic) + f64::from(b.total_complexity.cognitive);
            b_complexity
                .partial_cmp(&a_complexity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Keep only top N files
        file_metrics.truncate(top_files);
    }
}

/// Analyze files based on the specified mode (single, multiple, or project)
pub(super) async fn analyze_files_by_mode(
    file: Option<PathBuf>,
    files: Vec<PathBuf>,
    config: &ComplexityConfig,
) -> Result<Vec<FileComplexityMetrics>> {
    eprintln!("⏰ Analysis timeout set to {} seconds", config.timeout);

    let result = if let Some(single_file) = file {
        analyze_single_file(&single_file, config).await
    } else if !files.is_empty() {
        analyze_multiple_files(&files, config).await
    } else {
        let detected_toolchain = config.detect_toolchain();
        analyze_project(detected_toolchain, config).await
    };

    // Provide feedback on analysis results
    match &result {
        Ok(metrics) if metrics.is_empty() => {
            eprintln!("\n⚠️  Warning: No files were found or analyzed");
            eprintln!("   Possible reasons:");
            eprintln!("   - Directory is empty or contains no supported file types");
            eprintln!("   - Files are excluded by .gitignore patterns");
            eprintln!("   - Include patterns don't match any files");
            if !config.include.is_empty() {
                eprintln!("   - Current include patterns: {:?}", config.include);
            }
            eprintln!();
        }
        Ok(metrics) => {
            eprintln!("✅ Successfully analyzed {} file(s)", metrics.len());
        }
        Err(_) => {
            // Error will be returned and handled by caller
        }
    }

    result
}

/// Check for complexity violations and exit if required
pub(super) fn check_complexity_violations(
    file_metrics: &[FileComplexityMetrics],
    fail_on_violation: bool,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) {
    if !fail_on_violation {
        return;
    }

    let has_violations = has_complexity_violations(file_metrics, max_cyclomatic, max_cognitive);

    if has_violations {
        eprintln!("\n❌ Complexity violations found");
        std::process::exit(1);
    }
}

/// Check if any files have complexity violations
pub(crate) fn has_complexity_violations(
    file_metrics: &[FileComplexityMetrics],
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) -> bool {
    file_metrics.iter().any(|file| {
        file.functions.iter().any(|func| {
            let cyclomatic_exceeded = func.metrics.cyclomatic > max_cyclomatic.unwrap_or(20);
            let cognitive_exceeded = func.metrics.cognitive > max_cognitive.unwrap_or(15);
            cyclomatic_exceeded || cognitive_exceeded
        })
    })
}
