// Work Falsification Unification — CB-1622 and CB-1625 tests plus the
// parse_falsification_log helper test.
//
// Included from `check_falsification_unification.rs` — do NOT add `use`
// imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_execution {
    use super::*;
    use tempfile::tempdir;

    // ── CB-1622 roster execution coverage tests ──────────────────────────

    #[test]
    fn roster_coverage_skips_when_no_log_files() {
        let tmp = tempdir().unwrap();
        let c = contract_with_provable("T1", "contracts/k.yaml", "rope", "rope_test");
        write_contract_json(tmp.path(), "T1", &c);
        let check = check_roster_execution_coverage(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("falsification.log"));
    }

    #[test]
    fn roster_coverage_skips_when_no_provable_entries() {
        // Contract with no ProvableContract entries — irrelevant, skip overall.
        let tmp = tempdir().unwrap();
        let c = WorkContract::new("T1".into(), "deadbeef".into());
        write_contract_json(tmp.path(), "T1", &c);
        let check = check_roster_execution_coverage(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn roster_coverage_passes_when_every_entry_has_receipt() {
        let tmp = tempdir().unwrap();
        let c = contract_with_provable("T1", "contracts/k.yaml", "rope", "rope_test");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"ts":"2026-04-18T00:00:00Z","source":"inherited","yaml":"contracts/k.yaml","equation":"rope","test_id":"rope_test","status":"pass","duration_ms":10}"#,
                "\n"
            ),
        );
        let check = check_roster_execution_coverage(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn roster_coverage_fails_when_entry_unexecuted() {
        let tmp = tempdir().unwrap();
        let c = contract_with_provable("T1", "contracts/k.yaml", "rope", "rope_test");
        write_contract_json(tmp.path(), "T1", &c);
        // Log covers a DIFFERENT test_id
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"ts":"2026-04-18T00:00:00Z","yaml":"contracts/k.yaml","equation":"rope","test_id":"other_test","status":"pass"}"#,
                "\n"
            ),
        );
        let check = check_roster_execution_coverage(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("T1"));
        assert!(check.message.contains("rope_test"));
        assert!(check.message.contains("pmat work falsify"));
    }

    #[test]
    fn roster_coverage_ignores_malformed_log_lines() {
        let tmp = tempdir().unwrap();
        let c = contract_with_provable("T1", "contracts/k.yaml", "rope", "rope_test");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                "not json at all\n",
                r#"{"missing_fields": true}"#,
                "\n",
                r#"{"yaml":"contracts/k.yaml","test_id":"rope_test","status":"pass"}"#,
                "\n",
            ),
        );
        let check = check_roster_execution_coverage(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn roster_coverage_per_ticket_skip_when_only_some_have_log() {
        // T1 has a log; T2 doesn't. T2 is silently skipped.
        let tmp = tempdir().unwrap();
        let c1 = contract_with_provable("T1", "contracts/k.yaml", "rope", "rope_test");
        let c2 = contract_with_provable("T2", "contracts/k.yaml", "rope", "another_test");
        write_contract_json(tmp.path(), "T1", &c1);
        write_contract_json(tmp.path(), "T2", &c2);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"contracts/k.yaml","test_id":"rope_test","status":"pass"}"#,
                "\n"
            ),
        );
        let check = check_roster_execution_coverage(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
        assert!(check.message.contains("1 ticket"));
    }

    #[test]
    fn parse_falsification_log_extracts_valid_lines() {
        let input = concat!(
            r#"{"yaml":"a.yaml","test_id":"t1","status":"pass"}"#,
            "\n",
            "\n",
            r#"{"source":"manual","method":"TdgRegression","status":"pass"}"#,
            "\n",
            r#"{"yaml":"b.yaml","test_id":"t2","status":"fail"}"#,
            "\n",
        );
        let parsed = parse_falsification_log(input);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], (PathBuf::from("a.yaml"), "t1".into()));
        assert_eq!(parsed[1], (PathBuf::from("b.yaml"), "t2".into()));
    }

    // ── CB-1625 inherited failure fatal tests ────────────────────────────

    #[test]
    fn inherited_failure_skips_when_no_logs() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L3");
        write_contract_json(tmp.path(), "T1", &c);
        let check = check_inherited_failure_fatal(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("falsification.log"));
    }

    #[test]
    fn inherited_failure_skips_when_no_contracts_at_all() {
        let tmp = tempdir().unwrap();
        let check = check_inherited_failure_fatal(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn inherited_failure_passes_when_all_inherited_pass() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L1"); // even L1 counts
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"a.yaml","test_id":"t1","status":"pass","duration_ms":2}"#,
                "\n",
                r#"{"yaml":"a.yaml","test_id":"t2","status":"pass","duration_ms":4}"#,
                "\n",
            ),
        );
        let check = check_inherited_failure_fatal(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
        assert!(check.message.contains("2 inherited"));
    }

    #[test]
    fn inherited_failure_fails_on_inherited_fail() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L1");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"rope.yaml","test_id":"t1","status":"pass","duration_ms":2}"#,
                "\n",
                r#"{"yaml":"rope.yaml","test_id":"t2","status":"fail","duration_ms":9}"#,
                "\n",
            ),
        );
        let check = check_inherited_failure_fatal(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("T1:2"));
        assert!(check.message.contains("rope.yaml::t2"));
        assert!(check.message.contains("status=fail"));
    }

    #[test]
    fn inherited_failure_fails_on_timeout_too() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L3");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            r#"{"yaml":"k.yaml","test_id":"t","status":"timeout","duration_ms":60000}"#,
        );
        let check = check_inherited_failure_fatal(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("status=timeout"));
    }

    #[test]
    fn inherited_failure_ignores_manual_lines() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L1");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                // Manual line (method, not yaml/test_id) — ignored even though it "fails"
                r#"{"method":"UnitTest","status":"fail","duration_ms":2}"#,
                "\n",
                // Inherited line — all pass
                r#"{"yaml":"a.yaml","test_id":"t","status":"pass","duration_ms":1}"#,
                "\n",
            ),
        );
        let check = check_inherited_failure_fatal(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn inherited_failure_ignores_malformed_lines() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L1");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                "not-json\n",
                r#"{"yaml":"a.yaml","test_id":"t","status":"pass","duration_ms":1}"#,
                "\n",
            ),
        );
        // Malformed lines belong to CB-1628 — 1625 stays Pass here
        let check = check_inherited_failure_fatal(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn inherited_failure_counts_failures_across_tickets() {
        let tmp = tempdir().unwrap();
        let c1 = contract_at_level("T1", "L1");
        let c2 = contract_at_level("T2", "L1");
        write_contract_json(tmp.path(), "T1", &c1);
        write_contract_json(tmp.path(), "T2", &c2);
        write_log(
            tmp.path(),
            "T1",
            r#"{"yaml":"a.yaml","test_id":"t1","status":"fail","duration_ms":1}"#,
        );
        write_log(
            tmp.path(),
            "T2",
            r#"{"yaml":"b.yaml","test_id":"t2","status":"fail","duration_ms":1}"#,
        );
        let check = check_inherited_failure_fatal(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("2 inherited"));
        assert!(check.message.contains("T1"));
        assert!(check.message.contains("T2"));
    }
}
