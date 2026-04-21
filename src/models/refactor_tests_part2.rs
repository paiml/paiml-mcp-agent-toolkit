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

    // Covers the `_` fall-through arm in Violation::to_op for ViolationType
    // variants not explicitly matched (LongFunction, DeadCode, PoorNaming).
    #[test]
    fn test_violation_to_op_fallthrough_variants() {
        for vtype in [
            ViolationType::LongFunction,
            ViolationType::DeadCode,
            ViolationType::PoorNaming,
        ] {
            let violation = Violation {
                violation_type: vtype.clone(),
                location: Location {
                    file: PathBuf::from("test.rs"),
                    line: 1,
                    column: 1,
                },
                severity: Severity::Low,
                description: "Test".to_string(),
                suggested_fix: None,
            };

            match violation.to_op() {
                RefactorOp::SimplifyExpression { expr, simplified } => {
                    assert_eq!(expr, "complex");
                    assert_eq!(simplified, "simple");
                }
                other => panic!(
                    "Expected SimplifyExpression fall-through for {:?}, got {:?}",
                    vtype, other
                ),
            }
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

    // ========================================================================
    // RefactorStateMachine advance - late state arms
    // ========================================================================

    #[test]
    fn test_state_machine_advance_full_cycle_single_target() {
        // Walk a single "complex" target through every non-Complete arm of advance():
        // Scan -> Analyze -> Plan -> Refactor -> Test -> Lint -> Emit -> Checkpoint -> Complete.
        let targets = vec![PathBuf::from("src/complex_module.rs")];
        let mut sm = RefactorStateMachine::new(targets, RefactorConfig::default());

        sm.advance().unwrap(); // Scan -> Analyze
        sm.advance().unwrap(); // Analyze -> Plan
        sm.advance().unwrap(); // Plan (violations present for "complex") -> Refactor
        assert!(matches!(sm.current, State::Refactor { .. }));

        sm.advance().unwrap(); // Refactor -> Test
        assert!(matches!(sm.current, State::Test { .. }));

        sm.advance().unwrap(); // Test -> Lint
        assert!(matches!(sm.current, State::Lint { .. }));

        sm.advance().unwrap(); // Lint -> Emit
        assert!(matches!(sm.current, State::Emit { .. }));

        sm.advance().unwrap(); // Emit -> Checkpoint
        assert!(matches!(sm.current, State::Checkpoint { .. }));

        sm.advance().unwrap(); // Checkpoint -> Complete (next_target = None for single-target run)
        assert!(matches!(sm.current, State::Complete { .. }));
    }

    #[test]
    fn test_state_machine_advance_checkpoint_to_analyze_multi_target() {
        // Two targets: first file complete through Checkpoint, next_target returns Some,
        // so Checkpoint advances to Analyze on the second target.
        let targets = vec![
            PathBuf::from("src/complex_a.rs"),
            PathBuf::from("src/complex_b.rs"),
        ];
        let mut sm = RefactorStateMachine::new(targets, RefactorConfig::default());

        // Drive to Checkpoint (7 advances: Scan→Analyze→Plan→Refactor→Test→Lint→Emit→Checkpoint)
        for _ in 0..7 {
            sm.advance().unwrap();
        }
        assert!(matches!(sm.current, State::Checkpoint { .. }));

        sm.advance().unwrap(); // Checkpoint -> Analyze (second target)
        assert!(matches!(sm.current, State::Analyze { .. }));
    }

    #[test]
    fn test_state_machine_advance_plan_empty_multi_target_goes_analyze() {
        // Two non-complex targets: Plan finds no violations on first, next_target
        // returns Some, so Plan -> Analyze (second target), not Complete.
        let targets = vec![
            PathBuf::from("src/simple_a.rs"),
            PathBuf::from("src/simple_b.rs"),
        ];
        let mut sm = RefactorStateMachine::new(targets, RefactorConfig::default());

        sm.advance().unwrap(); // Scan -> Analyze
        sm.advance().unwrap(); // Analyze -> Plan (empty violations)
        sm.advance().unwrap(); // Plan -> Analyze (next target)
        assert!(matches!(sm.current, State::Analyze { .. }));
    }

    #[test]
    fn test_state_machine_advance_complete_is_terminal() {
        // From Complete, advance() must return Ok without pushing history.
        let mut sm = RefactorStateMachine::new(vec![], RefactorConfig::default());
        assert!(matches!(sm.current, State::Complete { .. }));
        let before = sm.history.len();

        let result = sm.advance();
        assert!(result.is_ok());
        assert!(matches!(sm.current, State::Complete { .. }));
        assert_eq!(sm.history.len(), before, "Complete must not advance");
    }

    /// refactor_impls.rs:96-101 — State::Plan arm where violations[0].suggested_fix
    /// is None triggers the `unwrap_or(RefactorOp::SimplifyExpression { .. })`
    /// fallback. find_violations() always returns Some, so this branch requires
    /// a hand-crafted Plan state with a None-suggested_fix violation injected
    /// directly into sm.current before calling advance().
    #[test]
    fn test_state_machine_advance_plan_unwrap_or_fallback_when_suggested_fix_is_none() {
        let mut sm = RefactorStateMachine::new(
            vec![PathBuf::from("src/anything.rs")],
            RefactorConfig::default(),
        );

        // Override the current state with a Plan containing a None-suggested_fix
        // violation so the advance() unwrap_or fallback path fires.
        sm.current = State::Plan {
            violations: vec![Violation {
                violation_type: ViolationType::HighComplexity,
                location: Location {
                    file: PathBuf::from("src/anything.rs"),
                    line: 1,
                    column: 1,
                },
                severity: Severity::High,
                description: "no suggested fix".to_string(),
                suggested_fix: None,
            }],
        };

        sm.advance().unwrap();
        // Must land in Refactor with the SimplifyExpression fallback op.
        match &sm.current {
            State::Refactor {
                operation: RefactorOp::SimplifyExpression { expr, simplified },
            } => {
                assert_eq!(expr, "complex");
                assert_eq!(simplified, "simple");
            }
            other => panic!(
                "unwrap_or fallback must produce SimplifyExpression, got {other:?}"
            ),
        }
    }
