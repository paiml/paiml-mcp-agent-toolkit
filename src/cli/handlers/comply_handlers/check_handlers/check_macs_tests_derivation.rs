// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

#[cfg(all(test, not(coverage_nightly)))]
mod tests_macs_derivation {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    fn seed_ticket(project: &Path, ticket: &str) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("contract.json"),
            serde_json::to_string_pretty(&json!({
                "work_item_id": ticket,
                "chain_of_thought": [
                    {"id": "CoT-1", "assumption": "root (E1)",
                     "implication": "alpha holds",
                     "evidence_method": "cargo test alpha"},
                    {"id": "CoT-2", "assumption": "depends on (CoT-1)",
                     "implication": "beta holds",
                     "evidence_method": "cargo test beta"}
                ]
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(dir.join("cot-digest.json"), r#"{"sha": "irrelevant-here"}"#).unwrap();
    }

    fn write_artifact(project: &Path, ticket: &str, body: &str) {
        let dir = project.join("contracts").join("work");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{ticket}.cot.yaml")), body).unwrap();
    }

    #[test]
    fn cb1658_skips_without_derivations() {
        let project = tempdir().unwrap();
        std::fs::create_dir_all(project.path().join(".pmat-work")).unwrap();
        let check = check_derivation_completeness(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn cb1658_red_when_counts_mismatch() {
        let project = tempdir().unwrap();
        seed_ticket(project.path(), "T-D1");
        write_artifact(
            project.path(),
            "T-D1",
            "proof_obligations:\n- id: \"PO-CoT-1\"\n  statement: \"alpha holds\"\n  evidence_method: \"cargo test alpha\"\nfalsifiable_claims:\n- hypothesis: \"alpha holds\"\n  method: \"cargo test alpha\"\n  from_step: \"CoT-1\"\n",
        );
        let check = check_derivation_completeness(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("2 step(s)"), "{}", check.message);
    }

    #[test]
    fn cb1658_red_on_paraphrase_drift() {
        let project = tempdir().unwrap();
        seed_ticket(project.path(), "T-D2");
        write_artifact(
            project.path(),
            "T-D2",
            "proof_obligations:\n- id: \"PO-CoT-1\"\n  statement: \"alpha holds\"\n  evidence_method: \"cargo test alpha\"\n- id: \"PO-CoT-2\"\n  statement: \"beta holds\"\n  evidence_method: \"cargo test beta\"\nfalsifiable_claims:\n- hypothesis: \"alpha mostly holds\"\n  method: \"cargo test alpha\"\n  from_step: \"CoT-1\"\n- hypothesis: \"beta holds\"\n  method: \"cargo test beta\"\n  from_step: \"CoT-2\"\n",
        );
        let check = check_derivation_completeness(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("paraphrase drift"), "{}", check.message);
    }

    #[test]
    fn cb1658_green_on_faithful_derivation() {
        let project = tempdir().unwrap();
        seed_ticket(project.path(), "T-D3");
        // Render through the real deriver — the artifact matches by construction.
        let contract: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                project.path().join(".pmat-work/T-D3/contract.json"),
            )
            .unwrap(),
        )
        .unwrap();
        let steps = crate::models::work_cot::parse_steps(&contract);
        let yaml = crate::models::work_cot::render_derivation("T-D3", &steps, false);
        write_artifact(project.path(), "T-D3", &yaml);
        let check = check_derivation_completeness(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }
}
