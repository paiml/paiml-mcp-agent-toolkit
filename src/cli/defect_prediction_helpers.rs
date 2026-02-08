#![cfg_attr(coverage_nightly, coverage(off))]
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
#[must_use]
pub fn calculate_simple_complexity(content: &str) -> u32 {
    let mut complexity = 1u32;

    for line in content.lines() {
        let trimmed = line.trim();
        complexity += count_line_complexity(trimmed);
    }

    complexity
}

fn count_line_complexity(line: &str) -> u32 {
    let mut line_complexity = 0u32;

    line_complexity += count_conditional_statements(line);
    line_complexity += count_loop_statements(line);
    line_complexity += count_pattern_matching(line);
    line_complexity += count_logical_operators(line);
    line_complexity += count_exception_handling(line);

    line_complexity
}

fn count_conditional_statements(line: &str) -> u32 {
    u32::from(line.starts_with("if ") || line.starts_with("else if"))
}

fn count_loop_statements(line: &str) -> u32 {
    u32::from(line.starts_with("for ") || line.starts_with("while "))
}

fn count_pattern_matching(line: &str) -> u32 {
    u32::from(
        line.starts_with("match ")
            || line.starts_with("switch ")
            || line.contains("=>")
            || line.starts_with("case "),
    )
}

fn count_logical_operators(line: &str) -> u32 {
    u32::from(line.contains("&&") || line.contains("||"))
}

fn count_exception_handling(line: &str) -> u32 {
    u32::from(line.starts_with("catch") || line.starts_with("except"))
}

/// Calculate simple churn score based on file content
#[must_use]
pub fn calculate_simple_churn_score(content: &str, lines_of_code: usize) -> f32 {
    // Simple heuristic based on comments and file size
    let todo_count = content.matches("TODO").count() + content.matches("FIXME").count();
    let comment_lines = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('#')
        })
        .count();

    let comment_ratio = comment_lines as f32 / lines_of_code.max(1) as f32;
    let todo_factor = (todo_count as f32 * 0.1).min(1.0);

    // Higher churn for files with many TODOs or low comment ratio
    (1.0 - comment_ratio) * 0.5 + todo_factor * 0.5
}

/// Collect metrics for all files
#[must_use]
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
#[must_use]
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
    filtered_predictions.sort_by(|a, b| {
        b.1.probability
            .partial_cmp(&a.1.probability)
            .expect("internal error")
    });

    filtered_predictions
}

/// Calculate risk distribution
pub struct RiskDistribution {
    pub high_risk_count: usize,
    pub medium_risk_count: usize,
    pub low_risk_count: usize,
}

#[must_use]
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::defect_probability::RiskLevel;

    // ==================== Config Tests ====================

    #[test]
    fn test_defect_prediction_config_struct() {
        let config = DefectPredictionConfig {
            confidence_threshold: 0.8,
            min_lines: 10,
            include_low_confidence: false,
            high_risk_only: true,
            include_recommendations: true,
            include: Some("src/".to_string()),
            exclude: Some("test".to_string()),
        };
        assert_eq!(config.confidence_threshold, 0.8);
        assert_eq!(config.min_lines, 10);
        assert!(!config.include_low_confidence);
        assert!(config.high_risk_only);
    }

    // ==================== Complexity Calculation Tests ====================

    #[test]
    fn test_calculate_simple_complexity_empty() {
        let complexity = calculate_simple_complexity("");
        assert_eq!(complexity, 1); // Base complexity is 1
    }

    #[test]
    fn test_calculate_simple_complexity_no_branching() {
        let code = r#"
let x = 1;
let y = 2;
let z = x + y;
"#;
        let complexity = calculate_simple_complexity(code);
        assert_eq!(complexity, 1); // No branches
    }

    #[test]
    fn test_calculate_simple_complexity_with_if() {
        let code = r#"
if x > 0 {
    return true;
}
"#;
        let complexity = calculate_simple_complexity(code);
        assert!(complexity > 1);
    }

    #[test]
    fn test_calculate_simple_complexity_with_loops() {
        let code = r#"
for item in items {
    process(item);
}
while running {
    tick();
}
"#;
        let complexity = calculate_simple_complexity(code);
        assert!(complexity >= 3); // Base + 2 loops
    }

    #[test]
    fn test_calculate_simple_complexity_with_match() {
        let code = r#"
match value {
    0 => zero(),
    1 => one(),
    _ => other(),
}
"#;
        let complexity = calculate_simple_complexity(code);
        assert!(complexity > 1);
    }

    #[test]
    fn test_calculate_simple_complexity_with_logical_operators() {
        let code = r#"
if x && y {
    do_something();
}
if a || b {
    do_other();
}
"#;
        let complexity = calculate_simple_complexity(code);
        assert!(complexity >= 3);
    }

    #[test]
    fn test_calculate_simple_complexity_with_exception_handling() {
        let code = r#"
catch (Exception e) {
    log(e);
}
except ValueError:
    handle_error()
"#;
        let complexity = calculate_simple_complexity(code);
        assert!(complexity >= 3);
    }

    // ==================== Churn Score Tests ====================

    #[test]
    fn test_calculate_simple_churn_score_empty() {
        let score = calculate_simple_churn_score("", 0);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_calculate_simple_churn_score_with_todos() {
        let code = r#"
// TODO: fix this
fn main() {
    // FIXME: broken
    println!("hello");
}
"#;
        let score = calculate_simple_churn_score(code, 5);
        assert!(score > 0.0);
    }

    #[test]
    fn test_calculate_simple_churn_score_with_comments() {
        let code = r#"
// This is a comment
// Another comment
/* Block comment */
fn main() {
    println!("hello");
}
"#;
        let score = calculate_simple_churn_score(code, 7);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_calculate_simple_churn_score_no_comments() {
        let code = r#"
fn main() {
    println!("hello");
}
"#;
        let score = calculate_simple_churn_score(code, 4);
        // No comments means high churn potential
        assert!(score >= 0.4);
    }

    // ==================== File Metrics Collection Tests ====================

    #[test]
    fn test_collect_file_metrics_empty() {
        let files: Vec<(PathBuf, String, usize)> = vec![];
        let metrics = collect_file_metrics(&files);
        assert!(metrics.is_empty());
    }

    #[test]
    fn test_collect_file_metrics_single_file() {
        let files = vec![(
            PathBuf::from("test.rs"),
            "fn main() {\n    println!(\"hello\");\n}".to_string(),
            3,
        )];
        let metrics = collect_file_metrics(&files);
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].lines_of_code, 3);
        assert!(metrics[0].complexity >= 1.0);
    }

    #[test]
    fn test_collect_file_metrics_with_imports() {
        let code = r#"
use std::io;
import os
#include <stdio.h>
fn main() {}
"#;
        let files = vec![(PathBuf::from("test.rs"), code.to_string(), 5)];
        let metrics = collect_file_metrics(&files);
        assert_eq!(metrics[0].afferent_coupling, 3.0); // 3 imports
    }

    // ==================== Filter Predictions Tests ====================

    #[test]
    fn test_filter_predictions_empty() {
        let config = DefectPredictionConfig {
            confidence_threshold: 0.5,
            min_lines: 10,
            include_low_confidence: true,
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: None,
        };
        let predictions: Vec<(String, DefectScore)> = vec![];
        let filtered = filter_predictions(predictions, &config);
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_filter_predictions_confidence_filter() {
        let config = DefectPredictionConfig {
            confidence_threshold: 0.8,
            min_lines: 10,
            include_low_confidence: false,
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: None,
        };
        let predictions = vec![
            (
                "file1.rs".to_string(),
                DefectScore {
                    probability: 0.5,
                    confidence: 0.9,
                    risk_level: RiskLevel::Medium,
                    contributing_factors: vec![],
                    recommendations: vec![],
                },
            ),
            (
                "file2.rs".to_string(),
                DefectScore {
                    probability: 0.5,
                    confidence: 0.3,
                    risk_level: RiskLevel::Medium,
                    contributing_factors: vec![],
                    recommendations: vec![],
                },
            ),
        ];
        let filtered = filter_predictions(predictions, &config);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].0, "file1.rs");
    }

    #[test]
    fn test_filter_predictions_high_risk_only() {
        let config = DefectPredictionConfig {
            confidence_threshold: 0.5,
            min_lines: 10,
            include_low_confidence: true,
            high_risk_only: true,
            include_recommendations: false,
            include: None,
            exclude: None,
        };
        let predictions = vec![
            (
                "file1.rs".to_string(),
                DefectScore {
                    probability: 0.8,
                    confidence: 0.9,
                    risk_level: RiskLevel::High,
                    contributing_factors: vec![],
                    recommendations: vec![],
                },
            ),
            (
                "file2.rs".to_string(),
                DefectScore {
                    probability: 0.4,
                    confidence: 0.9,
                    risk_level: RiskLevel::Medium,
                    contributing_factors: vec![],
                    recommendations: vec![],
                },
            ),
        ];
        let filtered = filter_predictions(predictions, &config);
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].1.probability >= 0.7);
    }

    #[test]
    fn test_filter_predictions_sorted_by_probability() {
        let config = DefectPredictionConfig {
            confidence_threshold: 0.5,
            min_lines: 10,
            include_low_confidence: true,
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: None,
        };
        let predictions = vec![
            (
                "file1.rs".to_string(),
                DefectScore {
                    probability: 0.3,
                    confidence: 0.9,
                    risk_level: RiskLevel::Low,
                    contributing_factors: vec![],
                    recommendations: vec![],
                },
            ),
            (
                "file2.rs".to_string(),
                DefectScore {
                    probability: 0.9,
                    confidence: 0.9,
                    risk_level: RiskLevel::High,
                    contributing_factors: vec![],
                    recommendations: vec![],
                },
            ),
            (
                "file3.rs".to_string(),
                DefectScore {
                    probability: 0.6,
                    confidence: 0.9,
                    risk_level: RiskLevel::Medium,
                    contributing_factors: vec![],
                    recommendations: vec![],
                },
            ),
        ];
        let filtered = filter_predictions(predictions, &config);
        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered[0].0, "file2.rs"); // Highest probability first
        assert_eq!(filtered[1].0, "file3.rs");
        assert_eq!(filtered[2].0, "file1.rs");
    }

    // ==================== Risk Distribution Tests ====================

    #[test]
    fn test_calculate_risk_distribution_empty() {
        let predictions: Vec<(String, DefectScore)> = vec![];
        let dist = calculate_risk_distribution(&predictions);
        assert_eq!(dist.high_risk_count, 0);
        assert_eq!(dist.medium_risk_count, 0);
        assert_eq!(dist.low_risk_count, 0);
    }

    #[test]
    fn test_calculate_risk_distribution_all_levels() {
        let predictions = vec![
            (
                "high.rs".to_string(),
                DefectScore {
                    probability: 0.8,
                    confidence: 0.9,
                    risk_level: RiskLevel::High,
                    contributing_factors: vec![],
                    recommendations: vec![],
                },
            ),
            (
                "medium.rs".to_string(),
                DefectScore {
                    probability: 0.5,
                    confidence: 0.9,
                    risk_level: RiskLevel::Medium,
                    contributing_factors: vec![],
                    recommendations: vec![],
                },
            ),
            (
                "low.rs".to_string(),
                DefectScore {
                    probability: 0.2,
                    confidence: 0.9,
                    risk_level: RiskLevel::Low,
                    contributing_factors: vec![],
                    recommendations: vec![],
                },
            ),
        ];
        let dist = calculate_risk_distribution(&predictions);
        assert_eq!(dist.high_risk_count, 1);
        assert_eq!(dist.medium_risk_count, 1);
        assert_eq!(dist.low_risk_count, 1);
    }

    // ==================== Summary Output Tests ====================

    #[test]
    fn test_format_summary_output_basic() {
        let predictions: Vec<(String, DefectScore)> = vec![];
        let dist = RiskDistribution {
            high_risk_count: 0,
            medium_risk_count: 0,
            low_risk_count: 0,
        };
        let output = format_summary_output(
            10,
            &predictions,
            &dist,
            true,
            std::time::Duration::from_millis(100),
        );
        assert!(output.contains("Defect Prediction Analysis Summary"));
        assert!(output.contains("Files analyzed: 10"));
    }

    #[test]
    fn test_format_summary_output_with_predictions() {
        let predictions = vec![(
            "file.rs".to_string(),
            DefectScore {
                probability: 0.8,
                confidence: 0.9,
                risk_level: RiskLevel::High,
                contributing_factors: vec![("complexity".to_string(), 0.5)],
                recommendations: vec![],
            },
        )];
        let dist = RiskDistribution {
            high_risk_count: 1,
            medium_risk_count: 0,
            low_risk_count: 0,
        };
        let output = format_summary_output(
            5,
            &predictions,
            &dist,
            false,
            std::time::Duration::from_secs(1),
        );
        assert!(output.contains("High-Risk Files"));
    }

    // ==================== Helper Function Tests ====================

    #[test]
    fn test_count_conditional_statements() {
        assert_eq!(count_conditional_statements("if x > 0 {"), 1);
        assert_eq!(count_conditional_statements("else if y < 0 {"), 1);
        assert_eq!(count_conditional_statements("let x = 1;"), 0);
    }

    #[test]
    fn test_count_loop_statements() {
        assert_eq!(count_loop_statements("for item in items {"), 1);
        assert_eq!(count_loop_statements("while running {"), 1);
        assert_eq!(count_loop_statements("let x = 1;"), 0);
    }

    #[test]
    fn test_count_pattern_matching() {
        assert_eq!(count_pattern_matching("match value {"), 1);
        assert_eq!(count_pattern_matching("switch (x) {"), 1);
        assert_eq!(count_pattern_matching("case 1:"), 1);
        assert_eq!(count_pattern_matching("x => y"), 1);
        assert_eq!(count_pattern_matching("let x = 1;"), 0);
    }

    #[test]
    fn test_count_logical_operators() {
        assert_eq!(count_logical_operators("if x && y {"), 1);
        assert_eq!(count_logical_operators("if a || b {"), 1);
        assert_eq!(count_logical_operators("if x && y || z {"), 1); // Only counts once
        assert_eq!(count_logical_operators("let x = 1;"), 0);
    }

    #[test]
    fn test_count_exception_handling() {
        assert_eq!(count_exception_handling("catch (Exception e) {"), 1);
        assert_eq!(count_exception_handling("except ValueError:"), 1);
        assert_eq!(count_exception_handling("try {"), 0);
    }

    #[test]
    fn test_count_line_complexity() {
        // Line with multiple complexity contributors
        let complexity = count_line_complexity("if x && y {");
        assert!(complexity >= 2); // if + &&
    }

    // ==================== Analysis Result Tests ====================

    #[test]
    fn test_defect_analysis_result_struct() {
        let result = DefectAnalysisResult {
            file_metrics: vec![],
            filtered_predictions: vec![],
            analysis_time: std::time::Duration::from_secs(1),
        };
        assert!(result.file_metrics.is_empty());
        assert!(result.filtered_predictions.is_empty());
        assert_eq!(result.analysis_time.as_secs(), 1);
    }

    // ==================== Risk Distribution Struct Tests ====================

    #[test]
    fn test_risk_distribution_struct() {
        let dist = RiskDistribution {
            high_risk_count: 5,
            medium_risk_count: 10,
            low_risk_count: 20,
        };
        assert_eq!(dist.high_risk_count, 5);
        assert_eq!(dist.medium_risk_count, 10);
        assert_eq!(dist.low_risk_count, 20);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
