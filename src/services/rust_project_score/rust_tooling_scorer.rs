//! RustToolingScorer - Rust Tooling Compliance Category (37 points)
//!
//! Analyzes Rust project compliance with standard tooling:
//! - Clippy (tiered scoring): 10pts
//!   - Correctness: 5pts (zero warnings)
//!   - Suspicious: 3pts (zero warnings)
//!   - Pedantic: 2pts (zero warnings)
//! - rustfmt compliance: 5pts
//! - cargo-audit (security): 7pts (risk-based scoring)
//! - cargo-deny (policy): 3pts
//! - **v2.0 Workspace Lints (12pts)**: Based on "Learn from Rust Giants" spec
//!   - Workspace-level lints configured: 5pts
//!   - High-value lint categories (correctness, suspicious, perf): 4pts
//!   - .clippy.toml with disallowed-methods: 3pts

#![cfg_attr(coverage_nightly, coverage(off))]

use super::models::{CategoryScore, FileCache, ScoringMode};
use super::scorer::{Scorer, ScorerError, ScorerResult};
use std::path::Path;
use std::process::Command;

/// Count of vulnerabilities by severity level
#[derive(Debug, Default)]
struct VulnerabilityCount {
    critical: u32,
    high: u32,
    medium: u32,
    low: u32,
}

/// Rust Tooling Compliance scorer
#[derive(Debug, Clone)]
pub struct RustToolingScorer {
    /// Category name
    name: String,
    /// Maximum possible points
    max_points: f64,
}

impl RustToolingScorer {
    /// Create a new RustToolingScorer
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            name: "Rust Tooling & CI/CD".to_string(),
            max_points: 130.0, // v2.0: 25 + 12 (lints) + 37 (CI/CD) + 35 (metadata) + 10 (MSRV) + 11 (profiles)
        }
    }
}

impl Default for RustToolingScorer {
    fn default() -> Self {
        Self::new()
    }
}

// Tool-based scoring: clippy, rustfmt, cargo-audit, cargo-deny, mode-based wrappers
include!("rust_tooling_scorer_tools.rs");

// Workspace lint configuration scoring
include!("rust_tooling_scorer_lints.rs");

// CI/CD integration and build automation scoring
include!("rust_tooling_scorer_ci.rs");

// Metadata scoring: docs.rs, workspace organization, release automation
include!("rust_tooling_scorer_metadata.rs");

// MSRV tracking and release profile optimization scoring
include!("rust_tooling_scorer_msrv.rs");

// Scorer trait implementation and Send+Sync impls
include!("rust_tooling_scorer_trait_impl.rs");

// Tests extracted to rust_tooling_scorer_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "rust_tooling_scorer_tests.rs"]
mod tests;
