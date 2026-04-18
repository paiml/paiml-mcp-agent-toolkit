// Tests for CB-1606 Lean theorem linkage —
// included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_lean_theorem {
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

    // ── CB-1606 Lean theorem linkage tests ───────────────────────────────

    /// Overwrite a previously-written contract with an additional note field so
    /// we can simulate BLOCK-ON-PROOF references without building a fresh
    /// helper. We serialize directly to JSON.
    fn append_contract_note(project: &Path, ticket: &str, note: &str) {
        let path = project
            .join(".pmat-work")
            .join(ticket)
            .join("contract.json");
        let bytes = std::fs::read(&path).unwrap();
        let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        if let Some(obj) = v.as_object_mut() {
            obj.insert("_note".to_string(), serde_json::json!(note));
        }
        std::fs::write(&path, serde_json::to_vec_pretty(&v).unwrap()).unwrap();
    }

    #[test]
    fn yaml_lean_theorem_status_parses_proved() {
        let s = "lean_theorem:\n  status: proved\n  name: foo\n";
        assert_eq!(yaml_lean_theorem_status(s).as_deref(), Some("proved"));
    }

    #[test]
    fn yaml_lean_theorem_status_parses_quoted() {
        let s = "lean_theorem:\n  status: \"pending\"\n";
        assert_eq!(yaml_lean_theorem_status(s).as_deref(), Some("pending"));
    }

    #[test]
    fn yaml_lean_theorem_status_missing_returns_none() {
        let s = "equations:\n  rope: {}\n";
        assert!(yaml_lean_theorem_status(s).is_none());
    }

    #[test]
    fn yaml_lean_theorem_status_block_without_status() {
        // Block exists but has no status key — None (nothing to evaluate)
        let s = "lean_theorem:\n  name: foo\n";
        assert!(yaml_lean_theorem_status(s).is_none());
    }

    #[test]
    fn lean_theorem_skip_without_bindings() {
        let tmp = tempdir().unwrap();
        let r = check_binding_lean_theorem(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("implements:"));
    }

    #[test]
    fn lean_theorem_skip_when_no_yaml_has_lean_block() {
        let tmp = tempdir().unwrap();
        write_yaml(tmp.path(), "k", "equations:\n  rope: {}\n");
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        let r = check_binding_lean_theorem(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("lean_theorem"));
    }

    #[test]
    fn lean_theorem_pass_when_all_proved() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nlean_theorem:\n  status: proved\n  name: rope_proof\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        let r = check_binding_lean_theorem(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("status: proved"));
    }

    #[test]
    fn lean_theorem_fail_when_unproved_without_link() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nlean_theorem:\n  status: pending\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        let r = check_binding_lean_theorem(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("pending"));
        assert!(r.message.contains("BLOCK-ON-PROOF"));
    }

    #[test]
    fn lean_theorem_pass_when_unproved_with_link() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nlean_theorem:\n  status: pending\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        append_contract_note(tmp.path(), "T-1", "Follow-up: BLOCK-ON-PROOF-42");
        let r = check_binding_lean_theorem(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("BLOCK-ON-PROOF"));
    }

    #[test]
    fn lean_theorem_case_insensitive_proved_match() {
        let tmp = tempdir().unwrap();
        // Status "PROVED" (uppercase) still counts as proved
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nlean_theorem:\n  status: PROVED\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        let r = check_binding_lean_theorem(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn lean_theorem_case_insensitive_link_match() {
        // `block-on-proof` (lowercase) in contract.json still counts.
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nlean_theorem:\n  status: failing\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        append_contract_note(tmp.path(), "T-1", "see block-on-proof task");
        let r = check_binding_lean_theorem(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }
}
