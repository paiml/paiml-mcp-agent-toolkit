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
            let _ = &violations;
        }

        #[tokio::test]
        async fn test_run_satd_analysis() {
            let temp_dir = create_test_project();
            let profile = make_test_profile();

            let violations = run_satd_analysis(temp_dir.path(), &profile).await.unwrap();
            // May or may not have violations
            let _ = &violations;
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
