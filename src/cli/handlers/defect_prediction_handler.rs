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

/// Configuration for defect prediction analysis
#[derive(Debug, Clone)]
pub struct DefectPredictionConfig {
    pub project_path: PathBuf,
    pub confidence_threshold: f32,
    pub min_lines: usize,
    pub include_low_confidence: bool,
    pub format: DefectPredictionOutputFormat,
    pub high_risk_only: bool,
    pub include_recommendations: bool,
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub output: Option<PathBuf>,
    pub perf: bool,
    pub top_files: usize,
}

/// Refactored handler for defect prediction analysis using the facade pattern.
///
/// This reduces complexity from 23 to ~8 by delegating to the facade service.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_analyze_defect_prediction(config: DefectPredictionConfig) -> Result<()> {
    // GH-666: a nonexistent path scanned zero files and this reported
    // "Analyzed 0 files: 0 high risk, 0 medium risk, 0 low risk /
    // ✓ Defect prediction analysis complete" with exit 0.
    crate::cli::ensure_analysis_path_exists(&config.project_path)?;

    // Print analysis header
    print_analysis_header(
        &config.project_path,
        config.high_risk_only,
        config.include_low_confidence,
    );

    // Create service registry and facade
    let registry = Arc::new(ServiceRegistry::new());
    let facade = DefectPredictionFacade::new(registry);

    // Build analysis request
    let request = DefectPredictionRequest {
        project_path: config.project_path.clone(),
        confidence_threshold: config.confidence_threshold,
        min_lines: config.min_lines,
        include_low_confidence: config.include_low_confidence,
        high_risk_only: config.high_risk_only,
        include_recommendations: config.include_recommendations,
        include: config.include.map(|s| vec![s]),
        exclude: config.exclude.map(|s| vec![s]),
        top_files: config.top_files,
    };

    // Perform analysis using facade
    let result = facade.analyze_project(request).await?;

    // Format and output results
    output_results(result, config.format, config.output).await?;

    {
        use crate::cli::colors as c;
        eprintln!("{}", c::pass("Defect prediction analysis complete"));
    }
    Ok(())
}

/// Print analysis header information
fn print_analysis_header(project_path: &Path, high_risk_only: bool, include_low_confidence: bool) {
    use crate::cli::colors as c;
    eprintln!("{}", c::dim("Analyzing defect probability..."));
    eprintln!(
        "  {}Project path:{} {}",
        c::BOLD,
        c::RESET,
        c::path(&project_path.display().to_string())
    );
    eprintln!(
        "  {}High risk only:{} {}{}{}",
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        high_risk_only,
        c::RESET
    );
    eprintln!(
        "  {}Include low confidence:{} {}{}{}",
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        include_low_confidence,
        c::RESET
    );
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
        println!("{content}");
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
    use crate::cli::colors as c;
    use std::fmt::Write;

    let mut output = String::new();
    let _ = writeln!(output, "{}\n", c::header("Defect Prediction Summary"));
    let _ = writeln!(output, "  {}", result.summary);
    let _ = writeln!(output, "\n{}", c::subheader("Top Risk Files"));

    for (i, prediction) in result.predictions.iter().take(10).enumerate() {
        let risk_color = match prediction.risk_level {
            crate::services::facades::defect_prediction_facade::RiskLevel::Critical => c::RED,
            crate::services::facades::defect_prediction_facade::RiskLevel::High => c::RED,
            crate::services::facades::defect_prediction_facade::RiskLevel::Medium => c::YELLOW,
            crate::services::facades::defect_prediction_facade::RiskLevel::Low => c::GREEN,
        };
        let _ = writeln!(
            output,
            "  {}. {} - {}{:.1}% risk{} ({}{:?}{})",
            c::number(&(i + 1).to_string()),
            c::path(&prediction.file_path),
            risk_color,
            prediction.defect_probability * 100.0,
            c::RESET,
            risk_color,
            prediction.risk_level,
            c::RESET,
        );
    }

    if !result.recommendations.is_empty() {
        let _ = writeln!(output, "\n{}", c::subheader("Recommendations"));
        for rec in &result.recommendations {
            let _ = writeln!(output, "  - {rec}");
        }
    }

    output
}

/// Format as detailed report
fn format_detailed(result: &DefectPredictionResult) -> String {
    use crate::cli::colors as c;
    use std::fmt::Write;

    let mut output = String::new();
    let _ = writeln!(
        output,
        "{}\n",
        c::header("Defect Prediction Detailed Report")
    );
    let _ = writeln!(
        output,
        "  {}Total files analyzed:{} {}{}{}",
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        result.total_files_analyzed,
        c::RESET
    );
    let _ = writeln!(
        output,
        "  {}High risk files:{} {}{}{}",
        c::BOLD,
        c::RESET,
        c::RED,
        result.high_risk_files,
        c::RESET
    );
    let _ = writeln!(
        output,
        "  {}Medium risk files:{} {}{}{}",
        c::BOLD,
        c::RESET,
        c::YELLOW,
        result.medium_risk_files,
        c::RESET
    );
    let _ = writeln!(
        output,
        "  {}Low risk files:{} {}{}{}\n",
        c::BOLD,
        c::RESET,
        c::GREEN,
        result.low_risk_files,
        c::RESET
    );

    let _ = writeln!(output, "{}", c::subheader("File Analysis"));
    for prediction in &result.predictions {
        let risk_color = match prediction.risk_level {
            crate::services::facades::defect_prediction_facade::RiskLevel::Critical => c::RED,
            crate::services::facades::defect_prediction_facade::RiskLevel::High => c::RED,
            crate::services::facades::defect_prediction_facade::RiskLevel::Medium => c::YELLOW,
            crate::services::facades::defect_prediction_facade::RiskLevel::Low => c::GREEN,
        };
        let _ = writeln!(output, "\n  {}", c::path(&prediction.file_path));
        let _ = writeln!(
            output,
            "    {}Risk Level:{} {}{:?}{}",
            c::BOLD,
            c::RESET,
            risk_color,
            prediction.risk_level,
            c::RESET
        );
        let _ = writeln!(
            output,
            "    {}Defect Probability:{} {}{:.1}%{}",
            c::BOLD,
            c::RESET,
            c::BOLD_WHITE,
            prediction.defect_probability * 100.0,
            c::RESET
        );
        let _ = writeln!(
            output,
            "    {}Confidence:{} {}{:.1}%{}",
            c::BOLD,
            c::RESET,
            c::BOLD_WHITE,
            prediction.confidence * 100.0,
            c::RESET
        );

        let _ = writeln!(output, "    {}Risk Metrics:{}", c::BOLD, c::RESET);
        for (name, value) in [
            ("Complexity", prediction.metrics.complexity_score),
            ("Churn", prediction.metrics.churn_score),
            ("Coupling", prediction.metrics.coupling_score),
            ("Size", prediction.metrics.size_score),
            ("Duplication", prediction.metrics.duplication_score),
        ] {
            let _ = writeln!(
                output,
                "      {}{name}:{} {}{}{}",
                c::BOLD,
                c::RESET,
                c::BOLD_WHITE,
                risk_metric(value),
                c::RESET
            );
        }

        if !prediction.contributing_factors.is_empty() {
            let _ = writeln!(output, "    {}Contributing Factors:{}", c::BOLD, c::RESET);
            for factor in &prediction.contributing_factors {
                let _ = writeln!(output, "      - {factor}");
            }
        }
    }

    output
}

/// Render a risk factor, or say plainly that it was not measured.
///
/// These were compile-time constants (churn 0.3, coupling 0.2, duplication 0.1)
/// presented as measurements (GH #657). `None` must never print as 0.0 — a zero
/// risk score reads as a finding.
fn risk_metric(value: Option<f32>) -> String {
    value.map_or_else(|| "not measured".to_string(), |v| format!("{v:.1}"))
}

/// CSV cell for a risk factor: empty means not measured, never 0.000.
fn risk_metric_csv(value: Option<f32>) -> String {
    value.map_or_else(String::new, |v| format!("{v:.3}"))
}

/// Format as CSV
fn format_csv(result: &DefectPredictionResult) -> String {
    let mut output = String::new();
    output.push_str("File,Risk Level,Defect Probability,Confidence,Complexity,Churn,Coupling,Size,Duplication\n");

    for prediction in &result.predictions {
        output.push_str(&format!(
            "{},{:?},{:.3},{:.3},{},{},{},{},{}\n",
            prediction.file_path,
            prediction.risk_level,
            prediction.defect_probability,
            prediction.confidence,
            risk_metric_csv(prediction.metrics.complexity_score),
            risk_metric_csv(prediction.metrics.churn_score),
            risk_metric_csv(prediction.metrics.coupling_score),
            risk_metric_csv(prediction.metrics.size_score),
            risk_metric_csv(prediction.metrics.duplication_score)
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

#[cfg_attr(coverage_nightly, coverage(off))]
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
                    complexity_score: Some(0.8),
                    churn_score: Some(0.7),
                    coupling_score: Some(0.6),
                    size_score: Some(0.5),
                    duplication_score: Some(0.4),
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

    /// GH #657: an unmeasured risk factor must say so, never render as 0.0 — a
    /// zero risk score reads as a finding.
    #[test]
    fn unmeasured_risk_factors_render_as_not_measured() {
        assert_eq!(risk_metric(Some(0.42)), "0.4");
        assert_eq!(risk_metric(None), "not measured");
        assert_ne!(risk_metric(None), "0.0");

        // CSV leaves the cell empty rather than writing 0.000.
        assert_eq!(risk_metric_csv(Some(0.42)), "0.420");
        assert_eq!(risk_metric_csv(None), "");
    }

    /// A file whose churn could not be measured says so in the detailed view.
    #[test]
    fn detailed_output_names_the_unmeasured_factor() {
        let result = DefectPredictionResult {
            total_files_analyzed: 1,
            high_risk_files: 0,
            medium_risk_files: 0,
            low_risk_files: 1,
            predictions: vec![FilePrediction {
                file_path: "nogit.rs".to_string(),
                defect_probability: 0.2,
                risk_level: RiskLevel::Low,
                confidence: 0.75,
                metrics: FileRiskMetrics {
                    complexity_score: Some(0.1),
                    churn_score: None,
                    coupling_score: Some(0.2),
                    size_score: Some(0.05),
                    duplication_score: Some(0.0),
                },
                contributing_factors: vec![],
            }],
            summary: "s".to_string(),
            recommendations: vec![],
        };

        let output = format_detailed(&result);
        assert!(
            output.contains("Churn:") && output.contains("not measured"),
            "churn must be reported as unmeasured, not as a number: {output}"
        );
    }
}
