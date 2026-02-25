    // calculate_versions_behind Tests

    #[test]
    fn test_calculate_versions_behind_older_minor() {
        // Parse current version to get major.minor
        let parts: Vec<u32> = PMAT_VERSION
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 2 && parts[1] > 0 {
            let older = format!("{}.{}.0", parts[0], parts[1] - 1);
            let behind = calculate_versions_behind(&older);
            assert_eq!(behind, 1);
        }
    }

    #[test]
    fn test_calculate_versions_behind_same_version() {
        let behind = calculate_versions_behind(PMAT_VERSION);
        assert_eq!(behind, 0);
    }

    #[test]
    fn test_calculate_versions_behind_newer_version() {
        let parts: Vec<u32> = PMAT_VERSION
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 2 {
            let newer = format!("{}.{}.0", parts[0], parts[1] + 10);
            let behind = calculate_versions_behind(&newer);
            // saturating_sub returns 0 for negative result
            assert_eq!(behind, 0);
        }
    }

    #[test]
    fn test_calculate_versions_behind_invalid_version() {
        let behind = calculate_versions_behind("invalid");
        assert_eq!(behind, 0);
    }

    #[test]
    fn test_calculate_versions_behind_partial_version() {
        let behind = calculate_versions_behind("1");
        assert_eq!(behind, 0);
    }

    #[test]
    fn test_calculate_versions_behind_empty_string() {
        let behind = calculate_versions_behind("");
        assert_eq!(behind, 0);
    }

    // check_version_currency Tests

    #[test]
    fn test_check_version_currency_current() {
        let check = check_version_currency(PMAT_VERSION);
        assert_eq!(check.status, CheckStatus::Pass);
        assert_eq!(check.severity, Severity::Info);
        assert!(check.message.contains("latest"));
    }

    #[test]
    fn test_check_version_currency_slightly_behind() {
        let parts: Vec<u32> = PMAT_VERSION
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 2 && parts[1] >= 3 {
            let old = format!("{}.{}.0", parts[0], parts[1] - 3);
            let check = check_version_currency(&old);
            assert_eq!(check.status, CheckStatus::Warn);
            assert_eq!(check.severity, Severity::Warning);
        }
    }

    #[test]
    fn test_check_version_currency_very_behind() {
        let parts: Vec<u32> = PMAT_VERSION
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 2 && parts[1] > 10 {
            let old = format!("{}.{}.0", parts[0], parts[1] - 10);
            let check = check_version_currency(&old);
            assert_eq!(check.status, CheckStatus::Fail);
            assert_eq!(check.severity, Severity::Error);
        }
    }

    // check_config_files Tests

    #[test]
    fn test_check_config_files_none_present() {
        let temp = create_temp_project();
        let check = check_config_files(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("Missing"));
    }

    #[test]
    fn test_check_config_files_pmat_only() {
        let temp = create_pmat_project(PMAT_VERSION);
        let check = check_config_files(temp.path());
        // Only .pmat/project.toml present, missing .pmat-metrics.toml
        assert_eq!(check.status, CheckStatus::Warn);
    }

    #[test]
    fn test_check_config_files_all_present() {
        let temp = create_project_with_metrics(PMAT_VERSION);
        let check = check_config_files(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("All required"));
    }

    // check_hooks_installed Tests

    #[test]
    fn test_check_hooks_not_installed() {
        let temp = create_temp_project();
        let check = check_hooks_installed(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("No pre-commit"));
    }

    #[test]
    fn test_check_hooks_non_pmat_hook() {
        let temp = create_git_repo();
        let hook_content = "#!/bin/sh\necho 'some other hook'";
        fs::write(
            temp.path().join(".git").join("hooks").join("pre-commit"),
            hook_content,
        )
        .expect("Failed to write hook");

        let check = check_hooks_installed(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("may not be PMAT"));
    }

    #[test]
    fn test_check_hooks_pmat_hook_installed() {
        let temp = create_git_repo();
        let hook_content = "#!/bin/sh\n# PMAT hook\npmat check";
        fs::write(
            temp.path().join(".git").join("hooks").join("pre-commit"),
            hook_content,
        )
        .expect("Failed to write hook");

        let check = check_hooks_installed(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("PMAT hooks installed"));
    }

    #[test]
    fn test_check_hooks_pmat_lowercase() {
        let temp = create_git_repo();
        let hook_content = "#!/bin/sh\npmat validate";
        fs::write(
            temp.path().join(".git").join("hooks").join("pre-commit"),
            hook_content,
        )
        .expect("Failed to write hook");

        let check = check_hooks_installed(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    // check_quality_thresholds Tests

    #[test]
    fn test_check_quality_thresholds_missing() {
        let temp = create_temp_project();
        let check = check_quality_thresholds(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("No .pmat-metrics.toml"));
    }

    #[test]
    fn test_check_quality_thresholds_present() {
        let temp = create_project_with_metrics(PMAT_VERSION);
        let check = check_quality_thresholds(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("configured"));
    }

    // check_deprecated_features Tests

    #[test]
    fn test_check_deprecated_features_none() {
        let temp = create_temp_project();
        let check = check_deprecated_features(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("No deprecated"));
    }

    // check_compute_brick Tests

    #[test]
    fn test_check_compute_brick_not_applicable() {
        let temp = create_temp_project();
        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("Not a ComputeBrick"));
    }
