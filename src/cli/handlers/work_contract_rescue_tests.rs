// Tests for Rescue Protocol (Phase 4)
// Spec: docs/specifications/dbc.md §6

// === RescueStrategy display ===

#[test]
fn test_rescue_strategy_display() {
    assert_eq!(format!("{}", RescueStrategy::CoverageGapAnalysis), "CoverageGapAnalysis");
    assert_eq!(format!("{}", RescueStrategy::FiveWhysAnalysis), "FiveWhysAnalysis");
    assert_eq!(format!("{}", RescueStrategy::DeadCodeRemoval), "DeadCodeRemoval");
    assert_eq!(format!("{}", RescueStrategy::SatdResolution), "SatdResolution");
    assert_eq!(format!("{}", RescueStrategy::ComplexityReduction), "ComplexityReduction");
    assert_eq!(
        format!("{}", RescueStrategy::ManualIntervention { guidance: "fix it".to_string() }),
        "ManualIntervention"
    );
}

// === rescue_strategy_for mapping ===

#[test]
fn test_rescue_strategy_coverage_methods() {
    assert_eq!(
        rescue_strategy_for(&FalsificationMethod::AbsoluteCoverage),
        Some(RescueStrategy::CoverageGapAnalysis)
    );
    assert_eq!(
        rescue_strategy_for(&FalsificationMethod::DifferentialCoverage),
        Some(RescueStrategy::CoverageGapAnalysis)
    );
    assert_eq!(
        rescue_strategy_for(&FalsificationMethod::PerFileCoverage),
        Some(RescueStrategy::CoverageGapAnalysis)
    );
}

#[test]
fn test_rescue_strategy_tdg() {
    assert_eq!(
        rescue_strategy_for(&FalsificationMethod::TdgRegression),
        Some(RescueStrategy::FiveWhysAnalysis)
    );
}

#[test]
fn test_rescue_strategy_complexity() {
    assert_eq!(
        rescue_strategy_for(&FalsificationMethod::ComplexityRegression),
        Some(RescueStrategy::ComplexityReduction)
    );
}

#[test]
fn test_rescue_strategy_satd() {
    assert_eq!(
        rescue_strategy_for(&FalsificationMethod::SatdDetection),
        Some(RescueStrategy::SatdResolution)
    );
}

#[test]
fn test_rescue_strategy_dead_code() {
    assert_eq!(
        rescue_strategy_for(&FalsificationMethod::DeadCodeDetection),
        Some(RescueStrategy::DeadCodeRemoval)
    );
}

#[test]
fn test_rescue_strategy_manual_intervention() {
    match rescue_strategy_for(&FalsificationMethod::SupplyChainIntegrity) {
        Some(RescueStrategy::ManualIntervention { guidance }) => {
            assert!(guidance.contains("cargo audit"));
        }
        other => panic!("Expected ManualIntervention, got {:?}", other),
    }

    match rescue_strategy_for(&FalsificationMethod::FileSizeRegression) {
        Some(RescueStrategy::ManualIntervention { guidance }) => {
            assert!(guidance.contains("Split large files"));
        }
        other => panic!("Expected ManualIntervention, got {:?}", other),
    }

    match rescue_strategy_for(&FalsificationMethod::GitHubSync) {
        Some(RescueStrategy::ManualIntervention { guidance }) => {
            assert!(guidance.contains("git push"));
        }
        other => panic!("Expected ManualIntervention, got {:?}", other),
    }
}

#[test]
fn test_rescue_strategy_none_for_non_rescuable() {
    assert!(rescue_strategy_for(&FalsificationMethod::ManifestIntegrity).is_none());
    assert!(rescue_strategy_for(&FalsificationMethod::MetaFalsification).is_none());
    assert!(rescue_strategy_for(&FalsificationMethod::CoverageGaming).is_none());
    assert!(rescue_strategy_for(&FalsificationMethod::ExamplesCompile).is_none());
    assert!(rescue_strategy_for(&FalsificationMethod::BookValidation).is_none());
    assert!(rescue_strategy_for(&FalsificationMethod::LintPass).is_none());
    assert!(rescue_strategy_for(&FalsificationMethod::VariantCoverage).is_none());
    assert!(rescue_strategy_for(&FalsificationMethod::FixChainLimit).is_none());
    assert!(rescue_strategy_for(&FalsificationMethod::CrossCrateParity).is_none());
    assert!(rescue_strategy_for(&FalsificationMethod::RegressionGate).is_none());
    assert!(rescue_strategy_for(&FalsificationMethod::FormalProofVerification).is_none());
    assert!(rescue_strategy_for(&FalsificationMethod::SpecQuality).is_none());
}

// === execute_rescue ===

#[test]
fn test_execute_rescue_coverage_gap() {
    let dir = tempfile::tempdir().unwrap();
    let record = execute_rescue(
        dir.path(),
        "PMAT-500",
        "ensure.absolute_coverage",
        &RescueStrategy::CoverageGapAnalysis,
    );

    assert_eq!(record.work_item_id, "PMAT-500");
    assert_eq!(record.violated_clause, "ensure.absolute_coverage");
    assert_eq!(record.strategy, RescueStrategy::CoverageGapAnalysis);
    assert_eq!(record.outcome, RescueOutcome::ManualActionRequired);
    assert!(record.retry_allowed);
    assert!(!record.rescue_id.is_empty());
    assert!(record.diagnosis.summary.contains("Coverage gap"));
    assert!(!record.diagnosis.suggested_actions.is_empty());
}

#[test]
fn test_execute_rescue_five_whys() {
    let dir = tempfile::tempdir().unwrap();
    let record = execute_rescue(
        dir.path(),
        "PMAT-500",
        "ensure.tdg_regression",
        &RescueStrategy::FiveWhysAnalysis,
    );

    assert_eq!(record.strategy, RescueStrategy::FiveWhysAnalysis);
    assert!(record.diagnosis.summary.contains("Five Whys"));
}

#[test]
fn test_execute_rescue_manual_intervention() {
    let dir = tempfile::tempdir().unwrap();
    let record = execute_rescue(
        dir.path(),
        "PMAT-500",
        "ensure.supply_chain",
        &RescueStrategy::ManualIntervention {
            guidance: "Run cargo audit".to_string(),
        },
    );

    assert!(record.diagnosis.summary.contains("Run cargo audit"));
}

// === RescueRecord save/load ===

#[test]
fn test_rescue_record_save_and_load() {
    let dir = tempfile::tempdir().unwrap();
    let record = execute_rescue(
        dir.path(),
        "TEST-RES",
        "ensure.coverage",
        &RescueStrategy::CoverageGapAnalysis,
    );

    let path = record.save(dir.path()).unwrap();
    assert!(path.exists());
    assert!(path.to_string_lossy().contains("rescue"));

    let loaded = RescueRecord::load_all(dir.path(), "TEST-RES");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].rescue_id, record.rescue_id);
    assert_eq!(loaded[0].violated_clause, "ensure.coverage");
}

#[test]
fn test_rescue_record_load_empty() {
    let dir = tempfile::tempdir().unwrap();
    let loaded = RescueRecord::load_all(dir.path(), "NONEXISTENT");
    assert!(loaded.is_empty());
}

#[test]
fn test_rescue_record_serialization_round_trip() {
    let record = execute_rescue(
        std::path::Path::new("/tmp"),
        "PMAT-RT",
        "ensure.complexity",
        &RescueStrategy::ComplexityReduction,
    );

    let json = serde_json::to_string_pretty(&record).unwrap();
    let deserialized: RescueRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.rescue_id, record.rescue_id);
    assert_eq!(deserialized.work_item_id, "PMAT-RT");
    assert_eq!(deserialized.violated_clause, "ensure.complexity");
    assert_eq!(deserialized.strategy, RescueStrategy::ComplexityReduction);
    assert_eq!(deserialized.outcome, RescueOutcome::ManualActionRequired);
}

// === is_rescue_enabled ===

#[test]
fn test_rescue_enabled_pmat_default() {
    let config = DbcConfig::default();
    assert!(is_rescue_enabled(&Some(ContractProfile::Pmat), &config));
}

#[test]
fn test_rescue_disabled_universal_default() {
    let config = DbcConfig::default();
    assert!(!is_rescue_enabled(&Some(ContractProfile::Universal), &config));
}

#[test]
fn test_rescue_disabled_rust_default() {
    let config = DbcConfig::default();
    assert!(!is_rescue_enabled(&Some(ContractProfile::Rust), &config));
}

#[test]
fn test_rescue_enabled_via_config() {
    let config = DbcConfig {
        rescue_enabled: Some(true),
        ..Default::default()
    };
    // Config override: even Universal gets rescue
    assert!(is_rescue_enabled(&Some(ContractProfile::Universal), &config));
}

#[test]
fn test_rescue_disabled_via_config() {
    let config = DbcConfig {
        rescue_enabled: Some(false),
        ..Default::default()
    };
    // Config override: even Pmat loses rescue
    assert!(!is_rescue_enabled(&Some(ContractProfile::Pmat), &config));
}

#[test]
fn test_rescue_enabled_none_profile() {
    let config = DbcConfig::default();
    assert!(!is_rescue_enabled(&None, &config));
}

// === RescueDiagnosis ===

#[test]
fn test_rescue_diagnosis_has_suggested_actions() {
    let dir = tempfile::tempdir().unwrap();
    let record = execute_rescue(
        dir.path(),
        "PMAT-500",
        "ensure.coverage",
        &RescueStrategy::CoverageGapAnalysis,
    );

    assert!(record.diagnosis.suggested_actions.len() >= 2);
    assert!(record.diagnosis.suggested_actions.iter().any(|a| a.contains("pmat")));
}

#[test]
fn test_rescue_all_strategies_produce_diagnosis() {
    let dir = tempfile::tempdir().unwrap();
    let strategies = vec![
        RescueStrategy::CoverageGapAnalysis,
        RescueStrategy::FiveWhysAnalysis,
        RescueStrategy::DeadCodeRemoval,
        RescueStrategy::SatdResolution,
        RescueStrategy::ComplexityReduction,
        RescueStrategy::ManualIntervention {
            guidance: "Do something".to_string(),
        },
    ];

    for strategy in strategies {
        let record = execute_rescue(dir.path(), "TEST", "ensure.x", &strategy);
        assert!(!record.diagnosis.summary.is_empty(), "Strategy {:?} should produce summary", strategy);
        assert!(!record.diagnosis.suggested_actions.is_empty(), "Strategy {:?} should have actions", strategy);
    }
}
