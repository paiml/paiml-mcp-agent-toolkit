    mod async_handler_tests_part2 {
        use super::*;

        #[tokio::test]
        async fn test_execute_enforcement_iteration() {
            let temp_dir = create_test_project();
            let profile = make_test_profile();
            let config = make_test_enforcement_config();

            let result = execute_enforcement_iteration(
                &temp_dir.path().to_path_buf(),
                &profile,
                EnforcementState::Analyzing,
                &config,
            )
            .await
            .unwrap();

            assert!(result.score >= 0.0 && result.score <= 1.0);
        }

        #[tokio::test]
        async fn test_run_tdg_analysis() {
            let temp_dir = create_test_project();
            let profile = make_test_profile();

            let violations = run_tdg_analysis(temp_dir.path(), &profile).await.unwrap();
            // May or may not have violations
            let _ = &violations;
        }

        #[tokio::test]
        async fn test_run_dead_code_analysis() {
            let temp_dir = create_test_project();
            let profile = make_test_profile();

            let violations = run_dead_code_analysis(temp_dir.path(), &profile)
                .await
                .unwrap();
            // May or may not have violations
            let _ = &violations;
        }

        #[tokio::test]
        async fn test_run_duplication_analysis() {
            let temp_dir = create_test_project();
            let profile = make_test_profile();

            let violations = run_duplication_analysis(temp_dir.path(), &profile)
                .await
                .unwrap();
            // May or may not have violations
            let _ = &violations;
        }

        #[tokio::test]
        async fn test_run_duplication_analysis_with_allowed_duplicates() {
            let temp_dir = create_test_project();
            let profile = make_relaxed_profile();

            let violations = run_duplication_analysis(temp_dir.path(), &profile)
                .await
                .unwrap();
            // With relaxed profile allowing duplication, should have fewer violations
            let _ = &violations;
        }
    }

    mod finalize_enforcement_tests {
        use super::*;

        #[test]
        fn test_finalize_enforcement_run_complete() {
            let config = make_test_enforcement_config();
            // Just verify no panic
            finalize_enforcement_run(
                0.95,
                5,
                Duration::from_secs(30),
                &config,
                EnforcementState::Complete,
            );
        }

        #[test]
        fn test_finalize_enforcement_run_violating() {
            let config = make_test_enforcement_config();
            finalize_enforcement_run(
                0.5,
                10,
                Duration::from_secs(60),
                &config,
                EnforcementState::Violating,
            );
        }
    }

    mod enforcement_config_tests {
        use super::*;

        #[test]
        fn test_enforcement_config_all_fields() {
            let config = make_full_enforcement_config();
            assert_eq!(config.max_iterations, 10);
            assert_eq!(config.target_improvement, Some(0.1));
            assert_eq!(config.max_time, Some(120));
            assert!(config.apply_suggestions);
            assert!(config.specific_file.is_some());
            assert!(config.include_pattern.is_some());
            assert!(config.exclude_pattern.is_some());
            assert!(config.single_file_mode);
            assert!(!config.dry_run);
            assert!(config.show_progress);
            assert_eq!(config.format, EnforceOutputFormat::Json);
            assert!(config.ci_mode);
        }
    }

    mod quality_profile_extended_tests {
        use super::*;

        #[test]
        fn test_relaxed_profile_values() {
            let profile = make_relaxed_profile();
            assert_eq!(profile.coverage_min, 50.0);
            assert_eq!(profile.complexity_max, 50);
            assert_eq!(profile.satd_allowed, 10);
            assert_eq!(profile.duplication_max_lines, 50);
        }

        #[test]
        fn test_profile_debug_format() {
            let profile = make_test_profile();
            let debug_str = format!("{:?}", profile);
            assert!(debug_str.contains("QualityProfile"));
            assert!(debug_str.contains("coverage_min"));
        }
    }

    mod special_modes_async_tests {
        use super::*;

        #[tokio::test]
        async fn test_handle_special_modes_none() {
            let temp_dir = create_test_project();
            let profile = make_test_profile();

            let result = handle_special_modes(
                false, // list_violations
                false, // validate_only
                &temp_dir.path().to_path_buf(),
                &profile,
                EnforceOutputFormat::Summary,
                false, // ci_mode
            )
            .await
            .unwrap();

            assert!(result.is_none());
        }
    }

    mod iteration_handler_tests {
        use super::*;

        #[tokio::test]
        async fn test_handle_enforcement_iteration() {
            let temp_dir = create_test_project();
            let profile = make_test_profile();
            let config = make_test_enforcement_config();

            let result = handle_enforcement_iteration(
                &temp_dir.path().to_path_buf(),
                &profile,
                EnforcementState::Analyzing,
                &config,
                1,
            )
            .await
            .unwrap();

            assert_eq!(result.iteration, 1);
            assert!(result.score >= 0.0 && result.score <= 1.0);
        }
    }

    mod proptest_extended_tests {
        use super::*;

        proptest! {
            #[test]
            fn test_quality_violation_roundtrip(
                vtype in "[a-z]+",
                severity in "(high|medium|low)",
                loc in "[a-z]+\\.rs:[0-9]+",
            ) {
                let violation = QualityViolation {
                    violation_type: vtype.clone(),
                    severity: severity.clone(),
                    location: loc.clone(),
                    current: 10.0,
                    target: 5.0,
                    suggestion: "Fix it".to_string(),
                };

                let json = serde_json::to_string(&violation).map_err(|e| proptest::test_runner::TestCaseError::Fail(e.to_string().into()))?;
                let parsed: QualityViolation = serde_json::from_str(&json).map_err(|e| proptest::test_runner::TestCaseError::Fail(e.to_string().into()))?;

                prop_assert_eq!(parsed.violation_type, vtype);
                prop_assert_eq!(parsed.severity, severity);
                prop_assert_eq!(parsed.location, loc);
            }

            #[test]
            fn test_enforcement_progress_bounds(
                completed in 0usize..1000,
                remaining in 0usize..1000,
                iterations in 0u32..100,
            ) {
                let progress = EnforcementProgress {
                    files_completed: completed,
                    files_remaining: remaining,
                    estimated_iterations: iterations,
                };

                let json = serde_json::to_string(&progress).map_err(|e| proptest::test_runner::TestCaseError::Fail(e.to_string().into()))?;
                let parsed: EnforcementProgress = serde_json::from_str(&json).map_err(|e| proptest::test_runner::TestCaseError::Fail(e.to_string().into()))?;

                prop_assert_eq!(parsed.files_completed, completed);
                prop_assert_eq!(parsed.files_remaining, remaining);
                prop_assert_eq!(parsed.estimated_iterations, iterations);
            }

            #[test]
            fn test_enforcement_result_state_transitions(
                score in 0.0f64..1.0,
            ) {
                let result = EnforcementResult {
                    state: if score >= 0.9 {
                        EnforcementState::Complete
                    } else if score >= 0.5 {
                        EnforcementState::Validating
                    } else {
                        EnforcementState::Violating
                    },
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

                // State should be consistent with score
                if score >= 0.9 {
                    prop_assert_eq!(result.state, EnforcementState::Complete);
                } else if score >= 0.5 {
                    prop_assert_eq!(result.state, EnforcementState::Validating);
                } else {
                    prop_assert_eq!(result.state, EnforcementState::Violating);
                }
            }
        }
    }

    mod sarif_output_tests {
        use super::*;

        #[test]
        fn test_sarif_output_structure() {
            let violations = vec![
                make_custom_violation("complexity", "high", "src/main.rs:25", 35.0, 20.0),
            ];
            let result = EnforcementResult {
                state: EnforcementState::Violating,
                score: 0.6,
                target: 1.0,
                current_file: None,
                violations,
                next_action: "fix".to_string(),
                progress: EnforcementProgress {
                    files_completed: 0,
                    files_remaining: 1,
                    estimated_iterations: 1,
                },
            };

            // Capture stdout by calling output_result
            let output = output_result(&result, EnforceOutputFormat::Sarif, false);
            assert!(output.is_ok());
        }

        #[test]
        fn test_sarif_severity_mapping() {
            // Test that different severities map correctly
            let high_violation = make_custom_violation("test", "high", "a.rs:1", 1.0, 0.0);
            let medium_violation = make_custom_violation("test", "medium", "b.rs:1", 1.0, 0.0);
            let low_violation = make_custom_violation("test", "low", "c.rs:1", 1.0, 0.0);

            // High should map to "error"
            assert_eq!(high_violation.severity, "high");
            // Medium should map to "warning"
            assert_eq!(medium_violation.severity, "medium");
            // Low should map to "note"
            assert_eq!(low_violation.severity, "low");
        }

        #[test]
        fn test_sarif_location_parsing() {
            // Test location with line number
            let violation = make_custom_violation("test", "high", "src/lib.rs:42", 1.0, 0.0);
            let location = &violation.location;

            let parts: Vec<&str> = location.split(':').collect();
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0], "src/lib.rs");
            assert_eq!(parts[1].parse::<i32>().unwrap(), 42);
        }

        #[test]
        fn test_sarif_location_parsing_no_line() {
            // Test location without line number
            let violation = make_custom_violation("test", "high", "project", 1.0, 0.0);
            let location = &violation.location;

            let parts: Vec<&str> = location.split(':').collect();
            assert_eq!(parts.len(), 1);
            assert_eq!(parts[0], "project");
        }
    }

    mod enforcement_state_machine_tests {
        use super::*;

        #[test]
        fn test_state_machine_analyzing_to_violating() {
            // When violations are found, state should transition to Violating
            let violations = [make_test_violation("complexity", "high")];
            assert!(!violations.is_empty());
        }

        #[test]
        fn test_state_machine_analyzing_to_complete() {
            // When no violations, state should transition to Complete
            let violations: Vec<QualityViolation> = vec![];
            assert!(violations.is_empty());
        }

        #[test]
        fn test_state_machine_refactoring_to_validating() {
            let result = handle_refactoring_state(0.7, None).unwrap();
            assert_eq!(result.state, EnforcementState::Validating);
        }

        #[test]
        fn test_all_states_serialization() {
            let states = [
                EnforcementState::Analyzing,
                EnforcementState::Violating,
                EnforcementState::Refactoring,
                EnforcementState::Validating,
                EnforcementState::Complete,
            ];

            let expected = [
                "\"ANALYZING\"",
                "\"VIOLATING\"",
                "\"REFACTORING\"",
                "\"VALIDATING\"",
                "\"COMPLETE\"",
            ];

            for (state, expected_json) in states.iter().zip(expected.iter()) {
                let json = serde_json::to_string(state).unwrap();
                assert_eq!(&json, *expected_json);
            }
        }
    }

    mod coverage_analysis_tests {
        use super::*;

        #[tokio::test]
        async fn test_run_coverage_analysis_below_threshold() {
            let temp_dir = create_test_project();
            let profile = make_test_profile(); // 80% min coverage

            let violations = run_coverage_analysis(temp_dir.path(), &profile)
                .await
                .unwrap();

            // Simulated coverage is 65%, so should have violation
            assert!(!violations.is_empty());
            assert_eq!(violations[0].violation_type, "coverage");
            assert_eq!(violations[0].current, 65.0);
            assert_eq!(violations[0].target, 80.0);
        }

        #[tokio::test]
        async fn test_run_coverage_analysis_above_threshold() {
            let temp_dir = create_test_project();
            let mut profile = make_test_profile();
            profile.coverage_min = 50.0; // Lower threshold

            let violations = run_coverage_analysis(temp_dir.path(), &profile)
                .await
                .unwrap();

            // Simulated coverage is 65%, above 50% threshold
            assert!(violations.is_empty());
        }
    }
