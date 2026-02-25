    // === Checkpoint Tests ===

    #[tokio::test]
    async fn test_save_checkpoint_interactive() {
        let tmp_dir = tempdir().expect("create temp dir");
        let checkpoint_file = tmp_dir.path().join("checkpoint.json");

        let mode = EngineMode::Interactive {
            checkpoint_file: checkpoint_file.clone(),
            explain_level: ExplainLevel::Brief,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("test.rs")]);

        // Save checkpoint
        let result = engine.save_checkpoint().await;
        assert!(result.is_ok());

        // Verify file was created
        let checkpoint_path = tmp_dir.path().join("checkpoint.json");
        assert!(checkpoint_path.exists());
    }

    #[tokio::test]
    async fn test_save_checkpoint_batch() {
        let tmp_dir = tempdir().expect("create temp dir");

        let mode = EngineMode::Batch {
            checkpoint_dir: tmp_dir.path().to_path_buf(),
            resume: false,
            parallel_workers: 2,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("test.rs")]);

        let result = engine.save_checkpoint().await;
        assert!(result.is_ok());

        // Verify checkpoint file was created
        let checkpoint_path = tmp_dir.path().join("checkpoint.json");
        assert!(checkpoint_path.exists());
    }

    #[tokio::test]
    async fn test_save_checkpoint_server_noop() {
        let emit_buffer = Arc::new(RwLock::new(RingBuffer::new(10)));
        let mode = EngineMode::Server {
            emit_buffer,
            latency_target: Duration::from_millis(100),
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("test.rs")]);

        // Server mode save_checkpoint is a no-op
        let result = engine.save_checkpoint().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_checkpoint() {
        let tmp_dir = tempdir().expect("create temp dir");
        let checkpoint_path = tmp_dir.path().join("checkpoint.json");

        // Create a checkpoint file
        let state_machine = RefactorStateMachine::new(
            vec![PathBuf::from("original.rs")],
            RefactorConfig::default(),
        );
        let checkpoint_data = serde_json::to_string_pretty(&state_machine).expect("serialize");
        tokio::fs::write(&checkpoint_path, checkpoint_data)
            .await
            .expect("write checkpoint");

        // Create engine and load checkpoint
        let mode = EngineMode::Batch {
            checkpoint_dir: tmp_dir.path().to_path_buf(),
            resume: true,
            parallel_workers: 1,
        };

        let mut engine = create_test_engine(mode, vec![PathBuf::from("different.rs")]);

        // Load checkpoint - this should replace the state machine
        let result = engine.load_checkpoint(tmp_dir.path()).await;
        assert!(result.is_ok());

        // Verify state was loaded from checkpoint
        let sm = engine.state_machine.read().await;
        assert_eq!(sm.targets[0], PathBuf::from("original.rs"));
    }

    #[tokio::test]
    async fn test_load_checkpoint_nonexistent() {
        let tmp_dir = tempdir().expect("create temp dir");

        let mode = EngineMode::Batch {
            checkpoint_dir: tmp_dir.path().to_path_buf(),
            resume: true,
            parallel_workers: 1,
        };

        let mut engine = create_test_engine(mode, vec![PathBuf::from("test.rs")]);

        // Load from empty dir - should succeed (no checkpoint to load)
        let result = engine.load_checkpoint(tmp_dir.path()).await;
        assert!(result.is_ok());
    }

    // === analyze_incremental Tests ===

    #[tokio::test]
    async fn test_analyze_incremental_rust_file() {
        let tmp_dir = tempdir().expect("create temp dir");
        let rust_file = tmp_dir.path().join("test.rs");

        // Create a Rust file with some complexity markers
        let content = r#"
            fn complex_function() {
                if condition1 {
                    if condition2 {
                        for item in items {
                            match item {
                                A => {},
                                B => {},
                            }
                        }
                    }
                }
                // TODO: refactor this
                // FIXME: broken logic
            }
        "#;
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

        // Should have some complexity detected
        assert!(metrics.complexity[0] > 0);
        // Should have SATD markers detected (TODO, FIXME)
        assert!(metrics.satd > 0);
    }

    #[tokio::test]
    async fn test_analyze_incremental_typescript_file() {
        let tmp_dir = tempdir().expect("create temp dir");
        let ts_file = tmp_dir.path().join("test.ts");

        let content = r#"
            function test() {
                // TODO: implement
                // HACK: workaround
            }
        "#;
        tokio::fs::write(&ts_file, content)
            .await
            .expect("write file");

        let mode = EngineMode::Server {
            emit_buffer: Arc::new(RwLock::new(RingBuffer::new(10))),
            latency_target: Duration::from_millis(100),
        };

        let engine = create_test_engine(mode, vec![]);
        let metrics = engine.analyze_incremental(&ts_file).await.expect("analyze");

        // TypeScript files get default complexity values
        assert_eq!(metrics.complexity[0], 8);
        assert_eq!(metrics.complexity[1], 12);
        // Should detect SATD markers
        assert_eq!(metrics.satd, 2); // TODO and HACK
    }

    #[tokio::test]
    async fn test_analyze_incremental_python_file() {
        let tmp_dir = tempdir().expect("create temp dir");
        let py_file = tmp_dir.path().join("test.py");

        let content = r#"
            def test():
                # FIXME: this is broken
                pass
        "#;
        tokio::fs::write(&py_file, content)
            .await
            .expect("write file");

        let mode = EngineMode::Server {
            emit_buffer: Arc::new(RwLock::new(RingBuffer::new(10))),
            latency_target: Duration::from_millis(100),
        };

        let engine = create_test_engine(mode, vec![]);
        let metrics = engine.analyze_incremental(&py_file).await.expect("analyze");

        assert_eq!(metrics.complexity[0], 6);
        assert_eq!(metrics.complexity[1], 9);
        assert_eq!(metrics.satd, 1); // FIXME
    }

    #[tokio::test]
    async fn test_analyze_incremental_other_file() {
        let tmp_dir = tempdir().expect("create temp dir");
        let other_file = tmp_dir.path().join("test.txt");

        tokio::fs::write(&other_file, "plain text content")
            .await
            .expect("write file");

        let mode = EngineMode::Server {
            emit_buffer: Arc::new(RwLock::new(RingBuffer::new(10))),
            latency_target: Duration::from_millis(100),
        };

        let engine = create_test_engine(mode, vec![]);
        let metrics = engine
            .analyze_incremental(&other_file)
            .await
            .expect("analyze");

        // Other files get minimal default values
        assert_eq!(metrics.complexity[0], 3);
        assert_eq!(metrics.complexity[1], 4);
        assert_eq!(metrics.satd, 0);
    }

    #[tokio::test]
    async fn test_analyze_incremental_large_rust_file() {
        let tmp_dir = tempdir().expect("create temp dir");
        let large_file = tmp_dir.path().join("large.rs");

        // Create file larger than 50KB threshold
        let content = "fn f() {} // ".repeat(5000);
        tokio::fs::write(&large_file, content)
            .await
            .expect("write file");

        let mode = EngineMode::Server {
            emit_buffer: Arc::new(RwLock::new(RingBuffer::new(10))),
            latency_target: Duration::from_millis(100),
        };

        let engine = create_test_engine(mode, vec![]);
        let metrics = engine
            .analyze_incremental(&large_file)
            .await
            .expect("analyze");

        // Large files get default "likely complex" values
        assert_eq!(metrics.complexity[0], 20);
        assert_eq!(metrics.complexity[1], 25);
    }

    #[tokio::test]
    async fn test_analyze_incremental_nonexistent_file() {
        let mode = EngineMode::Server {
            emit_buffer: Arc::new(RwLock::new(RingBuffer::new(10))),
            latency_target: Duration::from_millis(100),
        };

        let engine = create_test_engine(mode, vec![]);
        let metrics = engine
            .analyze_incremental(Path::new("/nonexistent/file.rs"))
            .await
            .expect("analyze");

        // Unreadable Rust files get minimal values
        assert_eq!(metrics.complexity[0], 1);
        assert_eq!(metrics.complexity[1], 1);
    }

