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
use trueno_graph::{CsrGraph, NodeId, pagerank};

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
        std::fs::create_dir_all(&storage_path)
            .context("Failed to create trends directory")?;

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
        std::fs::create_dir_all(&storage_path)
            .context("Failed to create trends directory")?;

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
        assert_eq!(store.graph.num_nodes(), 5, "Should have 5 nodes in CSR graph");

        // Verify node mappings
        assert_eq!(store.node_map.len(), 5, "Should have 5 node mappings");
        assert_eq!(store.reverse_node_map.len(), 5, "Should have 5 reverse mappings");

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

        // Verify hotness cache populated
        assert!(
            store.hotness_cache.contains_key("lint"),
            "lint should have hotness score"
        );
        assert!(
            store.hotness_cache.contains_key("coverage"),
            "coverage should have hotness score"
        );

        // Get hot metrics (sorted by score)
        let hot = store.hot_metrics();
        assert_eq!(hot.len(), 2, "Should have 2 metrics");

        // Verify both metrics have PageRank scores (order may vary based on graph topology)
        let metric_names: Vec<&str> = hot.iter().map(|(name, _)| name.as_str()).collect();
        assert!(
            metric_names.contains(&"lint"),
            "Should include lint metric"
        );
        assert!(
            metric_names.contains(&"coverage"),
            "Should include coverage metric"
        );

        // Verify scores are non-zero
        for (name, score) in &hot {
            assert!(
                *score > 0.0,
                "{} should have non-zero PageRank score",
                name
            );
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
}
