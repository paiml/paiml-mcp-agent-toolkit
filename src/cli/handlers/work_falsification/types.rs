#![cfg_attr(coverage_nightly, coverage(off))]
//! Types and data structures for the falsification system.

use crate::cli::handlers::work_contract::{
    FalsificationMethod, FalsificationResult,
};
use serde::{Deserialize, Serialize};

/// Cache staleness thresholds (per spec v2.7)
pub(crate) const CACHE_WARN_HOURS: i64 = 1;
pub(crate) const CACHE_BLOCK_HOURS: i64 = 24;

/// Cached metric status
#[derive(Debug)]
pub(crate) struct CachedMetric {
    pub(crate) value: serde_json::Value,
    pub(crate) age_minutes: i64,
    pub(crate) is_stale_warn: bool,
    pub(crate) is_stale_block: bool,
}

/// Result of running all falsification tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationReport {
    /// Total number of claims tested
    pub total_claims: usize,

    /// Number of claims that passed (survived falsification)
    pub passed: usize,

    /// Number of claims that failed (were falsified)
    pub failed: usize,

    /// Number of warnings (non-blocking)
    pub warnings: usize,

    /// Individual claim results
    pub claim_results: Vec<ClaimResult>,

    /// Overall pass/fail
    pub all_passed: bool,
}

/// Result of testing a single claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimResult {
    /// Claim index (1-based for display)
    pub index: usize,

    /// Hypothesis being tested
    pub hypothesis: String,

    /// Method used for falsification
    pub method: FalsificationMethod,

    /// Result of the falsification attempt
    pub result: FalsificationResult,

    /// Is this a blocking failure or just a warning?
    pub is_blocking: bool,
}

impl FalsificationReport {
    /// Check if any blocking failures occurred
    pub fn has_blocking_failures(&self) -> bool {
        self.claim_results
            .iter()
            .any(|r| r.result.falsified && r.is_blocking)
    }

    /// Get all blocking failures
    pub fn blocking_failures(&self) -> Vec<&ClaimResult> {
        self.claim_results
            .iter()
            .filter(|r| r.result.falsified && r.is_blocking)
            .collect()
    }

    /// Get all warnings (non-blocking failures)
    pub fn warning_failures(&self) -> Vec<&ClaimResult> {
        self.claim_results
            .iter()
            .filter(|r| r.result.falsified && !r.is_blocking)
            .collect()
    }
}
