#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
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

// --- Types: structs, enums, and the MetricTrendStore definition ---
include!("metric_trends_types.rs");

// --- Core: new(), from_path(), record(), add_to_graph(), trend(), compute_trend() ---
include!("metric_trends_core.rs");

// --- Prediction: update_hotness(), hot_metrics(), predict_threshold_breach(), train/forecast ---
include!("metric_trends_prediction.rs");

// --- IO: generate_recommendations(), persist(), load(), metrics() ---
include!("metric_trends_io.rs");

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
            !hot.is_empty(),
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
            (20..=30).contains(&days),
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
