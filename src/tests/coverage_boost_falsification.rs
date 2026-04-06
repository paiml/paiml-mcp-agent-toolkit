#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for cli/handlers/work_falsification.rs
//! Tests: FalsificationReport methods (has_blocking_failures, blocking_failures, warning_failures)
//! Also tests ClaimResult construction, FalsificationMethod, FalsificationResult

use crate::cli::handlers::work_contract::{FalsificationMethod, FalsificationResult};
use crate::cli::handlers::work_falsification::{ClaimResult, FalsificationReport};

// ============ Helpers ============

fn make_claim(index: usize, hypothesis: &str, falsified: bool, is_blocking: bool) -> ClaimResult {
    ClaimResult {
        index,
        hypothesis: hypothesis.to_string(),
        method: FalsificationMethod::DifferentialCoverage,
        result: FalsificationResult {
            falsified,
            evidence: None,
            explanation: format!("Evidence for {hypothesis}"),
        },
        is_blocking,
    }
}

// ============ FalsificationReport - no claims ============

#[test]
fn test_empty_report_no_blocking() {
    let report = FalsificationReport {
        total_claims: 0,
        passed: 0,
        failed: 0,
        warnings: 0,
        claim_results: vec![],
        all_passed: true,
    };
    assert!(!report.has_blocking_failures());
    assert!(report.blocking_failures().is_empty());
    assert!(report.warning_failures().is_empty());
}

// ============ FalsificationReport - all passed ============

#[test]
fn test_all_passed_report() {
    let report = FalsificationReport {
        total_claims: 3,
        passed: 3,
        failed: 0,
        warnings: 0,
        claim_results: vec![
            make_claim(1, "Coverage >= 80%", false, true),
            make_claim(2, "TDG < 2.0", false, true),
            make_claim(3, "No new SATD", false, false),
        ],
        all_passed: true,
    };
    assert!(!report.has_blocking_failures());
    assert!(report.blocking_failures().is_empty());
    assert!(report.warning_failures().is_empty());
}

// ============ FalsificationReport - blocking failure ============

#[test]
fn test_blocking_failure() {
    let report = FalsificationReport {
        total_claims: 3,
        passed: 2,
        failed: 1,
        warnings: 0,
        claim_results: vec![
            make_claim(1, "Coverage >= 80%", true, true), // blocking failure
            make_claim(2, "TDG < 2.0", false, true),
            make_claim(3, "No new SATD", false, false),
        ],
        all_passed: false,
    };
    assert!(report.has_blocking_failures());
    let blocking = report.blocking_failures();
    assert_eq!(blocking.len(), 1);
    assert_eq!(blocking[0].hypothesis, "Coverage >= 80%");
    assert!(blocking[0].is_blocking);
    assert!(report.warning_failures().is_empty());
}

// ============ FalsificationReport - warning only (non-blocking) ============

#[test]
fn test_warning_failure_non_blocking() {
    let report = FalsificationReport {
        total_claims: 3,
        passed: 2,
        failed: 0,
        warnings: 1,
        claim_results: vec![
            make_claim(1, "Coverage >= 80%", false, true),
            make_claim(2, "TDG < 2.0", false, true),
            make_claim(3, "No new SATD", true, false), // warning (falsified but non-blocking)
        ],
        all_passed: false,
    };
    assert!(!report.has_blocking_failures());
    assert!(report.blocking_failures().is_empty());
    let warnings = report.warning_failures();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].hypothesis, "No new SATD");
    assert!(!warnings[0].is_blocking);
}

// ============ FalsificationReport - mixed failures ============

#[test]
fn test_mixed_blocking_and_warning() {
    let report = FalsificationReport {
        total_claims: 5,
        passed: 2,
        failed: 2,
        warnings: 1,
        claim_results: vec![
            make_claim(1, "Coverage >= 80%", true, true), // blocking
            make_claim(2, "TDG < 2.0", false, true),      // passed
            make_claim(3, "No new SATD", true, false),    // warning
            make_claim(4, "Complexity < 20", true, true), // blocking
            make_claim(5, "No dead code", false, false),  // passed
        ],
        all_passed: false,
    };
    assert!(report.has_blocking_failures());
    assert_eq!(report.blocking_failures().len(), 2);
    assert_eq!(report.warning_failures().len(), 1);
}

// ============ FalsificationReport - all blocking failures ============

#[test]
fn test_all_blocking_failures() {
    let report = FalsificationReport {
        total_claims: 3,
        passed: 0,
        failed: 3,
        warnings: 0,
        claim_results: vec![
            make_claim(1, "Claim A", true, true),
            make_claim(2, "Claim B", true, true),
            make_claim(3, "Claim C", true, true),
        ],
        all_passed: false,
    };
    assert!(report.has_blocking_failures());
    assert_eq!(report.blocking_failures().len(), 3);
    assert!(report.warning_failures().is_empty());
}

// ============ FalsificationReport - all warning failures ============

#[test]
fn test_all_warning_failures() {
    let report = FalsificationReport {
        total_claims: 3,
        passed: 0,
        failed: 0,
        warnings: 3,
        claim_results: vec![
            make_claim(1, "Warn A", true, false),
            make_claim(2, "Warn B", true, false),
            make_claim(3, "Warn C", true, false),
        ],
        all_passed: false,
    };
    assert!(!report.has_blocking_failures());
    assert!(report.blocking_failures().is_empty());
    assert_eq!(report.warning_failures().len(), 3);
}

// ============ ClaimResult construction ============

#[test]
fn test_claim_result_debug() {
    let claim = make_claim(1, "Test hypothesis", false, true);
    let debug = format!("{:?}", claim);
    assert!(debug.contains("Test hypothesis"));
}

#[test]
fn test_claim_result_fields() {
    let claim = ClaimResult {
        index: 42,
        hypothesis: "Coverage target met".to_string(),
        method: FalsificationMethod::AbsoluteCoverage,
        result: FalsificationResult::passed("83.5% >= 80% target"),
        is_blocking: true,
    };
    assert_eq!(claim.index, 42);
    assert!(!claim.result.falsified);
    assert!(claim.is_blocking);
}

// ============ FalsificationResult construction ============

#[test]
fn test_falsification_result_passed_helper() {
    let result = FalsificationResult::passed("All checks passed");
    assert!(!result.falsified);
    assert!(result.evidence.is_none());
    assert_eq!(result.explanation, "All checks passed");
}

#[test]
fn test_falsification_result_manual_construction() {
    let result = FalsificationResult {
        falsified: true,
        evidence: None,
        explanation: "Coverage dropped".to_string(),
    };
    assert!(result.falsified);
    assert!(result.explanation.contains("Coverage dropped"));
}

// ============ FalsificationMethod variants ============

#[test]
fn test_falsification_method_debug_all_variants() {
    let methods: Vec<FalsificationMethod> = vec![
        FalsificationMethod::ManifestIntegrity,
        FalsificationMethod::DifferentialCoverage,
        FalsificationMethod::AbsoluteCoverage,
        FalsificationMethod::TdgRegression,
        FalsificationMethod::ComplexityRegression,
        FalsificationMethod::FileSizeRegression,
        FalsificationMethod::SpecQuality,
        FalsificationMethod::RoadmapUpdate,
        FalsificationMethod::GitHubSync,
        FalsificationMethod::CoverageGaming,
        FalsificationMethod::SupplyChainIntegrity,
        FalsificationMethod::MetaFalsification,
        FalsificationMethod::ExamplesCompile,
        FalsificationMethod::BookValidation,
        FalsificationMethod::SatdDetection,
        FalsificationMethod::DeadCodeDetection,
        FalsificationMethod::PerFileCoverage,
        FalsificationMethod::LintPass,
        FalsificationMethod::VariantCoverage,
        FalsificationMethod::FixChainLimit,
        FalsificationMethod::CrossCrateParity,
        FalsificationMethod::RegressionGate,
        FalsificationMethod::FormalProofVerification,
    ];
    for method in methods {
        let _ = format!("{:?}", method);
    }
}

#[test]
fn test_falsification_method_serde() {
    let method = FalsificationMethod::DifferentialCoverage;
    let json = serde_json::to_string(&method).unwrap();
    let back: FalsificationMethod = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", back);
}
