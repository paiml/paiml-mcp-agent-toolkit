// Tests for CB-1605 Kani harness execution —
// included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_kani {
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

    fn write_yaml(project: &Path, name: &str, body: &str) -> PathBuf {
        let dir = project.join("contracts");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join(format!("{}.yaml", name));
        std::fs::write(&p, body).unwrap();
        p
    }

    // ── CB-1605 Kani harness execution tests ─────────────────────────────

    fn write_kani_report(project: &Path, ticket: &str, body: &str) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("kani-report.json"), body).unwrap();
    }

    #[test]
    fn yaml_kani_harnesses_parses_block_style() {
        let s = "equations:\n  rope: {}\nkani_harnesses:\n- verify_a\n- verify_b\n";
        let names = yaml_kani_harness_names(s).unwrap();
        assert_eq!(names, vec!["verify_a".to_string(), "verify_b".to_string()]);
    }

    #[test]
    fn yaml_kani_harnesses_parses_indented_style() {
        let s = "kani_harnesses:\n  - verify_a\n  - verify_b\n";
        let names = yaml_kani_harness_names(s).unwrap();
        assert_eq!(names, vec!["verify_a".to_string(), "verify_b".to_string()]);
    }

    #[test]
    fn yaml_kani_harnesses_parses_object_form() {
        let s = "kani_harnesses:\n- name: verify_a\n- name: verify_b\n";
        let names = yaml_kani_harness_names(s).unwrap();
        assert_eq!(names, vec!["verify_a".to_string(), "verify_b".to_string()]);
    }

    #[test]
    fn yaml_kani_harnesses_flow_empty_returns_none() {
        let s = "kani_harnesses: []\n";
        assert!(yaml_kani_harness_names(s).is_none());
    }

    #[test]
    fn yaml_kani_harnesses_missing_returns_none() {
        let s = "equations:\n  rope: {}\n";
        assert!(yaml_kani_harness_names(s).is_none());
    }

    #[test]
    fn yaml_kani_harnesses_ignores_comments_and_blanks() {
        let s = "kani_harnesses:\n# comment\n- verify_a\n\n- verify_b\n";
        let names = yaml_kani_harness_names(s).unwrap();
        assert_eq!(names, vec!["verify_a".to_string(), "verify_b".to_string()]);
    }

    #[test]
    fn parse_kani_report_canonical_shape() {
        let r = r#"{"harnesses":[{"name":"h1","success":true},{"name":"h2","success":false}]}"#;
        let out = parse_kani_harness_results(r).unwrap();
        assert_eq!(
            out,
            vec![("h1".to_string(), true), ("h2".to_string(), false)]
        );
    }

    #[test]
    fn parse_kani_report_results_alias() {
        let r = r#"{"results":[{"name":"h1","success":true}]}"#;
        let out = parse_kani_harness_results(r).unwrap();
        assert_eq!(out, vec![("h1".to_string(), true)]);
    }

    #[test]
    fn parse_kani_report_status_string_coerces() {
        let r =
            r#"{"harnesses":[{"name":"h1","status":"proved"},{"name":"h2","status":"failed"}]}"#;
        let out = parse_kani_harness_results(r).unwrap();
        assert_eq!(
            out,
            vec![("h1".to_string(), true), ("h2".to_string(), false)]
        );
    }

    #[test]
    fn kani_harnesses_skip_without_bindings() {
        let tmp = tempdir().unwrap();
        let r = check_binding_kani_harnesses(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("implements:"));
    }

    #[test]
    fn kani_harnesses_skip_when_no_yaml_declares() {
        let tmp = tempdir().unwrap();
        write_yaml(tmp.path(), "k", "equations:\n  rope: {}\n");
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        let r = check_binding_kani_harnesses(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("kani_harnesses"));
    }

    #[test]
    fn kani_harnesses_skip_when_declared_but_no_report() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nkani_harnesses:\n- verify_a\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        let r = check_binding_kani_harnesses(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("kani-report.json"));
    }

    #[test]
    fn kani_harnesses_pass_when_all_harnesses_succeed() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nkani_harnesses:\n- verify_a\n- verify_b\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        write_kani_report(
            tmp.path(),
            "T-1",
            r#"{"success":true,"harnesses":[{"name":"verify_a","success":true},{"name":"verify_b","success":true}]}"#,
        );
        let r = check_binding_kani_harnesses(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("succeeded"));
    }

    #[test]
    fn kani_harnesses_fails_when_declared_harness_missing() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nkani_harnesses:\n- verify_a\n- verify_b\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        // Report only covers verify_a — verify_b is missing
        write_kani_report(
            tmp.path(),
            "T-1",
            r#"{"harnesses":[{"name":"verify_a","success":true}]}"#,
        );
        let r = check_binding_kani_harnesses(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("verify_b"));
        assert!(r.message.contains("T-1"));
    }

    #[test]
    fn kani_harnesses_fails_when_harness_failed() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nkani_harnesses:\n- verify_a\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        write_kani_report(
            tmp.path(),
            "T-1",
            r#"{"harnesses":[{"name":"verify_a","success":false}]}"#,
        );
        let r = check_binding_kani_harnesses(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("verify_a"));
        assert!(r.message.contains("success=false"));
    }

    #[test]
    fn kani_harnesses_fails_on_malformed_report() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nkani_harnesses:\n- verify_a\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        // Report is valid JSON but lacks harnesses/results array
        write_kani_report(tmp.path(), "T-1", r#"{"success":true}"#);
        let r = check_binding_kani_harnesses(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("missing `harnesses`"));
    }
}
