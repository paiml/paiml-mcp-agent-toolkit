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
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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
