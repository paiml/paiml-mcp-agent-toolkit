#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
//! Sprint 31 Week 2 - Advanced Metrics Aggregation and Trending
//!
//! Provides sophisticated metrics aggregation, time-series analysis, and trending
//! capabilities for the TDG system. Supports rolling windows, percentile calculations,
//! and anomaly detection.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

// --- Type definitions: DataPoint, RollingWindow, AggregatedStats, metric point structs,
//     AlertThresholds, Alert, AlertSeverity, ExportFormat ---
include!("metrics_aggregator_types.rs");

// --- MetricsAggregator struct, Default impl, and all methods ---
include!("metrics_aggregator_impl.rs");

// --- Unit tests: type and data structure tests ---
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    include!("metrics_aggregator_tests.rs");

    // --- Async integration tests: MetricsAggregator, alerts, export, anomalies ---
    include!("metrics_aggregator_tests_async.rs");
}

// --- Property-based tests ---
