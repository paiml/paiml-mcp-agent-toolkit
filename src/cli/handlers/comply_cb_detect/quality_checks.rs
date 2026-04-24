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

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod coverage_target_state_tests {
    //! Covers the CoverageTargetState state machine in quality_checks.rs
    //! (68 uncov on broad, 0% cov). Exercises reset, update_from_line, and
    //! collect_violations without needing a full Makefile fixture.
    use super::*;

    // ── reset: restores a clean `active=true` state with a new line ──

    #[test]
    fn test_coverage_target_state_reset_clears_all_flags_and_sets_line() {
        let mut s = CoverageTargetState::default();
        // Pre-populate to verify reset actually clears things.
        s.has_nextest = true;
        s.has_llvm_cov = true;
        s.has_proptest_cases = true;
        s.has_lib_flag = true;
        s.runs_cargo_tests = true;
        s.line = 999;
        s.active = false;

        s.reset(42);

        assert!(s.active, "reset must activate the target");
        assert_eq!(s.line, 42);
        assert!(!s.has_nextest);
        assert!(!s.has_llvm_cov);
        assert!(!s.has_proptest_cases);
        assert!(!s.has_lib_flag);
        assert!(!s.runs_cargo_tests);
    }

    // ── update_from_line: each detector arm ──

    #[test]
    fn test_update_from_line_detects_nextest_and_marks_runs_cargo_tests() {
        let mut s = CoverageTargetState::default();
        s.update_from_line("\tcargo nextest run --lib");
        assert!(s.has_nextest);
        assert!(s.runs_cargo_tests);
    }

    #[test]
    fn test_update_from_line_detects_llvm_cov_does_not_mark_runs_cargo_tests() {
        let mut s = CoverageTargetState::default();
        s.update_from_line("\tcargo llvm-cov report --html");
        assert!(s.has_llvm_cov);
        assert!(
            !s.runs_cargo_tests,
            "plain llvm-cov report must NOT mark runs_cargo_tests"
        );
    }

    #[test]
    fn test_update_from_line_detects_cargo_llvm_cov_test_marks_runs_cargo_tests() {
        let mut s = CoverageTargetState::default();
        s.update_from_line("\tcargo llvm-cov test --lib");
        assert!(s.has_llvm_cov);
        assert!(s.runs_cargo_tests, "llvm-cov TEST marks runs_cargo_tests");
    }

    #[test]
    fn test_update_from_line_detects_cargo_test_alone() {
        let mut s = CoverageTargetState::default();
        s.update_from_line("\tcargo test --lib --quiet");
        assert!(s.runs_cargo_tests);
        assert!(s.has_lib_flag);
    }

    #[test]
    fn test_update_from_line_detects_proptest_cases_env_var() {
        let mut s = CoverageTargetState::default();
        s.update_from_line("\tPROPTEST_CASES=1000 cargo test");
        assert!(s.has_proptest_cases);

        let mut s = CoverageTargetState::default();
        s.update_from_line("\tQUICKCHECK_TESTS=1000 cargo test");
        assert!(s.has_proptest_cases);
    }

    #[test]
    fn test_update_from_line_comment_lines_are_skipped() {
        let mut s = CoverageTargetState::default();
        s.update_from_line("# cargo test --lib with nextest");
        assert!(!s.runs_cargo_tests);
        assert!(!s.has_nextest);
    }

    #[test]
    fn test_update_from_line_echo_lines_do_not_mark_runs_cargo_tests() {
        let mut s = CoverageTargetState::default();
        s.update_from_line("\t@echo \"Running nextest...\"");
        // `echo` should suppress the nextest + cargo test signals.
        assert!(!s.has_nextest);
        assert!(!s.runs_cargo_tests);
    }

    // ── collect_violations: gating + per-rule firing ──

    #[test]
    fn test_collect_violations_empty_when_not_running_cargo_tests() {
        let mut s = CoverageTargetState::default();
        s.reset(10);
        // runs_cargo_tests=false → all violations suppressed.
        let v = s.collect_violations("Makefile");
        assert!(v.is_empty());
    }

    #[test]
    fn test_collect_violations_cb127a_fires_for_nextest_plus_llvm_cov() {
        let mut s = CoverageTargetState::default();
        s.reset(20);
        s.runs_cargo_tests = true;
        s.has_nextest = true;
        s.has_llvm_cov = true;
        // Also meet proptest/lib requirements so those rules don't fire.
        s.has_proptest_cases = true;
        s.has_lib_flag = true;
        let v = s.collect_violations("Makefile");
        assert!(v.iter().any(|x| x.pattern_id == "CB-127-A"));
    }

    #[test]
    fn test_collect_violations_cb127b_fires_when_proptest_cases_missing() {
        let mut s = CoverageTargetState::default();
        s.reset(20);
        s.runs_cargo_tests = true;
        // has_proptest_cases=false by default.
        let v = s.collect_violations("Makefile");
        assert!(v.iter().any(|x| x.pattern_id == "CB-127-B"));
    }

    #[test]
    fn test_collect_violations_cb127c_fires_for_llvm_cov_without_lib_flag() {
        let mut s = CoverageTargetState::default();
        s.reset(20);
        s.runs_cargo_tests = true;
        s.has_llvm_cov = true;
        s.has_proptest_cases = true; // avoid CB-127-B
                                     // has_lib_flag=false by default.
        let v = s.collect_violations("Makefile");
        assert!(v.iter().any(|x| x.pattern_id == "CB-127-C"));
    }

    #[test]
    fn test_collect_violations_no_violations_when_all_good() {
        let mut s = CoverageTargetState::default();
        s.reset(20);
        s.runs_cargo_tests = true;
        s.has_llvm_cov = true;
        s.has_proptest_cases = true;
        s.has_lib_flag = true;
        // No nextest+llvm-cov combo; proptest+lib present → zero violations.
        let v = s.collect_violations("Makefile");
        assert!(v.is_empty());
    }

    #[test]
    fn test_collect_violations_carries_file_path_and_line() {
        let mut s = CoverageTargetState::default();
        s.reset(77);
        s.runs_cargo_tests = true;
        // Force CB-127-B to fire.
        let v = s.collect_violations("sub/Makefile");
        let cb_b = v.iter().find(|x| x.pattern_id == "CB-127-B").unwrap();
        assert_eq!(cb_b.file, "sub/Makefile");
        assert_eq!(cb_b.line, 77);
    }
}
