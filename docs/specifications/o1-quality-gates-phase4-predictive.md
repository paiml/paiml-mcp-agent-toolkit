# O(1) Quality Gates Phase 4: Predictive Quality Gates

**Status**: In Progress
**Sprint**: 48
**Ticket**: QUAL-O1-PHASE4
**Dependencies**: Phase 3.2 (CSR Storage), aprender ML library

## Overview

Use machine learning (aprender) to predict when quality metrics will exceed thresholds, enabling proactive optimization before CI breaks.

## Problem Statement

**Current**: Developers discover threshold violations *after* they occur
- Commit blocked by pre-commit hook
- Emergency optimization required
- Development velocity disrupted

**Goal**: Predict violations *before* they happen
- "Your lint time will exceed 30s in 12 days"
- Time to optimize proactively
- Maintain stable development velocity

## Architecture

### Prediction Pipeline

```
Historical Data (CSR Graph)
         ↓
Feature Engineering:
- Time-series data (timestamp, value)
- Rolling statistics (7-day mean/std)
- Trend acceleration (Δ²/Δt²)
         ↓
aprender LinearRegression
- Train on last 90 days
- Forecast next 30 days
- Compute confidence intervals
         ↓
Threshold Detection:
- Find first breach point
- Estimate days until breach
- Compute prediction confidence
         ↓
Recommendation Engine:
- Rule-based expert system
- Past successful optimizations
- Metric-specific advice
```

### Data Model

```rust
/// Prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// Metric name
    pub metric: String,

    /// Current value (last observation)
    pub current_value: f64,

    /// Threshold being checked
    pub threshold: f64,

    /// Days until threshold exceeded (None if no breach predicted)
    pub breach_in_days: Option<usize>,

    /// Predicted value at breach point
    pub predicted_value: Option<f64>,

    /// Prediction confidence (R² score, 0.0-1.0)
    pub confidence: f64,

    /// Actionable recommendations
    pub recommendations: Vec<String>,

    /// Forecast for next 30 days
    pub forecast: Vec<ForecastPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPoint {
    pub days_ahead: usize,
    pub predicted_value: f64,
    pub lower_bound: f64,  // 95% confidence interval
    pub upper_bound: f64,
}
```

## Implementation

### 1. Linear Regression Forecaster

```rust
use aprender::{Array1, Array2};

impl MetricTrendStore {
    /// Predict when metric will exceed threshold
    pub fn predict_threshold_breach(
        &mut self,
        metric: &str,
        threshold: f64,
        forecast_days: usize,
    ) -> Result<PredictionResult> {
        // Load historical data
        if !self.cache.contains_key(metric) {
            self.load(metric)?;
        }

        let observations = self.cache.get(metric)
            .context("Metric not found")?;

        if observations.len() < 7 {
            anyhow::bail!("Need at least 7 observations for prediction");
        }

        // Prepare training data (last 90 days)
        let now = chrono::Utc::now().timestamp();
        let cutoff = now - (90 * 86400);

        let training_data: Vec<_> = observations
            .iter()
            .filter(|obs| obs.timestamp >= cutoff)
            .cloned()
            .collect();

        // Train linear model
        let model = self.train_linear_model(&training_data)?;

        // Generate forecast
        let forecast = self.generate_forecast(&model, &training_data, forecast_days)?;

        // Find breach point
        let breach = forecast.iter()
            .enumerate()
            .find(|(_, point)| point.predicted_value > threshold);

        let (breach_in_days, predicted_value) = match breach {
            Some((days, point)) => (Some(days), Some(point.predicted_value)),
            None => (None, None),
        };

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            metric,
            breach_in_days,
            threshold,
        );

        Ok(PredictionResult {
            metric: metric.to_string(),
            current_value: observations.last().unwrap().value,
            threshold,
            breach_in_days,
            predicted_value,
            confidence: model.r_squared,
            recommendations,
            forecast,
        })
    }

    /// Train linear regression model on historical data
    fn train_linear_model(&self, observations: &[MetricObservation]) -> Result<LinearModel> {
        // Normalize timestamps to days since first observation
        let first_ts = observations[0].timestamp;

        // X: days since start (independent variable)
        let x: Vec<f64> = observations
            .iter()
            .map(|obs| (obs.timestamp - first_ts) as f64 / 86400.0)
            .collect();

        // Y: metric values (dependent variable)
        let y: Vec<f64> = observations
            .iter()
            .map(|obs| obs.value)
            .collect();

        // Simple linear regression: y = mx + b
        let n = x.len() as f64;
        let mean_x = x.iter().sum::<f64>() / n;
        let mean_y = y.iter().sum::<f64>() / n;

        // Slope (m)
        let numerator: f64 = x.iter()
            .zip(&y)
            .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
            .sum();

        let denominator: f64 = x.iter()
            .map(|xi| (xi - mean_x).powi(2))
            .sum();

        let slope = numerator / denominator;

        // Intercept (b)
        let intercept = mean_y - slope * mean_x;

        // Compute R² (coefficient of determination)
        let predictions: Vec<f64> = x.iter()
            .map(|xi| slope * xi + intercept)
            .collect();

        let ss_res: f64 = y.iter()
            .zip(&predictions)
            .map(|(yi, pred)| (yi - pred).powi(2))
            .sum();

        let ss_tot: f64 = y.iter()
            .map(|yi| (yi - mean_y).powi(2))
            .sum();

        let r_squared = 1.0 - (ss_res / ss_tot);

        Ok(LinearModel {
            slope,
            intercept,
            r_squared,
            last_timestamp: observations.last().unwrap().timestamp,
        })
    }

    /// Generate forecast for next N days
    fn generate_forecast(
        &self,
        model: &LinearModel,
        training_data: &[MetricObservation],
        forecast_days: usize,
    ) -> Result<Vec<ForecastPoint>> {
        let first_ts = training_data[0].timestamp;
        let last_day = (model.last_timestamp - first_ts) as f64 / 86400.0;

        // Compute standard error for confidence intervals
        let residuals: Vec<f64> = training_data
            .iter()
            .map(|obs| {
                let days = (obs.timestamp - first_ts) as f64 / 86400.0;
                let predicted = model.slope * days + model.intercept;
                obs.value - predicted
            })
            .collect();

        let sse: f64 = residuals.iter().map(|r| r.powi(2)).sum();
        let mse = sse / (training_data.len() as f64 - 2.0);
        let std_error = mse.sqrt();

        // Generate forecast points
        let mut forecast = Vec::new();

        for days_ahead in 1..=forecast_days {
            let future_day = last_day + days_ahead as f64;
            let predicted_value = model.slope * future_day + model.intercept;

            // 95% confidence interval (±1.96 * SE)
            let margin = 1.96 * std_error;

            forecast.push(ForecastPoint {
                days_ahead,
                predicted_value,
                lower_bound: predicted_value - margin,
                upper_bound: predicted_value + margin,
            });
        }

        Ok(forecast)
    }

    /// Generate actionable recommendations
    fn generate_recommendations(
        &self,
        metric: &str,
        breach_in_days: Option<usize>,
        threshold: f64,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if breach_in_days.is_none() {
            recommendations.push("No threshold breach predicted in next 30 days".to_string());
            recommendations.push("Continue current practices".to_string());
            return recommendations;
        }

        // Metric-specific recommendations
        match metric {
            "lint" => {
                recommendations.push("Remove unused dependencies (saves ~2-3s)".to_string());
                recommendations.push("Enable incremental clippy analysis".to_string());
                recommendations.push("Review enabled clippy lints (disable pedantic if not needed)".to_string());
                recommendations.push("Use cargo-cache to clean old artifacts".to_string());
            }
            "test-fast" => {
                recommendations.push("Parallelize test execution (use --test-threads)".to_string());
                recommendations.push("Use #[ignore] for slow property tests".to_string());
                recommendations.push("Implement test fixtures to reduce setup time".to_string());
                recommendations.push("Profile tests to identify slowest ones".to_string());
            }
            "coverage" => {
                recommendations.push("Run coverage only in CI (skip locally)".to_string());
                recommendations.push("Use --exclude for non-critical modules".to_string());
                recommendations.push("Skip expensive property-based tests in coverage".to_string());
                recommendations.push("Consider sampling coverage (not 100% runs)".to_string());
            }
            "build-release" => {
                recommendations.push("Enable LTO only in final release builds".to_string());
                recommendations.push("Reduce codegen-units for faster linking".to_string());
                recommendations.push("Use sccache for distributed compilation".to_string());
                recommendations.push("Review dependency tree for bloat".to_string());
            }
            _ => {
                recommendations.push(format!("Review {} history for optimization opportunities", metric));
                recommendations.push("Profile to identify bottlenecks".to_string());
            }
        }

        // Add urgency-based recommendations
        if let Some(days) = breach_in_days {
            if days <= 7 {
                recommendations.insert(0, "⚠️ URGENT: Threshold breach imminent - prioritize optimization".to_string());
            } else if days <= 14 {
                recommendations.insert(0, "⚠️ WARNING: Threshold breach in 2 weeks - schedule optimization".to_string());
            } else {
                recommendations.insert(0, format!("ℹ️ INFO: {} days until breach - plan optimization", days));
            }
        }

        recommendations
    }
}

/// Linear regression model
#[derive(Debug, Clone)]
struct LinearModel {
    slope: f64,
    intercept: f64,
    r_squared: f64,
    last_timestamp: i64,
}
```

### 2. CLI Integration

```rust
// In server/src/cli/commands.rs

Commands::PredictQuality {
    metric,
    threshold,
    days,
    format,
    all,
    failures_only,
} => {
    handlers::handle_predict_quality(
        metric,
        threshold,
        days,
        format,
        all,
        failures_only,
    ).await
}

// Command definition
#[command(visible_aliases = &["predict"])]
PredictQuality {
    /// Specific metric to predict (lint, test-fast, coverage, build-release)
    #[arg(long)]
    metric: Option<String>,

    /// Threshold value (ms or bytes)
    #[arg(long)]
    threshold: Option<f64>,

    /// Days to forecast (default: 30)
    #[arg(long, default_value_t = 30)]
    days: usize,

    /// Output format
    #[arg(long, value_enum, default_value = "table")]
    format: OutputFormat,

    /// Predict all metrics
    #[arg(long)]
    all: bool,

    /// Show only metrics at risk of breach
    #[arg(long)]
    failures_only: bool,
}
```

### 3. Handler Implementation

```rust
// In server/src/cli/handlers/predict_quality_handlers.rs

use crate::services::metric_trends::MetricTrendStore;
use anyhow::Result;

pub async fn handle_predict_quality(
    metric: Option<String>,
    threshold: Option<f64>,
    days: usize,
    format: OutputFormat,
    all: bool,
    failures_only: bool,
) -> Result<()> {
    let mut store = MetricTrendStore::new()?;

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

        if let Ok(prediction) = store.predict_threshold_breach(&metric_name, threshold_value, days) {
            // Filter if failures_only
            if failures_only && prediction.breach_in_days.is_none() {
                continue;
            }

            predictions.push(prediction);
        }
    }

    // Output results
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&predictions)?);
        }
        _ => {
            print_predictions_table(&predictions);
        }
    }

    Ok(())
}

fn print_predictions_table(predictions: &[PredictionResult]) {
    println!("\n\x1b[1;34m🔮 Quality Metrics Predictions\x1b[0m\n");

    for pred in predictions {
        println!("\x1b[1m{}\x1b[0m", pred.metric);
        println!("  Current: {:.1}ms", pred.current_value);
        println!("  Threshold: {:.1}ms", pred.threshold);

        if let Some(days) = pred.breach_in_days {
            if let Some(value) = pred.predicted_value {
                let urgency = if days <= 7 {
                    "\x1b[31m⚠️ URGENT\x1b[0m"
                } else if days <= 14 {
                    "\x1b[33m⚠️ WARNING\x1b[0m"
                } else {
                    "\x1b[34mℹ️ INFO\x1b[0m"
                };

                println!("  Breach: {} in {} days (predicted: {:.1}ms)", urgency, days, value);
                println!("  Confidence: {:.1}% (R²={:.3})", pred.confidence * 100.0, pred.confidence);
            }
        } else {
            println!("  Breach: \x1b[32m✅ No breach predicted\x1b[0m");
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
```

## Testing

### Unit Tests

```rust
#[test]
fn test_linear_model_training() {
    let mut store = MetricTrendStore::from_path("/tmp/pmat-test-prediction").unwrap();
    let now = chrono::Utc::now().timestamp();

    // Record observations with linear trend (increasing by 100ms/day)
    for i in 0..30 {
        let value = 20000.0 + (i as f64 * 100.0);
        let ts = now - ((29 - i) * 86400);
        store.record("lint", value, ts).unwrap();
    }

    // Train model
    let observations = store.cache.get("lint").unwrap();
    let model = store.train_linear_model(observations).unwrap();

    // Verify slope is ~100 (100ms/day)
    assert!((model.slope - 100.0).abs() < 10.0, "Slope should be ~100");

    // Verify R² is high (good fit)
    assert!(model.r_squared > 0.95, "R² should be >0.95 for linear data");
}

#[test]
fn test_threshold_breach_prediction() {
    let mut store = MetricTrendStore::from_path("/tmp/pmat-test-breach").unwrap();
    let now = chrono::Utc::now().timestamp();

    // Current: 25000ms, increasing 200ms/day
    // Threshold: 30000ms
    // Expected breach: (30000 - 25000) / 200 = 25 days

    for i in 0..20 {
        let value = 21000.0 + (i as f64 * 200.0);
        let ts = now - ((19 - i) * 86400);
        store.record("lint", value, ts).unwrap();
    }

    let prediction = store.predict_threshold_breach("lint", 30_000.0, 30).unwrap();

    assert!(prediction.breach_in_days.is_some(), "Should predict breach");
    let days = prediction.breach_in_days.unwrap();
    assert!(days >= 20 && days <= 30, "Breach should be in 20-30 days, got {}", days);
}

#[test]
fn test_no_breach_prediction() {
    let mut store = MetricTrendStore::from_path("/tmp/pmat-test-no-breach").unwrap();
    let now = chrono::Utc::now().timestamp();

    // Decreasing trend (improving) - should never breach
    for i in 0..20 {
        let value = 30000.0 - (i as f64 * 200.0);
        let ts = now - ((19 - i) * 86400);
        store.record("lint", value, ts).unwrap();
    }

    let prediction = store.predict_threshold_breach("lint", 35_000.0, 30).unwrap();

    assert!(prediction.breach_in_days.is_none(), "Should not predict breach for improving trend");
}
```

### Integration Tests

```rust
#[test]
fn test_cli_predict_quality() {
    // Setup test data
    let mut store = MetricTrendStore::new().unwrap();
    // ... record test data ...

    // Run CLI command
    let output = Command::new("pmat")
        .args(&["predict-quality", "--metric", "lint", "--threshold", "30000"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify output contains prediction
    assert!(stdout.contains("Breach"));
    assert!(stdout.contains("Confidence"));
    assert!(stdout.contains("Recommendations"));
}
```

## Success Criteria

- ✅ Linear regression model trains on historical data
- ✅ R² score computed for prediction confidence
- ✅ Forecast generated for next 30 days
- ✅ Threshold breach detected correctly
- ✅ Recommendations generated per metric
- ✅ CLI command outputs predictions
- ✅ JSON format supported for automation
- ✅ All tests passing

## Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Model training | <100ms | 90 days of data |
| Forecast generation | <50ms | 30-day forecast |
| Total prediction | <200ms | End-to-end |
| Prediction accuracy | >85% | Within ±20% of actual |

## Toyota Way Principles

- **Jidoka** (Built-in Quality): Proactive alerts prevent threshold violations
- **Andon Cord** (Stop the Line): Predict issues before they become emergencies
- **Kaizen** (Continuous Improvement): Learn from trends, optimize continuously
- **Genchi Genbutsu** (Go and See): ML predictions based on actual data, not estimates

## References

1. Linear Regression: "Introduction to Statistical Learning" (James et al., 2021)
2. Time Series Forecasting: "Forecasting: Principles and Practice" (Hyndman & Athanasopoulos, 2021)
3. Confidence Intervals: "Statistical Inference" (Casella & Berger, 2002)
4. aprender documentation: https://docs.rs/aprender
