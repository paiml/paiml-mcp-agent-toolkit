    // get_breaking_changes_since Tests

    // These four tests asserted the STUB: `get_breaking_changes_since` returned
    // `[]` for every version and `get_changelog_entries` returned a canned
    // 3-entry list regardless of the range asked for. Two of them even said so
    // ("Current implementation always returns empty"). Both functions now read
    // the real CHANGELOG.md, so the tests assert against it instead.

    #[test]
    fn test_get_breaking_changes_since_finds_real_major_releases() {
        // pmat's changelog starts at 2.172.0, so everything in it is "since 1.0.0".
        let changes = get_breaking_changes_since("1.0.0");
        assert!(
            !changes.is_empty(),
            "the changelog contains major releases after 1.0.0; an empty result \
             means the stub is back"
        );
    }

    #[test]
    fn test_get_breaking_changes_since_is_bounded_by_the_version_given() {
        // Nothing was released after the version currently being built.
        let changes = get_breaking_changes_since(PMAT_VERSION);
        assert!(
            changes.is_empty(),
            "nothing can be breaking *since* the current version, got {changes:?}"
        );
    }

    // get_changelog_entries Tests

    #[test]
    fn test_get_changelog_entries_covers_a_real_range() {
        let entries = get_changelog_entries("2.172.0", PMAT_VERSION);
        assert!(
            !entries.is_empty(),
            "2.172.0..{PMAT_VERSION} spans the whole changelog"
        );
    }

    #[test]
    fn test_get_changelog_entries_is_empty_below_the_oldest_release() {
        // The oldest section in CHANGELOG.md is 2.172.0, so this range holds
        // nothing. The stub returned three canned entries for it.
        let entries = get_changelog_entries("1.0.0", "2.0.0");
        assert!(
            entries.is_empty(),
            "no release exists in 1.0.0..2.0.0, got {entries:?}"
        );
    }

    #[test]
    fn test_changelog_entries_report_the_version_they_came_from() {
        let entries = get_changelog_entries("2.172.0", PMAT_VERSION);
        assert!(entries.iter().all(|e| !e.version.is_empty()));
        assert!(
            entries.iter().any(|e| e.version != PMAT_VERSION),
            "every entry carrying the current version is the stub's signature"
        );
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
            history: None,
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
            history: None,
        };
        print_compliance_text(&report);
    }
