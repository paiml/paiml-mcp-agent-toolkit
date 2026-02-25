#![cfg_attr(coverage_nightly, coverage(off))]
//! PDCA (Plan-Do-Check-Act) Execution Loop
//!
//! Implements the core improvement cycle with CITL learning.

use super::signal_collector::AggregatedCollector;
use super::types::*;
use anyhow::{bail, Result};
use std::path::Path;

/// Result of a PDCA iteration
#[derive(Debug, Clone)]
pub struct PdcaIterationResult {
    pub iteration: usize,
    pub defects_found: usize,
    pub defects_fixed: usize,
    pub defects_skipped: usize,
    pub metrics_before: ProjectMetrics,
    pub metrics_after: ProjectMetrics,
    pub converged: bool,
}

/// PDCA Loop executor
pub struct PdcaLoop {
    config: OracleConfig,
    targets: ConvergenceTargets,
    collector: AggregatedCollector,
}

// Core PDCA loop execution: construction, run loop, metrics, fixes, regression, progress
include!("pdca_loop_execution.rs");

// Tests for PDCA loop functionality
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
include!("pdca_loop_tests.rs");
