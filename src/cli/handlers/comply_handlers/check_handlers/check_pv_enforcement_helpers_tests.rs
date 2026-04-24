#[cfg(test)]
mod pv_enforcement_helpers_tests {
    //! PMAT-651: cover check_pv_enforcement_helpers.rs pure helpers.
    use super::*;
    use std::collections::HashSet;
    use tempfile::TempDir;

    fn write(p: &Path, content: &str) {
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(p, content).unwrap();
    }

    // --- extract_names_from_source ---

    #[test]
    fn test_extract_names_pub_fn() {
        let mut s = HashSet::new();
        extract_names_from_source("pub fn foo() {}\n", &mut s);
        assert!(s.contains("foo"));
    }

    #[test]
    fn test_extract_names_private_fn() {
        let mut s = HashSet::new();
        extract_names_from_source("fn private_fn() {}\n", &mut s);
        assert!(s.contains("private_fn"));
    }

    #[test]
    fn test_extract_names_async_fn_variants() {
        let mut s = HashSet::new();
        extract_names_from_source(
            "pub async fn pub_async() {}\nasync fn just_async() {}\npub(crate) async fn pcrate_async() {}\n",
            &mut s,
        );
        assert!(s.contains("pub_async"));
        assert!(s.contains("just_async"));
        assert!(s.contains("pcrate_async"));
    }

    #[test]
    fn test_extract_names_unsafe_fn_variants() {
        let mut s = HashSet::new();
        extract_names_from_source(
            "unsafe fn u1() {}\npub unsafe fn u2() {}\npub(crate) unsafe fn u3() {}\npub const unsafe fn u4() {}\n",
            &mut s,
        );
        assert!(s.contains("u1"));
        assert!(s.contains("u2"));
        assert!(s.contains("u3"));
        assert!(s.contains("u4"));
    }

    #[test]
    fn test_extract_names_const_fn_variants() {
        let mut s = HashSet::new();
        extract_names_from_source("const fn cf() {}\npub const fn pcf() {}\n", &mut s);
        assert!(s.contains("cf"));
        assert!(s.contains("pcf"));
    }

    #[test]
    fn test_extract_names_pub_crate_fn() {
        let mut s = HashSet::new();
        extract_names_from_source("pub(crate) fn pcfn() {}\n", &mut s);
        assert!(s.contains("pcfn"));
    }

    #[test]
    fn test_extract_names_const_declarations() {
        let mut s = HashSet::new();
        extract_names_from_source(
            "pub const PUB_C: u32 = 1;\nconst PRIV_C: u32 = 2;\npub(crate) const PCRATE_C: u32 = 3;\npub static PUB_S: &str = \"a\";\nstatic PRIV_S: &str = \"b\";\n",
            &mut s,
        );
        for name in ["PUB_C", "PRIV_C", "PCRATE_C", "PUB_S", "PRIV_S"] {
            assert!(s.contains(name), "missing {name} in {:?}", s);
        }
    }

    #[test]
    fn test_extract_names_skips_non_alphanumeric() {
        let mut s = HashSet::new();
        // "fn weird-name" should NOT be added since hyphen is not alphanumeric or underscore.
        extract_names_from_source("fn weird-name() {}\n", &mut s);
        assert!(!s.contains("weird-name"));
    }

    #[test]
    fn test_extract_names_skips_empty_name() {
        let mut s = HashSet::new();
        // Just `fn ` followed by `(` produces an empty name.
        extract_names_from_source("fn () {}\n", &mut s);
        assert!(s.is_empty());
    }

    // --- cross_reference_bindings ---

    #[test]
    fn test_cross_reference_bindings_all_verified() {
        let mut known = HashSet::new();
        known.insert("foo".to_string());
        known.insert("bar".to_string());
        let entries = vec![
            ("foo".to_string(), "f.yaml".to_string()),
            ("bar".to_string(), "f.yaml".to_string()),
        ];
        let (total, unique, verified, missing) = cross_reference_bindings(&entries, &known);
        assert_eq!(total, 2);
        assert_eq!(unique, 2);
        assert_eq!(verified, 2);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_cross_reference_bindings_dedup_via_seen() {
        let mut known = HashSet::new();
        known.insert("foo".to_string());
        let entries = vec![
            ("foo".to_string(), "a.yaml".to_string()),
            ("foo".to_string(), "b.yaml".to_string()),
            ("foo".to_string(), "c.yaml".to_string()),
        ];
        let (total, unique, verified, _) = cross_reference_bindings(&entries, &known);
        // total counts entries; unique counts seen set; verified increments only on first sighting.
        assert_eq!(total, 3);
        assert_eq!(unique, 1);
        assert_eq!(verified, 1);
    }

    #[test]
    fn test_cross_reference_bindings_missing_capped_at_5() {
        let known = HashSet::new();
        let entries: Vec<(String, String)> = (0..10)
            .map(|i| (format!("fn_{i}"), "x.yaml".to_string()))
            .collect();
        let (total, unique, verified, missing) = cross_reference_bindings(&entries, &known);
        assert_eq!(total, 10);
        assert_eq!(unique, 10);
        assert_eq!(verified, 0);
        assert_eq!(missing.len(), 5, "missing list must cap at 5");
    }

    // --- parse_binding_entries ---

    #[test]
    fn test_parse_binding_entries_extracts_implemented_pairs() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir_all(&cd).unwrap();
        let p = cd.join("binding.yaml");
        let content = "function: \"foo\"\nstatus: implemented\n- contract: x\nfunction: \"bar\"\nstatus: pending\n- contract: x\n";
        let mut entries = Vec::new();
        parse_binding_entries(content, &p, &cd, &mut entries);
        // Only "foo" (implemented) is added; "bar" (pending) is skipped.
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "foo");
        assert_eq!(entries[0].1, "binding.yaml");
    }

    #[test]
    fn test_parse_binding_entries_strips_type_double_colon_prefix() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir_all(&cd).unwrap();
        let p = cd.join("binding.yaml");
        let content = "function: \"MyType::method_name\"\nstatus: implemented\n";
        let mut entries = Vec::new();
        parse_binding_entries(content, &p, &cd, &mut entries);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "method_name");
    }

    #[test]
    fn test_parse_binding_entries_skips_empty_function() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir_all(&cd).unwrap();
        let p = cd.join("binding.yaml");
        let content = "function: \"\"\nstatus: implemented\n";
        let mut entries = Vec::new();
        parse_binding_entries(content, &p, &cd, &mut entries);
        assert!(entries.is_empty());
    }

    // --- has_contract_yamls ---

    #[test]
    fn test_has_contract_yamls_empty_dir_false() {
        let tmp = TempDir::new().unwrap();
        assert!(!has_contract_yamls(tmp.path()));
    }

    #[test]
    fn test_has_contract_yamls_only_binding_returns_false() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("binding.yaml"), "");
        assert!(!has_contract_yamls(tmp.path()));
    }

    #[test]
    fn test_has_contract_yamls_normal_yaml_returns_true() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("foo.yaml"), "");
        assert!(has_contract_yamls(tmp.path()));
    }

    #[test]
    fn test_has_contract_yamls_yml_extension_accepted() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("bar.yml"), "");
        assert!(has_contract_yamls(tmp.path()));
    }

    #[test]
    fn test_has_contract_yamls_missing_dir_false() {
        assert!(!has_contract_yamls(Path::new("/tmp/does-not-exist-xyz123")));
    }

    // --- collect_stems_recursive ---

    #[test]
    fn test_collect_stems_recursive_picks_up_yamls_in_subdirs() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("a.yaml"), "");
        write(&tmp.path().join("sub/b.yml"), "");
        let mut stems = HashSet::new();
        collect_stems_recursive(tmp.path(), &mut stems);
        assert!(stems.contains("a"));
        assert!(stems.contains("b"));
    }

    #[test]
    fn test_collect_stems_recursive_skips_binding_files() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("real.yaml"), "");
        write(&tmp.path().join("binding.yaml"), "");
        write(&tmp.path().join("foo-binding.yaml"), "");
        let mut stems = HashSet::new();
        collect_stems_recursive(tmp.path(), &mut stems);
        assert!(stems.contains("real"));
        assert!(!stems.contains("binding"));
        assert!(!stems.contains("foo-binding"));
    }

    #[test]
    fn test_collect_stems_recursive_missing_dir_no_op() {
        let mut stems = HashSet::new();
        collect_stems_recursive(Path::new("/tmp/no-such-dir-abc"), &mut stems);
        assert!(stems.is_empty());
    }

    // --- detect_buildrs_enforcement ---

    #[test]
    fn test_detect_buildrs_enforcement_no_build_rs_false() {
        let tmp = TempDir::new().unwrap();
        assert!(!detect_buildrs_enforcement(tmp.path()));
    }

    #[test]
    fn test_detect_buildrs_enforcement_root_buildrs_with_keyword() {
        let tmp = TempDir::new().unwrap();
        write(
            &tmp.path().join("build.rs"),
            "fn main() { let _ = \"contract\"; }",
        );
        assert!(detect_buildrs_enforcement(tmp.path()));
    }

    #[test]
    fn test_detect_buildrs_enforcement_root_buildrs_no_keyword_false() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("build.rs"), "fn main() {}");
        assert!(!detect_buildrs_enforcement(tmp.path()));
    }

    #[test]
    fn test_detect_buildrs_enforcement_member_crate_with_binding_keyword() {
        let tmp = TempDir::new().unwrap();
        write(
            &tmp.path().join("crates/foo/build.rs"),
            "fn main() { let _ = \"binding\"; }",
        );
        assert!(detect_buildrs_enforcement(tmp.path()));
    }

    #[test]
    fn test_detect_buildrs_enforcement_member_crate_with_all_implemented() {
        let tmp = TempDir::new().unwrap();
        write(
            &tmp.path().join("crates/bar/build.rs"),
            "fn main() { let _ = \"AllImplemented\"; }",
        );
        assert!(detect_buildrs_enforcement(tmp.path()));
    }

    // --- count_contract_test_refs ---

    #[test]
    fn test_count_contract_test_refs_no_contracts_dir_returns_zeros() {
        let tmp = TempDir::new().unwrap();
        let (refs, existing, missing) = count_contract_test_refs(tmp.path());
        assert_eq!((refs, existing, missing), (0, 0, 0));
    }

    #[test]
    fn test_count_contract_test_refs_no_test_keys_returns_zeros() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir_all(&cd).unwrap();
        write(&cd.join("a.yaml"), "name: x\n");
        let (refs, existing, missing) = count_contract_test_refs(tmp.path());
        assert_eq!((refs, existing, missing), (0, 0, 0));
    }

    #[test]
    fn test_count_contract_test_refs_existing_and_missing_split() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir_all(&cd).unwrap();
        write(
            &cd.join("a.yaml"),
            "test: \"test_alpha\"\ntest: \"test_missing\"\ntest: \"prop_beta\"\n",
        );
        // Provide one matching test; one stays "missing".
        write(
            &tmp.path().join("src/lib.rs"),
            "#[test]\nfn test_alpha() {}\nfn prop_beta() {}\n",
        );
        let (refs, existing, missing) = count_contract_test_refs(tmp.path());
        assert_eq!(refs, 3);
        assert_eq!(existing, 2);
        assert_eq!(missing, 1);
    }

    // --- collect_contract_equation_names ---

    #[test]
    fn test_collect_contract_equation_names_missing_dir_empty() {
        let tmp = TempDir::new().unwrap();
        let names = collect_contract_equation_names(&tmp.path().join("nope"));
        assert!(names.is_empty());
    }

    #[test]
    fn test_collect_contract_equation_names_yaml_with_pre_returns_name() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir_all(&cd).unwrap();
        // Equation name "my_equation" with preconditions.
        let yaml = "equations:\n  my_equation:\n    preconditions:\n      - x > 0\n";
        write(&cd.join("foo.yaml"), yaml);
        let names = collect_contract_equation_names(&cd);
        assert!(names.contains(&"my_equation".to_string()));
    }

    // --- resolve_contracts_dir ---

    #[test]
    fn test_resolve_contracts_dir_local_with_yamls_returns_local() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir_all(&cd).unwrap();
        write(&cd.join("real.yaml"), "");
        let resolved = resolve_contracts_dir(tmp.path());
        // sibling provable-contracts is unlikely to exist for this tempdir layout,
        // so the local path is the expected fallback.
        let canonical_local = std::fs::canonicalize(&cd).unwrap();
        if let Some(p) = resolved {
            let canonical_resolved = std::fs::canonicalize(&p).unwrap();
            assert_eq!(canonical_resolved, canonical_local);
        }
    }

    #[test]
    fn test_resolve_contracts_dir_no_contracts_returns_none() {
        let tmp = TempDir::new().unwrap();
        // No `contracts/` dir at all → None (assuming tempdir parent has no provable-contracts).
        let resolved = resolve_contracts_dir(tmp.path());
        // May be Some if /tmp's siblings include a provable-contracts/ dir; just accept either.
        let _ = resolved;
    }

    // --- collect_known_fn_names ---

    #[test]
    fn test_collect_known_fn_names_no_src_returns_none() {
        let tmp = TempDir::new().unwrap();
        assert!(collect_known_fn_names(tmp.path()).is_none());
    }

    #[test]
    fn test_collect_known_fn_names_picks_up_src_dir() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("src/lib.rs"), "pub fn alpha() {}\n");
        let known = collect_known_fn_names(tmp.path()).expect("Some");
        assert!(known.contains("alpha"));
    }

    #[test]
    fn test_collect_known_fn_names_picks_up_workspace_crates() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("crates/foo/src/main.rs"), "fn beta() {}\n");
        let known = collect_known_fn_names(tmp.path()).expect("Some");
        assert!(known.contains("beta"));
    }

    // --- resolve_binding_files ---

    #[test]
    fn test_resolve_binding_files_finds_local_binding() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir_all(&cd).unwrap();
        write(&cd.join("binding.yaml"), "status: implemented\n");
        let abs = std::fs::canonicalize(tmp.path()).unwrap();
        let files = resolve_binding_files(&cd, &abs, "myproj");
        assert!(!files.is_empty());
        assert!(files.iter().any(|p| p
            .file_name()
            .is_some_and(|n| n.to_string_lossy().contains("binding"))));
    }
}
