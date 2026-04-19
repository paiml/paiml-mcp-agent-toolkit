#[cfg(test)]
mod tests_commit_enforcement_p2 {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_cb1341_no_specs() {
        let dir = tempdir().unwrap();
        let check = check_spec_number_accuracy(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1341_valid_specs() {
        let dir = tempdir().unwrap();
        let specs = dir.path().join("docs/specifications/components");
        fs::create_dir_all(&specs).unwrap();
        fs::write(specs.join("test.md"), "# Test\n\nShort spec.\n").unwrap();

        let check = check_spec_number_accuracy(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1341_oversized_spec() {
        let dir = tempdir().unwrap();
        let specs = dir.path().join("docs/specifications/components");
        fs::create_dir_all(&specs).unwrap();
        let mut long_content = String::from("# Title\n");
        long_content.push_str(&"Line\n".repeat(510));
        fs::write(specs.join("big.md"), &long_content).unwrap();

        let check = check_spec_number_accuracy(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("big.md"));
    }

    // --- Phase 4: Differential Obligation Verification ---

    #[test]
    fn test_cb1350_no_binding_index() {
        let dir = tempdir().unwrap();
        let check = check_differential_obligations(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("binding-index.json"));
    }

    #[test]
    fn test_cb1350_empty_binding_index() {
        let dir = tempdir().unwrap();
        let pmat = dir.path().join(".pmat");
        fs::create_dir(&pmat).unwrap();
        fs::write(pmat.join("binding-index.json"), "{}").unwrap();

        let check = check_differential_obligations(dir.path());
        // No staged files in a tempdir (not a git repo), so should pass
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1350_invalid_json() {
        let dir = tempdir().unwrap();
        let pmat = dir.path().join(".pmat");
        fs::create_dir(&pmat).unwrap();
        fs::write(pmat.join("binding-index.json"), "not json").unwrap();

        let check = check_differential_obligations(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("not valid JSON"));
    }

    #[test]
    fn test_cb1350_binding_index_with_entries() {
        let dir = tempdir().unwrap();
        let pmat = dir.path().join(".pmat");
        fs::create_dir(&pmat).unwrap();
        fs::write(
            pmat.join("binding-index.json"),
            r#"{"src/lib.rs": ["validate_input", "parse_config"]}"#,
        ).unwrap();

        // Not a git repo, so no staged files → pass
        let check = check_differential_obligations(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1351_no_binding_index() {
        let dir = tempdir().unwrap();
        let check = check_binding_index_freshness(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1351_fresh_binding_index() {
        let dir = tempdir().unwrap();
        let pmat = dir.path().join(".pmat");
        fs::create_dir(&pmat).unwrap();
        fs::write(pmat.join("binding-index.json"), "{}").unwrap();

        let check = check_binding_index_freshness(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("fresh"));
    }

    #[test]
    fn test_cb1351_contracts_alt_path() {
        let dir = tempdir().unwrap();
        let contracts = dir.path().join("contracts");
        fs::create_dir(&contracts).unwrap();
        fs::write(contracts.join("binding-index.json"), "{}").unwrap();

        let check = check_binding_index_freshness(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_get_staged_files_non_git() {
        let dir = tempdir().unwrap();
        let files = get_staged_files(dir.path());
        assert!(files.is_empty());
    }

    // --- Phase 5: Assume-Guarantee Chains ---

    #[test]
    fn test_cb1352_no_work_dir() {
        let dir = tempdir().unwrap();
        let check = check_assume_guarantee_chains(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1352_no_ag_contracts() {
        let dir = tempdir().unwrap();
        let work = dir.path().join(".pmat-work/PMAT-001");
        fs::create_dir_all(&work).unwrap();
        fs::write(
            work.join("contract.json"),
            r#"{"work_item_id":"PMAT-001","version":"5.0"}"#,
        ).unwrap();

        let check = check_assume_guarantee_chains(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("No work contracts with assume-guarantee"));
    }

    #[test]
    fn test_cb1352_ag_contracts_no_conflict() {
        let dir = tempdir().unwrap();
        let work1 = dir.path().join(".pmat-work/PMAT-A");
        let work2 = dir.path().join(".pmat-work/PMAT-B");
        fs::create_dir_all(&work1).unwrap();
        fs::create_dir_all(&work2).unwrap();
        fs::write(
            work1.join("contract.json"),
            r#"{"work_item_id":"PMAT-A","guarantees":["parser_correctness"],"assumes":[],"files":["src/parser.rs"]}"#,
        ).unwrap();
        fs::write(
            work2.join("contract.json"),
            r#"{"work_item_id":"PMAT-B","assumes":["parser_correctness"],"guarantees":[],"files":["src/formatter.rs"]}"#,
        ).unwrap();

        let check = check_assume_guarantee_chains(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1353_no_work_dir() {
        let dir = tempdir().unwrap();
        let check = check_ag_cycle_detection(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1353_no_ag_relationships() {
        let dir = tempdir().unwrap();
        let work = dir.path().join(".pmat-work/PMAT-001");
        fs::create_dir_all(&work).unwrap();
        fs::write(
            work.join("contract.json"),
            r#"{"work_item_id":"PMAT-001","version":"5.0"}"#,
        ).unwrap();

        let check = check_ag_cycle_detection(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("No assume-guarantee"));
    }

    #[test]
    fn test_cb1353_acyclic_dag() {
        let dir = tempdir().unwrap();
        let work_a = dir.path().join(".pmat-work/PMAT-A");
        let work_b = dir.path().join(".pmat-work/PMAT-B");
        fs::create_dir_all(&work_a).unwrap();
        fs::create_dir_all(&work_b).unwrap();
        fs::write(
            work_a.join("contract.json"),
            r#"{"work_item_id":"PMAT-A","guarantees":["invariant_x"],"assumes":[]}"#,
        ).unwrap();
        fs::write(
            work_b.join("contract.json"),
            r#"{"work_item_id":"PMAT-B","assumes":["invariant_x"],"guarantees":["invariant_y"]}"#,
        ).unwrap();

        let check = check_ag_cycle_detection(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("acyclic"));
    }

    #[test]
    fn test_cb1353_cyclic_dag() {
        let dir = tempdir().unwrap();
        let work_a = dir.path().join(".pmat-work/PMAT-A");
        let work_b = dir.path().join(".pmat-work/PMAT-B");
        fs::create_dir_all(&work_a).unwrap();
        fs::create_dir_all(&work_b).unwrap();
        // A guarantees X, assumes Y; B guarantees Y, assumes X → cycle
        fs::write(
            work_a.join("contract.json"),
            r#"{"work_item_id":"PMAT-A","guarantees":["invariant_x"],"assumes":["invariant_y"]}"#,
        ).unwrap();
        fs::write(
            work_b.join("contract.json"),
            r#"{"work_item_id":"PMAT-B","guarantees":["invariant_y"],"assumes":["invariant_x"]}"#,
        ).unwrap();

        let check = check_ag_cycle_detection(dir.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("Cycle"));
    }

    #[test]
    fn test_extract_string_array() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"assumes":["a","b"],"guarantees":[]}"#
        ).unwrap();
        assert_eq!(extract_string_array(&v, "assumes"), vec!["a", "b"]);
        assert!(extract_string_array(&v, "guarantees").is_empty());
        assert!(extract_string_array(&v, "missing").is_empty());
    }

    // --- Phase 6: Contract Query Readiness ---

    #[test]
    fn test_cb1354_no_infrastructure() {
        let dir = tempdir().unwrap();
        let check = check_contract_query_readiness(dir.path());
        // pv CLI may be available in dev env, so could be Skip (0/4) or Warn (1/4)
        assert!(
            check.status == CheckStatus::Skip || check.status == CheckStatus::Warn,
            "Expected Skip or Warn, got {:?}: {}", check.status, check.message
        );
    }

    #[test]
    fn test_cb1354_partial_contracts_dir_only() {
        let dir = tempdir().unwrap();
        let contracts = dir.path().join("contracts");
        fs::create_dir(&contracts).unwrap();
        fs::write(contracts.join("core.yaml"), "name: core\n").unwrap();

        let check = check_contract_query_readiness(dir.path());
        // 1-2/4 components → Warn (pv CLI may add +1)
        assert!(
            check.status == CheckStatus::Warn || check.status == CheckStatus::Pass,
            "Expected Warn or Pass, got {:?}: {}", check.status, check.message
        );
        assert!(check.message.contains("contracts/YAML"));
    }

    #[test]
    fn test_cb1354_binding_yaml_and_contracts() {
        let dir = tempdir().unwrap();
        let contracts = dir.path().join("contracts");
        fs::create_dir(&contracts).unwrap();
        fs::write(contracts.join("core.yaml"), "name: core\n").unwrap();
        fs::write(contracts.join("binding.yaml"), "bindings: []\n").unwrap();

        let check = check_contract_query_readiness(dir.path());
        // 2-3/4 components (contracts/YAML + binding.yaml, maybe pv) → Warn or Pass
        assert!(
            check.status == CheckStatus::Warn || check.status == CheckStatus::Pass,
            "Expected Warn or Pass, got {:?}: {}", check.status, check.message
        );
    }

    #[test]
    fn test_cb1354_full_readiness() {
        let dir = tempdir().unwrap();
        let pmat = dir.path().join(".pmat");
        let contracts = dir.path().join("contracts");
        fs::create_dir(&pmat).unwrap();
        fs::create_dir(&contracts).unwrap();
        fs::write(pmat.join("binding-index.json"), "{}").unwrap();
        fs::write(contracts.join("core.yaml"), "name: core\n").unwrap();
        fs::write(contracts.join("binding.yaml"), "bindings: []\n").unwrap();
        // pv CLI may or may not be available — 3/4 is still Pass
        let check = check_contract_query_readiness(dir.path());
        // At least 3/4 → Pass
        assert!(check.status == CheckStatus::Pass || check.status == CheckStatus::Warn);
    }

    // --- refresh-bindings ---

    #[test]
    fn test_refresh_bindings_empty_project() {
        let dir = tempdir().unwrap();
        let result = handle_refresh_bindings(dir.path());
        assert!(result.is_ok());
        let idx = dir.path().join(".pmat/binding-index.json");
        assert!(idx.exists());
        let content: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&idx).unwrap()).unwrap();
        assert!(content.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_refresh_bindings_with_binding_yaml() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("binding.yaml"),
            "- name: validate_input\n  source_file: src/lib.rs\n  status: implemented\n- name: parse_config\n  source_file: src/config.rs\n  status: implemented\n",
        ).unwrap();

        let result = handle_refresh_bindings(dir.path());
        assert!(result.is_ok());
        let idx = dir.path().join(".pmat/binding-index.json");
        let content: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&idx).unwrap()).unwrap();
        let obj = content.as_object().unwrap();
        assert!(obj.contains_key("src/lib.rs"));
        assert!(obj.contains_key("src/config.rs"));
    }

    #[test]
    fn test_refresh_bindings_with_contracts_yaml() {
        let dir = tempdir().unwrap();
        let contracts = dir.path().join("contracts");
        fs::create_dir(&contracts).unwrap();
        fs::write(
            contracts.join("core.yaml"),
            "name: core\nfunctions:\n  - src/core.rs\n  - src/util.rs\n",
        ).unwrap();

        let result = handle_refresh_bindings(dir.path());
        assert!(result.is_ok());
        let idx = dir.path().join(".pmat/binding-index.json");
        let content: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&idx).unwrap()).unwrap();
        let obj = content.as_object().unwrap();
        assert!(obj.contains_key("src/core.rs"));
    }

    // --- CB-1342: Codegen Compiles ---

    #[test]
    fn test_cb1342_no_generated_code_no_pv() {
        let dir = tempdir().unwrap();
        let check = check_codegen_compiles(dir.path());
        // Skip or Pass depending on pv availability
        assert!(
            check.status == CheckStatus::Skip || check.status == CheckStatus::Pass,
            "Expected Skip or Pass, got {:?}: {}", check.status, check.message
        );
    }

    #[test]
    fn test_cb1342_clean_generated_code() {
        let dir = tempdir().unwrap();
        let contracts = dir.path().join("src/contracts");
        fs::create_dir_all(&contracts).unwrap();
        fs::write(
            contracts.join("generated.rs"),
            "fn validate() { debug_assert!(x > 0); }\n",
        ).unwrap();

        let check = check_codegen_compiles(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1342_unbalanced_braces() {
        let dir = tempdir().unwrap();
        let contracts = dir.path().join("src/contracts");
        fs::create_dir_all(&contracts).unwrap();
        fs::write(
            contracts.join("bad.rs"),
            "fn broken() { if true { } \n",
        ).unwrap();

        let check = check_codegen_compiles(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("unbalanced"));
    }

    #[test]
    fn test_cb1342_placeholder_in_codegen() {
        let dir = tempdir().unwrap();
        let contracts = dir.path().join("src/contracts");
        fs::create_dir_all(&contracts).unwrap();
        fs::write(
            contracts.join("stub.rs"),
            "fn check() { debug_assert!(TODO_PARAM > 0); }\n",
        ).unwrap();

        let check = check_codegen_compiles(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("placeholder"));
    }

    // KAIZEN-0175: work-contract YAML generator must emit a `metadata:` block
    // with version/description/references so `pv lint` accepts them.
    #[test]
    fn test_kaizen0175_generator_emits_metadata_block() {
        let dir = tempdir().unwrap();
        let work = dir.path().join(".pmat-work/TEST-123");
        fs::create_dir_all(&work).unwrap();
        fs::write(
            work.join("contract.json"),
            r#"{
                "work_item_id": "TEST-123",
                "verification_level": "L2",
                "falsifiable_claims": [{"claim": "foo > 0"}],
                "ensure": ["bar is valid"],
                "require": ["baz exists"]
            }"#,
        )
        .unwrap();

        let count = generate_work_contract_yamls(dir.path()).unwrap();
        assert_eq!(count, 1);

        let yaml = fs::read_to_string(dir.path().join("contracts/work/TEST-123.yaml")).unwrap();

        // Metadata block must be present with the three fields pv 0.31 requires.
        assert!(
            yaml.contains("metadata:\n"),
            "missing `metadata:` block: {yaml}"
        );
        assert!(
            yaml.contains("  version: \"1.0.0\"\n"),
            "missing metadata.version: {yaml}"
        );
        assert!(
            yaml.contains("  description: \"Auto-generated work-contract for TEST-123\"\n"),
            "missing metadata.description: {yaml}"
        );
        assert!(
            yaml.contains("  references:\n"),
            "missing metadata.references: {yaml}"
        );
        assert!(
            yaml.contains("    - \".pmat-work/TEST-123/contract.json\"\n"),
            "missing references entry: {yaml}"
        );

        // Metadata must precede surface/verification_summary so it is the first
        // map-entry (line 2 per pv's "missing field metadata at line 2 column 1").
        let meta_pos = yaml.find("metadata:").expect("metadata: present");
        let surface_pos = yaml.find("surface:").expect("surface: present");
        let vs_pos = yaml
            .find("verification_summary:")
            .expect("verification_summary: present");
        assert!(
            meta_pos < surface_pos && meta_pos < vs_pos,
            "metadata must come before surface and verification_summary: {yaml}"
        );
    }

    // KAIZEN-0190 (SCHEMA-003): work-contract YAMLs must declare
    // `metadata.kind: schema` so pv treats them as reference documents, not
    // mathematical kernel contracts. Without this, pv fires SCHEMA-003
    // ("equations must contain at least one equation") + PROVABILITY-001
    // (missing proof_obligations/falsification_tests/kani_harnesses) on every
    // work-contract YAML — 4 errors × 108 files = 432 errors.
    #[test]
    fn test_kaizen0190_generator_declares_schema_kind() {
        let dir = tempdir().unwrap();
        let work = dir.path().join(".pmat-work/TEST-190");
        fs::create_dir_all(&work).unwrap();
        fs::write(
            work.join("contract.json"),
            r#"{
                "work_item_id": "TEST-190",
                "verification_level": "L1"
            }"#,
        )
        .unwrap();

        let count = generate_work_contract_yamls(dir.path()).unwrap();
        assert_eq!(count, 1);

        let yaml = fs::read_to_string(dir.path().join("contracts/work/TEST-190.yaml")).unwrap();

        // The `kind: schema` line must live inside the metadata block, after
        // the `metadata:` header and before any subsequent top-level key.
        let meta_idx = yaml.find("metadata:").expect("metadata: present");
        let kind_idx = yaml
            .find("  kind: schema\n")
            .expect("missing `  kind: schema` inside metadata block");
        assert!(
            kind_idx > meta_idx,
            "kind: schema must appear inside metadata block: {yaml}"
        );

        // Sanity: the next top-level field (surface:) must come after kind:
        let surface_idx = yaml.find("surface:").expect("surface: present");
        assert!(
            kind_idx < surface_idx,
            "kind: schema must appear before surface: {yaml}"
        );
    }
}
