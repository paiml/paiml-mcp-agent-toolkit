//! Defect Prediction Analysis Handler
//!
//! Refactored handler using the service facade pattern to reduce complexity.

use crate::cli::DefectPredictionOutputFormat;
use crate::services::facades::defect_prediction_facade::{
    DefectPredictionFacade, DefectPredictionRequest, DefectPredictionResult,
};
use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Refactored handler for defect prediction analysis using the facade pattern.
///
/// This reduces complexity from 23 to ~8 by delegating to the facade service.
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
    _perf: bool,
    top_files: usize,
) -> Result<()> {
    // Print analysis header
    print_analysis_header(&project_path, high_risk_only, include_low_confidence);

    // Create service registry and facade
    let registry = Arc::new(ServiceRegistry::new());
    let facade = DefectPredictionFacade::new(registry);

    // Build analysis request
    let request = DefectPredictionRequest {
        project_path: project_path.clone(),
        confidence_threshold,
        min_lines,
        include_low_confidence,
        high_risk_only,
        include_recommendations,
        include: include.map(|s| vec![s]),
        exclude: exclude.map(|s| vec![s]),
        top_files,
    };

    // Perform analysis using facade
    let result = facade.analyze_project(request).await?;

    // Format and output results
    output_results(result, format, output).await?;

    eprintln!("✅ Defect prediction analysis complete");
    Ok(())
}

/// Print analysis header information
fn print_analysis_header(
    project_path: &Path,
    high_risk_only: bool,
    include_low_confidence: bool,
) {
    eprintln!("🔮 Analyzing defect probability...");
    eprintln!("📁 Project path: {}", project_path.display());
    eprintln!("🎯 High risk only: {}", high_risk_only);
    eprintln!("📊 Include low confidence: {}", include_low_confidence);
}

/// Output results in the requested format
async fn output_results(
    result: DefectPredictionResult,
    format: DefectPredictionOutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    let content = format_result(result, format)?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("📝 Written to {}", output_path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

/// Format the analysis result based on the requested format
fn format_result(
    result: DefectPredictionResult,
    format: DefectPredictionOutputFormat,
) -> Result<String> {
    match format {
        DefectPredictionOutputFormat::Summary => Ok(format_summary(&result)),
        DefectPredictionOutputFormat::Detailed => Ok(format_detailed(&result)),
        DefectPredictionOutputFormat::Json => {
            serde_json::to_string_pretty(&result).map_err(Into::into)
        }
        DefectPredictionOutputFormat::Csv => Ok(format_csv(&result)),
        DefectPredictionOutputFormat::Sarif => Ok(format_sarif(&result)),
    }
}

/// Format as summary
fn format_summary(result: &DefectPredictionResult) -> String {
    let mut output = String::new();
    output.push_str("# Defect Prediction Summary\n\n");
    output.push_str(&result.summary);
    output.push_str("\n\n## Top Risk Files\n");

    for (i, prediction) in result.predictions.iter().take(10).enumerate() {
        output.push_str(&format!(
            "{}. {} - {:.1}% risk ({:?})\n",
            i + 1,
            prediction.file_path,
            prediction.defect_probability * 100.0,
            prediction.risk_level
        ));
    }

    if !result.recommendations.is_empty() {
        output.push_str("\n## Recommendations\n");
        for rec in &result.recommendations {
            output.push_str(&format!("- {}\n", rec));
        }
    }

    output
}

/// Format as detailed report
fn format_detailed(result: &DefectPredictionResult) -> String {
    let mut output = String::new();
    output.push_str("# Defect Prediction Detailed Report\n\n");
    output.push_str(&format!(
        "Total files analyzed: {}\n",
        result.total_files_analyzed
    ));
    output.push_str(&format!("High risk files: {}\n", result.high_risk_files));
    output.push_str(&format!(
        "Medium risk files: {}\n",
        result.medium_risk_files
    ));
    output.push_str(&format!("Low risk files: {}\n\n", result.low_risk_files));

    output.push_str("## File Analysis\n");
    for prediction in &result.predictions {
        output.push_str(&format!("\n### {}\n", prediction.file_path));
        output.push_str(&format!("- Risk Level: {:?}\n", prediction.risk_level));
        output.push_str(&format!(
            "- Defect Probability: {:.1}%\n",
            prediction.defect_probability * 100.0
        ));
        output.push_str(&format!(
            "- Confidence: {:.1}%\n",
            prediction.confidence * 100.0
        ));

        output.push_str("- Risk Metrics:\n");
        output.push_str(&format!(
            "  - Complexity: {:.1}\n",
            prediction.metrics.complexity_score
        ));
        output.push_str(&format!(
            "  - Churn: {:.1}\n",
            prediction.metrics.churn_score
        ));
        output.push_str(&format!(
            "  - Coupling: {:.1}\n",
            prediction.metrics.coupling_score
        ));
        output.push_str(&format!("  - Size: {:.1}\n", prediction.metrics.size_score));
        output.push_str(&format!(
            "  - Duplication: {:.1}\n",
            prediction.metrics.duplication_score
        ));

        if !prediction.contributing_factors.is_empty() {
            output.push_str("- Contributing Factors:\n");
            for factor in &prediction.contributing_factors {
                output.push_str(&format!("  - {}\n", factor));
            }
        }
    }

    output
}

/// Format as CSV
fn format_csv(result: &DefectPredictionResult) -> String {
    let mut output = String::new();
    output.push_str("File,Risk Level,Defect Probability,Confidence,Complexity,Churn,Coupling,Size,Duplication\n");

    for prediction in &result.predictions {
        output.push_str(&format!(
            "{},{:?},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}\n",
            prediction.file_path,
            prediction.risk_level,
            prediction.defect_probability,
            prediction.confidence,
            prediction.metrics.complexity_score,
            prediction.metrics.churn_score,
            prediction.metrics.coupling_score,
            prediction.metrics.size_score,
            prediction.metrics.duplication_score
        ));
    }

    output
}

/// Format as SARIF
fn format_sarif(result: &DefectPredictionResult) -> String {
    let rules = vec![serde_json::json!({
        "id": "high-defect-risk",
        "shortDescription": {
            "text": "High defect probability detected"
        },
        "fullDescription": {
            "text": "Files with high defect probability require additional testing and review"
        }
    })];

    let results: Vec<_> = result
        .predictions
        .iter()
        .filter(|p| {
            matches!(
                p.risk_level,
                crate::services::facades::defect_prediction_facade::RiskLevel::High
                    | crate::services::facades::defect_prediction_facade::RiskLevel::Critical
            )
        })
        .map(|prediction| {
            serde_json::json!({
                "ruleId": "high-defect-risk",
                "level": if matches!(prediction.risk_level,
                    crate::services::facades::defect_prediction_facade::RiskLevel::Critical) {
                    "error"
                } else {
                    "warning"
                },
                "message": {
                    "text": format!(
                        "File has {:.1}% defect probability. Contributing factors: {}",
                        prediction.defect_probability * 100.0,
                        prediction.contributing_factors.join(", ")
                    )
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": prediction.file_path.clone()
                        }
                    }
                }],
                "properties": {
                    "defectProbability": prediction.defect_probability,
                    "confidence": prediction.confidence,
                    "riskLevel": format!("{:?}", prediction.risk_level)
                }
            })
        })
        .collect();

    serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-defect-prediction",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": rules
                }
            },
            "results": results
        }]
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::facades::defect_prediction_facade::{
        FilePrediction, FileRiskMetrics, RiskLevel,
    };

    #[test]
    fn test_format_summary() {
        let result = DefectPredictionResult {
            total_files_analyzed: 10,
            high_risk_files: 3,
            medium_risk_files: 4,
            low_risk_files: 3,
            predictions: vec![FilePrediction {
                file_path: "test.rs".to_string(),
                defect_probability: 0.8,
                risk_level: RiskLevel::High,
                confidence: 0.9,
                metrics: FileRiskMetrics {
                    complexity_score: 0.8,
                    churn_score: 0.7,
                    coupling_score: 0.6,
                    size_score: 0.5,
                    duplication_score: 0.4,
                },
                contributing_factors: vec!["High complexity".to_string()],
            }],
            summary: "Test summary".to_string(),
            recommendations: vec!["Test recommendation".to_string()],
        };

        let output = format_summary(&result);
        assert!(output.contains("Test summary"));
        assert!(output.contains("test.rs"));
        assert!(output.contains("80.0%"));
        assert!(output.contains("Test recommendation"));
    }
}
