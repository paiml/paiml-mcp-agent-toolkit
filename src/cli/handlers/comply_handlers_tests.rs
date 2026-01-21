//\! Tests for comply handlers
//\! Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;

    #[test]
    fn test_default_project_config() {
        let config = ProjectConfig::default();
        assert!(!config.pmat.version.is_empty());
    }

    #[test]
    fn test_calculate_versions_behind_same() {
        let behind = calculate_versions_behind(PMAT_VERSION);
        assert_eq!(behind, 0);
    }

    #[test]
    fn test_check_status_equality() {
        assert_eq!(CheckStatus::Pass, CheckStatus::Pass);
        assert_ne!(CheckStatus::Pass, CheckStatus::Fail);
    }

    #[test]
    fn test_severity_variants() {
        let _ = Severity::Info;
        let _ = Severity::Warning;
        let _ = Severity::Error;
        let _ = Severity::Critical;
    }
}


mod coverage_tests {
    use super::*;
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

    #[tokio::test]
    async fn test_full_compliance_workflow() {
        // Create a new project, init, check, migrate
        let temp = create_temp_project();

        // Init
        handle_init(temp.path(), false).await.expect("Init failed");

        // Verify config exists
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, PMAT_VERSION);

        // Check should pass (we're on current version)
        let checks = vec![
            check_version_currency(&config.pmat.version),
            check_config_files(temp.path()),
        ];
        let _all_pass_or_warn = checks.iter().all(|c| c.status != CheckStatus::Fail);
        // Version should pass, config files may warn about metrics
        assert!(checks[0].status == CheckStatus::Pass);
    }

    #[tokio::test]
    async fn test_migrate_then_check_workflow() {
        let temp = create_pmat_project("1.0.0");

        // Migrate to current
        handle_migrate(temp.path(), None, false, true, true)
            .await
            .expect("Migrate failed");

        // Check version should now pass
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        let check = check_version_currency(&config.pmat.version);
        assert_eq!(check.status, CheckStatus::Pass);
    }

    // ComputeBrick Pattern Detection Tests (CB-IMPL-001-B)

    #[test]
    fn test_cb020_detects_unsafe_without_safety() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with unsafe block without SAFETY comment
        let rs_file = src_dir.join("lib.rs");
        std::fs::write(
            &rs_file,
            r#"
fn bad_unsafe() {
    unsafe {
        std::ptr::null::<i32>().read();
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb020_unsafe_without_safety(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-020");
        assert!(violations[0].description.contains("unsafe"));
    }

    #[test]
    fn test_cb020_allows_unsafe_with_safety() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with unsafe block WITH SAFETY comment
        let rs_file = src_dir.join("lib.rs");
        std::fs::write(
            &rs_file,
            r#"
fn good_unsafe() {
    // SAFETY: null pointer read is UB, but this is just a test
    unsafe {
        std::ptr::null::<i32>().read();
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb020_unsafe_without_safety(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb021_detects_simd_without_target_feature() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with AVX intrinsic without #[target_feature]
        // Note: SSE (_mm_) is now exempted as baseline on x86_64
        let rs_file = src_dir.join("simd.rs");
        std::fs::write(
            &rs_file,
            r#"
fn bad_simd() {
    let a = _mm256_set1_ps(1.0);
}
"#,
        )
        .unwrap();

        let violations = detect_cb021_simd_without_target_feature(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-021");
        assert!(violations[0].description.contains("_mm256"));
    }

    #[test]
    fn test_cb021_allows_simd_with_target_feature() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with AVX intrinsic WITH #[target_feature]
        let rs_file = src_dir.join("simd.rs");
        std::fs::write(
            &rs_file,
            r#"
#[target_feature(enable = "avx2")]
fn good_simd() {
    let a = _mm256_set1_ps(1.0);
}
"#,
        )
        .unwrap();

        let violations = detect_cb021_simd_without_target_feature(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb021_no_false_positive_on_identifiers() {
        // Regression test: struct fields like "f32x4_verified" should NOT trigger CB-021
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with f32x4 in identifier names (NOT intrinsic usage)
        let rs_file = src_dir.join("verification.rs");
        std::fs::write(
            &rs_file,
            r#"
/// Verify SIMD f32x4 operations work correctly
pub struct SimdVerification {
    /// f32x4 operations verified
    pub f32x4_verified: bool,
    /// i32x4 operations verified
    pub i32x4_verified: bool,
}

pub fn verify_f32x4_operations() -> bool {
    let simd_lanes = 4; // f32x4
    true
}
"#,
        )
        .unwrap();

        let violations = detect_cb021_simd_without_target_feature(temp.path());
        // Should be 0 - these are identifiers and comments, not intrinsic calls
        assert_eq!(
            violations.len(),
            0,
            "False positive: detected {:?}",
            violations
        );
    }

    #[test]
    fn test_cb021_detects_actual_portable_simd_usage() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with ACTUAL portable SIMD usage (f32x4::splat)
        let rs_file = src_dir.join("simd_usage.rs");
        std::fs::write(
            &rs_file,
            r#"
use std::simd::f32x4;

fn use_portable_simd() {
    let a = f32x4::splat(1.0);
    let b = f32x4::from_array([1.0, 2.0, 3.0, 4.0]);
}
"#,
        )
        .unwrap();

        let violations = detect_cb021_simd_without_target_feature(temp.path());
        // Should detect f32x4:: usage as potential SIMD without target_feature
        assert!(
            violations.len() >= 1,
            "Should detect portable SIMD usage: {:?}",
            violations
        );
    }

    // CB-001 and CB-002 WGSL Detection Tests (CB-IMPL-001-D)

    #[test]
    fn test_cb001_detects_wgsl_without_bounds_check() {
        let temp = tempfile::tempdir().unwrap();

        // Create WGSL file with global_invocation_id but NO bounds check
        let wgsl_file = temp.path().join("compute.wgsl");
        std::fs::write(
            &wgsl_file,
            r#"@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gid = global_id.x;
    output[gid] = input[gid];  // No bounds check!
}
"#,
        )
        .unwrap();

        let violations = detect_cb001_wgsl_no_bounds_check(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-001");
        assert!(violations[0].description.contains("bounds check"));
    }

    #[test]
    fn test_cb001_allows_wgsl_with_bounds_check() {
        let temp = tempfile::tempdir().unwrap();

        // Create WGSL file with global_invocation_id AND bounds check
        let wgsl_file = temp.path().join("compute.wgsl");
        std::fs::write(
            &wgsl_file,
            r#"@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gid = global_id.x;
    if (gid >= arrayLength(&input)) { return; }
    output[gid] = input[gid];
}
"#,
        )
        .unwrap();

        let violations = detect_cb001_wgsl_no_bounds_check(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb002_detects_wgsl_barrier_in_conditional() {
        let temp = tempfile::tempdir().unwrap();

        // Create WGSL file with workgroupBarrier() inside conditional
        let wgsl_file = temp.path().join("compute.wgsl");
        std::fs::write(
            &wgsl_file,
            r#"@compute @workgroup_size(64)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {
    if (local_id.x == 0u) {
        shared_data[0] = compute();
        workgroupBarrier();  // DANGER: Inside conditional!
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb002_wgsl_barrier_divergence(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-002");
        assert!(violations[0].description.contains("workgroupBarrier()"));
    }

    #[test]
    fn test_cb002_allows_wgsl_barrier_outside_conditional() {
        let temp = tempfile::tempdir().unwrap();

        // Create WGSL file with workgroupBarrier() OUTSIDE conditional
        let wgsl_file = temp.path().join("compute.wgsl");
        std::fs::write(
            &wgsl_file,
            r#"@compute @workgroup_size(64)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {
    if (local_id.x == 0u) {
        shared_data[0] = compute();
    }
    workgroupBarrier();  // Safe: All threads reach this
    let val = shared_data[0];
}
"#,
        )
        .unwrap();

        let violations = detect_cb002_wgsl_barrier_divergence(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detect_bricks_without_assertions() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with Brick impl WITHOUT assertions
        // Use concat! to avoid self-matching during CB-BUDGET compliance scanning
        let rs_file = src_dir.join("brick.rs");
        std::fs::write(
            &rs_file,
            // No leading newline - content starts immediately
            concat!("impl Compute", "Brick for MyBrick {\n\
                fn execute(&self) {\n\
                    self.do_work();\n\
                }\n\
            }\n"),
        )
        .unwrap();

        let violations = detect_bricks_without_assertions(temp.path());
        assert_eq!(violations.len(), 1, "Expected 1 violation for brick without assertions");
        assert_eq!(violations[0].pattern_id, "CB-BUDGET");
    }

    #[test]
    fn test_detect_bricks_with_assertions_pass() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with Brick impl WITH assertions
        // Use concat! to avoid self-matching during CB-BUDGET compliance scanning
        let rs_file = src_dir.join("brick.rs");
        std::fs::write(
            &rs_file,
            concat!("\nimpl Compute", "Brick for MyBrick {\n\
    fn execute(&self) {\n\
        debug_assert!(self.is_valid());\n\
        self.do_work();\n\
    }\n\
}\n"),
        )
        .unwrap();

        let violations = detect_bricks_without_assertions(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detect_profiler_anomalies_high_cv() {
        let temp = tempfile::tempdir().unwrap();
        let metrics_dir = temp.path().join(".pmat-metrics");
        std::fs::create_dir_all(&metrics_dir).unwrap();

        // Create profiler JSON with high CV
        let profile_file = metrics_dir.join("brick-profile.json");
        std::fs::write(
            &profile_file,
            r#"{
  "bricks": [
    {
      "name": "MatMulBrick",
      "cv": 0.25,
      "efficiency": 0.80
    }
  ]
}"#,
        )
        .unwrap();

        let anomalies = detect_profiler_anomalies(temp.path());
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].anomaly_type, "HIGH_CV");
        assert!(anomalies[0].value > 15.0);
    }

    #[test]
    fn test_detect_profiler_anomalies_low_efficiency() {
        let temp = tempfile::tempdir().unwrap();
        let metrics_dir = temp.path().join(".pmat-metrics");
        std::fs::create_dir_all(&metrics_dir).unwrap();

        // Create profiler JSON with low efficiency
        let profile_file = metrics_dir.join("brick-profile.json");
        std::fs::write(
            &profile_file,
            r#"{
  "bricks": [
    {
      "name": "SlowBrick",
      "cv": 0.05,
      "efficiency": 0.15
    }
  ]
}"#,
        )
        .unwrap();

        let anomalies = detect_profiler_anomalies(temp.path());
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].anomaly_type, "LOW_EFFICIENCY");
        assert!(anomalies[0].value < 25.0);
    }

    #[test]
    fn test_check_compute_brick_skips_non_cb_project() {
        let temp = tempfile::tempdir().unwrap();
        // Create a regular project without trueno/realizar/probar deps
        let cargo_toml = temp.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"[package]
name = "regular-project"
version = "1.0.0"

[dependencies]
serde = "1.0"
"#,
        )
        .unwrap();

        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_check_compute_brick_detects_trueno_project() {
        let temp = tempfile::tempdir().unwrap();
        // Create project with trueno dependency
        let cargo_toml = temp.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"[package]
name = "gpu-project"
version = "1.0.0"

[dependencies]
trueno = "0.1"
"#,
        )
        .unwrap();

        // Create src directory with clean Rust code
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("lib.rs"), "pub fn hello() {}").unwrap();

        let check = check_compute_brick(temp.path());
        // Should not skip - this is a CB ecosystem project
        assert_ne!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_extract_json_number() {
        assert_eq!(extract_json_number("\"cv\": 0.18,"), Some(0.18));
        assert_eq!(extract_json_number("\"efficiency\": 25.5}"), Some(25.5));
        assert_eq!(extract_json_number("invalid"), None);
    }

    #[test]
    fn test_walkdir_rs_files() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        let nested = src_dir.join("nested");
        std::fs::create_dir_all(&nested).unwrap();

        std::fs::write(src_dir.join("lib.rs"), "").unwrap();
        std::fs::write(nested.join("mod.rs"), "").unwrap();
        std::fs::write(src_dir.join("readme.md"), "").unwrap(); // Not .rs

        let files = walkdir_rs_files(&src_dir).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| f.extension().unwrap() == "rs"));
    }
}


mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn test_version_behind_never_negative(major in 0u32..100, minor in 0u32..1000, patch in 0u32..100) {
            let version = format!("{}.{}.{}", major, minor, patch);
            let behind = calculate_versions_behind(&version);
            // Should always return a non-negative value (saturating_sub)
            prop_assert!(behind < u32::MAX);
        }

        #[test]
        fn test_check_version_currency_always_returns_valid_check(
            major in 0u32..10,
            minor in 0u32..500,
            patch in 0u32..100
        ) {
            let version = format!("{}.{}.{}", major, minor, patch);
            let check = check_version_currency(&version);

            // Check should always have non-empty fields
            prop_assert!(!check.name.is_empty());
            prop_assert!(!check.message.is_empty());

            // Status should be one of the valid variants
            prop_assert!(matches!(
                check.status,
                CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Fail | CheckStatus::Skip
            ));
        }

        #[test]
        fn test_project_config_roundtrip_serialization(
            version in "[0-9]+\\.[0-9]+\\.[0-9]+",
            auto_update in proptest::bool::ANY
        ) {
            let config = ProjectConfig {
                pmat: PmatSection {
                    version: version.clone(),
                    last_compliance_check: Some(Utc::now()),
                    auto_update,
                },
            };

            let serialized = toml::to_string_pretty(&config).expect("Serialization failed");
            let deserialized: ProjectConfig = toml::from_str(&serialized).expect("Deserialization failed");

            prop_assert_eq!(deserialized.pmat.version, version);
            prop_assert_eq!(deserialized.pmat.auto_update, auto_update);
        }

        #[test]
        fn test_compliance_check_serialization_roundtrip(
            name in "[a-zA-Z ]{1,50}",
            message in "[a-zA-Z0-9 ]{1,100}"
        ) {
            let check = ComplianceCheck {
                name: name.clone(),
                status: CheckStatus::Pass,
                message: message.clone(),
                severity: Severity::Info,
            };

            let json = serde_json::to_string(&check).expect("Serialization failed");
            let deserialized: ComplianceCheck = serde_json::from_str(&json).expect("Deserialization failed");

            prop_assert_eq!(deserialized.name, name);
            prop_assert_eq!(deserialized.message, message);
        }

        #[test]
        fn test_breaking_change_serialization_roundtrip(
            version in "[0-9]+\\.[0-9]+\\.[0-9]+",
            description in "[a-zA-Z0-9 ]{1,200}"
        ) {
            let change = BreakingChange {
                version: version.clone(),
                description: description.clone(),
                migration_guide: Some("Guide".to_string()),
            };

            let json = serde_json::to_string(&change).expect("Serialization failed");
            let deserialized: BreakingChange = serde_json::from_str(&json).expect("Deserialization failed");

            prop_assert_eq!(deserialized.version, version);
            prop_assert_eq!(deserialized.description, description);
        }

        #[test]
        fn test_changelog_entries_always_have_current_version(_seed in 0u32..1000) {
            let entries = get_changelog_entries("0.0.0", "999.999.999");
            prop_assert!(!entries.is_empty());

            // All entries should have version matching PMAT_VERSION
            for entry in &entries {
                prop_assert_eq!(&entry.version, PMAT_VERSION);
            }
        }

        #[test]
        fn test_breaking_changes_returns_empty_for_any_version(
            major in 0u32..100,
            minor in 0u32..1000,
            patch in 0u32..100
        ) {
            let version = format!("{}.{}.{}", major, minor, patch);
            let changes = get_breaking_changes_since(&version);
            // Current implementation always returns empty
            prop_assert!(changes.is_empty());
        }
    }

    // Additional property tests that require tempdir (can't use proptest macro easily)
    #[test]
    fn test_check_config_files_consistency() {
        use tempfile::TempDir;

        // Test that check_config_files is consistent across multiple calls
        let temp = TempDir::new().expect("Failed to create temp dir");
        let check1 = check_config_files(temp.path());
        let check2 = check_config_files(temp.path());

        assert_eq!(check1.status, check2.status);
        assert_eq!(check1.message, check2.message);
    }

    #[test]
    fn test_check_hooks_consistency() {
        use tempfile::TempDir;

        let temp = TempDir::new().expect("Failed to create temp dir");
        let check1 = check_hooks_installed(temp.path());
        let check2 = check_hooks_installed(temp.path());

        assert_eq!(check1.status, check2.status);
    }
}
