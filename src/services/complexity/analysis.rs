#![cfg_attr(coverage_nightly, coverage(off))]
//! Internal analysis helpers for aggregating complexity metrics.

use super::rules::{CognitiveComplexityRule, ComplexityRule, CyclomaticComplexityRule};
use super::types::{
    ComplexityHotspot, ComplexityReport, ComplexitySummary, ComplexityThresholds,
    FileComplexityMetrics, FunctionComplexity, Violation,
};

/// Intermediate data structure for analysis results
pub(super) struct AnalysisData {
    pub all_cyclomatic: Vec<u16>,
    pub all_cognitive: Vec<u16>,
    pub violations: Vec<Violation>,
    pub hotspots: Vec<ComplexityHotspot>,
    pub total_functions: usize,
}

/// Analyze file metrics and collect data
pub(super) fn analyze_file_metrics(
    file_metrics: &[FileComplexityMetrics],
    rules: &(CyclomaticComplexityRule, CognitiveComplexityRule),
    thresholds: &ComplexityThresholds,
) -> AnalysisData {
    let mut data = AnalysisData {
        all_cyclomatic: Vec::new(),
        all_cognitive: Vec::new(),
        violations: Vec::new(),
        hotspots: Vec::new(),
        total_functions: 0,
    };

    for file in file_metrics {
        process_file_functions(file, rules, thresholds, &mut data);
        process_file_classes(file, rules, &mut data);
    }

    data
}

/// Process functions in a file
fn process_file_functions(
    file: &FileComplexityMetrics,
    rules: &(CyclomaticComplexityRule, CognitiveComplexityRule),
    thresholds: &ComplexityThresholds,
    data: &mut AnalysisData,
) {
    let (cyclomatic_rule, cognitive_rule) = rules;

    for func in &file.functions {
        data.total_functions += 1;
        data.all_cyclomatic.push(func.metrics.cyclomatic);
        data.all_cognitive.push(func.metrics.cognitive);

        check_function_violations(
            func,
            file,
            cyclomatic_rule,
            cognitive_rule,
            &mut data.violations,
        );
        check_function_hotspots(func, file, thresholds, &mut data.hotspots);
    }
}

/// Process classes and their methods in a file
fn process_file_classes(
    file: &FileComplexityMetrics,
    rules: &(CyclomaticComplexityRule, CognitiveComplexityRule),
    data: &mut AnalysisData,
) {
    let (cyclomatic_rule, cognitive_rule) = rules;

    for class in &file.classes {
        for method in &class.methods {
            data.total_functions += 1;
            data.all_cyclomatic.push(method.metrics.cyclomatic);
            data.all_cognitive.push(method.metrics.cognitive);

            check_method_violations(
                method,
                file,
                cyclomatic_rule,
                cognitive_rule,
                &mut data.violations,
            );
        }
    }
}

/// Check function for complexity violations
fn check_function_violations(
    func: &FunctionComplexity,
    file: &FileComplexityMetrics,
    cyclomatic_rule: &CyclomaticComplexityRule,
    cognitive_rule: &CognitiveComplexityRule,
    violations: &mut Vec<Violation>,
) {
    if let Some(violation) =
        cyclomatic_rule.evaluate(&func.metrics, &file.path, func.line_start, Some(&func.name))
    {
        violations.push(violation);
    }

    if let Some(violation) =
        cognitive_rule.evaluate(&func.metrics, &file.path, func.line_start, Some(&func.name))
    {
        violations.push(violation);
    }
}

/// Check method for complexity violations
fn check_method_violations(
    method: &FunctionComplexity,
    file: &FileComplexityMetrics,
    cyclomatic_rule: &CyclomaticComplexityRule,
    cognitive_rule: &CognitiveComplexityRule,
    violations: &mut Vec<Violation>,
) {
    if let Some(violation) = cyclomatic_rule.evaluate(
        &method.metrics,
        &file.path,
        method.line_start,
        Some(&method.name),
    ) {
        violations.push(violation);
    }

    if let Some(violation) = cognitive_rule.evaluate(
        &method.metrics,
        &file.path,
        method.line_start,
        Some(&method.name),
    ) {
        violations.push(violation);
    }
}

/// Check function for complexity hotspots
fn check_function_hotspots(
    func: &FunctionComplexity,
    file: &FileComplexityMetrics,
    thresholds: &ComplexityThresholds,
    hotspots: &mut Vec<ComplexityHotspot>,
) {
    if func.metrics.cyclomatic > thresholds.cyclomatic_warn {
        hotspots.push(ComplexityHotspot {
            file: file.path.clone(),
            function: Some(func.name.clone()),
            line: func.line_start,
            complexity: func.metrics.cyclomatic,
            complexity_type: "cyclomatic".to_string(),
        });
    }
}

/// Summary statistics structure
pub(super) struct SummaryStats {
    pub median_cyclomatic: f32,
    pub median_cognitive: f32,
    pub max_cyclomatic: u16,
    pub max_cognitive: u16,
    pub p90_cyclomatic: u16,
    pub p90_cognitive: u16,
}

/// Calculate summary statistics from analysis data
pub(super) fn calculate_summary_statistics(data: &mut AnalysisData) -> SummaryStats {
    data.all_cyclomatic.sort_unstable();
    data.all_cognitive.sort_unstable();

    let p90_stats = calculate_percentiles(&data.all_cyclomatic, &data.all_cognitive);
    let median_stats = calculate_medians(&data.all_cyclomatic, &data.all_cognitive);
    let max_stats = calculate_max_values(&data.all_cyclomatic, &data.all_cognitive);

    // Sort and limit hotspots
    data.hotspots
        .sort_unstable_by_key(|b| std::cmp::Reverse(b.complexity));
    data.hotspots.truncate(10);

    SummaryStats {
        median_cyclomatic: median_stats.0,
        median_cognitive: median_stats.1,
        max_cyclomatic: max_stats.0,
        max_cognitive: max_stats.1,
        p90_cyclomatic: p90_stats.0,
        p90_cognitive: p90_stats.1,
    }
}

/// Calculate 90th percentile values
fn calculate_percentiles(all_cyclomatic: &[u16], all_cognitive: &[u16]) -> (u16, u16) {
    let p90_index = (all_cyclomatic.len() as f32 * 0.9) as usize;
    let p90_cyclomatic = all_cyclomatic.get(p90_index).copied().unwrap_or(0);
    let p90_cognitive = all_cognitive.get(p90_index).copied().unwrap_or(0);
    (p90_cyclomatic, p90_cognitive)
}

/// Calculate median values
fn calculate_medians(all_cyclomatic: &[u16], all_cognitive: &[u16]) -> (f32, f32) {
    let median_cyclomatic = calculate_median(all_cyclomatic);
    let median_cognitive = calculate_median(all_cognitive);
    (median_cyclomatic, median_cognitive)
}

/// Calculate median for a sorted array
fn calculate_median(values: &[u16]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }

    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        f32::from(values[mid - 1] + values[mid]) / 2.0
    } else {
        f32::from(values[mid])
    }
}

/// Calculate maximum values
fn calculate_max_values(all_cyclomatic: &[u16], all_cognitive: &[u16]) -> (u16, u16) {
    let max_cyclomatic = all_cyclomatic.iter().max().copied().unwrap_or(0);
    let max_cognitive = all_cognitive.iter().max().copied().unwrap_or(0);
    (max_cyclomatic, max_cognitive)
}

/// Calculate technical debt hours from violations
pub(super) fn calculate_technical_debt(violations: &[Violation]) -> f32 {
    let debt_minutes: f32 = violations
        .iter()
        .map(|v| match v {
            Violation::Error {
                value, threshold, ..
            } => f32::from(value - threshold) * 30.0,
            Violation::Warning {
                value, threshold, ..
            } => f32::from(value - threshold) * 15.0,
        })
        .sum();
    debt_minutes / 60.0
}

/// Build the final complexity report
pub(super) fn build_complexity_report(
    file_metrics: Vec<FileComplexityMetrics>,
    analysis_data: AnalysisData,
    summary_stats: SummaryStats,
    technical_debt_hours: f32,
) -> ComplexityReport {
    ComplexityReport {
        summary: ComplexitySummary {
            total_files: file_metrics.len(),
            total_functions: analysis_data.total_functions,
            median_cyclomatic: summary_stats.median_cyclomatic,
            median_cognitive: summary_stats.median_cognitive,
            max_cyclomatic: summary_stats.max_cyclomatic,
            max_cognitive: summary_stats.max_cognitive,
            p90_cyclomatic: summary_stats.p90_cyclomatic,
            p90_cognitive: summary_stats.p90_cognitive,
            technical_debt_hours,
        },
        violations: analysis_data.violations,
        hotspots: analysis_data.hotspots,
        files: file_metrics,
    }
}
