// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

#[cfg(all(test, not(coverage_nightly)))]
mod tests_macs_ladder {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    fn write_minimal_contract(project: &Path, ticket: &str, level: &str) {
        let base = crate::cli::handlers::work_contract::WorkContract::new(
            ticket.to_string(),
            "deadbeef".to_string(),
        );
        let mut value = serde_json::to_value(&base).unwrap();
        value["verification_level"] = serde_json::Value::String(level.to_string());
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("contract.json"),
            serde_json::to_string_pretty(&value).unwrap(),
        )
        .unwrap();
    }

    fn write_leveled_receipt(
        project: &Path,
        ticket: &str,
        claimed: &str,
        achieved: &str,
        allows: bool,
    ) {
        let dir = project.join(".pmat-work").join(ticket).join("falsification");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("receipt-2026-07-02T00-00-00.json"),
            serde_json::to_string_pretty(&json!({
                "schema_version": 2, "id": "r", "work_item_id": ticket,
                "claimed_level": claimed, "achieved_level": achieved,
                "summary": {"allows_completion": allows}
            }))
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn cb1653_skips_on_empty_store() {
        let project = tempdir().unwrap();
        std::fs::create_dir_all(project.path().join(".pmat-work")).unwrap();
        let check = check_ladder_claim_drift(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn cb1653_red_on_drift() {
        // A receipt that closed a ticket above its evidenced level = Fail.
        let project = tempdir().unwrap();
        write_leveled_receipt(project.path(), "T-OVER", "L4", "L1", true);
        let check = check_ladder_claim_drift(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("closed above"), "{}", check.message);
    }

    #[test]
    fn cb1653_blocked_receipt_is_not_drift() {
        // The gate did its job: claimed>achieved with allows_completion=false.
        let project = tempdir().unwrap();
        write_leveled_receipt(project.path(), "T-BLOCKED", "L4", "L1", false);
        let check = check_ladder_claim_drift(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn cb1653_warns_on_open_overclaim() {
        // An unbound contract claiming L3 evidences only L1 — advisory warn.
        let project = tempdir().unwrap();
        write_minimal_contract(project.path(), "T-OPEN", "L3");
        let check = check_ladder_claim_drift(project.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("T-OPEN"), "{}", check.message);
    }

    #[test]
    fn cb1653_green_after_fix() {
        // Claim lowered to the evidenced level: green.
        let project = tempdir().unwrap();
        write_minimal_contract(project.path(), "T-FIXED", "L1");
        write_leveled_receipt(project.path(), "T-FIXED", "L1", "L1", true);
        let check = check_ladder_claim_drift(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }
}
