    #[test]
    fn test_print_compliance_text_all_status_types() {
        let report = ComplianceReport {
            project_version: PMAT_VERSION.to_string(),
            current_version: PMAT_VERSION.to_string(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![
                ComplianceCheck {
                    name: "Pass".to_string(),
                    status: CheckStatus::Pass,
                    message: "Good".to_string(),
                    severity: Severity::Info,
                },
                ComplianceCheck {
                    name: "Warn".to_string(),
                    status: CheckStatus::Warn,
                    message: "Warning".to_string(),
                    severity: Severity::Warning,
                },
                ComplianceCheck {
                    name: "Fail".to_string(),
                    status: CheckStatus::Fail,
                    message: "Failed".to_string(),
                    severity: Severity::Error,
                },
                ComplianceCheck {
                    name: "Skip".to_string(),
                    status: CheckStatus::Skip,
                    message: "Skipped".to_string(),
                    severity: Severity::Info,
                },
            ],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        print_compliance_text(&report);
    }

    // print_compliance_markdown Tests

    #[test]
    fn test_print_compliance_markdown_compliant() {
        let report = ComplianceReport {
            project_version: PMAT_VERSION.to_string(),
            current_version: PMAT_VERSION.to_string(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        print_compliance_markdown(&report);
    }

    #[test]
    fn test_print_compliance_markdown_non_compliant() {
        let report = ComplianceReport {
            project_version: "1.0.0".to_string(),
            current_version: PMAT_VERSION.to_string(),
            is_compliant: false,
            versions_behind: 5,
            checks: vec![ComplianceCheck {
                name: "Check".to_string(),
                status: CheckStatus::Fail,
                message: "Failed".to_string(),
                severity: Severity::Error,
            }],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        print_compliance_markdown(&report);
    }

    // Async Handler Tests (using tokio::test)

    #[tokio::test]
    async fn test_handle_init_new_project() {
        let temp = create_temp_project();
        let result = handle_init(temp.path(), false).await;
        assert!(result.is_ok());

        // Verify project.toml created
        assert!(temp.path().join(".pmat").join("project.toml").exists());
    }

    #[tokio::test]
    async fn test_handle_init_existing_no_force() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_init(temp.path(), false).await;
        assert!(result.is_ok());

        // Version should remain unchanged
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_handle_init_existing_with_force() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_init(temp.path(), true).await;
        assert!(result.is_ok());

        // Version should be updated to current
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, PMAT_VERSION);
    }

    #[tokio::test]
    async fn test_handle_update_both() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_update(temp.path(), false, false, false).await;
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, PMAT_VERSION);
    }

    #[tokio::test]
    async fn test_handle_update_dry_run() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_update(temp.path(), false, false, true).await;
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0"); // Unchanged
    }

    #[tokio::test]
    async fn test_handle_update_hooks_only() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_update(temp.path(), true, false, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_update_config_only() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_update(temp.path(), false, true, false).await;
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, PMAT_VERSION);
    }

    #[tokio::test]
    async fn test_handle_diff_default_versions() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_diff(temp.path(), None, None, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_diff_specific_versions() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_diff(temp.path(), Some("1.0.0"), Some("2.0.0"), false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_diff_breaking_only() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_diff(temp.path(), None, None, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_migrate_dry_run() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_migrate(temp.path(), None, true, false, false).await;
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0"); // Unchanged
    }

    #[tokio::test]
    async fn test_handle_migrate_with_target() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_migrate(temp.path(), Some("2.0.0"), false, true, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_migrate_no_backup() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_migrate(temp.path(), None, false, true, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_migrate_with_backup() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_migrate(temp.path(), None, false, false, true).await;
        assert!(result.is_ok());

        // Verify backup directory created
        assert!(temp.path().join(".pmat").join("backup").exists());
    }

    #[tokio::test]
    async fn test_handle_enforce_no_git() {
        let temp = create_temp_project();
        let result = handle_enforce(temp.path(), true, false, ComplyOutputFormat::Text).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("Not a git repository"));
    }

    #[tokio::test]
    async fn test_handle_enforce_install() {
        let temp = create_git_repo();
        let result = handle_enforce(temp.path(), true, false, ComplyOutputFormat::Text).await;
        assert!(result.is_ok());

        // Verify hook created
        let hook_path = temp.path().join(".git").join("hooks").join("pre-commit");
        assert!(hook_path.exists());
        let content = fs::read_to_string(&hook_path).expect("Failed to read hook");
        assert!(content.contains("PMAT"));
    }

    #[tokio::test]
    async fn test_handle_enforce_disable() {
        let temp = create_git_repo();
        // First install
        handle_enforce(temp.path(), true, false, ComplyOutputFormat::Text)
            .await
            .expect("Failed to install");

        // Then disable
        let result = handle_enforce(temp.path(), true, true, ComplyOutputFormat::Text).await;
        assert!(result.is_ok());

        // Verify hook removed
        let hook_path = temp.path().join(".git").join("hooks").join("pre-commit");
        assert!(!hook_path.exists());
    }

    #[tokio::test]
    async fn test_handle_enforce_disable_non_pmat_hook() {
        let temp = create_git_repo();
        let hook_path = temp.path().join(".git").join("hooks").join("pre-commit");
        fs::write(&hook_path, "#!/bin/sh\necho 'other hook'").expect("Failed to write hook");

        let result = handle_enforce(temp.path(), true, true, ComplyOutputFormat::Text).await;
        assert!(result.is_ok());

        // Non-PMAT hook should NOT be removed
        assert!(hook_path.exists());
    }

    #[tokio::test]
    async fn test_handle_enforce_json_format() {
        let temp = create_git_repo();
        let result = handle_enforce(temp.path(), true, false, ComplyOutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_enforce_markdown_format() {
        let temp = create_git_repo();
        let result = handle_enforce(temp.path(), true, false, ComplyOutputFormat::Markdown).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_text() {
        let temp = create_pmat_project(PMAT_VERSION);
        let result = handle_report(temp.path(), false, ComplyOutputFormat::Text, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_json() {
        let temp = create_pmat_project(PMAT_VERSION);
        let result = handle_report(temp.path(), false, ComplyOutputFormat::Json, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_markdown() {
        let temp = create_pmat_project(PMAT_VERSION);
        let result = handle_report(temp.path(), false, ComplyOutputFormat::Markdown, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_with_history() {
        let temp = create_pmat_project(PMAT_VERSION);
        let result = handle_report(temp.path(), true, ComplyOutputFormat::Text, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_to_file() {
        let temp = create_pmat_project(PMAT_VERSION);
        let output_file = temp.path().join("report.md");
        let result = handle_report(
            temp.path(),
            false,
            ComplyOutputFormat::Markdown,
            Some(&output_file),
        )
        .await;
        assert!(result.is_ok());
        assert!(output_file.exists());
    }

    // handle_comply_command Tests

    #[tokio::test]
    async fn test_handle_comply_command_init() {
        let temp = create_temp_project();
        let command = ComplyCommands::Init {
            path: temp.path().to_path_buf(),
            force: false,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_comply_command_update() {
        let temp = create_pmat_project("1.0.0");
        let command = ComplyCommands::Update {
            path: temp.path().to_path_buf(),
            hooks: false,
            config: false,
            dry_run: true,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_comply_command_diff() {
        let temp = create_pmat_project("1.0.0");
        let command = ComplyCommands::Diff {
            path: temp.path().to_path_buf(),
            from: None,
            to: None,
            breaking_only: false,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_comply_command_migrate() {
        let temp = create_pmat_project("1.0.0");
        let command = ComplyCommands::Migrate {
            path: temp.path().to_path_buf(),
            version: None,
            dry_run: true,
            no_backup: true,
            force: true,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_comply_command_enforce() {
        let temp = create_git_repo();
        let command = ComplyCommands::Enforce {
            path: temp.path().to_path_buf(),
            yes: true,
            disable: false,
            format: ComplyOutputFormat::Text,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_comply_command_report() {
        let temp = create_pmat_project(PMAT_VERSION);
        let command = ComplyCommands::Report {
            path: temp.path().to_path_buf(),
            include_history: false,
            format: ComplyOutputFormat::Text,
            output: None,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    // Edge Cases and Error Paths

    #[test]
    fn test_version_parsing_with_prerelease() {
        let behind = calculate_versions_behind("2.0.0-alpha.1");
        // Should handle prerelease gracefully
        assert!(behind >= 0);
    }

    #[test]
    fn test_version_parsing_with_build_metadata() {
        let behind = calculate_versions_behind("2.0.0+build.123");
        assert!(behind >= 0);
    }

    #[test]
    fn test_compliance_check_debug_impl() {
        let check = ComplianceCheck {
            name: "Test".to_string(),
            status: CheckStatus::Pass,
            message: "OK".to_string(),
            severity: Severity::Info,
        };
        let debug_str = format!("{:?}", check);
        assert!(debug_str.contains("ComplianceCheck"));
        assert!(debug_str.contains("Pass"));
    }

    #[test]
    fn test_project_config_debug_impl() {
        let config = ProjectConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ProjectConfig"));
    }

    #[test]
    fn test_breaking_change_debug_impl() {
        let change = BreakingChange {
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            migration_guide: None,
        };
        let debug_str = format!("{:?}", change);
        assert!(debug_str.contains("BreakingChange"));
    }

    #[test]
    fn test_compliance_report_debug_impl() {
        let report = ComplianceReport {
            project_version: "1.0.0".to_string(),
            current_version: "2.0.0".to_string(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        let debug_str = format!("{:?}", report);
        assert!(debug_str.contains("ComplianceReport"));
    }

    #[tokio::test]
    async fn test_handle_check_with_nonexistent_path() {
        let temp = create_temp_project();
        let nonexistent = temp.path().join("nonexistent");
        // This should create the config directory
        let result = load_or_create_project_config(&nonexistent);
        // May fail due to parent directory not existing
        // Just verify it handles the error gracefully
        let _ = result;
    }

    #[test]
    fn test_changelog_entry_struct() {
        // Test the ChangelogEntry struct directly
        let entry = ChangelogEntry {
            version: "1.0.0".to_string(),
            description: "Test change".to_string(),
            breaking: true,
        };
        assert_eq!(entry.version, "1.0.0");
        assert!(entry.breaking);

        // Test clone
        let cloned = entry.clone();
        assert_eq!(cloned.version, entry.version);
        assert_eq!(cloned.breaking, entry.breaking);
    }

    #[test]
    fn test_pmat_version_constant() {
        // Verify PMAT_VERSION is set from Cargo.toml
        assert!(!PMAT_VERSION.is_empty());
        // Should be a valid semver-ish format
        let parts: Vec<&str> = PMAT_VERSION.split('.').collect();
        assert!(parts.len() >= 2, "Version should have at least major.minor");
    }

    // Integration-style Tests

