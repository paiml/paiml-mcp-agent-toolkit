// ==================== DefectReport Tests ====================

fn create_test_location() -> CodeLocation {
    CodeLocation {
        file_path: PathBuf::from("test.rs"),
        line: 1,
        column: None,
        span_end_line: None,
    }
}

#[test]
fn test_defect_report_new() {
    let report = DefectReport::new(
        DefectCategory::TypeErrors,
        Severity::High,
        create_test_location(),
    );

    assert_eq!(report.category, DefectCategory::TypeErrors);
    assert_eq!(report.severity, Severity::High);
    assert_eq!(report.confidence, 0.0);
    assert_eq!(report.decision, OracleDecision::Skip);
    assert!(report.signals.is_empty());
    assert!(report.suggested_fixes.is_empty());
}

#[test]
fn test_defect_report_add_signal() {
    let mut report = DefectReport::new(
        DefectCategory::TypeErrors,
        Severity::High,
        create_test_location(),
    );

    let signal = SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error".to_string(),
        error_code: Some("E0308".to_string()),
        weight: 1.0,
    };

    report.add_signal(signal);

    assert_eq!(report.signals.len(), 1);
    // Confidence should be recalculated
    assert!(report.confidence > 0.0);
}

#[test]
fn test_defect_report_confidence_calculation() {
    let mut report = DefectReport::new(
        DefectCategory::TypeErrors,
        Severity::High,
        create_test_location(),
    );

    // TypeErrors has 0.95 base confidence
    // With weight 1.0, confidence should be 0.95
    report.add_signal(SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error".to_string(),
        error_code: None,
        weight: 1.0,
    });

    assert!((report.confidence - 0.95).abs() < 0.01);
}

#[test]
fn test_defect_report_confidence_with_low_weight() {
    let mut report = DefectReport::new(
        DefectCategory::TypeErrors,
        Severity::High,
        create_test_location(),
    );

    report.add_signal(SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error".to_string(),
        error_code: None,
        weight: 0.5,
    });

    // 0.95 * 0.5 = 0.475
    assert!((report.confidence - 0.475).abs() < 0.01);
}

#[test]
fn test_defect_report_update_decision_auto_apply() {
    let mut report = DefectReport::new(
        DefectCategory::TypeErrors,
        Severity::High,
        create_test_location(),
    );

    report.add_signal(SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error".to_string(),
        error_code: None,
        weight: 1.0,
    });

    // With confidence 0.95, auto_apply threshold 0.9, should be AutoApply
    report.update_decision(0.9, 0.7);
    assert_eq!(report.decision, OracleDecision::AutoApply);
}

#[test]
fn test_defect_report_update_decision_human_review() {
    let mut report = DefectReport::new(
        DefectCategory::TypeErrors,
        Severity::High,
        create_test_location(),
    );

    report.add_signal(SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error".to_string(),
        error_code: None,
        weight: 0.8,
    });

    // 0.95 * 0.8 = 0.76
    report.update_decision(0.9, 0.7);
    assert_eq!(report.decision, OracleDecision::HumanReview);
}

#[test]
fn test_defect_report_update_decision_skip() {
    let mut report = DefectReport::new(
        DefectCategory::TypeErrors,
        Severity::High,
        create_test_location(),
    );

    report.add_signal(SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error".to_string(),
        error_code: None,
        weight: 0.5,
    });

    // 0.95 * 0.5 = 0.475, below review threshold
    report.update_decision(0.9, 0.7);
    assert_eq!(report.decision, OracleDecision::Skip);
}

// ==================== ConvergenceTargets Tests ====================

#[test]
fn test_convergence_targets_default() {
    let targets = ConvergenceTargets::default();

    assert_eq!(targets.test_coverage, 0.95);
    assert_eq!(targets.mutation_score, 0.85);
    assert_eq!(targets.max_compiler_errors, 0);
    assert_eq!(targets.max_clippy_warnings, 0);
    assert_eq!(targets.max_test_failures, 0);
    assert_eq!(targets.min_tdg_score, 95.0);
    assert_eq!(targets.min_rust_project_score, 90);
    assert_eq!(targets.max_satd_markers, 0);
    assert_eq!(targets.max_dead_code, 0);
    assert_eq!(targets.max_cyclomatic_complexity, 15);
    assert_eq!(targets.max_cognitive_complexity, 25);
    assert_eq!(targets.max_build_time, Duration::from_secs(60));
}

#[test]
fn test_convergence_targets_check_converged() {
    let targets = ConvergenceTargets::default();
    let metrics = ProjectMetrics {
        test_coverage: 0.96,
        mutation_score: 0.86,
        compiler_errors: 0,
        clippy_warnings: 0,
        test_failures: 0,
        tdg_score: 96.0,
        rust_project_score: 91,
        satd_markers: 0,
        dead_code_items: 0,
        max_cyclomatic_complexity: 10,
        max_cognitive_complexity: 20,
        build_time: Duration::from_secs(30),
    };

    let status = targets.check(&metrics);
    match status {
        ConvergenceStatus::Converged => (),
        ConvergenceStatus::NotConverged { remaining } => {
            panic!("Should be converged, but got: {:?}", remaining);
        }
    }
}

#[test]
fn test_convergence_targets_check_not_converged_coverage() {
    let targets = ConvergenceTargets::default();
    let metrics = ProjectMetrics {
        test_coverage: 0.80, // Below target
        ..Default::default()
    };

    let status = targets.check(&metrics);
    match status {
        ConvergenceStatus::Converged => panic!("Should not be converged"),
        ConvergenceStatus::NotConverged { remaining } => {
            assert!(remaining.iter().any(|s| s.contains("Coverage")));
        }
    }
}

#[test]
fn test_convergence_targets_check_multiple_failures() {
    let targets = ConvergenceTargets::default();
    let metrics = ProjectMetrics {
        test_coverage: 0.80,
        compiler_errors: 5,
        tdg_score: 50.0,
        ..Default::default()
    };

    let status = targets.check(&metrics);
    match status {
        ConvergenceStatus::Converged => panic!("Should not be converged"),
        ConvergenceStatus::NotConverged { remaining } => {
            assert!(remaining.len() >= 3);
        }
    }
}

// ==================== OracleConfig Tests ====================

#[test]
fn test_oracle_config_default() {
    let config = OracleConfig::default();

    assert_eq!(config.max_iterations, 100);
    assert_eq!(config.min_progress_per_iteration, 0.001);
    assert_eq!(config.stagnation_threshold, 5);
    assert!(config.andon_enabled);
    assert_eq!(config.require_human_approval_above, Some(10));
    assert_eq!(config.auto_apply_threshold, 0.9);
    assert_eq!(config.review_threshold, 0.7);
    assert_eq!(config.batch_size, 10);
}

#[test]
fn test_oracle_config_serialization() {
    let config = OracleConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let parsed: OracleConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.max_iterations, parsed.max_iterations);
    assert_eq!(config.batch_size, parsed.batch_size);
}

// ==================== SuggestedFix Tests ====================

#[test]
fn test_suggested_fix_creation() {
    let fix = SuggestedFix {
        description: "Apply clippy suggestion".to_string(),
        confidence: 0.95,
        fix_type: FixType::ClippyAutoFix,
    };

    assert_eq!(fix.confidence, 0.95);
}

#[test]
fn test_suggested_fix_serialization() {
    let fix = SuggestedFix {
        description: "Fix it".to_string(),
        confidence: 0.8,
        fix_type: FixType::Replacement {
            old: "a".to_string(),
            new: "b".to_string(),
        },
    };

    let json = serde_json::to_string(&fix).unwrap();
    let parsed: SuggestedFix = serde_json::from_str(&json).unwrap();

    assert_eq!(fix.description, parsed.description);
    assert_eq!(fix.confidence, parsed.confidence);
}

// ==================== ProjectMetrics Tests ====================

#[test]
fn test_project_metrics_default() {
    let metrics = ProjectMetrics::default();

    assert_eq!(metrics.test_coverage, 0.0);
    assert_eq!(metrics.mutation_score, 0.0);
    assert_eq!(metrics.compiler_errors, 0);
    assert_eq!(metrics.clippy_warnings, 0);
    assert_eq!(metrics.tdg_score, 0.0);
    assert_eq!(metrics.build_time, Duration::default());
}

#[test]
fn test_project_metrics_serialization() {
    let metrics = ProjectMetrics {
        test_coverage: 0.85,
        mutation_score: 0.75,
        compiler_errors: 2,
        clippy_warnings: 5,
        test_failures: 1,
        tdg_score: 80.0,
        rust_project_score: 85,
        satd_markers: 3,
        dead_code_items: 2,
        max_cyclomatic_complexity: 12,
        max_cognitive_complexity: 18,
        build_time: Duration::from_secs(45),
    };

    let json = serde_json::to_string(&metrics).unwrap();
    let parsed: ProjectMetrics = serde_json::from_str(&json).unwrap();

    assert_eq!(metrics.test_coverage, parsed.test_coverage);
    assert_eq!(metrics.compiler_errors, parsed.compiler_errors);
}

// ==================== Integration Tests ====================

#[test]
fn test_defect_report_full_workflow() {
    let mut report = DefectReport::new(
        DefectCategory::OwnershipBorrow,
        Severity::High,
        CodeLocation {
            file_path: PathBuf::from("src/lib.rs"),
            line: 42,
            column: Some(10),
            span_end_line: Some(42),
        },
    );

    // Add signals from multiple sources
    report.add_signal(SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "cannot move out of `x` because it is borrowed".to_string(),
        error_code: Some("E0505".to_string()),
        weight: 1.0,
    });

    report.add_signal(SignalEvidence {
        source: SignalSource::PmatComplexity,
        raw_message: "High complexity at this location".to_string(),
        error_code: None,
        weight: 0.7,
    });

    // Add suggested fix
    report.suggested_fixes.push(SuggestedFix {
        description: "Clone the value before borrowing".to_string(),
        confidence: 0.85,
        fix_type: FixType::Replacement {
            old: "let y = &x;".to_string(),
            new: "let y = x.clone();".to_string(),
        },
    });

    // Update decision
    report.update_decision(0.9, 0.7);

    // OwnershipBorrow has 0.92 confidence * max_weight(1.0) = 0.92
    assert!(report.confidence > 0.9);
    assert_eq!(report.decision, OracleDecision::AutoApply);
    assert_eq!(report.signals.len(), 2);
    assert_eq!(report.suggested_fixes.len(), 1);
}
