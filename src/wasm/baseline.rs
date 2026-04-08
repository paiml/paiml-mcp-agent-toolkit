#![cfg_attr(coverage_nightly, coverage(off))]
//! Anchored quality metrics and baseline management

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Multi-point baseline system for quality assessment
#[derive(Debug, Clone)]
pub struct QualityBaseline {
    release_anchor: Metrics,      // Last major release (immutable)
    stable_anchor: Metrics,       // Last stable tag
    rolling_window: RollingStats, // Recent 30 days
}

/// Quality metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub timestamp: DateTime<Utc>,
    pub complexity_p90: u32,
    pub complexity_p95: u32,
    pub complexity_p99: u32,
    pub binary_size: usize,
    pub init_time_ms: u32,
    pub memory_usage_mb: u32,
    pub function_count: usize,
    pub instruction_count: usize,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            complexity_p90: 10,
            complexity_p95: 15,
            complexity_p99: 20,
            binary_size: 1_000_000,
            init_time_ms: 10,
            memory_usage_mb: 50,
            function_count: 100,
            instruction_count: 10_000,
        }
    }
}

/// Rolling statistics for trend analysis
#[derive(Debug, Clone)]
pub struct RollingStats {
    window_days: usize,
    data_points: VecDeque<Metrics>,
}

/// Quality assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessment {
    pub violations: Vec<Violation>,
    pub overall_health: f64,
    pub recommendation: String,
}

impl QualityAssessment {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Is passing.
    pub fn is_passing(&self) -> bool {
        self.violations
            .iter()
            .all(|v| !matches!(v.severity(), Severity::Error))
    }
}

/// Quality violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Violation {
    ComplexityRegression {
        current: u32,
        limit: u32,
        severity: Severity,
    },
    ComplexityCreep {
        current: u32,
        baseline: u32,
        severity: Severity,
    },
    QualityErosion {
        slope: f64,
        severity: Severity,
    },
    BinarySizeIncrease {
        current: usize,
        baseline: usize,
        increase_percent: f64,
        severity: Severity,
    },
    PerformanceRegression {
        metric: String,
        current: f64,
        baseline: f64,
        severity: Severity,
    },
}

/// Severity levels for violations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

// --- Implementation split across include files ---

include!("baseline_evaluation.rs");
include!("baseline_violations.rs");
include!("baseline_tests.rs");
include!("baseline_tests_violations.rs");
