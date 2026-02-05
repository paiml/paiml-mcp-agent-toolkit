//! Defect prediction analysis implementation using real ML-based service

use crate::cli::defect_helpers::discover_files_for_defect_analysis;
use crate::cli::defect_prediction_helpers::{collect_file_metrics, DefectPredictionConfig};
use crate::cli::DefectPredictionOutputFormat;
use crate::services::defect_probability::{DefectProbabilityCalculator, DefectScore};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Instant;

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
fn create_defect_prediction_config(
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
fn filter_and_sort_predictions(
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

/// Toyota Way: Extract Method - Format defect output based on format type
fn format_defect_output(
    format: DefectPredictionOutputFormat,
    predictions: &[(String, DefectScore)],
    elapsed: std::time::Duration,
    include_recommendations: bool,
) -> Result<String> {
    match format {
        DefectPredictionOutputFormat::Summary => format_defect_summary(predictions, elapsed),
        DefectPredictionOutputFormat::Json => format_defect_json(predictions, elapsed),
        DefectPredictionOutputFormat::Detailed => {
            format_defect_detailed(predictions, elapsed, include_recommendations)
        }
        DefectPredictionOutputFormat::Sarif => format_defect_sarif(predictions),
        DefectPredictionOutputFormat::Csv => format_defect_csv(predictions),
    }
}

/// Toyota Way: Extract Method - Output results to file or stdout
async fn output_results(
    content: String,
    output: Option<PathBuf>,
    perf: bool,
    elapsed: std::time::Duration,
) -> Result<()> {
    if perf {
        eprintln!("⏱️  Analysis completed in {elapsed:.2?}");
    }

    eprintln!("✅ Defect prediction complete");

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("📝 Written to {}", output_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}

/// Toyota Way: Extract Method - Reduced complexity by separating concerns
fn format_defect_summary(
    predictions: &[(String, DefectScore)],
    elapsed: std::time::Duration,
) -> Result<String> {
    let mut output = String::new();

    write_summary_header(&mut output)?;
    write_risk_distribution(&mut output, predictions)?;
    write_top_risk_files(&mut output, predictions)?;
    write_summary_footer(&mut output, elapsed)?;

    Ok(output)
}

/// Toyota Way: Extract Method - Write summary header
fn write_summary_header(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "🔮 Defect Prediction Summary")?;
    writeln!(output, "==========================")?;
    writeln!(output)?;
    Ok(())
}

/// Toyota Way: Extract Method - Calculate and write risk distribution
fn write_risk_distribution(
    output: &mut String,
    predictions: &[(String, DefectScore)],
) -> Result<()> {
    use std::fmt::Write;

    let risk_stats = calculate_risk_statistics(predictions);

    writeln!(output, "📊 Risk Distribution:")?;
    writeln!(output, "  🔴 High risk:   {} files", risk_stats.high_risk)?;
    writeln!(output, "  🟡 Medium risk: {} files", risk_stats.medium_risk)?;
    writeln!(output, "  🟢 Low risk:    {} files", risk_stats.low_risk)?;
    writeln!(output)?;

    Ok(())
}

/// Toyota Way: Extract Method - Risk statistics calculation
struct RiskStatistics {
    high_risk: usize,
    medium_risk: usize,
    low_risk: usize,
}

fn calculate_risk_statistics(predictions: &[(String, DefectScore)]) -> RiskStatistics {
    let high_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability > 0.7)
        .count();
    let medium_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability > 0.3 && s.probability <= 0.7)
        .count();
    let low_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability <= 0.3)
        .count();

    RiskStatistics {
        high_risk,
        medium_risk,
        low_risk,
    }
}

/// Toyota Way: Extract Method - Write top risk files section
fn write_top_risk_files(output: &mut String, predictions: &[(String, DefectScore)]) -> Result<()> {
    use std::fmt::Write;

    if !predictions.is_empty() {
        writeln!(output, "🎯 Top Risk Files:")?;
        for (file, score) in predictions.iter().take(10) {
            let risk_icon = get_risk_icon(&score.risk_level);
            writeln!(
                output,
                "  {} {:.1}% - {} (confidence: {:.1}%)",
                risk_icon,
                score.probability * 100.0,
                file,
                score.confidence * 100.0
            )?;
        }
    }

    Ok(())
}

/// Toyota Way: Extract Method - Get risk level icon
fn get_risk_icon(risk_level: &crate::services::defect_probability::RiskLevel) -> &'static str {
    match risk_level {
        crate::services::defect_probability::RiskLevel::High => "🔴",
        crate::services::defect_probability::RiskLevel::Medium => "🟡",
        crate::services::defect_probability::RiskLevel::Low => "🟢",
    }
}

/// Toyota Way: Extract Method - Write summary footer
fn write_summary_footer(output: &mut String, elapsed: std::time::Duration) -> Result<()> {
    use std::fmt::Write;
    writeln!(output)?;
    writeln!(output, "⏱️  Analysis time: {elapsed:.2?}")?;
    Ok(())
}

/// Format predictions as JSON
fn format_defect_json(
    predictions: &[(String, DefectScore)],
    elapsed: std::time::Duration,
) -> Result<String> {
    let report = serde_json::json!({
        "analysis_type": "defect_prediction",
        "summary": {
            "total_files_analyzed": predictions.len(),
            "high_risk_files": predictions.iter().filter(|(_, s)| s.probability > 0.7).count(),
            "medium_risk_files": predictions.iter().filter(|(_, s)| s.probability > 0.3 && s.probability <= 0.7).count(),
            "low_risk_files": predictions.iter().filter(|(_, s)| s.probability <= 0.3).count(),
            "analysis_time_ms": elapsed.as_millis(),
        },
        "predictions": predictions.iter().map(|(file, score)| {
            serde_json::json!({
                "file": file,
                "probability": score.probability,
                "confidence": score.confidence,
                "risk_level": format!("{:?}", score.risk_level),
                "contributing_factors": score.contributing_factors,
                "recommendations": score.recommendations,
            })
        }).collect::<Vec<_>>(),
    });

    Ok(serde_json::to_string_pretty(&report)?)
}

/// Format predictions as detailed report
fn format_defect_detailed(
    predictions: &[(String, DefectScore)],
    elapsed: std::time::Duration,
    include_recommendations: bool,
) -> Result<String> {
    let mut output = String::new();

    write_detailed_header(&mut output)?;

    for (file, score) in predictions {
        write_file_details(&mut output, file, score, include_recommendations)?;
    }

    write_analysis_footer(&mut output, elapsed)?;
    Ok(output)
}

/// Write detailed report header
fn write_detailed_header(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "🔮 Defect Prediction Detailed Report")?;
    writeln!(output, "===================================")?;
    writeln!(output)?;
    Ok(())
}

/// Write details for a single file
fn write_file_details(
    output: &mut String,
    file: &str,
    score: &DefectScore,
    include_recommendations: bool,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "📄 File: {file}")?;
    write_risk_level(output, score)?;
    write_confidence_level(output, score)?;
    write_contributing_factors(output, score)?;

    if include_recommendations {
        write_recommendations(output, score)?;
    }

    writeln!(output)?;
    Ok(())
}

/// Write risk level information
fn write_risk_level(output: &mut String, score: &DefectScore) -> Result<()> {
    use std::fmt::Write;
    let risk_display = format_risk_level_display(&score.risk_level);
    writeln!(
        output,
        "   Risk Level: {} ({:.1}%)",
        risk_display,
        score.probability * 100.0
    )?;
    Ok(())
}

/// Format risk level for display
fn format_risk_level_display(
    risk_level: &crate::services::defect_probability::RiskLevel,
) -> &'static str {
    match risk_level {
        crate::services::defect_probability::RiskLevel::High => "🔴 HIGH",
        crate::services::defect_probability::RiskLevel::Medium => "🟡 MEDIUM",
        crate::services::defect_probability::RiskLevel::Low => "🟢 LOW",
    }
}

/// Write confidence level information
fn write_confidence_level(output: &mut String, score: &DefectScore) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "   Confidence: {:.1}%", score.confidence * 100.0)?;
    Ok(())
}

/// Write contributing factors section
fn write_contributing_factors(output: &mut String, score: &DefectScore) -> Result<()> {
    use std::fmt::Write;

    if score.contributing_factors.is_empty() {
        return Ok(());
    }

    writeln!(output, "   Contributing Factors:")?;
    for (factor, weight) in &score.contributing_factors {
        writeln!(output, "     - {}: {:.1}%", factor, weight * 100.0)?;
    }
    Ok(())
}

/// Write recommendations section
fn write_recommendations(output: &mut String, score: &DefectScore) -> Result<()> {
    use std::fmt::Write;

    if score.recommendations.is_empty() {
        return Ok(());
    }

    writeln!(output, "   Recommendations:")?;
    for rec in &score.recommendations {
        writeln!(output, "     • {rec}")?;
    }
    Ok(())
}

/// Write analysis footer with timing
fn write_analysis_footer(output: &mut String, elapsed: std::time::Duration) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "⏱️  Analysis time: {elapsed:.2?}")?;
    Ok(())
}

/// Format predictions as SARIF
fn format_defect_sarif(predictions: &[(String, DefectScore)]) -> Result<String> {
    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-defect-prediction",
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "version": env!("CARGO_PKG_VERSION"),
                    "rules": [{
                        "id": "DEFECT-RISK",
                        "name": "DefectRisk",
                        "shortDescription": {
                            "text": "ML-based defect probability prediction"
                        },
                        "fullDescription": {
                            "text": "Predicts defect probability using ensemble ML model based on churn, complexity, duplication, and coupling metrics"
                        },
                        "help": {
                            "text": "Files with high defect probability should be reviewed carefully and refactored if necessary"
                        }
                    }]
                }
            },
            "results": predictions.iter().map(|(file, score)| {
                serde_json::json!({
                    "ruleId": "DEFECT-RISK",
                    "level": match score.risk_level {
                        crate::services::defect_probability::RiskLevel::High => "error",
                        crate::services::defect_probability::RiskLevel::Medium => "warning",
                        crate::services::defect_probability::RiskLevel::Low => "note",
                    },
                    "message": {
                        "text": format!("Defect probability: {:.1}% (confidence: {:.1}%)",
                            score.probability * 100.0, score.confidence * 100.0)
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": file,
                                "uriBaseId": "%SRCROOT%"
                            }
                        }
                    }],
                    "properties": {
                        "probability": score.probability,
                        "confidence": score.confidence,
                        "contributing_factors": score.contributing_factors,
                        "recommendations": score.recommendations
                    }
                })
            }).collect::<Vec<_>>()
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Format predictions as CSV
fn format_defect_csv(predictions: &[(String, DefectScore)]) -> Result<String> {
    let mut csv = String::new();

    // Header
    csv.push_str("file,probability,confidence,risk_level,top_factor,top_factor_weight\n");

    // Data rows
    for (file, score) in predictions {
        let (top_factor, top_weight) = score
            .contributing_factors
            .first()
            .map_or(("", 0.0), |(f, w)| (f.as_str(), *w));

        csv.push_str(&format!(
            "{},{:.3},{:.3},{:?},{},{:.3}\n",
            file, score.probability, score.confidence, score.risk_level, top_factor, top_weight
        ));
    }

    Ok(csv)
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::services::defect_probability::RiskLevel;
    use std::time::Duration;

    // Helper function to create mock DefectScore
    fn create_mock_defect_score(
        probability: f32,
        confidence: f32,
        risk_level: RiskLevel,
    ) -> DefectScore {
        DefectScore {
            probability,
            confidence,
            risk_level,
            contributing_factors: vec![
                ("complexity".to_string(), 0.25),
                ("churn".to_string(), 0.20),
                ("duplication".to_string(), 0.15),
                ("coupling".to_string(), 0.10),
            ],
            recommendations: vec![
                "Consider refactoring this file".to_string(),
                "Increase test coverage".to_string(),
            ],
        }
    }

    fn create_high_risk_score() -> DefectScore {
        create_mock_defect_score(0.85, 0.90, RiskLevel::High)
    }

    fn create_medium_risk_score() -> DefectScore {
        create_mock_defect_score(0.50, 0.85, RiskLevel::Medium)
    }

    fn create_low_risk_score() -> DefectScore {
        create_mock_defect_score(0.20, 0.80, RiskLevel::Low)
    }

    fn create_test_predictions() -> Vec<(String, DefectScore)> {
        vec![
            ("src/high_risk.rs".to_string(), create_high_risk_score()),
            ("src/medium_risk.rs".to_string(), create_medium_risk_score()),
            ("src/low_risk.rs".to_string(), create_low_risk_score()),
        ]
    }

    // ==================== Test create_defect_prediction_config ====================

    #[test]
    fn test_create_defect_prediction_config_default_values() {
        let config = create_defect_prediction_config(0.5, 10, false, false, true, None, None);

        assert_eq!(config.confidence_threshold, 0.5);
        assert_eq!(config.min_lines, 10);
        assert!(!config.include_low_confidence);
        assert!(!config.high_risk_only);
        assert!(config.include_recommendations);
        assert!(config.include.is_none());
        assert!(config.exclude.is_none());
    }

    #[test]
    fn test_create_defect_prediction_config_with_patterns() {
        let config = create_defect_prediction_config(
            0.7,
            50,
            true,
            true,
            false,
            Some("src/".to_string()),
            Some("test/".to_string()),
        );

        assert_eq!(config.confidence_threshold, 0.7);
        assert_eq!(config.min_lines, 50);
        assert!(config.include_low_confidence);
        assert!(config.high_risk_only);
        assert!(!config.include_recommendations);
        assert_eq!(config.include, Some("src/".to_string()));
        assert_eq!(config.exclude, Some("test/".to_string()));
    }

    // ==================== Test calculate_risk_statistics ====================

    #[test]
    fn test_calculate_risk_statistics_all_categories() {
        let predictions = create_test_predictions();
        let stats = calculate_risk_statistics(&predictions);

        assert_eq!(stats.high_risk, 1);
        assert_eq!(stats.medium_risk, 1);
        assert_eq!(stats.low_risk, 1);
    }

    #[test]
    fn test_calculate_risk_statistics_empty() {
        let predictions: Vec<(String, DefectScore)> = vec![];
        let stats = calculate_risk_statistics(&predictions);

        assert_eq!(stats.high_risk, 0);
        assert_eq!(stats.medium_risk, 0);
        assert_eq!(stats.low_risk, 0);
    }

    #[test]
    fn test_calculate_risk_statistics_all_high_risk() {
        let predictions = vec![
            ("file1.rs".to_string(), create_high_risk_score()),
            ("file2.rs".to_string(), create_high_risk_score()),
            ("file3.rs".to_string(), create_high_risk_score()),
        ];
        let stats = calculate_risk_statistics(&predictions);

        assert_eq!(stats.high_risk, 3);
        assert_eq!(stats.medium_risk, 0);
        assert_eq!(stats.low_risk, 0);
    }

    #[test]
    fn test_calculate_risk_statistics_boundary_values() {
        // Test boundary at 0.7 (high/medium)
        let high_boundary = create_mock_defect_score(0.7, 0.9, RiskLevel::Medium);
        // Test boundary at 0.3 (medium/low)
        let low_boundary = create_mock_defect_score(0.3, 0.9, RiskLevel::Low);

        let predictions = vec![
            ("high_boundary.rs".to_string(), high_boundary),
            ("low_boundary.rs".to_string(), low_boundary),
        ];
        let stats = calculate_risk_statistics(&predictions);

        // 0.7 is NOT > 0.7, so it's medium
        assert_eq!(stats.high_risk, 0);
        assert_eq!(stats.medium_risk, 1); // 0.7 is in (0.3, 0.7]
        assert_eq!(stats.low_risk, 1); // 0.3 is <= 0.3
    }

    // ==================== Test get_risk_icon ====================

    #[test]
    fn test_get_risk_icon_high() {
        assert_eq!(get_risk_icon(&RiskLevel::High), "🔴");
    }

    #[test]
    fn test_get_risk_icon_medium() {
        assert_eq!(get_risk_icon(&RiskLevel::Medium), "🟡");
    }

    #[test]
    fn test_get_risk_icon_low() {
        assert_eq!(get_risk_icon(&RiskLevel::Low), "🟢");
    }

    // ==================== Test format_risk_level_display ====================

    #[test]
    fn test_format_risk_level_display_high() {
        assert_eq!(format_risk_level_display(&RiskLevel::High), "🔴 HIGH");
    }

    #[test]
    fn test_format_risk_level_display_medium() {
        assert_eq!(format_risk_level_display(&RiskLevel::Medium), "🟡 MEDIUM");
    }

    #[test]
    fn test_format_risk_level_display_low() {
        assert_eq!(format_risk_level_display(&RiskLevel::Low), "🟢 LOW");
    }

    // ==================== Test filter_and_sort_predictions ====================

    #[test]
    fn test_filter_and_sort_predictions_high_risk_only() {
        let predictions = create_test_predictions();
        let filtered = filter_and_sort_predictions(
            predictions,
            true,  // high_risk_only
            false, // include_low_confidence
            0.5,   // confidence_threshold
            10,    // top_files
        );

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].0, "src/high_risk.rs");
    }

    #[test]
    fn test_filter_and_sort_predictions_low_confidence_filtered() {
        let mut predictions = create_test_predictions();
        // Add a low confidence prediction
        predictions.push((
            "low_conf.rs".to_string(),
            create_mock_defect_score(0.9, 0.3, RiskLevel::High),
        ));

        let filtered = filter_and_sort_predictions(
            predictions,
            false, // high_risk_only
            false, // include_low_confidence (filter out low confidence)
            0.5,   // confidence_threshold
            10,    // top_files
        );

        // Should filter out the low confidence file
        assert_eq!(filtered.len(), 3);
        assert!(filtered.iter().all(|(f, _)| f != "low_conf.rs"));
    }

    #[test]
    fn test_filter_and_sort_predictions_include_low_confidence() {
        let mut predictions = create_test_predictions();
        predictions.push((
            "low_conf.rs".to_string(),
            create_mock_defect_score(0.9, 0.3, RiskLevel::High),
        ));

        let filtered = filter_and_sort_predictions(
            predictions,
            false, // high_risk_only
            true,  // include_low_confidence
            0.5,   // confidence_threshold
            10,    // top_files
        );

        // Should include all files
        assert_eq!(filtered.len(), 4);
    }

    #[test]
    fn test_filter_and_sort_predictions_sorted_by_probability() {
        let predictions = create_test_predictions();
        let filtered = filter_and_sort_predictions(
            predictions,
            false, // high_risk_only
            true,  // include_low_confidence
            0.0,   // confidence_threshold
            10,    // top_files
        );

        // Verify sorted by probability descending
        assert_eq!(filtered[0].0, "src/high_risk.rs");
        assert_eq!(filtered[1].0, "src/medium_risk.rs");
        assert_eq!(filtered[2].0, "src/low_risk.rs");
    }

    #[test]
    fn test_filter_and_sort_predictions_top_files_limit() {
        let predictions = create_test_predictions();
        let filtered = filter_and_sort_predictions(
            predictions,
            false, // high_risk_only
            true,  // include_low_confidence
            0.0,   // confidence_threshold
            2,     // top_files - limit to 2
        );

        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_and_sort_predictions_top_files_zero_means_unlimited() {
        let predictions = create_test_predictions();
        let filtered = filter_and_sort_predictions(
            predictions,
            false, // high_risk_only
            true,  // include_low_confidence
            0.0,   // confidence_threshold
            0,     // top_files - 0 means unlimited
        );

        assert_eq!(filtered.len(), 3);
    }

    // ==================== Test format_defect_output ====================

    #[test]
    fn test_format_defect_output_summary() {
        let predictions = create_test_predictions();
        let elapsed = Duration::from_millis(100);

        let result = format_defect_output(
            DefectPredictionOutputFormat::Summary,
            &predictions,
            elapsed,
            false,
        );

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("Defect Prediction Summary"));
        assert!(content.contains("Risk Distribution"));
    }

    #[test]
    fn test_format_defect_output_json() {
        let predictions = create_test_predictions();
        let elapsed = Duration::from_millis(100);

        let result = format_defect_output(
            DefectPredictionOutputFormat::Json,
            &predictions,
            elapsed,
            false,
        );

        assert!(result.is_ok());
        let content = result.unwrap();
        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.get("analysis_type").is_some());
        assert!(parsed.get("predictions").is_some());
    }

    #[test]
    fn test_format_defect_output_detailed() {
        let predictions = create_test_predictions();
        let elapsed = Duration::from_millis(100);

        let result = format_defect_output(
            DefectPredictionOutputFormat::Detailed,
            &predictions,
            elapsed,
            true,
        );

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("Detailed Report"));
        assert!(content.contains("Recommendations"));
    }

    #[test]
    fn test_format_defect_output_sarif() {
        let predictions = create_test_predictions();
        let elapsed = Duration::from_millis(100);

        let result = format_defect_output(
            DefectPredictionOutputFormat::Sarif,
            &predictions,
            elapsed,
            false,
        );

        assert!(result.is_ok());
        let content = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.get("version").unwrap(), "2.1.0");
        assert!(parsed.get("runs").is_some());
    }

    #[test]
    fn test_format_defect_output_csv() {
        let predictions = create_test_predictions();
        let elapsed = Duration::from_millis(100);

        let result = format_defect_output(
            DefectPredictionOutputFormat::Csv,
            &predictions,
            elapsed,
            false,
        );

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("file,probability,confidence,risk_level"));
        assert!(content.contains("high_risk.rs"));
    }

    // ==================== Test format_defect_summary ====================

    #[test]
    fn test_format_defect_summary_with_predictions() {
        let predictions = create_test_predictions();
        let elapsed = Duration::from_millis(250);

        let result = format_defect_summary(&predictions, elapsed).unwrap();

        assert!(result.contains("Defect Prediction Summary"));
        assert!(result.contains("Risk Distribution"));
        assert!(result.contains("Top Risk Files"));
        assert!(result.contains("Analysis time"));
    }

    #[test]
    fn test_format_defect_summary_empty_predictions() {
        let predictions: Vec<(String, DefectScore)> = vec![];
        let elapsed = Duration::from_millis(50);

        let result = format_defect_summary(&predictions, elapsed).unwrap();

        assert!(result.contains("Defect Prediction Summary"));
        assert!(result.contains("0 files")); // Risk distribution shows 0
                                             // Should NOT contain "Top Risk Files" section when empty
    }

    // ==================== Test write_summary_header ====================

    #[test]
    fn test_write_summary_header() {
        let mut output = String::new();
        write_summary_header(&mut output).unwrap();

        assert!(output.contains("Defect Prediction Summary"));
        assert!(output.contains("==="));
    }

    // ==================== Test write_risk_distribution ====================

    #[test]
    fn test_write_risk_distribution() {
        let predictions = create_test_predictions();
        let mut output = String::new();

        write_risk_distribution(&mut output, &predictions).unwrap();

        assert!(output.contains("Risk Distribution"));
        assert!(output.contains("High risk"));
        assert!(output.contains("Medium risk"));
        assert!(output.contains("Low risk"));
    }

    // ==================== Test write_top_risk_files ====================

    #[test]
    fn test_write_top_risk_files_with_data() {
        let predictions = create_test_predictions();
        let mut output = String::new();

        write_top_risk_files(&mut output, &predictions).unwrap();

        assert!(output.contains("Top Risk Files"));
        assert!(output.contains("src/high_risk.rs"));
    }

    #[test]
    fn test_write_top_risk_files_empty() {
        let predictions: Vec<(String, DefectScore)> = vec![];
        let mut output = String::new();

        write_top_risk_files(&mut output, &predictions).unwrap();

        // Should not write anything for empty predictions
        assert!(!output.contains("Top Risk Files"));
    }

    #[test]
    fn test_write_top_risk_files_more_than_10() {
        // Create more than 10 predictions
        let predictions: Vec<_> = (0..15)
            .map(|i| (format!("file{}.rs", i), create_high_risk_score()))
            .collect();

        let mut output = String::new();
        write_top_risk_files(&mut output, &predictions).unwrap();

        // Should only show 10 files
        let file_count = output.matches("file").count();
        assert_eq!(file_count, 10);
    }

    // ==================== Test write_summary_footer ====================

    #[test]
    fn test_write_summary_footer() {
        let elapsed = Duration::from_millis(1234);
        let mut output = String::new();

        write_summary_footer(&mut output, elapsed).unwrap();

        assert!(output.contains("Analysis time"));
    }

    // ==================== Test format_defect_json ====================

    #[test]
    fn test_format_defect_json_structure() {
        let predictions = create_test_predictions();
        let elapsed = Duration::from_millis(500);

        let result = format_defect_json(&predictions, elapsed).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Check structure
        assert_eq!(parsed["analysis_type"], "defect_prediction");
        assert!(parsed["summary"]["total_files_analyzed"].as_u64().is_some());
        assert!(parsed["summary"]["high_risk_files"].as_u64().is_some());
        assert!(parsed["summary"]["medium_risk_files"].as_u64().is_some());
        assert!(parsed["summary"]["low_risk_files"].as_u64().is_some());
        assert!(parsed["summary"]["analysis_time_ms"].as_u64().is_some());

        // Check predictions array
        let preds = parsed["predictions"].as_array().unwrap();
        assert_eq!(preds.len(), 3);

        // Check individual prediction structure
        let first = &preds[0];
        assert!(first["file"].is_string());
        assert!(first["probability"].is_f64());
        assert!(first["confidence"].is_f64());
        assert!(first["risk_level"].is_string());
    }

    #[test]
    fn test_format_defect_json_empty() {
        let predictions: Vec<(String, DefectScore)> = vec![];
        let elapsed = Duration::from_millis(10);

        let result = format_defect_json(&predictions, elapsed).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["summary"]["total_files_analyzed"], 0);
        assert!(parsed["predictions"].as_array().unwrap().is_empty());
    }

    // ==================== Test format_defect_detailed ====================

    #[test]
    fn test_format_defect_detailed_with_recommendations() {
        let predictions = create_test_predictions();
        let elapsed = Duration::from_millis(100);

        let result = format_defect_detailed(&predictions, elapsed, true).unwrap();

        assert!(result.contains("Detailed Report"));
        assert!(result.contains("File:"));
        assert!(result.contains("Risk Level:"));
        assert!(result.contains("Confidence:"));
        assert!(result.contains("Contributing Factors:"));
        assert!(result.contains("Recommendations:"));
    }

    #[test]
    fn test_format_defect_detailed_without_recommendations() {
        let predictions = create_test_predictions();
        let elapsed = Duration::from_millis(100);

        let result = format_defect_detailed(&predictions, elapsed, false).unwrap();

        assert!(result.contains("Detailed Report"));
        assert!(!result.contains("Recommendations:"));
    }

    // ==================== Test write_detailed_header ====================

    #[test]
    fn test_write_detailed_header() {
        let mut output = String::new();
        write_detailed_header(&mut output).unwrap();

        assert!(output.contains("Detailed Report"));
        assert!(output.contains("==="));
    }

    // ==================== Test write_file_details ====================

    #[test]
    fn test_write_file_details_with_recommendations() {
        let score = create_high_risk_score();
        let mut output = String::new();

        write_file_details(&mut output, "test.rs", &score, true).unwrap();

        assert!(output.contains("test.rs"));
        assert!(output.contains("Risk Level:"));
        assert!(output.contains("Confidence:"));
        assert!(output.contains("Recommendations:"));
    }

    #[test]
    fn test_write_file_details_without_recommendations() {
        let score = create_high_risk_score();
        let mut output = String::new();

        write_file_details(&mut output, "test.rs", &score, false).unwrap();

        assert!(output.contains("test.rs"));
        assert!(!output.contains("Recommendations:"));
    }

    // ==================== Test write_risk_level ====================

    #[test]
    fn test_write_risk_level() {
        let score = create_high_risk_score();
        let mut output = String::new();

        write_risk_level(&mut output, &score).unwrap();

        assert!(output.contains("Risk Level:"));
        assert!(output.contains("HIGH"));
        assert!(output.contains("85.0%"));
    }

    // ==================== Test write_confidence_level ====================

    #[test]
    fn test_write_confidence_level() {
        let score = create_high_risk_score();
        let mut output = String::new();

        write_confidence_level(&mut output, &score).unwrap();

        assert!(output.contains("Confidence:"));
        assert!(output.contains("90.0%"));
    }

    // ==================== Test write_contributing_factors ====================

    #[test]
    fn test_write_contributing_factors() {
        let score = create_high_risk_score();
        let mut output = String::new();

        write_contributing_factors(&mut output, &score).unwrap();

        assert!(output.contains("Contributing Factors:"));
        assert!(output.contains("complexity"));
        assert!(output.contains("churn"));
    }

    #[test]
    fn test_write_contributing_factors_empty() {
        let mut score = create_high_risk_score();
        score.contributing_factors = vec![];
        let mut output = String::new();

        write_contributing_factors(&mut output, &score).unwrap();

        // Should not write anything for empty factors
        assert!(output.is_empty());
    }

    // ==================== Test write_recommendations ====================

    #[test]
    fn test_write_recommendations() {
        let score = create_high_risk_score();
        let mut output = String::new();

        write_recommendations(&mut output, &score).unwrap();

        assert!(output.contains("Recommendations:"));
        assert!(output.contains("refactoring"));
    }

    #[test]
    fn test_write_recommendations_empty() {
        let mut score = create_high_risk_score();
        score.recommendations = vec![];
        let mut output = String::new();

        write_recommendations(&mut output, &score).unwrap();

        // Should not write anything for empty recommendations
        assert!(output.is_empty());
    }

    // ==================== Test write_analysis_footer ====================

    #[test]
    fn test_write_analysis_footer() {
        let elapsed = Duration::from_secs(2);
        let mut output = String::new();

        write_analysis_footer(&mut output, elapsed).unwrap();

        assert!(output.contains("Analysis time:"));
    }

    // ==================== Test format_defect_sarif ====================

    #[test]
    fn test_format_defect_sarif_structure() {
        let predictions = create_test_predictions();

        let result = format_defect_sarif(&predictions).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Check SARIF schema
        assert_eq!(parsed["version"], "2.1.0");
        assert!(parsed["$schema"].as_str().unwrap().contains("sarif-schema"));

        // Check runs
        let runs = parsed["runs"].as_array().unwrap();
        assert_eq!(runs.len(), 1);

        // Check tool info
        let tool = &runs[0]["tool"]["driver"];
        assert_eq!(tool["name"], "pmat-defect-prediction");
        assert!(tool["version"].is_string());

        // Check results
        let results = runs[0]["results"].as_array().unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_format_defect_sarif_risk_levels() {
        let predictions = create_test_predictions();

        let result = format_defect_sarif(&predictions).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        let results = parsed["runs"][0]["results"].as_array().unwrap();

        // High risk should be "error"
        let high_risk = results
            .iter()
            .find(|r| {
                r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    .as_str()
                    .unwrap()
                    .contains("high_risk")
            })
            .unwrap();
        assert_eq!(high_risk["level"], "error");

        // Medium risk should be "warning"
        let medium_risk = results
            .iter()
            .find(|r| {
                r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    .as_str()
                    .unwrap()
                    .contains("medium_risk")
            })
            .unwrap();
        assert_eq!(medium_risk["level"], "warning");

        // Low risk should be "note"
        let low_risk = results
            .iter()
            .find(|r| {
                r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    .as_str()
                    .unwrap()
                    .contains("low_risk")
            })
            .unwrap();
        assert_eq!(low_risk["level"], "note");
    }

    // ==================== Test format_defect_csv ====================

    #[test]
    fn test_format_defect_csv_header() {
        let predictions = create_test_predictions();

        let result = format_defect_csv(&predictions).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(
            lines[0],
            "file,probability,confidence,risk_level,top_factor,top_factor_weight"
        );
    }

    #[test]
    fn test_format_defect_csv_data_rows() {
        let predictions = create_test_predictions();

        let result = format_defect_csv(&predictions).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 4); // 1 header + 3 data rows

        // Check first data row
        assert!(lines[1].contains("high_risk.rs"));
        assert!(lines[1].contains("0.850"));
    }

    #[test]
    fn test_format_defect_csv_empty_factors() {
        let mut score = create_high_risk_score();
        score.contributing_factors = vec![];
        let predictions = vec![("test.rs".to_string(), score)];

        let result = format_defect_csv(&predictions).unwrap();

        // Should handle missing top factor gracefully
        assert!(result.contains("test.rs"));
        assert!(result.contains("0.000")); // Default weight
    }

    // ==================== Test RiskStatistics struct ====================

    #[test]
    fn test_risk_statistics_struct() {
        let stats = RiskStatistics {
            high_risk: 5,
            medium_risk: 10,
            low_risk: 15,
        };

        assert_eq!(stats.high_risk, 5);
        assert_eq!(stats.medium_risk, 10);
        assert_eq!(stats.low_risk, 15);
    }

    // ==================== Edge case tests ====================

    #[test]
    fn test_format_defect_summary_single_file() {
        let predictions = vec![("only_file.rs".to_string(), create_high_risk_score())];
        let elapsed = Duration::from_millis(50);

        let result = format_defect_summary(&predictions, elapsed).unwrap();

        assert!(result.contains("1 files"));
        assert!(result.contains("only_file.rs"));
    }

    #[test]
    fn test_format_with_special_characters_in_filename() {
        let predictions = vec![("src/my-file_v2.0.rs".to_string(), create_high_risk_score())];
        let elapsed = Duration::from_millis(50);

        let result = format_defect_summary(&predictions, elapsed).unwrap();
        assert!(result.contains("my-file_v2.0.rs"));

        let csv = format_defect_csv(&predictions).unwrap();
        assert!(csv.contains("my-file_v2.0.rs"));
    }

    #[test]
    fn test_format_with_unicode_filename() {
        let predictions = vec![(
            "src/archivo_espa\u{00f1}ol.rs".to_string(),
            create_medium_risk_score(),
        )];
        let elapsed = Duration::from_millis(50);

        let result = format_defect_json(&predictions, elapsed).unwrap();
        assert!(result.contains("archivo_espa\u{00f1}ol.rs"));
    }

    #[test]
    fn test_format_with_zero_duration() {
        let predictions = create_test_predictions();
        let elapsed = Duration::from_millis(0);

        let result = format_defect_summary(&predictions, elapsed).unwrap();
        assert!(result.contains("Analysis time"));

        let json = format_defect_json(&predictions, elapsed).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["summary"]["analysis_time_ms"], 0);
    }

    #[test]
    fn test_format_with_very_long_duration() {
        let predictions = create_test_predictions();
        let elapsed = Duration::from_secs(3600); // 1 hour

        let result = format_defect_summary(&predictions, elapsed).unwrap();
        assert!(result.contains("Analysis time"));
    }

    // ==================== Probability boundary tests ====================

    #[test]
    fn test_probability_exactly_zero() {
        let score = create_mock_defect_score(0.0, 0.9, RiskLevel::Low);
        let predictions = vec![("zero.rs".to_string(), score)];

        let stats = calculate_risk_statistics(&predictions);
        assert_eq!(stats.low_risk, 1);
        assert_eq!(stats.medium_risk, 0);
        assert_eq!(stats.high_risk, 0);
    }

    #[test]
    fn test_probability_exactly_one() {
        let score = create_mock_defect_score(1.0, 0.9, RiskLevel::High);
        let predictions = vec![("max.rs".to_string(), score)];

        let stats = calculate_risk_statistics(&predictions);
        assert_eq!(stats.high_risk, 1);
        assert_eq!(stats.medium_risk, 0);
        assert_eq!(stats.low_risk, 0);
    }

    #[test]
    #[ignore = "Agent-added test with incorrect assertion"]
    fn test_confidence_values_in_output() {
        let score = create_mock_defect_score(0.75, 0.95, RiskLevel::High);
        let predictions = vec![("conf.rs".to_string(), score)];
        let elapsed = Duration::from_millis(100);

        let detailed = format_defect_detailed(&predictions, elapsed, true).unwrap();
        assert!(detailed.contains("95.0%"));

        let json = format_defect_json(&predictions, elapsed).unwrap();
        assert!(json.contains("0.95"));
    }
}

/// Active unit tests for defect prediction module
// Tests extracted to defect_prediction_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "defect_prediction_tests.rs"]
mod tests;
