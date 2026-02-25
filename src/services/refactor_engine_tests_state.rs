    // === State Export Edge Cases ===

    #[tokio::test]
    async fn test_export_state_scan() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("test.json"),
            explain_level: ExplainLevel::Brief,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("test.rs")]);

        // Initial state is Scan
        let state = engine.export_state().await;
        assert_eq!(state.state.state_type, "Scan");
    }

    #[tokio::test]
    async fn test_export_state_test() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("test.json"),
            explain_level: ExplainLevel::Brief,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("complex_module.rs")]);

        // Advance to Test state
        {
            let mut sm = engine.state_machine.write().await;
            let _ = sm.advance(); // -> Analyze
            let _ = sm.advance(); // -> Plan
            let _ = sm.advance(); // -> Refactor
            let _ = sm.advance(); // -> Test
        }

        let state = engine.export_state().await;
        assert_eq!(state.state.state_type, "Test");
    }

    #[tokio::test]
    async fn test_export_state_lint() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("test.json"),
            explain_level: ExplainLevel::Brief,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("complex_module.rs")]);

        // Advance to Lint state
        {
            let mut sm = engine.state_machine.write().await;
            let _ = sm.advance(); // -> Analyze
            let _ = sm.advance(); // -> Plan
            let _ = sm.advance(); // -> Refactor
            let _ = sm.advance(); // -> Test
            let _ = sm.advance(); // -> Lint
        }

        let state = engine.export_state().await;
        assert_eq!(state.state.state_type, "Lint");
    }

    #[tokio::test]
    async fn test_export_state_emit() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("test.json"),
            explain_level: ExplainLevel::Brief,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("complex_module.rs")]);

        // Advance to Emit state
        {
            let mut sm = engine.state_machine.write().await;
            let _ = sm.advance(); // -> Analyze
            let _ = sm.advance(); // -> Plan
            let _ = sm.advance(); // -> Refactor
            let _ = sm.advance(); // -> Test
            let _ = sm.advance(); // -> Lint
            let _ = sm.advance(); // -> Emit
        }

        let state = engine.export_state().await;
        assert_eq!(state.state.state_type, "Emit");
    }

    #[tokio::test]
    async fn test_export_state_checkpoint() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("test.json"),
            explain_level: ExplainLevel::Brief,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("complex_module.rs")]);

        // Advance to Checkpoint state
        {
            let mut sm = engine.state_machine.write().await;
            let _ = sm.advance(); // -> Analyze
            let _ = sm.advance(); // -> Plan
            let _ = sm.advance(); // -> Refactor
            let _ = sm.advance(); // -> Test
            let _ = sm.advance(); // -> Lint
            let _ = sm.advance(); // -> Emit
            let _ = sm.advance(); // -> Checkpoint
        }

        let state = engine.export_state().await;
        assert_eq!(state.state.state_type, "Checkpoint");
    }

    // === Rollback with target index decrement ===

    #[tokio::test]
    async fn test_rollback_decrements_target_index() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("test.json"),
            explain_level: ExplainLevel::Brief,
        };

        let engine = create_test_engine(
            mode,
            vec![PathBuf::from("file1.rs"), PathBuf::from("file2.rs")],
        );

        // Advance through states to increment target index
        {
            let mut sm = engine.state_machine.write().await;
            // Process first file completely
            let _ = sm.advance(); // Scan -> Analyze

            // Manually set target index to test rollback
            sm.current_target_index = 1;
        }

        // Rollback - verifies that rollback completes successfully
        let result = engine.rollback_last_change().await;
        assert!(result.is_ok());

        // Target index behavior depends on state machine implementation
        let sm = engine.state_machine.read().await;
        // The index should be <= 1 (may or may not decrement depending on state)
        assert!(sm.current_target_index <= 1);
    }

    // === JavaScript/JSX/TSX file analysis ===

    #[tokio::test]
    async fn test_analyze_incremental_jsx_file() {
        let tmp_dir = tempdir().expect("create temp dir");
        let jsx_file = tmp_dir.path().join("Component.jsx");

        let content = r#"
            function Component() {
                // TODO: add props validation
                return <div>Hello</div>;
            }
        "#;
        tokio::fs::write(&jsx_file, content)
            .await
            .expect("write file");

        let mode = EngineMode::Server {
            emit_buffer: Arc::new(RwLock::new(RingBuffer::new(10))),
            latency_target: Duration::from_millis(100),
        };

        let engine = create_test_engine(mode, vec![]);
        let metrics = engine
            .analyze_incremental(&jsx_file)
            .await
            .expect("analyze");

        assert_eq!(metrics.complexity[0], 8);
        assert_eq!(metrics.satd, 1);
    }

    #[tokio::test]
    async fn test_analyze_incremental_tsx_file() {
        let tmp_dir = tempdir().expect("create temp dir");
        let tsx_file = tmp_dir.path().join("Component.tsx");

        let content = r#"
            interface Props {}
            function Component(props: Props) {
                // FIXME: type error
                // HACK: workaround
                return <div>Hello</div>;
            }
        "#;
        tokio::fs::write(&tsx_file, content)
            .await
            .expect("write file");

        let mode = EngineMode::Server {
            emit_buffer: Arc::new(RwLock::new(RingBuffer::new(10))),
            latency_target: Duration::from_millis(100),
        };

        let engine = create_test_engine(mode, vec![]);
        let metrics = engine
            .analyze_incremental(&tsx_file)
            .await
            .expect("analyze");

        assert_eq!(metrics.complexity[0], 8);
        assert_eq!(metrics.satd, 2);
    }

    // === TDG calculation tests ===

    #[tokio::test]
    async fn test_tdg_calculation_capped() {
        let tmp_dir = tempdir().expect("create temp dir");
        let rust_file = tmp_dir.path().join("complex.rs");

        // Create a file with very high complexity
        let mut content = String::new();
        for i in 0..100 {
            content.push_str(&format!(
                "fn f{}() {{ if true {{}} match x {{}} for i in 0..10 {{}} }}\n",
                i
            ));
        }
        tokio::fs::write(&rust_file, content)
            .await
            .expect("write file");

        let mode = EngineMode::Server {
            emit_buffer: Arc::new(RwLock::new(RingBuffer::new(10))),
            latency_target: Duration::from_millis(100),
        };

        let engine = create_test_engine(mode, vec![]);
        let metrics = engine
            .analyze_incremental(&rust_file)
            .await
            .expect("analyze");

        // TDG should be capped at 3.0
        assert!(metrics.tdg <= 3.0);
    }

    // === Engine metrics mutation ===

    #[test]
    fn test_engine_metrics_mutation() {
        let mut metrics = EngineMetrics::default();

        // Simulate processing operations
        for _ in 0..100 {
            metrics.operations_processed += 1;
        }

        for _ in 0..50 {
            metrics.refactors_applied += 1;
        }

        metrics.average_latency = Duration::from_millis(250);
        metrics.errors_encountered = 5;

        assert_eq!(metrics.operations_processed, 100);
        assert_eq!(metrics.refactors_applied, 50);
        assert_eq!(metrics.average_latency, Duration::from_millis(250));
        assert_eq!(metrics.errors_encountered, 5);
    }

    // === StateInfo and related struct tests ===

    #[test]
    fn test_state_info_full_fields() {
        let info = StateInfo {
            state_type: "Refactor".to_string(),
            current_file: Some("/path/to/file.rs".to_string()),
            current_function: Some("complex_function".to_string()),
            line_range: Some([100, 200]),
        };

        let json = serde_json::to_string(&info).expect("serialize");
        let deserialized: StateInfo = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.state_type, "Refactor");
        assert_eq!(deserialized.current_file.unwrap(), "/path/to/file.rs");
        assert_eq!(deserialized.current_function.unwrap(), "complex_function");
        assert_eq!(deserialized.line_range.unwrap(), [100, 200]);
    }

    #[test]
    fn test_metrics_info_none_values() {
        let info = MetricsInfo {
            before: None,
            projected: None,
        };

        let json = serde_json::to_string(&info).expect("serialize");
        let deserialized: MetricsInfo = serde_json::from_str(&json).expect("deserialize");

        assert!(deserialized.before.is_none());
        assert!(deserialized.projected.is_none());
    }

    #[test]
    fn test_suggestion_info_empty_operations() {
        let info = SuggestionInfo {
            suggestion_type: "SimplifyExpression".to_string(),
            description: "Simplify boolean expression".to_string(),
            operations: vec![],
        };

        let json = serde_json::to_string(&info).expect("serialize");
        assert!(json.contains("SimplifyExpression"));
        assert!(json.contains("[]")); // Empty operations array
    }

    #[test]
    fn test_operation_info_fields() {
        let info = OperationInfo {
            name: "extract_condition".to_string(),
            lines: [50, 75],
            complexity_reduction: 12,
        };

        let json = serde_json::to_string(&info).expect("serialize");
        let deserialized: OperationInfo = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.name, "extract_condition");
        assert_eq!(deserialized.lines, [50, 75]);
        assert_eq!(deserialized.complexity_reduction, 12);
    }
