// Tests for work_contract_lint.rs (DBC spec §13.3, §14.5)

#[test]
fn test_lint_severity_display() {
    assert_eq!(LintSeverity::Error.to_string(), "error");
    assert_eq!(LintSeverity::Warning.to_string(), "warning");
    assert_eq!(LintSeverity::Info.to_string(), "info");
}

#[test]
fn test_lint_v4_contract_passes() {
    let contract = WorkContract::new("test".to_string(), "abc".to_string());
    let tmp = tempfile::tempdir().unwrap();
    let report = lint_contract(&contract, tmp.path(), 0.0);

    // v4.0 contract should pass (VAL rules only apply to v5.0)
    assert!(report.passed);
    assert_eq!(report.error_count, 0);
}

#[test]
fn test_lint_val_004_empty_hypothesis() {
    let mut contract = WorkContract::new("test".to_string(), "abc".to_string());
    contract.claims.push(FalsifiableClaim {
        hypothesis: "   ".to_string(), // whitespace-only
        falsification_method: FalsificationMethod::LintPass,
        evidence_required: EvidenceType::BooleanCheck(false),
        result: None,
        override_info: None,
    });
    let tmp = tempfile::tempdir().unwrap();
    let report = lint_contract(&contract, tmp.path(), 0.0);

    let val004 = report
        .findings
        .iter()
        .find(|f| f.rule_id == "DBC-VAL-004");
    assert!(val004.is_some(), "DBC-VAL-004 should fire for empty hypothesis");
    assert_eq!(val004.unwrap().severity, LintSeverity::Error);
}

#[test]
fn test_lint_aud_003_unverified_claims() {
    let contract = WorkContract::new("test".to_string(), "abc".to_string());
    let tmp = tempfile::tempdir().unwrap();
    let report = lint_contract(&contract, tmp.path(), 0.0);

    let aud003 = report
        .findings
        .iter()
        .find(|f| f.rule_id == "DBC-AUD-003");
    assert!(aud003.is_some(), "DBC-AUD-003 should fire for unverified claims");
    assert_eq!(aud003.unwrap().severity, LintSeverity::Info);
    assert!(aud003.unwrap().message.contains("22 claim(s)"));
}

#[test]
fn test_lint_aud_003_all_verified() {
    let mut contract = WorkContract::new("test".to_string(), "abc".to_string());
    for claim in &mut contract.claims {
        claim.result = Some(FalsificationResult::passed("ok"));
    }
    let tmp = tempfile::tempdir().unwrap();
    let report = lint_contract(&contract, tmp.path(), 0.0);

    let aud003 = report
        .findings
        .iter()
        .find(|f| f.rule_id == "DBC-AUD-003");
    assert!(aud003.is_none(), "DBC-AUD-003 should not fire when all verified");
}

#[test]
fn test_lint_scr_001_score_below_threshold() {
    let mut contract = WorkContract::new("test".to_string(), "abc".to_string());
    contract.claims.clear(); // empty claims = low score
    let tmp = tempfile::tempdir().unwrap();
    let report = lint_contract(&contract, tmp.path(), 0.90);

    let scr001 = report
        .findings
        .iter()
        .find(|f| f.rule_id == "DBC-SCR-001");
    assert!(scr001.is_some(), "DBC-SCR-001 should fire when score < threshold");
    assert_eq!(scr001.unwrap().severity, LintSeverity::Error);
    assert!(!report.passed);
}

#[test]
fn test_lint_scr_001_score_above_threshold() {
    let contract = WorkContract::new("test".to_string(), "abc".to_string());
    let tmp = tempfile::tempdir().unwrap();
    let report = lint_contract(&contract, tmp.path(), 0.0);

    let scr001 = report
        .findings
        .iter()
        .find(|f| f.rule_id == "DBC-SCR-001");
    assert!(scr001.is_none(), "DBC-SCR-001 should not fire when threshold is 0");
}

#[test]
fn test_lint_drf_001_not_stale() {
    let contract = WorkContract::new("test".to_string(), "abc".to_string());
    let tmp = tempfile::tempdir().unwrap();
    let report = lint_contract(&contract, tmp.path(), 0.0);

    let drf001 = report
        .findings
        .iter()
        .find(|f| f.rule_id == "DBC-DRF-001");
    assert!(drf001.is_none(), "DBC-DRF-001 should not fire for fresh contract");
}

#[test]
fn test_lint_report_counts() {
    let mut contract = WorkContract::new("test".to_string(), "abc".to_string());
    // Add an empty hypothesis to trigger DBC-VAL-004 (error)
    contract.claims.push(FalsifiableClaim {
        hypothesis: "".to_string(),
        falsification_method: FalsificationMethod::LintPass,
        evidence_required: EvidenceType::BooleanCheck(false),
        result: None,
        override_info: None,
    });
    let tmp = tempfile::tempdir().unwrap();
    let report = lint_contract(&contract, tmp.path(), 0.0);

    assert!(report.error_count >= 1);
    assert!(!report.passed);
    assert_eq!(
        report.error_count + report.warning_count + report.info_count,
        report.findings.len()
    );
}

#[test]
fn test_lint_prv_001_no_violation_first_iteration() {
    let contract = WorkContract::new("test".to_string(), "abc".to_string());
    let tmp = tempfile::tempdir().unwrap();
    let report = lint_contract(&contract, tmp.path(), 0.0);

    let prv001 = report
        .findings
        .iter()
        .find(|f| f.rule_id == "DBC-PRV-001");
    assert!(prv001.is_none(), "DBC-PRV-001 should not fire on first iteration");
}

#[test]
fn test_lint_finding_has_rule_id() {
    let finding = LintFinding {
        rule_id: "DBC-VAL-001".to_string(),
        severity: LintSeverity::Warning,
        message: "test".to_string(),
        clause_id: None,
    };
    assert_eq!(finding.rule_id, "DBC-VAL-001");
    assert!(finding.clause_id.is_none());
}

#[test]
fn test_lint_finding_with_clause_id() {
    let finding = LintFinding {
        rule_id: "DBC-AUD-001".to_string(),
        severity: LintSeverity::Warning,
        message: "test".to_string(),
        clause_id: Some("ensure.coverage".to_string()),
    };
    assert_eq!(finding.clause_id.as_deref(), Some("ensure.coverage"));
}

#[test]
fn test_lint_report_serialization() {
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
    let json = serde_json::to_string(&report).unwrap();
    let deserialized: LintReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.findings.len(), 1);
    assert!(deserialized.passed);
}

// === Tests for new rules (§14.5) ===

#[test]
fn test_lint_scr_002_no_exclusions() {
    let contract = WorkContract::new("test".to_string(), "abc".to_string());
    let tmp = tempfile::tempdir().unwrap();
    let report = lint_contract(&contract, tmp.path(), 0.0);

    let scr002 = report.findings.iter().find(|f| f.rule_id == "DBC-SCR-002");
    assert!(scr002.is_none(), "DBC-SCR-002 should not fire without exclusions");
}

#[test]
fn test_lint_trd_001_no_trend_data() {
    let contract = WorkContract::new("test".to_string(), "abc".to_string());
    let tmp = tempfile::tempdir().unwrap();
    let report = lint_contract(&contract, tmp.path(), 0.0);

    let trd001 = report.findings.iter().find(|f| f.rule_id == "DBC-TRD-001");
    assert!(trd001.is_none(), "DBC-TRD-001 should not fire without trend data");
}

#[test]
fn test_lint_trd_002_no_rescue_data() {
    let contract = WorkContract::new("test".to_string(), "abc".to_string());
    let tmp = tempfile::tempdir().unwrap();
    let report = lint_contract(&contract, tmp.path(), 0.0);

    let trd002 = report.findings.iter().find(|f| f.rule_id == "DBC-TRD-002");
    assert!(trd002.is_none(), "DBC-TRD-002 should not fire without rescue data");
}

// === SARIF Output Tests (§13.4) ===

#[test]
fn test_sarif_output_structure() {
    let report = LintReport {
        findings: vec![LintFinding {
            rule_id: "DBC-VAL-001".to_string(),
            severity: LintSeverity::Warning,
            message: "test finding".to_string(),
            clause_id: None,
        }],
        passed: true,
        error_count: 0,
        warning_count: 1,
        info_count: 0,
    };

    let sarif = lint_report_to_sarif(&report, ".pmat-work/TEST/contract.json");

    assert_eq!(sarif["version"], "2.1.0");
    assert!(sarif["$schema"].as_str().unwrap().contains("sarif-schema"));
    assert_eq!(sarif["runs"].as_array().unwrap().len(), 1);

    let run = &sarif["runs"][0];
    assert_eq!(run["tool"]["driver"]["name"], "pmat-dbc-lint");
    assert_eq!(run["results"].as_array().unwrap().len(), 1);
    assert_eq!(run["results"][0]["ruleId"], "DBC-VAL-001");
    assert_eq!(run["results"][0]["level"], "warning");
}

#[test]
fn test_sarif_output_empty_report() {
    let report = LintReport {
        findings: vec![],
        passed: true,
        error_count: 0,
        warning_count: 0,
        info_count: 0,
    };

    let sarif = lint_report_to_sarif(&report, "contract.json");
    assert_eq!(sarif["runs"][0]["results"].as_array().unwrap().len(), 0);
}

#[test]
fn test_sarif_rule_catalog_completeness() {
    let sarif = lint_report_to_sarif(
        &LintReport {
            findings: vec![],
            passed: true,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
        },
        "c.json",
    );
    let rules = sarif["runs"][0]["tool"]["driver"]["rules"]
        .as_array()
        .unwrap();
    assert_eq!(rules.len(), 13, "SARIF should embed all 13 rules");

    // Verify all rule IDs are present
    let rule_ids: Vec<&str> = rules.iter().map(|r| r["id"].as_str().unwrap()).collect();
    assert!(rule_ids.contains(&"DBC-VAL-001"));
    assert!(rule_ids.contains(&"DBC-SCR-002"));
    assert!(rule_ids.contains(&"DBC-TRD-001"));
    assert!(rule_ids.contains(&"DBC-TRD-002"));
    assert!(rule_ids.contains(&"DBC-DRF-001"));
}

#[test]
fn test_sarif_severity_mapping() {
    let report = LintReport {
        findings: vec![
            LintFinding {
                rule_id: "DBC-VAL-004".to_string(),
                severity: LintSeverity::Error,
                message: "err".to_string(),
                clause_id: None,
            },
            LintFinding {
                rule_id: "DBC-AUD-003".to_string(),
                severity: LintSeverity::Info,
                message: "info".to_string(),
                clause_id: None,
            },
        ],
        passed: false,
        error_count: 1,
        warning_count: 0,
        info_count: 1,
    };

    let sarif = lint_report_to_sarif(&report, "c.json");
    let results = sarif["runs"][0]["results"].as_array().unwrap();
    assert_eq!(results[0]["level"], "error");
    assert_eq!(results[1]["level"], "note"); // Info maps to "note" in SARIF
}
