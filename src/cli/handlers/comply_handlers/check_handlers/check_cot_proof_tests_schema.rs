// Tests for CB-1640 (refs), CB-1641 (evidence), CB-1642 (existing test),
// CB-1643 (L3 expr), CB-1645 (derived YAML), CB-1647 (orphans), and helper
// sanity tests (parse_level, step_id, load_contract_values).
// Included from `check_cot_proof.rs` — do NOT add `use` imports or `#!` attributes here.

#[cfg(all(test, not(coverage_nightly)))]
mod tests_cot_schema {
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

    // ── CB-1645 derived YAML obligations tests ───────────────────────────

    fn write_derived_yaml(project: &Path, safe_id: &str, body: &str) {
        let dir = project.join("contracts/work");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{}.yaml", safe_id)), body).unwrap();
    }

    #[test]
    fn derived_yaml_skips_when_no_tickets() {
        let project = tempdir().unwrap();
        let check = check_derived_yaml_obligations_present(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("No `.pmat-work/` tickets"));
    }

    #[test]
    fn derived_yaml_skips_when_no_derivable_clauses() {
        let project = tempdir().unwrap();
        write_contract(project.path(), "T1", json!({ "work_item_id": "T1" }));
        let check = check_derived_yaml_obligations_present(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("derivable"));
    }

    #[test]
    fn derived_yaml_fails_when_file_missing() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "work_item_id": "T1",
                "ensure": ["returns sorted array"]
            }),
        );
        let check = check_derived_yaml_obligations_present(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("missing derived"));
        assert!(check.message.contains("T1"));
    }

    #[test]
    fn derived_yaml_fails_when_stale() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "work_item_id": "T1",
                "ensure": ["returns sorted array"]
            }),
        );
        // Derived YAML doesn't mention the postcondition → stale
        write_derived_yaml(
            project.path(),
            "T1",
            "name: \"T1\"\npostconditions:\n  - \"some other thing\"\n",
        );
        let check = check_derived_yaml_obligations_present(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("stale"));
        assert!(check.message.contains("returns sorted array"));
    }

    #[test]
    fn derived_yaml_passes_when_up_to_date() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "work_item_id": "T1",
                "ensure": ["returns sorted array"],
                "require": ["input is a slice of integers"]
            }),
        );
        write_derived_yaml(
            project.path(),
            "T1",
            "name: \"T1\"\n\
             preconditions:\n  - \"input is a slice of integers\"\n\
             postconditions:\n  - \"returns sorted array\"\n",
        );
        let check = check_derived_yaml_obligations_present(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn derived_yaml_sanitizes_special_chars_in_id() {
        let project = tempdir().unwrap();
        // A ticket id with a colon — generator replaces ':' with '_'
        write_contract(
            project.path(),
            "GH-168: fix leak",
            json!({
                "work_item_id": "GH-168: fix leak",
                "ensure": ["no memory leak"]
            }),
        );
        write_derived_yaml(
            project.path(),
            "GH-168__fix_leak",
            "postconditions:\n  - \"no memory leak\"\n",
        );
        let check = check_derived_yaml_obligations_present(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    // ── CB-1640 assumption reference resolution tests ────────────────────

    #[test]
    fn assumption_refs_skip_when_no_references_exist() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" }
                    }
                ]
            }),
        );
        let check = check_assumption_references_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn assumption_refs_pass_matching_prior_step_id() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "a" },
                        "implication": { "predicate": "b" }
                    },
                    {
                        "id": "CoT-2",
                        "assumption": { "predicate": "c", "references": ["CoT-1"] },
                        "implication": { "predicate": "d" }
                    }
                ]
            }),
        );
        let check = check_assumption_references_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn assumption_refs_pass_matching_prior_implication_predicate() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "implication": { "predicate": "array is sorted" }
                    },
                    {
                        "id": "CoT-2",
                        "assumption": {
                            "predicate": "search terminates",
                            "references": ["array is sorted"]
                        }
                    }
                ]
            }),
        );
        let check = check_assumption_references_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn assumption_refs_pass_matching_bound_equation() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "implements": [
                    { "equation": "rope_periodicity", "yaml_path": "rope.yaml" }
                ],
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": {
                            "predicate": "rope norm preserved",
                            "references": ["rope_periodicity"]
                        }
                    }
                ]
            }),
        );
        let check = check_assumption_references_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn assumption_refs_fail_on_unmatched_reference() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": {
                            "predicate": "p",
                            "references": ["nonexistent"]
                        }
                    }
                ]
            }),
        );
        let check = check_assumption_references_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("T1:CoT-1"));
        assert!(check.message.contains("nonexistent"));
    }

    #[test]
    fn assumption_refs_axiomatic_skipped_even_with_refs() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": {
                            "predicate": "p",
                            "references": ["nonexistent"]
                        },
                        "discharged_by": { "Axiomatic": { "reason": "base case" } }
                    }
                ]
            }),
        );
        let check = check_assumption_references_resolve(project.path());
        // Axiomatic discharge bypasses reference resolution (spec §Chain
        // Integrity Rule), so this should Skip not Fail.
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn assumption_refs_forward_reference_fails() {
        // CoT-1 references CoT-2 (not yet defined) → violation (only prior
        // steps count as resolvable).
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": {
                            "predicate": "p",
                            "references": ["CoT-2"]
                        }
                    },
                    {
                        "id": "CoT-2",
                        "implication": { "predicate": "q" }
                    }
                ]
            }),
        );
        let check = check_assumption_references_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
    }

    // ── CB-1641 / CB-1642 / CB-1643 evidence / existing-test / L3 expr ──

    #[test]
    fn evidence_method_skip_without_structured_steps() {
        let project = tempdir().unwrap();
        // Legacy schema — no structured fields
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    { "step": 1, "question": "q", "answer": "a" },
                ]
            }),
        );
        let check = check_step_has_evidence_method(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn evidence_method_passes_when_all_present() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" },
                        "implication": { "predicate": "q" },
                        "evidence_method": { "ReviewOnly": { "reviewer_sha": "abc" } }
                    }
                ]
            }),
        );
        let check = check_step_has_evidence_method(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn evidence_method_fails_when_missing() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" },
                        "implication": { "predicate": "q" }
                    }
                ]
            }),
        );
        let check = check_step_has_evidence_method(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("T1:CoT-1"));
    }

    #[test]
    fn existing_test_fails_on_missing_path() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "evidence_method": {
                            "ExistingTest": {
                                "path": "nonexistent/path.rs",
                                "name": "test_foo"
                            }
                        }
                    }
                ]
            }),
        );
        let check = check_existing_test_paths_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
    }

    #[test]
    fn existing_test_passes_when_path_exists() {
        let project = tempdir().unwrap();
        let file = project.path().join("tests/real.rs");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "").unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "evidence_method": {
                            "ExistingTest": {
                                "path": "tests/real.rs",
                                "name": "test_foo"
                            }
                        }
                    }
                ]
            }),
        );
        let check = check_existing_test_paths_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn existing_test_skips_when_no_entries() {
        let project = tempdir().unwrap();
        write_contract(project.path(), "T1", json!({ "chain_of_thought": [] }));
        let check = check_existing_test_paths_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn l3_expr_passes_when_assumption_expr_present() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L3",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p", "expr": "x.is_finite()" },
                        "implication": { "predicate": "q" }
                    }
                ]
            }),
        );
        let check = check_l3_structured_expr_present(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn l3_expr_fails_when_neither_expr_present() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L3",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" },
                        "implication": { "predicate": "q" }
                    }
                ]
            }),
        );
        let check = check_l3_structured_expr_present(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
    }

    #[test]
    fn l3_expr_skips_for_sub_l3_tickets() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L1",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" }
                    }
                ]
            }),
        );
        let check = check_l3_structured_expr_present(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    // ── CB-1647 orphan steps tests ───────────────────────────────────────

    #[test]
    fn orphan_steps_detected() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" }
                        // No discharged_by
                    }
                ]
            }),
        );
        let check = check_no_orphan_steps(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("T1:CoT-1"));
    }

    #[test]
    fn discharged_steps_pass() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" },
                        "discharged_by": { "Axiomatic": { "reason": "base case" } }
                    }
                ]
            }),
        );
        let check = check_no_orphan_steps(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    // ── Helper sanity tests ──────────────────────────────────────────────

    #[test]
    fn parse_level_handles_l_prefix() {
        assert_eq!(parse_level(&json!({ "verification_level": "L3" })), 3);
        assert_eq!(
            parse_level(&json!({ "verification_level": "L3 (kani_proof)" })),
            3
        );
        assert_eq!(parse_level(&json!({ "verification_level": "L5" })), 5);
        assert_eq!(parse_level(&json!({})), 0);
    }

    #[test]
    fn step_id_falls_back_to_numeric_step() {
        assert_eq!(step_id(&json!({ "id": "CoT-5" })), "CoT-5");
        assert_eq!(step_id(&json!({ "step": 2 })), "CoT-2");
        assert_eq!(step_id(&json!({})), "CoT-?");
    }

    #[test]
    fn load_contracts_skips_ledger_and_hidden() {
        let project = tempdir().unwrap();
        std::fs::create_dir_all(project.path().join(".pmat-work/ledger")).unwrap();
        std::fs::create_dir_all(project.path().join(".pmat-work/.hidden")).unwrap();
        write_contract(project.path(), "T1", json!({ "chain_of_thought": [] }));
        let loaded = load_contract_values(project.path());
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].0, "T1");
    }
}
