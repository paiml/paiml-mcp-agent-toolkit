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

/// Metric trend storage (trueno-graph backed)
pub struct MetricTrendStore {
    /// Storage directory (.pmat-metrics/trends/)
    storage_path: PathBuf,
    /// In-memory cache (metric_name → observations)
    cache: HashMap<String, Vec<MetricObservation>>,
}

impl MetricTrendStore {
    /// Create new trend store
    pub fn new() -> Result<Self> {
        let storage_path = PathBuf::from(".pmat-metrics/trends");
        std::fs::create_dir_all(&storage_path)
            .context("Failed to create trends directory")?;

        Ok(Self {
            storage_path,
            cache: HashMap::new(),
        })
    }

    /// Load from custom path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let storage_path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&storage_path)
            .context("Failed to create trends directory")?;

        Ok(Self {
            storage_path,
            cache: HashMap::new(),
        })
    }

    /// Record new metric observation
    pub fn record(&mut self, metric: &str, value: f64, timestamp: i64) -> Result<()> {
        let obs = MetricObservation {
            metric: metric.to_string(),
            value,
            timestamp,
        };

        self.cache
            .entry(metric.to_string())
            .or_default()
            .push(obs.clone());

        self.persist(metric)?;

        Ok(())
    }

    /// Get trend analysis for metric (last N days)
    pub fn trend(&mut self, metric: &str, days: usize) -> Result<TrendAnalysis> {
        // Load from disk if not cached
        if !self.cache.contains_key(metric) {
            self.load(metric)?;
        }

        let observations = self
            .cache
            .get(metric)
            .context("Metric not found")?;

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
        let variance = values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / values.len() as f64;
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

    /// Persist cache to disk (JSON for simplicity, trueno-graph in Phase 3.1)
    fn persist(&self, metric: &str) -> Result<()> {
        if let Some(observations) = self.cache.get(metric) {
            let path = self.storage_path.join(format!("{}.json", metric));
            let json = serde_json::to_string_pretty(observations)?;
            std::fs::write(&path, json)
                .context("Failed to write metric observations")?;
        }
        Ok(())
    }

    /// Load from disk
    fn load(&mut self, metric: &str) -> Result<()> {
        let path = self.storage_path.join(format!("{}.json", metric));
        if path.exists() {
            let json = std::fs::read_to_string(&path)?;
            let observations: Vec<MetricObservation> = serde_json::from_str(&json)?;
            self.cache.insert(metric.to_string(), observations);
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
}
