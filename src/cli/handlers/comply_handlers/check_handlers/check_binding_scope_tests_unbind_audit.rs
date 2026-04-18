// Tests for CB-1602 unbind audit —
// included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_unbind_audit {
    use super::*;
    use tempfile::tempdir;

    // ── CB-1602 unbind audit tests ────────────────────────────────────────

    fn write_unbind_ledger(project: &Path, body: &str) {
        let dir = project.join(".pmat-work").join("ledger");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("unbinds.json"), body).unwrap();
    }

    #[test]
    fn unbind_audit_skips_when_ledger_missing() {
        let tmp = tempdir().unwrap();
        let r = check_binding_unbind_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No unbind ledger"));
    }

    #[test]
    fn unbind_audit_passes_when_ledger_empty_array() {
        let tmp = tempdir().unwrap();
        write_unbind_ledger(tmp.path(), "[]");
        let r = check_binding_unbind_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("empty"));
    }

    #[test]
    fn unbind_audit_passes_when_all_have_debt_ticket() {
        let tmp = tempdir().unwrap();
        write_unbind_ledger(
            tmp.path(),
            r#"[
                {"ticket":"T-1","contract":"contracts/rope.yaml","debt_ticket":"DEBT-42"},
                {"ticket":"T-2","contract":"contracts/norm.yaml","debt_ticket":"DEBT-43"}
            ]"#,
        );
        let r = check_binding_unbind_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("2 unbind"));
    }

    #[test]
    fn unbind_audit_fails_on_missing_debt_ticket() {
        let tmp = tempdir().unwrap();
        write_unbind_ledger(
            tmp.path(),
            r#"[{"ticket":"T-1","contract":"contracts/rope.yaml"}]"#,
        );
        let r = check_binding_unbind_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("debt_ticket"));
    }

    #[test]
    fn unbind_audit_fails_on_empty_debt_ticket() {
        let tmp = tempdir().unwrap();
        write_unbind_ledger(
            tmp.path(),
            r#"[{"ticket":"T-1","contract":"c","debt_ticket":"  "}]"#,
        );
        let r = check_binding_unbind_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
    }

    #[test]
    fn unbind_audit_fails_on_malformed_json() {
        let tmp = tempdir().unwrap();
        write_unbind_ledger(tmp.path(), "not-json");
        let r = check_binding_unbind_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("not valid JSON"));
    }

    #[test]
    fn unbind_audit_fails_when_top_level_object() {
        let tmp = tempdir().unwrap();
        write_unbind_ledger(tmp.path(), r#"{"ticket":"T-1","debt_ticket":"DEBT-1"}"#);
        let r = check_binding_unbind_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("JSON array"));
    }

    #[test]
    fn unbind_audit_aggregates_multiple_failures() {
        let tmp = tempdir().unwrap();
        write_unbind_ledger(
            tmp.path(),
            r#"[
                {"ticket":"T-1","contract":"c"},
                {"ticket":"T-2","contract":"c","debt_ticket":"DEBT-2"},
                {"ticket":"T-3","contract":"c","debt_ticket":""}
            ]"#,
        );
        let r = check_binding_unbind_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("2 unbind"));
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("T-3"));
        assert!(!r.message.contains("T-2 missing"));
    }
}
