// Tests for Design by Contract types - Part 2: Subcontracting + Profiles
// Spec: docs/specifications/dbc.md

// === SubcontractingViolation display ===

#[test]
fn test_subcontracting_violation_display() {
    let v = SubcontractingViolation::PostconditionDropped {
        clause: "ensure.coverage".to_string(),
    };
    assert!(format!("{}", v).contains("ensure.coverage"));
    assert!(format!("{}", v).contains("dropped"));
}

// === validate_subcontracting tests ===

#[test]
fn test_subcontracting_ok_when_strengthened() {
    let parent = vec![ContractClause {
        id: "ensure.coverage".to_string(),
        kind: ClauseKind::Ensure,
        description: "Coverage >= 95%".to_string(),
        falsification_method: FalsificationMethod::AbsoluteCoverage,
        threshold: Some(ClauseThreshold::Numeric {
            metric: "coverage".to_string(),
            op: ThresholdOp::Gte,
            value: 95.0,
        }),
        blocking: true,
        source: ClauseSource::Default,
    }];
    let child = vec![ContractClause {
        id: "ensure.coverage".to_string(),
        kind: ClauseKind::Ensure,
        description: "Coverage >= 96%".to_string(),
        falsification_method: FalsificationMethod::AbsoluteCoverage,
        threshold: Some(ClauseThreshold::Numeric {
            metric: "coverage".to_string(),
            op: ThresholdOp::Gte,
            value: 96.0,
        }),
        blocking: true,
        source: ClauseSource::Default,
    }];
    assert!(validate_subcontracting(&parent, &child).is_ok());
}

#[test]
fn test_subcontracting_fails_when_weakened() {
    let parent = vec![ContractClause {
        id: "ensure.coverage".to_string(),
        kind: ClauseKind::Ensure,
        description: "Coverage >= 95%".to_string(),
        falsification_method: FalsificationMethod::AbsoluteCoverage,
        threshold: Some(ClauseThreshold::Numeric {
            metric: "coverage".to_string(),
            op: ThresholdOp::Gte,
            value: 95.0,
        }),
        blocking: true,
        source: ClauseSource::Default,
    }];
    let child = vec![ContractClause {
        id: "ensure.coverage".to_string(),
        kind: ClauseKind::Ensure,
        description: "Coverage >= 80%".to_string(),
        falsification_method: FalsificationMethod::AbsoluteCoverage,
        threshold: Some(ClauseThreshold::Numeric {
            metric: "coverage".to_string(),
            op: ThresholdOp::Gte,
            value: 80.0,
        }),
        blocking: true,
        source: ClauseSource::Default,
    }];
    assert!(validate_subcontracting(&parent, &child).is_err());
}

#[test]
fn test_subcontracting_fails_when_dropped() {
    let parent = vec![ContractClause {
        id: "ensure.coverage".to_string(),
        kind: ClauseKind::Ensure,
        description: "Coverage >= 95%".to_string(),
        falsification_method: FalsificationMethod::AbsoluteCoverage,
        threshold: Some(ClauseThreshold::Numeric {
            metric: "coverage".to_string(),
            op: ThresholdOp::Gte,
            value: 95.0,
        }),
        blocking: true,
        source: ClauseSource::Default,
    }];
    let child: Vec<ContractClause> = vec![];
    assert!(validate_subcontracting(&parent, &child).is_err());
}

#[test]
fn test_subcontracting_ok_with_new_postcondition() {
    let parent: Vec<ContractClause> = vec![];
    let child = vec![ContractClause {
        id: "ensure.mutation_score".to_string(),
        kind: ClauseKind::Ensure,
        description: "Mutation score >= 80%".to_string(),
        falsification_method: FalsificationMethod::AbsoluteCoverage,
        threshold: Some(ClauseThreshold::Numeric {
            metric: "mutation".to_string(),
            op: ThresholdOp::Gte,
            value: 80.0,
        }),
        blocking: true,
        source: ClauseSource::Default,
    }];
    assert!(validate_subcontracting(&parent, &child).is_ok());
}

// === apply_exclusions tests ===

#[test]
fn test_apply_exclusions_removes_matching() {
    let clauses = vec![
        ContractClause {
            id: "ensure.coverage".to_string(),
            kind: ClauseKind::Ensure,
            description: "Coverage".to_string(),
            falsification_method: FalsificationMethod::AbsoluteCoverage,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.tests_pass".to_string(),
            kind: ClauseKind::Ensure,
            description: "Tests".to_string(),
            falsification_method: FalsificationMethod::MetaFalsification,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
    ];

    let without = vec!["ensure.coverage".to_string()];
    let (active, excluded) = apply_exclusions(clauses, &without);

    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, "ensure.tests_pass");
    assert_eq!(excluded.len(), 1);
    assert_eq!(excluded[0].id, "ensure.coverage");
    assert_eq!(excluded[0].reason, "developer_excluded");
}

#[test]
fn test_apply_exclusions_empty_without() {
    let clauses = vec![ContractClause {
        id: "ensure.coverage".to_string(),
        kind: ClauseKind::Ensure,
        description: "Coverage".to_string(),
        falsification_method: FalsificationMethod::AbsoluteCoverage,
        threshold: None,
        blocking: true,
        source: ClauseSource::Default,
    }];

    let (active, excluded) = apply_exclusions(clauses, &[]);
    assert_eq!(active.len(), 1);
    assert!(excluded.is_empty());
}

// === classify_claims tests ===

#[test]
fn test_classify_claims_separates_triad() {
    let clauses = vec![
        ContractClause {
            id: "require.compiles".to_string(),
            kind: ClauseKind::Require,
            description: "Compiles".to_string(),
            falsification_method: FalsificationMethod::ManifestIntegrity,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "invariant.lint".to_string(),
            kind: ClauseKind::Invariant,
            description: "Lint".to_string(),
            falsification_method: FalsificationMethod::LintPass,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.coverage".to_string(),
            kind: ClauseKind::Ensure,
            description: "Coverage".to_string(),
            falsification_method: FalsificationMethod::AbsoluteCoverage,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
    ];

    let (require, ensure, invariant) = classify_claims(&clauses);
    assert_eq!(require.len(), 1);
    assert_eq!(ensure.len(), 1);
    assert_eq!(invariant.len(), 1);
    assert_eq!(require[0].id, "require.compiles");
    assert_eq!(ensure[0].id, "ensure.coverage");
    assert_eq!(invariant[0].id, "invariant.lint");
}

// === Profile claim counts ===

#[test]
fn test_universal_profile_has_6_claims() {
    let config = DbcConfig::default();
    let claims = claims_for_profile(&ContractProfile::Universal, &config);
    assert_eq!(claims.len(), 6, "Universal profile should have 6 claims");

    let (require, ensure, invariant) = classify_claims(&claims);
    assert_eq!(require.len(), 2);
    assert_eq!(invariant.len(), 2);
    assert_eq!(ensure.len(), 2);
}

#[test]
fn test_rust_profile_has_14_claims() {
    let config = DbcConfig::default();
    let claims = claims_for_profile(&ContractProfile::Rust, &config);
    assert_eq!(claims.len(), 14, "Rust profile should have 14 claims");

    let (require, ensure, invariant) = classify_claims(&claims);
    assert_eq!(require.len(), 2);
    assert_eq!(invariant.len(), 4);
    assert_eq!(ensure.len(), 8);
}

#[test]
fn test_pmat_profile_has_25_claims() {
    let config = DbcConfig::default();
    let claims = claims_for_profile(&ContractProfile::Pmat, &config);
    assert_eq!(claims.len(), 25, "Pmat profile should have 25 claims");

    let (require, ensure, invariant) = classify_claims(&claims);
    assert_eq!(require.len(), 4);
    assert_eq!(invariant.len(), 7);
    assert_eq!(ensure.len(), 14);
}

#[test]
fn test_custom_profile_filters_claims() {
    let config = DbcConfig::default();
    let profile = ContractProfile::Custom {
        claim_ids: vec![
            "require.compiles".to_string(),
            "ensure.coverage".to_string(),
        ],
    };
    let claims = claims_for_profile(&profile, &config);
    assert_eq!(claims.len(), 2);
}

// === Config threshold overrides ===

#[test]
fn test_config_threshold_overrides() {
    let config = DbcConfig {
        thresholds: DbcThresholdOverrides {
            coverage_pct: Some(80.0),
            max_complexity: Some(25),
            max_file_lines: Some(600),
        },
        ..Default::default()
    };
    let claims = claims_for_profile(&ContractProfile::Rust, &config);

    // Find coverage claim and verify threshold
    let cov = claims.iter().find(|c| c.id == "ensure.coverage").unwrap();
    if let Some(ClauseThreshold::Numeric { value, .. }) = &cov.threshold {
        assert_eq!(*value, 80.0);
    } else {
        panic!("Expected Numeric threshold for coverage");
    }

    // Find complexity claim and verify threshold
    let cx = claims
        .iter()
        .find(|c| c.id == "invariant.complexity")
        .unwrap();
    if let Some(ClauseThreshold::Numeric { value, .. }) = &cx.threshold {
        assert_eq!(*value, 25.0);
    } else {
        panic!("Expected Numeric threshold for complexity");
    }
}

// === ContractProfile name ===

#[test]
fn test_profile_names() {
    assert_eq!(ContractProfile::Universal.name(), "Universal");
    assert_eq!(ContractProfile::Rust.name(), "Rust");
    assert_eq!(ContractProfile::Pmat.name(), "Pmat");
    assert_eq!(
        ContractProfile::Stack {
            manifest_path: PathBuf::from(".dbc-stack.toml")
        }
        .name(),
        "Stack"
    );
    assert_eq!(
        ContractProfile::Custom {
            claim_ids: vec![]
        }
        .name(),
        "Custom"
    );
}

// === DbcConfig parsing ===

#[test]
fn test_dbc_config_parse_profile_override() {
    let toml = r#"
[dbc]
profile = "universal"
"#;
    let config = DbcConfig::parse_toml(toml).unwrap();
    assert_eq!(
        config.profile_override,
        Some(ContractProfile::Universal)
    );
}

#[test]
fn test_dbc_config_parse_custom_claims() {
    let toml = r#"
[dbc]
profile = "custom"
claims = ["require.compiles", "ensure.tests_pass"]
"#;
    let config = DbcConfig::parse_toml(toml).unwrap();
    match config.profile_override {
        Some(ContractProfile::Custom { claim_ids }) => {
            assert_eq!(claim_ids.len(), 2);
            assert!(claim_ids.contains(&"require.compiles".to_string()));
        }
        _ => panic!("Expected Custom profile"),
    }
}

#[test]
fn test_dbc_config_parse_thresholds() {
    let toml = r#"
[dbc.thresholds]
coverage_pct = 80.0
max_complexity = 25
max_file_lines = 600
"#;
    let config = DbcConfig::parse_toml(toml).unwrap();
    assert_eq!(config.thresholds.coverage_pct, Some(80.0));
    assert_eq!(config.thresholds.max_complexity, Some(25));
    assert_eq!(config.thresholds.max_file_lines, Some(600));
}

#[test]
fn test_dbc_config_parse_rescue() {
    let toml = r#"
[dbc.rescue]
enabled = true
"#;
    let config = DbcConfig::parse_toml(toml).unwrap();
    assert_eq!(config.rescue_enabled, Some(true));
}

#[test]
fn test_dbc_config_empty_toml() {
    let config = DbcConfig::parse_toml("").unwrap();
    assert!(config.profile_override.is_none());
    assert!(config.thresholds.coverage_pct.is_none());
}
