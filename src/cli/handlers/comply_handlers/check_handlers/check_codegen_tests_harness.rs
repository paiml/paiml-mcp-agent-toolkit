// Tests for CB-1637 L2+ public function coverage and CB-1639 Kani harness
// macro reference checks.
// Included from check_codegen.rs — do NOT add `use` imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_harness {
    use super::*;
    use tempfile::tempdir;

    // ── CB-1637 L2+ public function coverage tests ──────────────────────

    /// Materialise a ticket at `.pmat-work/<id>/contract.json` with the
    /// given `verification_level`.
    fn cb1637_write_ticket(root: &Path, id: &str, level: &str) {
        let dir = root.join(".pmat-work").join(id);
        std::fs::create_dir_all(&dir).unwrap();
        let body = format!(r#"{{"verification_level":"{}"}}"#, level);
        std::fs::write(dir.join("contract.json"), body).unwrap();
    }

    /// Materialise `.pmat-work/<id>/modified-files.json` (top-level array shape).
    fn cb1637_write_modified(root: &Path, id: &str, files: &[&str]) {
        let dir = root.join(".pmat-work").join(id);
        std::fs::create_dir_all(&dir).unwrap();
        let body = serde_json::to_string(files).unwrap();
        std::fs::write(dir.join("modified-files.json"), body).unwrap();
    }

    /// Write `src/<rel>` with `contents`, creating parent dirs.
    fn cb1637_write_src(root: &Path, rel: &str, contents: &str) {
        let p = root.join("src").join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(p, contents).unwrap();
    }

    #[test]
    fn cb1637_skips_when_no_pmat_work_dir() {
        let tmp = tempdir().unwrap();
        let r = check_l2_public_fn_coverage(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/` directory"));
    }

    #[test]
    fn cb1637_skips_when_no_tickets() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-work")).unwrap();
        let r = check_l2_public_fn_coverage(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/<ID>/contract.json`"));
    }

    #[test]
    fn cb1637_skips_when_only_l1_tickets() {
        let tmp = tempdir().unwrap();
        cb1637_write_ticket(tmp.path(), "PMAT-100", "L1");
        cb1637_write_ticket(tmp.path(), "PMAT-101", "L0");
        let r = check_l2_public_fn_coverage(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No ticket targets L2 or higher"));
    }

    #[test]
    fn cb1637_skips_when_l2_ticket_has_no_modified_files() {
        let tmp = tempdir().unwrap();
        cb1637_write_ticket(tmp.path(), "PMAT-200", "L2");
        let r = check_l2_public_fn_coverage(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("modified-files.json"));
    }

    #[test]
    fn cb1637_skips_when_modified_files_are_out_of_scope() {
        let tmp = tempdir().unwrap();
        cb1637_write_ticket(tmp.path(), "PMAT-200", "L2");
        cb1637_write_modified(
            tmp.path(),
            "PMAT-200",
            &["docs/x.md", "tests/integration.rs", "src/missing.rs"],
        );
        let r = check_l2_public_fn_coverage(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("declaring `pub fn`"));
    }

    #[test]
    fn cb1637_skips_when_modified_file_has_no_pub_fn() {
        let tmp = tempdir().unwrap();
        cb1637_write_ticket(tmp.path(), "PMAT-200", "L2");
        cb1637_write_modified(tmp.path(), "PMAT-200", &["src/a.rs"]);
        cb1637_write_src(tmp.path(), "a.rs", "fn private_helper() {}\n");
        let r = check_l2_public_fn_coverage(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn cb1637_passes_when_pub_fn_and_matching_attribute() {
        let tmp = tempdir().unwrap();
        cb1637_write_ticket(tmp.path(), "PMAT-200", "L2");
        cb1637_write_modified(tmp.path(), "PMAT-200", &["src/a.rs"]);
        cb1637_write_src(
            tmp.path(),
            "a.rs",
            "#[pmat_work_contract(id = \"PMAT-200\", ensure = \"E1\")]\npub fn f() {}\n",
        );
        let r = check_l2_public_fn_coverage(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("1 file(s)"));
    }

    #[test]
    fn cb1637_fails_when_pub_fn_without_matching_attribute() {
        let tmp = tempdir().unwrap();
        cb1637_write_ticket(tmp.path(), "PMAT-200", "L2");
        cb1637_write_modified(tmp.path(), "PMAT-200", &["src/a.rs"]);
        cb1637_write_src(tmp.path(), "a.rs", "pub fn f() {}\n");
        let r = check_l2_public_fn_coverage(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("PMAT-200"));
        assert!(r.message.contains("src/a.rs"));
    }

    #[test]
    fn cb1637_fails_when_attribute_is_for_wrong_ticket() {
        let tmp = tempdir().unwrap();
        cb1637_write_ticket(tmp.path(), "PMAT-200", "L2");
        cb1637_write_modified(tmp.path(), "PMAT-200", &["src/a.rs"]);
        cb1637_write_src(
            tmp.path(),
            "a.rs",
            "#[pmat_work_contract(id = \"PMAT-999\")]\npub fn f() {}\n",
        );
        let r = check_l2_public_fn_coverage(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("PMAT-200"));
    }

    #[test]
    fn cb1637_l1_ticket_does_not_trigger_failure() {
        // L1 ticket with pub fn but no attribute should not cause a fail,
        // because this check only gates L2+.
        let tmp = tempdir().unwrap();
        cb1637_write_ticket(tmp.path(), "PMAT-100", "L1");
        cb1637_write_modified(tmp.path(), "PMAT-100", &["src/a.rs"]);
        cb1637_write_src(tmp.path(), "a.rs", "pub fn f() {}\n");
        let r = check_l2_public_fn_coverage(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("L2 or higher"));
    }

    #[test]
    fn cb1637_accepts_annotated_level_strings() {
        let tmp = tempdir().unwrap();
        cb1637_write_ticket(tmp.path(), "PMAT-200", "L3 (kani_proof)");
        cb1637_write_modified(tmp.path(), "PMAT-200", &["src/a.rs"]);
        cb1637_write_src(
            tmp.path(),
            "a.rs",
            "#[pmat_work_contract(id = \"PMAT-200\")]\npub fn f() {}\n",
        );
        let r = check_l2_public_fn_coverage(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1637_pub_crate_modifier_counts_as_pub_fn() {
        let tmp = tempdir().unwrap();
        cb1637_write_ticket(tmp.path(), "PMAT-200", "L2");
        cb1637_write_modified(tmp.path(), "PMAT-200", &["src/a.rs"]);
        cb1637_write_src(tmp.path(), "a.rs", "pub(crate) fn f() {}\n");
        let r = check_l2_public_fn_coverage(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("src/a.rs"));
    }

    #[test]
    fn cb1637_pub_async_fn_counts_as_pub_fn() {
        let tmp = tempdir().unwrap();
        cb1637_write_ticket(tmp.path(), "PMAT-200", "L2");
        cb1637_write_modified(tmp.path(), "PMAT-200", &["src/a.rs"]);
        cb1637_write_src(tmp.path(), "a.rs", "pub async fn f() {}\n");
        let r = check_l2_public_fn_coverage(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
    }

    #[test]
    fn cb1637_aggregates_violations_across_tickets() {
        let tmp = tempdir().unwrap();
        cb1637_write_ticket(tmp.path(), "PMAT-A", "L2");
        cb1637_write_ticket(tmp.path(), "PMAT-B", "L3");
        cb1637_write_modified(tmp.path(), "PMAT-A", &["src/a.rs"]);
        cb1637_write_modified(tmp.path(), "PMAT-B", &["src/b.rs"]);
        cb1637_write_src(tmp.path(), "a.rs", "pub fn a() {}\n");
        cb1637_write_src(tmp.path(), "b.rs", "pub fn b() {}\n");
        let r = check_l2_public_fn_coverage(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("2 file(s)"));
        assert!(r.message.contains("PMAT-A"));
        assert!(r.message.contains("PMAT-B"));
    }

    #[test]
    fn cb1637_pub_struct_does_not_count_as_pub_fn() {
        let tmp = tempdir().unwrap();
        cb1637_write_ticket(tmp.path(), "PMAT-200", "L2");
        cb1637_write_modified(tmp.path(), "PMAT-200", &["src/a.rs"]);
        cb1637_write_src(tmp.path(), "a.rs", "pub struct S;\npub const X: u32 = 1;\n");
        let r = check_l2_public_fn_coverage(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
    }

    #[test]
    fn cb1637_helpers_parse_verification_level() {
        let l2 = serde_json::json!({"verification_level":"L2"});
        let l3_annotated = serde_json::json!({"verification_level":"L3 (kani_proof)"});
        let lower_case = serde_json::json!({"verification_level":"l4"});
        let bogus = serde_json::json!({"verification_level":"strong"});
        let missing = serde_json::json!({});
        assert!(contract_level_at_least(&l2, 2));
        assert!(!contract_level_at_least(&l2, 3));
        assert!(contract_level_at_least(&l3_annotated, 2));
        assert!(contract_level_at_least(&l3_annotated, 3));
        assert!(!contract_level_at_least(&l3_annotated, 4));
        assert!(contract_level_at_least(&lower_case, 4));
        assert!(!contract_level_at_least(&bogus, 2));
        assert!(!contract_level_at_least(&missing, 2));
    }

    #[test]
    fn cb1637_helpers_detect_pub_fn_variants() {
        assert!(file_has_pub_fn("pub fn f() {}"));
        assert!(file_has_pub_fn("pub(crate) fn f() {}"));
        assert!(file_has_pub_fn("    pub async fn f() {}"));
        assert!(file_has_pub_fn("pub extern \"C\" fn f() {}"));
        assert!(!file_has_pub_fn("fn f() {}"));
        assert!(!file_has_pub_fn("pub struct S;"));
        assert!(!file_has_pub_fn("// pub fn f() {}"));
    }

    // ── CB-1639 Kani harness macro reference tests ──────────────────────

    /// Materialise a ticket at `.pmat-work/<id>/contract.json` with a
    /// `verification_level` and optional `implements:` bindings.
    fn cb1639_write_ticket(
        root: &Path,
        id: &str,
        level: &str,
        implements: &[(&str, &str, &str)], // (contract, equation, file)
    ) {
        let dir = root.join(".pmat-work").join(id);
        std::fs::create_dir_all(&dir).unwrap();
        let impl_json = serde_json::Value::Array(
            implements
                .iter()
                .map(|(c, e, f)| {
                    serde_json::json!({
                        "contract": c,
                        "equation": e,
                        "file": f,
                        "sha": "deadbeef",
                        "bound_at": "2026-04-18T00:00:00Z",
                    })
                })
                .collect(),
        );
        let body = serde_json::json!({
            "verification_level": level,
            "implements": impl_json,
        });
        std::fs::write(dir.join("contract.json"), body.to_string()).unwrap();
    }

    /// Write a YAML at `root/<rel>` with the given contents.
    fn cb1639_write_yaml(root: &Path, rel: &str, contents: &str) {
        let p = root.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(p, contents).unwrap();
    }

    /// Write a harness file at `root/<rel>` with the given body.
    fn cb1639_write_harness(root: &Path, rel: &str, body: &str) {
        let p = root.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(p, body).unwrap();
    }

    #[test]
    fn cb1639_skips_when_no_pmat_work_dir() {
        let tmp = tempdir().unwrap();
        let r = check_kani_harness_macro_reference(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/`"));
    }

    #[test]
    fn cb1639_skips_when_no_tickets() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-work")).unwrap();
        let r = check_kani_harness_macro_reference(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/<ID>/contract.json`"));
    }

    #[test]
    fn cb1639_skips_when_only_below_l4() {
        let tmp = tempdir().unwrap();
        cb1639_write_ticket(tmp.path(), "PMAT-1", "L3", &[]);
        cb1639_write_ticket(tmp.path(), "PMAT-2", "L1", &[]);
        let r = check_kani_harness_macro_reference(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("L4 or higher"));
    }

    #[test]
    fn cb1639_skips_when_l4_has_no_bindings() {
        let tmp = tempdir().unwrap();
        cb1639_write_ticket(tmp.path(), "PMAT-1", "L4", &[]);
        let r = check_kani_harness_macro_reference(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("`implements:` bindings"));
    }

    #[test]
    fn cb1639_skips_when_yaml_has_no_kani_harnesses() {
        let tmp = tempdir().unwrap();
        cb1639_write_yaml(tmp.path(), "contracts/x.yaml", "equations:\n  rope: {}\n");
        cb1639_write_ticket(
            tmp.path(),
            "PMAT-1",
            "L4",
            &[("x", "rope", "contracts/x.yaml")],
        );
        let r = check_kani_harness_macro_reference(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("`kani_harnesses:`"));
    }

    #[test]
    fn cb1639_skips_when_no_harness_body_found() {
        let tmp = tempdir().unwrap();
        cb1639_write_yaml(
            tmp.path(),
            "contracts/x.yaml",
            "kani_harnesses:\n  - verify_rope\n",
        );
        cb1639_write_ticket(
            tmp.path(),
            "PMAT-1",
            "L4",
            &[("x", "rope", "contracts/x.yaml")],
        );
        // No harness body exists in kani/, tests/, harnesses/, or src/.
        let r = check_kani_harness_macro_reference(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("Kani integration pending"));
    }

    #[test]
    fn cb1639_passes_when_harness_body_references_macro_module() {
        let tmp = tempdir().unwrap();
        cb1639_write_yaml(
            tmp.path(),
            "contracts/x.yaml",
            "kani_harnesses:\n  - verify_rope\n",
        );
        cb1639_write_ticket(
            tmp.path(),
            "PMAT-200",
            "L4",
            &[("x", "rope", "contracts/x.yaml")],
        );
        cb1639_write_harness(
            tmp.path(),
            "kani/rope.rs",
            "use contracts::work::PMAT_200;\n#[kani::proof]\nfn verify_rope() { }\n",
        );
        let r = check_kani_harness_macro_reference(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("1 Kani harness body"));
    }

    #[test]
    fn cb1639_passes_when_harness_file_has_attribute_form() {
        let tmp = tempdir().unwrap();
        cb1639_write_yaml(
            tmp.path(),
            "contracts/x.yaml",
            "kani_harnesses:\n  - verify_rope\n",
        );
        cb1639_write_ticket(
            tmp.path(),
            "PMAT-200",
            "L4",
            &[("x", "rope", "contracts/x.yaml")],
        );
        cb1639_write_harness(
            tmp.path(),
            "kani/rope.rs",
            "#[pmat_work_contract(id = \"PMAT-200\")]\nfn verify_rope() { }\n",
        );
        let r = check_kani_harness_macro_reference(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1639_fails_when_harness_body_is_orphaned() {
        let tmp = tempdir().unwrap();
        cb1639_write_yaml(
            tmp.path(),
            "contracts/x.yaml",
            "kani_harnesses:\n  - verify_rope\n",
        );
        cb1639_write_ticket(
            tmp.path(),
            "PMAT-200",
            "L4",
            &[("x", "rope", "contracts/x.yaml")],
        );
        // Harness body exists but doesn't reference any generated macro.
        cb1639_write_harness(tmp.path(), "kani/rope.rs", "fn verify_rope() { }\n");
        let r = check_kani_harness_macro_reference(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("verify_rope"));
        assert!(r.message.contains("PMAT-200"));
    }

    #[test]
    fn cb1639_fails_when_attribute_id_mismatches() {
        let tmp = tempdir().unwrap();
        cb1639_write_yaml(
            tmp.path(),
            "contracts/x.yaml",
            "kani_harnesses:\n  - verify_rope\n",
        );
        cb1639_write_ticket(
            tmp.path(),
            "PMAT-200",
            "L4",
            &[("x", "rope", "contracts/x.yaml")],
        );
        // Attribute references a different ticket — should still fail for PMAT-200.
        cb1639_write_harness(
            tmp.path(),
            "kani/rope.rs",
            "#[pmat_work_contract(id = \"PMAT-999\")]\nfn verify_rope() { }\n",
        );
        let r = check_kani_harness_macro_reference(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
    }

    #[test]
    fn cb1639_parses_yaml_string_form_harnesses() {
        let yaml = "kani_harnesses:\n  - verify_foo\n  - verify_bar\n";
        let names = yaml_kani_harness_names(yaml);
        assert_eq!(names, vec!["verify_foo", "verify_bar"]);
    }

    #[test]
    fn cb1639_parses_yaml_object_form_harnesses() {
        let yaml = "kani_harnesses:\n  - name: verify_foo\n    sha: abc\n  - name: verify_bar\n";
        let names = yaml_kani_harness_names(yaml);
        assert_eq!(names, vec!["verify_foo", "verify_bar"]);
    }

    #[test]
    fn cb1639_yaml_parser_ignores_other_sections() {
        let yaml =
            "equations:\n  rope:\n    - verify_not_a_harness\nkani_harnesses:\n  - real_one\n";
        let names = yaml_kani_harness_names(yaml);
        assert_eq!(names, vec!["real_one"]);
    }

    #[test]
    fn cb1639_file_reference_detection_matches_both_forms() {
        assert!(file_references_generated_macros(
            "use contracts::work::PMAT_200;",
            "PMAT-200"
        ));
        assert!(file_references_generated_macros(
            "contracts::work::PMAT_200::require_R1!()",
            "PMAT-200"
        ));
        assert!(file_references_generated_macros(
            "#[pmat_work_contract(id = \"PMAT-200\")]",
            "PMAT-200"
        ));
        assert!(!file_references_generated_macros(
            "// just a comment about pmat",
            "PMAT-200"
        ));
        assert!(!file_references_generated_macros(
            "contracts::work::PMAT_999",
            "PMAT-200"
        ));
    }

    #[test]
    fn cb1639_find_file_declaring_harness_returns_none_when_absent() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("kani")).unwrap();
        std::fs::write(tmp.path().join("kani/other.rs"), "fn something_else() {}\n").unwrap();
        let hit = find_file_declaring_harness(tmp.path(), "verify_foo");
        assert!(hit.is_none());
    }

    #[test]
    fn cb1639_find_file_prefers_kani_over_src() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("kani")).unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        std::fs::write(tmp.path().join("src/foo.rs"), "fn verify_foo() {}\n").unwrap();
        std::fs::write(tmp.path().join("kani/foo.rs"), "fn verify_foo() {}\n").unwrap();
        let hit = find_file_declaring_harness(tmp.path(), "verify_foo").unwrap();
        assert!(hit.to_string_lossy().contains("kani"));
    }
}
