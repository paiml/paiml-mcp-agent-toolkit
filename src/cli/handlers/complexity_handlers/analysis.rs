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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn analyze_single_file(
    file_path: &Path,
    config: &ComplexityConfig,
) -> Result<Vec<FileComplexityMetrics>> {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
    // Auto-detection used to RESTRICT the walk to the one language it guessed.
    // A directory holding a.go, app.ts and main.py therefore reported
    // "Files analyzed: 1 / Total functions: 1" — whichever toolchain detection
    // happened to win that run — and printed the summary as if it covered the
    // project, with no hint that two of three source files were skipped.
    // Detection is only a label now; an explicit `--toolchain` still restricts.
    let explicit_toolchain = config.toolchain.as_deref();

    if let Some(toolchain) = explicit_toolchain {
        eprintln!("🔍 Analyzing {toolchain} files only (--toolchain {toolchain})...");
        crate::cli::analysis_utilities::analyze_project_files(
            &config.project_path,
            Some(toolchain),
            &config.include,
            config.max_cyclomatic,
            config.max_cognitive,
        )
        .await
    } else {
        match detected_toolchain {
            Some(toolchain) => {
                eprintln!("🔍 Analyzing {toolchain} project complexity (all languages)...");
            }
            None => eprintln!("🔍 Analyzing project complexity (multi-language)..."),
        }
        crate::cli::analysis_utilities::analyze_project_files(
            &config.project_path,
            None, // Analyze every supported language, not just the detected one
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
        eprintln!(
            "ℹ️  Filtered {} file(s) with no functions exceeding thresholds ({})",
            filtered_count,
            describe_thresholds(max_cyclomatic, max_cognitive)
        );
    }

    filtered_count
}

/// Name the thresholds that were actually in force.
///
/// An unset threshold used to be printed as its saturating sentinel —
/// "cognitive > 65535" — which reads as a real limit that no function can ever
/// exceed, and told the user a gate was running that was not. A threshold that
/// was never set is simply not named.
fn describe_thresholds(max_cyclomatic: Option<u16>, max_cognitive: Option<u16>) -> String {
    let mut in_force = Vec::new();
    if let Some(threshold) = max_cyclomatic {
        in_force.push(format!("cyclomatic > {threshold}"));
    }
    if let Some(threshold) = max_cognitive {
        in_force.push(format!("cognitive > {threshold}"));
    }
    if in_force.is_empty() {
        return "no thresholds set".to_string();
    }
    in_force.join(", ")
}

/// Aggregate over every analyzed file, then list only the top-N slice.
///
/// The summary and the list are built here together so they cannot drift: the
/// handler used to aggregate AFTER truncation and then overwrite
/// `summary.total_files` with the project count, which is how one unchanged
/// 1070-file tree reported `total_files: 1070` next to `total_functions: 159`
/// (true value 10148) and `technical_debt_hours: 388.75` (true 1644.25).
///
/// `analyzed` is consumed for the aggregate; `listed` is what the renderer
/// prints. See `contracts/pmat-no-fabrication-v1.yaml` — a cap must never be
/// presented as a total.
pub(super) fn build_report_over_analyzed_files(
    analyzed: Vec<FileComplexityMetrics>,
    listed: Vec<FileComplexityMetrics>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) -> crate::services::complexity::ComplexityReport {
    let analyzed_count = analyzed.len();
    let mut report = crate::services::complexity::aggregate_results_with_thresholds(
        analyzed,
        max_cyclomatic,
        max_cognitive,
    );
    report.summary.total_files = analyzed_count;
    report.files = listed;
    report
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

#[cfg(test)]
mod multi_language_tests {
    //! Regression tests for two defects in this module: a detected toolchain
    //! silently restricted the project walk to one language, and an unset
    //! threshold was reported as the unreachable sentinel 65535.
    use super::{analyze_project, describe_thresholds};
    use crate::cli::handlers::complexity_handlers::ComplexityConfig;

    fn write_polyglot(dir: &std::path::Path) {
        std::fs::write(
            dir.join("a.go"),
            "package main\nfunc Add(a int, b int) int { if a > b { return a }\n return b }\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("app.ts"),
            "export function add(a: number, b: number): number { return a > b ? a : b; }\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("main.py"),
            "def add(a, b):\n    if a > b:\n        return a\n    return b\n",
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_detected_toolchain_does_not_drop_other_languages() {
        let temp = tempfile::TempDir::new().unwrap();
        write_polyglot(temp.path());

        // No `--toolchain` flag: detection may name any one language, but it
        // must not become the whole project.
        let config = ComplexityConfig::from_args(
            temp.path().to_path_buf(),
            None,
            None,
            None,
            Vec::new(),
            60,
            0,
        );
        let metrics = analyze_project(Some("typescript".to_string()), &config)
            .await
            .unwrap();

        let mut extensions: Vec<String> = metrics
            .iter()
            .filter_map(|m| {
                std::path::Path::new(&m.path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(str::to_string)
            })
            .collect();
        extensions.sort();
        extensions.dedup();

        assert!(
            extensions.len() >= 2,
            "detecting one toolchain must not restrict the walk to it; analyzed {extensions:?}"
        );
    }

    #[tokio::test]
    async fn test_explicit_toolchain_still_restricts() {
        let temp = tempfile::TempDir::new().unwrap();
        write_polyglot(temp.path());

        let config = ComplexityConfig::from_args(
            temp.path().to_path_buf(),
            Some("go".to_string()),
            None,
            None,
            Vec::new(),
            60,
            0,
        );
        let metrics = analyze_project(Some("go".to_string()), &config)
            .await
            .unwrap();

        assert!(
            metrics.iter().all(|m| m.path.ends_with(".go")),
            "--toolchain go must analyze only Go files"
        );
    }

    #[test]
    fn test_unset_threshold_is_not_reported_as_65535() {
        let described = describe_thresholds(Some(20), None);
        assert_eq!(described, "cyclomatic > 20");
        assert!(
            !described.contains("65535"),
            "an unset cognitive threshold must not be printed as u16::MAX"
        );
        assert_eq!(
            describe_thresholds(Some(20), Some(15)),
            "cyclomatic > 20, cognitive > 15"
        );
    }
}
