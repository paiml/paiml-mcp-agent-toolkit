//! Helper functions for defect prediction analysis

use crate::cli::defect_prediction_helpers::{
    calculate_simple_churn_score, calculate_simple_complexity, DefectPredictionConfig,
};
use crate::services::defect_probability::{DefectProbabilityCalculator, DefectScore, FileMetrics};
use anyhow::Result;
use std::fmt::Write;
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
    predictions.sort_by(|a, b| b.1.probability.partial_cmp(&a.1.probability).unwrap());

    Ok(predictions)
}

/// Format defect predictions as JSON
/// Formats defect predictions as JSON
///
/// # Examples
///
/// ```rust
/// use pmat::cli::defect_helpers::format_defect_json;
/// use pmat::services::defect_probability::{DefectScore, RiskLevel};
///
/// let predictions = vec![
///     ("src/main.rs".to_string(), DefectScore {
///         probability: 0.8,
///         confidence: 0.9,
///         contributing_factors: vec![("complexity".to_string(), 0.5)],
///         risk_level: RiskLevel::High,
///         recommendations: vec!["Reduce complexity".to_string()],
///     })
/// ];
///
/// let json = format_defect_json(&predictions).unwrap();
/// assert!(json.contains("defect_predictions"));
/// assert!(json.contains("src/main.rs"));
/// ```
pub fn format_defect_json(predictions: &[(String, DefectScore)]) -> Result<String> {
    let json_data = serde_json::json!({
        "defect_predictions": predictions.iter().map(|(file, score)| {
            serde_json::json!({
                "file": file,
                "probability": score.probability,
                "confidence": score.confidence,
                "risk_factors": score.contributing_factors,
            })
        }).collect::<Vec<_>>(),
        "summary": {
            "total_files": predictions.len(),
            "high_risk_files": predictions.iter().filter(|(_, s)| s.probability > 0.7).count(),
            "medium_risk_files": predictions.iter().filter(|(_, s)| s.probability > 0.4 && s.probability <= 0.7).count(),
            "low_risk_files": predictions.iter().filter(|(_, s)| s.probability <= 0.4).count(),
        }
    });

    serde_json::to_string_pretty(&json_data).map_err(Into::into)
}

/// Format defect predictions as summary
/// Formats defect predictions as a summary
///
/// # Examples
///
/// ```rust
/// use pmat::cli::defect_helpers::format_defect_summary;
/// use pmat::services::defect_probability::{DefectScore, RiskLevel};
///
/// let predictions = vec![
///     ("src/main.rs".to_string(), DefectScore {
///         probability: 0.8,
///         confidence: 0.9,
///         contributing_factors: vec![],
///         risk_level: RiskLevel::High,
///         recommendations: vec![],
///     })
/// ];
///
/// let summary = format_defect_summary(&predictions).unwrap();
/// assert!(summary.contains("Defect Prediction Summary"));
/// assert!(summary.contains("**Total files analyzed**: 1"));
/// ```
pub fn format_defect_summary(predictions: &[(String, DefectScore)]) -> Result<String> {
    let mut output = String::new();

    writeln!(&mut output, "# Defect Prediction Summary\n")?;
    writeln!(
        &mut output,
        "**Total files analyzed**: {}",
        predictions.len()
    )?;

    let high_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability > 0.7)
        .count();
    let medium_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability > 0.4 && s.probability <= 0.7)
        .count();
    let low_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability <= 0.4)
        .count();

    writeln!(&mut output, "\n## Risk Distribution:")?;
    writeln!(&mut output, "- 🔴 High Risk (>70%): {} files", high_risk)?;
    writeln!(
        &mut output,
        "- 🟡 Medium Risk (40-70%): {} files",
        medium_risk
    )?;
    writeln!(&mut output, "- 🟢 Low Risk (<40%): {} files", low_risk)?;

    if !predictions.is_empty() {
        writeln!(&mut output, "\n## Top 10 High-Risk Files:")?;
        for (i, (file, score)) in predictions.iter().take(10).enumerate() {
            writeln!(
                &mut output,
                "{}. {} - {:.1}% probability",
                i + 1,
                file,
                score.probability * 100.0
            )?;
        }
    }

    Ok(output)
}

/// Format defect predictions as markdown
pub fn format_defect_markdown(
    predictions: &[(String, DefectScore)],
    include_recommendations: bool,
) -> Result<String> {
    let mut output = String::new();

    writeln!(&mut output, "# Defect Prediction Report\n")?;

    write_summary_section(&mut output, predictions)?;
    write_risk_distribution_table(&mut output, predictions)?;
    write_detailed_predictions(&mut output, predictions, include_recommendations)?;

    Ok(output)
}

/// Write summary section (cognitive complexity ≤3)
fn write_summary_section(output: &mut String, predictions: &[(String, DefectScore)]) -> Result<()> {
    writeln!(output, "## Summary\n")?;
    writeln!(output, "**Total files analyzed**: {}", predictions.len())?;
    Ok(())
}

/// Write risk distribution table (cognitive complexity ≤8)
fn write_risk_distribution_table(
    output: &mut String,
    predictions: &[(String, DefectScore)],
) -> Result<()> {
    let (high_risk, medium_risk, low_risk) = calculate_risk_counts(predictions);
    let total = predictions.len() as f64;

    writeln!(output, "\n### Risk Distribution")?;
    writeln!(output, "| Risk Level | Count | Percentage |")?;
    writeln!(output, "|------------|-------|------------|")?;

    write_risk_row(output, "High (>70%)", high_risk, total)?;
    write_risk_row(output, "Medium (40-70%)", medium_risk, total)?;
    write_risk_row(output, "Low (<40%)", low_risk, total)?;

    Ok(())
}

/// Calculate risk counts (cognitive complexity ≤6)
fn calculate_risk_counts(predictions: &[(String, DefectScore)]) -> (usize, usize, usize) {
    let high_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability > 0.7)
        .count();

    let medium_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability > 0.4 && s.probability <= 0.7)
        .count();

    let low_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability <= 0.4)
        .count();

    (high_risk, medium_risk, low_risk)
}

/// Write a single risk row (cognitive complexity ≤3)
fn write_risk_row(output: &mut String, label: &str, count: usize, total: f64) -> Result<()> {
    writeln!(
        output,
        "| {} | {} | {:.1}% |",
        label,
        count,
        (count as f64 / total) * 100.0
    )?;
    Ok(())
}

/// Write detailed predictions section (cognitive complexity ≤7)
fn write_detailed_predictions(
    output: &mut String,
    predictions: &[(String, DefectScore)],
    include_recommendations: bool,
) -> Result<()> {
    writeln!(output, "\n## Detailed Predictions\n")?;

    for (file, score) in predictions.iter().take(20) {
        write_single_prediction(output, file, score, include_recommendations)?;
    }

    Ok(())
}

/// Write a single prediction (cognitive complexity ≤8)
fn write_single_prediction(
    output: &mut String,
    file: &str,
    score: &DefectScore,
    include_recommendations: bool,
) -> Result<()> {
    writeln!(output, "### {}\n", file)?;

    write_prediction_metrics(output, score)?;

    if include_recommendations {
        write_recommendations(output, score.probability as f64)?;
    }

    writeln!(output)?;
    Ok(())
}

/// Write prediction metrics (cognitive complexity ≤4)
fn write_prediction_metrics(output: &mut String, score: &DefectScore) -> Result<()> {
    writeln!(
        output,
        "- **Probability**: {:.1}%",
        score.probability as f64 * 100.0
    )?;
    writeln!(
        output,
        "- **Confidence**: {:.1}%",
        score.confidence as f64 * 100.0
    )?;
    writeln!(
        output,
        "- **Risk Factors**: {:?}",
        score.contributing_factors
    )?;
    Ok(())
}

/// Write recommendations based on probability (cognitive complexity ≤7)
fn write_recommendations(output: &mut String, probability: f64) -> Result<()> {
    writeln!(output, "\n#### Recommendations:")?;

    if probability > 0.7 {
        writeln!(output, "- 🔴 High priority for code review")?;
        writeln!(output, "- Add comprehensive test coverage")?;
        writeln!(output, "- Consider refactoring to reduce complexity")?;
    } else if probability > 0.4 {
        writeln!(output, "- 🟡 Schedule for regular review")?;
        writeln!(output, "- Improve test coverage")?;
    } else {
        writeln!(output, "- 🟢 Monitor during regular maintenance")?;
    }

    Ok(())
}

/// Format defect predictions as SARIF
pub fn format_defect_sarif(
    predictions: &[(String, DefectScore)],
    _project_path: &Path,
) -> Result<String> {
    let mut results = Vec::new();

    for (file, score) in predictions {
        let level = if score.probability > 0.7 {
            "error"
        } else if score.probability > 0.4 {
            "warning"
        } else {
            "note"
        };

        let rule_id = if score.probability > 0.7 {
            "high-defect-probability"
        } else if score.probability > 0.4 {
            "medium-defect-probability"
        } else {
            "low-defect-probability"
        };

        results.push(serde_json::json!({
            "ruleId": rule_id,
            "level": level,
            "message": {
                "text": format!(
                    "File has {:.1}% defect probability with {:.1}% confidence. Risk factors: {:?}",
                    score.probability * 100.0,
                    score.confidence * 100.0,
                    score.contributing_factors
                )
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": file
                    }
                }
            }]
        }));
    }

    let sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-defect-predictor",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": generate_defect_rules(),
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&sarif).map_err(Into::into)
}

/// Generate SARIF rules for defect prediction
fn generate_defect_rules() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "high-defect-probability",
            "name": "High Defect Probability",
            "shortDescription": {
                "text": "File has high probability of containing defects"
            },
            "fullDescription": {
                "text": "Files with >70% defect probability require immediate review"
            },
            "defaultConfiguration": {
                "level": "error"
            }
        }),
        serde_json::json!({
            "id": "medium-defect-probability",
            "name": "Medium Defect Probability",
            "shortDescription": {
                "text": "File has medium probability of containing defects"
            },
            "fullDescription": {
                "text": "Files with 40-70% defect probability should be reviewed"
            },
            "defaultConfiguration": {
                "level": "warning"
            }
        }),
        serde_json::json!({
            "id": "low-defect-probability",
            "name": "Low Defect Probability",
            "shortDescription": {
                "text": "File has low probability of containing defects"
            },
            "fullDescription": {
                "text": "Files with <40% defect probability are lower risk"
            },
            "defaultConfiguration": {
                "level": "note"
            }
        }),
    ]
}
