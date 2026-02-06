//! Metrics Command Handlers for CommandDispatcher
//!
//! Extracted from command_dispatcher mod.rs for file health compliance (CB-040).
//! Contains show-metrics and record-metric command execution.

#![cfg_attr(coverage_nightly, coverage(off))]

use super::CommandDispatcher;
use crate::cli::OutputFormat;

impl CommandDispatcher {
    /// Execute show-metrics command (Phase 3.1 O(1) Quality Gates)
    pub(crate) async fn execute_show_metrics_command(
        trend: bool,
        days: usize,
        metric: Option<String>,
        format: OutputFormat,
        failures_only: bool,
    ) -> anyhow::Result<()> {
        use crate::services::metric_trends::{MetricTrendStore, TrendDirection};

        if !trend {
            anyhow::bail!("Only --trend mode is currently supported");
        }

        let mut store = MetricTrendStore::new()?;

        let metrics = if let Some(m) = metric {
            vec![m]
        } else {
            store.metrics()?
        };

        // Load all metrics into graph first (for PageRank)
        for metric_name in &metrics {
            let _ = store.trend(metric_name, days); // This loads data and populates graph
        }

        // Update PageRank hotness scores (after data is loaded)
        store.update_hotness()?;

        match format {
            OutputFormat::Json => {
                let mut results = serde_json::Map::new();

                // Add hot metrics ranking
                let hot_metrics = store.hot_metrics();
                let mut hot_map = serde_json::Map::new();
                for (name, score) in hot_metrics {
                    hot_map.insert(name, serde_json::json!(score));
                }
                results.insert(
                    "hot_metrics".to_string(),
                    serde_json::Value::Object(hot_map),
                );

                // Add trend analysis
                for metric_name in metrics {
                    if let Ok(trend_analysis) = store.trend(&metric_name, days) {
                        if failures_only && trend_analysis.direction != TrendDirection::Regressing {
                            continue;
                        }
                        results.insert(metric_name, serde_json::to_value(trend_analysis)?);
                    }
                }
                println!("{}", serde_json::to_string_pretty(&results)?);
            }
            _ => {
                // Table output (default)
                println!(
                    "\n\x1b[1;34mQuality Metrics Trends ({} days)\x1b[0m\n",
                    days
                );

                // Show hot metrics ranking (PageRank)
                let hot_metrics = store.hot_metrics();
                if !hot_metrics.is_empty() {
                    println!("\x1b[1;33mHot Metrics (PageRank)\x1b[0m");
                    for (idx, (name, score)) in hot_metrics.iter().enumerate().take(5) {
                        println!("  {}. {} (score: {:.4})", idx + 1, name, score);
                    }
                    println!();
                }

                // Sort metrics by hotness for display
                let mut sorted_metrics: Vec<(String, f32)> = metrics
                    .iter()
                    .map(|m| {
                        let score = hot_metrics
                            .iter()
                            .find(|(name, _)| name == m)
                            .map(|(_, s)| *s)
                            .unwrap_or(0.0);
                        (m.clone(), score)
                    })
                    .collect();
                sorted_metrics.sort_by(|a, b| b.1.total_cmp(&a.1));

                for (metric_name, _hotness) in sorted_metrics {
                    if let Ok(trend_analysis) = store.trend(&metric_name, days) {
                        if failures_only && trend_analysis.direction != TrendDirection::Regressing {
                            continue;
                        }

                        let direction_symbol = match trend_analysis.direction {
                            TrendDirection::Improving => "\x1b[32mImproving\x1b[0m",
                            TrendDirection::Stable => "\x1b[33mStable\x1b[0m",
                            TrendDirection::Regressing => "\x1b[31mRegressing\x1b[0m",
                        };

                        println!("\x1b[1m{}\x1b[0m", metric_name);
                        println!("  Direction: {}", direction_symbol);
                        println!("  Mean: {:.2}", trend_analysis.mean);
                        println!("  Std Dev: {:.2}", trend_analysis.std_dev);
                        println!(
                            "  Min/Max: {:.2} / {:.2}",
                            trend_analysis.min, trend_analysis.max
                        );
                        println!("  Slope: {:.2}/day", trend_analysis.slope);
                        println!("  Observations: {}", trend_analysis.count);

                        // Add recommendations for regressing metrics
                        if trend_analysis.direction == TrendDirection::Regressing {
                            let recommendations = Self::generate_metric_recommendations(
                                &metric_name,
                                trend_analysis.slope,
                            );
                            if !recommendations.is_empty() {
                                println!("  \x1b[1;33mRecommendations:\x1b[0m");
                                for rec in recommendations {
                                    println!("    - {}", rec);
                                }
                            }
                        }

                        println!();
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute record-metric command (Phase 3.4 O(1) Quality Gates - CI/CD)
    pub(crate) async fn execute_record_metric_command(
        metric: String,
        value: f64,
        timestamp: Option<i64>,
    ) -> anyhow::Result<()> {
        use crate::services::metric_trends::MetricTrendStore;

        let mut store = MetricTrendStore::new()?;

        // Use provided timestamp or current time
        let ts = timestamp.unwrap_or_else(|| chrono::Utc::now().timestamp());

        // Record the observation
        store.record(&metric, value, ts)?;

        println!("Recorded {} = {:.2} at timestamp {}", metric, value, ts);

        // Show quick stats
        if let Ok(trend_analysis) = store.trend(&metric, 30) {
            println!(
                "   Last 30 days: mean={:.2}, slope={:.2}/day",
                trend_analysis.mean, trend_analysis.slope
            );
        }

        Ok(())
    }

    /// Generate metric-specific recommendations
    pub(crate) fn generate_metric_recommendations(metric: &str, slope_per_day: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        let days_to_critical = match metric {
            "lint" => {
                let threshold = 30_000.0;
                let current_estimate = 26_500.0; // Approximate
                ((threshold - current_estimate) / slope_per_day).max(0.0)
            }
            "test-fast" => {
                let threshold = 300_000.0;
                let current_estimate = 107_000.0;
                ((threshold - current_estimate) / slope_per_day).max(0.0)
            }
            "coverage" => {
                let threshold = 600_000.0;
                let current_estimate = 480_000.0;
                ((threshold - current_estimate) / slope_per_day).max(0.0)
            }
            "build-release" => {
                let threshold = 900_000.0;
                let current_estimate = 717_000.0;
                ((threshold - current_estimate) / slope_per_day).max(0.0)
            }
            _ => f64::MAX,
        };

        if days_to_critical < 30.0 {
            recommendations.push(format!(
                "WARNING: Approaching threshold in ~{:.0} days",
                days_to_critical
            ));
        }

        match metric {
            "lint" => {
                recommendations.push("Remove unused dependencies (saves ~2-3s)".to_string());
                recommendations.push("Enable incremental clippy analysis".to_string());
                recommendations
                    .push("Review enabled lints (disable pedantic if not needed)".to_string());
            }
            "test-fast" => {
                recommendations.push("Add #[ignore] to slow integration tests".to_string());
                recommendations.push("Use proptest with reduced cases for CI".to_string());
                recommendations.push("Parallelize test execution with nextest".to_string());
            }
            "coverage" => {
                recommendations.push("Exclude slow tests from coverage run".to_string());
                recommendations.push("Use cargo-llvm-cov with --skip-functions flag".to_string());
                recommendations
                    .push("Consider sampling-based coverage for large projects".to_string());
            }
            "build-release" => {
                recommendations.push(
                    "Enable sccache with CARGO_INCREMENTAL=0 (required for cache hits)".to_string(),
                );
                recommendations.push(
                    "Use per-project target dirs (avoid shared CARGO_TARGET_DIR lock contention)"
                        .to_string(),
                );
                recommendations
                    .push("Review feature flags (disable optional features)".to_string());
                recommendations.push("Use mold/lld linker for faster linking".to_string());
            }
            _ => {}
        }

        recommendations
    }
}
