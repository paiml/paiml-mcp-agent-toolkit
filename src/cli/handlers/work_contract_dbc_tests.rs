// Tests for Design by Contract types
// Spec: docs/specifications/dbc.md

// === compare_thresholds tests ===

#[test]
fn test_compare_thresholds_none_none() {
    assert_eq!(compare_thresholds(&None, &None), ThresholdComparison::Equal);
}

#[test]
fn test_compare_thresholds_none_to_some_is_strengthened() {
    let child = Some(ClauseThreshold::Numeric {
        metric: "coverage".to_string(),
        op: ThresholdOp::Gte,
        value: 95.0,
    });
    assert_eq!(
        compare_thresholds(&None, &child),
        ThresholdComparison::Strengthened
    );
}

#[test]
fn test_compare_thresholds_some_to_none_is_weakened() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "coverage".to_string(),
        op: ThresholdOp::Gte,
        value: 95.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &None),
        ThresholdComparison::Weakened
    );
}

#[test]
fn test_compare_gte_higher_is_strengthened() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "coverage".to_string(),
        op: ThresholdOp::Gte,
        value: 95.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "coverage".to_string(),
        op: ThresholdOp::Gte,
        value: 96.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Strengthened
    );
}

#[test]
fn test_compare_gte_lower_is_weakened() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "coverage".to_string(),
        op: ThresholdOp::Gte,
        value: 95.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "coverage".to_string(),
        op: ThresholdOp::Gte,
        value: 90.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Weakened
    );
}

#[test]
fn test_compare_gte_equal() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "coverage".to_string(),
        op: ThresholdOp::Gte,
        value: 95.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "coverage".to_string(),
        op: ThresholdOp::Gte,
        value: 95.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Equal
    );
}

#[test]
fn test_compare_lte_lower_is_strengthened() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "complexity".to_string(),
        op: ThresholdOp::Lte,
        value: 20.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "complexity".to_string(),
        op: ThresholdOp::Lte,
        value: 15.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Strengthened
    );
}

#[test]
fn test_compare_lte_higher_is_weakened() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "complexity".to_string(),
        op: ThresholdOp::Lte,
        value: 20.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "complexity".to_string(),
        op: ThresholdOp::Lte,
        value: 25.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Weakened
    );
}

#[test]
fn test_compare_eq_same_value() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Eq,
        value: 42.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Eq,
        value: 42.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Equal
    );
}

#[test]
fn test_compare_eq_different_value_is_incompatible() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Eq,
        value: 42.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Eq,
        value: 43.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Incompatible
    );
}

#[test]
fn test_compare_different_ops_incompatible() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Gte,
        value: 95.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Lte,
        value: 95.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Incompatible
    );
}

#[test]
fn test_compare_different_types_incompatible() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Gte,
        value: 95.0,
    });
    let child = Some(ClauseThreshold::Boolean { expected: true });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Incompatible
    );
}

#[test]
fn test_compare_boolean_true_true_equal() {
    let parent = Some(ClauseThreshold::Boolean { expected: true });
    let child = Some(ClauseThreshold::Boolean { expected: true });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Equal
    );
}

#[test]
fn test_compare_boolean_false_to_true_strengthened() {
    let parent = Some(ClauseThreshold::Boolean { expected: false });
    let child = Some(ClauseThreshold::Boolean { expected: true });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Strengthened
    );
}

#[test]
fn test_compare_boolean_true_to_false_weakened() {
    let parent = Some(ClauseThreshold::Boolean { expected: true });
    let child = Some(ClauseThreshold::Boolean { expected: false });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Weakened
    );
}

#[test]
fn test_compare_delta_gte_higher_strengthened() {
    let parent = Some(ClauseThreshold::Delta {
        metric: "tdg".to_string(),
        op: ThresholdOp::Gte,
        value: 0.5,
    });
    let child = Some(ClauseThreshold::Delta {
        metric: "tdg".to_string(),
        op: ThresholdOp::Gte,
        value: 1.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Strengthened
    );
}

#[test]
fn test_compare_gt_higher_strengthened() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Gt,
        value: 10.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Gt,
        value: 15.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Strengthened
    );
}

#[test]
fn test_compare_lt_lower_strengthened() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Lt,
        value: 20.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Lt,
        value: 15.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Strengthened
    );
}

// === ContractQuality tests ===

#[test]
fn test_contract_quality_full() {
    let q = ContractQuality::calculate(14, 14);
    assert_eq!(q.score, 1.0);
    assert_eq!(q.rating, "Full");
}

#[test]
fn test_contract_quality_strong() {
    let q = ContractQuality::calculate(12, 14);
    assert!(q.score > 0.8);
    assert_eq!(q.rating, "Strong");
}

#[test]
fn test_contract_quality_partial() {
    let q = ContractQuality::calculate(8, 14);
    assert!(q.score >= 0.5);
    assert_eq!(q.rating, "Partial");
}

#[test]
fn test_contract_quality_weak() {
    let q = ContractQuality::calculate(3, 14);
    assert!(q.score < 0.5);
    assert_eq!(q.rating, "Weak");
}

#[test]
fn test_contract_quality_zero_applicable() {
    let q = ContractQuality::calculate(0, 0);
    assert_eq!(q.score, 0.0);
    assert_eq!(q.rating, "Weak");
}

// === ClauseKind display ===

#[test]
fn test_clause_kind_display() {
    assert_eq!(format!("{}", ClauseKind::Require), "require");
    assert_eq!(format!("{}", ClauseKind::Ensure), "ensure");
    assert_eq!(format!("{}", ClauseKind::Invariant), "invariant");
}

// === ThresholdOp display ===

#[test]
fn test_threshold_op_display() {
    assert_eq!(format!("{}", ThresholdOp::Gte), ">=");
    assert_eq!(format!("{}", ThresholdOp::Lte), "<=");
    assert_eq!(format!("{}", ThresholdOp::Eq), "==");
    assert_eq!(format!("{}", ThresholdOp::Gt), ">");
    assert_eq!(format!("{}", ThresholdOp::Lt), "<");
}

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

// === WorkContract v5.0 ===

#[test]
fn test_work_contract_new_is_v4() {
    let contract = WorkContract::new("TEST-1".to_string(), "abc123".to_string());
    assert_eq!(contract.version, "4.0");
    assert!(!contract.is_dbc());
    assert!(contract.require.is_empty());
    assert!(contract.ensure.is_empty());
    assert!(contract.invariant.is_empty());
}

#[test]
fn test_work_contract_with_dbc_is_v5() {
    let dir = tempfile::tempdir().unwrap();
    // Create a minimal git repo to trigger Universal profile
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let contract =
        WorkContract::with_dbc("TEST-1".to_string(), "abc123".to_string(), dir.path(), &[], 1)
            .unwrap();

    assert_eq!(contract.version, "5.0");
    assert!(contract.is_dbc());
    assert_eq!(
        contract.profile,
        Some(ContractProfile::Universal)
    );
    assert_eq!(contract.require.len(), 2);
    assert_eq!(contract.invariant.len(), 2);
    assert_eq!(contract.ensure.len(), 2);
    assert_eq!(contract.triad_claim_count(), 6);
}

#[test]
fn test_work_contract_with_dbc_rust_profile() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

    let contract =
        WorkContract::with_dbc("TEST-2".to_string(), "abc123".to_string(), dir.path(), &[], 1)
            .unwrap();

    assert_eq!(
        contract.profile,
        Some(ContractProfile::Rust)
    );
    assert_eq!(contract.triad_claim_count(), 14);
}

#[test]
fn test_work_contract_with_dbc_pmat_profile() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
    std::fs::create_dir_all(dir.path().join(".pmat")).unwrap();
    std::fs::write(dir.path().join(".pmat").join("context.db"), "").unwrap();

    let result =
        WorkContract::with_dbc("TEST-3".to_string(), "abc123".to_string(), dir.path(), &[], 1);

    match result {
        Ok(contract) => {
            assert_eq!(
                contract.profile,
                Some(ContractProfile::Pmat)
            );
            assert_eq!(contract.triad_claim_count(), 25);
        }
        Err(e) => {
            // Pmat profile requires cargo-llvm-cov; skip if not installed
            let msg = e.to_string();
            assert!(
                msg.contains("missing tool") || msg.contains("cargo-llvm-cov"),
                "Unexpected error: {}", msg
            );
        }
    }
}

#[test]
fn test_work_contract_with_dbc_exclusions() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let without = vec!["ensure.git_sync".to_string()];
    let contract =
        WorkContract::with_dbc("TEST-4".to_string(), "abc123".to_string(), dir.path(), &without, 1)
            .unwrap();

    assert_eq!(contract.triad_claim_count(), 5); // 6 - 1 excluded
    assert_eq!(contract.excluded_claims.len(), 1);
    assert_eq!(contract.excluded_claims[0].id, "ensure.git_sync");

    // Contract quality should reflect exclusion
    let quality = contract.contract_quality.as_ref().unwrap();
    assert_eq!(quality.active_claims, 5);
    assert_eq!(quality.applicable_claims, 6);
    assert!(quality.score < 1.0);
}

#[test]
fn test_work_contract_with_dbc_config_override() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

    // Override to Universal even though Cargo.toml exists
    let config_dir = dir.path().join(".pmat-work");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(
        config_dir.join("config.toml"),
        "[dbc]\nprofile = \"universal\"\n",
    )
    .unwrap();

    let contract =
        WorkContract::with_dbc("TEST-5".to_string(), "abc123".to_string(), dir.path(), &[], 1)
            .unwrap();

    assert_eq!(
        contract.profile,
        Some(ContractProfile::Universal)
    );
    assert_eq!(contract.triad_claim_count(), 6); // Universal, not Rust
}

// === v5.0 serialization round-trip ===

#[test]
fn test_work_contract_v5_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let contract =
        WorkContract::with_dbc("TEST-RT".to_string(), "abc123".to_string(), dir.path(), &[], 1)
            .unwrap();

    // Save
    let saved_path = contract.save(dir.path()).unwrap();
    assert!(saved_path.exists());

    // Load
    let loaded = WorkContract::load(dir.path(), "TEST-RT").unwrap();
    assert_eq!(loaded.version, "5.0");
    assert!(loaded.is_dbc());
    assert_eq!(loaded.require.len(), 2);
    assert_eq!(loaded.ensure.len(), 2);
    assert_eq!(loaded.invariant.len(), 2);
    assert_eq!(loaded.profile, Some(ContractProfile::Universal));
}

// === v4.0 backward compat ===

#[test]
fn test_v4_contract_loads_with_empty_triad() {
    let dir = tempfile::tempdir().unwrap();

    // Create a v4.0 contract (no triad fields)
    let v4 = WorkContract::new("TEST-V4".to_string(), "abc123".to_string());
    v4.save(dir.path()).unwrap();

    // Load it back
    let loaded = WorkContract::load(dir.path(), "TEST-V4").unwrap();
    assert_eq!(loaded.version, "4.0");
    assert!(!loaded.is_dbc());
    assert!(loaded.require.is_empty());
    assert!(loaded.ensure.is_empty());
    assert!(loaded.invariant.is_empty());
    assert!(loaded.profile.is_none());
    assert_eq!(loaded.iteration, 1);
    assert_eq!(loaded.claims.len(), 22);
}

// === Profile detection ===

#[test]
fn test_profile_detect_universal() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    assert_eq!(ContractProfile::detect(dir.path()), ContractProfile::Universal);
}

#[test]
fn test_profile_detect_rust() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"x\"").unwrap();
    assert_eq!(ContractProfile::detect(dir.path()), ContractProfile::Rust);
}

#[test]
fn test_profile_detect_pmat() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"x\"").unwrap();
    std::fs::create_dir_all(dir.path().join(".pmat")).unwrap();
    std::fs::write(dir.path().join(".pmat").join("context.idx"), "").unwrap();
    assert_eq!(ContractProfile::detect(dir.path()), ContractProfile::Pmat);
}

#[test]
fn test_profile_detect_stack() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(
        dir.path().join(".dbc-stack.toml"),
        "[stack]\nname = \"test\"\n",
    )
    .unwrap();
    match ContractProfile::detect(dir.path()) {
        ContractProfile::Stack { manifest_path } => {
            assert!(manifest_path.ends_with(".dbc-stack.toml"));
        }
        other => panic!("Expected Stack, got {:?}", other),
    }
}

// === CheckpointRecord tests ===

#[test]
fn test_checkpoint_record_new_all_pass() {
    let results = vec![
        InvariantResult {
            clause_id: "invariant.lint".to_string(),
            passed: true,
            explanation: "Lint clean".to_string(),
        },
        InvariantResult {
            clause_id: "invariant.complexity".to_string(),
            passed: true,
            explanation: "Max complexity: 18 (limit: 20)".to_string(),
        },
    ];

    let record = CheckpointRecord::new(
        "PMAT-500".to_string(),
        "abc123def".to_string(),
        1,
        results,
    );

    assert!(record.all_invariants_hold);
    assert_eq!(record.work_item_id, "PMAT-500");
    assert_eq!(record.git_sha, "abc123def");
    assert_eq!(record.iteration, 1);
    assert_eq!(record.invariant_results.len(), 2);
    assert!(!record.checkpoint_id.is_empty());
}

#[test]
fn test_checkpoint_record_new_with_failure() {
    let results = vec![
        InvariantResult {
            clause_id: "invariant.lint".to_string(),
            passed: true,
            explanation: "Lint clean".to_string(),
        },
        InvariantResult {
            clause_id: "invariant.complexity".to_string(),
            passed: false,
            explanation: "Max complexity: 32 exceeds limit 20".to_string(),
        },
    ];

    let record = CheckpointRecord::new(
        "PMAT-500".to_string(),
        "def456".to_string(),
        1,
        results,
    );

    assert!(!record.all_invariants_hold);
    assert_eq!(record.invariant_results.len(), 2);
}

#[test]
fn test_checkpoint_record_empty_invariants() {
    let record = CheckpointRecord::new(
        "PMAT-500".to_string(),
        "abc123".to_string(),
        1,
        vec![],
    );

    assert!(record.all_invariants_hold); // vacuously true
    assert!(record.invariant_results.is_empty());
}

#[test]
fn test_checkpoint_record_save_and_load() {
    let dir = tempfile::tempdir().unwrap();

    let results = vec![
        InvariantResult {
            clause_id: "invariant.lint".to_string(),
            passed: true,
            explanation: "Lint clean".to_string(),
        },
    ];

    let record = CheckpointRecord::new(
        "TEST-CK".to_string(),
        "abc123".to_string(),
        1,
        results,
    );

    // Save
    let path = record.save(dir.path()).unwrap();
    assert!(path.exists());
    assert!(path.to_string_lossy().contains("checkpoints"));

    // Load all
    let loaded = CheckpointRecord::load_all(dir.path(), "TEST-CK");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].work_item_id, "TEST-CK");
    assert_eq!(loaded[0].checkpoint_id, record.checkpoint_id);
    assert!(loaded[0].all_invariants_hold);
}

#[test]
fn test_checkpoint_record_load_multiple() {
    let dir = tempfile::tempdir().unwrap();

    // Save two checkpoints
    let r1 = CheckpointRecord::new(
        "TEST-MULTI".to_string(),
        "sha1".to_string(),
        1,
        vec![InvariantResult {
            clause_id: "invariant.lint".to_string(),
            passed: true,
            explanation: "ok".to_string(),
        }],
    );
    r1.save(dir.path()).unwrap();

    let r2 = CheckpointRecord::new(
        "TEST-MULTI".to_string(),
        "sha2".to_string(),
        1,
        vec![InvariantResult {
            clause_id: "invariant.lint".to_string(),
            passed: false,
            explanation: "lint failed".to_string(),
        }],
    );
    r2.save(dir.path()).unwrap();

    let loaded = CheckpointRecord::load_all(dir.path(), "TEST-MULTI");
    assert_eq!(loaded.len(), 2);
    // Sorted by timestamp, first should pass, second should fail
    assert!(loaded[0].all_invariants_hold);
    assert!(!loaded[1].all_invariants_hold);
}

#[test]
fn test_checkpoint_record_load_nonexistent() {
    let dir = tempfile::tempdir().unwrap();
    let loaded = CheckpointRecord::load_all(dir.path(), "NONEXISTENT");
    assert!(loaded.is_empty());
}

#[test]
fn test_checkpoint_serialization_round_trip() {
    let results = vec![
        InvariantResult {
            clause_id: "invariant.complexity".to_string(),
            passed: true,
            explanation: "Max complexity: 15".to_string(),
        },
    ];

    let record = CheckpointRecord::new(
        "PMAT-RT".to_string(),
        "deadbeef".to_string(),
        2,
        results,
    );

    let json = serde_json::to_string_pretty(&record).unwrap();
    let deserialized: CheckpointRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.checkpoint_id, record.checkpoint_id);
    assert_eq!(deserialized.work_item_id, "PMAT-RT");
    assert_eq!(deserialized.iteration, 2);
    assert_eq!(deserialized.invariant_results.len(), 1);
    assert!(deserialized.all_invariants_hold);
}

// === Subcontracting with iteration ===

#[test]
fn test_iteration_field_default() {
    let contract = WorkContract::new("TEST".to_string(), "abc".to_string());
    assert_eq!(contract.iteration, 1);
}

#[test]
fn test_iteration_persists_through_save_load() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let mut contract =
        WorkContract::with_dbc("TEST-IT".to_string(), "abc123".to_string(), dir.path(), &[], 1)
            .unwrap();
    // Simulate iteration increment
    contract.iteration = 2;
    contract.save(dir.path()).unwrap();

    let loaded = WorkContract::load(dir.path(), "TEST-IT").unwrap();
    assert_eq!(loaded.iteration, 2);
}
