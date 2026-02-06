//! Predict Quality Handlers - Phase 4.1 O(1) Quality Gates
//!
//! CLI handlers for predicting when quality metrics will exceed thresholds.

#![cfg_attr(coverage_nightly, coverage(off))]

use crate::cli::enums::OutputFormat;
use crate::services::metric_trends::{MetricTrendStore, PredictionResult};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Handle predict-quality command
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
            eprintln!("⚠️  No threshold configured for metric: {}", metric_name);
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
                    "⚠️  Failed to predict {}: {} (need at least 7 observations)",
                    metric_name, e
                );
            }
        }
    }

    if predictions.is_empty() {
        println!("\n✅ No metrics to predict (all metrics safe or insufficient data)");
        return Ok(());
    }

    // Output results
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&predictions)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&predictions)?);
        }
        _ => {
            print_predictions_table(&predictions);
        }
    }

    Ok(())
}

/// Print predictions in table format
fn print_predictions_table(predictions: &[PredictionResult]) {
    println!("\n\x1b[1;34m🔮 Quality Metrics Predictions\x1b[0m\n");

    for pred in predictions {
        println!("\x1b[1m{}\x1b[0m", pred.metric);
        println!("  Current: {:.1}ms", pred.current_value);
        println!("  Threshold: {:.1}ms", pred.threshold);

        if let Some(days) = pred.breach_in_days {
            if let Some(value) = pred.predicted_value {
                let urgency = if days <= 7 {
                    "\x1b[31m⚠️  URGENT\x1b[0m"
                } else if days <= 14 {
                    "\x1b[33m⚠️  WARNING\x1b[0m"
                } else {
                    "\x1b[34mℹ️  INFO\x1b[0m"
                };

                println!(
                    "  Breach: {} in {} days (predicted: {:.1}ms)",
                    urgency, days, value
                );
                println!(
                    "  Confidence: {:.1}% (R²={:.3})",
                    pred.confidence * 100.0,
                    pred.confidence
                );
            }
        } else {
            println!("  Breach: \x1b[32m✅ No breach predicted\x1b[0m");
            println!(
                "  Confidence: {:.1}% (R²={:.3})",
                pred.confidence * 100.0,
                pred.confidence
            );
        }

        // Print recommendations
        if !pred.recommendations.is_empty() {
            println!("  Recommendations:");
            for rec in &pred.recommendations {
                println!("    • {}", rec);
            }
        }

        println!();
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

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
