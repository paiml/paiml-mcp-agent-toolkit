    // === should_emit and create_payload Tests ===

    #[test]
    fn test_should_emit_high_cyclomatic() {
        let mode = EngineMode::Server {
            emit_buffer: Arc::new(RwLock::new(RingBuffer::new(10))),
            latency_target: Duration::from_millis(100),
        };

        let engine = create_test_engine(mode, vec![]);

        let metrics = ComplexityInfo {
            complexity: [20, 10], // High cyclomatic
            tdg: 1.0,
            satd: 0,
        };

        assert!(engine.should_emit(&metrics));
    }

    #[test]
    fn test_should_emit_high_cognitive() {
        let mode = EngineMode::Server {
            emit_buffer: Arc::new(RwLock::new(RingBuffer::new(10))),
            latency_target: Duration::from_millis(100),
        };

        let engine = create_test_engine(mode, vec![]);

        let metrics = ComplexityInfo {
            complexity: [10, 25], // High cognitive
            tdg: 1.0,
            satd: 0,
        };

        assert!(engine.should_emit(&metrics));
    }

    #[test]
    fn test_should_emit_high_tdg() {
        let mode = EngineMode::Server {
            emit_buffer: Arc::new(RwLock::new(RingBuffer::new(10))),
            latency_target: Duration::from_millis(100),
        };

        let engine = create_test_engine(mode, vec![]);

        let metrics = ComplexityInfo {
            complexity: [5, 8],
            tdg: 2.5, // High TDG
            satd: 0,
        };

        assert!(engine.should_emit(&metrics));
    }

    #[test]
    fn test_should_emit_below_thresholds() {
        let mode = EngineMode::Server {
            emit_buffer: Arc::new(RwLock::new(RingBuffer::new(10))),
            latency_target: Duration::from_millis(100),
        };

        let engine = create_test_engine(mode, vec![]);

        let metrics = ComplexityInfo {
            complexity: [5, 8],
            tdg: 1.0,
            satd: 0,
        };

        assert!(!engine.should_emit(&metrics));
    }

    #[test]
    fn test_create_payload() {
        let mode = EngineMode::Server {
            emit_buffer: Arc::new(RwLock::new(RingBuffer::new(10))),
            latency_target: Duration::from_millis(100),
        };

        let engine = create_test_engine(mode, vec![]);

        let metrics = ComplexityInfo {
            complexity: [15, 20],
            tdg: 1.5,
            satd: 3,
        };

        let payload = engine.create_payload(Path::new("test.rs"), metrics);

        assert_eq!(payload.tdg_score, 1.5);
        assert_eq!(payload.complexity, (15, 20));
        assert!(payload.refactor_available);
        assert!(matches!(
            payload.refactor_type,
            RefactorType::ExtractFunction
        ));
        assert!(payload.timestamp > 0);
    }

    // === run_batch Tests ===

    #[tokio::test]
    async fn test_run_batch_empty_targets() {
        let tmp_dir = tempdir().expect("create temp dir");

        let mode = EngineMode::Batch {
            checkpoint_dir: tmp_dir.path().to_path_buf(),
            resume: false,
            parallel_workers: 2,
        };

        let mut engine = create_test_engine(mode, vec![]);

        let result = engine.run().await;
        assert!(result.is_ok());

        let summary = result.unwrap();
        assert_eq!(summary.files_processed, 0);
        assert_eq!(summary.refactors_applied, 0);
    }

    #[tokio::test]
    async fn test_run_batch_with_resume() {
        let tmp_dir = tempdir().expect("create temp dir");

        // Create a checkpoint file first
        let checkpoint_path = tmp_dir.path().join("checkpoint.json");
        let state_machine =
            RefactorStateMachine::new(vec![PathBuf::from("test.rs")], RefactorConfig::default());
        let checkpoint_data = serde_json::to_string(&state_machine).expect("serialize");
        tokio::fs::write(&checkpoint_path, checkpoint_data)
            .await
            .expect("write");

        let mode = EngineMode::Batch {
            checkpoint_dir: tmp_dir.path().to_path_buf(),
            resume: true,
            parallel_workers: 1,
        };

        let mut engine = create_test_engine(mode, vec![PathBuf::from("other.rs")]);

        let result = engine.run().await;
        assert!(result.is_ok());
    }

