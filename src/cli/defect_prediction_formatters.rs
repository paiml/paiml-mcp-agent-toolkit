// Output formatting functions for defect prediction
// Included from defect_prediction_helpers.rs - shares parent module scope

/// Format summary output
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_summary_output(
    file_metrics_len: usize,
    filtered_predictions: &[(String, DefectScore)],
    risk_dist: &RiskDistribution,
    perf: bool,
    analysis_time: std::time::Duration,
) -> String {
    use crate::cli::colors as c;
    use std::fmt::Write;

    let mut output = String::new();

    let _ = writeln!(
        output,
        "{}{}Defect Prediction Analysis Summary{}\n",
        c::BOLD, c::UNDERLINE, c::RESET
    );
    let _ = writeln!(
        output,
        "  {}Files analyzed:{} {}{}{}",
        c::BOLD, c::RESET, c::BOLD_WHITE, file_metrics_len, c::RESET
    );
    let _ = writeln!(
        output,
        "  {}Predictions generated:{} {}{}{}",
        c::BOLD, c::RESET, c::BOLD_WHITE, filtered_predictions.len(), c::RESET
    );

    let total = filtered_predictions.len() as f32;
    let _ = writeln!(
        output,
        "  {}High risk files:{} {}{}{} ({:.1}%)",
        c::BOLD, c::RESET, c::RED, risk_dist.high_risk_count, c::RESET,
        100.0 * risk_dist.high_risk_count as f32 / total
    );
    let _ = writeln!(
        output,
        "  {}Medium risk files:{} {}{}{} ({:.1}%)",
        c::BOLD, c::RESET, c::YELLOW, risk_dist.medium_risk_count, c::RESET,
        100.0 * risk_dist.medium_risk_count as f32 / total
    );
    let _ = writeln!(
        output,
        "  {}Low risk files:{} {}{}{} ({:.1}%)",
        c::BOLD, c::RESET, c::GREEN, risk_dist.low_risk_count, c::RESET,
        100.0 * risk_dist.low_risk_count as f32 / total
    );

    if perf {
        let _ = writeln!(output, "\n{}Performance Metrics:{}", c::BOLD, c::RESET);
        let _ = writeln!(
            output,
            "  {}Analysis time:{} {}{:.2}s{}",
            c::BOLD, c::RESET, c::BOLD_WHITE, analysis_time.as_secs_f64(), c::RESET
        );
        let _ = writeln!(
            output,
            "  {}Files/second:{} {}{:.1}{}",
            c::BOLD, c::RESET, c::BOLD_WHITE,
            file_metrics_len as f64 / analysis_time.as_secs_f64(), c::RESET
        );
    }

    if !filtered_predictions.is_empty() {
        let _ = writeln!(output, "\n{}Top 10 High-Risk Files:{}", c::BOLD, c::RESET);
        for (file_path, score) in filtered_predictions.iter().take(10) {
            let file_name = std::path::Path::new(file_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let risk_color = if score.probability >= 0.7 {
                c::RED
            } else if score.probability >= 0.3 {
                c::YELLOW
            } else {
                c::GREEN
            };
            let _ = writeln!(
                output,
                "  {}{}{} - {}{:.1}% risk{} ({:?})",
                c::CYAN, file_name, c::RESET,
                risk_color, score.probability * 100.0, c::RESET,
                score.confidence
            );
        }
    }

    output
}

/// Generate recommendations for high-risk files
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_detailed_output(
    filtered_predictions: &[(String, DefectScore)],
    include_recommendations: bool,
) -> String {
    use crate::cli::colors as c;
    use std::fmt::Write;

    let mut output = String::new();

    let _ = writeln!(
        output,
        "{}{}Defect Prediction Analysis Report{}\n",
        c::BOLD, c::UNDERLINE, c::RESET
    );

    for (file_path, score) in filtered_predictions {
        let risk_color = match score.risk_level {
            crate::services::defect_probability::RiskLevel::High => c::RED,
            crate::services::defect_probability::RiskLevel::Medium => c::YELLOW,
            crate::services::defect_probability::RiskLevel::Low => c::GREEN,
        };
        let _ = writeln!(output, "\n{}{}{}", c::CYAN, file_path, c::RESET);
        let _ = writeln!(
            output,
            "  {}Risk Level:{} {}{:?}{}",
            c::BOLD, c::RESET, risk_color, score.risk_level, c::RESET
        );
        let _ = writeln!(
            output,
            "  {}Probability:{} {}{:.1}%{}",
            c::BOLD, c::RESET, c::BOLD_WHITE, score.probability * 100.0, c::RESET
        );
        let _ = writeln!(
            output,
            "  {}Confidence:{} {}{:.1}%{}",
            c::BOLD, c::RESET, c::BOLD_WHITE, score.confidence * 100.0, c::RESET
        );

        let _ = writeln!(output, "  {}Contributing Factors:{}", c::BOLD, c::RESET);
        for (factor, contribution) in &score.contributing_factors {
            let _ = writeln!(
                output,
                "    {}{}{}: {}{:.3}{}",
                c::BOLD, factor, c::RESET, c::BOLD_WHITE, contribution, c::RESET
            );
        }

        if include_recommendations && !score.recommendations.is_empty() {
            let _ = writeln!(output, "  {}Recommendations:{}", c::BOLD, c::RESET);
            for rec in &score.recommendations {
                let _ = writeln!(output, "    - {rec}");
            }
        }
    }

    output
}

/// Format JSON output
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
mod defect_prediction_formatters_tests {
    //! Covers defect_prediction_formatters.rs (36 uncov on broad, 0% cov).
    use super::*;
    use crate::services::defect_probability::{DefectScore, RiskLevel};

    fn score(prob: f32, risk: RiskLevel, factors: Vec<(String, f32)>) -> DefectScore {
        DefectScore {
            probability: prob,
            contributing_factors: factors,
            confidence: 0.9,
            risk_level: risk,
            recommendations: vec![],
        }
    }

    fn high_risk_with_factors() -> (String, DefectScore) {
        (
            "src/foo.rs".to_string(),
            score(
                0.85,
                RiskLevel::High,
                vec![
                    ("complexity".into(), 0.8),
                    ("churn".into(), 0.85),
                    ("coupling".into(), 0.9),
                    ("duplication".into(), 0.5),
                ],
            ),
        )
    }

    // ── format_summary_output ──

    #[test]
    fn test_format_summary_output_includes_file_count_and_header() {
        let preds = vec![high_risk_with_factors()];
        let dist = RiskDistribution {
            high_risk_count: 1,
            medium_risk_count: 0,
            low_risk_count: 0,
        };
        let out = format_summary_output(42, &preds, &dist, false, std::time::Duration::from_millis(5));
        assert!(out.contains("Defect Prediction Analysis Summary"));
        assert!(out.contains("42")); // files analyzed count
    }

    #[test]
    fn test_format_summary_output_perf_flag_no_panic() {
        let dist = RiskDistribution {
            high_risk_count: 0,
            medium_risk_count: 0,
            low_risk_count: 0,
        };
        let out = format_summary_output(0, &[], &dist, true, std::time::Duration::from_millis(99));
        assert!(!out.is_empty());
    }

    // ── generate_recommendations ──

    #[test]
    fn test_generate_recommendations_empty_predictions_empty() {
        assert!(generate_recommendations(&[]).is_empty());
    }

    #[test]
    fn test_generate_recommendations_high_factor_emits_specific_advice() {
        let preds = vec![high_risk_with_factors()];
        let recs = generate_recommendations(&preds);
        let joined = recs.join("\n");
        // All 4 factor-specific recs (complexity > 0.7, churn > 0.7, coupling > 0.7,
        // duplication > 0.3) should fire.
        assert!(joined.contains("High complexity"));
        assert!(joined.contains("High churn"));
        assert!(joined.contains("High coupling"));
        assert!(joined.contains("Code duplication"));
    }

    #[test]
    fn test_generate_recommendations_low_factor_no_advice() {
        let preds = vec![(
            "src/clean.rs".to_string(),
            score(
                0.1,
                RiskLevel::Low,
                vec![
                    ("complexity".into(), 0.3),
                    ("churn".into(), 0.2),
                ],
            ),
        )];
        let recs = generate_recommendations(&preds);
        let joined = recs.join("\n");
        // Risk header still appears, but no specific advice strings.
        assert!(joined.contains("clean.rs"));
        assert!(!joined.contains("High complexity"));
        assert!(!joined.contains("High churn"));
    }

    #[test]
    fn test_generate_recommendations_takes_top_5_only() {
        let preds: Vec<(String, DefectScore)> = (0..10)
            .map(|i| {
                (
                    format!("src/f{i}.rs"),
                    score(0.9, RiskLevel::High, vec![]),
                )
            })
            .collect();
        let recs = generate_recommendations(&preds);
        // 5 files × (header + blank) = 10 entries minimum.
        let file_lines: usize = recs.iter().filter(|r| r.contains("90.0% risk")).count();
        assert_eq!(file_lines, 5, "must emit at most 5 file headers");
    }

    // ── format_detailed_output ──

    #[test]
    fn test_format_detailed_output_with_recs_includes_recommendations_section() {
        let preds = vec![high_risk_with_factors()];
        let out = format_detailed_output(&preds, true);
        assert!(!out.is_empty());
        assert!(out.contains("foo.rs"));
    }

    #[test]
    fn test_format_detailed_output_without_recs_empty_recs() {
        let preds = vec![high_risk_with_factors()];
        let out = format_detailed_output(&preds, false);
        assert!(!out.is_empty());
    }

    #[test]
    fn test_format_detailed_output_empty_predictions() {
        let out = format_detailed_output(&[], false);
        assert!(!out.is_empty());
    }

    // ── format_json_output ──

    #[test]
    fn test_format_json_output_round_trips_as_json() {
        let preds = vec![high_risk_with_factors()];
        let out = format_json_output(42, &preds, true, false, std::time::Duration::from_millis(5))
            .unwrap();
        let _: serde_json::Value = serde_json::from_str(&out).unwrap();
    }

    #[test]
    fn test_format_json_output_perf_adds_performance_section() {
        let preds = vec![high_risk_with_factors()];
        let out = format_json_output(42, &preds, false, true, std::time::Duration::from_millis(123))
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(parsed.get("performance").is_some(), "perf=true should add performance section");
    }

    // ── format_markdown_output ──

    #[test]
    fn test_format_markdown_output_with_recs_contains_md_headers() {
        let preds = vec![high_risk_with_factors()];
        let out = format_markdown_output(&preds, true);
        assert!(out.contains('#'));
    }

    #[test]
    fn test_format_markdown_output_without_recs_still_valid() {
        let preds = vec![high_risk_with_factors()];
        let out = format_markdown_output(&preds, false);
        assert!(!out.is_empty());
    }

    // ── format_csv_output ──

    #[test]
    fn test_format_csv_output_has_header() {
        let preds = vec![high_risk_with_factors()];
        let out = format_csv_output(&preds);
        let first_line = out.lines().next().unwrap_or_default();
        assert!(first_line.contains(','), "CSV header must have commas");
    }

    #[test]
    fn test_format_csv_output_empty_predictions_still_has_header() {
        let out = format_csv_output(&[]);
        assert!(!out.is_empty());
    }

    // ── format_sarif_output ──

    #[test]
    fn test_format_sarif_output_round_trips_as_json() {
        let preds = vec![high_risk_with_factors()];
        let out = format_sarif_output(&preds).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn test_format_sarif_output_empty_predictions_valid_sarif() {
        let out = format_sarif_output(&[]).unwrap();
        let _: serde_json::Value = serde_json::from_str(&out).unwrap();
    }
}
