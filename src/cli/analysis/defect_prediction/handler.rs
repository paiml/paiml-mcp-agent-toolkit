//! Main handler for defect prediction analysis

use crate::cli::defect_helpers::discover_files_for_defect_analysis;
use crate::cli::defect_prediction_helpers::{collect_file_metrics, DefectPredictionConfig};
use crate::cli::DefectPredictionOutputFormat;
use crate::services::defect_probability::{DefectProbabilityCalculator, DefectScore};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::output_formats::{format_defect_output, output_results};

/// Handle defect prediction analysis with real ML-based implementation
/// Toyota Way: Extract Method - Reduced complexity by separating concerns
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_defect_prediction(
    project_path: PathBuf,
    confidence_threshold: f32,
    min_lines: usize,
    include_low_confidence: bool,
    format: DefectPredictionOutputFormat,
    high_risk_only: bool,
    include_recommendations: bool,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
    top_files: usize,
) -> Result<()> {
    let start_time = Instant::now();
    print_analysis_header(&project_path, confidence_threshold, high_risk_only);

    let config = create_defect_prediction_config(
        confidence_threshold,
        min_lines,
        include_low_confidence,
        high_risk_only,
        include_recommendations,
        include,
        exclude,
    );

    let files = discover_and_validate_files(&project_path, &config).await?;
    let predictions = calculate_defect_predictions(&files)?;
    let filtered_predictions = filter_and_sort_predictions(
        predictions,
        high_risk_only,
        include_low_confidence,
        confidence_threshold,
        top_files,
    );

    let elapsed = start_time.elapsed();
    let content = format_defect_output(
        format,
        &filtered_predictions,
        elapsed,
        include_recommendations,
    )?;
    output_results(content, output, perf, elapsed).await?;

    Ok(())
}

/// Format predictions as summary
/// Toyota Way: Extract Method - Print analysis header information
fn print_analysis_header(project_path: &Path, confidence_threshold: f32, high_risk_only: bool) {
    eprintln!("🔮 Analyzing defect probability using ML-based analysis...");
    eprintln!("📁 Project path: {}", project_path.display());
    eprintln!("🎯 Confidence threshold: {confidence_threshold}");
    eprintln!("📊 High risk only: {high_risk_only}");
}

/// Toyota Way: Extract Method - Create configuration object
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn create_defect_prediction_config(
    confidence_threshold: f32,
    min_lines: usize,
    include_low_confidence: bool,
    high_risk_only: bool,
    include_recommendations: bool,
    include: Option<String>,
    exclude: Option<String>,
) -> DefectPredictionConfig {
    DefectPredictionConfig {
        confidence_threshold,
        min_lines,
        include_low_confidence,
        high_risk_only,
        include_recommendations,
        include,
        exclude,
    }
}

/// Toyota Way: Extract Method - Discover and validate files for analysis
async fn discover_and_validate_files(
    project_path: &Path,
    config: &DefectPredictionConfig,
) -> Result<Vec<(std::path::PathBuf, String, usize)>> {
    let files = discover_files_for_defect_analysis(project_path, config).await?;
    eprintln!("📂 Found {} files matching criteria", files.len());

    if files.is_empty() {
        eprintln!("⚠️  No files found matching the criteria");
        return Err(anyhow::anyhow!("No files found matching criteria"));
    }

    Ok(files)
}

/// Toyota Way: Extract Method - Calculate defect predictions using ML service
fn calculate_defect_predictions(
    files: &[(std::path::PathBuf, String, usize)],
) -> Result<Vec<(String, DefectScore)>> {
    let file_metrics = collect_file_metrics(files);
    let calculator = DefectProbabilityCalculator::new();

    Ok(file_metrics
        .into_iter()
        .map(|metrics| {
            let score = calculator.calculate(&metrics);
            (metrics.file_path, score)
        })
        .collect())
}

/// Toyota Way: Extract Method - Filter and sort predictions based on criteria
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn filter_and_sort_predictions(
    mut predictions: Vec<(String, DefectScore)>,
    high_risk_only: bool,
    include_low_confidence: bool,
    confidence_threshold: f32,
    top_files: usize,
) -> Vec<(String, DefectScore)> {
    if high_risk_only {
        predictions.retain(|(_, score)| score.probability > 0.7);
    }

    if !include_low_confidence {
        predictions.retain(|(_, score)| score.confidence > confidence_threshold);
    }

    predictions.sort_by(|a, b| {
        b.1.probability
            .partial_cmp(&a.1.probability)
            .expect("internal error")
    });

    if top_files > 0 && predictions.len() > top_files {
        predictions.truncate(top_files);
    }

    predictions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::defect_probability::RiskLevel;

    fn score(p: f32, c: f32) -> DefectScore {
        DefectScore {
            probability: p,
            confidence: c,
            contributing_factors: vec![],
            risk_level: if p > 0.7 {
                RiskLevel::High
            } else if p > 0.3 {
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            },
            recommendations: vec![],
        }
    }

    fn pred(file: &str, p: f32, c: f32) -> (String, DefectScore) {
        (file.to_string(), score(p, c))
    }

    // ── print_analysis_header ───────────────────────────────────────────────

    #[test]
    fn test_print_analysis_header_no_panic() {
        // eprintln side-effect — exercise both flag combos
        print_analysis_header(Path::new("."), 0.5, true);
        print_analysis_header(Path::new("."), 0.8, false);
    }

    // ── create_defect_prediction_config ─────────────────────────────────────

    #[test]
    fn test_create_defect_prediction_config_propagates_fields() {
        let cfg = create_defect_prediction_config(
            0.75,
            10,
            true,
            true,
            false,
            Some("*.rs".to_string()),
            Some("tests/*".to_string()),
        );
        assert_eq!(cfg.confidence_threshold, 0.75);
        assert_eq!(cfg.min_lines, 10);
        assert!(cfg.include_low_confidence);
        assert!(cfg.high_risk_only);
        assert!(!cfg.include_recommendations);
        assert_eq!(cfg.include, Some("*.rs".to_string()));
        assert_eq!(cfg.exclude, Some("tests/*".to_string()));
    }

    #[test]
    fn test_create_defect_prediction_config_no_filters() {
        let cfg = create_defect_prediction_config(0.5, 0, false, false, true, None, None);
        assert!(cfg.include.is_none());
        assert!(cfg.exclude.is_none());
        assert!(cfg.include_recommendations);
    }

    // ── filter_and_sort_predictions ─────────────────────────────────────────

    #[test]
    fn test_filter_and_sort_predictions_high_risk_only() {
        let preds = vec![
            pred("h.rs", 0.9, 0.8),
            pred("m.rs", 0.5, 0.8),
            pred("l.rs", 0.1, 0.8),
        ];
        let out = filter_and_sort_predictions(preds, true, true, 0.0, 0);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].0, "h.rs");
    }

    #[test]
    fn test_filter_and_sort_predictions_at_07_dropped_when_high_risk_only() {
        // > 0.7 is high; == 0.7 falls through (filter is `> 0.7`, not `>= 0.7`)
        let preds = vec![pred("a.rs", 0.7, 0.9)];
        let out = filter_and_sort_predictions(preds, true, true, 0.0, 0);
        assert!(out.is_empty());
    }

    #[test]
    fn test_filter_and_sort_predictions_filters_low_confidence() {
        let preds = vec![pred("h.rs", 0.9, 0.5), pred("m.rs", 0.5, 0.9)];
        // include_low_confidence = false, threshold = 0.7 → drop confidence ≤ 0.7
        let out = filter_and_sort_predictions(preds, false, false, 0.7, 0);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].0, "m.rs");
    }

    #[test]
    fn test_filter_and_sort_predictions_include_low_confidence_keeps_all() {
        let preds = vec![pred("h.rs", 0.9, 0.5), pred("m.rs", 0.5, 0.9)];
        let out = filter_and_sort_predictions(preds, false, true, 0.7, 0);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn test_filter_and_sort_predictions_sorts_descending_by_probability() {
        let preds = vec![
            pred("low.rs", 0.1, 0.9),
            pred("high.rs", 0.9, 0.9),
            pred("mid.rs", 0.5, 0.9),
        ];
        let out = filter_and_sort_predictions(preds, false, true, 0.0, 0);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].0, "high.rs");
        assert_eq!(out[1].0, "mid.rs");
        assert_eq!(out[2].0, "low.rs");
    }

    #[test]
    fn test_filter_and_sort_predictions_truncates_to_top_files() {
        let preds = vec![
            pred("a.rs", 0.9, 0.9),
            pred("b.rs", 0.8, 0.9),
            pred("c.rs", 0.7, 0.9),
            pred("d.rs", 0.6, 0.9),
        ];
        let out = filter_and_sort_predictions(preds, false, true, 0.0, 2);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].0, "a.rs");
        assert_eq!(out[1].0, "b.rs");
    }

    #[test]
    fn test_filter_and_sort_predictions_top_zero_keeps_all() {
        let preds = vec![pred("a.rs", 0.9, 0.9), pred("b.rs", 0.5, 0.9)];
        let out = filter_and_sort_predictions(preds, false, true, 0.0, 0);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn test_filter_and_sort_predictions_top_larger_than_len_keeps_all() {
        let preds = vec![pred("a.rs", 0.9, 0.9), pred("b.rs", 0.5, 0.9)];
        let out = filter_and_sort_predictions(preds, false, true, 0.0, 100);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn test_filter_and_sort_predictions_combined_filters() {
        let preds = vec![
            pred("h_hc.rs", 0.95, 0.95), // high prob, high conf — kept
            pred("h_lc.rs", 0.85, 0.5),  // high prob, low conf — dropped (low conf)
            pred("m_hc.rs", 0.5, 0.9),   // medium — dropped (high_risk_only)
            pred("l_hc.rs", 0.1, 0.9),   // low — dropped (high_risk_only)
        ];
        let out = filter_and_sort_predictions(preds, true, false, 0.7, 5);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].0, "h_hc.rs");
    }
}
