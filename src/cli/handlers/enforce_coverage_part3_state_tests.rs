    mod continue_enforcement_tests {
        use super::*;

        #[test]
        fn test_should_continue_with_time_limit_not_exceeded() {
            let config = EnforcementConfig {
                max_time: Some(60),
                max_iterations: 100,
                ..make_test_enforcement_config()
            };
            let start_time = Instant::now();
            // Should continue since time limit not exceeded
            assert!(should_continue_enforcement(
                EnforcementState::Analyzing,
                0,
                &config,
                start_time
            ));
        }

        #[test]
        fn test_should_continue_with_no_time_limit() {
            let config = EnforcementConfig {
                max_time: None,
                max_iterations: 100,
                ..make_test_enforcement_config()
            };
            let start_time = Instant::now();
            assert!(should_continue_enforcement(
                EnforcementState::Analyzing,
                50,
                &config,
                start_time
            ));
        }

        #[test]
        fn test_should_continue_validating_state() {
            let config = make_test_enforcement_config();
            let start_time = Instant::now();
            assert!(should_continue_enforcement(
                EnforcementState::Validating,
                0,
                &config,
                start_time
            ));
        }

        #[test]
        fn test_should_continue_refactoring_state() {
            let config = make_test_enforcement_config();
            let start_time = Instant::now();
            assert!(should_continue_enforcement(
                EnforcementState::Refactoring,
                0,
                &config,
                start_time
            ));
        }

        #[test]
        fn test_should_continue_violating_state() {
            let config = make_test_enforcement_config();
            let start_time = Instant::now();
            assert!(should_continue_enforcement(
                EnforcementState::Violating,
                0,
                &config,
                start_time
            ));
        }
    }

    mod target_improvement_tests {
        use super::*;

        #[test]
        fn test_should_stop_exact_target() {
            // Slightly above target to handle f32->f64 precision issues
            let result = should_stop_for_target_improvement(Some(0.3), 0.81, 0.5);
            assert!(result);
        }

        #[test]
        fn test_should_stop_above_target() {
            // Above target
            let result = should_stop_for_target_improvement(Some(0.2), 0.9, 0.5);
            assert!(result);
        }

        #[test]
        fn test_should_not_stop_below_target() {
            // Below target
            let result = should_stop_for_target_improvement(Some(0.5), 0.7, 0.5);
            assert!(!result);
        }

        #[test]
        fn test_check_improvement_targets_none() {
            let config = make_test_enforcement_config();
            let result = check_improvement_targets(&config, 0.9, 0.5);
            assert!(!result);
        }

        #[test]
        fn test_check_improvement_targets_zero() {
            let mut config = make_test_enforcement_config();
            config.target_improvement = Some(0.0);
            let result = check_improvement_targets(&config, 0.8, 0.5);
            assert!(result);
        }
    }

    mod state_handler_extended_tests {
        use super::*;

        #[test]
        fn test_handle_complete_enforcement_state() {
            let result = handle_complete_enforcement_state().unwrap();
            assert_eq!(result.state, EnforcementState::Complete);
            assert_eq!(result.score, 1.0);
            assert!(result.violations.is_empty());
        }

        #[test]
        fn test_handle_refactoring_enforcement_state() {
            let result = handle_refactoring_enforcement_state(0.5, None).unwrap();
            assert_eq!(result.state, EnforcementState::Validating);
            assert_eq!(result.score, 0.6); // 0.5 + 0.1
        }

        #[test]
        fn test_handle_refactoring_enforcement_state_with_file() {
            let file = PathBuf::from("src/lib.rs");
            let result = handle_refactoring_enforcement_state(0.8, Some(&file)).unwrap();
            assert_eq!(result.current_file, Some("src/lib.rs".to_string()));
        }

        #[test]
        fn test_handle_violating_state_with_many_violations() {
            let violations: Vec<QualityViolation> = (0..10)
                .map(|i| make_custom_violation("complexity", "high", &format!("file{i}.rs:1"), 30.0, 10.0))
                .collect();

            let result = handle_violating_state(violations.clone(), 0.3, false, false, None).unwrap();
            assert_eq!(result.violations.len(), 10);
            assert_eq!(result.state, EnforcementState::Violating);
        }

        #[test]
        fn test_handle_violating_state_empty_violations() {
            let violations: Vec<QualityViolation> = vec![];
            let result = handle_violating_state(violations, 0.8, true, false, None).unwrap();
            assert_eq!(result.state, EnforcementState::Refactoring);
        }
    }
