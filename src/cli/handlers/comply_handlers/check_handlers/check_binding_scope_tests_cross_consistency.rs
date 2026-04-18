// Tests for CB-1608 cross-binding consistency —
// included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_cross_consistency {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn write_contract(project: &Path, ticket: &str, yaml_file: &Path, equation: &str, sha: &str) {
        let mut c = WorkContract::new(ticket.to_string(), "deadbeef".to_string());
        c.implements.push(ContractBinding {
            contract: "k".to_string(),
            equation: equation.to_string(),
            file: yaml_file.to_path_buf(),
            sha: sha.to_string(),
            bound_at: chrono::Utc::now(),
        });
        c.save(project).unwrap();
    }

    // ── CB-1608 cross-binding consistency tests ──────────────────────────

    fn cbc_save_two_bindings(project: &Path, ticket: &str, yaml_a: &Path, yaml_b: &Path) {
        use crate::cli::handlers::work_contract::WorkContract;
        let mut c = WorkContract::new(ticket.into(), "deadbeef".into());
        c.implements.push(ContractBinding {
            contract: "k".into(),
            equation: "rope".into(),
            file: yaml_a.to_path_buf(),
            sha: "deadbeef".into(),
            bound_at: chrono::Utc::now(),
        });
        c.implements.push(ContractBinding {
            contract: "k".into(),
            equation: "softmax".into(),
            file: yaml_b.to_path_buf(),
            sha: "deadbeef".into(),
            bound_at: chrono::Utc::now(),
        });
        c.save(project).unwrap();
    }

    fn cbc_write_log(project: &Path, ticket: &str, lines: &[&str]) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        let body = lines.join("\n") + "\n";
        std::fs::write(dir.join("falsification.log"), body).unwrap();
    }

    #[test]
    fn cbc_skips_when_no_tickets() {
        let tmp = tempdir().unwrap();
        let r = check_binding_cross_consistency(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/*/contract.json`"));
    }

    #[test]
    fn cbc_skips_when_no_multibind_ticket_has_log() {
        let tmp = tempdir().unwrap();
        // Single-bind ticket, no log
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        let r = check_binding_cross_consistency(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("multi-bind"));
    }

    #[test]
    fn cbc_skips_when_multibind_has_no_log() {
        let tmp = tempdir().unwrap();
        cbc_save_two_bindings(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/a.yaml"),
            &PathBuf::from("contracts/b.yaml"),
        );
        // no falsification.log
        let r = check_binding_cross_consistency(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
    }

    #[test]
    fn cbc_passes_when_all_bindings_green() {
        let tmp = tempdir().unwrap();
        cbc_save_two_bindings(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/a.yaml"),
            &PathBuf::from("contracts/b.yaml"),
        );
        cbc_write_log(
            tmp.path(),
            "T-1",
            &[
                r#"{"yaml":"contracts/a.yaml","equation":"rope","test_id":"t1","status":"pass"}"#,
                r#"{"yaml":"contracts/b.yaml","equation":"softmax","test_id":"t2","status":"pass"}"#,
            ],
        );
        let r = check_binding_cross_consistency(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cbc_fails_on_mixed_pass_and_fail() {
        let tmp = tempdir().unwrap();
        cbc_save_two_bindings(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/a.yaml"),
            &PathBuf::from("contracts/b.yaml"),
        );
        cbc_write_log(
            tmp.path(),
            "T-1",
            &[
                r#"{"yaml":"contracts/a.yaml","equation":"rope","test_id":"t1","status":"pass"}"#,
                r#"{"yaml":"contracts/b.yaml","equation":"softmax","test_id":"t2","status":"fail"}"#,
            ],
        );
        let r = check_binding_cross_consistency(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("passing"));
        assert!(r.message.contains("failing"));
    }

    #[test]
    fn cbc_ignores_binding_with_no_evidence() {
        // Binding B has no log entry → "unknown". Only A is green.
        // Not a mix of pass+fail → Pass.
        let tmp = tempdir().unwrap();
        cbc_save_two_bindings(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/a.yaml"),
            &PathBuf::from("contracts/b.yaml"),
        );
        cbc_write_log(
            tmp.path(),
            "T-1",
            &[r#"{"yaml":"contracts/a.yaml","equation":"rope","test_id":"t1","status":"pass"}"#],
        );
        let r = check_binding_cross_consistency(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cbc_passes_when_binding_pass_and_other_untested() {
        let tmp = tempdir().unwrap();
        cbc_save_two_bindings(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/a.yaml"),
            &PathBuf::from("contracts/b.yaml"),
        );
        // only empty log (no entries for either binding)
        cbc_write_log(tmp.path(), "T-1", &[]);
        let r = check_binding_cross_consistency(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cbc_parses_mixed_status_within_single_binding_as_failing() {
        // One binding has both pass+fail rows → that binding is "failing".
        // Pair with a green binding → Fail.
        let tmp = tempdir().unwrap();
        cbc_save_two_bindings(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/a.yaml"),
            &PathBuf::from("contracts/b.yaml"),
        );
        cbc_write_log(
            tmp.path(),
            "T-1",
            &[
                r#"{"yaml":"contracts/a.yaml","equation":"rope","test_id":"t1","status":"pass"}"#,
                r#"{"yaml":"contracts/a.yaml","equation":"rope","test_id":"t1","status":"pass"}"#,
                r#"{"yaml":"contracts/b.yaml","equation":"softmax","test_id":"t2","status":"pass"}"#,
                r#"{"yaml":"contracts/b.yaml","equation":"softmax","test_id":"t2","status":"fail"}"#,
            ],
        );
        let r = check_binding_cross_consistency(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
    }

    #[test]
    fn cbc_aggregates_multiple_tickets_reports_only_violators() {
        let tmp = tempdir().unwrap();
        cbc_save_two_bindings(
            tmp.path(),
            "T-GOOD",
            &PathBuf::from("contracts/a.yaml"),
            &PathBuf::from("contracts/b.yaml"),
        );
        cbc_write_log(
            tmp.path(),
            "T-GOOD",
            &[
                r#"{"yaml":"contracts/a.yaml","equation":"rope","test_id":"t1","status":"pass"}"#,
                r#"{"yaml":"contracts/b.yaml","equation":"softmax","test_id":"t2","status":"pass"}"#,
            ],
        );

        cbc_save_two_bindings(
            tmp.path(),
            "T-BAD",
            &PathBuf::from("contracts/a.yaml"),
            &PathBuf::from("contracts/b.yaml"),
        );
        cbc_write_log(
            tmp.path(),
            "T-BAD",
            &[
                r#"{"yaml":"contracts/a.yaml","equation":"rope","test_id":"t1","status":"pass"}"#,
                r#"{"yaml":"contracts/b.yaml","equation":"softmax","test_id":"t2","status":"fail"}"#,
            ],
        );

        let r = check_binding_cross_consistency(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("T-BAD"));
        assert!(!r.message.contains("T-GOOD"), "{}", r.message);
    }

    #[test]
    fn cbc_ignores_malformed_log_lines() {
        let tmp = tempdir().unwrap();
        cbc_save_two_bindings(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/a.yaml"),
            &PathBuf::from("contracts/b.yaml"),
        );
        cbc_write_log(
            tmp.path(),
            "T-1",
            &[
                "not-json-at-all",
                r#"{"yaml":"contracts/a.yaml","equation":"rope","test_id":"t1","status":"pass"}"#,
                r#"{"yaml":"contracts/b.yaml","equation":"softmax","test_id":"t2","status":"pass"}"#,
            ],
        );
        let r = check_binding_cross_consistency(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cbc_parse_inherited_log_entries_skips_manual_rows() {
        // Manual rows (no yaml field) must be skipped.
        let log = concat!(
            "{\"method\":\"Kani\",\"status\":\"pass\"}\n",
            "{\"yaml\":\"a.yaml\",\"equation\":\"rope\",\"test_id\":\"t1\",\"status\":\"pass\"}\n",
        );
        let entries = parse_inherited_log_entries(log);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].1, "rope");
    }
}
