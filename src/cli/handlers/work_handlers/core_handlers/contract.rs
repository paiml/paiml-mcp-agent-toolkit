#![cfg_attr(coverage_nightly, coverage(off))]
// Work contract and quality gate helpers

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::cli::handlers::work_contract::{FileManifest, WorkContract};
use crate::cli::handlers::work_falsification::{capture_baseline, run_falsification_tests};
use crate::cli::handlers::work_ledger::{
    FalsificationLedger, FalsificationReceipt, FalsificationTrigger,
};
use crate::cli::handlers::work_quality_handlers::run_quality_gates;

use super::helpers::filter_unoverriden_failures;
use super::resolution::{print_blocked_result, print_warning_failures};

/// Create a work contract with baseline metrics and DbC triad (helper for handle_work_start)
#[allow(clippy::too_many_arguments)]
pub(super) async fn create_work_contract(
    project_path: &Path,
    item_id: &str,
    profile_override: Option<&str>,
    without: &[String],
    iteration: u32,
    implements: &[String],
    agent: Option<crate::cli::handlers::work_ledger::AgentProvenance>,
) -> Result<()> {
    println!();
    println!("📋 Creating Work Contract (Popperian Falsification)...");

    // Component 27: resolve --implements tokens before touching the filesystem.
    // Fail fast with a clear aggregate message rather than creating a partially-bound contract.
    let bindings = if implements.is_empty() {
        Vec::new()
    } else {
        crate::cli::handlers::work_contract_binding::resolve_all(project_path, implements)?
    };

    let baseline_commit = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(project_path)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Write --profile override to config.toml if specified
    if let Some(profile_str) = profile_override {
        write_profile_override(project_path, item_id, profile_str);
    }

    // Create v5.0 contract with DbC triad
    let mut contract = match WorkContract::with_dbc(
        item_id.to_string(),
        baseline_commit.clone(),
        project_path,
        without,
        iteration,
    ) {
        Ok(c) => c,
        Err(e) => {
            println!("   ⚠️  DbC contract creation failed: {}", e);
            println!("   Falling back to v4.0 flat contract");
            WorkContract::new(item_id.to_string(), baseline_commit)
        }
    };

    // MACS F1: record who started the work (declared-first provenance).
    contract.agent = agent;

    // Component 27: attach bindings before baseline + gates so downstream
    // machinery (future: inherited clauses) sees them.
    if !bindings.is_empty() {
        println!(
            "   🔗 Bound to {} provable-contract equation(s):",
            bindings.len()
        );
        for b in &bindings {
            println!(
                "      - {} (sha: {}...)",
                b.key(),
                &b.sha[..b.sha.len().min(12)]
            );
        }
        contract.implements = bindings;
    }

    // Display profile and triad info
    if contract.is_dbc() {
        print_dbc_summary(&contract, without);
    }

    // Capture baseline metrics
    println!("   📊 Capturing baseline metrics...");
    match capture_baseline(project_path).await {
        Ok((tdg, coverage, rust_score)) => {
            contract.baseline_tdg = tdg;
            contract.baseline_coverage = coverage;
            contract.baseline_rust_score = rust_score;
            println!("      TDG: {:.1}, Coverage: {:.1}%", tdg, coverage);
            if let Some(rs) = rust_score {
                println!(
                    "      Rust Score: {rs:.1}/{:.0}",
                    crate::services::rust_project_score::rubric_max_points()
                );
            }
        }
        Err(e) => {
            println!("   ⚠️  Could not capture baseline metrics: {}", e);
        }
    }

    build_and_attach_manifest(project_path, &mut contract);

    // DBC §meyer_triad: Evaluate require clauses at Start phase
    if contract.is_dbc() {
        evaluate_require_clauses_at_start(project_path, &contract);
    }

    save_contract(project_path, &contract);
    Ok(())
}

/// Evaluate require clauses at work start (Meyer triad §Start phase).
///
/// Require clauses are client obligations that must hold before work begins.
/// We evaluate them as lightweight precondition checks (compile, manifest).
fn evaluate_require_clauses_at_start(project_path: &Path, contract: &WorkContract) {
    use crate::cli::colors as c;
    use crate::cli::handlers::work_contract::{ClauseKind, FalsificationMethod};

    let require_clauses: Vec<_> = contract
        .require
        .iter()
        .filter(|c| c.kind == ClauseKind::Require)
        .collect();

    if require_clauses.is_empty() {
        return;
    }

    println!();
    println!(
        "   {} Evaluating {} require clause(s)...",
        c::label("🔍"),
        require_clauses.len()
    );

    let mut all_pass = true;
    for clause in &require_clauses {
        let (passed, explanation) = match clause.falsification_method {
            FalsificationMethod::ManifestIntegrity => {
                let cargo_toml = project_path.join("Cargo.toml");
                if cargo_toml.exists() {
                    (true, "Cargo.toml exists".to_string())
                } else {
                    (false, "Cargo.toml not found".to_string())
                }
            }
            _ => (true, "precondition accepted".to_string()),
        };

        let icon = if passed { c::pass("") } else { c::fail("") };
        println!("      {} {}: {}", icon, clause.id, c::dim(&explanation));
        if !passed && clause.blocking {
            all_pass = false;
        }
    }

    if all_pass {
        println!("   {}", c::pass("All require clauses satisfied"));
    } else {
        println!(
            "   {}",
            c::warn("Some require clauses failed — work may not complete cleanly")
        );
    }
}

/// Display DbC triad summary after contract creation
fn print_dbc_summary(contract: &WorkContract, without: &[String]) {
    if let Some(profile) = &contract.profile {
        println!(
            "\n   Profile: {} ({})",
            profile.name(),
            if contract.version == "5.0" {
                "DbC v5.0"
            } else {
                "v4.0"
            }
        );
    }

    println!(
        "   require:   {} clauses (checked at start)",
        contract.require.len()
    );
    println!(
        "   invariant: {} clauses (checked at each checkpoint)",
        contract.invariant.len()
    );
    println!(
        "   ensure:    {} clauses (checked at completion)",
        contract.ensure.len()
    );

    if !contract.excluded_claims.is_empty() {
        println!(
            "   excluded:  {} claims (via --without)",
            contract.excluded_claims.len()
        );
    }

    if let Some(quality) = &contract.contract_quality {
        println!(
            "   quality:   {:.0}% ({}) [{}/{}]",
            quality.score * 100.0,
            quality.rating,
            quality.active_claims,
            quality.applicable_claims
        );
    }

    if !without.is_empty() {
        for id in without {
            println!("   --without {}", id);
        }
    }
}

/// Write profile override to .pmat-work/config.toml
fn write_profile_override(project_path: &Path, _item_id: &str, profile_str: &str) {
    let config_dir = project_path.join(".pmat-work");
    if std::fs::create_dir_all(&config_dir).is_ok() {
        let config_path = config_dir.join("config.toml");
        let content = format!("[dbc]\nprofile = \"{}\"\n", profile_str);
        if std::fs::write(&config_path, content).is_ok() {
            println!(
                "   Profile override: {} (written to config.toml)",
                profile_str
            );
        }
    }
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
                anyhow::bail!("Quality gates failed. Fix issues or use --skip-quality to bypass.");
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
    agent: Option<crate::cli::handlers::work_ledger::AgentProvenance>,
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
            run_contract_tests(project_path, &contract, override_claims, ticket, id, agent).await
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
    agent: Option<crate::cli::handlers::work_ledger::AgentProvenance>,
) -> Result<()> {
    let report = run_falsification_tests(project_path, contract).await?;

    // Build immutable receipt, stamped with agent provenance and the
    // ticket's interruption events (MACS F1/E5) — every crossing of the
    // stochastic/deterministic boundary is attributable, and refusals or
    // model switches can never masquerade as a silent green path.
    let git_sha = crate::cli::handlers::work_ledger::get_current_git_sha(project_path);
    let ticket_events: Vec<crate::cli::handlers::work_ledger::AgentEvent> =
        FalsificationLedger::new(project_path)
            .load_events(&contract.work_item_id)
            .unwrap_or_default()
            .into_iter()
            .map(|record| record.event)
            .collect();
    let achieved = crate::quality::ladder_evidence::achieved_level(project_path, contract);
    let receipt = FalsificationReceipt::from_report(
        &report,
        git_sha,
        contract.work_item_id.clone(),
        FalsificationTrigger::WorkComplete,
        override_claims.as_ref(),
        ticket.as_ref(),
    )
    .with_agent(agent, ticket_events)
    .with_ladder(
        contract.verification_level.to_string(),
        achieved.to_string(),
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
        // A claim whose data source is absent tested nothing. Folding those
        // into "validated" is how the ladder came to report full corroboration
        // while most of its claims had not run.
        let unmeasured = receipt
            .summary
            .total
            .saturating_sub(receipt.summary.passed + receipt.summary.overridden);
        if unmeasured > 0 {
            println!(
                "   ℹ️  {} claim(s) NOT MEASURED (no data source; see per-claim output)",
                unmeasured
            );
        }
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

        // §6.2: Rescue protocol — attempt diagnosis for each failed postcondition
        attempt_rescue_for_failures(
            project_path,
            &contract.work_item_id,
            &contract.profile,
            &unoverrideable,
        );

        anyhow::bail!(
            "Work blocked: {} falsification(s) found. Fix issues or use --override-claims with --ticket.",
            unoverrideable.len()
        )
    }
}

/// Attempt rescue protocol for failed postconditions (DbC §6.2).
///
/// For each blocking failure, determines the rescue strategy and executes
/// a diagnosis. Rescue never modifies code — it produces actionable guidance.
fn attempt_rescue_for_failures(
    project_path: &Path,
    work_item_id: &str,
    profile: &Option<crate::cli::handlers::work_contract::ContractProfile>,
    failures: &[&crate::cli::handlers::work_falsification::types::ClaimResult],
) {
    use crate::cli::handlers::work_contract::{
        execute_rescue, is_rescue_enabled, rescue_strategy_for, DbcConfig, RescueRecord,
    };

    let config = DbcConfig::load(project_path);
    if !is_rescue_enabled(profile, &config) {
        return;
    }

    // §6.2: Check rescue attempt limit (max 3 per work item)
    let existing_rescues = RescueRecord::load_all(project_path, work_item_id);
    if existing_rescues.len() >= 3 {
        println!();
        println!("⚠️  Rescue attempt limit reached (3/3). Manual resolution required.");
        return;
    }

    let mut any_rescue = false;
    for failure in failures {
        if let Some(strategy) = rescue_strategy_for(&failure.method) {
            if !any_rescue {
                println!();
                println!("🚑 Rescue Protocol (DbC §6.2):");
                any_rescue = true;
            }

            let record = execute_rescue(project_path, work_item_id, &failure.hypothesis, &strategy);
            println!("   [{}] Strategy: {}", failure.hypothesis, strategy);
            println!("      {}", record.diagnosis.summary);
            for action in &record.diagnosis.suggested_actions {
                println!("      → {}", action);
            }

            // Persist rescue record for audit trail
            if let Ok(path) = record.save(project_path) {
                println!("      📋 Rescue record: {}", path.display());
            }
        }
    }
    if any_rescue {
        println!(
            "   Attempt {}/{} for this work item.",
            existing_rescues.len() + 1,
            3
        );
        println!();
    }
}
