#![allow(unused)]
//! Graph metrics analysis - calculates centrality and other graph metrics
//! Uses a local SimpleGraph implementation (no petgraph dependency)
//!
//! Split into submodules:
//! - graph_metrics_types.rs: NodeIndex, SimpleGraph, NodeMetrics, GraphMetricsResult
//! - graph_metrics_handler.rs: CLI handler, file collection, dependency extraction
//! - graph_metrics_algorithms.rs: Centrality, PageRank, filtering algorithms
//! - graph_metrics_export.rs: GraphML export and output formatting

#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// Types: NodeIndex, SimpleGraph, NodeMetrics, GraphMetricsResult
include!("graph_metrics_types.rs");

// Handler: handle_analyze_graph_metrics, file collection, dependency extraction
include!("graph_metrics_handler.rs");

// Algorithms: calculate_metrics, centrality, PageRank, filtering
include!("graph_metrics_algorithms.rs");

// Export: GraphML export and output formatting (JSON, Human, CSV, Markdown)
include!("graph_metrics_export.rs");

// Tests extracted to graph_metrics_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "graph_metrics_tests.rs"]
mod tests;
