#![cfg_attr(coverage_nightly, coverage(off))]
//! Convergence criteria and status tracking
//!
//! Implements quality gates for the "perfect" project state.

use super::types::*;
use serde::{Deserialize, Serialize};

/// Convergence tracker for monitoring progress
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConvergenceTracker {
    pub iterations: usize,
    pub history: Vec<ConvergenceSnapshot>,
    pub best_metrics: Option<ProjectMetrics>,
    pub current_status: Option<ConvergenceStatus>,
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceSnapshot {
    pub iteration: usize,
    pub metrics: ProjectMetrics,
    pub defects_remaining: usize,
    pub status: ConvergenceStatus,
}

// --- Implementation ---
include!("convergence_tracker.rs");

// --- Tests ---
include!("convergence_tests.rs");
