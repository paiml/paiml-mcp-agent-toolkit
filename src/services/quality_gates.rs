#![cfg_attr(coverage_nightly, coverage(off))]
//! Quality gate enforcement for code standards.
//!
//! This module implements automated quality checks and verification gates that
//! ensure code meets defined standards before it can be accepted. Quality gates
//! check for dead code percentage, complexity metrics, test coverage, and other
//! code quality indicators following the Toyota Way principle of Jidoka
//! (automation with a human touch).
//!
//! # Quality Checks
//!
//! - **Dead Code**: Ensures dead code stays within acceptable limits
//! - **Complexity**: Verifies complexity entropy and distribution
//! - **Coverage**: Checks test coverage meets minimum requirements
//! - **Provability**: Validates pure functions and state invariants
//!
//! # Example
//!
//! ```ignore
//! use pmat::services::quality_gates::{QAVerification, DeepContextResult};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let verifier = QAVerification::new();
//! let context_result = DeepContextResult::default();
//!
//! match verifier.verify(&context_result) {
//!     Ok(verification) => {
//!         println!("Overall status: {:?}", verification.overall);
//!         println!("Dead code: {:.1}%", verification.dead_code.actual * 100.0);
//!         println!("Complexity P99: {}", verification.complexity.p99);
//!     }
//!     Err(e) => println!("Verification failed: {}", e),
//! }
//! # Ok(())
//! # }
//! ```ignore

use anyhow::Result;
use rustc_hash::FxHashMap;

use crate::services::deep_context::{DeepContextResult, FunctionComplexityForQA};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAVerificationResult {
    pub timestamp: String,
    pub version: String,
    pub dead_code: DeadCodeVerification,
    pub complexity: ComplexityVerification,
    pub provability: ProvabilityVerification,
    pub overall: VerificationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeVerification {
    pub status: VerificationStatus,
    pub expected_range: [f64; 2],
    pub actual: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityVerification {
    pub status: VerificationStatus,
    pub entropy: f64,
    pub cv: f64,
    pub p99: u32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvabilityVerification {
    pub status: VerificationStatus,
    pub pure_reducer_coverage: f64,
    pub state_invariants_tested: u32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum VerificationStatus {
    Pass,
    Partial,
    Fail,
}

type QualityCheck = Box<dyn Fn(&DeepContextResult) -> Result<(), String> + Send + Sync>;

pub struct QAVerification {
    checks: Vec<(&'static str, QualityCheck)>,
}

impl QAVerification {
    #[must_use]
    pub fn new() -> Self {
        let mut checks: Vec<(&'static str, QualityCheck)> = vec![];

        // Add all quality checks
        Self::add_dead_code_checks(&mut checks);
        Self::add_complexity_checks(&mut checks);
        Self::add_coverage_checks(&mut checks);
        Self::add_section_checks(&mut checks);

        Self { checks }
    }
}

impl Default for QAVerification {
    fn default() -> Self {
        Self::new()
    }
}

// Check implementations: add_dead_code_checks, add_complexity_checks,
// add_coverage_checks, add_section_checks, calculate_complexity_entropy
include!("quality_gates_checks.rs");

// Reporting: verify(), generate_verification_report()
include!("quality_gates_reporting.rs");

// Tests
include!("quality_gates_tests.rs");
