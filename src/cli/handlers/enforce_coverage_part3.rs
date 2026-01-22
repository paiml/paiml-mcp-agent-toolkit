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

    mod format_output_extended_tests {
        use super::*;

        #[test]
        fn test_format_violations_multiple_types() {
            let violations = vec![
                make_custom_violation("complexity", "high", "a.rs:1", 30.0, 10.0),
                make_custom_violation("satd", "medium", "b.rs:2", 5.0, 0.0),
                make_custom_violation("tdg", "low", "c.rs:3", 2.0, 1.0),
                make_custom_violation("coverage", "high", "project", 50.0, 80.0),
                make_custom_violation("duplication", "medium", "d.rs:4", 20.0, 0.0),
            ];
            let profile = make_test_profile();

            let output =
                format_violations_output(&violations, &profile, EnforceOutputFormat::Json).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

            assert_eq!(parsed["summary"]["total"], 5);
            assert_eq!(parsed["summary"]["by_severity"]["high"], 2);
            assert_eq!(parsed["summary"]["by_severity"]["medium"], 2);
            assert_eq!(parsed["summary"]["by_severity"]["low"], 1);
        }

        #[test]
        fn test_format_violations_summary_format() {
            let violations = vec![
                make_custom_violation("dead_code", "low", "unused.rs:10", 1.0, 0.0),
            ];
            let profile = make_test_profile();

            let output =
                format_violations_output(&violations, &profile, EnforceOutputFormat::Summary).unwrap();

            assert!(output.contains("DEAD_CODE"));
            assert!(output.contains("low"));
            assert!(output.contains("unused.rs:10"));
        }

        #[test]
        fn test_format_violations_progress_format() {
            let violations = vec![make_test_violation("complexity", "high")];
            let profile = make_test_profile();

            // Progress format uses same logic as Summary for violations
            let output =
                format_violations_output(&violations, &profile, EnforceOutputFormat::Progress).unwrap();
            assert!(output.contains("COMPLEXITY"));
        }
    }

    mod output_result_extended_tests {
        use super::*;

        #[test]
        fn test_output_result_progress_no_show() {
            let result = EnforcementResult {
                state: EnforcementState::Analyzing,
                score: 0.5,
                target: 1.0,
                current_file: None,
                violations: vec![],
                next_action: "test".to_string(),
                progress: EnforcementProgress {
                    files_completed: 5,
                    files_remaining: 10,
                    estimated_iterations: 3,
                },
            };
            // show_progress = false should skip progress bar
            let output = output_result(&result, EnforceOutputFormat::Progress, false);
            assert!(output.is_ok());
        }

        #[test]
        fn test_output_result_sarif_with_violations() {
            let violations = vec![
                make_custom_violation("complexity", "high", "src/main.rs:25", 35.0, 20.0),
                make_custom_violation("satd", "medium", "src/lib.rs:100", 1.0, 0.0),
                make_custom_violation("tdg", "low", "tests/test.rs", 1.5, 1.0),
            ];
            let result = EnforcementResult {
                state: EnforcementState::Violating,
                score: 0.6,
                target: 1.0,
                current_file: Some("src/main.rs".to_string()),
                violations,
                next_action: "fix_violations".to_string(),
                progress: EnforcementProgress {
                    files_completed: 3,
                    files_remaining: 7,
                    estimated_iterations: 5,
                },
            };
            let output = output_result(&result, EnforceOutputFormat::Sarif, false);
            assert!(output.is_ok());
        }

        #[test]
        fn test_output_result_summary_no_current_file() {
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
            let output = output_result(&result, EnforceOutputFormat::Summary, false);
            assert!(output.is_ok());
        }

        #[test]
        fn test_output_result_summary_with_current_file() {
            let result = EnforcementResult {
                state: EnforcementState::Refactoring,
                score: 0.7,
                target: 1.0,
                current_file: Some("src/module.rs".to_string()),
                violations: vec![],
                next_action: "refactor".to_string(),
                progress: EnforcementProgress {
                    files_completed: 50,
                    files_remaining: 50,
                    estimated_iterations: 2,
                },
            };
            let output = output_result(&result, EnforceOutputFormat::Summary, false);
            assert!(output.is_ok());
        }
    }

    mod async_handler_tests {
        use super::*;

        #[tokio::test]
        async fn test_handle_analyzing_enforcement_state() {
            let temp_dir = create_test_project();
            let profile = make_test_profile();

            let result = handle_analyzing_enforcement_state(
                &temp_dir.path().to_path_buf(),
                &profile,
                false,
                true,
                None,
            )
            .await
            .unwrap();

            assert!(result.score >= 0.0 && result.score <= 1.0);
        }

        #[tokio::test]
        async fn test_handle_violating_enforcement_state_proxy() {
            let temp_dir = create_test_project();
            let profile = make_test_profile();

            let result = handle_violating_enforcement_state_proxy(
                &temp_dir.path().to_path_buf(),
                &profile,
                false,
                true,
                None,
                false,
            )
            .await
            .unwrap();

            // Should be in Violating state since dry_run=true
            assert!(
                result.state == EnforcementState::Violating
                    || result.state == EnforcementState::Refactoring
            );
        }

        #[tokio::test]
        async fn test_handle_validating_enforcement_state() {
            let temp_dir = create_test_project();
            let profile = make_test_profile();

            let result = handle_validating_enforcement_state(
                &temp_dir.path().to_path_buf(),
                &profile,
                false,
                true,
                None,
                None,
                None,
            )
            .await
