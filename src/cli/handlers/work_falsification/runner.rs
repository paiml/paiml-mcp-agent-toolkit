#![cfg_attr(coverage_nightly, coverage(off))]
//! Falsification test runner and dispatcher.
//!
//! Orchestrates running all falsification tests against a work contract
//! and produces a FalsificationReport.

use super::advanced_checks;
use super::churn_checks;
use super::core_checks;
use super::supply_chain_checks;
use super::types::{ClaimResult, FalsificationReport};
use crate::cli::handlers::work_contract::{
    EvidenceType, FalsifiableClaim, FalsificationMethod, FalsificationResult, WorkContract,
};
use anyhow::Result;
use std::path::Path;

/// Run all falsification tests against the work contract
pub async fn run_falsification_tests(
    project_path: &Path,
    contract: &WorkContract,
) -> Result<FalsificationReport> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let mut claim_results = Vec::new();
    let total_claims = contract.claims.len();

    println!();
    println!(
        "Running Popperian Falsification ({} claims to validate)",
        total_claims
    );
    println!();

    for (i, claim) in contract.claims.iter().enumerate() {
        let index = i + 1;
        println!("[{}/{}] {}", index, total_claims, claim.hypothesis);
        print!("      Falsification: ");

        let (result, is_blocking) = run_single_falsification(project_path, contract, claim).await?;

        let status = if result.falsified {
            if is_blocking {
                "FAILED"
            } else {
                "WARNING"
            }
        } else {
            "PASSED"
        };

        println!("{}", result.explanation);
        println!("      Result: {}", status);

        if result.falsified {
            if let Some(ref evidence) = result.evidence {
                print_evidence(evidence);
            }
        }
        println!();

        claim_results.push(ClaimResult {
            index,
            hypothesis: claim.hypothesis.clone(),
            method: claim.falsification_method.clone(),
            result,
            is_blocking,
        });
    }

    let passed = claim_results.iter().filter(|r| !r.result.falsified).count();
    let failed = claim_results
        .iter()
        .filter(|r| r.result.falsified && r.is_blocking)
        .count();
    let warnings = claim_results
        .iter()
        .filter(|r| r.result.falsified && !r.is_blocking)
        .count();

    let all_passed = failed == 0;

    Ok(FalsificationReport {
        total_claims,
        passed,
        failed,
        warnings,
        claim_results,
        all_passed,
    })
}

/// Determine if a falsification method should block completion on failure.
///
/// Returns `true` for always-blocking methods, or reads from contract thresholds
/// for configurable methods (v2.6/v3.1 additions).
fn determine_blocking_status(
    method: &FalsificationMethod,
    thresholds: &crate::cli::handlers::work_contract::ContractThresholds,
) -> bool {
    match method {
        // Always blocking
        FalsificationMethod::ManifestIntegrity
        | FalsificationMethod::MetaFalsification
        | FalsificationMethod::CoverageGaming
        | FalsificationMethod::DifferentialCoverage
        | FalsificationMethod::AbsoluteCoverage
        | FalsificationMethod::TdgRegression
        | FalsificationMethod::ComplexityRegression
        | FalsificationMethod::SupplyChainIntegrity
        | FalsificationMethod::SpecQuality
        | FalsificationMethod::RoadmapUpdate
        | FalsificationMethod::GitHubSync
        | FalsificationMethod::ExamplesCompile
        | FalsificationMethod::BookValidation
        | FalsificationMethod::PerFileCoverage
        | FalsificationMethod::FixChainLimit => true,

        // Configurable via thresholds (v4.1 file health enforcement)
        FalsificationMethod::FileSizeRegression => thresholds.block_on_file_size,

        // Configurable via thresholds
        FalsificationMethod::SatdDetection => thresholds.block_on_new_satd,
        FalsificationMethod::DeadCodeDetection => thresholds.block_on_new_dead_code,
        FalsificationMethod::LintPass => thresholds.require_lint_pass,
        FalsificationMethod::VariantCoverage => thresholds.block_on_untested_variants,
        FalsificationMethod::CrossCrateParity => thresholds.block_on_cross_crate_failure,
        FalsificationMethod::RegressionGate => thresholds.block_on_regression,
        FalsificationMethod::FormalProofVerification => thresholds.require_proof_verification,
    }
}

/// Dispatch a falsification test to the appropriate handler.
///
/// Each variant maps to a single test function. Sync tests are called directly;
/// async tests are `.await`ed.
async fn dispatch_falsification_test(
    project_path: &Path,
    contract: &WorkContract,
    claim: &FalsifiableClaim,
) -> Result<FalsificationResult> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    match claim.falsification_method {
        FalsificationMethod::ManifestIntegrity => {
            core_checks::test_manifest_integrity(project_path, &contract.baseline_file_manifest)
        }
        FalsificationMethod::MetaFalsification => {
            supply_chain_checks::test_meta_falsification(project_path)
        }
        FalsificationMethod::CoverageGaming => core_checks::test_coverage_gaming(project_path),
        FalsificationMethod::DifferentialCoverage => {
            core_checks::test_differential_coverage(project_path, &contract.baseline_commit).await
        }
        FalsificationMethod::AbsoluteCoverage => {
            core_checks::test_absolute_coverage(project_path, contract.thresholds.min_coverage_pct)
                .await
        }
        FalsificationMethod::TdgRegression => {
            core_checks::test_tdg_regression(project_path, contract.baseline_tdg).await
        }
        FalsificationMethod::ComplexityRegression => core_checks::test_complexity_regression(
            project_path,
            contract.thresholds.max_function_complexity,
        ),
        FalsificationMethod::SupplyChainIntegrity => {
            supply_chain_checks::test_supply_chain_integrity(project_path).await
        }
        FalsificationMethod::FileSizeRegression => {
            core_checks::test_file_size_regression(project_path, contract.thresholds.max_file_lines)
        }
        FalsificationMethod::SpecQuality => core_checks::test_spec_quality(
            project_path,
            &contract.work_item_id,
            contract.thresholds.min_spec_score,
        ),
        FalsificationMethod::RoadmapUpdate => {
            core_checks::test_roadmap_update(project_path, &contract.baseline_commit)
        }
        FalsificationMethod::GitHubSync => core_checks::test_github_sync(project_path),
        FalsificationMethod::ExamplesCompile => {
            supply_chain_checks::test_examples_compile(project_path).await
        }
        FalsificationMethod::BookValidation => {
            supply_chain_checks::test_book_validation(project_path).await
        }
        FalsificationMethod::SatdDetection => {
            advanced_checks::test_satd_detection(project_path, &contract.baseline_commit).await
        }
        FalsificationMethod::DeadCodeDetection => {
            advanced_checks::test_dead_code_detection(project_path, &contract.baseline_commit).await
        }
        FalsificationMethod::PerFileCoverage => {
            advanced_checks::test_per_file_coverage(
                project_path,
                contract.thresholds.min_per_file_coverage_pct,
            )
            .await
        }
        FalsificationMethod::LintPass => advanced_checks::test_lint_pass(project_path).await,
        FalsificationMethod::VariantCoverage => {
            churn_checks::test_variant_coverage(project_path, &contract.baseline_commit)
        }
        FalsificationMethod::FixChainLimit => {
            churn_checks::test_fix_chain_limit(project_path, contract.thresholds.max_fix_chain)
        }
        FalsificationMethod::CrossCrateParity => {
            supply_chain_checks::test_cross_crate_parity(project_path).await
        }
        FalsificationMethod::RegressionGate => {
            supply_chain_checks::test_regression_gate(project_path).await
        }
        FalsificationMethod::FormalProofVerification => {
            advanced_checks::test_formal_proof_verification(
                project_path,
                contract.thresholds.max_sorry_count,
            )
        }
    }
}

/// Run a single falsification test
async fn run_single_falsification(
    project_path: &Path,
    contract: &WorkContract,
    claim: &FalsifiableClaim,
) -> Result<(FalsificationResult, bool)> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let result = dispatch_falsification_test(project_path, contract, claim).await?;
    let is_blocking = determine_blocking_status(&claim.falsification_method, &contract.thresholds);
    Ok((result, is_blocking))
}

/// Print evidence details
fn print_evidence(evidence: &EvidenceType) {
    match evidence {
        EvidenceType::FileList(files) => {
            println!("      Evidence (files):");
            for file in files.iter().take(5) {
                println!("        - {}", file.display());
            }
            if files.len() > 5 {
                println!("        ... and {} more", files.len() - 5);
            }
        }
        EvidenceType::NumericComparison { actual, threshold } => {
            println!(
                "      Evidence: actual={:.1}, threshold={:.1}",
                actual, threshold
            );
        }
        EvidenceType::GitState {
            unpushed_commits,
            dirty_files,
        } => {
            println!(
                "      Evidence: {} unpushed, {} dirty",
                unpushed_commits, dirty_files
            );
        }
        EvidenceType::BooleanCheck(value) => {
            println!("      Evidence: {}", value);
        }
        EvidenceType::CounterExample { details } => {
            println!("      Evidence: {}", details);
        }
    }
}
