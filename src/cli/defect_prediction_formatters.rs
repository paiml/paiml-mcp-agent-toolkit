// Output formatting functions for defect prediction
// Included from defect_prediction_helpers.rs - shares parent module scope

/// Format summary output
#[must_use]
pub fn format_summary_output(
    file_metrics_len: usize,
    filtered_predictions: &[(String, DefectScore)],
    risk_dist: &RiskDistribution,
    perf: bool,
    analysis_time: std::time::Duration,
) -> String {
    let mut output = String::new();

    output.push_str("Defect Prediction Analysis Summary\n");
    output.push_str("=================================\n");
    output.push_str(&format!("Files analyzed: {file_metrics_len}\n"));
    output.push_str(&format!(
        "Predictions generated: {}\n",
        filtered_predictions.len()
    ));

    let total = filtered_predictions.len() as f32;
    output.push_str(&format!(
        "High risk files: {} ({:.1}%)\n",
        risk_dist.high_risk_count,
        100.0 * risk_dist.high_risk_count as f32 / total
    ));
    output.push_str(&format!(
        "Medium risk files: {} ({:.1}%)\n",
        risk_dist.medium_risk_count,
        100.0 * risk_dist.medium_risk_count as f32 / total
    ));
    output.push_str(&format!(
        "Low risk files: {} ({:.1}%)\n",
        risk_dist.low_risk_count,
        100.0 * risk_dist.low_risk_count as f32 / total
    ));

    if perf {
        output.push_str("\nPerformance Metrics:\n");
        output.push_str(&format!(
            "Analysis time: {:.2}s\n",
            analysis_time.as_secs_f64()
        ));
        output.push_str(&format!(
            "Files/second: {:.1}\n",
            file_metrics_len as f64 / analysis_time.as_secs_f64()
        ));
    }

    if !filtered_predictions.is_empty() {
        output.push_str("\nTop 10 High-Risk Files:\n");
        for (file_path, score) in filtered_predictions.iter().take(10) {
            output.push_str(&format!(
                "  {} - {:.1}% risk ({:?})\n",
                std::path::Path::new(file_path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                score.probability * 100.0,
                score.confidence
            ));
        }
    }

    output
}

/// Generate recommendations for high-risk files
#[allow(dead_code)]
#[must_use]
pub fn generate_recommendations(predictions: &[(String, DefectScore)]) -> Vec<String> {
    let mut recommendations = Vec::new();

    for (file_path, score) in predictions.iter().take(5) {
        let file_name = std::path::Path::new(file_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        let mut recs = vec![format!(
            "**{}** ({:.1}% risk):",
            file_name,
            score.probability * 100.0
        )];

        // Check contributing factors
        for (factor, value) in &score.contributing_factors {
            match factor.as_str() {
                "complexity" if *value > 0.7 => {
                    recs.push(
                        "  - High complexity: Consider refactoring into smaller functions"
                            .to_string(),
                    );
                }
                "churn" if *value > 0.7 => {
                    recs.push(
                        "  - High churn: Increase test coverage and code reviews".to_string(),
                    );
                }
                "coupling" if *value > 0.7 => {
                    recs.push(
                        "  - High coupling: Reduce dependencies and improve modularity".to_string(),
                    );
                }
                "duplication" if *value > 0.3 => {
                    recs.push("  - Code duplication: Extract common functionality".to_string());
                }
                _ => {}
            }
        }

        recommendations.extend(recs);
        recommendations.push(String::new());
    }

    recommendations
}

/// Format detailed output
#[must_use]
pub fn format_detailed_output(
    filtered_predictions: &[(String, DefectScore)],
    include_recommendations: bool,
) -> String {
    let mut output = String::new();

    output.push_str("Defect Prediction Analysis Report\n");
    output.push_str("================================\n");

    for (file_path, score) in filtered_predictions {
        output.push_str(&format!("\n{file_path}\n"));
        output.push_str(&format!("  Risk Level: {:?}\n", score.risk_level));
        output.push_str(&format!(
            "  Probability: {:.1}%\n",
            score.probability * 100.0
        ));
        output.push_str(&format!("  Confidence: {:.1}%\n", score.confidence * 100.0));

        output.push_str("  Contributing Factors:\n");
        for (factor, contribution) in &score.contributing_factors {
            output.push_str(&format!("    {factor}: {contribution:.3}\n"));
        }

        if include_recommendations && !score.recommendations.is_empty() {
            output.push_str("  Recommendations:\n");
            for rec in &score.recommendations {
                output.push_str(&format!("    - {rec}\n"));
            }
        }
    }

    output
}

/// Format JSON output
pub fn format_json_output(
    file_metrics_len: usize,
    filtered_predictions: &[(String, DefectScore)],
    include_recommendations: bool,
    perf: bool,
    analysis_time: std::time::Duration,
) -> Result<String> {
    let risk_dist = calculate_risk_distribution(filtered_predictions);

    let mut result = serde_json::json!({
        "summary": {
            "total_files": file_metrics_len,
            "predictions": filtered_predictions.len(),
            "high_risk": risk_dist.high_risk_count,
            "medium_risk": risk_dist.medium_risk_count,
            "low_risk": risk_dist.low_risk_count
        },
        "predictions": filtered_predictions.iter().map(|(path, score)| {
            serde_json::json!({
                "file": path,
                "probability": score.probability,
                "confidence": score.confidence,
                "risk_level": score.risk_level,
                "contributing_factors": score.contributing_factors,
                "recommendations": if include_recommendations { Some(&score.recommendations) } else { None }
            })
        }).collect::<Vec<_>>()
    });

    if perf {
        result["performance"] = serde_json::json!({
            "analysis_time_ms": analysis_time.as_millis(),
            "files_per_second": file_metrics_len as f64 / analysis_time.as_secs_f64()
        });
    }

    serde_json::to_string_pretty(&result).map_err(Into::into)
}

/// Format markdown output
#[allow(dead_code)]
#[must_use]
pub fn format_markdown_output(
    filtered_predictions: &[(String, DefectScore)],
    include_recommendations: bool,
) -> String {
    let mut output = String::new();

    output.push_str("# Defect Prediction Analysis\n\n");

    let risk_dist = calculate_risk_distribution(filtered_predictions);
    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- **Total Predictions**: {}\n",
        filtered_predictions.len()
    ));
    output.push_str(&format!(
        "- **High Risk**: {} files\n",
        risk_dist.high_risk_count
    ));
    output.push_str(&format!(
        "- **Medium Risk**: {} files\n",
        risk_dist.medium_risk_count
    ));
    output.push_str(&format!(
        "- **Low Risk**: {} files\n\n",
        risk_dist.low_risk_count
    ));

    output.push_str("## High Risk Files\n\n");
    output.push_str("| File | Risk | Confidence | Main Factors |\n");
    output.push_str("|------|------|------------|-------------|\n");

    for (file_path, score) in filtered_predictions
        .iter()
        .filter(|(_, s)| s.probability >= 0.7)
        .take(20)
    {
        let file_name = std::path::Path::new(file_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        let main_factors: Vec<String> = score
            .contributing_factors
            .iter()
            .filter(|(_, v)| *v > 0.2)
            .map(|(k, v)| format!("{k}: {v:.2}"))
            .collect();

        output.push_str(&format!(
            "| {} | {:.1}% | {:.1}% | {} |\n",
            file_name,
            score.probability * 100.0,
            score.confidence * 100.0,
            main_factors.join(", ")
        ));
    }

    if include_recommendations {
        output.push_str("\n## Recommendations\n\n");
        let recommendations = generate_recommendations(filtered_predictions);
        for rec in recommendations {
            output.push_str(&format!("{rec}\n"));
        }
    }

    output
}

/// Format CSV output
#[must_use]
pub fn format_csv_output(filtered_predictions: &[(String, DefectScore)]) -> String {
    let mut output = String::new();

    output.push_str("file,probability,confidence,risk_level,churn_factor,complexity_factor,duplication_factor,coupling_factor\n");

    for (file_path, score) in filtered_predictions {
        let factors = &score.contributing_factors;
        output.push_str(&format!(
            "{},{:.3},{:.3},{:?},{:.3},{:.3},{:.3},{:.3}\n",
            file_path,
            score.probability,
            score.confidence,
            score.risk_level,
            factors
                .iter()
                .find(|(k, _)| k == "churn")
                .map_or(0.0, |(_, v)| *v),
            factors
                .iter()
                .find(|(k, _)| k == "complexity")
                .map_or(0.0, |(_, v)| *v),
            factors
                .iter()
                .find(|(k, _)| k == "duplication")
                .map_or(0.0, |(_, v)| *v),
            factors
                .iter()
                .find(|(k, _)| k == "coupling")
                .map_or(0.0, |(_, v)| *v)
        ));
    }

    output
}

/// Format SARIF output
pub fn format_sarif_output(filtered_predictions: &[(String, DefectScore)]) -> Result<String> {
    let sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                }
            },
            "results": filtered_predictions.iter().map(|(file_path, score)| {
                let level = match score.probability {
                    p if p >= 0.7 => "error",
                    p if p >= 0.3 => "warning",
                    _ => "note"
                };
                serde_json::json!({
                    "ruleId": "defect-prediction",
                    "level": level,
                    "message": {
                        "text": format!("High defect probability: {:.1}% (confidence: {:.1}%)",
                            score.probability * 100.0, score.confidence * 100.0)
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": file_path
                            }
                        }
                    }],
                    "properties": {
                        "defect_probability": score.probability,
                        "confidence": score.confidence,
                        "risk_level": format!("{:?}", score.risk_level)
                    }
                })
            }).collect::<Vec<_>>()
        }]
    });

    serde_json::to_string_pretty(&sarif).map_err(Into::into)
}
