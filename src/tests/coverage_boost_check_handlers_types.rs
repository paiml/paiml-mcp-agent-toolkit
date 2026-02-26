// =============================================================================
// ProjectConfig and PmatSection tests
// =============================================================================

#[test]
fn test_project_config_default() {
    let config = ProjectConfig::default();
    assert_eq!(config.pmat.version, PMAT_VERSION);
    assert!(config.pmat.last_compliance_check.is_some());
    assert!(!config.pmat.auto_update);
}

#[test]
fn test_pmat_section_serialization() {
    let section = PmatSection {
        version: "1.0.0".to_string(),
        last_compliance_check: Some(Utc::now()),
        auto_update: true,
    };

    let serialized = toml::to_string(&section).unwrap();
    assert!(serialized.contains("version = \"1.0.0\""));
    assert!(serialized.contains("auto_update = true"));
}

#[test]
fn test_project_config_roundtrip() {
    let config = ProjectConfig::default();
    let serialized = toml::to_string_pretty(&config).unwrap();
    let deserialized: ProjectConfig = toml::from_str(&serialized).unwrap();

    assert_eq!(config.pmat.version, deserialized.pmat.version);
    assert_eq!(config.pmat.auto_update, deserialized.pmat.auto_update);
}

// =============================================================================
// CheckStatus tests
// =============================================================================

#[test]
fn test_check_status_variants() {
    assert_eq!(CheckStatus::Pass, CheckStatus::Pass);
    assert_eq!(CheckStatus::Warn, CheckStatus::Warn);
    assert_eq!(CheckStatus::Fail, CheckStatus::Fail);
    assert_eq!(CheckStatus::Skip, CheckStatus::Skip);
}

#[test]
fn test_check_status_serialization() {
    let status = CheckStatus::Pass;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"Pass\"");

    let status = CheckStatus::Fail;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"Fail\"");
}

#[test]
fn test_check_status_inequality() {
    assert_ne!(CheckStatus::Pass, CheckStatus::Fail);
    assert_ne!(CheckStatus::Warn, CheckStatus::Skip);
}

// =============================================================================
// Severity tests
// =============================================================================

#[test]
fn test_severity_variants() {
    assert_eq!(Severity::Info, Severity::Info);
    assert_eq!(Severity::Warning, Severity::Warning);
    assert_eq!(Severity::Error, Severity::Error);
    assert_eq!(Severity::Critical, Severity::Critical);
}

#[test]
fn test_severity_from_config_severity() {
    let severity: Severity = CheckSeverity::Info.into();
    assert_eq!(severity, Severity::Info);

    let severity: Severity = CheckSeverity::Warning.into();
    assert_eq!(severity, Severity::Warning);

    let severity: Severity = CheckSeverity::Error.into();
    assert_eq!(severity, Severity::Error);

    let severity: Severity = CheckSeverity::Critical.into();
    assert_eq!(severity, Severity::Critical);
}

#[test]
fn test_severity_serialization() {
    let severity = Severity::Critical;
    let json = serde_json::to_string(&severity).unwrap();
    assert_eq!(json, "\"Critical\"");

    let deserialized: Severity = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, Severity::Critical);
}

// =============================================================================
// ComplianceCheck tests
// =============================================================================

#[test]
fn test_compliance_check_creation() {
    let check = ComplianceCheck {
        name: "Test Check".to_string(),
        status: CheckStatus::Pass,
        message: "All good".to_string(),
        severity: Severity::Info,
    };

    assert_eq!(check.name, "Test Check");
    assert_eq!(check.status, CheckStatus::Pass);
    assert_eq!(check.message, "All good");
    assert_eq!(check.severity, Severity::Info);
}

#[test]
fn test_compliance_check_serialization() {
    let check = ComplianceCheck {
        name: "Version Check".to_string(),
        status: CheckStatus::Warn,
        message: "Version behind".to_string(),
        severity: Severity::Warning,
    };

    let json = serde_json::to_string(&check).unwrap();
    assert!(json.contains("\"name\":\"Version Check\""));
    assert!(json.contains("\"status\":\"Warn\""));
    assert!(json.contains("\"severity\":\"Warning\""));
}

#[test]
fn test_compliance_check_clone() {
    let check = ComplianceCheck {
        name: "Clone Test".to_string(),
        status: CheckStatus::Fail,
        message: "Failed".to_string(),
        severity: Severity::Error,
    };

    let cloned = check.clone();
    assert_eq!(check.name, cloned.name);
    assert_eq!(check.status, cloned.status);
    assert_eq!(check.severity, cloned.severity);
}

// =============================================================================
// BreakingChange tests
// =============================================================================

#[test]
fn test_breaking_change_creation() {
    let change = BreakingChange {
        version: "2.0.0".to_string(),
        description: "Major API change".to_string(),
        migration_guide: Some("See docs".to_string()),
    };

    assert_eq!(change.version, "2.0.0");
    assert_eq!(change.description, "Major API change");
    assert!(change.migration_guide.is_some());
}

#[test]
fn test_breaking_change_without_guide() {
    let change = BreakingChange {
        version: "1.5.0".to_string(),
        description: "Minor breaking change".to_string(),
        migration_guide: None,
    };

    assert!(change.migration_guide.is_none());
}

#[test]
fn test_breaking_change_serialization() {
    let change = BreakingChange {
        version: "3.0.0".to_string(),
        description: "Complete rewrite".to_string(),
        migration_guide: Some("Start fresh".to_string()),
    };

    let json = serde_json::to_string(&change).unwrap();
    assert!(json.contains("\"version\":\"3.0.0\""));
    assert!(json.contains("\"migration_guide\":\"Start fresh\""));
}

// =============================================================================
// ComplianceReport tests
// =============================================================================

#[test]
fn test_compliance_report_compliant() {
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

    assert!(report.is_compliant);
    assert_eq!(report.versions_behind, 0);
    assert!(report.breaking_changes.is_empty());
}

#[test]
fn test_compliance_report_not_compliant() {
    let report = ComplianceReport {
        project_version: "0.1.0".to_string(),
        current_version: PMAT_VERSION.to_string(),
        is_compliant: false,
        versions_behind: 10,
        checks: vec![ComplianceCheck {
            name: "Version Currency".to_string(),
            status: CheckStatus::Fail,
            message: "Too old".to_string(),
            severity: Severity::Error,
        }],
        breaking_changes: vec![BreakingChange {
            version: "1.0.0".to_string(),
            description: "Breaking change".to_string(),
            migration_guide: None,
        }],
        recommendations: vec!["Update now".to_string()],
        timestamp: Utc::now(),
    };

    assert!(!report.is_compliant);
    assert_eq!(report.versions_behind, 10);
    assert!(!report.breaking_changes.is_empty());
    assert!(!report.recommendations.is_empty());
}

#[test]
fn test_compliance_report_serialization() {
    let report = ComplianceReport {
        project_version: "1.0.0".to_string(),
        current_version: PMAT_VERSION.to_string(),
        is_compliant: true,
        versions_behind: 0,
        checks: vec![],
        breaking_changes: vec![],
        recommendations: vec![],
        timestamp: Utc::now(),
    };

    let json = serde_json::to_string_pretty(&report).unwrap();
    assert!(json.contains("\"project_version\":"));
    assert!(json.contains("\"is_compliant\": true"));
}
