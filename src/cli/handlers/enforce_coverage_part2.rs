    mod helper_function_tests_part2 {
        use super::*;

        #[test]
        fn test_should_continue_enforcement_max_iterations() {
            let config = EnforcementConfig {
                max_iterations: 5,
                ..make_test_enforcement_config()
            };
            let start_time = Instant::now();
            let result =
                should_continue_enforcement(EnforcementState::Analyzing, 5, &config, start_time);
            assert!(!result);
        }

        #[test]
        fn test_should_continue_enforcement_in_progress() {
            let config = make_test_enforcement_config();
            let start_time = Instant::now();
            let result =
                should_continue_enforcement(EnforcementState::Analyzing, 2, &config, start_time);
            assert!(result);
        }

        #[test]
        fn test_should_stop_for_target_improvement_no_target() {
            let result = should_stop_for_target_improvement(None, 0.8, 0.5);
            assert!(!result);
        }

        #[test]
        fn test_should_stop_for_target_improvement_reached() {
            let result = should_stop_for_target_improvement(Some(0.2), 0.8, 0.5);
            assert!(result);
        }

        #[test]
        fn test_should_stop_for_target_improvement_not_reached() {
            let result = should_stop_for_target_improvement(Some(0.5), 0.6, 0.5);
            assert!(!result);
        }

        #[test]
        fn test_check_improvement_targets_achieved() {
            let mut config = make_test_enforcement_config();
            config.target_improvement = Some(0.1);
            let result = check_improvement_targets(&config, 0.8, 0.5);
            assert!(result);
        }

        #[test]
        fn test_check_improvement_targets_not_achieved() {
            let mut config = make_test_enforcement_config();
            config.target_improvement = Some(0.5);
            let result = check_improvement_targets(&config, 0.6, 0.5);
            assert!(!result);
        }
    }

    // ========== State Handler Tests ==========

    mod state_handler_tests {
        use super::*;

        #[test]
        fn test_handle_complete_state() {
            let result = handle_complete_state().unwrap();
            assert_eq!(result.state, EnforcementState::Complete);
            assert_eq!(result.score, 1.0);
            assert_eq!(result.target, 1.0);
            assert!(result.violations.is_empty());
            assert_eq!(result.next_action, "none");
        }

        #[test]
        fn test_handle_refactoring_state() {
            let result = handle_refactoring_state(0.7, None).unwrap();
            assert_eq!(result.state, EnforcementState::Validating);
            // Use approximate comparison for floating point
            assert!((result.score - 0.8).abs() < 0.01); // 0.7 + 0.1 improvement
            assert!(result.violations.is_empty());
            assert_eq!(result.next_action, "validate_changes");
        }

        #[test]
        fn test_handle_refactoring_state_with_file() {
            let file = PathBuf::from("test.rs");
            let result = handle_refactoring_state(0.6, Some(&file)).unwrap();
            assert_eq!(result.current_file, Some("test.rs".to_string()));
        }

        #[test]
        fn test_handle_violating_state_apply_suggestions() {
            let violations = vec![make_test_violation("complexity", "high")];
            let result = handle_violating_state(violations, 0.5, true, false, None).unwrap();
            assert_eq!(result.state, EnforcementState::Refactoring);
            assert_eq!(result.next_action, "apply_refactoring");
        }

        #[test]
        fn test_handle_violating_state_dry_run() {
            let violations = vec![make_test_violation("complexity", "high")];
            let result = handle_violating_state(violations, 0.5, true, true, None).unwrap();
            assert_eq!(result.state, EnforcementState::Violating);
            assert_eq!(result.next_action, "manual_intervention_required");
        }

        #[test]
        fn test_handle_violating_state_no_suggestions() {
            let violations = vec![make_test_violation("complexity", "high")];
            let result = handle_violating_state(violations, 0.5, false, false, None).unwrap();
            assert_eq!(result.state, EnforcementState::Violating);
            assert_eq!(result.next_action, "manual_intervention_required");
        }
    }

    include!("enforce_coverage_part2_checks.rs");
    include!("enforce_coverage_part2_proptest.rs");
