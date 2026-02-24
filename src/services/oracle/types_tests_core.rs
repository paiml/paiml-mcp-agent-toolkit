// ==================== DefectCategory Tests ====================

#[test]
fn test_defect_category_from_rustc_error_type_errors() {
    assert_eq!(
        DefectCategory::from_rustc_error("E0308"),
        Some(DefectCategory::TypeErrors)
    );
    assert_eq!(
        DefectCategory::from_rustc_error("E0412"),
        Some(DefectCategory::TypeErrors)
    );
}

#[test]
fn test_defect_category_from_rustc_error_ownership_borrow() {
    for code in [
        "E0382", "E0502", "E0503", "E0505", "E0499", "E0597", "E0716", "E0515",
    ] {
        assert_eq!(
            DefectCategory::from_rustc_error(code),
            Some(DefectCategory::OwnershipBorrow),
            "Code {} should map to OwnershipBorrow",
            code
        );
    }
}

#[test]
fn test_defect_category_from_rustc_error_memory_safety() {
    assert_eq!(
        DefectCategory::from_rustc_error("E0507"),
        Some(DefectCategory::MemorySafety)
    );
    assert_eq!(
        DefectCategory::from_rustc_error("E0133"),
        Some(DefectCategory::MemorySafety)
    );
}

#[test]
fn test_defect_category_from_rustc_error_trait_bounds() {
    assert_eq!(
        DefectCategory::from_rustc_error("E0277"),
        Some(DefectCategory::TraitBounds)
    );
}

#[test]
fn test_defect_category_from_rustc_error_stdlib_mapping() {
    assert_eq!(
        DefectCategory::from_rustc_error("E0425"),
        Some(DefectCategory::StdlibMapping)
    );
    assert_eq!(
        DefectCategory::from_rustc_error("E0433"),
        Some(DefectCategory::StdlibMapping)
    );
}

#[test]
fn test_defect_category_from_rustc_error_unknown() {
    assert_eq!(DefectCategory::from_rustc_error("E9999"), None);
    assert_eq!(DefectCategory::from_rustc_error("unknown"), None);
}

#[test]
fn test_defect_category_rustc_confidence() {
    assert_eq!(DefectCategory::TypeErrors.rustc_confidence(), 0.95);
    assert_eq!(DefectCategory::OwnershipBorrow.rustc_confidence(), 0.92);
    assert_eq!(DefectCategory::MemorySafety.rustc_confidence(), 0.90);
    assert_eq!(DefectCategory::TraitBounds.rustc_confidence(), 0.95);
    assert_eq!(DefectCategory::StdlibMapping.rustc_confidence(), 0.85);
    assert_eq!(DefectCategory::ASTTransform.rustc_confidence(), 0.85);
    assert_eq!(DefectCategory::OperatorPrecedence.rustc_confidence(), 0.80);
    assert_eq!(DefectCategory::Configuration.rustc_confidence(), 0.75);
    // Default for other categories
    assert_eq!(DefectCategory::Concurrency.rustc_confidence(), 0.70);
    assert_eq!(DefectCategory::PerformanceIssues.rustc_confidence(), 0.70);
}

// ==================== Severity Tests ====================

#[test]
fn test_severity_ordering() {
    assert!(Severity::Low < Severity::Medium);
    assert!(Severity::Medium < Severity::High);
    assert!(Severity::High < Severity::Critical);
}

#[test]
fn test_severity_equality() {
    assert_eq!(Severity::Low, Severity::Low);
    assert_ne!(Severity::Low, Severity::High);
}

#[test]
fn test_severity_serialization() {
    let severity = Severity::High;
    let json = serde_json::to_string(&severity).unwrap();
    let parsed: Severity = serde_json::from_str(&json).unwrap();
    assert_eq!(severity, parsed);
}

// ==================== SignalSource Tests ====================

#[test]
fn test_signal_source_serialization() {
    let sources = [
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
        let json = serde_json::to_string(&source).unwrap();
        let parsed: SignalSource = serde_json::from_str(&json).unwrap();
        assert_eq!(source, parsed);
    }
}

// ==================== SignalEvidence Tests ====================

#[test]
fn test_signal_evidence_creation() {
    let evidence = SignalEvidence {
        source: SignalSource::Rustc,
        raw_message: "type mismatch".to_string(),
        error_code: Some("E0308".to_string()),
        weight: 0.95,
    };

    assert_eq!(evidence.source, SignalSource::Rustc);
    assert!(evidence.error_code.is_some());
    assert_eq!(evidence.weight, 0.95);
}

#[test]
fn test_signal_evidence_serialization() {
    let evidence = SignalEvidence {
        source: SignalSource::Clippy,
        raw_message: "warning".to_string(),
        error_code: None,
        weight: 0.8,
    };

    let json = serde_json::to_string(&evidence).unwrap();
    let parsed: SignalEvidence = serde_json::from_str(&json).unwrap();

    assert_eq!(evidence.source, parsed.source);
    assert_eq!(evidence.weight, parsed.weight);
}

// ==================== CodeLocation Tests ====================

#[test]
fn test_code_location_creation() {
    let location = CodeLocation {
        file_path: PathBuf::from("src/main.rs"),
        line: 42,
        column: Some(10),
        span_end_line: Some(45),
    };

    assert_eq!(location.line, 42);
    assert_eq!(location.column, Some(10));
}

#[test]
fn test_code_location_serialization() {
    let location = CodeLocation {
        file_path: PathBuf::from("test.rs"),
        line: 1,
        column: None,
        span_end_line: None,
    };

    let json = serde_json::to_string(&location).unwrap();
    let parsed: CodeLocation = serde_json::from_str(&json).unwrap();

    assert_eq!(location.file_path, parsed.file_path);
    assert_eq!(location.line, parsed.line);
}

// ==================== FixType Tests ====================

#[test]
fn test_fix_type_clippy_auto() {
    let fix = FixType::ClippyAutoFix;
    let json = serde_json::to_string(&fix).unwrap();
    assert!(json.contains("ClippyAutoFix"));
}

#[test]
fn test_fix_type_diff_patch() {
    let fix = FixType::DiffPatch("--- a/file\n+++ b/file".to_string());
    let json = serde_json::to_string(&fix).unwrap();
    assert!(json.contains("DiffPatch"));
}

#[test]
fn test_fix_type_replacement() {
    let fix = FixType::Replacement {
        old: "old_code".to_string(),
        new: "new_code".to_string(),
    };
    let json = serde_json::to_string(&fix).unwrap();
    assert!(json.contains("Replacement"));
}

#[test]
fn test_fix_type_insert_after() {
    let fix = FixType::InsertAfter {
        anchor: "fn main()".to_string(),
        content: "let x = 1;".to_string(),
    };
    let json = serde_json::to_string(&fix).unwrap();
    assert!(json.contains("InsertAfter"));
}

#[test]
fn test_fix_type_delete_lines() {
    let fix = FixType::DeleteLines { start: 10, end: 20 };
    let json = serde_json::to_string(&fix).unwrap();
    assert!(json.contains("DeleteLines"));
}

// ==================== OracleDecision Tests ====================

#[test]
fn test_oracle_decision_serialization() {
    for decision in [
        OracleDecision::AutoApply,
        OracleDecision::HumanReview,
        OracleDecision::Skip,
    ] {
        let json = serde_json::to_string(&decision).unwrap();
        let parsed: OracleDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(decision, parsed);
    }
}
