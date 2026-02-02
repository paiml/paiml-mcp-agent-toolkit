//! Coverage boost tests for services/oracle/signal_collector module
//!
//! Tests for SignalCollector trait implementations, AggregatedCollector,
//! signal aggregation, and conversion to defect reports.

use crate::services::oracle::signal_collector::{
    AggregatedCollector, ClippyCollector, RustcCollector, SignalCollector, TestCollector,
};
use crate::services::oracle::types::{
    CodeLocation, ConvergenceStatus, ConvergenceTargets, DefectCategory, DefectReport, FixType,
    OracleConfig, OracleDecision, ProjectMetrics, Severity, SignalEvidence, SignalSource,
    SuggestedFix,
};
use std::path::PathBuf;
use std::time::Duration;

// ============ RustcCollector Tests ============

#[test]
fn test_rustc_collector_source_type() {
    let collector = RustcCollector;
    let source = collector.source();
    assert_eq!(source, SignalSource::Rustc);
}

#[test]
fn test_rustc_collector_source_serialization() {
    let collector = RustcCollector;
    let source = collector.source();
    let json = serde_json::to_string(&source).expect("Should serialize");
    assert!(json.contains("Rustc"));
}

#[test]
fn test_rustc_collector_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<RustcCollector>();
}

// ============ ClippyCollector Tests ============

#[test]
fn test_clippy_collector_source_type() {
    let collector = ClippyCollector;
    let source = collector.source();
    assert_eq!(source, SignalSource::Clippy);
}

#[test]
fn test_clippy_collector_source_serialization() {
    let collector = ClippyCollector;
    let source = collector.source();
    let json = serde_json::to_string(&source).expect("Should serialize");
    assert!(json.contains("Clippy"));
}

#[test]
fn test_clippy_collector_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ClippyCollector>();
}

// ============ TestCollector Tests ============

#[test]
fn test_test_collector_source_type() {
    let collector = TestCollector;
    let source = collector.source();
    assert_eq!(source, SignalSource::CargoTest);
}

#[test]
fn test_test_collector_source_serialization() {
    let collector = TestCollector;
    let source = collector.source();
    let json = serde_json::to_string(&source).expect("Should serialize");
    assert!(json.contains("CargoTest"));
}

#[test]
fn test_test_collector_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<TestCollector>();
}

// ============ AggregatedCollector Construction Tests ============

#[test]
fn test_aggregated_collector_new_default_collectors() {
    let collector = AggregatedCollector::new();
    assert_eq!(collector.collector_count(), 3);
}

#[test]
fn test_aggregated_collector_default_trait() {
    let collector = AggregatedCollector::default();
    assert_eq!(collector.collector_count(), 3);
}

#[test]
fn test_aggregated_collector_new_vs_default() {
    let new_collector = AggregatedCollector::new();
    let default_collector = AggregatedCollector::default();
    assert_eq!(
        new_collector.collector_count(),
        default_collector.collector_count()
    );
}

#[test]
fn test_aggregated_collector_add_collector_increments_count() {
    let mut collector = AggregatedCollector::new();
    let initial_count = collector.collector_count();
    collector.add_collector(Box::new(RustcCollector));
    assert_eq!(collector.collector_count(), initial_count + 1);
}

#[test]
fn test_aggregated_collector_add_multiple_collectors() {
    let mut collector = AggregatedCollector::new();
    collector.add_collector(Box::new(RustcCollector));
    collector.add_collector(Box::new(ClippyCollector));
    collector.add_collector(Box::new(TestCollector));
    assert_eq!(collector.collector_count(), 6); // 3 default + 3 added
}

#[test]
fn test_aggregated_collector_add_same_collector_type() {
    let mut collector = AggregatedCollector::new();
    collector.add_collector(Box::new(RustcCollector));
    collector.add_collector(Box::new(RustcCollector));
    assert_eq!(collector.collector_count(), 5); // 3 default + 2 RustcCollectors
}

// ============ SignalEvidence Tests ============

#[test]
fn test_signal_evidence_with_error_code() {
    let signal = SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error[E0308]: mismatched types".to_string(),
        error_code: Some("E0308".to_string()),
        weight: 1.0,
    };
    assert!(signal.error_code.is_some());
    assert_eq!(signal.error_code.as_ref().unwrap(), "E0308");
}

#[test]
fn test_signal_evidence_without_error_code() {
    let signal = SignalEvidence {
        source: SignalSource::CargoTest,
        raw_message: "Test failed".to_string(),
        error_code: None,
        weight: 1.0,
    };
    assert!(signal.error_code.is_none());
}

#[test]
fn test_signal_evidence_weight_full() {
    let signal = SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error".to_string(),
        error_code: None,
        weight: 1.0,
    };
    assert!((signal.weight - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_signal_evidence_weight_partial() {
    let signal = SignalEvidence {
        source: SignalSource::Clippy,
        raw_message: "warning".to_string(),
        error_code: None,
        weight: 0.5,
    };
    assert!((signal.weight - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_signal_evidence_weight_zero() {
    let signal = SignalEvidence {
        source: SignalSource::PmatSatd,
        raw_message: "info".to_string(),
        error_code: None,
        weight: 0.0,
    };
    assert!((signal.weight - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_signal_evidence_clone() {
    let signal = SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error".to_string(),
        error_code: Some("E0001".to_string()),
        weight: 0.9,
    };
    let cloned = signal.clone();
    assert_eq!(signal.source, cloned.source);
    assert_eq!(signal.raw_message, cloned.raw_message);
    assert_eq!(signal.error_code, cloned.error_code);
    assert!((signal.weight - cloned.weight).abs() < f32::EPSILON);
}

#[test]
fn test_signal_evidence_debug() {
    let signal = SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error".to_string(),
        error_code: None,
        weight: 1.0,
    };
    let debug = format!("{:?}", signal);
    assert!(debug.contains("SignalEvidence"));
    assert!(debug.contains("Rustc"));
}

#[test]
fn test_signal_evidence_all_sources() {
    let sources = vec![
        SignalSource::Rustc,
        SignalSource::Clippy,
        SignalSource::CargoTest,
        SignalSource::CargoBuild,
        SignalSource::LlvmCov,
        SignalSource::CargoMutants,
        SignalSource::PmatTdg,
        SignalSource::PmatComplexity,
        SignalSource::PmatSatd,
        SignalSource::PmatDeadCode,
        SignalSource::PmatRustProjectScore,
        SignalSource::PmatFiveWhys,
        SignalSource::PmatChurn,
    ];

    for source in sources {
        let signal = SignalEvidence {
            source,
            raw_message: "test".to_string(),
            error_code: None,
            weight: 1.0,
        };
        assert_eq!(signal.source, source);
    }
}

// ============ signals_to_defects Tests ============

#[test]
fn test_signals_to_defects_empty_signals() {
    let collector = AggregatedCollector::new();
    let signals: Vec<SignalEvidence> = vec![];
    let defects = collector.signals_to_defects(signals);
    assert!(defects.is_empty());
}

#[test]
fn test_signals_to_defects_single_rustc_error() {
    let collector = AggregatedCollector::new();
    let signals = vec![SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error[E0308]: mismatched types".to_string(),
        error_code: Some("E0308".to_string()),
        weight: 1.0,
    }];
    let defects = collector.signals_to_defects(signals);
    assert_eq!(defects.len(), 1);
    assert_eq!(defects[0].severity, Severity::Critical);
    assert_eq!(defects[0].category, DefectCategory::TypeErrors);
}

#[test]
fn test_signals_to_defects_clippy_warning() {
    let collector = AggregatedCollector::new();
    let signals = vec![SignalEvidence {
        source: SignalSource::Clippy,
        raw_message: "warning: unused variable".to_string(),
        error_code: Some("clippy::unused_variable".to_string()),
        weight: 0.5,
    }];
    let defects = collector.signals_to_defects(signals);
    assert_eq!(defects.len(), 1);
    assert_eq!(defects[0].severity, Severity::Medium);
}

#[test]
fn test_signals_to_defects_test_failure() {
    let collector = AggregatedCollector::new();
    let signals = vec![SignalEvidence {
        source: SignalSource::CargoTest,
        raw_message: "Test failed: test_something".to_string(),
        error_code: None,
        weight: 1.0,
    }];
    let defects = collector.signals_to_defects(signals);
    assert_eq!(defects.len(), 1);
    assert_eq!(defects[0].severity, Severity::High);
}

#[test]
fn test_signals_to_defects_other_source_low_severity() {
    let collector = AggregatedCollector::new();
    let signals = vec![SignalEvidence {
        source: SignalSource::PmatSatd,
        raw_message: "TODO found".to_string(),
        error_code: None,
        weight: 0.3,
    }];
    let defects = collector.signals_to_defects(signals);
    assert_eq!(defects.len(), 1);
    assert_eq!(defects[0].severity, Severity::Low);
}

#[test]
fn test_signals_to_defects_multiple_signals_creates_multiple_defects() {
    let collector = AggregatedCollector::new();
    let signals = vec![
        SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error".to_string(),
            error_code: Some("E0308".to_string()),
            weight: 1.0,
        },
        SignalEvidence {
            source: SignalSource::Clippy,
            raw_message: "warning".to_string(),
            error_code: None,
            weight: 0.5,
        },
        SignalEvidence {
            source: SignalSource::CargoTest,
            raw_message: "test failed".to_string(),
            error_code: None,
            weight: 1.0,
        },
    ];
    let defects = collector.signals_to_defects(signals);
    assert_eq!(defects.len(), 3);
}

#[test]
fn test_signals_to_defects_ownership_error() {
    let collector = AggregatedCollector::new();
    let signals = vec![SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error[E0382]: borrow of moved value".to_string(),
        error_code: Some("E0382".to_string()),
        weight: 1.0,
    }];
    let defects = collector.signals_to_defects(signals);
    assert_eq!(defects.len(), 1);
    assert_eq!(defects[0].category, DefectCategory::OwnershipBorrow);
}

#[test]
fn test_signals_to_defects_memory_safety_error() {
    let collector = AggregatedCollector::new();
    let signals = vec![SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error[E0507]: cannot move out of".to_string(),
        error_code: Some("E0507".to_string()),
        weight: 1.0,
    }];
    let defects = collector.signals_to_defects(signals);
    assert_eq!(defects.len(), 1);
    assert_eq!(defects[0].category, DefectCategory::MemorySafety);
}

#[test]
fn test_signals_to_defects_trait_bounds_error() {
    let collector = AggregatedCollector::new();
    let signals = vec![SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error[E0277]: the trait bound".to_string(),
        error_code: Some("E0277".to_string()),
        weight: 1.0,
    }];
    let defects = collector.signals_to_defects(signals);
    assert_eq!(defects.len(), 1);
    assert_eq!(defects[0].category, DefectCategory::TraitBounds);
}

#[test]
fn test_signals_to_defects_unknown_error_code_uses_configuration() {
    let collector = AggregatedCollector::new();
    let signals = vec![SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error[E9999]: unknown".to_string(),
        error_code: Some("E9999".to_string()),
        weight: 1.0,
    }];
    let defects = collector.signals_to_defects(signals);
    assert_eq!(defects.len(), 1);
    assert_eq!(defects[0].category, DefectCategory::Configuration);
}

#[test]
fn test_signals_to_defects_no_error_code_uses_configuration() {
    let collector = AggregatedCollector::new();
    let signals = vec![SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error: something".to_string(),
        error_code: None,
        weight: 1.0,
    }];
    let defects = collector.signals_to_defects(signals);
    assert_eq!(defects.len(), 1);
    assert_eq!(defects[0].category, DefectCategory::Configuration);
}

#[test]
fn test_signals_to_defects_preserves_signal_in_defect() {
    let collector = AggregatedCollector::new();
    let signals = vec![SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error test".to_string(),
        error_code: Some("E0308".to_string()),
        weight: 0.8,
    }];
    let defects = collector.signals_to_defects(signals);
    assert_eq!(defects[0].signals.len(), 1);
    assert_eq!(defects[0].signals[0].raw_message, "error test");
}

// ============ DefectCategory Tests ============

#[test]
fn test_defect_category_from_rustc_all_ownership_codes() {
    let ownership_codes = ["E0382", "E0502", "E0503", "E0505", "E0499", "E0597", "E0716", "E0515"];
    for code in ownership_codes {
        assert_eq!(
            DefectCategory::from_rustc_error(code),
            Some(DefectCategory::OwnershipBorrow),
            "Code {} should map to OwnershipBorrow",
            code
        );
    }
}

#[test]
fn test_defect_category_from_rustc_ast_transform() {
    assert_eq!(
        DefectCategory::from_rustc_error("E0599"),
        Some(DefectCategory::ASTTransform)
    );
}

#[test]
fn test_defect_category_from_rustc_operator_precedence() {
    assert_eq!(
        DefectCategory::from_rustc_error("E0615"),
        Some(DefectCategory::OperatorPrecedence)
    );
}

#[test]
fn test_defect_category_from_rustc_configuration() {
    assert_eq!(
        DefectCategory::from_rustc_error("E0658"),
        Some(DefectCategory::Configuration)
    );
}

#[test]
fn test_defect_category_confidence_all_categories() {
    // Test all category confidences
    assert!((DefectCategory::TypeErrors.rustc_confidence() - 0.95).abs() < f32::EPSILON);
    assert!((DefectCategory::OwnershipBorrow.rustc_confidence() - 0.92).abs() < f32::EPSILON);
    assert!((DefectCategory::MemorySafety.rustc_confidence() - 0.90).abs() < f32::EPSILON);
    assert!((DefectCategory::TraitBounds.rustc_confidence() - 0.95).abs() < f32::EPSILON);
    assert!((DefectCategory::StdlibMapping.rustc_confidence() - 0.85).abs() < f32::EPSILON);
    assert!((DefectCategory::ASTTransform.rustc_confidence() - 0.85).abs() < f32::EPSILON);
    assert!((DefectCategory::OperatorPrecedence.rustc_confidence() - 0.80).abs() < f32::EPSILON);
    assert!((DefectCategory::Configuration.rustc_confidence() - 0.75).abs() < f32::EPSILON);
    // Default confidence for other categories
    assert!((DefectCategory::Concurrency.rustc_confidence() - 0.70).abs() < f32::EPSILON);
    assert!((DefectCategory::PerformanceIssues.rustc_confidence() - 0.70).abs() < f32::EPSILON);
    assert!((DefectCategory::Security.rustc_confidence() - 0.70).abs() < f32::EPSILON);
}

// ============ CodeLocation Tests ============

#[test]
fn test_code_location_with_all_fields() {
    let location = CodeLocation {
        file_path: PathBuf::from("/src/main.rs"),
        line: 42,
        column: Some(10),
        span_end_line: Some(50),
    };
    assert_eq!(location.file_path, PathBuf::from("/src/main.rs"));
    assert_eq!(location.line, 42);
    assert_eq!(location.column, Some(10));
    assert_eq!(location.span_end_line, Some(50));
}

#[test]
fn test_code_location_minimal() {
    let location = CodeLocation {
        file_path: PathBuf::from("test.rs"),
        line: 1,
        column: None,
        span_end_line: None,
    };
    assert_eq!(location.line, 1);
    assert!(location.column.is_none());
}

#[test]
fn test_code_location_clone() {
    let location = CodeLocation {
        file_path: PathBuf::from("test.rs"),
        line: 10,
        column: Some(5),
        span_end_line: None,
    };
    let cloned = location.clone();
    assert_eq!(location.file_path, cloned.file_path);
    assert_eq!(location.line, cloned.line);
}

// ============ DefectReport Tests ============

#[test]
fn test_defect_report_new_has_uuid() {
    let location = CodeLocation {
        file_path: PathBuf::from("test.rs"),
        line: 1,
        column: None,
        span_end_line: None,
    };
    let report = DefectReport::new(DefectCategory::TypeErrors, Severity::High, location);
    assert!(!report.id.is_empty());
    // UUID should have standard format
    assert!(report.id.len() >= 32);
}

#[test]
fn test_defect_report_new_initial_state() {
    let location = CodeLocation {
        file_path: PathBuf::from("test.rs"),
        line: 1,
        column: None,
        span_end_line: None,
    };
    let report = DefectReport::new(DefectCategory::TypeErrors, Severity::High, location);
    assert_eq!(report.confidence, 0.0);
    assert!(report.signals.is_empty());
    assert!(report.suggested_fixes.is_empty());
    assert_eq!(report.decision, OracleDecision::Skip);
}

#[test]
fn test_defect_report_add_signal_recalculates_confidence() {
    let location = CodeLocation {
        file_path: PathBuf::from("test.rs"),
        line: 1,
        column: None,
        span_end_line: None,
    };
    let mut report = DefectReport::new(DefectCategory::TypeErrors, Severity::High, location);
    assert_eq!(report.confidence, 0.0);

    report.add_signal(SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error".to_string(),
        error_code: None,
        weight: 1.0,
    });

    // TypeErrors confidence (0.95) * weight (1.0) = 0.95
    assert!((report.confidence - 0.95).abs() < 0.01);
}

#[test]
fn test_defect_report_add_multiple_signals_uses_max_weight() {
    let location = CodeLocation {
        file_path: PathBuf::from("test.rs"),
        line: 1,
        column: None,
        span_end_line: None,
    };
    let mut report = DefectReport::new(DefectCategory::TypeErrors, Severity::High, location);

    report.add_signal(SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error".to_string(),
        error_code: None,
        weight: 0.5,
    });

    report.add_signal(SignalEvidence {
        source: SignalSource::Clippy,
        raw_message: "warning".to_string(),
        error_code: None,
        weight: 0.8,
    });

    // Max weight is 0.8, so confidence = 0.95 * 0.8 = 0.76
    assert!((report.confidence - 0.76).abs() < 0.01);
}

#[test]
fn test_defect_report_update_decision_auto_apply() {
    let location = CodeLocation {
        file_path: PathBuf::from("test.rs"),
        line: 1,
        column: None,
        span_end_line: None,
    };
    let mut report = DefectReport::new(DefectCategory::TypeErrors, Severity::Critical, location);
    report.add_signal(SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error".to_string(),
        error_code: None,
        weight: 1.0,
    });

    report.update_decision(0.9, 0.7);
    assert_eq!(report.decision, OracleDecision::AutoApply);
}

#[test]
fn test_defect_report_update_decision_human_review() {
    let location = CodeLocation {
        file_path: PathBuf::from("test.rs"),
        line: 1,
        column: None,
        span_end_line: None,
    };
    let mut report = DefectReport::new(DefectCategory::Configuration, Severity::Medium, location);
    report.add_signal(SignalEvidence {
        source: SignalSource::Clippy,
        raw_message: "warning".to_string(),
        error_code: None,
        weight: 1.0,
    });

    // Configuration confidence (0.75) * weight (1.0) = 0.75
    report.update_decision(0.9, 0.7);
    assert_eq!(report.decision, OracleDecision::HumanReview);
}

#[test]
fn test_defect_report_update_decision_skip() {
    let location = CodeLocation {
        file_path: PathBuf::from("test.rs"),
        line: 1,
        column: None,
        span_end_line: None,
    };
    let mut report = DefectReport::new(DefectCategory::Configuration, Severity::Low, location);
    report.add_signal(SignalEvidence {
        source: SignalSource::Clippy,
        raw_message: "warning".to_string(),
        error_code: None,
        weight: 0.5,
    });

    // Configuration confidence (0.75) * weight (0.5) = 0.375
    report.update_decision(0.9, 0.7);
    assert_eq!(report.decision, OracleDecision::Skip);
}

// ============ Severity Tests ============

#[test]
fn test_severity_ordering_low_medium() {
    assert!(Severity::Low < Severity::Medium);
}

#[test]
fn test_severity_ordering_medium_high() {
    assert!(Severity::Medium < Severity::High);
}

#[test]
fn test_severity_ordering_high_critical() {
    assert!(Severity::High < Severity::Critical);
}

#[test]
fn test_severity_ordering_low_critical() {
    assert!(Severity::Low < Severity::Critical);
}

#[test]
fn test_severity_equality() {
    assert_eq!(Severity::Low, Severity::Low);
    assert_eq!(Severity::Medium, Severity::Medium);
    assert_eq!(Severity::High, Severity::High);
    assert_eq!(Severity::Critical, Severity::Critical);
}

// ============ FixType Tests ============

#[test]
fn test_fix_type_clippy_auto_fix() {
    let fix = SuggestedFix {
        description: "Apply clippy fix".to_string(),
        confidence: 0.95,
        fix_type: FixType::ClippyAutoFix,
    };
    assert!(matches!(fix.fix_type, FixType::ClippyAutoFix));
}

#[test]
fn test_fix_type_diff_patch() {
    let fix = SuggestedFix {
        description: "Apply patch".to_string(),
        confidence: 0.8,
        fix_type: FixType::DiffPatch("--- a/file\n+++ b/file\n@@ -1,1 +1,1 @@\n-old\n+new".to_string()),
    };
    if let FixType::DiffPatch(patch) = &fix.fix_type {
        assert!(patch.contains("---"));
        assert!(patch.contains("+++"));
    } else {
        panic!("Expected DiffPatch");
    }
}

#[test]
fn test_fix_type_replacement() {
    let fix = SuggestedFix {
        description: "Replace code".to_string(),
        confidence: 0.9,
        fix_type: FixType::Replacement {
            old: "let x = 1;".to_string(),
            new: "let x: i32 = 1;".to_string(),
        },
    };
    if let FixType::Replacement { old, new } = &fix.fix_type {
        assert_eq!(old, "let x = 1;");
        assert!(new.contains("i32"));
    } else {
        panic!("Expected Replacement");
    }
}

#[test]
fn test_fix_type_insert_after() {
    let fix = SuggestedFix {
        description: "Insert after".to_string(),
        confidence: 0.7,
        fix_type: FixType::InsertAfter {
            anchor: "fn main()".to_string(),
            content: "let config = Config::new();".to_string(),
        },
    };
    if let FixType::InsertAfter { anchor, content } = &fix.fix_type {
        assert_eq!(anchor, "fn main()");
        assert!(content.contains("Config"));
    } else {
        panic!("Expected InsertAfter");
    }
}

#[test]
fn test_fix_type_delete_lines() {
    let fix = SuggestedFix {
        description: "Delete lines".to_string(),
        confidence: 0.85,
        fix_type: FixType::DeleteLines { start: 10, end: 15 },
    };
    if let FixType::DeleteLines { start, end } = &fix.fix_type {
        assert_eq!(*start, 10);
        assert_eq!(*end, 15);
    } else {
        panic!("Expected DeleteLines");
    }
}

// ============ OracleDecision Tests ============

#[test]
fn test_oracle_decision_equality() {
    assert_eq!(OracleDecision::AutoApply, OracleDecision::AutoApply);
    assert_eq!(OracleDecision::HumanReview, OracleDecision::HumanReview);
    assert_eq!(OracleDecision::Skip, OracleDecision::Skip);
}

#[test]
fn test_oracle_decision_inequality() {
    assert_ne!(OracleDecision::AutoApply, OracleDecision::Skip);
    assert_ne!(OracleDecision::HumanReview, OracleDecision::AutoApply);
}

// ============ ConvergenceTargets Tests ============

#[test]
fn test_convergence_targets_default_values() {
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
    match targets.check(&metrics) {
        ConvergenceStatus::Converged => (),
        ConvergenceStatus::NotConverged { remaining } => {
            panic!("Expected converged, got: {:?}", remaining)
        }
    }
}

#[test]
fn test_convergence_targets_check_not_converged_coverage() {
    let targets = ConvergenceTargets::default();
    let metrics = ProjectMetrics {
        test_coverage: 0.80, // Below 0.95
        ..Default::default()
    };
    match targets.check(&metrics) {
        ConvergenceStatus::Converged => panic!("Expected not converged"),
        ConvergenceStatus::NotConverged { remaining } => {
            assert!(remaining.iter().any(|s| s.contains("Coverage")));
        }
    }
}

#[test]
fn test_convergence_targets_check_not_converged_compiler_errors() {
    let targets = ConvergenceTargets::default();
    let metrics = ProjectMetrics {
        test_coverage: 0.96,
        mutation_score: 0.86,
        compiler_errors: 5, // Above 0
        ..Default::default()
    };
    match targets.check(&metrics) {
        ConvergenceStatus::Converged => panic!("Expected not converged"),
        ConvergenceStatus::NotConverged { remaining } => {
            assert!(remaining.iter().any(|s| s.contains("Compiler errors")));
        }
    }
}

#[test]
fn test_convergence_targets_check_multiple_failures() {
    let targets = ConvergenceTargets::default();
    let metrics = ProjectMetrics {
        test_coverage: 0.50,
        mutation_score: 0.50,
        compiler_errors: 10,
        clippy_warnings: 20,
        test_failures: 5,
        ..Default::default()
    };
    match targets.check(&metrics) {
        ConvergenceStatus::Converged => panic!("Expected not converged"),
        ConvergenceStatus::NotConverged { remaining } => {
            assert!(remaining.len() >= 5);
        }
    }
}

// ============ OracleConfig Tests ============

#[test]
fn test_oracle_config_default_values() {
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
fn test_oracle_config_custom_values() {
    let config = OracleConfig {
        max_iterations: 50,
        min_progress_per_iteration: 0.01,
        stagnation_threshold: 3,
        andon_enabled: false,
        require_human_approval_above: None,
        auto_apply_threshold: 0.95,
        review_threshold: 0.6,
        batch_size: 5,
    };
    assert_eq!(config.max_iterations, 50);
    assert!(!config.andon_enabled);
    assert!(config.require_human_approval_above.is_none());
}

// ============ ProjectMetrics Tests ============

#[test]
fn test_project_metrics_default() {
    let metrics = ProjectMetrics::default();
    assert_eq!(metrics.test_coverage, 0.0);
    assert_eq!(metrics.mutation_score, 0.0);
    assert_eq!(metrics.compiler_errors, 0);
    assert_eq!(metrics.clippy_warnings, 0);
    assert_eq!(metrics.test_failures, 0);
    assert_eq!(metrics.tdg_score, 0.0);
    assert_eq!(metrics.rust_project_score, 0);
    assert_eq!(metrics.satd_markers, 0);
    assert_eq!(metrics.dead_code_items, 0);
    assert_eq!(metrics.max_cyclomatic_complexity, 0);
    assert_eq!(metrics.max_cognitive_complexity, 0);
    assert_eq!(metrics.build_time, Duration::default());
}

#[test]
fn test_project_metrics_custom() {
    let metrics = ProjectMetrics {
        test_coverage: 0.92,
        mutation_score: 0.88,
        compiler_errors: 1,
        clippy_warnings: 3,
        test_failures: 0,
        tdg_score: 88.5,
        rust_project_score: 85,
        satd_markers: 5,
        dead_code_items: 2,
        max_cyclomatic_complexity: 18,
        max_cognitive_complexity: 22,
        build_time: Duration::from_secs(45),
    };
    assert_eq!(metrics.test_coverage, 0.92);
    assert_eq!(metrics.rust_project_score, 85);
}

// ============ Serialization Round-trip Tests ============

#[test]
fn test_signal_evidence_serialization_roundtrip() {
    let signal = SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "error[E0308]".to_string(),
        error_code: Some("E0308".to_string()),
        weight: 0.95,
    };
    let json = serde_json::to_string(&signal).expect("Should serialize");
    let parsed: SignalEvidence = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(signal.source, parsed.source);
    assert_eq!(signal.error_code, parsed.error_code);
}

#[test]
fn test_code_location_serialization_roundtrip() {
    let location = CodeLocation {
        file_path: PathBuf::from("src/lib.rs"),
        line: 100,
        column: Some(15),
        span_end_line: Some(105),
    };
    let json = serde_json::to_string(&location).expect("Should serialize");
    let parsed: CodeLocation = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(location.file_path, parsed.file_path);
    assert_eq!(location.line, parsed.line);
}

#[test]
fn test_convergence_targets_serialization_roundtrip() {
    let targets = ConvergenceTargets::default();
    let json = serde_json::to_string(&targets).expect("Should serialize");
    let parsed: ConvergenceTargets = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(targets.test_coverage, parsed.test_coverage);
    assert_eq!(targets.max_compiler_errors, parsed.max_compiler_errors);
}

#[test]
fn test_oracle_config_serialization_roundtrip() {
    let config = OracleConfig::default();
    let json = serde_json::to_string(&config).expect("Should serialize");
    let parsed: OracleConfig = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(config.max_iterations, parsed.max_iterations);
    assert_eq!(config.batch_size, parsed.batch_size);
}

#[test]
fn test_project_metrics_serialization_roundtrip() {
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
    let json = serde_json::to_string(&metrics).expect("Should serialize");
    let parsed: ProjectMetrics = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(metrics.test_coverage, parsed.test_coverage);
    assert_eq!(metrics.rust_project_score, parsed.rust_project_score);
}

#[test]
fn test_defect_category_serialization_all_variants() {
    let categories = vec![
        DefectCategory::MemorySafety,
        DefectCategory::Concurrency,
        DefectCategory::OwnershipBorrow,
        DefectCategory::TypeErrors,
        DefectCategory::TypeAnnotationGap,
        DefectCategory::TraitBounds,
        DefectCategory::OperatorPrecedence,
        DefectCategory::PerformanceIssues,
        DefectCategory::Security,
        DefectCategory::Configuration,
        DefectCategory::ApiMisuse,
        DefectCategory::IntegrationFailure,
        DefectCategory::StdlibMapping,
        DefectCategory::DocumentationGap,
        DefectCategory::TestingGap,
        DefectCategory::ASTTransform,
        DefectCategory::ComprehensionBug,
        DefectCategory::IteratorChain,
    ];

    for category in categories {
        let json = serde_json::to_string(&category).expect("Should serialize");
        let parsed: DefectCategory = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(category, parsed);
    }
}

// ============ Edge Case Tests ============

#[test]
fn test_signal_evidence_empty_message() {
    let signal = SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: String::new(),
        error_code: None,
        weight: 0.0,
    };
    assert!(signal.raw_message.is_empty());
}

#[test]
fn test_code_location_zero_line() {
    let location = CodeLocation {
        file_path: PathBuf::from("test.rs"),
        line: 0,
        column: None,
        span_end_line: None,
    };
    assert_eq!(location.line, 0);
}

#[test]
fn test_defect_report_empty_signals_zero_confidence() {
    let location = CodeLocation {
        file_path: PathBuf::from("test.rs"),
        line: 1,
        column: None,
        span_end_line: None,
    };
    let report = DefectReport::new(DefectCategory::TypeErrors, Severity::High, location);
    assert_eq!(report.confidence, 0.0);
}

#[test]
fn test_convergence_check_all_metrics_failing() {
    let targets = ConvergenceTargets::default();
    let metrics = ProjectMetrics {
        test_coverage: 0.0,
        mutation_score: 0.0,
        compiler_errors: 100,
        clippy_warnings: 100,
        test_failures: 100,
        tdg_score: 0.0,
        rust_project_score: 0,
        satd_markers: 100,
        dead_code_items: 100,
        max_cyclomatic_complexity: 100,
        max_cognitive_complexity: 100,
        build_time: Duration::from_secs(1000),
    };
    match targets.check(&metrics) {
        ConvergenceStatus::Converged => panic!("Should not converge"),
        ConvergenceStatus::NotConverged { remaining } => {
            // All metrics should fail except build_time which isn't currently checked
            assert!(remaining.len() >= 10);
        }
    }
}
