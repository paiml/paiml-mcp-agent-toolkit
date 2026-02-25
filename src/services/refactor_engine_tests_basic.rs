    // === Sprint 46 Phase 8: TDD Tests for refactor_engine.rs ===

    #[test]
    fn test_engine_mode_variants() {
        // Test Server mode
        let server_mode = EngineMode::Server {
            emit_buffer: Arc::new(RwLock::new(RingBuffer::new(100))),
            latency_target: Duration::from_millis(100),
        };
        assert!(matches!(server_mode, EngineMode::Server { .. }));

        // Test Interactive mode
        let interactive_mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("checkpoint.json"),
            explain_level: ExplainLevel::Detailed,
        };
        assert!(matches!(interactive_mode, EngineMode::Interactive { .. }));

        // Test Batch mode
        let batch_mode = EngineMode::Batch {
            checkpoint_dir: PathBuf::from("/tmp/checkpoints"),
            resume: true,
            parallel_workers: 4,
        };
        if let EngineMode::Batch {
            parallel_workers,
            resume,
            ..
        } = batch_mode
        {
            assert_eq!(parallel_workers, 4);
            assert!(resume);
        }
    }

    #[test]
    fn test_explain_level() {
        let brief = ExplainLevel::Brief;
        let detailed = ExplainLevel::Detailed;
        let verbose = ExplainLevel::Verbose;

        // Test serialization
        let brief_json = serde_json::to_string(&brief).expect("internal error");
        assert!(brief_json.contains("Brief"));

        let detailed_json = serde_json::to_string(&detailed).expect("internal error");
        assert!(detailed_json.contains("Detailed"));

        let verbose_json = serde_json::to_string(&verbose).expect("internal error");
        assert!(verbose_json.contains("Verbose"));
    }

    #[test]
    fn test_ring_buffer_creation() {
        let buffer: RingBuffer<i32> = RingBuffer::new(10);
        assert_eq!(buffer.capacity, 10);
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn test_ring_buffer_push() {
        let mut buffer = RingBuffer::new(3);

        // Push items
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        assert_eq!(buffer.buffer.len(), 3);
        assert_eq!(buffer.buffer[0], 1);
        assert_eq!(buffer.buffer[2], 3);

        // Push beyond capacity - should wrap around
        buffer.push(4);
        assert_eq!(buffer.buffer.len(), 3);
        assert_eq!(buffer.buffer[0], 2); // First item should be removed
        assert_eq!(buffer.buffer[2], 4); // New item at end
    }

    #[test]
    fn test_ring_buffer_drain() {
        let mut buffer = RingBuffer::new(5);
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        let drained: Vec<i32> = buffer.drain();
        assert_eq!(drained, vec![1, 2, 3]);
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn test_engine_metrics_default() {
        let metrics = EngineMetrics::default();

        assert_eq!(metrics.operations_processed, 0);
        assert_eq!(metrics.refactors_applied, 0);
        assert_eq!(metrics.average_latency, Duration::from_secs(0));
        assert_eq!(metrics.errors_encountered, 0);
    }

    #[test]
    fn test_engine_metrics_record_operations() {
        let mut metrics = EngineMetrics::default();

        metrics.operations_processed += 10;
        metrics.refactors_applied += 5;
        metrics.average_latency = Duration::from_millis(150);
        metrics.errors_encountered += 1;

        assert_eq!(metrics.operations_processed, 10);
        assert_eq!(metrics.refactors_applied, 5);
        assert_eq!(metrics.average_latency, Duration::from_millis(150));
        assert_eq!(metrics.errors_encountered, 1);
    }

    #[test]
    fn test_engine_error_variants() {
        // Test StateMachine error
        let state_error = EngineError::StateMachine("Invalid state".to_string());
        assert_eq!(
            state_error.to_string(),
            "State machine error: Invalid state"
        );

        // Test Analysis error
        let analysis_error = EngineError::Analysis("Parse failed".to_string());
        assert_eq!(analysis_error.to_string(), "Analysis error: Parse failed");

        // Test IO error conversion
        let io_error: EngineError =
            std::io::Error::new(std::io::ErrorKind::NotFound, "File not found").into();
        assert!(io_error.to_string().contains("IO error"));
    }

    #[test]
    fn test_engine_error_from_string() {
        let error: EngineError = "Test error".to_string().into();
        assert!(matches!(error, EngineError::StateMachine(_)));
        assert_eq!(error.to_string(), "State machine error: Test error");
    }

    #[tokio::test]
    #[ignore = "Test needs update for new UnifiedEngine API"]
    async fn test_unified_engine_new_server() {
        // TODO: Update to use new UnifiedEngine::new() with proper parameters
        // Need ast_engine, cache, mode, config, and targets
    }

    #[tokio::test]
    #[ignore = "Test needs update for new UnifiedEngine API"]
    async fn test_unified_engine_new_interactive() {
        // TODO: Update to use new UnifiedEngine::new() with proper parameters
        // Need to create EngineMode::Interactive and pass all required params
    }

    #[tokio::test]
    #[ignore = "Test needs update for new UnifiedEngine API"]
    async fn test_unified_engine_new_batch() {
        // TODO: Update to use new UnifiedEngine::new() with proper parameters
        // Need to create EngineMode::Batch and pass all required params
    }

    #[tokio::test]
    async fn test_state_machine_transitions() {
        let targets = vec![PathBuf::from("test.rs")];
        let config = RefactorConfig::default();
        let state_machine = RefactorStateMachine::new(targets.clone(), config);

        // Check initial state (using Scan as the initial state)
        assert!(matches!(state_machine.current, State::Scan { .. }));

        // State machine structure is verified
        assert_eq!(state_machine.targets.len(), 1);
        assert_eq!(state_machine.current_target_index, 0);
    }

    #[test]
    fn test_refactor_config_default() {
        let config = RefactorConfig::default();

        // Verify default values are sensible
        assert!(config.target_complexity > 0);
        assert!(config.max_function_lines > 0);
        assert!(config.memory_limit_mb > 0);
    }

    #[test]
    fn test_summary_creation() {
        let summary = Summary {
            files_processed: 10,
            refactors_applied: 8,
            complexity_reduction: 25.5,
            satd_removed: 12,
            total_time: Duration::from_secs(120),
        };

        assert_eq!(summary.files_processed, 10);
        assert_eq!(summary.refactors_applied, 8);
        assert_eq!(summary.complexity_reduction, 25.5);
        assert_eq!(summary.satd_removed, 12);
        assert_eq!(summary.total_time, Duration::from_secs(120));
    }

    #[test]
    fn test_refactor_type_variants() {
        // RefactorType enum no longer exists - using RefactorOp instead
        // This test was for a deprecated type
    }

    // The following tests are commented out because UnifiedEngine no longer exists
    // These tests need to be rewritten for the current refactor engine implementation

    // test_engine_is_complete() - needs rewrite for current engine
    // test_engine_get_state() - needs rewrite for current engine
