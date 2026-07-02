//! MACS F2 (Component 32): evidence-computed verification-ladder level.
//!
//! Sub-spec: `docs/specifications/components/modern-agentic-coding-support.md`
//! Contract: `contracts/macs-ladder-v1.yaml#gate_monotone`
//!
//! `achieved_level` is computed from evidence on every call and NEVER stored
//! (a persisted value goes stale the moment a kani report is deleted or a
//! YAML edited). The completion gate compares it against the ticket's claimed
//! `verification_level` and blocks with `LadderShortfall` on over-claims —
//! C28's "completion gates are hard".

use crate::cli::handlers::work_contract::WorkContract;
use crate::cli::handlers::work_verification_level::VerificationLevel;
use std::path::Path;

/// Compute the verification level actually evidenced by a ticket.
///
/// Evidence ladder (C28 definitions, weakest binding wins):
/// - **L0**: nothing executable.
/// - **L1**: contract macros compile and run under `cargo test` — granted at
///   completion time because `pmat verify` (fail-fast tests) gates every
///   commit; an unbound ticket can never evidence more than L1.
/// - **L2**: bound to >=1 provable-contract equation (`implements[]`).
/// - **L3**: every bound YAML declares non-empty `falsification_tests[]`.
/// - **L4**: L3 ceiling of `kani_harnesses[]` present in every bound YAML
///   *plus* an actual execution record (`.pmat-work/<ID>/kani-report.json`
///   with `success: true` — CB-1614's evidence artifact). Declared harnesses
///   without a run never count.
/// - **L5**: the YAML ceiling reports `lean_theorem.status == "proved"` and
///   the kani execution record exists (the ladder is monotone: L5 evidence
///   subsumes L4's).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn achieved_level(project_path: &Path, contract: &WorkContract) -> VerificationLevel {
    if contract.implements.is_empty() {
        return VerificationLevel::L1;
    }

    // Min-fold the per-binding ceilings: a ticket is only as verified as its
    // weakest bound equation.
    let mut level = VerificationLevel::L5;
    for binding in &contract.implements {
        let yaml_path = project_path.join(&binding.file);
        let ceiling = match std::fs::read_to_string(&yaml_path) {
            Ok(yaml) => VerificationLevel::max_attainable_from_yaml(&yaml),
            // Bound but YAML unreadable: the binding itself is the only
            // evidence left standing.
            Err(_) => VerificationLevel::L2,
        };
        level = level.min(ceiling);
    }

    // L4+ requires an actual kani execution record, not declared harnesses.
    if level >= VerificationLevel::L4 && !kani_report_success(project_path, &contract.work_item_id)
    {
        level = VerificationLevel::L3.min(level);
    }
    level
}

/// MACS-005 completion gate: a ticket cannot close above its evidenced level.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn check_ladder_shortfall(project_path: &Path, contract: &WorkContract) -> anyhow::Result<()> {
    let claimed = contract.verification_level;
    let achieved = achieved_level(project_path, contract);
    if achieved < claimed {
        anyhow::bail!(
            "LadderShortfall: '{}' claims {} but evidence supports only {} \
             (C28: completion gates are hard).\n\
             Either add the missing evidence (bind equations with --implements, \
             add falsification_tests[]/kani-report.json/lean proof) or lower the \
             claim: pmat work migrate --levels, then edit the ticket's \
             verification_level to {}.",
            contract.work_item_id,
            claimed,
            achieved,
            achieved
        );
    }
    Ok(())
}

/// True iff `.pmat-work/<ID>/kani-report.json` exists and records success.
fn kani_report_success(project_path: &Path, work_item_id: &str) -> bool {
    let path = project_path
        .join(".pmat-work")
        .join(work_item_id)
        .join("kani-report.json");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return false;
    };
    serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| v.get("success").and_then(serde_json::Value::as_bool))
        .unwrap_or(false)
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::handlers::work_contract::ContractBinding;

    fn bound_contract(ticket: &str, yaml_rel: &str) -> WorkContract {
        let mut contract = WorkContract::new(ticket.to_string(), "deadbeef".to_string());
        contract.implements = vec![ContractBinding {
            contract: "fixture-v1".to_string(),
            equation: "eq".to_string(),
            file: std::path::PathBuf::from(yaml_rel),
            sha: "0".repeat(64),
            bound_at: chrono::Utc::now(),
        }];
        contract
    }

    fn write_yaml(project: &Path, rel: &str, body: &str) {
        let path = project.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    fn write_kani_report(project: &Path, ticket: &str, success: bool) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("kani-report.json"),
            format!("{{\"success\": {success}}}"),
        )
        .unwrap();
    }

    const YAML_L3: &str = "equations:\n  eq: {}\nfalsification_tests:\n  - id: t\n";
    const YAML_L4: &str =
        "equations:\n  eq: {}\nfalsification_tests:\n  - id: t\nkani_harnesses:\n  - name: h\n";
    const YAML_L5: &str = "equations:\n  eq: {}\nfalsification_tests:\n  - id: t\nkani_harnesses:\n  - name: h\nlean_theorem:\n  status: proved\n";
    const YAML_L5_SORRY: &str = "equations:\n  eq: {}\nfalsification_tests:\n  - id: t\nkani_harnesses:\n  - name: h\nlean_theorem:\n  status: pending\n";

    #[test]
    fn unbound_ticket_achieves_l1() {
        let project = tempfile::tempdir().unwrap();
        let contract = WorkContract::new("T-UNBOUND".to_string(), "abc".to_string());
        assert_eq!(
            achieved_level(project.path(), &contract),
            VerificationLevel::L1
        );
    }

    #[test]
    fn claim_l3_with_passing_falsification_closes() {
        let project = tempfile::tempdir().unwrap();
        write_yaml(project.path(), "contracts/fixture-v1.yaml", YAML_L3);
        let mut contract = bound_contract("T-L3", "contracts/fixture-v1.yaml");
        contract.verification_level = VerificationLevel::L3;
        assert_eq!(
            achieved_level(project.path(), &contract),
            VerificationLevel::L3
        );
        assert!(check_ladder_shortfall(project.path(), &contract).is_ok());
    }

    #[test]
    fn claim_l4_without_kani_blocks() {
        let project = tempfile::tempdir().unwrap();
        write_yaml(project.path(), "contracts/fixture-v1.yaml", YAML_L4);
        let mut contract = bound_contract("T-L4", "contracts/fixture-v1.yaml");
        contract.verification_level = VerificationLevel::L4;
        // Harnesses are declared, but no execution record exists.
        assert_eq!(
            achieved_level(project.path(), &contract),
            VerificationLevel::L3
        );
        let err = check_ladder_shortfall(project.path(), &contract).unwrap_err();
        assert!(err.to_string().contains("LadderShortfall"), "{err}");
        assert!(err.to_string().contains("claims L4"), "{err}");
    }

    #[test]
    fn claim_l4_with_kani_report_closes() {
        let project = tempfile::tempdir().unwrap();
        write_yaml(project.path(), "contracts/fixture-v1.yaml", YAML_L4);
        write_kani_report(project.path(), "T-L4OK", true);
        let mut contract = bound_contract("T-L4OK", "contracts/fixture-v1.yaml");
        contract.verification_level = VerificationLevel::L4;
        assert_eq!(
            achieved_level(project.path(), &contract),
            VerificationLevel::L4
        );
        assert!(check_ladder_shortfall(project.path(), &contract).is_ok());
    }

    #[test]
    fn failed_kani_report_does_not_count() {
        let project = tempfile::tempdir().unwrap();
        write_yaml(project.path(), "contracts/fixture-v1.yaml", YAML_L4);
        write_kani_report(project.path(), "T-L4FAIL", false);
        let mut contract = bound_contract("T-L4FAIL", "contracts/fixture-v1.yaml");
        contract.verification_level = VerificationLevel::L4;
        assert_eq!(
            achieved_level(project.path(), &contract),
            VerificationLevel::L3
        );
    }

    #[test]
    fn claim_l5_with_sorry_blocks() {
        let project = tempfile::tempdir().unwrap();
        write_yaml(project.path(), "contracts/fixture-v1.yaml", YAML_L5_SORRY);
        write_kani_report(project.path(), "T-L5", true);
        let mut contract = bound_contract("T-L5", "contracts/fixture-v1.yaml");
        contract.verification_level = VerificationLevel::L5;
        // lean_theorem status != proved caps the ceiling at L4.
        assert_eq!(
            achieved_level(project.path(), &contract),
            VerificationLevel::L4
        );
        let err = check_ladder_shortfall(project.path(), &contract).unwrap_err();
        assert!(err.to_string().contains("LadderShortfall"), "{err}");
    }

    #[test]
    fn claim_l5_with_proved_lean_closes() {
        let project = tempfile::tempdir().unwrap();
        write_yaml(project.path(), "contracts/fixture-v1.yaml", YAML_L5);
        write_kani_report(project.path(), "T-L5OK", true);
        let mut contract = bound_contract("T-L5OK", "contracts/fixture-v1.yaml");
        contract.verification_level = VerificationLevel::L5;
        assert_eq!(
            achieved_level(project.path(), &contract),
            VerificationLevel::L5
        );
        assert!(check_ladder_shortfall(project.path(), &contract).is_ok());
    }

    #[test]
    fn weakest_binding_wins() {
        let project = tempfile::tempdir().unwrap();
        write_yaml(project.path(), "contracts/strong-v1.yaml", YAML_L5);
        write_yaml(project.path(), "contracts/weak-v1.yaml", YAML_L3);
        write_kani_report(project.path(), "T-MIX", true);
        let mut contract = bound_contract("T-MIX", "contracts/strong-v1.yaml");
        contract.implements.push(ContractBinding {
            contract: "weak-v1".to_string(),
            equation: "eq".to_string(),
            file: std::path::PathBuf::from("contracts/weak-v1.yaml"),
            sha: "0".repeat(64),
            bound_at: chrono::Utc::now(),
        });
        assert_eq!(
            achieved_level(project.path(), &contract),
            VerificationLevel::L3
        );
    }

    #[test]
    fn achieved_never_stored() {
        // The level is recomputed from evidence on every call: adding the
        // kani execution record raises the result without any stored state.
        let project = tempfile::tempdir().unwrap();
        write_yaml(project.path(), "contracts/fixture-v1.yaml", YAML_L4);
        let contract = bound_contract("T-FRESH", "contracts/fixture-v1.yaml");
        assert_eq!(
            achieved_level(project.path(), &contract),
            VerificationLevel::L3
        );
        write_kani_report(project.path(), "T-FRESH", true);
        assert_eq!(
            achieved_level(project.path(), &contract),
            VerificationLevel::L4,
            "evidence change must be visible without any stored/staled value"
        );
    }
}
