// TDG helper functions - percentile, primary factor, refactoring estimates, filtering

/// Calculate percentile value from sorted array
pub fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    let index = (sorted_values.len() as f64 * p) as usize;
    let index = index.min(sorted_values.len() - 1);
    sorted_values[index]
}

/// Identify the primary contributing factor from TDG components
pub fn identify_primary_factor(components: &crate::models::tdg::TDGComponents) -> String {
    let mut factors = [
        (components.complexity * 0.30, "High Complexity"),
        (components.churn * 0.35, "Frequent Changes"),
        (components.coupling * 0.15, "High Coupling"),
        (components.domain_risk * 0.10, "Domain Risk"),
        (components.duplication * 0.10, "Code Duplication"),
    ];

    factors.sort_unstable_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    factors[0].1.to_string()
}

/// Estimate refactoring hours based on TDG score
pub fn estimate_refactoring_hours(tdg_score: f64) -> f64 {
    // Empirical formula: hours = base * multiplier^tdg
    let base_hours = 2.0;
    let multiplier: f64 = 1.8;
    base_hours * multiplier.powf(tdg_score)
}

/// Resolve file path relative to project directory
fn resolve_file_path(project_path: &Path, file_path: PathBuf) -> PathBuf {
    if file_path.is_absolute() {
        file_path
    } else {
        project_path.join(&file_path)
    }
}

/// Check if score should be included based on filters
fn should_include_score(
    score: &crate::models::tdg::TDGScore,
    threshold: f64,
    critical_only: bool,
) -> bool {
    if critical_only && score.value <= 2.5 {
        return false;
    }
    if score.value < threshold {
        return false;
    }
    true
}

/// Apply sorting and `top_files` limit to results
fn apply_results_filtering(
    mut results: Vec<(crate::models::tdg::TDGScore, PathBuf)>,
    top_files: usize,
) -> Vec<(crate::models::tdg::TDGScore, PathBuf)> {
    // Sort by TDG score descending
    results.sort_unstable_by(|a, b| {
        b.0.value
            .partial_cmp(&a.0.value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Apply top_files limit
    if top_files > 0 && results.len() > top_files {
        results.truncate(top_files);
    }

    results
}

/// Create a summary from individual file results
fn create_summary_from_file_results(
    results: &[(crate::models::tdg::TDGScore, PathBuf)],
) -> crate::models::tdg::TDGSummary {
    use crate::models::tdg::{TDGHotspot, TDGSeverity, TDGSummary};

    let total_files = results.len();
    let critical_files = results
        .iter()
        .filter(|(s, _)| matches!(s.severity, TDGSeverity::Critical))
        .count();
    let warning_files = results
        .iter()
        .filter(|(s, _)| matches!(s.severity, TDGSeverity::Warning))
        .count();

    let tdg_values: Vec<f64> = results.iter().map(|(s, _)| s.value).collect();
    let average_tdg = if tdg_values.is_empty() {
        0.0
    } else {
        tdg_values.iter().sum::<f64>() / tdg_values.len() as f64
    };

    // Calculate percentiles
    let mut sorted_values = tdg_values;
    sorted_values.sort_unstable_by(|a, b| {
        a.partial_cmp(b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let p95_tdg = percentile(&sorted_values, 0.95);
    let p99_tdg = percentile(&sorted_values, 0.99);

    // Create hotspots
    let hotspots = results
        .iter()
        .map(|(score, path)| TDGHotspot {
            path: path.display().to_string(),
            tdg_score: score.value,
            primary_factor: identify_primary_factor(&score.components),
            estimated_hours: estimate_refactoring_hours(score.value),
        })
        .collect();

    let estimated_debt_hours = results
        .iter()
        .map(|(s, _)| estimate_refactoring_hours(s.value))
        .sum();

    TDGSummary {
        total_files,
        critical_files,
        warning_files,
        average_tdg,
        p95_tdg,
        p99_tdg,
        estimated_debt_hours,
        hotspots,
    }
}
