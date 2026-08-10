//! Metrics Command Handlers for CommandDispatcher
//!
//! Extracted from command_dispatcher mod.rs for file health compliance (CB-040).
//! Contains show-metrics and record-metric command execution.

#![cfg_attr(coverage_nightly, coverage(off))]

use super::CommandDispatcher;
use crate::cli::colors as c;
use crate::cli::OutputFormat;

impl CommandDispatcher {
    /// Execute show-metrics command (Phase 3.1 O(1) Quality Gates)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) async fn execute_show_metrics_command(
        trend: bool,
        days: usize,
        metric: Option<String>,
        format: OutputFormat,
        failures_only: bool,
    ) -> anyhow::Result<()> {
        use crate::services::metric_trends::{MetricTrendStore, TrendDirection};

        // `--trend` is the documented opt-in for the trend view (direction,
        // std dev, slope, recommendations). It used to be dropped here with
        // `let _ = trend`, so `show-metrics` and `show-metrics --trend`
        // produced byte-identical output and the flag meant nothing. Without
        // it the command reports the observations it holds, and nothing it
        // would need a regression to say.
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

                // Add trend analysis (only the observations without `--trend`)
                for metric_name in metrics {
                    if let Ok(trend_analysis) = store.trend(&metric_name, days) {
                        if failures_only && trend_analysis.direction != TrendDirection::Regressing {
                            continue;
                        }
                        let value = if trend {
                            serde_json::to_value(&trend_analysis)?
                        } else {
                            serde_json::json!({
                                "metric": trend_analysis.metric,
                                "count": trend_analysis.count,
                                "mean": trend_analysis.mean,
                                "min": trend_analysis.min,
                                "max": trend_analysis.max,
                            })
                        };
                        results.insert(metric_name, value);
                    }
                }
                println!("{}", serde_json::to_string_pretty(&results)?);
            }
            _ => {
                // Table output (default)
                let heading = if trend {
                    "Quality Metrics Trends"
                } else {
                    "Quality Metrics"
                };
                println!(
                    "\n{}{} ({} days){}\n",
                    c::BOLD_BLUE,
                    heading,
                    days,
                    c::RESET
                );

                // Show hot metrics ranking (PageRank)
                let hot_metrics = store.hot_metrics();
                if !hot_metrics.is_empty() {
                    println!("{}Hot Metrics (PageRank){}", c::BOLD_YELLOW, c::RESET);
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

                        print!(
                            "{}",
                            Self::render_metric_block(&metric_name, &trend_analysis, trend)
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Render one metric's block for the table view.
    ///
    /// `trend` is `show-metrics --trend`. The trend view is direction, spread,
    /// slope and the regression recommendations (that is what the workflow docs
    /// show `--trend` printing); without the flag the block states only what was
    /// observed. Both views used to be the same text, which is what made the
    /// flag inert.
    pub(crate) fn render_metric_block(
        metric_name: &str,
        analysis: &crate::services::metric_trends::TrendAnalysis,
        trend: bool,
    ) -> String {
        use crate::services::metric_trends::TrendDirection;
        use std::fmt::Write as _;

        let mut out = String::new();
        let _ = writeln!(out, "{}{}{}", c::BOLD, metric_name, c::RESET);

        if trend {
            let direction_symbol = match analysis.direction {
                TrendDirection::Improving => format!("{}Improving{}", c::GREEN, c::RESET),
                TrendDirection::Stable => format!("{}Stable{}", c::YELLOW, c::RESET),
                TrendDirection::Regressing => format!("{}Regressing{}", c::RED, c::RESET),
            };
            let _ = writeln!(out, "  Direction: {direction_symbol}");
        }

        let _ = writeln!(out, "  Mean: {:.2}", analysis.mean);
        if trend {
            let _ = writeln!(out, "  Std Dev: {:.2}", analysis.std_dev);
        }
        let _ = writeln!(out, "  Min/Max: {:.2} / {:.2}", analysis.min, analysis.max);
        if trend {
            let _ = writeln!(out, "  Slope: {:.2}/day", analysis.slope);
        }
        let _ = writeln!(out, "  Observations: {}", analysis.count);

        // Recommendations answer "this is regressing, now what" — part of the
        // trend report, not of a plain observation listing.
        if trend && analysis.direction == TrendDirection::Regressing {
            let recommendations =
                Self::generate_metric_recommendations(metric_name, analysis.slope);
            if !recommendations.is_empty() {
                let _ = writeln!(out, "  {}Recommendations:{}", c::BOLD_YELLOW, c::RESET);
                for rec in recommendations {
                    let _ = writeln!(out, "    - {rec}");
                }
            }
        }

        out.push('\n');
        out
    }

    /// Execute record-metric command (Phase 3.4 O(1) Quality Gates - CI/CD)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

#[cfg(test)]
mod trend_flag_tests {
    use super::*;
    use crate::services::metric_trends::{TrendAnalysis, TrendDirection};

    fn regressing_lint() -> TrendAnalysis {
        TrendAnalysis {
            metric: "lint".to_string(),
            count: 12,
            mean: 23_390.5,
            std_dev: 2156.3,
            min: 20_000.0,
            max: 25_500.0,
            direction: TrendDirection::Regressing,
            slope: 235.46,
            p_value: 0.01,
        }
    }

    /// `--trend` was `let _ = trend`: `show-metrics` and `show-metrics --trend`
    /// printed byte-identical reports, so the documented opt-in changed nothing.
    #[test]
    fn trend_flag_selects_the_trend_view() {
        let analysis = regressing_lint();
        let plain = CommandDispatcher::render_metric_block("lint", &analysis, false);
        let with_trend = CommandDispatcher::render_metric_block("lint", &analysis, true);

        assert_ne!(plain, with_trend, "--trend must change the report");

        for trend_only in ["Direction", "Std Dev", "Slope", "Recommendations"] {
            assert!(
                with_trend.contains(trend_only),
                "--trend must report {trend_only}"
            );
            assert!(
                !plain.contains(trend_only),
                "{trend_only} belongs to the trend view, not the default listing"
            );
        }
    }

    /// The observations themselves are not the trend report; both views state them.
    #[test]
    fn observations_are_reported_either_way() {
        let analysis = regressing_lint();
        for trend in [false, true] {
            let block = CommandDispatcher::render_metric_block("lint", &analysis, trend);
            assert!(block.contains("Observations: 12"));
            assert!(block.contains("Mean: 23390.50"));
            assert!(block.contains("Min/Max: 20000.00 / 25500.00"));
        }
    }
}
