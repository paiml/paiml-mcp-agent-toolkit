// Tests for CB-1601 SHA drift + CB-1607 equation exists +
// CB-1609 file tracked skip — included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_sha_equation {
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

    #[test]
    fn sha_drift_passes_when_aligned() {
        let tmp = tempdir().unwrap();
        let yaml = write_yaml(tmp.path(), "k", "equations:\n  rope: {}\n");
        let bytes = std::fs::read(&yaml).unwrap();
        let sha = sha256_hex(&bytes);
        let rel = PathBuf::from("contracts/k.yaml");
        write_contract(tmp.path(), "T-1", &rel, "rope", &sha);
        let r = check_binding_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn sha_drift_fails_on_edit() {
        let tmp = tempdir().unwrap();
        let yaml = write_yaml(tmp.path(), "k", "equations:\n  rope: {}\n");
        let bytes = std::fs::read(&yaml).unwrap();
        let sha = sha256_hex(&bytes);
        let rel = PathBuf::from("contracts/k.yaml");
        write_contract(tmp.path(), "T-1", &rel, "rope", &sha);
        // Mutate the YAML
        std::fs::write(&yaml, "equations:\n  rope: {a: 1}\n").unwrap();
        let r = check_binding_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("SHA drift"));
    }

    #[test]
    fn sha_drift_skips_without_bindings() {
        let tmp = tempdir().unwrap();
        let r = check_binding_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn equation_exists_passes_on_known_name() {
        let tmp = tempdir().unwrap();
        write_yaml(tmp.path(), "k", "equations:\n  rope: {}\n  softmax: {}\n");
        let rel = PathBuf::from("contracts/k.yaml");
        write_contract(tmp.path(), "T-1", &rel, "rope", "deadbeef");
        let r = check_binding_equation_exists(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn equation_exists_fails_on_typo() {
        let tmp = tempdir().unwrap();
        write_yaml(tmp.path(), "k", "equations:\n  rope: {}\n");
        let rel = PathBuf::from("contracts/k.yaml");
        write_contract(tmp.path(), "T-1", &rel, "ropee", "deadbeef");
        let r = check_binding_equation_exists(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("ropee"));
    }

    #[test]
    fn yaml_equation_names_parses_minimal() {
        let s = "version: 1\nequations:\n  rope:\n    preconditions: []\n  softmax: {}\n";
        let names = yaml_equation_names(s).unwrap();
        assert_eq!(names, vec!["rope".to_string(), "softmax".to_string()]);
    }

    #[test]
    fn yaml_equation_names_ignores_comments() {
        let s = "equations:\n  # top-level comment line\n  rope: {}\n";
        let names = yaml_equation_names(s).unwrap();
        assert_eq!(names, vec!["rope".to_string()]);
    }

    #[test]
    fn file_tracked_skips_without_bindings() {
        let tmp = tempdir().unwrap();
        let r = check_binding_file_tracked(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }
}
