// TDG handlers - extracted for file health (CB-040)
/// Analyzes Technical Debt Gradient (TDG) for a project.
///
/// Technical Debt Gradient measures the rate of technical debt accumulation
/// relative to code complexity and change frequency. Critical for identifying
/// files that are both complex and frequently modified, indicating high
/// maintenance burden and defect risk.
///
/// # Parameters
///
/// * `path` - Root directory of the project to analyze
/// * `threshold` - TDG threshold above which files are considered problematic
/// * `top` - Number of top TDG violating files to report
/// * `format` - Output format for the TDG analysis results
/// * `include_components` - Whether to include component-level TDG breakdown
/// * `output` - Optional output file path
/// * `critical_only` - Only report files above critical TDG threshold
/// * `verbose` - Include detailed TDG calculation methodology
///
/// # Returns
///
/// * `Ok(())` - TDG analysis completed successfully
/// * `Err(anyhow::Error)` - Analysis failed (file access, calculation, or output)
///
/// # TDG Calculation
///
/// TDG = (Complexity Score × Churn Frequency) / Code Size
///
/// Where:
/// - **Complexity Score**: Cyclomatic complexity + cognitive complexity
/// - **Churn Frequency**: Git commits per file over analysis period
/// - **Code Size**: Lines of code normalization factor
///
/// # Interpretation
///
/// - **TDG < 0.5**: Well-maintained, low-risk files
/// - **0.5 <= TDG < 1.0**: Moderate technical debt, monitor
/// - **1.0 <= TDG < 2.0**: High technical debt, prioritize refactoring
/// - **TDG >= 2.0**: Critical technical debt, immediate attention required
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::analysis_utilities::handle_analyze_tdg;
/// use pmat::cli::TdgOutputFormat;
/// use std::path::{Path, PathBuf};
/// use tempfile::tempdir;
/// use std::fs;
///
/// # tokio_test::block_on(async {
/// // Create a temporary project
/// let dir = tempdir().unwrap();
/// let main_rs = dir.path().join("main.rs");
/// fs::write(&main_rs, "fn complex_function() { /* complex code */ }").unwrap();
///
/// // Standard TDG analysis
/// let result = handle_analyze_tdg(
///     dir.path().to_path_buf(),
///     None,  // file - project mode
///     vec![], // files - project mode
///     1.0,  // threshold
///     10,   // top files
///     TdgOutputFormat::Table,
///     false, // no component breakdown
///     None,  // stdout output
///     false, // all files
///     false, // normal verbosity
///     vec![], // include patterns
///     false, // watch mode
/// ).await;
///
/// assert!(result.is_ok());
///
/// // Critical TDG analysis with detailed output
/// let critical_result = handle_analyze_tdg(
///     dir.path().to_path_buf(),
///     None,  // file - project mode
///     vec![], // files - project mode
///     2.0,  // critical threshold
///     5,    // top 5 files
///     TdgOutputFormat::Json,
///     true,  // include components
///     Some(dir.path().join("tdg-report.txt")),
///     true,  // critical only
///     true,  // verbose
///     vec![], // include patterns
///     false, // watch mode
/// ).await;
///
/// assert!(critical_result.is_ok());
/// # });
/// ```
///
/// # CLI Usage Examples
///
/// ```bash
/// # Standard TDG analysis
/// pmat analyze tdg /path/to/project --threshold 1.0 --top-files 10
///
/// # Critical debt identification
/// pmat analyze tdg /path/to/project --threshold 2.0 --critical-only \
///   --format full --output critical-debt.txt
///
/// # Component-level TDG analysis
/// pmat analyze tdg /path/to/project --include-components --verbose \
///   --format json --output tdg-detailed.json
/// ```ignore

// Helper utilities: percentile, primary factor identification, filtering
include!("tdg_helpers.rs");

// Output formatting: table, json, markdown, sarif
include!("tdg_formatting.rs");

// Core analysis: single file, multiple files, project
include!("tdg_analysis.rs");

// Watch mode (feature-gated)
include!("tdg_watch.rs");

// Main entry point and orchestration
include!("tdg_handler.rs");
