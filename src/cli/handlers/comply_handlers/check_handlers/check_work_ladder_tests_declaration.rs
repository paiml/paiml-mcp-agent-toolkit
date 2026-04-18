// Tests for CB-1610 (parses), CB-1611 (bound_by_yaml), CB-1617 (downgrade
// audit), and CB-1619 (completion matches). Included into
// `check_work_ladder.rs`; follows the `mod tests_<group> { use super::*; … }`
// nesting pattern so the inner `use` does not leak into the outer scope.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_work_ladder_declaration {
    use super::*;
    use crate::cli::handlers::work_contract::ContractBinding;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn parses_passes_when_all_valid() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        make_contract("T-2", "L1").save(tmp.path()).unwrap();
        let r = check_ladder_parses(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn parses_fails_on_typo() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3 ").save(tmp.path()).unwrap();
        let r = check_ladder_parses(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
    }

    #[test]
    fn parses_skips_with_no_tickets() {
        let tmp = tempdir().unwrap();
        let r = check_ladder_parses(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn bound_by_yaml_fails_overclaim() {
        let tmp = tempdir().unwrap();
        // YAML caps at L3 (has falsification_tests but no kani)
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nfalsification_tests:\n  - id: t\n",
        );
        let mut c = make_contract("T-1", "L4"); // claim > max
        c.implements.push(ContractBinding {
            contract: "k".into(),
            equation: "rope".into(),
            file: PathBuf::from("contracts/k.yaml"),
            sha: "deadbeef".into(),
            bound_at: chrono::Utc::now(),
        });
        c.save(tmp.path()).unwrap();

        let r = check_ladder_bound_by_yaml(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
    }

    #[test]
    fn bound_by_yaml_passes_when_within_ceiling() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nkani_harnesses:\n  - name: h\n",
        );
        let mut c = make_contract("T-1", "L3");
        c.implements.push(ContractBinding {
            contract: "k".into(),
            equation: "rope".into(),
            file: PathBuf::from("contracts/k.yaml"),
            sha: "deadbeef".into(),
            bound_at: chrono::Utc::now(),
        });
        c.save(tmp.path()).unwrap();

        let r = check_ladder_bound_by_yaml(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn bound_by_yaml_weakest_binding_dominates() {
        let tmp = tempdir().unwrap();
        // Two bindings: one caps at L4 (kani), one caps at L2 (equations only)
        write_yaml(
            tmp.path(),
            "strong",
            "equations:\n  e: {}\nkani_harnesses:\n  - name: h\n",
        );
        write_yaml(tmp.path(), "weak", "equations:\n  e: {}\n");
        let mut c = make_contract("T-1", "L3"); // L3 > L2 weakest → fail
        c.implements.push(ContractBinding {
            contract: "strong".into(),
            equation: "e".into(),
            file: PathBuf::from("contracts/strong.yaml"),
            sha: "abc".into(),
            bound_at: chrono::Utc::now(),
        });
        c.implements.push(ContractBinding {
            contract: "weak".into(),
            equation: "e".into(),
            file: PathBuf::from("contracts/weak.yaml"),
            sha: "def".into(),
            bound_at: chrono::Utc::now(),
        });
        c.save(tmp.path()).unwrap();

        let r = check_ladder_bound_by_yaml(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("cap at L2"));
    }

    #[test]
    fn bound_by_yaml_skips_unbound_only() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap(); // no bindings
        let r = check_ladder_bound_by_yaml(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn downgrade_audit_passes_when_all_reasons_present() {
        let tmp = tempdir().unwrap();
        let ledger_dir = tmp.path().join(".pmat-work").join("ledger");
        std::fs::create_dir_all(&ledger_dir).unwrap();
        let ledger = ledger_dir.join("downgrades.json");
        std::fs::write(
            &ledger,
            r#"[{"ticket":"T-1","reason":"blocked on kani"},{"ticket":"T-2","reason":"scope cut"}]"#,
        )
        .unwrap();
        let r = check_ladder_downgrade_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn downgrade_audit_fails_on_missing_reason() {
        let tmp = tempdir().unwrap();
        let ledger_dir = tmp.path().join(".pmat-work").join("ledger");
        std::fs::create_dir_all(&ledger_dir).unwrap();
        std::fs::write(
            ledger_dir.join("downgrades.json"),
            r#"[{"ticket":"T-1","reason":""}]"#,
        )
        .unwrap();
        let r = check_ladder_downgrade_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
    }

    #[test]
    fn downgrade_audit_skips_when_ledger_missing() {
        let tmp = tempdir().unwrap();
        let r = check_ladder_downgrade_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn completion_matches_passes_when_equal() {
        let tmp = tempdir().unwrap();
        let c = make_contract("T-1", "L3");
        c.save(tmp.path()).unwrap();
        let dir = tmp.path().join(".pmat-work").join("T-1");
        std::fs::write(
            dir.join("verification-report.json"),
            r#"{"target_level":"L3","achieved_level":"L3"}"#,
        )
        .unwrap();
        let r = check_ladder_completion_matches(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn completion_matches_fails_on_giveup() {
        let tmp = tempdir().unwrap();
        let c = make_contract("T-1", "L3");
        c.save(tmp.path()).unwrap();
        let dir = tmp.path().join(".pmat-work").join("T-1");
        std::fs::write(
            dir.join("verification-report.json"),
            r#"{"target_level":"L4","achieved_level":"L2"}"#,
        )
        .unwrap();
        let r = check_ladder_completion_matches(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
    }

    #[test]
    fn completion_matches_skips_without_report() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        let r = check_ladder_completion_matches(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }
}
