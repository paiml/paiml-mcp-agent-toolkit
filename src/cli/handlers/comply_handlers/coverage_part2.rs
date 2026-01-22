    #[test]
    fn test_check_compute_brick_with_probar_dep() {
        let temp = create_temp_project();
        let cargo_content = r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
probar = "0.1"
"#;
        fs::write(temp.path().join("Cargo.toml"), cargo_content)
            .expect("Failed to write Cargo.toml");

        let check = check_compute_brick(temp.path());
        // Has probar but no .pmat-gates.toml
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("[compute-brick]"));
    }

    #[test]
    fn test_check_compute_brick_with_brick_dir() {
        let temp = create_temp_project();
        fs::create_dir_all(temp.path().join("src").join("brick"))
            .expect("Failed to create brick dir");

        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
    }

    #[test]
    fn test_check_compute_brick_fully_configured() {
        let temp = create_temp_project();
        fs::create_dir_all(temp.path().join("src").join("brick"))
            .expect("Failed to create brick dir");

        let gates_content = r#"
[compute-brick]
enabled = true
"#;
        fs::write(temp.path().join(".pmat-gates.toml"), gates_content)
            .expect("Failed to write gates");

        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_check_compute_brick_probar_without_coverage() {
        let temp = create_temp_project();
        let cargo_content = r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
probar = "0.1"
"#;
        fs::write(temp.path().join("Cargo.toml"), cargo_content)
            .expect("Failed to write Cargo.toml");

        let gates_content = r#"
[compute-brick]
enabled = true
"#;
        fs::write(temp.path().join(".pmat-gates.toml"), gates_content)
            .expect("Failed to write gates");

        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("GUI coverage"));
    }

    // check_cargo_lock Tests

    #[test]
    fn test_check_cargo_lock_missing() {
        let temp = create_rust_project(false, false);
        let check = check_cargo_lock(temp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("Missing Cargo.lock"));
    }

    #[test]
    fn test_check_cargo_lock_present() {
        let temp = create_rust_project(false, true);
        let check = check_cargo_lock(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("reproducible builds"));
    }

    // check_msrv Tests

    #[test]
    fn test_check_msrv_no_cargo_toml() {
        let temp = create_temp_project();
        let check = check_msrv(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("No Cargo.toml"));
    }

    #[test]
    fn test_check_msrv_missing() {
        let temp = create_rust_project(false, false);
        let check = check_msrv(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("No rust-version"));
    }

    #[test]
    fn test_check_msrv_present() {
        let temp = create_rust_project(true, false);
        let check = check_msrv(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("rust-version field present"));
    }

    // check_ci_configured Tests

    #[test]
    fn test_check_ci_not_configured() {
        let temp = create_temp_project();
        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("No CI configuration"));
    }

    #[test]
    fn test_check_ci_github_actions() {
        let temp = create_temp_project();
        let workflows_dir = temp.path().join(".github").join("workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create workflows dir");
        fs::write(workflows_dir.join("ci.yml"), "name: CI").expect("Failed to write workflow");

        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("GitHub Actions"));
    }

    #[test]
    fn test_check_ci_github_actions_empty() {
        let temp = create_temp_project();
        let workflows_dir = temp.path().join(".github").join("workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create workflows dir");

        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
    }

    #[test]
    fn test_check_ci_gitlab() {
        let temp = create_temp_project();
        fs::write(temp.path().join(".gitlab-ci.yml"), "stages:\n  - build")
            .expect("Failed to write gitlab-ci");

        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("GitLab CI"));
    }

    #[test]
    fn test_check_ci_jenkins() {
        let temp = create_temp_project();
        fs::write(temp.path().join("Jenkinsfile"), "pipeline { }")
            .expect("Failed to write Jenkinsfile");

        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("Jenkins"));
    }

    // check_paiml_deps_workspace Tests

    #[test]
    fn test_check_paiml_deps_no_cargo_toml() {
        let temp = tempfile::tempdir().expect("Failed to create temp dir");
        let check = check_paiml_deps_workspace(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("No Cargo.toml"));
    }

    #[test]
    fn test_check_paiml_deps_no_paiml_deps() {
        let temp = create_temp_project();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "test-project"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#,
        )
        .expect("Failed to write Cargo.toml");

        let check = check_paiml_deps_workspace(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("No PAIML stack dependencies"));
    }

    #[test]
    fn test_check_paiml_deps_with_trueno() {
        let temp = create_temp_project();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "test-project"
version = "0.1.0"

[dependencies]
trueno = "0.11"
serde = "1.0"
"#,
        )
        .expect("Failed to write Cargo.toml");

        let check = check_paiml_deps_workspace(temp.path());
        // Status depends on whether ~/src/trueno exists and its git state
        // But check name should always be correct
        assert_eq!(check.name, "PAIML Deps Workspace");
    }

    #[test]
    fn test_check_paiml_deps_with_multiple_paiml_deps() {
        let temp = create_temp_project();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "test-project"
version = "0.1.0"

[dependencies]
trueno = "0.11"
trueno-graph = "0.1"
aprender = "0.24"
"#,
        )
        .expect("Failed to write Cargo.toml");

        let check = check_paiml_deps_workspace(temp.path());
        assert_eq!(check.name, "PAIML Deps Workspace");
        // Message should mention PAIML deps count or dirty status
        assert!(
            check.message.contains("PAIML") || check.message.contains("dirty"),
            "Expected message about PAIML deps, got: {}",
            check.message
        );
    }

    // get_breaking_changes_since Tests

    #[test]
    fn test_get_breaking_changes_since_returns_empty() {
        let changes = get_breaking_changes_since("1.0.0");
        assert!(changes.is_empty());
    }

    #[test]
    fn test_get_breaking_changes_since_any_version() {
        let changes = get_breaking_changes_since("0.0.1");
        assert!(changes.is_empty());
    }

    // get_changelog_entries Tests

    #[test]
    fn test_get_changelog_entries_returns_entries() {
        let entries = get_changelog_entries("1.0.0", "2.0.0");
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_get_changelog_entries_contain_expected_features() {
        let entries = get_changelog_entries("1.0.0", PMAT_VERSION);
        let has_comply = entries.iter().any(|e| e.description.contains("comply"));
        assert!(has_comply);
    }

    #[test]
    fn test_changelog_entry_breaking_flag() {
        let entries = get_changelog_entries("1.0.0", "2.0.0");
        // Current implementation has no breaking changes
        let breaking_count = entries.iter().filter(|e| e.breaking).count();
        assert_eq!(breaking_count, 0);
    }

    // load_or_create_project_config Tests

    #[test]
    fn test_load_or_create_config_creates_new() {
        let temp = create_temp_project();
        let config =
            load_or_create_project_config(temp.path()).expect("Failed to load/create config");
        assert_eq!(config.pmat.version, PMAT_VERSION);

        // Verify file was created
        assert!(temp.path().join(".pmat").join("project.toml").exists());
    }

    #[test]
    fn test_load_or_create_config_loads_existing() {
        let temp = create_pmat_project("1.0.0");
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0");
    }

    #[test]
    fn test_load_or_create_config_invalid_toml() {
        let temp = create_temp_project();
        let pmat_dir = temp.path().join(".pmat");
        fs::create_dir_all(&pmat_dir).expect("Failed to create .pmat");
        fs::write(pmat_dir.join("project.toml"), "invalid { toml").expect("Failed to write");

        let result = load_or_create_project_config(temp.path());
        assert!(result.is_err());
    }

    // update_last_check_timestamp Tests

    #[test]
    fn test_update_last_check_timestamp() {
        let temp = create_pmat_project(PMAT_VERSION);

        let result = update_last_check_timestamp(temp.path());
        assert!(result.is_ok());

        // Verify timestamp was updated
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert!(config.pmat.last_compliance_check.is_some());
    }

    #[test]
    fn test_update_last_check_timestamp_no_config() {
        let temp = create_temp_project();
        let result = update_last_check_timestamp(temp.path());
        // Should succeed even if config doesn't exist
        assert!(result.is_ok());
    }

    // migrate_project_version Tests

    #[test]
    fn test_migrate_project_version_dry_run() {
        let temp = create_pmat_project("1.0.0");
        let result = migrate_project_version(temp.path(), "2.0.0", true);
        assert!(result.is_ok());
        assert!(result.unwrap()); // dry_run always returns true

        // Verify version NOT changed
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0");
    }

    #[test]
    fn test_migrate_project_version_actual() {
        let temp = create_pmat_project("1.0.0");
        let result = migrate_project_version(temp.path(), "2.0.0", false);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify version changed
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "2.0.0");
    }

    #[test]
    fn test_migrate_project_version_same_version() {
        let temp = create_pmat_project("1.0.0");
        let result = migrate_project_version(temp.path(), "1.0.0", false);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // No change needed
    }

    // migrate_gitignore Tests

    #[test]
    fn test_migrate_gitignore_no_file() {
        let temp = create_temp_project();
        let result = migrate_gitignore(temp.path(), false);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_migrate_gitignore_adds_entries() {
        let temp = create_temp_project();
        fs::write(temp.path().join(".gitignore"), "target/\n").expect("Failed to write gitignore");

        let result = migrate_gitignore(temp.path(), false);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let content = fs::read_to_string(temp.path().join(".gitignore")).expect("Failed to read");
        assert!(content.contains(".pmat/backup/"));
        assert!(content.contains(".pmat-qa/"));
    }

    #[test]
    fn test_migrate_gitignore_already_has_entries() {
        let temp = create_temp_project();
        fs::write(
            temp.path().join(".gitignore"),
            "target/\n.pmat/backup/\n.pmat-qa/\n",
        )
        .expect("Failed to write gitignore");

        let result = migrate_gitignore(temp.path(), false);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // No changes needed
    }

    #[test]
    fn test_migrate_gitignore_dry_run() {
        let temp = create_temp_project();
        fs::write(temp.path().join(".gitignore"), "target/\n").expect("Failed to write gitignore");

        let result = migrate_gitignore(temp.path(), true);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Would need update

        // Verify file NOT changed
        let content = fs::read_to_string(temp.path().join(".gitignore")).expect("Failed to read");
        assert!(!content.contains(".pmat/backup/"));
    }

    #[test]
    fn test_migrate_gitignore_no_trailing_newline() {
        let temp = create_temp_project();
        fs::write(temp.path().join(".gitignore"), "target/").expect("Failed to write gitignore");

        let result = migrate_gitignore(temp.path(), false);
        assert!(result.is_ok());

        let content = fs::read_to_string(temp.path().join(".gitignore")).expect("Failed to read");
        // Should handle missing trailing newline
        assert!(content.contains("# PMAT"));
    }

    // update_project_config Tests

    #[test]
    fn test_update_project_config_updates_to_current() {
        let temp = create_pmat_project("1.0.0");
        let result = update_project_config(temp.path(), false);
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, PMAT_VERSION);
    }

    #[test]
    fn test_update_project_config_dry_run() {
        let temp = create_pmat_project("1.0.0");
        let result = update_project_config(temp.path(), true);
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0"); // Unchanged
    }

    // print_compliance_text Tests

    #[test]
    fn test_print_compliance_text_compliant() {
        let report = ComplianceReport {
            project_version: PMAT_VERSION.to_string(),
            current_version: PMAT_VERSION.to_string(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![ComplianceCheck {
                name: "Test".to_string(),
                status: CheckStatus::Pass,
                message: "OK".to_string(),
                severity: Severity::Info,
            }],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        // This just tests it doesn't panic
        print_compliance_text(&report);
    }

    #[test]
    fn test_print_compliance_text_non_compliant() {
        let report = ComplianceReport {
            project_version: "1.0.0".to_string(),
            current_version: PMAT_VERSION.to_string(),
            is_compliant: false,
            versions_behind: 10,
            checks: vec![ComplianceCheck {
                name: "Version".to_string(),
                status: CheckStatus::Fail,
                message: "Outdated".to_string(),
                severity: Severity::Error,
            }],
            breaking_changes: vec![],
            recommendations: vec!["Update PMAT".to_string()],
            timestamp: Utc::now(),
        };
        print_compliance_text(&report);
    }

