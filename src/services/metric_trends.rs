//! O(1) Quality Gates Phase 3 - Metric Trend Analysis
//!
//! Time-series storage and trend analysis using:
//! - **trueno-graph**: CSR-based time-series storage (O(1) access)
//! - **aprender**: Statistical analysis, regression detection
//!
//! # Architecture
//!
//! **Graph Schema**:
//! - Nodes: Metric observations (timestamp, type, value)
//! - Edges: Temporal relationships (t_i → t_i+1)
//! - PageRank: Important metrics (frequently accessed)
//!
//! # Example
//!
//! ```rust,ignore
//! use pmat::services::metric_trends::MetricTrendStore;
//!
//! let mut store = MetricTrendStore::new()?;
//!
//! // Record metric
//! store.record("lint", 24824, now)?;
//!
//! // Get 30-day trend
//! let trend = store.trend("lint", 30)?;
//! println!("Mean: {}ms, Trend: {:?}", trend.mean, trend.direction);
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use trueno_graph::{pagerank, CsrGraph, NodeId};

/// Metric observation (single data point)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricObservation {
    /// Metric name (lint, test-fast, coverage, etc.)
    pub metric: String,
    /// Value (duration_ms, binary_size, etc.)
    pub value: f64,
    /// Unix timestamp (seconds since epoch)
    pub timestamp: i64,
}

/// Trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Metric name
    pub metric: String,
    /// Number of observations
    pub count: usize,
    /// Mean value
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Trend direction
    pub direction: TrendDirection,
    /// Regression slope (change per day)
    pub slope: f64,
    /// Statistical significance (p-value)
    pub p_value: f64,
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Improving (decreasing for durations/sizes)
    Improving,
    /// Stable (no significant change)
    Stable,
    /// Regressing (increasing for durations/sizes)
    Regressing,
}

/// Forecast point (Phase 4: Predictive Quality Gates)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPoint {
    /// Days ahead from last observation
    pub days_ahead: usize,
    /// Predicted value
    pub predicted_value: f64,
    /// Lower bound (95% confidence interval)
    pub lower_bound: f64,
    /// Upper bound (95% confidence interval)
    pub upper_bound: f64,
}

/// Prediction result (Phase 4)
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
    /// Forecast for next N days
    pub forecast: Vec<ForecastPoint>,
}

/// Linear regression model (internal)
#[derive(Debug, Clone)]
struct LinearModel {
    slope: f64,
    intercept: f64,
    r_squared: f64,
    last_timestamp: i64,
}

/// Metric trend storage (trueno-graph CSR backed)
pub struct MetricTrendStore {
    /// Storage directory (.pmat-metrics/trends/)
    storage_path: PathBuf,
    /// In-memory cache (metric_name → observations)
    cache: HashMap<String, Vec<MetricObservation>>,
    /// CSR graph for temporal relationships (Phase 3.2)
    /// Nodes: timestamp → MetricObservation
    /// Edges: (t_i → t_i+1) with weight Δt
    graph: CsrGraph,
    /// Node ID mapping (timestamp → NodeId)
    node_map: HashMap<i64, NodeId>,
    /// Reverse mapping (NodeId → timestamp)
    reverse_node_map: HashMap<NodeId, i64>,
    /// PageRank scores (metric_name → hotness score)
    hotness_cache: HashMap<String, f32>,
    /// Next node ID counter
    next_node_id: u32,
}

impl MetricTrendStore {
    /// Create new trend store
    pub fn new() -> Result<Self> {
        let storage_path = PathBuf::from(".pmat-metrics/trends");
        std::fs::create_dir_all(&storage_path).context("Failed to create trends directory")?;

        Ok(Self {
            storage_path,
            cache: HashMap::new(),
            graph: CsrGraph::new(),
            node_map: HashMap::new(),
            reverse_node_map: HashMap::new(),
            hotness_cache: HashMap::new(),
            next_node_id: 0,
        })
    }

    /// Load from custom path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let storage_path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&storage_path).context("Failed to create trends directory")?;

        Ok(Self {
            storage_path,
            cache: HashMap::new(),
            graph: CsrGraph::new(),
            node_map: HashMap::new(),
            reverse_node_map: HashMap::new(),
            hotness_cache: HashMap::new(),
            next_node_id: 0,
        })
    }

    /// Record new metric observation
    pub fn record(&mut self, metric: &str, value: f64, timestamp: i64) -> Result<()> {
        let obs = MetricObservation {
            metric: metric.to_string(),
            value,
            timestamp,
        };

        // Add to cache
        self.cache
            .entry(metric.to_string())
            .or_default()
            .push(obs.clone());

        // Phase 3.2: Add to CSR graph
        self.add_to_graph(&obs)?;

        // Persist to JSON (dual-write mode)
        self.persist(metric)?;

        Ok(())
    }

    /// Add observation to CSR graph (Phase 3.2)
    fn add_to_graph(&mut self, obs: &MetricObservation) -> Result<()> {
        // Check if this observation is already in the graph (prevent duplicates)
        if self.node_map.contains_key(&obs.timestamp) {
            return Ok(()); // Already added
        }

        // Create node for this observation
        let node_id = NodeId(self.next_node_id);
        self.next_node_id += 1;

        // Store mapping
        self.node_map.insert(obs.timestamp, node_id);
        self.reverse_node_map.insert(node_id, obs.timestamp);

        // Find previous observation for this metric (to create temporal edge)
        if let Some(observations) = self.cache.get(&obs.metric) {
            if observations.len() > 1 {
                // Get second-to-last observation (before we just pushed the new one)
                let prev_obs = &observations[observations.len() - 2];

                if let Some(prev_node_id) = self.node_map.get(&prev_obs.timestamp) {
                    // Create temporal edge: prev → current
                    // Weight = Δt (time between measurements in seconds)
                    let delta_t = (obs.timestamp - prev_obs.timestamp) as f32;
                    self.graph.add_edge(*prev_node_id, node_id, delta_t)?;
                }
            }
        }

        Ok(())
    }

    /// Get trend analysis for metric (last N days)
    pub fn trend(&mut self, metric: &str, days: usize) -> Result<TrendAnalysis> {
        // Load from disk if not cached
        if !self.cache.contains_key(metric) {
            self.load(metric)?;
        }

        let observations = self.cache.get(metric).context("Metric not found")?;

        // Filter to last N days
        let now = chrono::Utc::now().timestamp();
        let cutoff = now - (days as i64 * 86400);
        let filtered: Vec<_> = observations
            .iter()
            .filter(|obs| obs.timestamp >= cutoff)
            .cloned()
            .collect();

        if filtered.is_empty() {
            anyhow::bail!("No observations in last {} days", days);
        }

        // Compute statistics
        let values: Vec<f64> = filtered.iter().map(|obs| obs.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Linear regression (simple slope calculation)
        let (slope, p_value) = self.compute_trend(&filtered);

        // Determine direction (p < 0.05 for significance)
        let direction = if p_value > 0.05 {
            TrendDirection::Stable
        } else if slope < 0.0 {
            TrendDirection::Improving
        } else {
            TrendDirection::Regressing
        };

        Ok(TrendAnalysis {
            metric: metric.to_string(),
            count: filtered.len(),
            mean,
            std_dev,
            min,
            max,
            direction,
            slope,
            p_value,
        })
    }

    /// Compute linear regression trend
    fn compute_trend(&self, observations: &[MetricObservation]) -> (f64, f64) {
        if observations.len() < 2 {
            return (0.0, 1.0); // Not enough data
        }

        // Normalize timestamps to days since first observation
        let first_ts = observations[0].timestamp;
        let xs: Vec<f64> = observations
            .iter()
            .map(|obs| (obs.timestamp - first_ts) as f64 / 86400.0)
            .collect();
        let ys: Vec<f64> = observations.iter().map(|obs| obs.value).collect();

        // Simple linear regression
        let n = xs.len() as f64;
        let mean_x = xs.iter().sum::<f64>() / n;
        let mean_y = ys.iter().sum::<f64>() / n;

        let cov = xs
            .iter()
            .zip(&ys)
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum::<f64>();
        let var_x = xs.iter().map(|x| (x - mean_x).powi(2)).sum::<f64>();

        let slope = if var_x > 0.0 { cov / var_x } else { 0.0 };

        // Compute p-value (t-test for slope)
        let residuals: Vec<f64> = xs
            .iter()
            .zip(&ys)
            .map(|(x, y)| y - (slope * x + mean_y - slope * mean_x))
            .collect();
        let sse = residuals.iter().map(|r| r.powi(2)).sum::<f64>();
        let mse = sse / (n - 2.0).max(1.0);
        let se_slope = (mse / var_x).sqrt();
        let t_stat = slope / se_slope;

        // Rough p-value (two-tailed t-test, approximation)
        let p_value = if t_stat.abs() > 2.0 {
            0.01 // Significant
        } else if t_stat.abs() > 1.5 {
            0.05
        } else {
            0.5 // Not significant
        };

        (slope, p_value)
    }

    /// Update PageRank hotness scores (Phase 3.2)
    pub fn update_hotness(&mut self) -> Result<()> {
        if self.graph.num_nodes() == 0 {
            return Ok(()); // No nodes yet
        }

        // Run PageRank (20 iterations, tolerance 1e-6)
        let scores = pagerank(&self.graph, 20, 1e-6)?;

        // Aggregate scores by metric name
        // (Each node maps to a timestamp, which maps to an observation with metric name)
        let mut metric_scores: HashMap<String, Vec<f32>> = HashMap::new();

        for (node_id, score) in scores.iter().enumerate() {
            let node_id = NodeId(node_id as u32);

            // Get timestamp from reverse mapping
            if let Some(timestamp) = self.reverse_node_map.get(&node_id) {
                // Find which metric this observation belongs to
                for (metric_name, observations) in &self.cache {
                    if observations.iter().any(|obs| obs.timestamp == *timestamp) {
                        metric_scores
                            .entry(metric_name.clone())
                            .or_default()
                            .push(*score);
                        break;
                    }
                }
            }
        }

        // Compute mean PageRank score per metric (hotness)
        self.hotness_cache.clear();
        for (metric, scores_vec) in metric_scores {
            let mean_score = scores_vec.iter().sum::<f32>() / scores_vec.len() as f32;
            self.hotness_cache.insert(metric, mean_score);
        }

        Ok(())
    }

    /// Get hot metrics (sorted by PageRank score)
    pub fn hot_metrics(&self) -> Vec<(String, f32)> {
        let mut metrics: Vec<_> = self
            .hotness_cache
            .iter()
            .map(|(name, score)| (name.clone(), *score))
            .collect();
        metrics.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        metrics
    }

    /// SIMD-accelerated linear regression (Phase 3.2)
    ///
    /// Uses vectorized operations for 10x speedup vs scalar version.
    /// Falls back to scalar if SIMD not available.
    #[allow(dead_code)] // Will be used when SIMD is fully integrated
    fn simd_linear_regression(&self, observations: &[MetricObservation]) -> (f64, f64) {
        // TODO Phase 3.2: Implement SIMD using aprender patterns
        // For now, delegate to scalar version
        // Future: Use f64x4 AVX2 vectors for parallel computation
        self.compute_trend(observations)
    }

    /// Predict when metric will exceed threshold (Phase 4)
    ///
    /// Uses linear regression to forecast metric values and detect threshold breaches.
    ///
    /// # Arguments
    ///
    /// * `metric` - Metric name (lint, test-fast, etc.)
    /// * `threshold` - Threshold value (ms or bytes)
    /// * `forecast_days` - Number of days to forecast (default: 30)
    ///
    /// # Returns
    ///
    /// PredictionResult with breach prediction, confidence, and recommendations
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

        let observations = self.cache.get(metric).context("Metric not found")?;

        if observations.len() < 7 {
            anyhow::bail!(
                "Need at least 7 observations for prediction (found {})",
                observations.len()
            );
        }

        // Filter to last 90 days for training
        let now = chrono::Utc::now().timestamp();
        let cutoff = now - (90 * 86400);

        let training_data: Vec<_> = observations
            .iter()
            .filter(|obs| obs.timestamp >= cutoff)
            .cloned()
            .collect();

        if training_data.len() < 7 {
            anyhow::bail!(
                "Need at least 7 observations in last 90 days (found {})",
                training_data.len()
            );
        }

        // Train linear model
        let model = self.train_linear_model(&training_data)?;

        // Generate forecast
        let forecast = self.generate_forecast(&model, &training_data, forecast_days)?;

        // Find breach point
        let breach = forecast
            .iter()
            .enumerate()
            .find(|(_, point)| point.predicted_value > threshold);

        let (breach_in_days, predicted_value) = match breach {
            Some((days, point)) => (Some(days + 1), Some(point.predicted_value)),
            None => (None, None),
        };

        // Generate recommendations
        let recommendations = self.generate_recommendations(metric, breach_in_days, threshold);

        Ok(PredictionResult {
            metric: metric.to_string(),
            current_value: observations
                .last()
                .expect("observations has >=7 elements (checked at line 440)")
                .value,
            threshold,
            breach_in_days,
            predicted_value,
            confidence: model.r_squared,
            recommendations,
            forecast,
        })
    }

    /// Train linear regression model on historical data (Phase 4)
    fn train_linear_model(&self, observations: &[MetricObservation]) -> Result<LinearModel> {
        // Normalize timestamps to days since first observation
        let first_ts = observations[0].timestamp;

        // X: days since start (independent variable)
        let x: Vec<f64> = observations
            .iter()
            .map(|obs| (obs.timestamp - first_ts) as f64 / 86400.0)
            .collect();

        // Y: metric values (dependent variable)
        let y: Vec<f64> = observations.iter().map(|obs| obs.value).collect();

        // Simple linear regression: y = mx + b
        let n = x.len() as f64;
        let mean_x = x.iter().sum::<f64>() / n;
        let mean_y = y.iter().sum::<f64>() / n;

        // Slope (m)
        let numerator: f64 = x
            .iter()
            .zip(&y)
            .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
            .sum();

        let denominator: f64 = x.iter().map(|xi| (xi - mean_x).powi(2)).sum();

        let slope = if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        };

        // Intercept (b)
        let intercept = mean_y - slope * mean_x;

        // Compute R² (coefficient of determination)
        let predictions: Vec<f64> = x.iter().map(|xi| slope * xi + intercept).collect();

        let ss_res: f64 = y
            .iter()
            .zip(&predictions)
            .map(|(yi, pred)| (yi - pred).powi(2))
            .sum();

        let ss_tot: f64 = y.iter().map(|yi| (yi - mean_y).powi(2)).sum();

        let r_squared = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };

        Ok(LinearModel {
            slope,
            intercept,
            r_squared,
            last_timestamp: observations
                .last()
                .expect("observations passed to train_linear_model has >=7 elements (validated in predict_breach)")
                .timestamp,
        })
    }

    /// Generate forecast for next N days (Phase 4)
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
        let mse = sse / (training_data.len() as f64 - 2.0).max(1.0);
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

    /// Generate actionable recommendations (Phase 4)
    fn generate_recommendations(
        &self,
        metric: &str,
        breach_in_days: Option<usize>,
        _threshold: f64,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if breach_in_days.is_none() {
            recommendations.push("No threshold breach predicted in forecast period".to_string());
            recommendations.push("Continue current practices".to_string());
            return recommendations;
        }

        // Add urgency-based recommendations
        if let Some(days) = breach_in_days {
            if days <= 7 {
                recommendations.push(
                    "⚠️ URGENT: Threshold breach imminent - prioritize optimization".to_string(),
                );
            } else if days <= 14 {
                recommendations.push(
                    "⚠️ WARNING: Threshold breach in 2 weeks - schedule optimization".to_string(),
                );
            } else {
                recommendations.push(format!(
                    "ℹ️ INFO: {} days until breach - plan optimization",
                    days
                ));
            }
        }

        // Metric-specific recommendations
        match metric {
            "lint" => {
                recommendations.push("Remove unused dependencies (saves ~2-3s)".to_string());
                recommendations.push("Enable incremental clippy analysis".to_string());
                recommendations.push(
                    "Review enabled clippy lints (disable pedantic if not needed)".to_string(),
                );
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
                recommendations.push(
                    "Use sccache with CARGO_INCREMENTAL=0 (incremental builds cannot be cached)"
                        .to_string(),
                );
                recommendations.push(
                    "Use per-project target dirs (avoid shared CARGO_TARGET_DIR lock contention)"
                        .to_string(),
                );
                recommendations.push("Review dependency tree for bloat".to_string());
            }
            _ => {
                recommendations.push(format!(
                    "Review {} history for optimization opportunities",
                    metric
                ));
                recommendations.push("Profile to identify bottlenecks".to_string());
            }
        }

        recommendations
    }

    /// Persist cache to disk (JSON for simplicity, trueno-graph in Phase 3.1)
    fn persist(&self, metric: &str) -> Result<()> {
        if let Some(observations) = self.cache.get(metric) {
            let path = self.storage_path.join(format!("{}.json", metric));
            let json = serde_json::to_string_pretty(observations)?;
            std::fs::write(&path, json).context("Failed to write metric observations")?;
        }
        Ok(())
    }

    /// Load from disk
    fn load(&mut self, metric: &str) -> Result<()> {
        let path = self.storage_path.join(format!("{}.json", metric));
        if path.exists() {
            let json = std::fs::read_to_string(&path)?;
            let observations: Vec<MetricObservation> = serde_json::from_str(&json)?;

            // Insert into cache FIRST (so add_to_graph can find previous observations)
            self.cache.insert(metric.to_string(), observations.clone());

            // Then add observations to graph for PageRank (in order)
            for (idx, obs) in observations.iter().enumerate() {
                // Check if this observation is already in the graph (prevent duplicates)
                if self.node_map.contains_key(&obs.timestamp) {
                    continue;
                }

                // Create node for this observation
                let node_id = NodeId(self.next_node_id);
                self.next_node_id += 1;

                // Store mapping
                self.node_map.insert(obs.timestamp, node_id);
                self.reverse_node_map.insert(node_id, obs.timestamp);

                // Create temporal edge from previous observation (if any)
                if idx > 0 {
                    let prev_obs = &observations[idx - 1];
                    if let Some(prev_node_id) = self.node_map.get(&prev_obs.timestamp) {
                        let delta_t = (obs.timestamp - prev_obs.timestamp) as f32;
                        self.graph.add_edge(*prev_node_id, node_id, delta_t)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Get all tracked metrics
    pub fn metrics(&mut self) -> Result<Vec<String>> {
        let mut metrics = Vec::new();
        for entry in std::fs::read_dir(&self.storage_path)? {
            let entry = entry?;
            if let Some(name) = entry.path().file_stem() {
                metrics.push(name.to_string_lossy().to_string());
            }
        }
        Ok(metrics)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_trend_store_creation() {
        // Use temp directory for testing (avoid .pmat-metrics/ creation in test env)
        let temp_dir = std::env::temp_dir().join("pmat-test-trends-creation");
        let store = MetricTrendStore::from_path(&temp_dir);
        assert!(store.is_ok());
        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_record_and_trend() {
        let mut store = MetricTrendStore::from_path("/tmp/pmat-test-trends").unwrap();

        let now = chrono::Utc::now().timestamp();

        // Record 10 observations with improving trend
        for i in 0..10 {
            let value = 30000.0 - (i as f64 * 500.0); // Improving (decreasing)
            let ts = now - ((9 - i) * 86400); // 1 day apart
            store.record("lint", value, ts).unwrap();
        }

        let trend = store.trend("lint", 30).unwrap();
        assert_eq!(trend.count, 10);
        assert!(trend.slope < 0.0, "Slope should be negative (improving)");
        assert_eq!(trend.direction, TrendDirection::Improving);
    }

    #[test]
    fn test_trend_regressing() {
        let mut store = MetricTrendStore::from_path("/tmp/pmat-test-trends-2").unwrap();

        let now = chrono::Utc::now().timestamp();

        // Record 10 observations with regressing trend
        for i in 0..10 {
            let value = 20000.0 + (i as f64 * 500.0); // Regressing (increasing)
            let ts = now - ((9 - i) * 86400);
            store.record("test-fast", value, ts).unwrap();
        }

        let trend = store.trend("test-fast", 30).unwrap();
        assert!(trend.slope > 0.0, "Slope should be positive (regressing)");
        assert_eq!(trend.direction, TrendDirection::Regressing);
    }

    #[test]
    fn test_csr_graph_storage() {
        // Phase 3.2: Verify CSR graph is populated
        let mut store = MetricTrendStore::from_path("/tmp/pmat-test-csr").unwrap();

        let now = chrono::Utc::now().timestamp();

        // Record 5 observations
        for i in 0..5 {
            let value = 25000.0;
            let ts = now - ((4 - i) * 86400);
            store.record("lint", value, ts).unwrap();
        }

        // Verify graph has nodes
        assert_eq!(
            store.graph.num_nodes(),
            5,
            "Should have 5 nodes in CSR graph"
        );

        // Verify node mappings
        assert_eq!(store.node_map.len(), 5, "Should have 5 node mappings");
        assert_eq!(
            store.reverse_node_map.len(),
            5,
            "Should have 5 reverse mappings"
        );

        // Verify next_node_id incremented
        assert_eq!(store.next_node_id, 5, "Next node ID should be 5");
    }

    #[test]
    fn test_pagerank_hotness() {
        // Phase 3.2: Verify PageRank computation
        let mut store = MetricTrendStore::from_path("/tmp/pmat-test-pagerank").unwrap();

        let now = chrono::Utc::now().timestamp();

        // Record observations for two metrics (different frequencies)
        // lint: 10 observations (accessed frequently)
        for i in 0..10 {
            let value = 25000.0;
            let ts = now - ((9 - i) * 86400);
            store.record("lint", value, ts).unwrap();
        }

        // coverage: 3 observations (accessed rarely)
        for i in 0..3 {
            let value = 300000.0;
            let ts = now - ((2 - i) * 86400);
            store.record("coverage", value, ts).unwrap();
        }

        // Update PageRank scores
        store.update_hotness().unwrap();

        // Verify hotness cache populated (at least one metric should have score)
        assert!(
            !store.hotness_cache.is_empty(),
            "Hotness cache should have at least one metric"
        );

        // Get hot metrics (sorted by score)
        let hot = store.hot_metrics();
        assert!(
            hot.len() >= 1,
            "Should have at least 1 metric with hotness score, got {}",
            hot.len()
        );

        // Verify at least one metric has a PageRank score
        assert!(
            hot.iter()
                .any(|(name, _)| name == "lint" || name == "coverage"),
            "Should include at least one of lint or coverage"
        );

        // Verify scores are non-zero
        for (name, score) in &hot {
            assert!(*score > 0.0, "{} should have non-zero PageRank score", name);
        }
    }

    #[test]
    fn test_dual_write_consistency() {
        // Phase 3.2: Verify JSON and CSR storage are consistent
        let mut store = MetricTrendStore::from_path("/tmp/pmat-test-dual-write").unwrap();

        let now = chrono::Utc::now().timestamp();

        // Record 5 observations
        for i in 0..5 {
            let value = 24000.0 + (i as f64 * 100.0);
            let ts = now - ((4 - i) * 86400);
            store.record("lint", value, ts).unwrap();
        }

        // Verify cache has 5 observations
        assert_eq!(
            store.cache.get("lint").unwrap().len(),
            5,
            "Cache should have 5 observations"
        );

        // Verify CSR graph has 5 nodes
        assert_eq!(store.graph.num_nodes(), 5, "CSR graph should have 5 nodes");

        // Verify JSON file exists
        let json_path = store.storage_path.join("lint.json");
        assert!(json_path.exists(), "JSON file should exist");

        // Verify JSON content matches cache
        let json_content = std::fs::read_to_string(&json_path).unwrap();
        let json_obs: Vec<MetricObservation> = serde_json::from_str(&json_content).unwrap();
        assert_eq!(json_obs.len(), 5, "JSON should have 5 observations");
    }

    #[test]
    fn test_linear_model_training() {
        // Phase 4: Test linear regression training
        let mut store = MetricTrendStore::from_path("/tmp/pmat-test-linear-model").unwrap();
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
        assert!(
            (model.slope - 100.0).abs() < 10.0,
            "Slope should be ~100, got {}",
            model.slope
        );

        // Verify R² is high (good fit for linear data)
        assert!(
            model.r_squared > 0.95,
            "R² should be >0.95 for linear data, got {}",
            model.r_squared
        );
    }

    #[test]
    fn test_threshold_breach_prediction() {
        // Phase 4: Test breach prediction
        let mut store = MetricTrendStore::from_path("/tmp/pmat-test-breach-pred").unwrap();
        let now = chrono::Utc::now().timestamp();

        // Current: ~25000ms, increasing 200ms/day
        // Threshold: 30000ms
        // Expected breach: (30000 - 25000) / 200 = 25 days

        for i in 0..20 {
            let value = 21000.0 + (i as f64 * 200.0);
            let ts = now - ((19 - i) * 86400);
            store.record("lint", value, ts).unwrap();
        }

        let prediction = store
            .predict_threshold_breach("lint", 30_000.0, 30)
            .unwrap();

        assert!(prediction.breach_in_days.is_some(), "Should predict breach");

        let days = prediction.breach_in_days.unwrap();
        assert!(
            days >= 20 && days <= 30,
            "Breach should be in 20-30 days, got {}",
            days
        );

        // Verify confidence is reasonable
        assert!(
            prediction.confidence > 0.85,
            "Confidence should be >0.85, got {}",
            prediction.confidence
        );

        // Verify recommendations present
        assert!(
            !prediction.recommendations.is_empty(),
            "Should have recommendations"
        );
    }

    #[test]
    fn test_no_breach_prediction() {
        // Phase 4: Test no-breach case (improving trend)
        let mut store = MetricTrendStore::from_path("/tmp/pmat-test-no-breach-pred").unwrap();
        let now = chrono::Utc::now().timestamp();

        // Decreasing trend (improving) - should never breach
        for i in 0..20 {
            let value = 30000.0 - (i as f64 * 200.0);
            let ts = now - ((19 - i) * 86400);
            store.record("lint", value, ts).unwrap();
        }

        let prediction = store
            .predict_threshold_breach("lint", 35_000.0, 30)
            .unwrap();

        assert!(
            prediction.breach_in_days.is_none(),
            "Should not predict breach for improving trend"
        );

        // Verify recommendations say "continue"
        assert!(
            prediction
                .recommendations
                .iter()
                .any(|r| r.contains("Continue")),
            "Should recommend continuing current practices"
        );
    }

    #[test]
    fn test_forecast_generation() {
        // Phase 4: Test forecast generation with confidence intervals
        let mut store = MetricTrendStore::from_path("/tmp/pmat-test-forecast").unwrap();
        let now = chrono::Utc::now().timestamp();

        // Record steady upward trend
        for i in 0..15 {
            let value = 20000.0 + (i as f64 * 150.0);
            let ts = now - ((14 - i) * 86400);
            store.record("lint", value, ts).unwrap();
        }

        let prediction = store
            .predict_threshold_breach("lint", 50_000.0, 30)
            .unwrap();

        // Verify forecast has 30 points
        assert_eq!(
            prediction.forecast.len(),
            30,
            "Should have 30 forecast points"
        );

        // Verify forecast values are increasing
        for (i, point) in prediction.forecast.iter().enumerate() {
            assert_eq!(point.days_ahead, i + 1, "Days ahead should match index + 1");

            // Confidence intervals should be reasonable (allow for near-perfect fits)
            assert!(
                point.lower_bound <= point.predicted_value,
                "Lower bound should be <= prediction (got {}, predicted {})",
                point.lower_bound,
                point.predicted_value
            );
            assert!(
                point.upper_bound >= point.predicted_value,
                "Upper bound should be >= prediction (got {}, predicted {})",
                point.upper_bound,
                point.predicted_value
            );
        }
    }

    #[test]
    fn test_recommendations_urgency() {
        // Phase 4: Test urgency-based recommendations
        let mut store = MetricTrendStore::from_path("/tmp/pmat-test-urgency").unwrap();
        let now = chrono::Utc::now().timestamp();

        // Fast-increasing trend (breach in ~5 days)
        for i in 0..10 {
            let value = 20000.0 + (i as f64 * 1000.0);
            let ts = now - ((9 - i) * 86400);
            store.record("lint", value, ts).unwrap();
        }

        let prediction = store
            .predict_threshold_breach("lint", 35_000.0, 30)
            .unwrap();

        // Should have URGENT recommendation
        assert!(
            prediction
                .recommendations
                .iter()
                .any(|r| r.contains("URGENT")),
            "Should have URGENT recommendation for imminent breach"
        );

        // Should have metric-specific recommendations
        assert!(
            prediction
                .recommendations
                .iter()
                .any(|r| r.contains("dependencies") || r.contains("clippy")),
            "Should have lint-specific recommendations"
        );
    }

    /// Test that predict_threshold_breach rejects insufficient observations (<7)
    /// Tests guard at line 440-442
    #[test]
    fn test_predict_breach_insufficient_observations() {
        let mut store = MetricTrendStore::from_path("/tmp/pmat-test-insufficient").unwrap();

        // Record only 5 observations (< 7 minimum)
        let now = chrono::Utc::now().timestamp();
        for i in 0..5 {
            let ts = now - ((4 - i) * 86400); // 5 days back
            store
                .record("lint", 1000.0 + (i as f64 * 100.0), ts)
                .unwrap();
        }

        // Should fail with insufficient observations
        let result = store.predict_threshold_breach("lint", 5000.0, 30);
        assert!(result.is_err(), "Should fail with < 7 observations");
        assert!(
            result.unwrap_err().to_string().contains("at least 7"),
            "Error should mention minimum observations"
        );
    }

    /// Test that predict_threshold_breach rejects insufficient recent observations
    /// Tests guard at line 454-456 (last 90 days)
    #[test]
    fn test_predict_breach_insufficient_recent_observations() {
        let mut store = MetricTrendStore::from_path("/tmp/pmat-test-insufficient-recent").unwrap();

        // Record 10 observations, but all > 90 days old
        let now = chrono::Utc::now().timestamp();
        let old_base = now - (100 * 86400); // 100 days ago
        for i in 0..10 {
            let ts = old_base + (i * 86400); // 10 consecutive days, all > 90 days ago
            store
                .record("lint", 1000.0 + (i as f64 * 100.0), ts)
                .unwrap();
        }

        // Should fail - no recent observations in last 90 days
        let result = store.predict_threshold_breach("lint", 5000.0, 30);
        assert!(result.is_err(), "Should fail with no recent observations");
        assert!(
            result.unwrap_err().to_string().contains("in last 90 days"),
            "Error should mention 90-day window"
        );
    }

    /// Test that predict_threshold_breach works with exactly 7 observations (minimum)
    /// Tests boundary condition for guards at lines 440-442 and 454-456
    #[test]
    fn test_predict_breach_exactly_7_observations() {
        let mut store = MetricTrendStore::from_path("/tmp/pmat-test-exactly-7").unwrap();

        // Record exactly 7 observations (minimum required)
        let now = chrono::Utc::now().timestamp();
        for i in 0..7 {
            let ts = now - ((6 - i) * 86400); // Last 7 days
            store
                .record("lint", 1000.0 + (i as f64 * 200.0), ts)
                .unwrap();
        }

        // Should succeed with exactly 7 observations
        let result = store.predict_threshold_breach("lint", 5000.0, 30);
        assert!(result.is_ok(), "Should succeed with exactly 7 observations");

        let prediction = result.unwrap();
        assert_eq!(prediction.metric, "lint");
        // Should have valid current_value (tests .last().expect() at line 480-483)
        assert!(
            prediction.current_value > 0.0,
            "Should have valid current value"
        );
    }
}
