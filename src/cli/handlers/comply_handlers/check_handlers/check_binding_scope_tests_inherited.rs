// Tests for CB-1603 inherited clause integrity —
// included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_inherited {
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

    // ── CB-1603 inherited clause integrity tests ─────────────────────────

    #[test]
    fn yaml_precond_parses_list() {
        let s = "equations:\n  rope:\n    preconditions:\n    - \"foo\"\n    - \"bar\"\n  softmax: {}\n";
        let p = yaml_equation_preconditions(s, "rope").unwrap();
        assert_eq!(p, vec!["foo".to_string(), "bar".to_string()]);
    }

    #[test]
    fn yaml_precond_returns_none_for_missing_equation() {
        let s = "equations:\n  rope:\n    preconditions:\n    - \"foo\"\n";
        assert!(yaml_equation_preconditions(s, "softmax").is_none());
    }

    #[test]
    fn yaml_precond_returns_none_when_field_absent() {
        let s = "equations:\n  rope:\n    invariants:\n    - \"x\"\n";
        assert!(yaml_equation_preconditions(s, "rope").is_none());
    }

    #[test]
    fn inherited_clauses_skip_without_bindings() {
        let tmp = tempdir().unwrap();
        let r = check_binding_inherited_clauses(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn inherited_clauses_skip_when_no_yaml_preconds() {
        let tmp = tempdir().unwrap();
        write_yaml(tmp.path(), "k", "equations:\n  rope: {}\n");
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        let r = check_binding_inherited_clauses(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("YAML preconditions"));
    }

    #[test]
    fn inherited_clauses_fails_when_require_missing_entry() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope:\n    preconditions:\n    - \"input normalized\"\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        // Ticket's contract.require is empty → inheritance broken
        let r = check_binding_inherited_clauses(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("input normalized"));
    }

    #[test]
    fn inherited_clauses_passes_when_require_has_description() {
        use crate::cli::handlers::work_contract::{
            ClauseKind, ClauseSource, ContractClause, FalsificationMethod,
        };
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope:\n    preconditions:\n    - \"input normalized\"\n",
        );
        let yaml = tmp.path().join("contracts/k.yaml");
        let sha = sha256_hex(&std::fs::read(&yaml).unwrap());
        let mut c =
            crate::cli::handlers::work_contract::WorkContract::new("T-1".into(), "deadbeef".into());
        c.implements.push(ContractBinding {
            contract: "k".into(),
            equation: "rope".into(),
            file: PathBuf::from("contracts/k.yaml"),
            sha,
            bound_at: chrono::Utc::now(),
        });
        c.require.push(ContractClause {
            id: "require.normalized".into(),
            kind: ClauseKind::Require,
            description: "input normalized".into(),
            falsification_method: FalsificationMethod::ManifestIntegrity,
            threshold: None,
            blocking: false,
            source: ClauseSource::Manual,
        });
        c.save(tmp.path()).unwrap();
        let r = check_binding_inherited_clauses(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }
}
