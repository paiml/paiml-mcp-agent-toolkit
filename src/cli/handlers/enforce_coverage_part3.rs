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
            // Used to assert that the name "default" resolved successfully to
            // coverage_min == 80.0. That only held because every arm of
            // `load_quality_profile` returned `QualityProfile::default()`, so any
            // string produced the extreme thresholds. "default" is not one of the
            // documented profiles; the *default* profile is the CLI default,
            // `extreme`, and that is what must carry the default thresholds.
            assert!(
                initialize_enforcement_environment("default", None, &None, false).is_err(),
                "\"default\" is not a documented profile name"
            );

            let extreme = initialize_enforcement_environment("extreme", None, &None, false).unwrap();
            assert_eq!(extreme.coverage_min, QualityProfile::default().coverage_min);

            // A named non-default profile must not come back with the default
            // thresholds — that was the whole defect.
            let standard =
                initialize_enforcement_environment("standard", None, &None, false).unwrap();
            assert!(standard.coverage_min < extreme.coverage_min);
        }

        #[test]
        fn test_initialize_enforcement_environment_unknown_profile() {
            // Used to assert an unknown name silently succeeded and yielded the
            // extreme profile (coverage_min 80.0) — a typo'd `--profile` then
            // enforced thresholds the user never asked for. It must now be a hard
            // error that names the valid profiles.
            let err = initialize_enforcement_environment("unknown-profile", None, &None, false)
                .unwrap_err()
                .to_string();
            assert!(
                err.contains("unknown-profile"),
                "error must echo the bad name, got {err}"
            );
            assert!(
                err.contains("standard") && err.contains("strict") && err.contains("extreme"),
                "error must list the valid profiles, got {err}"
            );
        }
    }

    // ========== State and Control Flow Tests ==========
    include!("enforce_coverage_part3_state_tests.rs");

    // ========== Output Format and Async Handler Tests ==========
    include!("enforce_coverage_part3_output_tests.rs");
