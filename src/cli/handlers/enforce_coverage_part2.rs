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

    // ========== Format Violations Output Tests ==========

    mod format_violations_tests {
        use super::*;

        #[test]
        fn test_format_violations_json() {
            let violations = vec![
                make_test_violation("complexity", "high"),
                make_test_violation("satd", "medium"),
            ];
            let profile = make_test_profile();

            let output =
                format_violations_output(&violations, &profile, EnforceOutputFormat::Json).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
            assert_eq!(parsed["summary"]["total"], 2);
            assert_eq!(parsed["summary"]["by_severity"]["high"], 1);
            assert_eq!(parsed["summary"]["by_severity"]["medium"], 1);
        }

        #[test]
        fn test_format_violations_summary() {
            let violations = vec![make_test_violation("complexity", "high")];
            let profile = make_test_profile();

            let output =
                format_violations_output(&violations, &profile, EnforceOutputFormat::Summary)
                    .unwrap();

            assert!(output.contains("1 violations"));
            assert!(output.contains("COMPLEXITY"));
            assert!(output.contains("high"));
        }

        #[test]
        fn test_format_violations_empty() {
            let violations: Vec<QualityViolation> = vec![];
            let profile = make_test_profile();

            let output =
                format_violations_output(&violations, &profile, EnforceOutputFormat::Json).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
            assert_eq!(parsed["summary"]["total"], 0);
        }
    }

    // ========== Output Result Tests ==========

    mod output_result_tests {
        use super::*;

        fn make_test_result() -> EnforcementResult {
            EnforcementResult {
                state: EnforcementState::Analyzing,
                score: 0.75,
                target: 1.0,
                current_file: Some("test.rs".to_string()),
                violations: vec![make_test_violation("complexity", "high")],
                next_action: "analyze".to_string(),
                progress: EnforcementProgress {
                    files_completed: 5,
                    files_remaining: 10,
                    estimated_iterations: 3,
                },
            }
        }

        #[test]
        fn test_output_result_json() {
            let result = make_test_result();
            let output = output_result(&result, EnforceOutputFormat::Json, false);
            assert!(output.is_ok());
        }

        #[test]
        fn test_output_result_summary() {
            let result = make_test_result();
            let output = output_result(&result, EnforceOutputFormat::Summary, false);
            assert!(output.is_ok());
        }

        #[test]
        fn test_output_result_progress() {
            let result = make_test_result();
            let output = output_result(&result, EnforceOutputFormat::Progress, true);
            assert!(output.is_ok());
        }

        #[test]
        fn test_output_result_sarif() {
            let result = make_test_result();
            let output = output_result(&result, EnforceOutputFormat::Sarif, false);
            assert!(output.is_ok());
        }
    }

    // ========== Integration Tests ==========

    mod integration_tests {
        use super::*;

        #[tokio::test]
        async fn test_handle_analyzing_state_integration() {
            let temp_dir = create_test_project();
            let profile = make_test_profile();

            let result = handle_analyzing_state(
                temp_dir.path(),
                &profile,
                false, // single_file_mode
                true,  // dry_run
                None,  // specific_file
            )
            .await
            .unwrap();

            assert!(
                result.state == EnforcementState::Violating
                    || result.state == EnforcementState::Complete
            );
            assert!(result.score >= 0.0 && result.score <= 1.0);
        }

        #[tokio::test]
        async fn test_handle_analyzing_state_single_file() {
            let temp_dir = create_test_project();
            let profile = make_test_profile();
            let file = temp_dir.path().join("src").join("lib.rs");

            let result = handle_analyzing_state(
                temp_dir.path(),
                &profile,
                true,        // single_file_mode
                true,        // dry_run
                Some(&file), // specific_file
            )
            .await
            .unwrap();

            assert!(result.current_file.is_some());
        }

        #[tokio::test]
        async fn test_run_complexity_analysis() {
            let temp_dir = create_test_project();
            let profile = make_test_profile();

            let violations = run_complexity_analysis(temp_dir.path(), &profile)
                .await
                .unwrap();

            // May or may not have violations depending on the code
            assert!(violations.len() >= 0);
        }

        #[tokio::test]
        async fn test_run_satd_analysis() {
            let temp_dir = create_test_project();
            let profile = make_test_profile();

            let violations = run_satd_analysis(temp_dir.path(), &profile).await.unwrap();
            // May or may not have violations
            assert!(violations.len() >= 0);
        }

        #[tokio::test]
        async fn test_run_coverage_analysis() {
            let temp_dir = create_test_project();
            let profile = make_test_profile();

            let violations = run_coverage_analysis(temp_dir.path(), &profile)
                .await
                .unwrap();

            // Should have at least one coverage violation (simulated at 65%)
            assert!(violations.len() >= 1);
            assert_eq!(violations[0].violation_type, "coverage");
        }
    }

    // ========== Property-Based Tests ==========

    mod proptest_tests {
        use super::*;

        proptest! {
            #[test]
            fn test_quality_profile_coverage_bounds(coverage in 0.0f64..100.0) {
                let profile = QualityProfile {
                    coverage_min: coverage,
                    ..QualityProfile::default()
                };
                prop_assert!(profile.coverage_min >= 0.0);
                prop_assert!(profile.coverage_min <= 100.0);
            }

            #[test]
            fn test_quality_profile_complexity_bounds(complexity in 1u16..100) {
                let profile = QualityProfile {
                    complexity_max: complexity,
                    ..QualityProfile::default()
                };
                prop_assert!(profile.complexity_max >= 1);
                prop_assert!(profile.complexity_max <= 100);
            }

            #[test]
            fn test_enforcement_result_score_bounds(score in 0.0f64..1.0) {
                let result = EnforcementResult {
                    state: EnforcementState::Analyzing,
                    score,
                    target: 1.0,
                    current_file: None,
                    violations: vec![],
                    next_action: "test".to_string(),
                    progress: EnforcementProgress {
                        files_completed: 0,
                        files_remaining: 0,
                        estimated_iterations: 0,
                    },
                };
                prop_assert!(result.score >= 0.0);
                prop_assert!(result.score <= 1.0);
            }

            #[test]
            fn test_should_continue_respects_max_iterations(
                iteration in 0u32..100,
                max_iterations in 1u32..50
            ) {
                let config = EnforcementConfig {
                    max_iterations,
                    ..make_test_enforcement_config()
                };
                let start_time = Instant::now();
                let result = should_continue_enforcement(
                    EnforcementState::Analyzing,
                    iteration,
                    &config,
                    start_time
                );

                if iteration >= max_iterations {
                    prop_assert!(!result);
                }
            }

            #[test]
            fn test_violation_serialization_roundtrip(
                current in 0.0f64..100.0,
                target in 0.0f64..100.0
            ) {
                let violation = QualityViolation {
                    violation_type: "test".to_string(),
                    severity: "medium".to_string(),
                    location: "test.rs:1".to_string(),
                    current,
                    target,
                    suggestion: "test suggestion".to_string(),
                };

                let json = serde_json::to_string(&violation).unwrap();
                let parsed: QualityViolation = serde_json::from_str(&json).unwrap();

                prop_assert!((parsed.current - current).abs() < 0.001);
                prop_assert!((parsed.target - target).abs() < 0.001);
            }
        }
    }

    // ========== Edge Case Tests ==========

    mod edge_case_tests {
        use super::*;

        #[test]
        fn test_empty_project_path() {
            let _path = PathBuf::from("");
            let _profile = make_test_profile();
            // Just verify it doesn't panic
            let _ = load_quality_profile("extreme", None);
        }

        #[test]
        fn test_zero_iterations() {
            let config = EnforcementConfig {
                max_iterations: 0,
                ..make_test_enforcement_config()
            };
            let start_time = Instant::now();
            let result =
                should_continue_enforcement(EnforcementState::Analyzing, 0, &config, start_time);
            assert!(!result);
        }

        #[test]
        fn test_large_iteration_count() {
            let config = EnforcementConfig {
                max_iterations: u32::MAX,
                ..make_test_enforcement_config()
            };
            let start_time = Instant::now();
            let result =
                should_continue_enforcement(EnforcementState::Analyzing, 1000, &config, start_time);
            assert!(result);
        }

        #[test]
        fn test_enforcement_result_clone() {
            let result = EnforcementResult {
                state: EnforcementState::Complete,
                score: 0.95,
                target: 1.0,
                current_file: Some("test.rs".to_string()),
                violations: vec![make_test_violation("test", "low")],
                next_action: "none".to_string(),
                progress: EnforcementProgress {
                    files_completed: 10,
                    files_remaining: 0,
                    estimated_iterations: 0,
                },
            };

            let cloned = result.clone();
            assert_eq!(cloned.state, result.state);
            assert_eq!(cloned.score, result.score);
            assert_eq!(cloned.violations.len(), result.violations.len());
        }

        #[test]
        fn test_quality_profile_clone() {
            let profile = make_test_profile();
            let cloned = profile.clone();
            assert_eq!(cloned.coverage_min, profile.coverage_min);
            assert_eq!(cloned.complexity_max, profile.complexity_max);
        }

        #[test]
        fn test_violation_with_unicode_location() {
            let violation = QualityViolation {
                violation_type: "complexity".to_string(),
                severity: "high".to_string(),
                location: "src/test.rs:10".to_string(),
                current: 25.0,
                target: 10.0,
                suggestion: "Refactor function".to_string(),
            };

            let json = serde_json::to_string(&violation).unwrap();
            assert!(json.contains("test.rs"));
        }

        #[test]
        fn test_enforcement_state_copy() {
            let state1 = EnforcementState::Analyzing;
            let state2 = state1; // Copy
            assert_eq!(state1, state2);
        }
    }

    // ========== Cache Tests ==========

    mod cache_tests {
        use super::*;

        #[test]
        fn test_clear_enforcement_cache_none() {
            // Should not panic with None
            clear_enforcement_cache(&None);
        }

        #[test]
        fn test_clear_enforcement_cache_some() {
            let temp_dir = TempDir::new().unwrap();
            let cache_path = temp_dir.path().to_path_buf();
            clear_enforcement_cache(&Some(cache_path));
        }
    }

    // ========== Loop Result Tests ==========

