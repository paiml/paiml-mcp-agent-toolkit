// Work Falsification Unification — CB-1628 per-run log line tests.
//
// Included from `check_falsification_unification.rs` — do NOT add `use`
// imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_log_line {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn per_run_log_skips_when_no_logs() {
        let tmp = tempdir().unwrap();
        let check = check_per_run_log_line(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn per_run_log_passes_when_all_inherited_lines_complete() {
        let tmp = tempdir().unwrap();
        let c = contract_with_provable("T1", "a.yaml", "e", "t1");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"a.yaml","test_id":"t1","status":"pass","duration_ms":12}"#,
                "\n",
                r#"{"yaml":"a.yaml","test_id":"t2","status":"fail","duration_ms":99}"#,
                "\n",
            ),
        );
        let check = check_per_run_log_line(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn per_run_log_ignores_manual_lines() {
        // Manual lines carry `method`, not `yaml` — not subject to field check.
        let tmp = tempdir().unwrap();
        let c = WorkContract::new("T1".into(), "deadbeef".into());
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"method":"TdgRegression","status":"pass","duration_ms":100}"#,
                "\n",
            ),
        );
        let check = check_per_run_log_line(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn per_run_log_fails_on_missing_duration_ms() {
        let tmp = tempdir().unwrap();
        let c = contract_with_provable("T1", "a.yaml", "e", "t1");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(r#"{"yaml":"a.yaml","test_id":"t1","status":"pass"}"#, "\n",),
        );
        let check = check_per_run_log_line(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("duration_ms"));
        assert!(check.message.contains("T1"));
    }

    #[test]
    fn per_run_log_fails_on_malformed_json() {
        let tmp = tempdir().unwrap();
        let c = WorkContract::new("T1".into(), "deadbeef".into());
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"a.yaml","test_id":"t1","status":"pass","duration_ms":1}"#,
                "\n",
                "not json{\n",
            ),
        );
        let check = check_per_run_log_line(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("malformed"));
    }

    #[test]
    fn per_run_log_empty_lines_ignored() {
        let tmp = tempdir().unwrap();
        let c = WorkContract::new("T1".into(), "deadbeef".into());
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"a.yaml","test_id":"t1","status":"pass","duration_ms":1}"#,
                "\n",
                "\n",
                "   \n",
                r#"{"yaml":"b.yaml","test_id":"t2","status":"pass","duration_ms":2}"#,
                "\n",
            ),
        );
        let check = check_per_run_log_line(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }
}
