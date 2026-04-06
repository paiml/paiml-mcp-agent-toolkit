//! Output format dispatch and serialization formats (SARIF, CSV)

use crate::cli::DefectPredictionOutputFormat;
use crate::services::defect_probability::DefectScore;
use anyhow::Result;
use std::path::PathBuf;

use super::detailed_format::{format_defect_detailed, format_defect_json};
use super::summary_format::format_defect_summary;

/// Toyota Way: Extract Method - Format defect output based on format type
pub(crate) fn format_defect_output(
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
pub(crate) async fn output_results(
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

/// Format predictions as SARIF
pub(crate) fn format_defect_sarif(predictions: &[(String, DefectScore)]) -> Result<String> {
    debug_assert!(!predictions.is_empty(), "predictions must not be empty");
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
pub(crate) fn format_defect_csv(predictions: &[(String, DefectScore)]) -> Result<String> {
    debug_assert!(!predictions.is_empty(), "predictions must not be empty");
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
