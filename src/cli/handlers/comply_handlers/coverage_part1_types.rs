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
