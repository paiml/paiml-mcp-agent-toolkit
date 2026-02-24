#![cfg_attr(coverage_nightly, coverage(off))]
//! Data Science Integration for PMAT-REPORT-V1
//!
//! Integrates existing algorithms:
//! - K-Means clustering (aprender)
//! - PageRank centrality (trueno-graph)
//! - Louvain community detection (aprender)
//! - Isolation Forest (simplified implementation)
//! - Time series analysis (native)

use super::types::{
    AnomalyPoint, CodeCommunity, Finding, FindingCluster, MetricTrend, TrendDirection,
};
use aprender::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

/// Data science analyzer for rich reporting
pub struct DataScienceAnalyzer {
    /// Number of clusters for K-means
    k_clusters: usize,
    /// PageRank damping factor (reserved for future PageRank tuning)
    #[allow(dead_code)]
    pagerank_damping: f64,
    /// Louvain resolution (reserved for future Louvain tuning)
    #[allow(dead_code)]
    louvain_resolution: f64,
    /// Anomaly threshold
    anomaly_threshold: f64,
}

impl Default for DataScienceAnalyzer {
    fn default() -> Self {
        DataScienceAnalyzer {
            k_clusters: 4,
            pagerank_damping: 0.85,
            louvain_resolution: 1.0,
            anomaly_threshold: 0.7,
        }
    }
}

impl DataScienceAnalyzer {
    /// Create a new analyzer with custom parameters
    pub fn new(
        k_clusters: usize,
        pagerank_damping: f64,
        louvain_resolution: f64,
        anomaly_threshold: f64,
    ) -> Self {
        DataScienceAnalyzer {
            k_clusters,
            pagerank_damping,
            louvain_resolution,
            anomaly_threshold,
        }
    }
}

// K-means clustering: cluster_findings, finding_to_features, build_cluster_summaries
include!("data_science_clustering.rs");

// PageRank centrality and Louvain community detection
include!("data_science_graph.rs");

// Z-score anomaly detection: detect_anomalies, explain_anomaly, suggest_anomaly_action
include!("data_science_anomaly.rs");

// Trend analysis: analyze_trends, calculate_trend_direction, values_to_sparkline, forecast_next
include!("data_science_trends.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    include!("data_science_tests.rs");
}
