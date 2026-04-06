#![cfg_attr(coverage_nightly, coverage(off))]
//! Gaming Detector: Anti-Gaming Detection for Coverage and Quality Metrics
//!
//! Detects attempts to game coverage metrics through:
//! - `#[cfg(not(coverage))]` patterns
//! - `.codecov.yml` exclusion changes
//! - Test file deletions
//! - CUDA/AVX file hiding
//!
//! Based on: docs/specifications/improve-pmat-work.md

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Gaming violation detected in codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingViolation {
    /// File where violation was found
    pub file: PathBuf,

    /// Line number (0 if not applicable)
    pub line: usize,

    /// Type of gaming pattern detected
    pub pattern: GamingPattern,

    /// Severity of the violation
    pub severity: Severity,

    /// Human-readable explanation
    pub explanation: String,
}

/// Types of gaming patterns we detect
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GamingPattern {
    /// `#[cfg(not(coverage))]` to exclude code from coverage
    CfgNotCoverage,

    /// `#[cfg(not(tarpaulin))]` to exclude from tarpaulin
    CfgNotTarpaulin,

    /// `#[cfg(not(llvm_cov))]` to exclude from llvm-cov
    CfgNotLlvmCov,

    /// New exclusion added to `.codecov.yml`
    NewCodecovExclusion(String),

    /// Test file deleted during work
    TestFileDeletion,

    /// Test module marked `#[ignore]` during work
    TestModuleIgnored,

    /// CUDA/AVX file removed from manifest
    CriticalFileRemoved,

    /// Coverage exclusion comment pattern
    CoverageExclusionComment,
}

/// Severity of gaming violation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    /// Must be fixed before completion
    Critical,

    /// Should be fixed, but can be overridden
    Warning,

    /// Informational only
    Info,
}

/// Results of gaming detection scan
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GamingDetectionResult {
    /// All violations found
    pub violations: Vec<GamingViolation>,

    /// Number of files scanned
    pub files_scanned: usize,

    /// Patterns that were checked
    pub patterns_checked: Vec<String>,
}

impl GamingDetectionResult {
    /// Check if any critical violations were found
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_critical_violations(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity == Severity::Critical)
    }

    /// Get only critical violations
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn critical_violations(&self) -> Vec<&GamingViolation> {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Critical)
            .collect()
    }

    /// Count violations by severity
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn count_by_severity(&self, severity: Severity) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == severity)
            .count()
    }
}

// --- Include files for implementation and tests ---
include!("gaming_detector_analysis.rs");
include!("gaming_detector_tests.rs");
