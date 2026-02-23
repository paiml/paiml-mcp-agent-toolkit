#![cfg_attr(coverage_nightly, coverage(off))]
//! Filter functions for Big-O complexity analysis results

use crate::models::complexity_bound::BigOClass;
use crate::services::big_o_analyzer::{BigOAnalysisReport, FunctionComplexity};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tracing::debug;

/// Apply all report filters
pub(super) fn apply_report_filters(
    report: &mut BigOAnalysisReport,
    high_complexity_only: bool,
    top_files: usize,
    perf: bool,
) {
    if high_complexity_only {
        apply_high_complexity_filter(report, perf);
    }

    if top_files > 0 {
        apply_top_files_filter(report, top_files);
    }
}

/// Filter to keep only high complexity functions
pub(super) fn apply_high_complexity_filter(report: &mut BigOAnalysisReport, perf: bool) {
    let original_count = report.high_complexity_functions.len();

    report
        .high_complexity_functions
        .retain(|f| is_high_complexity_class(&f.time_complexity.class));

    if perf {
        debug!(
            "Filtered from {} to {} high complexity functions",
            original_count,
            report.high_complexity_functions.len()
        );
    }
}

/// Check if complexity class is considered high
pub(super) fn is_high_complexity_class(class: &BigOClass) -> bool {
    matches!(
        class,
        BigOClass::Quadratic | BigOClass::Cubic | BigOClass::Exponential | BigOClass::Factorial
    )
}

/// Apply top files filter
pub(super) fn apply_top_files_filter(report: &mut BigOAnalysisReport, top_files: usize) {
    let file_functions = group_functions_by_file(&report.high_complexity_functions);
    let file_scores = calculate_file_complexity_scores(&file_functions);
    let top_file_paths = get_top_file_paths(file_scores, top_files);

    report
        .high_complexity_functions
        .retain(|f| top_file_paths.contains(&f.file_path));
}

/// Group functions by their file paths
pub(super) fn group_functions_by_file(
    functions: &[FunctionComplexity],
) -> HashMap<PathBuf, Vec<FunctionComplexity>> {
    let mut file_functions: HashMap<PathBuf, Vec<_>> = HashMap::new();
    for func in functions.iter().cloned() {
        file_functions
            .entry(func.file_path.clone())
            .or_default()
            .push(func);
    }

    file_functions
}

/// Calculate complexity scores for files
pub(super) fn calculate_file_complexity_scores(
    file_functions: &HashMap<PathBuf, Vec<FunctionComplexity>>,
) -> Vec<(PathBuf, f64)> {
    let mut file_scores: Vec<(PathBuf, f64)> = file_functions
        .iter()
        .map(|(path, funcs)| {
            let score: f64 = funcs
                .iter()
                .map(|f| get_complexity_class_score(&f.time_complexity.class))
                .sum();
            (path.clone(), score)
        })
        .collect();

    file_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    file_scores
}

/// Get numeric score for complexity class
pub(super) fn get_complexity_class_score(class: &BigOClass) -> f64 {
    match class {
        BigOClass::Constant => 1.0,
        BigOClass::Logarithmic => 2.0,
        BigOClass::Linear => 3.0,
        BigOClass::Linearithmic => 4.0,
        BigOClass::Quadratic => 5.0,
        BigOClass::Cubic => 6.0,
        BigOClass::Exponential => 7.0,
        BigOClass::Factorial => 8.0,
        BigOClass::Unknown => 3.0,
    }
}

/// Get top file paths from scores
pub(super) fn get_top_file_paths(
    file_scores: Vec<(PathBuf, f64)>,
    top_files: usize,
) -> HashSet<PathBuf> {
    file_scores
        .into_iter()
        .take(top_files)
        .map(|(path, _)| path)
        .collect()
}
