    // === Async Engine Tests ===

    #[tokio::test]
    async fn test_export_state_analyze() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("test.json"),
            explain_level: ExplainLevel::Brief,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("src/test.rs")]);

        // Advance to Analyze state
        {
            let mut sm = engine.state_machine.write().await;
            let _ = sm.advance();
        }

        let state = engine.export_state().await;
        assert_eq!(state.state.state_type, "Analyze");
        assert!(state.state.current_file.is_some());
    }

    #[tokio::test]
    async fn test_export_state_plan() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("test.json"),
            explain_level: ExplainLevel::Brief,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("complex_module.rs")]);

        // Advance through states to reach Plan
        {
            let mut sm = engine.state_machine.write().await;
            let _ = sm.advance(); // Scan -> Analyze
            let _ = sm.advance(); // Analyze -> Plan
        }

        let state = engine.export_state().await;
        assert_eq!(state.state.state_type, "Plan");
    }

    #[tokio::test]
    async fn test_export_state_refactor() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("test.json"),
            explain_level: ExplainLevel::Brief,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("complex_module.rs")]);

        // Advance to Refactor state
        {
            let mut sm = engine.state_machine.write().await;
            let _ = sm.advance(); // Scan -> Analyze
            let _ = sm.advance(); // Analyze -> Plan
            let _ = sm.advance(); // Plan -> Refactor
        }

        let state = engine.export_state().await;
        assert_eq!(state.state.state_type, "Refactor");
    }

    #[tokio::test]
    async fn test_export_state_complete() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("test.json"),
            explain_level: ExplainLevel::Brief,
        };

        let engine = create_test_engine(mode, vec![]); // Empty targets -> Complete state

        let state = engine.export_state().await;
        assert_eq!(state.state.state_type, "Complete");
    }

    #[tokio::test]
    async fn test_explain_current_state_analyze() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("test.json"),
            explain_level: ExplainLevel::Verbose,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("src/test.rs")]);

        // Advance to Analyze state
        {
            let mut sm = engine.state_machine.write().await;
            let _ = sm.advance();
        }

        let explanation = engine
            .explain_current_state()
            .await
            .expect("get explanation");
        assert!(explanation.contains("analyzing"));
        assert!(explanation.contains("complexity"));
    }

    #[tokio::test]
    async fn test_explain_current_state_plan() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("test.json"),
            explain_level: ExplainLevel::Detailed,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("complex_module.rs")]);

        // Advance to Plan state
        {
            let mut sm = engine.state_machine.write().await;
            let _ = sm.advance(); // -> Analyze
            let _ = sm.advance(); // -> Plan
        }

        let explanation = engine
            .explain_current_state()
            .await
            .expect("get explanation");
        assert!(explanation.contains("Planning"));
        assert!(explanation.contains("violations"));
    }

    #[tokio::test]
    async fn test_explain_current_state_refactor() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("test.json"),
            explain_level: ExplainLevel::Verbose,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("complex_module.rs")]);

        // Advance to Refactor state
        {
            let mut sm = engine.state_machine.write().await;
            let _ = sm.advance(); // -> Analyze
            let _ = sm.advance(); // -> Plan
            let _ = sm.advance(); // -> Refactor
        }

        let explanation = engine
            .explain_current_state()
            .await
            .expect("get explanation");
        assert!(explanation.contains("refactoring"));
    }

    #[tokio::test]
    async fn test_step_with_explanation() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("test.json"),
            explain_level: ExplainLevel::Detailed,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("test.rs")]);

        let result = engine.step_with_explanation().await.expect("step");
        assert!(result.success);
        assert!(!result.explanation.is_empty());
        assert!(result.metrics_changed);
    }

    #[tokio::test]
    async fn test_rollback_last_change() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("test.json"),
            explain_level: ExplainLevel::Brief,
        };

        let engine = create_test_engine(mode, vec![PathBuf::from("test.rs")]);

        // First, make some transitions to build history
        {
            let mut sm = engine.state_machine.write().await;
            let _ = sm.advance(); // Scan -> Analyze
            let _ = sm.advance(); // Analyze -> Plan
        }

        // Now rollback
        let result = engine.rollback_last_change().await;
        assert!(result.is_ok());

        // Verify we're back to Analyze state
        let sm = engine.state_machine.read().await;
        assert!(matches!(sm.current, State::Analyze { .. }));
    }

    #[tokio::test]
    async fn test_rollback_empty_history_error() {
        let mode = EngineMode::Interactive {
            checkpoint_file: PathBuf::from("test.json"),
            explain_level: ExplainLevel::Brief,
        };

        let engine = create_test_engine(mode, vec![]); // Empty -> Complete state, no history

        let result = engine.rollback_last_change().await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, EngineError::StateMachine(_)));
    }

