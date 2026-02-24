// Tests for oracle types: DefectCategory, Severity, CodeLocation, DefectReport, OracleDecision, FixType.
// Included by signal_collector.rs - shares parent module scope.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests_types {
    use super::*;
    use std::path::PathBuf;

    // ==================== DefectCategory Error Code Mapping Tests ====================

    #[test]
    fn test_defect_category_from_type_error() {
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
    fn test_defect_category_from_ownership_error() {
        assert_eq!(
            DefectCategory::from_rustc_error("E0382"),
            Some(DefectCategory::OwnershipBorrow)
        );
        assert_eq!(
            DefectCategory::from_rustc_error("E0502"),
            Some(DefectCategory::OwnershipBorrow)
        );
        assert_eq!(
            DefectCategory::from_rustc_error("E0505"),
            Some(DefectCategory::OwnershipBorrow)
        );
    }

    #[test]
    fn test_defect_category_from_memory_safety_error() {
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
    fn test_defect_category_from_trait_bounds_error() {
        assert_eq!(
            DefectCategory::from_rustc_error("E0277"),
            Some(DefectCategory::TraitBounds)
        );
    }

    #[test]
    fn test_defect_category_from_stdlib_mapping_error() {
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
    fn test_defect_category_from_unknown_error() {
        assert_eq!(DefectCategory::from_rustc_error("E9999"), None);
        assert_eq!(DefectCategory::from_rustc_error(""), None);
        assert_eq!(DefectCategory::from_rustc_error("invalid"), None);
    }

    // ==================== DefectCategory Confidence Tests ====================

    #[test]
    fn test_defect_category_rustc_confidence() {
        assert!((DefectCategory::TypeErrors.rustc_confidence() - 0.95).abs() < f32::EPSILON);
        assert!((DefectCategory::OwnershipBorrow.rustc_confidence() - 0.92).abs() < f32::EPSILON);
        assert!((DefectCategory::MemorySafety.rustc_confidence() - 0.90).abs() < f32::EPSILON);
        assert!((DefectCategory::TraitBounds.rustc_confidence() - 0.95).abs() < f32::EPSILON);
        assert!((DefectCategory::StdlibMapping.rustc_confidence() - 0.85).abs() < f32::EPSILON);
        assert!((DefectCategory::Concurrency.rustc_confidence() - 0.70).abs() < f32::EPSILON);
    }

    // ==================== CodeLocation Tests ====================

    #[test]
    fn test_code_location_creation() {
        let location = CodeLocation {
            file_path: PathBuf::from("/src/main.rs"),
            line: 42,
            column: Some(10),
            span_end_line: Some(45),
        };

        assert_eq!(location.file_path, PathBuf::from("/src/main.rs"));
        assert_eq!(location.line, 42);
        assert_eq!(location.column, Some(10));
        assert_eq!(location.span_end_line, Some(45));
    }

    #[test]
    fn test_code_location_serialization() {
        let location = CodeLocation {
            file_path: PathBuf::from("/src/lib.rs"),
            line: 100,
            column: None,
            span_end_line: None,
        };

        let serialized = serde_json::to_string(&location).expect("Should serialize");
        let deserialized: CodeLocation =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(location.file_path, deserialized.file_path);
        assert_eq!(location.line, deserialized.line);
        assert_eq!(location.column, deserialized.column);
    }

    // ==================== Severity Tests ====================

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    #[test]
    fn test_severity_serialization() {
        let severities = vec![
            Severity::Low,
            Severity::Medium,
            Severity::High,
            Severity::Critical,
        ];

        for severity in severities {
            let serialized = serde_json::to_string(&severity).expect("Should serialize");
            let deserialized: Severity =
                serde_json::from_str(&serialized).expect("Should deserialize");
            assert_eq!(severity, deserialized);
        }
    }

    // ==================== SignalSource Tests ====================

    #[test]
    fn test_signal_source_variants() {
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
            let serialized = serde_json::to_string(&source).expect("Should serialize");
            let deserialized: SignalSource =
                serde_json::from_str(&serialized).expect("Should deserialize");
            assert_eq!(source, deserialized);
        }
    }

    // ==================== DefectReport Tests ====================

    #[test]
    fn test_defect_report_new() {
        let location = CodeLocation {
            file_path: PathBuf::from("/src/main.rs"),
            line: 10,
            column: None,
            span_end_line: None,
        };
        let report = DefectReport::new(DefectCategory::TypeErrors, Severity::High, location);

        assert!(!report.id.is_empty()); // UUID generated
        assert_eq!(report.category, DefectCategory::TypeErrors);
        assert_eq!(report.severity, Severity::High);
        assert!((report.confidence - 0.0).abs() < f32::EPSILON);
        assert!(report.signals.is_empty());
        assert!(report.suggested_fixes.is_empty());
        assert_eq!(report.decision, OracleDecision::Skip);
    }

    #[test]
    fn test_defect_report_add_signal() {
        let location = CodeLocation {
            file_path: PathBuf::from("/src/lib.rs"),
            line: 20,
            column: None,
            span_end_line: None,
        };
        let mut report = DefectReport::new(DefectCategory::TypeErrors, Severity::High, location);

        let signal = SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error".to_string(),
            error_code: Some("E0308".to_string()),
            weight: 1.0,
        };
        report.add_signal(signal);

        assert_eq!(report.signals.len(), 1);
        // Confidence should be category confidence * max signal weight
        // TypeErrors confidence = 0.95, signal weight = 1.0
        assert!((report.confidence - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_defect_report_update_decision_auto_apply() {
        let location = CodeLocation {
            file_path: PathBuf::from("/src/main.rs"),
            line: 1,
            column: None,
            span_end_line: None,
        };
        let mut report =
            DefectReport::new(DefectCategory::TypeErrors, Severity::Critical, location);

        // Add signal to set confidence
        let signal = SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error".to_string(),
            error_code: Some("E0308".to_string()),
            weight: 1.0,
        };
        report.add_signal(signal);

        // Update decision with low thresholds
        report.update_decision(0.9, 0.7);

        // TypeErrors confidence (0.95) >= 0.9, should be AutoApply
        assert_eq!(report.decision, OracleDecision::AutoApply);
    }

    #[test]
    fn test_defect_report_update_decision_human_review() {
        let location = CodeLocation {
            file_path: PathBuf::from("/src/main.rs"),
            line: 1,
            column: None,
            span_end_line: None,
        };
        let mut report =
            DefectReport::new(DefectCategory::Configuration, Severity::Medium, location);

        // Add signal with lower weight
        let signal = SignalEvidence {
            source: SignalSource::Clippy,
            raw_message: "warning".to_string(),
            error_code: None,
            weight: 0.9,
        };
        report.add_signal(signal);

        // Update decision
        report.update_decision(0.9, 0.5);

        // Configuration confidence (0.75) * 0.9 = 0.675, between 0.5 and 0.9
        assert_eq!(report.decision, OracleDecision::HumanReview);
    }

    #[test]
    fn test_defect_report_update_decision_skip() {
        let location = CodeLocation {
            file_path: PathBuf::from("/src/main.rs"),
            line: 1,
            column: None,
            span_end_line: None,
        };
        let mut report = DefectReport::new(DefectCategory::Configuration, Severity::Low, location);

        // Add signal with low weight
        let signal = SignalEvidence {
            source: SignalSource::Clippy,
            raw_message: "warning".to_string(),
            error_code: None,
            weight: 0.3,
        };
        report.add_signal(signal);

        // Update decision with high thresholds
        report.update_decision(0.9, 0.7);

        // Configuration confidence (0.75) * 0.3 = 0.225, below 0.7
        assert_eq!(report.decision, OracleDecision::Skip);
    }

    // ==================== OracleDecision Tests ====================

    #[test]
    fn test_oracle_decision_serialization() {
        let decisions = vec![
            OracleDecision::AutoApply,
            OracleDecision::HumanReview,
            OracleDecision::Skip,
        ];

        for decision in decisions {
            let serialized = serde_json::to_string(&decision).expect("Should serialize");
            let deserialized: OracleDecision =
                serde_json::from_str(&serialized).expect("Should deserialize");
            assert_eq!(decision, deserialized);
        }
    }

    // ==================== FixType Tests ====================

    #[test]
    fn test_fix_type_clippy_auto_fix() {
        let fix = SuggestedFix {
            description: "Apply clippy fix".to_string(),
            confidence: 0.95,
            fix_type: FixType::ClippyAutoFix,
        };

        assert_eq!(fix.description, "Apply clippy fix");
        assert!((fix.confidence - 0.95).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fix_type_replacement() {
        let fix = SuggestedFix {
            description: "Replace old with new".to_string(),
            confidence: 0.8,
            fix_type: FixType::Replacement {
                old: "old_code".to_string(),
                new: "new_code".to_string(),
            },
        };

        if let FixType::Replacement { old, new } = &fix.fix_type {
            assert_eq!(old, "old_code");
            assert_eq!(new, "new_code");
        } else {
            panic!("Expected Replacement fix type");
        }
    }

    #[test]
    fn test_fix_type_serialization() {
        let fix = SuggestedFix {
            description: "Test fix".to_string(),
            confidence: 0.7,
            fix_type: FixType::DiffPatch("@@ -1,3 +1,3 @@\n-old\n+new".to_string()),
        };

        let serialized = serde_json::to_string(&fix).expect("Should serialize");
        let deserialized: SuggestedFix =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(fix.description, deserialized.description);
    }
}
