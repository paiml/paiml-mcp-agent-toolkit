#![cfg_attr(coverage_nightly, coverage(off))]
use super::types::*;
use std::fs;
use std::path::Path;

// =============================================================================
// CB-125, CB-126, CB-127: Coverage Quality & Test Performance (v2.2)
// Per improve-pmat-comply.md v2.2.0 specification
// =============================================================================

// =============================================================================
// CB-400: Shell & Makefile Quality (bashrs integration)
// Uses bashrs for deterministic, idempotent, and safe shell scripting.
//
// Sub-checks:
// - CB-400: Git hooks quality (pre-commit, pre-push, etc.)
// - CB-401: Makefile quality
// - CB-402: Shell script quality (*.sh)
// =============================================================================

/// Result of bashrs lint check
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BashrsLintResult {
    pub file: String,
    pub issues: Vec<BashrsIssue>,
    pub passed: bool,
}

/// Individual bashrs issue
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BashrsIssue {
    pub code: String,
    pub message: String,
    pub line: usize,
    pub severity: String,
}

/// State for tracking coverage target parsing
#[derive(Default)]
pub(super) struct CoverageTargetState {
    pub(super) active: bool,
    pub(super) line: usize,
    pub(super) has_nextest: bool,
    pub(super) has_llvm_cov: bool,
    pub(super) has_proptest_cases: bool,
    pub(super) has_lib_flag: bool,
    /// Whether this target actually runs cargo tests (vs. report/clean/alias/deno)
    pub(super) runs_cargo_tests: bool,
}

impl CoverageTargetState {
    pub(super) fn reset(&mut self, line: usize) {
        self.active = true;
        self.line = line;
        self.has_nextest = false;
        self.has_llvm_cov = false;
        self.has_proptest_cases = false;
        self.has_lib_flag = false;
        self.runs_cargo_tests = false;
    }

    pub(super) fn update_from_line(&mut self, line: &str) {
        let trimmed = line.trim();
        // Skip comments and echo statements
        if trimmed.starts_with('#') || trimmed.starts_with("@#") {
            return;
        }
        let is_echo = trimmed.starts_with("@echo") || trimmed.starts_with("echo");
        if !is_echo && line.contains("nextest") {
            self.has_nextest = true;
            self.runs_cargo_tests = true;
        }
        if line.contains("llvm-cov") || line.contains("cargo-llvm-cov") {
            self.has_llvm_cov = true;
        }
        // Detect actual test execution: `cargo test` or `cargo llvm-cov test`
        // Exclude report-only commands like `cargo llvm-cov report`
        if !is_echo && (line.contains("cargo test") || line.contains("cargo llvm-cov test")) {
            self.runs_cargo_tests = true;
        }
        if line.contains("PROPTEST_CASES") || line.contains("QUICKCHECK_TESTS") {
            self.has_proptest_cases = true;
        }
        if line.contains("--lib") {
            self.has_lib_flag = true;
        }
    }

    pub(super) fn collect_violations(&self, file_path: &str) -> Vec<CbPatternViolation> {
        let mut violations = Vec::new();

        // Only flag targets that actually run cargo tests.
        // Skip: alias/delegate targets, report-only, clean, open, invalidate, deno targets.
        if !self.runs_cargo_tests {
            return violations;
        }

        if self.has_nextest && self.has_llvm_cov {
            violations.push(CbPatternViolation {
                pattern_id: "CB-127-A".to_string(),
                file: file_path.to_string(),
                line: self.line,
                description: "CRITICAL: nextest + llvm-cov causes profraw explosion. \
                    Use 'cargo llvm-cov test' instead"
                    .to_string(),
                severity: Severity::Error,
            });
        }
        if !self.has_proptest_cases {
            violations.push(CbPatternViolation {
                pattern_id: "CB-127-B".to_string(),
                file: file_path.to_string(),
                line: self.line,
                description: "Coverage target missing PROPTEST_CASES/QUICKCHECK_TESTS".to_string(),
                severity: Severity::Warning,
            });
        }
        if !self.has_lib_flag && self.has_llvm_cov {
            violations.push(CbPatternViolation {
                pattern_id: "CB-127-C".to_string(),
                file: file_path.to_string(),
                line: self.line,
                description: "Coverage target missing --lib flag".to_string(),
                severity: Severity::Warning,
            });
        }
        violations
    }
}

// CB-125: Coverage exclusion gaming detection
include!("quality_checks_coverage_gaming.rs");

// CB-126, CB-127: Slow test and coverage detection
include!("quality_checks_slow_tests.rs");

// CB-400/401/402: Shell & Makefile quality (bashrs integration)
include!("quality_checks_bashrs.rs");
