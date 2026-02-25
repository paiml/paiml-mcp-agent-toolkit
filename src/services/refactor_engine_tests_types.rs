    // === RingBuffer Tests ===

    #[test]
    fn test_ring_buffer_new_zero_capacity() {
        let buffer: RingBuffer<i32> = RingBuffer::new(0);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_ring_buffer_push_single_item() {
        let mut buffer = RingBuffer::new(5);
        buffer.push(42);
        assert_eq!(buffer.len(), 1);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_ring_buffer_overflow_eviction() {
        let mut buffer = RingBuffer::new(2);
        buffer.push("first");
        buffer.push("second");
        buffer.push("third");

        let items = buffer.drain();
        assert_eq!(items, vec!["second", "third"]);
    }

    #[test]
    fn test_ring_buffer_drain_empty() {
        let mut buffer: RingBuffer<u32> = RingBuffer::new(10);
        let items = buffer.drain();
        assert!(items.is_empty());
    }

    #[test]
    fn test_ring_buffer_multiple_overflow_cycles() {
        let mut buffer = RingBuffer::new(3);
        for i in 0..10 {
            buffer.push(i);
        }
        assert_eq!(buffer.len(), 3);
        let items = buffer.drain();
        assert_eq!(items, vec![7, 8, 9]);
    }

    // === ExplainLevel Tests ===

    #[test]
    fn test_explain_level_deserialization() {
        let brief: ExplainLevel = serde_json::from_str("\"Brief\"").expect("deserialize Brief");
        assert!(matches!(brief, ExplainLevel::Brief));

        let detailed: ExplainLevel =
            serde_json::from_str("\"Detailed\"").expect("deserialize Detailed");
        assert!(matches!(detailed, ExplainLevel::Detailed));

        let verbose: ExplainLevel =
            serde_json::from_str("\"Verbose\"").expect("deserialize Verbose");
        assert!(matches!(verbose, ExplainLevel::Verbose));
    }

    #[test]
    fn test_explain_level_clone() {
        let level = ExplainLevel::Verbose;
        let cloned = level.clone();
        assert!(matches!(cloned, ExplainLevel::Verbose));
    }

    // === Command Tests ===

    #[test]
    fn test_command_serialization_roundtrip() {
        let commands = vec![
            Command::Continue,
            Command::Skip,
            Command::Rollback,
            Command::Checkpoint,
            Command::Explain,
            Command::Exit,
        ];

        for cmd in commands {
            let json = serde_json::to_string(&cmd).expect("serialize command");
            let deserialized: Command = serde_json::from_str(&json).expect("deserialize command");
            // Verify round-trip works
            let json2 = serde_json::to_string(&deserialized).expect("serialize again");
            assert_eq!(json, json2);
        }
    }

    // === EngineError Tests ===

    #[test]
    fn test_engine_error_serialization() {
        let err = EngineError::Serialization(serde_json::from_str::<i32>("not json").unwrap_err());
        assert!(err.to_string().contains("Serialization error"));
    }

    #[test]
    fn test_engine_error_io_variants() {
        use std::io::ErrorKind;

        let kinds = vec![
            ErrorKind::NotFound,
            ErrorKind::PermissionDenied,
            ErrorKind::AlreadyExists,
            ErrorKind::InvalidInput,
        ];

        for kind in kinds {
            let io_err = std::io::Error::new(kind, "test error");
            let engine_err: EngineError = io_err.into();
            assert!(engine_err.to_string().contains("IO error"));
        }
    }

    // === InteractiveState Tests ===

    #[test]
    fn test_interactive_state_serialization() {
        let state = InteractiveState {
            state: StateInfo {
                state_type: "Analyze".to_string(),
                current_file: Some("test.rs".to_string()),
                current_function: None,
                line_range: Some([10, 50]),
            },
            metrics: MetricsInfo {
                before: Some(ComplexityInfo {
                    complexity: [15, 20],
                    tdg: 1.8,
                    satd: 3,
                }),
                projected: Some(ComplexityInfo {
                    complexity: [8, 12],
                    tdg: 0.9,
                    satd: 0,
                }),
            },
            suggestion: Some(SuggestionInfo {
                suggestion_type: "ExtractFunction".to_string(),
                description: "Extract complex logic".to_string(),
                operations: vec![OperationInfo {
                    name: "helper".to_string(),
                    lines: [20, 40],
                    complexity_reduction: 7,
                }],
            }),
            commands: vec!["continue".to_string(), "exit".to_string()],
            explanation: Some("Testing explanation".to_string()),
        };

        let json = serde_json::to_string_pretty(&state).expect("serialize state");
        assert!(json.contains("Analyze"));
        assert!(json.contains("test.rs"));
        assert!(json.contains("ExtractFunction"));
    }

    #[test]
    fn test_step_result_serialization() {
        let result = StepResult {
            success: true,
            explanation: "Transitioned successfully".to_string(),
            metrics_changed: true,
            new_state: "Plan".to_string(),
        };

        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: StepResult = serde_json::from_str(&json).expect("deserialize");

        assert!(deserialized.success);
        assert!(deserialized.metrics_changed);
        assert_eq!(deserialized.new_state, "Plan");
    }

    // === ComplexityInfo Tests ===

    #[test]
    fn test_complexity_info_boundary_values() {
        let max_complexity = ComplexityInfo {
            complexity: [u16::MAX, u16::MAX],
            tdg: f32::MAX,
            satd: u32::MAX,
        };

        let json = serde_json::to_string(&max_complexity).expect("serialize");
        let deserialized: ComplexityInfo = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.complexity[0], u16::MAX);
        assert_eq!(deserialized.satd, u32::MAX);
    }

    // === UnifiedEngine Tests ===

    #[test]
    fn test_unified_engine_server_mode() {
        let emit_buffer = Arc::new(RwLock::new(RingBuffer::new(100)));
        let mode = EngineMode::Server {
            emit_buffer: emit_buffer.clone(),
            latency_target: Duration::from_millis(50),
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("test.rs")]);
        assert!(matches!(engine.mode, EngineMode::Server { .. }));
    }

    #[test]
    fn test_unified_engine_interactive_mode() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("/tmp/checkpoint.json"),
            explain_level: ExplainLevel::Detailed,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("src/main.rs")]);
        assert!(matches!(engine.mode, EngineMode::Interactive { .. }));
    }

    #[test]
    fn test_unified_engine_batch_mode() {
        let mode = EngineMode::Batch {
            checkpoint_dir: PathBuf::from("/tmp/checkpoints"),
            resume: false,
            parallel_workers: 8,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("lib.rs")]);
        assert!(matches!(engine.mode, EngineMode::Batch { .. }));
    }

    #[test]
    fn test_unified_engine_empty_targets() {
        let mode = EngineMode::Batch {
            checkpoint_dir: PathBuf::from("/tmp"),
            resume: false,
            parallel_workers: 1,
        };

        let engine = create_test_engine(mode, vec![]);
        // Empty targets should still create engine
        assert!(matches!(engine.mode, EngineMode::Batch { .. }));
    }

