// Tests for CB-1612 (L1 Test Evidence). Included into
// `check_work_ladder.rs`.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_work_ladder_l1_evidence {
    use super::*;
    use tempfile::tempdir;

    // ─── CB-1612: L1 test evidence ───────────────────────────────────────────

    fn write_verification_report(project: &Path, id: &str, body: &str) {
        let dir = project.join(".pmat-work").join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("verification-report.json"), body).unwrap();
    }

    #[test]
    fn l1_evidence_skips_with_no_tickets() {
        let tmp = tempdir().unwrap();
        let r = check_ladder_l1_test_evidence(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/*/contract.json`"));
    }

    #[test]
    fn l1_evidence_skips_when_no_report_yet() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        let r = check_ladder_l1_test_evidence(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("verification-report.json"));
    }

    #[test]
    fn l1_evidence_skips_when_report_lacks_evidence_field() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        write_verification_report(tmp.path(), "T-1", r#"{"target_level":"L3"}"#);
        let r = check_ladder_l1_test_evidence(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("l1_test_evidence"));
    }

    #[test]
    fn l1_evidence_passes_on_boolean_true() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        write_verification_report(tmp.path(), "T-1", r#"{"l1_test_evidence": true}"#);
        let r = check_ladder_l1_test_evidence(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn l1_evidence_passes_on_success_object() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        write_verification_report(
            tmp.path(),
            "T-1",
            r#"{"l1_test_evidence": {"success": true}}"#,
        );
        let r = check_ladder_l1_test_evidence(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
    }

    #[test]
    fn l1_evidence_passes_on_exit_code_zero() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        write_verification_report(
            tmp.path(),
            "T-1",
            r#"{"l1_test_evidence": {"exit_code": 0, "duration_ms": 42}}"#,
        );
        let r = check_ladder_l1_test_evidence(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
    }

    #[test]
    fn l1_evidence_passes_on_status_pass_variants() {
        for variant in ["pass", "Passed", "OK", "success"] {
            let tmp = tempdir().unwrap();
            make_contract("T-1", "L3").save(tmp.path()).unwrap();
            write_verification_report(
                tmp.path(),
                "T-1",
                &format!(r#"{{"l1_test_evidence": {{"status": "{}"}}}}"#, variant),
            );
            let r = check_ladder_l1_test_evidence(tmp.path());
            assert_eq!(r.status, CheckStatus::Pass, "{}: {}", variant, r.message);
        }
    }

    #[test]
    fn l1_evidence_fails_on_boolean_false() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        write_verification_report(tmp.path(), "T-1", r#"{"l1_test_evidence": false}"#);
        let r = check_ladder_l1_test_evidence(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("evidence=false"));
    }

    #[test]
    fn l1_evidence_fails_on_nonzero_exit_code() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        write_verification_report(
            tmp.path(),
            "T-1",
            r#"{"l1_test_evidence": {"exit_code": 101}}"#,
        );
        let r = check_ladder_l1_test_evidence(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("exit_code=101"));
    }

    #[test]
    fn l1_evidence_fails_on_failure_status() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        write_verification_report(
            tmp.path(),
            "T-1",
            r#"{"l1_test_evidence": {"status": "fail"}}"#,
        );
        let r = check_ladder_l1_test_evidence(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("status=fail"));
    }

    #[test]
    fn l1_evidence_fails_on_unrecognized_shape() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        // neither boolean, success, exit_code, nor status fields
        write_verification_report(
            tmp.path(),
            "T-1",
            r#"{"l1_test_evidence": {"note": "skipped"}}"#,
        );
        let r = check_ladder_l1_test_evidence(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("unrecognized"));
    }

    #[test]
    fn l1_evidence_aggregates_across_tickets() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        make_contract("T-2", "L3").save(tmp.path()).unwrap();
        write_verification_report(tmp.path(), "T-1", r#"{"l1_test_evidence": true}"#);
        write_verification_report(
            tmp.path(),
            "T-2",
            r#"{"l1_test_evidence": {"exit_code": 1}}"#,
        );
        let r = check_ladder_l1_test_evidence(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("1 ticket"));
        assert!(r.message.contains("T-2"));
        assert!(!r.message.contains("T-1 →"));
    }

    #[test]
    fn l1_evidence_skips_when_report_is_malformed_json() {
        // Malformed report is silently skipped — CB-1619/other checks
        // own structural validation. This check only consumes l1_test_evidence.
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        write_verification_report(tmp.path(), "T-1", "not-json");
        let r = check_ladder_l1_test_evidence(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }
}
