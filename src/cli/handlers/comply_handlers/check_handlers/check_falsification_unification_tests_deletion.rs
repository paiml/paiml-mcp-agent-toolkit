// Work Falsification Unification — CB-1624 manual-deletion audit tests.
//
// Included from `check_falsification_unification.rs` — do NOT add `use`
// imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_deletion {
    use super::*;
    use tempfile::tempdir;

    fn write_roster_mutations(project: &Path, body: &str) {
        let dir = project.join(".pmat-work").join("ledger");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("roster-mutations.json"), body).unwrap();
    }

    #[test]
    fn manual_deletion_skips_without_pmat_work_dir() {
        let tmp = tempdir().unwrap();
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/`"));
    }

    #[test]
    fn manual_deletion_skips_without_ledger() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-work")).unwrap();
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("roster-mutations.json"));
    }

    #[test]
    fn manual_deletion_passes_on_empty_ledger() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(tmp.path(), "[]");
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
        assert!(r.message.contains("empty"));
    }

    #[test]
    fn manual_deletion_passes_when_no_delete_entries() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(
            tmp.path(),
            r#"[
                {"ticket":"T-1","action":"add","target":{"yaml":"a.yaml","test_id":"t1"}},
                {"ticket":"T-1","action":"update","target":{"yaml":"a.yaml","test_id":"t2"}}
            ]"#,
        );
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
        assert!(r.message.contains("0 deletion"));
    }

    #[test]
    fn manual_deletion_passes_when_delete_is_via_unbind() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(
            tmp.path(),
            r#"[
                {"ticket":"T-1","action":"delete","target":{"yaml":"a.yaml","test_id":"t1"},"via_unbind":true},
                {"ticket":"T-2","action":"delete","target":{"yaml":"b.yaml","test_id":"t2"},"via_unbind":true}
            ]"#,
        );
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
        assert!(r.message.contains("2 deletion"));
        assert!(r.message.contains("via `pmat work unbind`"));
    }

    #[test]
    fn manual_deletion_fails_on_delete_without_via_unbind_flag() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(
            tmp.path(),
            r#"[
                {"ticket":"T-1","action":"delete","target":{"yaml":"a.yaml","test_id":"t1"}}
            ]"#,
        );
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("bypassed"));
    }

    #[test]
    fn manual_deletion_fails_when_via_unbind_is_false() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(
            tmp.path(),
            r#"[
                {"ticket":"T-1","action":"delete","target":{"yaml":"a.yaml","test_id":"t1"},"via_unbind":false}
            ]"#,
        );
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
    }

    #[test]
    fn manual_deletion_detects_capitalized_action_variants() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(
            tmp.path(),
            r#"[
                {"ticket":"T-1","action":"Delete","target":{"yaml":"a.yaml","test_id":"t1"}},
                {"ticket":"T-2","action":"DELETED","target":{"yaml":"b.yaml","test_id":"t2"}},
                {"ticket":"T-3","action":"deletion","target":{"yaml":"c.yaml","test_id":"t3"}}
            ]"#,
        );
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("3 manual deletion"));
    }

    #[test]
    fn manual_deletion_mixed_ledger_reports_only_offenders() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(
            tmp.path(),
            r#"[
                {"ticket":"T-1","action":"add","target":{"yaml":"a.yaml","test_id":"t1"}},
                {"ticket":"T-2","action":"delete","target":{"yaml":"b.yaml","test_id":"t2"},"via_unbind":true},
                {"ticket":"T-3","action":"delete","target":{"yaml":"c.yaml","test_id":"t3"}}
            ]"#,
        );
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("1 manual deletion"));
        assert!(r.message.contains("T-3"));
        assert!(!r.message.contains("T-2"));
    }

    #[test]
    fn manual_deletion_fails_on_malformed_json() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(tmp.path(), "{not-json");
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("not valid JSON"));
    }

    #[test]
    fn manual_deletion_fails_on_non_array_ledger() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(tmp.path(), r#"{"ticket":"T-1"}"#);
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("JSON array"));
    }

    #[test]
    fn manual_deletion_handles_missing_target_field() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(
            tmp.path(),
            r#"[
                {"ticket":"T-1","action":"delete"}
            ]"#,
        );
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
    }
}
