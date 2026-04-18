// Work Falsification Unification — CB-1629 L4 timeout gate tests.
//
// Included from `check_falsification_unification.rs` — do NOT add `use`
// imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_l4_timeout {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn l4_timeout_skips_when_no_l4_ticket() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L3");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"a.yaml","test_id":"t1","status":"timeout","duration_ms":60000}"#,
                "\n",
            ),
        );
        let check = check_l4_timeout_gate(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn l4_timeout_skips_when_l4_ticket_has_no_log() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L4");
        write_contract_json(tmp.path(), "T1", &c);
        let check = check_l4_timeout_gate(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn l4_timeout_passes_when_no_timeout_recorded() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L4");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"a.yaml","test_id":"t1","status":"pass","duration_ms":500}"#,
                "\n",
                r#"{"yaml":"a.yaml","test_id":"t2","status":"fail","duration_ms":200}"#,
                "\n",
            ),
        );
        let check = check_l4_timeout_gate(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn l4_timeout_fails_on_inherited_timeout() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L4 (kani_proof)");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"a.yaml","test_id":"rope_big","status":"timeout","duration_ms":60000}"#,
                "\n",
            ),
        );
        let check = check_l4_timeout_gate(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("T1"));
        assert!(check.message.contains("rope_big"));
    }

    #[test]
    fn l4_timeout_fails_on_manual_timeout_too() {
        // L4 is source-agnostic — manual-source timeouts also fail.
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L5");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"method":"TdgRegression","status":"timeout","duration_ms":300000}"#,
                "\n",
            ),
        );
        let check = check_l4_timeout_gate(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("TdgRegression"));
    }

    #[test]
    fn is_l4_or_higher_accepts_ladder() {
        let mut c = WorkContract::new("T".into(), "deadbeef".into());
        for (s, want) in [
            ("L0", false),
            ("L3", false),
            ("L4", true),
            ("L4 (kani_proof)", true),
            ("L5", true),
        ] {
            c.verification_level = s.into();
            assert_eq!(is_l4_or_higher(&c), want, "{}", s);
        }
    }
}
