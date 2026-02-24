    // ========================================================================
    // NestingStrategy Tests
    // ========================================================================

    #[test]
    fn test_nesting_strategy_variants() {
        let strategies = vec![
            NestingStrategy::EarlyReturn,
            NestingStrategy::ExtractCondition,
            NestingStrategy::GuardClause,
            NestingStrategy::StreamChain,
        ];

        for strategy in strategies {
            let json = serde_json::to_string(&strategy).unwrap();
            let _: NestingStrategy = serde_json::from_str(&json).unwrap();
        }
    }

    // ========================================================================
    // ViolationType Tests
    // ========================================================================

    #[test]
    fn test_violation_type_variants() {
        let types = vec![
            ViolationType::HighComplexity,
            ViolationType::DeepNesting,
            ViolationType::LongFunction,
            ViolationType::SelfAdmittedTechDebt,
            ViolationType::DeadCode,
            ViolationType::PoorNaming,
        ];

        for vtype in types {
            let json = serde_json::to_string(&vtype).unwrap();
            let _: ViolationType = serde_json::from_str(&json).unwrap();
        }
    }

    // ========================================================================
    // Severity Tests
    // ========================================================================

    #[test]
    fn test_severity_variants() {
        let severities = vec![
            Severity::Low,
            Severity::Medium,
            Severity::High,
            Severity::Critical,
        ];

        for severity in severities {
            let json = serde_json::to_string(&severity).unwrap();
            let _: Severity = serde_json::from_str(&json).unwrap();
        }
    }

    // ========================================================================
    // Violation Tests
    // ========================================================================

    #[test]
    fn test_violation_to_op_with_suggested_fix() {
        let violation = Violation {
            violation_type: ViolationType::HighComplexity,
            location: Location {
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 1,
            },
            severity: Severity::High,
            description: "Test violation".to_string(),
            suggested_fix: Some(RefactorOp::SimplifyExpression {
                expr: "x".to_string(),
                simplified: "y".to_string(),
            }),
        };

        let op = violation.to_op();
        if let RefactorOp::SimplifyExpression { expr, .. } = op {
            assert_eq!(expr, "x");
        } else {
            panic!("Expected SimplifyExpression from suggested_fix");
        }
    }

    #[test]
    fn test_violation_to_op_high_complexity() {
        let violation = Violation {
            violation_type: ViolationType::HighComplexity,
            location: Location {
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 1,
            },
            severity: Severity::High,
            description: "Test".to_string(),
            suggested_fix: None,
        };

        let op = violation.to_op();
        if let RefactorOp::ExtractFunction { .. } = op {
            // Expected
        } else {
            panic!("Expected ExtractFunction for HighComplexity");
        }
    }

    #[test]
    fn test_violation_to_op_deep_nesting() {
        let violation = Violation {
            violation_type: ViolationType::DeepNesting,
            location: Location {
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 1,
            },
            severity: Severity::Medium,
            description: "Test".to_string(),
            suggested_fix: None,
        };

        let op = violation.to_op();
        if let RefactorOp::FlattenNesting { .. } = op {
            // Expected
        } else {
            panic!("Expected FlattenNesting for DeepNesting");
        }
    }

    #[test]
    fn test_violation_to_op_satd() {
        let violation = Violation {
            violation_type: ViolationType::SelfAdmittedTechDebt,
            location: Location {
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 1,
            },
            severity: Severity::Low,
            description: "Test".to_string(),
            suggested_fix: None,
        };

        let op = violation.to_op();
        if let RefactorOp::RemoveSatd { .. } = op {
            // Expected
        } else {
            panic!("Expected RemoveSatd for SelfAdmittedTechDebt");
        }
    }

    // ========================================================================
    // SatdFix Tests
    // ========================================================================

    #[test]
    fn test_satd_fix_variants() {
        let fixes = vec![
            SatdFix::Remove,
            SatdFix::Replace {
                with: "fixed".to_string(),
            },
            SatdFix::Implement {
                solution: "impl".to_string(),
            },
        ];

        for fix in fixes {
            let json = serde_json::to_string(&fix).unwrap();
            let _: SatdFix = serde_json::from_str(&json).unwrap();
        }
    }

    // ========================================================================
    // RefactorType Tests
    // ========================================================================

    #[test]
    fn test_refactor_type_values() {
        assert_eq!(RefactorType::None as u8, 0);
        assert_eq!(RefactorType::ExtractFunction as u8, 1);
        assert_eq!(RefactorType::FlattenNesting as u8, 2);
        assert_eq!(RefactorType::SimplifyLogic as u8, 3);
        assert_eq!(RefactorType::RemoveDeadCode as u8, 4);
    }

    // ========================================================================
    // DefectPayload Tests
    // ========================================================================

    #[test]
    fn test_defect_payload_creation() {
        let payload = DefectPayload {
            file_hash: 12345,
            tdg_score: 1.5,
            complexity: (10, 15),
            dead_symbols: 2,
            timestamp: 1000,
            severity_flags: 0b0101,
            refactor_available: true,
            refactor_type: RefactorType::ExtractFunction,
            estimated_improvement: 0.3,
            _padding: [0; 2],
        };

        assert_eq!(payload.file_hash, 12345);
        assert_eq!(payload.complexity, (10, 15));
        assert!(payload.refactor_available);
    }

    // ========================================================================
    // BytePos Tests
    // ========================================================================

    #[test]
    fn test_byte_pos() {
        let pos = BytePos {
            byte: 1000,
            line: 50,
            column: 10,
        };

        assert_eq!(pos.byte, 1000);
        assert_eq!(pos.line, 50);
        assert_eq!(pos.column, 10);
    }

    // ========================================================================
    // Location Tests
    // ========================================================================

    #[test]
    fn test_location() {
        let loc = Location {
            file: PathBuf::from("/path/to/file.rs"),
            line: 100,
            column: 5,
        };

        assert_eq!(loc.file, PathBuf::from("/path/to/file.rs"));
        assert_eq!(loc.line, 100);
        assert_eq!(loc.column, 5);
    }

    // ========================================================================
    // FileId Tests
    // ========================================================================

    #[test]
    fn test_file_id() {
        let file_id = FileId {
            path: PathBuf::from("src/main.rs"),
            hash: 0xDEADBEEF,
        };

        assert_eq!(file_id.path, PathBuf::from("src/main.rs"));
        assert_eq!(file_id.hash, 0xDEADBEEF);
    }
