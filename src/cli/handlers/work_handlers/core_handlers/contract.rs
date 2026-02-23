#![cfg_attr(coverage_nightly, coverage(off))]
// Work contract and quality gate helpers

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::cli::handlers::work_contract::{FileManifest, WorkContract};
use crate::cli::handlers::work_falsification::{
    capture_baseline, run_falsification_tests,
};
use crate::cli::handlers::work_ledger::{
    FalsificationLedger, FalsificationReceipt, FalsificationTrigger,
};
use crate::cli::handlers::work_quality_handlers::run_quality_gates;

use super::helpers::filter_unoverriden_failures;
use super::resolution::{print_blocked_result, print_warning_failures};

/// Create a work contract with baseline metrics (helper for handle_work_start)
pub(super) async fn create_work_contract(project_path: &Path, item_id: &str) {
    println!();
    println!("📋 Creating Work Contract (Popperian Falsification)...");

    let baseline_commit = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(project_path)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let mut contract = WorkContract::new(item_id.to_string(), baseline_commit);

    match capture_baseline(project_path).await {
        Ok((tdg, coverage, rust_score)) => {
            contract.baseline_tdg = tdg;
            contract.baseline_coverage = coverage;
            contract.baseline_rust_score = rust_score;
        }
        Err(e) => {
            println!("   ⚠️  Could not capture baseline metrics: {}", e);
        }
    }

    build_and_attach_manifest(project_path, &mut contract);
    save_contract(project_path, &contract);
}

/// Build file manifest and attach to contract (helper for create_work_contract)
pub(super) fn build_and_attach_manifest(project_path: &Path, contract: &mut WorkContract) {
    println!("   📂 Building file manifest...");
    match FileManifest::build(project_path) {
        Ok(manifest) => {
            let file_count = manifest.files.len();
            let protected_count = manifest.coverage_required.len();
            contract.baseline_file_manifest = manifest;
            println!(
                "      ✅ {} files tracked, {} protected from exclusion",
                file_count, protected_count
            );
        }
        Err(e) => {
            println!("   ⚠️  Could not build file manifest: {}", e);
        }
    }
}

/// Save contract to disk (helper for create_work_contract)
pub(super) fn save_contract(project_path: &Path, contract: &WorkContract) {
    match contract.save(project_path) {
        Ok(contract_path) => {
            println!("   ✅ Contract saved: {}", contract_path.display());
        }
        Err(e) => {
            println!("   ⚠️  Could not save contract: {}", e);
        }
    }
}

/// Run quality gates and report results (helper for handle_work_complete)
pub(super) async fn run_quality_check(project_path: &PathBuf, skip_quality: bool) -> Result<()> {
    if skip_quality {
        println!("⚠️  Quality gates SKIPPED (--skip-quality)");
        println!();
        return Ok(());
    }

    println!("🔍 Running quality gates...");
    println!();

    match run_quality_gates(project_path).await {
        Ok(passed) => {
            if passed {
                println!("✅ All quality gates passed");
                println!();
            } else {
                anyhow::bail!(
                    "Quality gates failed. Fix issues or use --skip-quality to bypass."
                );
            }
        }
        Err(e) => {
            println!("⚠️  Quality gates error: {}", e);
            println!("   Continuing (use strict mode to block on errors)");
            println!();
        }
    }
    Ok(())
}

/// Run contract-based falsification (helper for handle_work_complete)
///
/// Legacy mode (no contract) is now a hard error — users must run `pmat work start` first.
pub(super) async fn run_contract_falsification(
    project_path: &Path,
    item_id: &str,
    override_claims: &Option<Vec<String>>,
    ticket: &Option<String>,
    id: &str,
) -> Result<()> {
    if !WorkContract::exists(project_path, item_id) {
        anyhow::bail!(
            "No work contract found for '{}'. Run 'pmat work start {}' to create one.\n\
             Contracts are required for falsification-gated completion.",
            item_id,
            id
        );
    }

    println!("📜 Loading Work Contract...");
    match WorkContract::load(project_path, item_id) {
        Ok(contract) => {
            println!(
                "   Baseline: {} (TDG: {:.1}, Coverage: {:.1}%)",
                contract
                    .baseline_commit
                    .get(..8.min(contract.baseline_commit.len()))
                    .unwrap_or(&contract.baseline_commit),
                contract.baseline_tdg,
                contract.baseline_coverage
            );
            run_contract_tests(project_path, &contract, override_claims, ticket, id).await
        }
        Err(e) => {
            anyhow::bail!(
                "Could not load contract for '{}': {}. Re-run 'pmat work start {}' to recreate.",
                item_id,
                e,
                id
            );
        }
    }
}

/// Run falsification tests against a loaded contract, produce receipt, gate on result
pub(super) async fn run_contract_tests(
    project_path: &Path,
    contract: &WorkContract,
    override_claims: &Option<Vec<String>>,
    ticket: &Option<String>,
    id: &str,
) -> Result<()> {
    let report = run_falsification_tests(project_path, contract).await?;

    // Build immutable receipt
    let git_sha = crate::cli::handlers::work_ledger::get_current_git_sha(project_path);
    let receipt = FalsificationReceipt::from_report(
        &report,
        git_sha,
        contract.work_item_id.clone(),
        FalsificationTrigger::WorkComplete,
        override_claims.as_ref(),
        ticket.as_ref(),
    );

    // Persist receipt and append to global ledger
    let ledger = FalsificationLedger::new(project_path);
    let receipt_path = ledger.persist_receipt(&receipt)?;
    ledger.append_to_ledger(&receipt)?;

    // Gate on receipt summary
    if receipt.summary.allows_completion {
        println!(
            "✅ FALSIFICATION RESULT: PASSED ({}/{} claims validated)",
            receipt.summary.passed, receipt.summary.total
        );
        if receipt.summary.overridden > 0 {
            println!(
                "   ⚠️  {} claim(s) overridden (ticket: {})",
                receipt.summary.overridden,
                ticket.as_deref().unwrap_or("unknown")
            );
        }
        print_warning_failures(&report);
        println!("   📋 Receipt: {}", receipt_path.display());
        println!();
        Ok(())
    } else {
        let failures = report.blocking_failures();
        let unoverrideable = filter_unoverriden_failures(&failures, override_claims.as_ref());
        print_blocked_result(&report, &unoverrideable, id);
        println!("   📋 Receipt: {}", receipt_path.display());
        anyhow::bail!(
            "Work blocked: {} falsification(s) found. Fix issues or use --override-claims with --ticket.",
            unoverrideable.len()
        )
    }
}
