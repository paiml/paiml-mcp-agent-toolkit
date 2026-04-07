#[cfg(test)]
mod tests_commit_enforcement {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_cb1320_readme_with_all_sections() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("README.md"),
            "# My Project\n\n[![badge](https://shields.io/badge)]\n\n## Installation\n\nRun it.\n\n## Usage\n\nUse it.\n\n## License\n\nMIT\n",
        ).unwrap();

        let check = check_readme_layout(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1320_readme_missing_sections() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("README.md"), "# Project\n\nHello.\n").unwrap();

        let check = check_readme_layout(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("install"));
    }

    #[test]
    fn test_cb1320_no_readme() {
        let dir = tempdir().unwrap();
        let check = check_readme_layout(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
    }

    #[test]
    fn test_cb1325_changelog_valid() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("CHANGELOG.md"),
            "# Changelog\n\n## [1.0.0] - 2026-04-05\n\n### Added\n\n- Initial release\n",
        ).unwrap();

        let check = check_changelog_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1325_no_changelog() {
        let dir = tempdir().unwrap();
        let check = check_changelog_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1332_fresh_cache() {
        let dir = tempdir().unwrap();
        let pmat = dir.path().join(".pmat");
        fs::create_dir(&pmat).unwrap();
        fs::write(pmat.join("baseline.json"), "{}").unwrap();

        let check = check_cache_staleness(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1332_no_pmat_dir() {
        let dir = tempdir().unwrap();
        let check = check_cache_staleness(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1335_clean_hooks() {
        let dir = tempdir().unwrap();
        let hooks = dir.path().join(".git/hooks");
        fs::create_dir_all(&hooks).unwrap();
        fs::write(
            hooks.join("pre-commit"),
            "#!/bin/sh\npmat hook pre-commit --format --complexity\n",
        ).unwrap();

        let check = check_hook_determinism(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1335_nondeterministic_hook() {
        let dir = tempdir().unwrap();
        let hooks = dir.path().join(".git/hooks");
        fs::create_dir_all(&hooks).unwrap();
        fs::write(
            hooks.join("pre-commit"),
            "#!/bin/sh\n# Generated at: 2026-04-05T10:00:00Z\npmat hook pre-commit\n",
        ).unwrap();

        let check = check_hook_determinism(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("timestamp"));
    }

    #[test]
    fn test_cb1337_no_expensive_ops() {
        let dir = tempdir().unwrap();
        let hooks = dir.path().join(".git/hooks");
        fs::create_dir_all(&hooks).unwrap();
        fs::write(
            hooks.join("pre-commit"),
            "#!/bin/sh\npmat hook pre-commit --all\n",
        ).unwrap();

        let check = check_hook_performance(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1337_expensive_ops() {
        let dir = tempdir().unwrap();
        let hooks = dir.path().join(".git/hooks");
        fs::create_dir_all(&hooks).unwrap();
        fs::write(
            hooks.join("pre-commit"),
            "#!/bin/sh\ncargo test\ncargo build\n",
        ).unwrap();

        let check = check_hook_performance(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("cargo test"));
    }

    #[test]
    fn test_cb1321_no_dockerfile() {
        let dir = tempdir().unwrap();
        let check = check_dockerfile_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1321_good_dockerfile() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Dockerfile"),
            "FROM rust:1.80-slim\nRUN apt-get update\nUSER app\nCMD [\"./app\"]\n",
        ).unwrap();

        let check = check_dockerfile_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1321_latest_tag() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Dockerfile"),
            "FROM ubuntu:latest\nRUN echo hi\n",
        ).unwrap();

        let check = check_dockerfile_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains(":latest"));
    }

    #[test]
    fn test_cb1326_badges_present() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("README.md"),
            "# Proj\n[![CI](https://github.com/org/repo/actions/workflows/ci.yml/badge.svg)](x)\n[![crates.io](https://crates.io/v/proj)](y)\n## License\nMIT\n",
        ).unwrap();

        let check = check_badge_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1326_badges_missing() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("README.md"), "# Proj\nHello\n").unwrap();

        let check = check_badge_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("Missing"));
    }

    #[test]
    fn test_cb1333_no_src() {
        let dir = tempdir().unwrap();
        let check = check_hook_single_writer(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1333_single_writer() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src/hooks");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("registry.rs"),
            "fn install() { let p = \"hooks/pre-commit\"; fs::write(p, content); }",
        ).unwrap();

        let check = check_hook_single_writer(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1336_no_injection() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("hooks.rs"),
            "fn gen() { let s = format!(\"pre-commit hook\"); }",
        ).unwrap();

        let check = check_hook_no_injection(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1334_no_src() {
        let dir = tempdir().unwrap();
        let check = check_hook_atomic_writes(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1334_atomic_write() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("hooks.rs"),
            "fn install() { let p = \"hooks/pre-commit\"; fs::write(&tmp, c); fs::rename(&tmp, p); }",
        ).unwrap();

        let check = check_hook_atomic_writes(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1331_no_work_dir() {
        let dir = tempdir().unwrap();
        let check = check_work_contract_validity(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1331_valid_contract() {
        let dir = tempdir().unwrap();
        let work = dir.path().join(".pmat-work/PMAT-001");
        fs::create_dir_all(&work).unwrap();
        fs::write(
            work.join("contract.json"),
            r#"{"version":"5.0","work_item_id":"PMAT-001"}"#,
        ).unwrap();

        let check = check_work_contract_validity(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("1 valid"));
    }

    #[test]
    fn test_cb1331_invalid_json() {
        let dir = tempdir().unwrap();
        let work = dir.path().join(".pmat-work/PMAT-BAD");
        fs::create_dir_all(&work).unwrap();
        fs::write(work.join("contract.json"), "not json").unwrap();

        let check = check_work_contract_validity(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("invalid JSON"));
    }

    #[test]
    fn test_cb1330_no_contracts() {
        let dir = tempdir().unwrap();
        let check = check_verification_ratchet(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1338_no_binding() {
        let dir = tempdir().unwrap();
        let check = check_no_ghost_bindings(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1339_no_contracts() {
        let dir = tempdir().unwrap();
        let check = check_no_placeholder_preconditions(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1340_no_binding() {
        let dir = tempdir().unwrap();
        let check = check_enforcement_penetration(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1343_no_contracts() {
        let dir = tempdir().unwrap();
        let check = check_assertion_placement(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1322_no_svgs() {
        let dir = tempdir().unwrap();
        let check = check_svg_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1324_no_book() {
        let dir = tempdir().unwrap();
        let check = check_mdbook_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1324_valid_book() {
        let dir = tempdir().unwrap();
        let book_src = dir.path().join("book/src");
        fs::create_dir_all(&book_src).unwrap();
        fs::write(book_src.join("SUMMARY.md"), "# Summary\n\n- [Intro](intro.md)\n").unwrap();
        fs::write(book_src.join("intro.md"), "# Intro\n").unwrap();

        let check = check_mdbook_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1324_broken_link() {
        let dir = tempdir().unwrap();
        let book_src = dir.path().join("book/src");
        fs::create_dir_all(&book_src).unwrap();
        fs::write(book_src.join("SUMMARY.md"), "# Summary\n\n- [Missing](gone.md)\n").unwrap();

        let check = check_mdbook_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("gone.md"));
    }

    #[test]
    fn test_extract_level() {
        assert_eq!(extract_level("target_level: L3", "target_level"), Some(3));
        assert_eq!(extract_level("current_level: L1", "current_level"), Some(1));
        assert_eq!(extract_level("target_level: \"L5\"", "target_level"), Some(5));
        assert_eq!(extract_level("no level here", "target_level"), None);
    }

    #[test]
    fn test_count_braces_outside_literals() {
        // Normal code
        assert_eq!(count_braces_outside_literals("fn foo() {"), (1, 0));
        assert_eq!(count_braces_outside_literals("}"), (0, 1));
        assert_eq!(count_braces_outside_literals("{ }"), (1, 1));
        // Braces inside strings should be IGNORED
        assert_eq!(count_braces_outside_literals(r#"let s = "{{{{";"#), (0, 0));
        assert_eq!(count_braces_outside_literals(r#"let s = "}}}}";"#), (0, 0));
        assert_eq!(count_braces_outside_literals(r#"let s = "{}{{}}{";"#), (0, 0));
        // Braces inside char literals
        assert_eq!(count_braces_outside_literals("let c = '{';"), (0, 0));
        assert_eq!(count_braces_outside_literals("let c = '}';"), (0, 0));
        // Mixed: real brace outside string literal with braces
        assert_eq!(count_braces_outside_literals(r#"fn f() { let s = "{"; }"#), (1, 1));
        // Escaped quotes in strings
        assert_eq!(count_braces_outside_literals(r#"let s = "\"{\"";"#), (0, 0));
        // Raw strings r"..." — internal braces ignored
        assert_eq!(count_braces_outside_literals(r##"let s = r"{{{{";"##), (0, 0));
        // Raw string r#"..."# with embedded quotes and braces
        assert_eq!(
            count_braces_outside_literals(r######"let s = r#"end" fn foo() { "start"#;"######),
            (0, 0)
        );
        // Raw string should NOT leak its content as real braces
        assert_eq!(
            count_braces_outside_literals(r##"fn f() { let s = r#"}"#; }"##),
            (1, 1)
        );
        // r##"..."## — double hash raw strings
        assert_eq!(
            count_braces_outside_literals(r####"let s = r##"contains # and {"##;"####),
            (0, 0)
        );
    }

    #[test]
    fn test_cb1323_no_forjar() {
        let dir = tempdir().unwrap();
        let check = check_forjar_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1323_clean_forjar() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("forjar.yaml"),
            "name: myproject\nsteps:\n  - build\n  - test\n",
        ).unwrap();

        let check = check_forjar_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1323_secret_in_forjar() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("forjar.yaml"),
            "name: myproject\npassword: hunter2\napi_key: sk-abc123\n",
        ).unwrap();

        let check = check_forjar_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("plaintext"));
    }

}
