// Tests for L3/L4/L5 level-specific evidence checks: CB-1613 (L3
// falsification), CB-1614 (L4 Kani), and CB-1616 (L5 Lean). Included into
// `check_work_ladder.rs`.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_work_ladder_levels {
    use super::*;
    use tempfile::tempdir;

    // ─── CB-1613: L3 falsification evidence ──────────────────────────────────

    fn write_falsification_log(project: &Path, id: &str, body: &str) {
        let dir = project.join(".pmat-work").join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("falsification.log"), body).unwrap();
    }

    #[test]
    fn l3_falsification_skips_with_no_tickets() {
        let tmp = tempdir().unwrap();
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/*/contract.json`"));
    }

    #[test]
    fn l3_falsification_skips_with_only_l1_tickets() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L1").save(tmp.path()).unwrap();
        make_contract("T-2", "L2").save(tmp.path()).unwrap();
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L3+"));
    }

    #[test]
    fn l3_falsification_skips_when_no_log_yet() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        make_contract("T-2", "L4 (kani_proof)")
            .save(tmp.path())
            .unwrap();
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L3+ ticket has a"));
        assert!(r.message.contains("2 eligible"));
    }

    #[test]
    fn l3_falsification_passes_when_all_entries_pass() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        write_falsification_log(
            tmp.path(),
            "T-1",
            concat!(
                r#"{"yaml":"k.yaml","test_id":"t1","status":"pass","duration_ms":5}"#,
                "\n",
                r#"{"method":"rope","status":"pass","duration_ms":2}"#,
                "\n",
            ),
        );
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn l3_falsification_fails_on_failing_entry() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        write_falsification_log(
            tmp.path(),
            "T-1",
            r#"{"yaml":"k.yaml","test_id":"t1","status":"fail","duration_ms":5}"#,
        );
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("status=fail"));
    }

    #[test]
    fn l3_falsification_fails_on_timeout() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        write_falsification_log(
            tmp.path(),
            "T-1",
            r#"{"yaml":"k.yaml","test_id":"t1","status":"timeout","duration_ms":30000}"#,
        );
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("status=timeout"));
    }

    #[test]
    fn l3_falsification_fails_on_malformed_json() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        write_falsification_log(tmp.path(), "T-1", "not-json\n");
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("malformed JSON"));
    }

    #[test]
    fn l3_falsification_ignores_below_l3() {
        let tmp = tempdir().unwrap();
        // L2 ticket with a failing log — must NOT fail the check
        make_contract("T-1", "L2").save(tmp.path()).unwrap();
        write_falsification_log(
            tmp.path(),
            "T-1",
            r#"{"yaml":"k.yaml","test_id":"t1","status":"fail"}"#,
        );
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L3+"));
    }

    #[test]
    fn l3_falsification_per_ticket_skip_when_some_have_no_log() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        make_contract("T-2", "L3").save(tmp.path()).unwrap();
        // Only T-1 has a log; T-2 is in-progress
        write_falsification_log(
            tmp.path(),
            "T-1",
            r#"{"yaml":"k.yaml","test_id":"t1","status":"pass","duration_ms":1}"#,
        );
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("1 L3+ log"));
    }

    #[test]
    fn is_l3_or_higher_accepts_ladder() {
        for (s, want) in [
            ("L0", false),
            ("L1", false),
            ("L2", false),
            ("L3", true),
            ("L4", true),
            ("L4 (kani_proof)", true),
            ("L5", true),
            ("strong", false),
        ] {
            let mut c = WorkContract::new("T".into(), "deadbeef".into());
            c.verification_level = VerificationLevel::parse_migrating(s);
            assert_eq!(is_l3_or_higher(&c), want, "for '{}'", s);
        }
    }

    // ─── CB-1614: L4 Kani evidence ───────────────────────────────────────────

    fn write_kani_report(project: &Path, id: &str, body: &str) {
        let dir = project.join(".pmat-work").join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("kani-report.json"), body).unwrap();
    }

    #[test]
    fn l4_kani_skips_with_no_tickets() {
        let tmp = tempdir().unwrap();
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/*/contract.json`"));
    }

    #[test]
    fn l4_kani_skips_without_l4_ticket() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L4+"));
    }

    #[test]
    fn l4_kani_skips_when_no_report_yet() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        make_contract("T-2", "L5").save(tmp.path()).unwrap();
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("2 eligible"));
    }

    #[test]
    fn l4_kani_passes_on_success_report() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        write_kani_report(
            tmp.path(),
            "T-1",
            r#"{"success":true,"harnesses":[{"name":"h","status":"pass"}]}"#,
        );
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn l4_kani_fails_on_failure_report() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        write_kani_report(tmp.path(), "T-1", r#"{"success":false}"#);
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("success=false"));
    }

    #[test]
    fn l4_kani_fails_on_malformed_report() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        write_kani_report(tmp.path(), "T-1", "not-json");
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("malformed"));
    }

    #[test]
    fn l4_kani_fails_when_success_missing() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        write_kani_report(tmp.path(), "T-1", r#"{"harnesses":[]}"#);
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("missing `success`"));
    }

    #[test]
    fn l4_kani_ignores_below_l4() {
        let tmp = tempdir().unwrap();
        // L3 ticket with a failing report — must NOT fail this check
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        write_kani_report(tmp.path(), "T-1", r#"{"success":false}"#);
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L4+"));
    }

    #[test]
    fn l4_kani_accepts_annotated_level() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4 (kani_proof)")
            .save(tmp.path())
            .unwrap();
        write_kani_report(tmp.path(), "T-1", r#"{"success":true}"#);
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn is_l4_or_higher_accepts_ladder() {
        for (s, want) in [
            ("L3", false),
            ("L4", true),
            ("L4 (kani_proof)", true),
            ("L5", true),
            ("bogus", false),
        ] {
            let mut c = WorkContract::new("T".into(), "deadbeef".into());
            c.verification_level = VerificationLevel::parse_migrating(s);
            assert_eq!(is_l4_or_higher(&c), want, "for '{}'", s);
        }
    }

    // ─── CB-1616: L5 Lean proof zero-sorry ───────────────────────────────────

    fn write_lean_proof(project: &Path, id: &str, body: &str) {
        let dir = project.join(".pmat-work").join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("lean-proof.json"), body).unwrap();
    }

    #[test]
    fn l5_lean_skips_with_no_tickets() {
        let tmp = tempdir().unwrap();
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/*/contract.json`"));
    }

    #[test]
    fn l5_lean_skips_without_l5_ticket() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L5"));
    }

    #[test]
    fn l5_lean_skips_when_no_report_yet() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L5").save(tmp.path()).unwrap();
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("1 eligible"));
    }

    #[test]
    fn l5_lean_passes_on_zero_sorry() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L5").save(tmp.path()).unwrap();
        write_lean_proof(
            tmp.path(),
            "T-1",
            r#"{"sorry_count":0,"theorems":[{"name":"rope_correct","status":"proved"}]}"#,
        );
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn l5_lean_fails_on_nonzero_sorry() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L5").save(tmp.path()).unwrap();
        write_lean_proof(tmp.path(), "T-1", r#"{"sorry_count":3}"#);
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("sorry_count=3"));
    }

    #[test]
    fn l5_lean_fails_on_negative_sorry() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L5").save(tmp.path()).unwrap();
        write_lean_proof(tmp.path(), "T-1", r#"{"sorry_count":-1}"#);
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("non-negative"));
    }

    #[test]
    fn l5_lean_fails_on_malformed_report() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L5").save(tmp.path()).unwrap();
        write_lean_proof(tmp.path(), "T-1", "not-json");
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("malformed"));
    }

    #[test]
    fn l5_lean_fails_when_sorry_count_missing() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L5").save(tmp.path()).unwrap();
        write_lean_proof(tmp.path(), "T-1", r#"{"theorems":[]}"#);
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("missing `sorry_count`"));
    }

    #[test]
    fn l5_lean_ignores_below_l5() {
        let tmp = tempdir().unwrap();
        // L4 with failing lean proof — must NOT fail this check
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        write_lean_proof(tmp.path(), "T-1", r#"{"sorry_count":5}"#);
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L5"));
    }

    #[test]
    fn is_l5_is_exact_match() {
        for (s, want) in [
            ("L3", false),
            ("L4", false),
            ("L4 (kani_proof)", false),
            ("L5", true),
            ("L5 (lean)", true),
            ("bogus", false),
        ] {
            let mut c = WorkContract::new("T".into(), "deadbeef".into());
            c.verification_level = VerificationLevel::parse_migrating(s);
            assert_eq!(is_l5(&c), want, "for '{}'", s);
        }
    }
}
