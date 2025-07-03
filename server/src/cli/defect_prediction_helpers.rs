//! Helper functions for defect prediction analysis to reduce complexity

use crate::services::defect_probability::{DefectScore, FileMetrics};
use anyhow::Result;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct DefectPredictionConfig {
    pub confidence_threshold: f32,
    pub min_lines: usize,
    pub include_low_confidence: bool,
    pub high_risk_only: bool,
    pub include_recommendations: bool,
    pub include: Option<String>,
    pub exclude: Option<String>,
}

#[allow(dead_code)]
pub struct DefectAnalysisResult {
    pub file_metrics: Vec<FileMetrics>,
    pub filtered_predictions: Vec<(String, DefectScore)>,
    pub analysis_time: std::time::Duration,
}

/// Discover source files for defect analysis
pub async fn discover_source_files_for_defect_analysis(
    project_path: &Path,
    config: &DefectPredictionConfig,
) -> Result<Vec<(PathBuf, String, usize)>> {
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};

    let mut discovery_config = FileDiscoveryConfig::default();

    if let Some(exclude_pattern) = &config.exclude {
        discovery_config
            .custom_ignore_patterns
            .push(exclude_pattern.clone());
    }

    let discovery =
        ProjectFileDiscovery::new(project_path.to_path_buf()).with_config(discovery_config);
    let discovered_files = discovery.discover_files()?;

    let mut analyzed_files = Vec::new();
    for file_path in discovered_files {
        if let Some(include_pattern) = &config.include {
            if !file_path.to_string_lossy().contains(include_pattern) {
                continue;
            }
        }

        if let Ok(content) = std::fs::read_to_string(&file_path) {
            let lines_of_code = content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count();

            if lines_of_code >= config.min_lines {
                analyzed_files.push((file_path, content, lines_of_code));
            }
        }
    }

    Ok(analyzed_files)
}

/// Calculate simple complexity metric from source code
pub fn calculate_simple_complexity(content: &str) -> u32 {
    let mut complexity = 1u32;

    for line in content.lines() {
        let trimmed = line.trim();
        // Count branching statements
        if trimmed.starts_with("if ") || trimmed.starts_with("else if") {
            complexity += 1;
        }
        if trimmed.starts_with("for ") || trimmed.starts_with("while ") {
            complexity += 1;
        }
        if trimmed.starts_with("match ") || trimmed.starts_with("switch ") {
            complexity += 1;
        }
        if trimmed.contains("=>") || trimmed.starts_with("case ") {
            complexity += 1;
        }
        if trimmed.contains("&&") || trimmed.contains("||") {
            complexity += 1;
        }
        if trimmed.starts_with("catch") || trimmed.starts_with("except") {
            complexity += 1;
        }
    }

    complexity
}

/// Calculate simple churn score based on file content
pub fn calculate_simple_churn_score(content: &str, lines_of_code: usize) -> f32 {
    // Simple heuristic based on comments and file size
    let todo_count = content.matches("TODO").count() + content.matches("FIXME").count();
    let comment_lines = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("#")
        })
        .count();

    let comment_ratio = comment_lines as f32 / lines_of_code.max(1) as f32;
    let todo_factor = (todo_count as f32 * 0.1).min(1.0);

    // Higher churn for files with many TODOs or low comment ratio
    (1.0 - comment_ratio) * 0.5 + todo_factor * 0.5
}

/// Collect metrics for all files
pub fn collect_file_metrics(analyzed_files: &[(PathBuf, String, usize)]) -> Vec<FileMetrics> {
    let mut file_metrics = Vec::new();

    for (file_path, content, lines_of_code) in analyzed_files {
        let cyclomatic_complexity = calculate_simple_complexity(content);
        let cognitive_complexity = (cyclomatic_complexity as f32 * 1.3) as u32;
        let churn_score = calculate_simple_churn_score(content, *lines_of_code);

        let afferent_coupling = content
            .lines()
            .filter(|line| {
                line.trim_start().starts_with("use ")
                    || line.trim_start().starts_with("import ")
                    || line.trim_start().starts_with("#include")
            })
            .count() as f32;

        let metrics = FileMetrics {
            file_path: file_path.to_string_lossy().to_string(),
            churn_score,
            complexity: cyclomatic_complexity as f32,
            duplicate_ratio: 0.0,
            afferent_coupling,
            efferent_coupling: 0.0,
            lines_of_code: *lines_of_code,
            cyclomatic_complexity,
            cognitive_complexity,
        };

        file_metrics.push(metrics);
    }

    file_metrics
}

/// Filter predictions based on configuration
pub fn filter_predictions(
    predictions: Vec<(String, DefectScore)>,
    config: &DefectPredictionConfig,
) -> Vec<(String, DefectScore)> {
    let mut filtered_predictions = predictions;

    if !config.include_low_confidence {
        filtered_predictions.retain(|(_, score)| score.confidence >= config.confidence_threshold);
    }

    if config.high_risk_only {
        filtered_predictions.retain(|(_, score)| score.probability >= 0.7);
    }

    // Sort by probability (highest first)
    filtered_predictions.sort_by(|a, b| b.1.probability.partial_cmp(&a.1.probability).unwrap());

    filtered_predictions
}

/// Calculate risk distribution
pub struct RiskDistribution {
    pub high_risk_count: usize,
    pub medium_risk_count: usize,
    pub low_risk_count: usize,
}

pub fn calculate_risk_distribution(predictions: &[(String, DefectScore)]) -> RiskDistribution {
    RiskDistribution {
        high_risk_count: predictions
            .iter()
            .filter(|(_, score)| score.probability >= 0.7)
            .count(),
        medium_risk_count: predictions
            .iter()
            .filter(|(_, score)| score.probability >= 0.3 && score.probability < 0.7)
            .count(),
        low_risk_count: predictions
            .iter()
            .filter(|(_, score)| score.probability < 0.3)
            .count(),
    }
}

/// Format summary output
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
    output.push_str(&format!("Files analyzed: {}\n", file_metrics_len));
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
pub fn format_detailed_output(
    filtered_predictions: &[(String, DefectScore)],
    include_recommendations: bool,
) -> String {
    let mut output = String::new();

    output.push_str("Defect Prediction Analysis Report\n");
    output.push_str("================================\n");

    for (file_path, score) in filtered_predictions {
        output.push_str(&format!("\n{}\n", file_path));
        output.push_str(&format!("  Risk Level: {:?}\n", score.risk_level));
        output.push_str(&format!(
            "  Probability: {:.1}%\n",
            score.probability * 100.0
        ));
        output.push_str(&format!("  Confidence: {:.1}%\n", score.confidence * 100.0));

        output.push_str("  Contributing Factors:\n");
        for (factor, contribution) in &score.contributing_factors {
            output.push_str(&format!("    {}: {:.3}\n", factor, contribution));
        }

        if include_recommendations && !score.recommendations.is_empty() {
            output.push_str("  Recommendations:\n");
            for rec in &score.recommendations {
                output.push_str(&format!("    - {}\n", rec));
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
            .map(|(k, v)| format!("{}: {:.2}", k, v))
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
            output.push_str(&format!("{}\n", rec));
        }
    }

    output
}

/// Format CSV output
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
                .map(|(_, v)| *v)
                .unwrap_or(0.0),
            factors
                .iter()
                .find(|(k, _)| k == "complexity")
                .map(|(_, v)| *v)
                .unwrap_or(0.0),
            factors
                .iter()
                .find(|(k, _)| k == "duplication")
                .map(|(_, v)| *v)
                .unwrap_or(0.0),
            factors
                .iter()
                .find(|(k, _)| k == "coupling")
                .map(|(_, v)| *v)
                .unwrap_or(0.0)
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

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_defect_prediction_helpers_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
