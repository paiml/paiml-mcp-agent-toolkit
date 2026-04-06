#![cfg_attr(coverage_nightly, coverage(off))]
//! Public API for aggregating complexity results across files.

use super::analysis::{
    analyze_file_metrics, build_complexity_report, calculate_summary_statistics,
    calculate_technical_debt,
};
use super::rules::{CognitiveComplexityRule, CyclomaticComplexityRule};
use super::types::{ComplexityReport, ComplexityThresholds, FileComplexityMetrics};

/// Aggregate complexity results from multiple files
/// Aggregates file-level complexity metrics into a comprehensive report
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::complexity::{aggregate_results, FileComplexityMetrics};
///
/// let metrics = vec![];
/// let report = aggregate_results(metrics);
/// assert_eq!(report.summary.total_files, 0);
/// ```
/// Aggregates complexity metrics from multiple files into a summary report
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::complexity::{aggregate_results, FileComplexityMetrics, ComplexityMetrics};
///
/// let file = FileComplexityMetrics {
///     path: "src/main.rs".to_string(),
///     total_complexity: ComplexityMetrics {
///         cyclomatic: 10,
///         cognitive: 8,
///         nesting_max: 3,
///         lines: 50,
///         halstead: None,
///     },
///     functions: vec![],
///     classes: vec![],
/// };
///
/// let report = aggregate_results(vec![file]);
/// assert_eq!(report.files.len(), 1);
/// ```
#[must_use]
pub fn aggregate_results(file_metrics: Vec<FileComplexityMetrics>) -> ComplexityReport {
    debug_assert!(!file_metrics.is_empty(), "file_metrics must not be empty");
    aggregate_results_with_thresholds(file_metrics, None, None)
}

/// Aggregate complexity results with custom thresholds
///
/// This function allows customizing the complexity thresholds used to determine violations,
/// addressing issue #32 where `--max-cyclomatic` didn't affect report output.
///
/// # Arguments
///
/// * `file_metrics` - Vector of file complexity metrics to aggregate
/// * `max_cyclomatic` - Optional custom maximum cyclomatic complexity threshold
/// * `max_cognitive` - Optional custom maximum cognitive complexity threshold
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::complexity::*;
///
/// let metrics = ComplexityMetrics {
///     cyclomatic: 25,
///     cognitive: 30,
///     nesting_max: 3,
///     lines: 100,
///     halstead: None,
/// };
///
/// let func = FunctionComplexity {
///     name: "complex_function".to_string(),
///     line_start: 10,
///     line_end: 50,
///     metrics,
/// };
///
/// let file = FileComplexityMetrics {
///     path: "src/main.rs".to_string(),
///     total_complexity: metrics,
///     functions: vec![func],
///     classes: vec![],
/// };
///
/// // With custom threshold of 20, the function with complexity 25 will be a violation
/// let report = aggregate_results_with_thresholds(vec![file.clone()], Some(20), None);
/// assert_eq!(report.violations.len(), 2); // Both cyclomatic (25) and cognitive (30) exceed 20
/// assert!(matches!(report.violations[0], Violation::Error { .. }));
///
/// // With cyclomatic threshold of 35 and cognitive threshold of 35, no violations
/// let report2 = aggregate_results_with_thresholds(vec![file], Some(35), Some(35));
/// assert_eq!(report2.violations.len(), 0);
/// ```
#[must_use]
pub fn aggregate_results_with_thresholds(
    file_metrics: Vec<FileComplexityMetrics>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) -> ComplexityReport {
    debug_assert!(!file_metrics.is_empty(), "file_metrics must not be empty");
    let thresholds = build_custom_thresholds(max_cyclomatic, max_cognitive);
    let rules = create_complexity_rules(&thresholds);
    let mut analysis_data = analyze_file_metrics(&file_metrics, &rules, &thresholds);
    let summary_stats = calculate_summary_statistics(&mut analysis_data);
    let technical_debt = calculate_technical_debt(&analysis_data.violations);

    build_complexity_report(file_metrics, analysis_data, summary_stats, technical_debt)
}

/// Build custom thresholds from optional parameters
fn build_custom_thresholds(
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) -> ComplexityThresholds {
    let mut thresholds = ComplexityThresholds::default();

    if let Some(max_cyc) = max_cyclomatic {
        // Narrow warning band: only warn within 2 of error threshold
        thresholds.cyclomatic_warn = max_cyc.saturating_sub(2).max(1);
        thresholds.cyclomatic_error = max_cyc;
    }
    if let Some(max_cog) = max_cognitive {
        thresholds.cognitive_warn = max_cog.saturating_sub(2).max(1);
        thresholds.cognitive_error = max_cog;
    }

    thresholds
}

/// Create complexity rules from thresholds
fn create_complexity_rules(
    thresholds: &ComplexityThresholds,
) -> (CyclomaticComplexityRule, CognitiveComplexityRule) {
    let cyclomatic_rule = CyclomaticComplexityRule::new(thresholds);
    let cognitive_rule = CognitiveComplexityRule::new(thresholds);
    (cyclomatic_rule, cognitive_rule)
}
