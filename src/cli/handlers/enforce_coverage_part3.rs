    mod loop_result_tests {
        use super::*;

        #[test]
        fn test_enforcement_loop_result() {
            let result = EnforcementLoopResult {
                final_iteration: 5,
                final_state: EnforcementState::Complete,
                final_score: 0.95,
            };

            assert_eq!(result.final_iteration, 5);
            assert_eq!(result.final_state, EnforcementState::Complete);
            assert_eq!(result.final_score, 0.95);
        }

        #[test]
        fn test_enforcement_iteration_result() {
            let result = EnforcementIterationResult {
                iteration: 3,
                state: EnforcementState::Refactoring,
                score: 0.8,
            };

            assert_eq!(result.iteration, 3);
            assert_eq!(result.state, EnforcementState::Refactoring);
            assert_eq!(result.score, 0.8);
        }
    }

    // ========== Additional Coverage Tests for Uncovered Paths ==========

    mod print_functions_tests {
        use super::*;

        #[test]
        fn test_print_enforcement_header() {
            // Just verify it doesn't panic
            let path = PathBuf::from("/test/project");
            print_enforcement_header(&path);
        }

        #[test]
        fn test_print_enforcement_summary() {
            // Just verify it doesn't panic
            print_enforcement_summary(0.85, 5, Duration::from_secs(10));
        }

        #[test]
        fn test_print_enforcement_summary_zero_values() {
            print_enforcement_summary(0.0, 0, Duration::from_millis(0));
        }

        #[test]
        fn test_print_enforcement_summary_max_values() {
            print_enforcement_summary(1.0, u32::MAX, Duration::from_secs(3600));
        }

        #[test]
        fn test_print_progress_bar_low_score() {
            let result = EnforcementResult {
                state: EnforcementState::Analyzing,
                score: 0.1,
                target: 1.0,
                current_file: None,
                violations: vec![],
                next_action: "test".to_string(),
                progress: EnforcementProgress {
                    files_completed: 0,
                    files_remaining: 10,
                    estimated_iterations: 5,
                },
            };
            print_progress_bar(&result);
        }

        #[test]
        fn test_print_progress_bar_high_score() {
            let result = EnforcementResult {
                state: EnforcementState::Complete,
                score: 0.95,
                target: 1.0,
                current_file: None,
                violations: vec![],
                next_action: "none".to_string(),
                progress: EnforcementProgress {
                    files_completed: 100,
                    files_remaining: 0,
                    estimated_iterations: 0,
                },
            };
            print_progress_bar(&result);
        }

        #[test]
        fn test_print_progress_bar_perfect_score() {
            let result = EnforcementResult {
                state: EnforcementState::Complete,
                score: 1.0,
                target: 1.0,
                current_file: None,
                violations: vec![],
                next_action: "none".to_string(),
                progress: EnforcementProgress {
                    files_completed: 100,
                    files_remaining: 0,
                    estimated_iterations: 0,
                },
            };
            print_progress_bar(&result);
        }
    }

    mod initialization_tests {
        use super::*;

        #[test]
        fn test_initialize_enforcement_environment_no_cache_clear() {
            let result =
                initialize_enforcement_environment("extreme", None, &None, false).unwrap();
            assert_eq!(result.coverage_min, 80.0);
        }

        #[test]
        fn test_initialize_enforcement_environment_with_cache_clear() {
            let temp_dir = TempDir::new().unwrap();
            let cache_path = Some(temp_dir.path().to_path_buf());
            let result =
                initialize_enforcement_environment("extreme", None, &cache_path, true).unwrap();
            assert_eq!(result.coverage_min, 80.0);
        }

        #[test]
        fn test_initialize_enforcement_environment_default_profile() {
            let result = initialize_enforcement_environment("default", None, &None, false).unwrap();
            assert_eq!(result.coverage_min, 80.0);
        }

        #[test]
        fn test_initialize_enforcement_environment_unknown_profile() {
            let result =
                initialize_enforcement_environment("unknown-profile", None, &None, false).unwrap();
            // Should return default profile
            assert_eq!(result.coverage_min, 80.0);
        }
    }

    // ========== State and Control Flow Tests ==========
    include!("enforce_coverage_part3_state_tests.rs");

    // ========== Output Format and Async Handler Tests ==========
    include!("enforce_coverage_part3_output_tests.rs");
