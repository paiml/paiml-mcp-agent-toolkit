//! Output format dispatch and serialization formats (SARIF, CSV)

use crate::cli::DefectPredictionOutputFormat;
use crate::services::defect_probability::DefectScore;
use anyhow::Result;
use std::path::PathBuf;

use super::detailed_format::{format_defect_detailed, format_defect_json};
use super::summary_format::format_defect_summary;

/// Toyota Way: Extract Method - Format defect output based on format type
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_defect_sarif(predictions: &[(String, DefectScore)]) -> Result<String> {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_defect_csv(predictions: &[(String, DefectScore)]) -> Result<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::defect_probability::RiskLevel;
    use std::time::Duration;

    fn score(p: f32, risk: RiskLevel) -> DefectScore {
        DefectScore {
            probability: p,
            confidence: 0.9,
            contributing_factors: vec![("complexity".to_string(), 0.5)],
            risk_level: risk,
            recommendations: vec!["refactor".to_string()],
        }
    }

    fn pred(file: &str, p: f32, r: RiskLevel) -> (String, DefectScore) {
        (file.to_string(), score(p, r))
    }

    // ── format_defect_output dispatcher ─────────────────────────────────────

    #[test]
    fn test_format_defect_output_summary_arm() {
        let preds = vec![pred("a.rs", 0.9, RiskLevel::High)];
        let out = format_defect_output(
            DefectPredictionOutputFormat::Summary,
            &preds,
            Duration::from_millis(50),
            false,
        )
        .unwrap();
        assert!(out.contains("Defect Prediction Summary"));
    }

    #[test]
    fn test_format_defect_output_json_arm() {
        let preds = vec![pred("a.rs", 0.9, RiskLevel::High)];
        let out = format_defect_output(
            DefectPredictionOutputFormat::Json,
            &preds,
            Duration::from_millis(50),
            false,
        )
        .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["analysis_type"], "defect_prediction");
    }

    #[test]
    fn test_format_defect_output_detailed_arm() {
        let preds = vec![pred("a.rs", 0.9, RiskLevel::High)];
        let out = format_defect_output(
            DefectPredictionOutputFormat::Detailed,
            &preds,
            Duration::from_millis(50),
            true,
        )
        .unwrap();
        assert!(out.contains("Defect Prediction Detailed Report"));
    }

    #[test]
    fn test_format_defect_output_sarif_arm() {
        let preds = vec![pred("a.rs", 0.9, RiskLevel::High)];
        let out = format_defect_output(
            DefectPredictionOutputFormat::Sarif,
            &preds,
            Duration::from_millis(50),
            false,
        )
        .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["version"], "2.1.0");
    }

    #[test]
    fn test_format_defect_output_csv_arm() {
        let preds = vec![pred("a.rs", 0.9, RiskLevel::High)];
        let out = format_defect_output(
            DefectPredictionOutputFormat::Csv,
            &preds,
            Duration::from_millis(50),
            false,
        )
        .unwrap();
        assert!(out.starts_with("file,probability,"));
    }

    // ── format_defect_sarif ─────────────────────────────────────────────────

    #[test]
    fn test_format_defect_sarif_empty_predictions() {
        let out = format_defect_sarif(&[]).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["version"], "2.1.0");
        assert_eq!(
            parsed["runs"][0]["tool"]["driver"]["name"],
            "pmat-defect-prediction"
        );
        assert_eq!(parsed["runs"][0]["results"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_format_defect_sarif_high_risk_maps_to_error_level() {
        let preds = vec![pred("a.rs", 0.9, RiskLevel::High)];
        let out = format_defect_sarif(&preds).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        let result = &parsed["runs"][0]["results"][0];
        assert_eq!(result["level"], "error");
        assert_eq!(result["ruleId"], "DEFECT-RISK");
        // URI propagation
        assert_eq!(
            result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"],
            "a.rs"
        );
    }

    #[test]
    fn test_format_defect_sarif_medium_risk_maps_to_warning() {
        let preds = vec![pred("m.rs", 0.5, RiskLevel::Medium)];
        let out = format_defect_sarif(&preds).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["runs"][0]["results"][0]["level"], "warning");
    }

    #[test]
    fn test_format_defect_sarif_low_risk_maps_to_note() {
        let preds = vec![pred("l.rs", 0.1, RiskLevel::Low)];
        let out = format_defect_sarif(&preds).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["runs"][0]["results"][0]["level"], "note");
    }

    #[test]
    fn test_format_defect_sarif_includes_message_with_pcts() {
        let preds = vec![pred("a.rs", 0.85, RiskLevel::High)];
        let out = format_defect_sarif(&preds).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        let msg = parsed["runs"][0]["results"][0]["message"]["text"]
            .as_str()
            .unwrap();
        assert!(msg.contains("85.0%"));
        assert!(msg.contains("90.0%")); // confidence
    }

    #[test]
    fn test_format_defect_sarif_properties_include_factors() {
        let preds = vec![pred("a.rs", 0.85, RiskLevel::High)];
        let out = format_defect_sarif(&preds).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        let props = &parsed["runs"][0]["results"][0]["properties"];
        assert!(props["probability"].is_number());
        assert!(props["confidence"].is_number());
        assert!(props["contributing_factors"].is_array());
        assert!(props["recommendations"].is_array());
    }

    // ── format_defect_csv ───────────────────────────────────────────────────

    #[test]
    fn test_format_defect_csv_empty_emits_header_only() {
        let out = format_defect_csv(&[]).unwrap();
        assert!(out.starts_with("file,probability,confidence,risk_level,"));
        // Single line — no data rows
        assert_eq!(out.lines().count(), 1);
    }

    #[test]
    fn test_format_defect_csv_single_row_with_factor() {
        let preds = vec![pred("a.rs", 0.85, RiskLevel::High)];
        let out = format_defect_csv(&preds).unwrap();
        let lines: Vec<_> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[1].starts_with("a.rs,0.850,0.900,High,complexity,0.500"));
    }

    #[test]
    fn test_format_defect_csv_no_factor_uses_blank_defaults() {
        let mut s = score(0.5, RiskLevel::Medium);
        s.contributing_factors.clear();
        let preds = vec![("b.rs".to_string(), s)];
        let out = format_defect_csv(&preds).unwrap();
        // Empty factor → "" and 0.000
        let row = out.lines().nth(1).unwrap();
        assert!(row.contains("b.rs,0.500,0.900,Medium,,0.000"));
    }

    #[test]
    fn test_format_defect_csv_multiple_rows() {
        let preds = vec![
            pred("a.rs", 0.9, RiskLevel::High),
            pred("b.rs", 0.5, RiskLevel::Medium),
            pred("c.rs", 0.1, RiskLevel::Low),
        ];
        let out = format_defect_csv(&preds).unwrap();
        // 1 header + 3 data rows
        assert_eq!(out.lines().count(), 4);
    }

    // ── output_results ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_output_results_to_stdout_when_no_path() {
        let result =
            output_results("hello".to_string(), None, false, Duration::from_millis(0)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_output_results_writes_to_file_when_path_set() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let out_path = tmp.path().join("report.json");
        let result = output_results(
            "{\"x\":1}".to_string(),
            Some(out_path.clone()),
            false,
            Duration::from_millis(0),
        )
        .await;
        assert!(result.is_ok());
        let written = std::fs::read_to_string(&out_path).unwrap();
        assert_eq!(written, "{\"x\":1}");
    }

    #[tokio::test]
    async fn test_output_results_perf_flag_no_panic() {
        // Just exercise the perf-enabled branch
        let result = output_results("x".to_string(), None, true, Duration::from_millis(123)).await;
        assert!(result.is_ok());
    }
}
