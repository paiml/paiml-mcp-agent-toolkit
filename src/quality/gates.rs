#![cfg_attr(coverage_nightly, coverage(off))]
//! Quality gate executor for TICKET-PMAT-5020
//!
//! Executes quality checks and generates reports.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

/// Quality gate execution result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateResult {
    /// Gate name
    pub name: String,
    /// Whether gate passed
    pub passed: bool,
    /// Execution time
    #[serde(with = "serde_millis")]
    pub duration: Duration,
    /// Output/error message
    pub message: String,
}

mod serde_millis {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub(super) fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

/// Overall quality gate report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityReport {
    /// Individual gate results
    pub gates: Vec<GateResult>,
    /// Overall pass/fail
    pub passed: bool,
    /// Total execution time
    #[serde(with = "serde_millis")]
    pub total_duration: Duration,
    /// Timestamp
    pub timestamp: String,
}

/// Quality gate configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateConfig {
    /// Run clippy
    pub run_clippy: bool,
    /// Clippy severity (-D warnings)
    pub clippy_strict: bool,
    /// Run tests
    pub run_tests: bool,
    /// Test timeout (seconds)
    pub test_timeout: u64,
    /// Check coverage
    pub check_coverage: bool,
    /// Minimum coverage percentage
    pub min_coverage: f64,
    /// Check complexity
    pub check_complexity: bool,
    /// Maximum cyclomatic complexity
    pub max_complexity: u32,
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            run_clippy: true,
            clippy_strict: true,
            run_tests: true,
            test_timeout: 300,
            check_coverage: true,
            min_coverage: 80.0,
            check_complexity: true,
            max_complexity: 10,
        }
    }
}

/// Quality gate errors
#[derive(Debug, thiserror::Error)]
pub enum GateError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Timeout exceeded: {0}s")]
    Timeout(u64),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, GateError>;

// Individual gate check implementations (clippy, tests, coverage, complexity)
include!("gates_checks.rs");

// Gate orchestration (execute_all_gates) and report formatting (format_report)
include!("gates_runner.rs");

// Unit and integration tests
include!("gates_tests.rs");
