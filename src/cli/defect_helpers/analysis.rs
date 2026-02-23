#![cfg_attr(coverage_nightly, coverage(off))]
//! File discovery and defect probability analysis

use crate::cli::defect_prediction_helpers::{
    calculate_simple_churn_score, calculate_simple_complexity, DefectPredictionConfig,
};
use crate::services::defect_probability::{DefectProbabilityCalculator, DefectScore, FileMetrics};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Discover files for defect analysis
pub async fn discover_files_for_defect_analysis(
    project_path: &Path,
    config: &DefectPredictionConfig,
) -> Result<Vec<(PathBuf, String, usize)>> {
    use crate::cli::defect_prediction_helpers::discover_source_files_for_defect_analysis;

    discover_source_files_for_defect_analysis(project_path, config).await
}

/// Analyze defect probability for files
pub async fn analyze_defect_probability(
    files: &[(PathBuf, String, usize)],
    config: &DefectPredictionConfig,
) -> Result<Vec<(String, DefectScore)>> {
    eprintln!("📊 Analyzing {} files...", files.len());

    let calculator = DefectProbabilityCalculator::new();
    let mut predictions = Vec::new();

    for (path, content, line_count) in files {
        let metrics = FileMetrics {
            file_path: path.to_string_lossy().to_string(),
            complexity: calculate_simple_complexity(content) as f32,
            churn_score: calculate_simple_churn_score(content, *line_count),
            duplicate_ratio: 0.0,   // Simplified
            afferent_coupling: 0.0, // Simplified
            efferent_coupling: 0.0, // Simplified
            lines_of_code: *line_count,
            cyclomatic_complexity: 10, // Simplified
            cognitive_complexity: 10,  // Simplified
        };

        let score = calculator.calculate(&metrics);
        predictions.push((path.to_string_lossy().to_string(), score));
    }

    // Apply filters
    if config.high_risk_only {
        predictions.retain(|(_, score)| score.probability > 0.7);
    }

    if !config.include_low_confidence {
        predictions.retain(|(_, score)| score.confidence > config.confidence_threshold);
    }

    // Sort by probability
    predictions.sort_by(|a, b| {
        b.1.probability
            .partial_cmp(&a.1.probability)
            .expect("internal error")
    });

    Ok(predictions)
}
