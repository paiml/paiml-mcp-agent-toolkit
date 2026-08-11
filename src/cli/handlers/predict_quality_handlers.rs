//! Predict Quality Handlers - Phase 4.1 O(1) Quality Gates
//!
//! CLI handlers for predicting when quality metrics will exceed thresholds.

#![cfg_attr(coverage_nightly, coverage(off))]

use crate::cli::colors as c;
use crate::cli::enums::OutputFormat;
use crate::services::metric_trends::{MetricTrendStore, PredictionResult};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Handle predict-quality command
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_predict_quality(
    metric: Option<String>,
    threshold: Option<f64>,
    days: usize,
    format: OutputFormat,
    all: bool,
    failures_only: bool,
) -> Result<()> {
    // Support PMAT_METRICS_DIR for testing
    let mut store = if let Ok(metrics_dir) = std::env::var("PMAT_METRICS_DIR") {
        let trends_path = PathBuf::from(metrics_dir).join("trends");
        MetricTrendStore::from_path(trends_path)?
    } else {
        MetricTrendStore::new()?
    };

    // Default thresholds from .pmat-metrics.toml
    let default_thresholds = HashMap::from([
        ("lint".to_string(), 30_000.0),
        ("test-fast".to_string(), 300_000.0),
        ("coverage".to_string(), 600_000.0),
        ("build-release".to_string(), 50_000_000.0),
    ]);

    // Determine which metrics to analyze
    let metrics_to_check = if all {
        store.metrics()?
    } else if let Some(m) = metric {
        vec![m]
    } else {
        anyhow::bail!("Must specify --metric or --all");
    };

    // Generate predictions
    let mut predictions = Vec::new();

    for metric_name in metrics_to_check {
        let threshold_value = threshold
            .or_else(|| default_thresholds.get(&metric_name).copied())
            .unwrap_or(0.0);

        if threshold_value == 0.0 {
            eprintln!(
                "{} No threshold configured for metric: {}",
                c::warn(""),
                c::label(&metric_name)
            );
            continue;
        }

        match store.predict_threshold_breach(&metric_name, threshold_value, days) {
            Ok(prediction) => {
                // Filter if failures_only
                if failures_only && prediction.breach_in_days.is_none() {
                    continue;
                }

                predictions.push(prediction);
            }
            Err(e) => {
                eprintln!(
                    "{} Failed to predict {}: {} {}",
                    c::warn(""),
                    c::label(&metric_name),
                    e,
                    c::dim("(need at least 7 observations)")
                );
            }
        }
    }

    println!("{}", render_predictions(&predictions, format)?);

    Ok(())
}

/// Render predictions in the DECLARED format.
///
/// The empty case used to `println!` a prose line and return before this match
/// ever ran, so on any fresh checkout — where no metric has the 7 observations a
/// prediction needs — `predict-quality --all --format json` wrote
/// `✓ No metrics to predict …` to stdout and every `| jq` consumer failed. A
/// declared machine format has to produce that format on every path, and the
/// honest empty answer in JSON is `[]`.
fn render_predictions(predictions: &[PredictionResult], format: OutputFormat) -> Result<String> {
    Ok(match format {
        OutputFormat::Json => serde_json::to_string_pretty(predictions)?,
        OutputFormat::Yaml => serde_yaml_ng::to_string(predictions)?,
        _ if predictions.is_empty() => format!(
            "\n{}",
            c::pass("No metrics to predict (all metrics safe or insufficient data)")
        ),
        _ => format_predictions_table(predictions),
    })
}

/// Render predictions in table format
fn format_predictions_table(predictions: &[PredictionResult]) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "\n{}\n", c::header("Quality Metrics Predictions"));

    for pred in predictions {
        let _ = writeln!(out, "{}", c::subheader(&pred.metric));
        let _ = writeln!(out, "  {}: {:.1}ms", c::dim("Current"), pred.current_value);
        let _ = writeln!(out, "  {}: {:.1}ms", c::dim("Threshold"), pred.threshold);

        if let Some(days) = pred.breach_in_days {
            if let Some(value) = pred.predicted_value {
                let urgency = if days <= 7 {
                    format!("{}URGENT{}", c::BOLD_RED, c::RESET)
                } else if days <= 14 {
                    format!("{}WARNING{}", c::BOLD_YELLOW, c::RESET)
                } else {
                    format!("{}INFO{}", c::BOLD_BLUE, c::RESET)
                };

                let _ = writeln!(
                    out,
                    "  {}: {} in {} days (predicted: {:.1}ms)",
                    c::dim("Breach"),
                    urgency,
                    c::number(&days.to_string()),
                    value
                );
                let _ = writeln!(
                    out,
                    "  {}: {} (R²={:.3})",
                    c::dim("Confidence"),
                    c::pct(pred.confidence * 100.0, 80.0, 50.0),
                    pred.confidence
                );
            }
        } else {
            let _ = writeln!(
                out,
                "  {}: {}",
                c::dim("Breach"),
                c::pass("No breach predicted")
            );
            let _ = writeln!(
                out,
                "  {}: {} (R²={:.3})",
                c::dim("Confidence"),
                c::pct(pred.confidence * 100.0, 80.0, 50.0),
                pred.confidence
            );
        }

        // Print recommendations
        if !pred.recommendations.is_empty() {
            let _ = writeln!(out, "  {}:", c::dim("Recommendations"));
            for rec in &pred.recommendations {
                let _ = writeln!(out, "    {} {}", c::dim("•"), rec);
            }
        }

        let _ = writeln!(out);
    }

    out
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh checkout has no metric with the 7 observations a prediction
    /// needs, so this is the ordinary case, not an edge case: `--format json`
    /// printed `✓ No metrics to predict …` on stdout and broke every `| jq`.
    #[test]
    fn empty_predictions_still_produce_the_declared_format() {
        let json = render_predictions(&[], OutputFormat::Json).expect("json");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(parsed, serde_json::json!([]));

        let yaml = render_predictions(&[], OutputFormat::Yaml).expect("yaml");
        let parsed: serde_yaml_ng::Value = serde_yaml_ng::from_str(&yaml).expect("valid yaml");
        assert_eq!(parsed, serde_yaml_ng::Value::Sequence(vec![]));

        // The human formats keep the prose.
        let table = render_predictions(&[], OutputFormat::Table).expect("table");
        assert!(table.contains("No metrics to predict"), "{table:?}");
    }

    #[tokio::test]
    #[ignore] // Flaky - requires specific environment
    async fn test_predict_quality_no_metric() {
        // Should fail if no metric specified and --all not set
        let result =
            handle_predict_quality(None, None, 30, OutputFormat::Table, false, false).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Must specify"));
    }
}

#[cfg(test)]
mod predict_quality_print_tests {
    //! Covers format_predictions_table in predict_quality_handlers.rs
    //! (46 uncov on broad, 0% cov). Drives all four urgency arms,
    //! the no-breach arm, and the recommendations arm.
    use super::*;
    use crate::services::metric_trends::PredictionResult;

    fn pred(metric: &str, breach_days: Option<usize>, recs: Vec<&str>) -> PredictionResult {
        PredictionResult {
            metric: metric.into(),
            current_value: 100.0,
            threshold: 200.0,
            breach_in_days: breach_days,
            predicted_value: breach_days.map(|_| 250.0),
            confidence: 0.85,
            recommendations: recs.into_iter().map(String::from).collect(),
            forecast: vec![],
        }
    }

    #[test]
    fn test_print_predictions_table_no_breach_arm() {
        // breach_in_days=None → "No breach predicted" arm fires.
        let p = pred("lint_p99", None, vec![]);
        format_predictions_table(&[p]);
    }

    #[test]
    fn test_print_predictions_table_urgent_arm() {
        // days <= 7 → URGENT.
        let p = pred("test_p99", Some(3), vec![]);
        format_predictions_table(&[p]);
    }

    #[test]
    fn test_print_predictions_table_warning_arm() {
        // 7 < days <= 14 → WARNING.
        let p = pred("test_p99", Some(10), vec![]);
        format_predictions_table(&[p]);
    }

    #[test]
    fn test_print_predictions_table_info_arm() {
        // days > 14 → INFO.
        let p = pred("test_p99", Some(30), vec![]);
        format_predictions_table(&[p]);
    }

    #[test]
    fn test_print_predictions_table_with_recommendations() {
        let p = pred("complexity", Some(5), vec!["Refactor X", "Extract Y"]);
        format_predictions_table(&[p]);
    }

    #[test]
    fn test_print_predictions_table_empty_input_no_panic() {
        format_predictions_table(&[]);
    }

    #[test]
    fn test_print_predictions_table_multiple_predictions() {
        let preds = vec![
            pred("a", None, vec![]),
            pred("b", Some(3), vec!["fix it"]),
            pred("c", Some(20), vec![]),
        ];
        format_predictions_table(&preds);
    }
}
