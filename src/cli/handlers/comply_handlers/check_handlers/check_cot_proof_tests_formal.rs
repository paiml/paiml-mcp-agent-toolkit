// Tests for CB-1648 (L4 Axiomatic discharge bounded) and CB-1649 (L5 Lean
// theorem mapping).
// Included from `check_cot_proof.rs` — do NOT add `use` imports or `#!` attributes here.

#[cfg(all(test, not(coverage_nightly)))]
mod tests_cot_formal {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    fn write_contract(project: &Path, ticket: &str, contract: Value) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("contract.json"),
            serde_json::to_string_pretty(&contract).unwrap(),
        )
        .unwrap();
    }

    // ── CB-1648 L4 Axiomatic discharge bounded tests ─────────────────────

    #[test]
    fn cb1648_skips_when_no_tickets() {
        let project = tempdir().unwrap();
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/<ID>/contract.json`"));
    }

    #[test]
    fn cb1648_skips_when_no_l4_ticket() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({ "verification_level": "L3", "chain_of_thought": [] }),
        );
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L4+ ticket"));
    }

    #[test]
    fn cb1648_skips_when_no_axiomatic_discharge() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L4",
                "chain_of_thought": [
                    { "id": "CoT-1", "assumption": { "predicate": "p" } }
                ]
            }),
        );
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("Axiomatic"));
    }

    #[test]
    fn cb1648_passes_when_axiomatic_has_reason() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L4",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "discharged_by": {
                            "Axiomatic": { "reason": "IEEE-754 finite arithmetic" }
                        }
                    }
                ]
            }),
        );
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("1 Axiomatic"));
    }

    #[test]
    fn cb1648_passes_when_axiomatic_matches_bound_equation() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L4",
                "implements": [
                    {
                        "contract": "pmat-core",
                        "equation": "rope",
                        "file": "contracts/pmat-core.yaml",
                        "sha": "abc",
                        "bound_at": "2026-04-18T00:00:00Z"
                    }
                ],
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "discharged_by": {
                            "Axiomatic": { "lemma": "rope invariant" }
                        }
                    }
                ]
            }),
        );
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1648_fails_when_axiomatic_has_no_reason_or_equation_match() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L4",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "discharged_by": { "Axiomatic": {} }
                    }
                ]
            }),
        );
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("CoT-1"));
        assert!(r.message.contains("T1"));
    }

    #[test]
    fn cb1648_fails_when_reason_empty_and_no_equation() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L5",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "discharged_by": {
                            "Axiomatic": { "reason": "   " }
                        }
                    }
                ]
            }),
        );
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
    }

    #[test]
    fn cb1648_aggregates_multiple_violations() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L4",
                "chain_of_thought": [
                    { "id": "CoT-1", "discharged_by": { "Axiomatic": {} } },
                    { "id": "CoT-2", "discharged_by": { "Axiomatic": {} } },
                ]
            }),
        );
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("2 unchecked"));
    }

    #[test]
    fn cb1648_ignores_non_l4_axiomatic_violations() {
        let project = tempdir().unwrap();
        // L3 ticket with unchecked axiom — not our concern at L4 gate.
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L3",
                "chain_of_thought": [
                    { "id": "CoT-1", "discharged_by": { "Axiomatic": {} } }
                ]
            }),
        );
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("L4+"));
    }

    // ── CB-1649 L5 Lean theorem mapping tests ────────────────────────────

    #[test]
    fn cb1649_skips_when_no_tickets() {
        let project = tempdir().unwrap();
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn cb1649_skips_when_no_l5_ticket() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({ "verification_level": "L4", "chain_of_thought": [] }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L5 ticket"));
    }

    #[test]
    fn cb1649_skips_when_no_structured_steps_in_l5() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L5",
                "chain_of_thought": [
                    { "step": 1, "question": "q", "answer": "a" }
                ]
            }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("migration pending"));
    }

    #[test]
    fn cb1649_passes_when_step_has_lean_theorem_key() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L5",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" },
                        "lean_theorem": "Rope.preserves_norm"
                    }
                ]
            }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1649_passes_when_step_has_lean_lemma_key() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L5",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "implication": { "predicate": "q" },
                        "lean_lemma": "Arith.add_comm"
                    }
                ]
            }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1649_passes_when_evidence_method_is_lean_theorem() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L5",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" },
                        "evidence_method": { "LeanTheorem": { "name": "Rope.norm" } }
                    }
                ]
            }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1649_passes_when_discharged_by_lean() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L5",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" },
                        "discharged_by": { "Lean": { "lemma": "FinSet.nonempty" } }
                    }
                ]
            }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1649_fails_when_structured_step_has_no_lean_mapping() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L5",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" }
                    }
                ]
            }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("CoT-1"));
        assert!(r.message.contains("T1"));
    }

    #[test]
    fn cb1649_fails_when_lean_theorem_is_empty_string() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L5",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" },
                        "lean_theorem": "   "
                    }
                ]
            }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
    }

    #[test]
    fn cb1649_step_has_lean_mapping_recognises_all_forms() {
        assert!(step_has_lean_mapping(&json!({ "lean_theorem": "Foo.bar" })));
        assert!(step_has_lean_mapping(&json!({ "lean_lemma": "Foo.baz" })));
        assert!(step_has_lean_mapping(&json!({
            "evidence_method": { "LeanTheorem": { "name": "Foo" } }
        })));
        assert!(step_has_lean_mapping(&json!({
            "evidence_method": { "LeanLemma": { "name": "Foo" } }
        })));
        assert!(step_has_lean_mapping(&json!({
            "discharged_by": { "Lean": { "lemma": "Foo" } }
        })));
        assert!(!step_has_lean_mapping(&json!({})));
        assert!(!step_has_lean_mapping(&json!({ "lean_theorem": "" })));
        assert!(!step_has_lean_mapping(&json!({
            "evidence_method": { "ExistingTest": { "path": "t.rs" } }
        })));
    }

    #[test]
    fn cb1649_ignores_non_l5_tickets() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L4",
                "chain_of_thought": [
                    { "id": "CoT-1", "assumption": { "predicate": "p" } }
                ]
            }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
    }
}
