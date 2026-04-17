// Tests for work_contract_lint_config.rs (DBC spec §13.6, §13.7, §14.6)

#[test]
fn test_lint_config_default() {
    let config = LintConfig::default();
    assert_eq!(config.min_score, 0.0);
    assert!(!config.strict);
    assert!(config.rules.is_empty());
    assert!(config.suppress.is_empty());
    assert!(config.trend.enabled);
    assert_eq!(config.trend.retention_days, 90);
    assert!((config.trend.drift_threshold - 0.05).abs() < f64::EPSILON);
}

#[test]
fn test_lint_config_parse_basic() {
    let toml = r#"
min_score = 0.60
strict = true
"#;
    let config = LintConfig::parse(toml).unwrap();
    assert!((config.min_score - 0.60).abs() < f64::EPSILON);
    assert!(config.strict);
}

#[test]
fn test_lint_config_parse_rule_overrides() {
    let toml = r#"
[lint.rules]
DBC-AUD-003 = "error"
DBC-SCR-002 = "off"
"#;
    let config = LintConfig::parse(toml).unwrap();
    assert_eq!(config.rules.get("DBC-AUD-003").unwrap(), "error");
    assert_eq!(config.rules.get("DBC-SCR-002").unwrap(), "off");
}

#[test]
fn test_lint_config_parse_trend() {
    let toml = r#"
[lint.trend]
enabled = false
retention_days = 30
drift_threshold = 0.10
"#;
    let config = LintConfig::parse(toml).unwrap();
    assert!(!config.trend.enabled);
    assert_eq!(config.trend.retention_days, 30);
    assert!((config.trend.drift_threshold - 0.10).abs() < f64::EPSILON);
}

#[test]
fn test_effective_severity_default() {
    let config = LintConfig::default();
    assert_eq!(
        config.effective_severity("DBC-VAL-001", LintSeverity::Warning),
        Some(LintSeverity::Warning)
    );
}

#[test]
fn test_effective_severity_strict_mode() {
    let config = LintConfig {
        strict: true,
        ..Default::default()
    };
    // Warnings promoted to errors in strict mode
    assert_eq!(
        config.effective_severity("DBC-VAL-001", LintSeverity::Warning),
        Some(LintSeverity::Error)
    );
    // Errors stay as errors
    assert_eq!(
        config.effective_severity("DBC-VAL-002", LintSeverity::Error),
        Some(LintSeverity::Error)
    );
    // Info stays as info (strict only promotes warnings)
    assert_eq!(
        config.effective_severity("DBC-AUD-003", LintSeverity::Info),
        Some(LintSeverity::Info)
    );
}

#[test]
fn test_effective_severity_override() {
    let mut config = LintConfig::default();
    config
        .rules
        .insert("DBC-AUD-003".to_string(), "error".to_string());
    assert_eq!(
        config.effective_severity("DBC-AUD-003", LintSeverity::Info),
        Some(LintSeverity::Error)
    );
}

#[test]
fn test_effective_severity_off() {
    let mut config = LintConfig::default();
    config
        .rules
        .insert("DBC-SCR-002".to_string(), "off".to_string());
    assert_eq!(
        config.effective_severity("DBC-SCR-002", LintSeverity::Warning),
        None
    );
}

#[test]
fn test_effective_severity_suppressed() {
    let mut config = LintConfig::default();
    config.suppress.push("DBC-AUD-002".to_string());
    assert_eq!(
        config.effective_severity("DBC-AUD-002", LintSeverity::Info),
        None
    );
}

#[test]
fn test_is_suppressed() {
    let mut config = LintConfig::default();
    config.suppress.push("DBC-AUD-002".to_string());
    config
        .rules
        .insert("DBC-SCR-002".to_string(), "off".to_string());

    assert!(config.is_suppressed("DBC-AUD-002"));
    assert!(config.is_suppressed("DBC-SCR-002"));
    assert!(!config.is_suppressed("DBC-VAL-001"));
}

#[test]
fn test_apply_lint_config_suppress() {
    let report = LintReport {
        findings: vec![
            LintFinding {
                rule_id: "DBC-VAL-001".to_string(),
                severity: LintSeverity::Warning,
                message: "test".to_string(),
                clause_id: None,
            },
            LintFinding {
                rule_id: "DBC-AUD-003".to_string(),
                severity: LintSeverity::Info,
                message: "test".to_string(),
                clause_id: None,
            },
        ],
        passed: true,
        error_count: 0,
        warning_count: 1,
        info_count: 1,
    };

    let mut config = LintConfig::default();
    config.suppress.push("DBC-AUD-003".to_string());

    let filtered = apply_lint_config(&report, &config);
    assert_eq!(filtered.findings.len(), 1);
    assert_eq!(filtered.findings[0].rule_id, "DBC-VAL-001");
}

#[test]
fn test_apply_lint_config_strict_promotes_warnings() {
    let report = LintReport {
        findings: vec![LintFinding {
            rule_id: "DBC-VAL-001".to_string(),
            severity: LintSeverity::Warning,
            message: "test".to_string(),
            clause_id: None,
        }],
        passed: true,
        error_count: 0,
        warning_count: 1,
        info_count: 0,
    };

    let config = LintConfig {
        strict: true,
        ..Default::default()
    };

    let filtered = apply_lint_config(&report, &config);
    assert_eq!(filtered.findings[0].severity, LintSeverity::Error);
    assert_eq!(filtered.error_count, 1);
    assert!(!filtered.passed); // Now fails because warning became error
}

#[test]
fn test_apply_lint_config_findings_sorted_by_severity() {
    let report = LintReport {
        findings: vec![
            LintFinding {
                rule_id: "DBC-AUD-003".to_string(),
                severity: LintSeverity::Info,
                message: "info".to_string(),
                clause_id: None,
            },
            LintFinding {
                rule_id: "DBC-VAL-002".to_string(),
                severity: LintSeverity::Error,
                message: "error".to_string(),
                clause_id: None,
            },
            LintFinding {
                rule_id: "DBC-VAL-001".to_string(),
                severity: LintSeverity::Warning,
                message: "warning".to_string(),
                clause_id: None,
            },
        ],
        passed: false,
        error_count: 1,
        warning_count: 1,
        info_count: 1,
    };

    let config = LintConfig::default();
    let filtered = apply_lint_config(&report, &config);

    assert_eq!(filtered.findings[0].severity, LintSeverity::Error);
    assert_eq!(filtered.findings[1].severity, LintSeverity::Warning);
    assert_eq!(filtered.findings[2].severity, LintSeverity::Info);
}

#[test]
fn test_lint_config_load_missing_file() {
    let tmp = tempfile::tempdir().unwrap();
    let config = LintConfig::load(tmp.path());
    assert_eq!(config.min_score, 0.0);
    assert!(!config.strict);
}

#[test]
fn test_changed_contracts_since_empty_repo() {
    let tmp = tempfile::tempdir().unwrap();
    let result = changed_contracts_since(tmp.path(), "HEAD~1");
    assert!(result.is_empty());
}

#[test]
fn test_codebase_score_no_contracts() {
    let tmp = tempfile::tempdir().unwrap();
    let score = compute_codebase_score(tmp.path());
    assert_eq!(score.contract_count, 0);
    assert_eq!(score.composite, 0.0);
    assert_eq!(score.grade, ScoreGrade::F);
}

#[test]
fn test_codebase_score_with_contract() {
    let tmp = tempfile::tempdir().unwrap();
    let work_dir = tmp.path().join(".pmat-work").join("TEST-001");
    std::fs::create_dir_all(&work_dir).unwrap();

    let contract = WorkContract::new("TEST-001".to_string(), "abc".to_string());
    let contract_json = serde_json::to_string_pretty(&contract).unwrap();
    std::fs::write(work_dir.join("contract.json"), contract_json).unwrap();

    let score = compute_codebase_score(tmp.path());
    assert_eq!(score.contract_count, 1);
    assert!(score.mean_score > 0.0);
    assert!(score.composite > 0.0);
}

#[test]
fn test_codebase_score_serialization() {
    let score = CodebaseScore {
        contract_count: 3,
        contract_coverage: 0.67,
        mean_score: 0.55,
        min_score: 0.30,
        max_score: 0.80,
        mean_drift: 0.15,
        lint_pass_rate: 1.0,
        composite: 0.60,
        grade: ScoreGrade::C,
    };
    let json = serde_json::to_string(&score).unwrap();
    let deserialized: CodebaseScore = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.contract_count, 3);
    assert_eq!(deserialized.grade, ScoreGrade::C);
}

#[test]
fn test_lint_config_override_to_info() {
    let mut config = LintConfig::default();
    config
        .rules
        .insert("DBC-VAL-002".to_string(), "info".to_string());

    // Error downgraded to info
    assert_eq!(
        config.effective_severity("DBC-VAL-002", LintSeverity::Error),
        Some(LintSeverity::Info)
    );
}

#[test]
fn test_lint_trend_config_default() {
    let config = LintTrendConfig::default();
    assert!(config.enabled);
    assert_eq!(config.retention_days, 90);
    assert!((config.drift_threshold - 0.05).abs() < f64::EPSILON);
}
