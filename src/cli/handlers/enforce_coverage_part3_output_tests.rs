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
            .unwrap();

            assert!(
                result.state == EnforcementState::Complete
                    || result.state == EnforcementState::Violating
            );
        }
    }
