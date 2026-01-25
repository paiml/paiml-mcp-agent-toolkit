// Coverage tests for comply handlers
// Extracted for file health compliance (CB-040)

use super::super::*;
use std::fs;
use tempfile::TempDir;

    // Test Fixture Helpers

    /// Create a temporary directory with basic PMAT structure
    fn create_temp_project() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    /// Create a project with .pmat directory and project.toml
    fn create_pmat_project(version: &str) -> TempDir {
        let temp = create_temp_project();
        let pmat_dir = temp.path().join(".pmat");
        fs::create_dir_all(&pmat_dir).expect("Failed to create .pmat dir");

        let config = format!(
            r#"[pmat]
version = "{}"
auto_update = false
"#,
            version
        );
        fs::write(pmat_dir.join("project.toml"), config).expect("Failed to write project.toml");
        temp
    }

    /// Create a project with .pmat-metrics.toml
    fn create_project_with_metrics(version: &str) -> TempDir {
        let temp = create_pmat_project(version);
        let metrics_content = r#"
[thresholds]
lint = 30000
test-fast = 300000
"#;
        fs::write(temp.path().join(".pmat-metrics.toml"), metrics_content)
            .expect("Failed to write metrics");
        temp
    }

    /// Create a git repository structure
    fn create_git_repo() -> TempDir {
        let temp = create_temp_project();
        let hooks_dir = temp.path().join(".git").join("hooks");
        fs::create_dir_all(&hooks_dir).expect("Failed to create .git/hooks");
        temp
    }

    /// Create a Rust project with Cargo.toml
    fn create_rust_project(with_msrv: bool, with_lock: bool) -> TempDir {
        let temp = create_temp_project();
        let cargo_content = if with_msrv {
            r#"[package]
name = "test"
version = "0.1.0"
rust-version = "1.75"
"#
        } else {
            r#"[package]
name = "test"
version = "0.1.0"
"#
        };
        fs::write(temp.path().join("Cargo.toml"), cargo_content)
            .expect("Failed to write Cargo.toml");
        if with_lock {
            fs::write(temp.path().join("Cargo.lock"), "# lock file")
                .expect("Failed to write Cargo.lock");
        }
        temp
    }

    // ProjectConfig Tests

    #[test]
    fn test_project_config_default_has_current_version() {
        let config = ProjectConfig::default();
        assert_eq!(config.pmat.version, PMAT_VERSION);
    }

    #[test]
    fn test_project_config_default_has_timestamp() {
        let config = ProjectConfig::default();
        assert!(config.pmat.last_compliance_check.is_some());
    }

    #[test]
    fn test_project_config_default_auto_update_is_false() {
        let config = ProjectConfig::default();
        assert!(!config.pmat.auto_update);
    }

    #[test]
    fn test_project_config_serialization() {
        let config = ProjectConfig::default();
        let serialized = toml::to_string_pretty(&config).expect("Serialization failed");
        assert!(serialized.contains("[pmat]"));
        assert!(serialized.contains("version"));
    }

    #[test]
    fn test_project_config_deserialization() {
        let toml_str = r#"
[pmat]
version = "1.0.0"
auto_update = true
"#;
        let config: ProjectConfig = toml::from_str(toml_str).expect("Deserialization failed");
        assert_eq!(config.pmat.version, "1.0.0");
        assert!(config.pmat.auto_update);
        assert!(config.pmat.last_compliance_check.is_none());
    }

    #[test]
    fn test_pmat_section_clone() {
        let section = PmatSection {
            version: "2.0.0".to_string(),
            last_compliance_check: Some(Utc::now()),
            auto_update: true,
        };
        let cloned = section.clone();
        assert_eq!(cloned.version, section.version);
        assert_eq!(cloned.auto_update, section.auto_update);
    }

    // ComplianceReport Tests

    #[test]
    fn test_compliance_report_serialization() {
        let report = ComplianceReport {
            project_version: "1.0.0".to_string(),
            current_version: "2.0.0".to_string(),
            is_compliant: true,
            versions_behind: 10,
            checks: vec![],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        let json = serde_json::to_string(&report).expect("JSON serialization failed");
        assert!(json.contains("project_version"));
        assert!(json.contains("is_compliant"));
    }

    #[test]
    fn test_compliance_report_with_checks() {
        let check = ComplianceCheck {
            name: "Test Check".to_string(),
            status: CheckStatus::Pass,
            message: "All good".to_string(),
            severity: Severity::Info,
        };
        let report = ComplianceReport {
            project_version: "1.0.0".to_string(),
            current_version: "1.0.0".to_string(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![check],
            breaking_changes: vec![],
            recommendations: vec!["Upgrade soon".to_string()],
            timestamp: Utc::now(),
        };
        assert_eq!(report.checks.len(), 1);
        assert_eq!(report.recommendations.len(), 1);
    }

    // ComplianceCheck Tests

    #[test]
    fn test_compliance_check_clone() {
        let check = ComplianceCheck {
            name: "Test".to_string(),
            status: CheckStatus::Warn,
            message: "Warning message".to_string(),
            severity: Severity::Warning,
        };
        let cloned = check.clone();
        assert_eq!(cloned.name, check.name);
        assert_eq!(cloned.status, check.status);
    }

    #[test]
    fn test_compliance_check_serialization() {
        let check = ComplianceCheck {
            name: "Version Check".to_string(),
            status: CheckStatus::Fail,
            message: "Outdated".to_string(),
            severity: Severity::Error,
        };
        let json = serde_json::to_string(&check).expect("Serialization failed");
        assert!(json.contains("Version Check"));
        assert!(json.contains("Fail"));
    }

    // CheckStatus Tests

    #[test]
    fn test_check_status_all_variants() {
        let pass = CheckStatus::Pass;
        let warn = CheckStatus::Warn;
        let fail = CheckStatus::Fail;
        let skip = CheckStatus::Skip;

        assert_eq!(pass, CheckStatus::Pass);
        assert_eq!(warn, CheckStatus::Warn);
        assert_eq!(fail, CheckStatus::Fail);
        assert_eq!(skip, CheckStatus::Skip);
    }

    #[test]
    fn test_check_status_inequality() {
        assert_ne!(CheckStatus::Pass, CheckStatus::Warn);
        assert_ne!(CheckStatus::Warn, CheckStatus::Fail);
        assert_ne!(CheckStatus::Fail, CheckStatus::Skip);
        assert_ne!(CheckStatus::Skip, CheckStatus::Pass);
    }

    #[test]
    fn test_check_status_copy() {
        let status = CheckStatus::Pass;
        let copied = status;
        assert_eq!(copied, CheckStatus::Pass);
    }

    // Severity Tests

    #[test]
    fn test_severity_all_variants() {
        let info = Severity::Info;
        let warning = Severity::Warning;
        let error = Severity::Error;
        let critical = Severity::Critical;

        assert_eq!(info, Severity::Info);
        assert_eq!(warning, Severity::Warning);
        assert_eq!(error, Severity::Error);
        assert_eq!(critical, Severity::Critical);
    }

    #[test]
    fn test_severity_inequality() {
        assert_ne!(Severity::Info, Severity::Warning);
        assert_ne!(Severity::Warning, Severity::Error);
        assert_ne!(Severity::Error, Severity::Critical);
    }

    #[test]
    fn test_severity_serialization() {
        let severity = Severity::Critical;
        let json = serde_json::to_string(&severity).expect("Serialization failed");
        assert!(json.contains("Critical"));
    }

    // BreakingChange Tests

    #[test]
    fn test_breaking_change_with_migration_guide() {
        let change = BreakingChange {
            version: "2.0.0".to_string(),
            description: "API changed".to_string(),
            migration_guide: Some("Follow these steps...".to_string()),
        };
        assert_eq!(change.version, "2.0.0");
        assert!(change.migration_guide.is_some());
    }

    #[test]
    fn test_breaking_change_without_migration_guide() {
        let change = BreakingChange {
            version: "2.0.0".to_string(),
            description: "Removed feature X".to_string(),
            migration_guide: None,
        };
        assert!(change.migration_guide.is_none());
    }

    #[test]
    fn test_breaking_change_clone() {
        let change = BreakingChange {
            version: "1.5.0".to_string(),
            description: "Config format changed".to_string(),
            migration_guide: Some("Update your config".to_string()),
        };
        let cloned = change.clone();
        assert_eq!(cloned.version, change.version);
        assert_eq!(cloned.migration_guide, change.migration_guide);
    }

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

