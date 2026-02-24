    // ========================================================================
    // RefactorConfig Tests
    // ========================================================================

    #[test]
    fn test_refactor_config_default() {
        let config = RefactorConfig::default();

        assert_eq!(config.target_complexity, 20);
        assert!(config.remove_satd);
        assert_eq!(config.max_function_lines, 50);
        assert_eq!(config.parallel_workers, 4);
        assert_eq!(config.memory_limit_mb, 512);
        assert_eq!(config.batch_size, 10);
        assert!(config.priority_expression.is_none());
        assert!(config.auto_commit_template.is_none());
    }

    #[test]
    fn test_refactor_config_serialization() {
        let config = RefactorConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RefactorConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.target_complexity, config.target_complexity);
        assert_eq!(deserialized.remove_satd, config.remove_satd);
    }

    // ========================================================================
    // Thresholds Tests
    // ========================================================================

    #[test]
    fn test_thresholds_default() {
        let thresholds = Thresholds::default();

        assert_eq!(thresholds.cyclomatic_warn, 10);
        assert_eq!(thresholds.cyclomatic_error, 20);
        assert_eq!(thresholds.cognitive_warn, 15);
        assert_eq!(thresholds.cognitive_error, 30);
        assert!((thresholds.tdg_warn - 1.5).abs() < 0.01);
        assert!((thresholds.tdg_error - 2.0).abs() < 0.01);
    }

    // ========================================================================
    // RefactorStrategies Tests
    // ========================================================================

    #[test]
    fn test_refactor_strategies_default() {
        let strategies = RefactorStrategies::default();

        assert!(strategies.prefer_functional);
        assert!(strategies.use_early_returns);
        assert!(strategies.extract_helpers);
    }

    // ========================================================================
    // MetricSet Tests
    // ========================================================================

    #[test]
    fn test_metric_set_default() {
        let metrics = MetricSet::default();

        assert_eq!(metrics.complexity, (0, 0));
        assert!((metrics.tdg_score - 0.0).abs() < 0.01);
        assert!(metrics.dead_code.is_empty());
        assert_eq!(metrics.satd_count, 0);
        assert!((metrics.provability - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_metric_set_serialization() {
        let metrics = MetricSet {
            complexity: (15, 20),
            tdg_score: 1.5,
            dead_code: vec![true, false, true],
            satd_count: 5,
            provability: 0.8,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: MetricSet = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.complexity, (15, 20));
        assert_eq!(deserialized.satd_count, 5);
    }

    // ========================================================================
    // Summary Tests
    // ========================================================================

    #[test]
    fn test_summary_default() {
        let summary = Summary::default();

        assert_eq!(summary.files_processed, 0);
        assert_eq!(summary.refactors_applied, 0);
        assert!((summary.complexity_reduction - 0.0).abs() < 0.01);
        assert_eq!(summary.satd_removed, 0);
        assert_eq!(summary.total_time, Duration::from_secs(0));
    }

    // ========================================================================
    // State Tests
    // ========================================================================

    #[test]
    fn test_state_scan() {
        let targets = vec![PathBuf::from("file1.rs"), PathBuf::from("file2.rs")];
        let state = State::Scan {
            targets: targets.clone(),
        };

        if let State::Scan { targets: t } = state {
            assert_eq!(t.len(), 2);
        } else {
            panic!("Expected Scan state");
        }
    }

    #[test]
    fn test_state_analyze() {
        let file_id = FileId {
            path: PathBuf::from("test.rs"),
            hash: 12345,
        };
        let state = State::Analyze { current: file_id };

        if let State::Analyze { current } = state {
            assert_eq!(current.hash, 12345);
        } else {
            panic!("Expected Analyze state");
        }
    }

    #[test]
    fn test_state_complete() {
        let summary = Summary::default();
        let state = State::Complete { summary };

        if let State::Complete { summary: s } = state {
            assert_eq!(s.files_processed, 0);
        } else {
            panic!("Expected Complete state");
        }
    }

    // ========================================================================
    // RefactorStateMachine Tests
    // ========================================================================

    #[test]
    fn test_state_machine_new_with_targets() {
        let targets = vec![PathBuf::from("file1.rs")];
        let config = RefactorConfig::default();
        let sm = RefactorStateMachine::new(targets.clone(), config);

        assert_eq!(sm.targets.len(), 1);
        assert_eq!(sm.current_target_index, 0);

        if let State::Scan { targets: t } = &sm.current {
            assert_eq!(t.len(), 1);
        } else {
            panic!("Expected Scan state");
        }
    }

    #[test]
    fn test_state_machine_new_empty_targets() {
        let targets: Vec<PathBuf> = vec![];
        let config = RefactorConfig::default();
        let sm = RefactorStateMachine::new(targets, config);

        if let State::Complete { .. } = &sm.current {
            // Expected
        } else {
            panic!("Expected Complete state for empty targets");
        }
    }

    #[test]
    fn test_state_machine_advance_from_scan() {
        let targets = vec![PathBuf::from("test.rs")];
        let config = RefactorConfig::default();
        let mut sm = RefactorStateMachine::new(targets, config);

        let result = sm.advance();
        assert!(result.is_ok());

        if let State::Analyze { .. } = result.unwrap() {
            // Expected
        } else {
            panic!("Expected Analyze state after advancing from Scan");
        }
    }

    #[test]
    fn test_state_machine_advance_from_empty_scan() {
        let config = RefactorConfig::default();
        let mut sm = RefactorStateMachine {
            current: State::Scan { targets: vec![] },
            history: vec![],
            config,
            targets: vec![],
            current_target_index: 0,
        };

        let result = sm.advance();
        assert!(result.is_ok());

        if let State::Complete { .. } = result.unwrap() {
            // Expected
        } else {
            panic!("Expected Complete state for empty scan");
        }
    }

    #[test]
    fn test_state_machine_history_tracking() {
        let targets = vec![PathBuf::from("test.rs")];
        let config = RefactorConfig::default();
        let mut sm = RefactorStateMachine::new(targets, config);

        assert!(sm.history.is_empty());

        let _ = sm.advance(); // Scan -> Analyze
        assert_eq!(sm.history.len(), 1);

        let _ = sm.advance(); // Analyze -> Plan
        assert_eq!(sm.history.len(), 2);
    }

    // ========================================================================
    // RefactorOp Tests
    // ========================================================================

    #[test]
    fn test_refactor_op_extract_function() {
        let op = RefactorOp::ExtractFunction {
            name: "helper".to_string(),
            start: BytePos {
                byte: 0,
                line: 1,
                column: 1,
            },
            end: BytePos {
                byte: 100,
                line: 10,
                column: 1,
            },
            params: vec!["x".to_string(), "y".to_string()],
        };

        if let RefactorOp::ExtractFunction { name, params, .. } = op {
            assert_eq!(name, "helper");
            assert_eq!(params.len(), 2);
        } else {
            panic!("Expected ExtractFunction");
        }
    }

    #[test]
    fn test_refactor_op_flatten_nesting() {
        let op = RefactorOp::FlattenNesting {
            function: "complex_func".to_string(),
            strategy: NestingStrategy::EarlyReturn,
        };

        if let RefactorOp::FlattenNesting { strategy, .. } = op {
            if let NestingStrategy::EarlyReturn = strategy {
                // Expected
            } else {
                panic!("Expected EarlyReturn strategy");
            }
        } else {
            panic!("Expected FlattenNesting");
        }
    }

    #[test]
    fn test_refactor_op_serialization() {
        let op = RefactorOp::SimplifyExpression {
            expr: "a && b || c".to_string(),
            simplified: "simplified".to_string(),
        };

        let json = serde_json::to_string(&op).unwrap();
        let deserialized: RefactorOp = serde_json::from_str(&json).unwrap();

        if let RefactorOp::SimplifyExpression { expr, simplified } = deserialized {
            assert_eq!(expr, "a && b || c");
            assert_eq!(simplified, "simplified");
        } else {
            panic!("Expected SimplifyExpression");
        }
    }

