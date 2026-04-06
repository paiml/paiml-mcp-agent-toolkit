//! Work Contract: Popperian Falsification-Based Quality Enforcement
//!
//! Every claim made by `pmat work complete` must be falsifiable.
//! If ANY claim cannot be verified, work is BLOCKED.
//!
//! Based on: docs/specifications/improve-pmat-work.md

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// Core: WorkContract struct, impl
include!("work_contract_core.rs");

// Legacy Debt: acknowledge_legacy_debt, DebtItem, write_debt_tickets
include!("work_contract_debt.rs");

// Thresholds: ContractThresholds struct, Default impl, helper fns
include!("work_contract_thresholds.rs");

// Manifest: FileManifest, FileEntry, FileCategory
include!("work_contract_manifest.rs");

// Falsification: FalsifiableClaim, OverrideInfo, FalsificationMethod, EvidenceType, FalsificationResult
include!("work_contract_falsification.rs");

// Design by Contract: ContractClause, ClauseKind, ClauseSource, ClauseThreshold, ThresholdOp
include!("work_contract_dbc.rs");

// Contract Profiles: ContractProfile, DbcConfig, claim generation, toolchain checking
include!("work_contract_profile.rs");

// Claim Generators: universal_claims, rust_claims, pmat_claims
include!("work_contract_claims.rs");

// Stack Manifests: third-party tool stacks, TOFU security, command restrictions
include!("work_contract_stack.rs");

// Rescue Protocol: strategies, dispatch, rescue records (Meyer §11)
include!("work_contract_rescue.rs");

// Contract Scoring: 5-dimension quality scoring (DBC spec §13.4-13.5)
include!("work_contract_scoring.rs");

// DBC Lint Rules: 13-rule quality gate (DBC spec §13.3, §14.5)
include!("work_contract_lint.rs");

// Lint Configuration + Diff-Aware Linting + Codebase Scoring (DBC spec §13.6, §13.7, §14.6)
include!("work_contract_lint_config.rs");

// Runtime Violation Tracking + Trust Hash Chain (DBC spec §14.7)
include!("work_contract_violations.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod rescue_tests {
    use super::*;

    include!("work_contract_rescue_tests.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod dbc_tests {
    use super::*;

    include!("work_contract_dbc_tests.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod dbc_tests_profile {
    use super::*;

    include!("work_contract_dbc_tests_profile.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod dbc_tests_checkpoint {
    use super::*;

    include!("work_contract_dbc_tests_checkpoint.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod stack_tests {
    use super::*;

    include!("work_contract_stack_tests.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod scoring_tests {
    use super::*;

    include!("work_contract_scoring_tests.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod lint_tests {
    use super::*;

    include!("work_contract_lint_tests.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod lint_config_tests {
    use super::*;

    include!("work_contract_lint_config_tests.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod violations_tests {
    use super::*;

    include!("work_contract_violations_tests.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_thresholds_default() {
        let thresholds = ContractThresholds::default();
        assert_eq!(thresholds.min_coverage_pct, 95.0);
        assert_eq!(thresholds.min_per_file_coverage_pct, 95.0);
        assert_eq!(thresholds.max_tdg_regression, 0.0);
        assert_eq!(thresholds.max_function_complexity, 20);
        assert_eq!(thresholds.max_file_lines, 500);
        assert_eq!(thresholds.min_spec_score, 95);
        assert!(thresholds.require_github_sync);
        // v2.6 comply spec additions
        assert!(thresholds.block_on_new_satd);
        assert!(thresholds.block_on_new_dead_code);
        assert!(thresholds.require_lint_pass);
    }

    #[test]
    fn test_work_contract_default_claims() {
        let contract = WorkContract::new("test-item".to_string(), "abc123".to_string());
        assert_eq!(contract.claims.len(), 22); // 22 Popperian falsification claims (v4.0)

        // Verify all claim types are present
        let methods: Vec<_> = contract
            .claims
            .iter()
            .map(|c| &c.falsification_method)
            .collect();
        assert!(methods.contains(&&FalsificationMethod::ManifestIntegrity));
        assert!(methods.contains(&&FalsificationMethod::CoverageGaming));
        assert!(methods.contains(&&FalsificationMethod::AbsoluteCoverage));
        assert!(methods.contains(&&FalsificationMethod::TdgRegression));
    }

    #[test]
    fn test_falsification_result_passed() {
        let result = FalsificationResult::passed("Tests passed");
        assert!(!result.falsified);
        assert!(result.evidence.is_none());
    }

    #[test]
    fn test_falsification_result_failed() {
        let result = FalsificationResult::failed(
            "Coverage below threshold",
            EvidenceType::NumericComparison {
                actual: 80.0,
                threshold: 95.0,
            },
        );
        assert!(result.falsified);
        assert!(result.evidence.is_some());
    }

    #[test]
    fn test_simd_pattern_detection() {
        let simd_code = r#"
            use std::arch::x86_64::*;

            #[target_feature(enable = "avx2")]
            unsafe fn process() {
                let a = _mm256_set1_epi32(1);
            }
        "#;

        assert!(FileEntry::contains_simd_patterns(simd_code));

        let normal_code = r#"
            fn normal_function() {
                println!("Hello");
            }
        "#;

        assert!(!FileEntry::contains_simd_patterns(normal_code));
    }

    #[test]
    fn test_file_category_rust_source() {
        // Normal rust file should be RustSource
        let content = "fn main() { println!(\"hello\"); }";
        assert!(!FileEntry::contains_simd_patterns(content));
    }
}

// Coverage-instrumented tests (NOT coverage(off)) for default_claims
#[cfg(test)]
mod coverage_instrumented_tests {
    use super::*;

    #[test]
    fn test_default_claims_returns_17_claims() {
        let claims = WorkContract::default_claims();
        assert_eq!(
            claims.len(),
            22,
            "Expected 22 Popperian falsification claims (v4.0)"
        );
    }

    #[test]
    fn test_default_claims_all_methods_present() {
        let claims = WorkContract::default_claims();
        let methods: Vec<_> = claims.iter().map(|c| &c.falsification_method).collect();

        assert!(methods.contains(&&FalsificationMethod::ManifestIntegrity));
        assert!(methods.contains(&&FalsificationMethod::MetaFalsification));
        assert!(methods.contains(&&FalsificationMethod::CoverageGaming));
        assert!(methods.contains(&&FalsificationMethod::DifferentialCoverage));
        assert!(methods.contains(&&FalsificationMethod::AbsoluteCoverage));
        assert!(methods.contains(&&FalsificationMethod::TdgRegression));
        assert!(methods.contains(&&FalsificationMethod::ComplexityRegression));
        assert!(methods.contains(&&FalsificationMethod::SupplyChainIntegrity));
        assert!(methods.contains(&&FalsificationMethod::FileSizeRegression));
        assert!(methods.contains(&&FalsificationMethod::SpecQuality));
        assert!(methods.contains(&&FalsificationMethod::GitHubSync));
        assert!(methods.contains(&&FalsificationMethod::ExamplesCompile));
        assert!(methods.contains(&&FalsificationMethod::BookValidation));
        assert!(methods.contains(&&FalsificationMethod::SatdDetection));
        assert!(methods.contains(&&FalsificationMethod::DeadCodeDetection));
        assert!(methods.contains(&&FalsificationMethod::PerFileCoverage));
        assert!(methods.contains(&&FalsificationMethod::LintPass));
        assert!(methods.contains(&&FalsificationMethod::VariantCoverage));
        assert!(methods.contains(&&FalsificationMethod::FixChainLimit));
        assert!(methods.contains(&&FalsificationMethod::CrossCrateParity));
        assert!(methods.contains(&&FalsificationMethod::RegressionGate));
    }

    #[test]
    fn test_default_claims_all_have_no_results() {
        let claims = WorkContract::default_claims();
        for claim in &claims {
            assert!(
                claim.result.is_none(),
                "Claim '{}' should start with no result",
                claim.hypothesis
            );
            assert!(
                claim.override_info.is_none(),
                "Claim '{}' should have no override",
                claim.hypothesis
            );
        }
    }

    #[test]
    fn test_default_claims_hypotheses_non_empty() {
        let claims = WorkContract::default_claims();
        for claim in &claims {
            assert!(
                !claim.hypothesis.is_empty(),
                "Claim should have non-empty hypothesis"
            );
            assert!(
                claim.hypothesis.len() > 5,
                "Hypothesis too short: '{}'",
                claim.hypothesis
            );
        }
    }

    #[test]
    fn test_default_claims_coverage_threshold_95() {
        let claims = WorkContract::default_claims();
        let cov_claim = claims
            .iter()
            .find(|c| c.falsification_method == FalsificationMethod::AbsoluteCoverage)
            .expect("AbsoluteCoverage claim should exist");

        if let EvidenceType::NumericComparison { threshold, .. } = &cov_claim.evidence_required {
            assert_eq!(*threshold, 95.0);
        } else {
            panic!("AbsoluteCoverage should use NumericComparison evidence");
        }
    }

    #[test]
    fn test_default_claims_complexity_threshold_20() {
        let claims = WorkContract::default_claims();
        let complexity_claim = claims
            .iter()
            .find(|c| c.falsification_method == FalsificationMethod::ComplexityRegression)
            .expect("ComplexityRegression claim should exist");

        if let EvidenceType::NumericComparison { threshold, .. } =
            &complexity_claim.evidence_required
        {
            assert_eq!(*threshold, 20.0);
        } else {
            panic!("ComplexityRegression should use NumericComparison evidence");
        }
    }

    #[test]
    fn test_default_claims_file_size_threshold_500() {
        let claims = WorkContract::default_claims();
        let size_claim = claims
            .iter()
            .find(|c| c.falsification_method == FalsificationMethod::FileSizeRegression)
            .expect("FileSizeRegression claim should exist");

        if let EvidenceType::NumericComparison { threshold, .. } = &size_claim.evidence_required {
            assert_eq!(*threshold, 500.0);
        } else {
            panic!("FileSizeRegression should use NumericComparison evidence");
        }
    }

    #[test]
    fn test_work_contract_new_captures_baseline() {
        let contract = WorkContract::new("PMAT-100".to_string(), "abc123def".to_string());
        assert_eq!(contract.work_item_id, "PMAT-100");
        assert_eq!(contract.baseline_commit, "abc123def");
        assert_eq!(contract.baseline_tdg, 0.0);
        assert_eq!(contract.baseline_coverage, 0.0);
        assert!(contract.baseline_rust_score.is_none());
        assert_eq!(contract.claims.len(), 22);
    }

    #[test]
    fn test_contract_thresholds_default_values() {
        let t = ContractThresholds::default();
        assert_eq!(t.min_coverage_pct, 95.0);
        assert_eq!(t.max_function_complexity, 20);
        assert_eq!(t.max_file_lines, 500);
        assert!(t.require_github_sync);
        assert!(t.block_on_new_satd);
        assert!(t.block_on_new_dead_code);
        assert!(t.require_lint_pass);
    }

    #[test]
    fn test_falsification_result_passed() {
        let result = FalsificationResult::passed("All checks passed");
        assert!(!result.falsified);
        assert!(result.evidence.is_none());
        assert_eq!(result.explanation, "All checks passed");
    }

    #[test]
    fn test_falsification_result_failed_with_evidence() {
        let result = FalsificationResult::failed(
            "Coverage too low",
            EvidenceType::NumericComparison {
                actual: 80.0,
                threshold: 95.0,
            },
        );
        assert!(result.falsified);
        assert!(result.evidence.is_some());
        assert_eq!(result.explanation, "Coverage too low");
    }
}
