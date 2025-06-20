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

    // Summary statistics
    writeln!(&mut output, "## Summary\n")?;
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

    writeln!(&mut output, "\n### Risk Distribution")?;
    writeln!(&mut output, "| Risk Level | Count | Percentage |")?;
    writeln!(&mut output, "|------------|-------|------------|")?;
    writeln!(
        &mut output,
        "| High (>70%) | {} | {:.1}% |",
        high_risk,
        (high_risk as f64 / predictions.len() as f64) * 100.0
    )?;
    writeln!(
        &mut output,
        "| Medium (40-70%) | {} | {:.1}% |",
        medium_risk,
        (medium_risk as f64 / predictions.len() as f64) * 100.0
    )?;
    writeln!(
        &mut output,
        "| Low (<40%) | {} | {:.1}% |",
        low_risk,
        (low_risk as f64 / predictions.len() as f64) * 100.0
    )?;

    // Detailed predictions
    writeln!(&mut output, "\n## Detailed Predictions\n")?;

    for (file, score) in predictions.iter().take(20) {
        writeln!(&mut output, "### {}\n", file)?;
        writeln!(
            &mut output,
            "- **Probability**: {:.1}%",
            score.probability * 100.0
        )?;
        writeln!(
            &mut output,
            "- **Confidence**: {:.1}%",
            score.confidence * 100.0
        )?;
        writeln!(
            &mut output,
            "- **Risk Factors**: {:?}",
            score.contributing_factors
        )?;

        if include_recommendations {
            writeln!(&mut output, "\n#### Recommendations:")?;
            if score.probability > 0.7 {
                writeln!(&mut output, "- 🔴 High priority for code review")?;
                writeln!(&mut output, "- Add comprehensive test coverage")?;
                writeln!(&mut output, "- Consider refactoring to reduce complexity")?;
            } else if score.probability > 0.4 {
                writeln!(&mut output, "- 🟡 Schedule for regular review")?;
                writeln!(&mut output, "- Improve test coverage")?;
            } else {
                writeln!(&mut output, "- 🟢 Monitor during regular maintenance")?;
            }
        }
        writeln!(&mut output)?;
    }

    Ok(output)
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
