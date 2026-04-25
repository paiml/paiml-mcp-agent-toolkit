#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_summary_all_pass() {
        let checks = vec![
            HealthCheck {
                name: "Test1".to_string(),
                status: CheckStatus::Pass,
                message: "OK".to_string(),
                details: None,
            },
            HealthCheck {
                name: "Test2".to_string(),
                status: CheckStatus::Pass,
                message: "OK".to_string(),
                details: None,
            },
        ];

        let summary = calculate_summary(&checks);
        assert_eq!(summary.total_checks, 2);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 0);
    }

    #[test]
    fn test_calculate_summary_mixed() {
        let checks = vec![
            HealthCheck {
                name: "Test1".to_string(),
                status: CheckStatus::Pass,
                message: "OK".to_string(),
                details: None,
            },
            HealthCheck {
                name: "Test2".to_string(),
                status: CheckStatus::Warn,
                message: "Warning".to_string(),
                details: None,
            },
            HealthCheck {
                name: "Test3".to_string(),
                status: CheckStatus::Fail,
                message: "Failed".to_string(),
                details: None,
            },
        ];

        let summary = calculate_summary(&checks);
        assert_eq!(summary.total_checks, 3);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.warned, 1);
        assert_eq!(summary.failed, 1);
    }

    #[test]
    fn test_parse_coverage_valid() {
        let output = "Filename                      Regions    Missed Regions     Cover   Functions  Missed Functions  Executed\n\
                      TOTAL   1234   234   81.0%";
        let coverage = parse_coverage_percentage(output);
        assert_eq!(coverage, 81.0);
    }

    #[test]
    fn test_parse_coverage_invalid() {
        let output = "No coverage data";
        let coverage = parse_coverage_percentage(output);
        assert_eq!(coverage, 0.0);
    }

    #[test]
    fn test_determine_checks_quick_mode() {
        let checks = determine_checks_to_run(true, false, false, false, false, false, false);
        assert!(checks.build);
        assert!(!checks.tests);
        assert!(!checks.coverage);
        assert!(!checks.complexity);
        assert!(!checks.satd);
    }

    #[test]
    fn test_determine_checks_all_mode() {
        let checks = determine_checks_to_run(false, true, false, false, false, false, false);
        assert!(checks.build);
        assert!(checks.tests);
        assert!(checks.coverage);
        assert!(checks.complexity);
        assert!(checks.satd);
    }

    #[test]
    fn test_determine_checks_default_no_flags() {
        let checks = determine_checks_to_run(false, false, false, false, false, false, false);
        assert!(checks.build);
        assert!(!checks.tests);
        assert!(!checks.coverage);
        assert!(!checks.complexity);
        assert!(!checks.satd);
    }

    #[test]
    fn test_determine_checks_specific_flags() {
        let checks = determine_checks_to_run(false, false, true, true, false, false, false);
        assert!(checks.build);
        assert!(checks.tests);
        assert!(!checks.coverage);
        assert!(!checks.complexity);
        assert!(!checks.satd);
    }

    #[test]
    fn test_determine_checks_quick_overrides_all() {
        let checks = determine_checks_to_run(true, true, false, false, false, false, false);
        assert!(checks.build);
        assert!(!checks.tests);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn summary_totals_match(passed in 0u32..100, warned in 0u32..100, failed in 0u32..100, skipped in 0u32..100) {
            let mut checks = Vec::new();

            for _ in 0..passed {
                checks.push(HealthCheck {
                    name: "Pass".to_string(),
                    status: CheckStatus::Pass,
                    message: "OK".to_string(),
                    details: None,
                });
            }

            for _ in 0..warned {
                checks.push(HealthCheck {
                    name: "Warn".to_string(),
                    status: CheckStatus::Warn,
                    message: "Warning".to_string(),
                    details: None,
                });
            }

            for _ in 0..failed {
                checks.push(HealthCheck {
                    name: "Fail".to_string(),
                    status: CheckStatus::Fail,
                    message: "Failed".to_string(),
                    details: None,
                });
            }

            for _ in 0..skipped {
                checks.push(HealthCheck {
                    name: "Skip".to_string(),
                    status: CheckStatus::Skip,
                    message: "Skipped".to_string(),
                    details: None,
                });
            }

            let summary = calculate_summary(&checks);

            prop_assert_eq!(summary.total_checks, checks.len());
            prop_assert_eq!(summary.passed, passed as usize);
            prop_assert_eq!(summary.warned, warned as usize);
            prop_assert_eq!(summary.failed, failed as usize);
            prop_assert_eq!(summary.skipped, skipped as usize);
        }
    }
}

// TICKET-PMAT-6010: Tests for parallel health check execution
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod parallel_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Slow test (115s) - zero tolerance for slow tests in coverage
    async fn test_run_checks_parallel_returns_all_results() {
        let project_dir = PathBuf::from(".");
        let check_types = vec![CheckType::Build, CheckType::Complexity, CheckType::Satd];

        let results = run_checks_parallel(&project_dir, check_types).await;

        assert!(results.is_ok());
        let checks = results.unwrap();
        assert_eq!(checks.len(), 3);

        // Verify all check names are present
        let names: Vec<_> = checks.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"Build"));
        assert!(names.contains(&"Complexity"));
        assert!(names.contains(&"SATD"));
    }

    #[tokio::test]
    async fn test_run_checks_parallel_empty_list() {
        let project_dir = PathBuf::from(".");
        let check_types = vec![];

        let results = run_checks_parallel(&project_dir, check_types).await;

        assert!(results.is_ok());
        let checks = results.unwrap();
        assert_eq!(checks.len(), 0);
    }

    /// SLOW: 100s - excluded from fast test suite
    #[tokio::test]
    #[ignore = "requires health check setup"]
    async fn test_run_checks_parallel_single_check() {
        let project_dir = PathBuf::from(".");
        let check_types = vec![CheckType::Build];

        let results = run_checks_parallel(&project_dir, check_types).await;

        assert!(results.is_ok());
        let checks = results.unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].name, "Build");
    }

    #[test]
    fn test_check_type_coverage() {
        // Verify CheckType enum has all expected variants
        let types = [
            CheckType::Build,
            CheckType::Tests,
            CheckType::Coverage,
            CheckType::Complexity,
            CheckType::Satd,
        ];

        // If this compiles, all types exist
        assert_eq!(types.len(), 5);
    }

    // ─────────────────────────────────────────────────────────────────────
    // print_health_report / print_health_table / print_health_yaml smoke
    //
    // These print to stderr/stdout with no return value, so we cover them
    // for line counts (no panic) under each format arm + edge case.
    // health_handler_output.rs:41-145.
    // ─────────────────────────────────────────────────────────────────────

    fn make_check(name: &str, status: CheckStatus, with_details: bool) -> HealthCheck {
        HealthCheck {
            name: name.to_string(),
            status,
            message: format!("{name} message"),
            details: if with_details {
                Some("some/path".to_string())
            } else {
                None
            },
        }
    }

    fn make_report(checks: Vec<HealthCheck>, healthy: bool) -> HealthReport {
        let summary = calculate_summary(&checks);
        HealthReport {
            healthy,
            checks,
            summary,
        }
    }

    #[test]
    fn test_print_health_report_json_arm() {
        let report = make_report(vec![make_check("Build", CheckStatus::Pass, true)], true);
        // OutputFormat::Json arm — serde_json round-trip, no panic.
        print_health_report(&report, &OutputFormat::Json).unwrap();
    }

    #[test]
    fn test_print_health_report_yaml_arm() {
        let report = make_report(
            vec![make_check("Build", CheckStatus::Pass, false)],
            true,
        );
        print_health_report(&report, &OutputFormat::Yaml).unwrap();
    }

    #[test]
    fn test_print_health_report_table_arm() {
        let report = make_report(
            vec![make_check("Build", CheckStatus::Pass, false)],
            true,
        );
        print_health_report(&report, &OutputFormat::Table).unwrap();
    }

    #[test]
    fn test_print_health_report_default_arm_for_unknown_format() {
        // Any other OutputFormat falls through to print_health_table.
        let report = make_report(
            vec![make_check("Build", CheckStatus::Pass, false)],
            true,
        );
        print_health_report(&report, &OutputFormat::Csv).unwrap();
    }

    #[test]
    fn test_print_health_table_with_all_4_status_arms_and_details() {
        // Covers all CheckStatus icon arms + the `details` Some branch.
        let checks = vec![
            make_check("P", CheckStatus::Pass, true),
            make_check("W", CheckStatus::Warn, true),
            make_check("F", CheckStatus::Fail, true),
            make_check("S", CheckStatus::Skip, true),
        ];
        let report = make_report(checks, false);
        print_health_table(&report);
    }

    #[test]
    fn test_print_health_table_healthy_arm() {
        // healthy = true → "Project is healthy!" branch.
        let report = make_report(
            vec![make_check("P", CheckStatus::Pass, false)],
            true,
        );
        print_health_table(&report);
    }

    #[test]
    fn test_print_health_table_unhealthy_arm() {
        // healthy = false → "Project has N issue(s)" branch.
        let report = make_report(
            vec![make_check("F", CheckStatus::Fail, false)],
            false,
        );
        print_health_table(&report);
    }

    #[test]
    fn test_print_health_yaml_with_details_some_and_none() {
        // Covers both `details: Some(...)` and `details: None` branches in
        // the YAML printer's per-check loop.
        let checks = vec![
            make_check("with-details", CheckStatus::Pass, true),
            make_check("no-details", CheckStatus::Skip, false),
        ];
        let report = make_report(checks, true);
        print_health_yaml(&report);
    }

    #[test]
    fn test_print_health_yaml_empty_checks() {
        let report = make_report(vec![], true);
        print_health_yaml(&report);
    }
}

// ── Wave 35 PR2: R5 status-classifier tests for health_handler_checks.rs ──

#[cfg(test)]
mod r5_classifier_tests {
    use super::*;

    // ── classify_coverage_status ────────────────────────────────────────────

    #[test]
    fn test_classify_coverage_at_80_passes() {
        // Boundary: >= 80 → Pass
        assert_eq!(classify_coverage_status(80.0), CheckStatus::Pass);
    }

    #[test]
    fn test_classify_coverage_above_80_passes() {
        assert_eq!(classify_coverage_status(95.5), CheckStatus::Pass);
        assert_eq!(classify_coverage_status(100.0), CheckStatus::Pass);
    }

    #[test]
    fn test_classify_coverage_at_60_warns() {
        // Boundary: >= 60 && < 80 → Warn
        assert_eq!(classify_coverage_status(60.0), CheckStatus::Warn);
    }

    #[test]
    fn test_classify_coverage_between_60_and_80_warns() {
        assert_eq!(classify_coverage_status(65.0), CheckStatus::Warn);
        assert_eq!(classify_coverage_status(79.9), CheckStatus::Warn);
    }

    #[test]
    fn test_classify_coverage_below_60_fails() {
        assert_eq!(classify_coverage_status(0.0), CheckStatus::Fail);
        assert_eq!(classify_coverage_status(59.9), CheckStatus::Fail);
    }

    // ── classify_complexity_status ──────────────────────────────────────────

    #[test]
    fn test_classify_complexity_zero_violations_passes() {
        assert_eq!(classify_complexity_status(0), CheckStatus::Pass);
    }

    #[test]
    fn test_classify_complexity_one_to_five_warns() {
        // Inclusive range 1..=5 → Warn
        for v in 1..=5 {
            assert_eq!(classify_complexity_status(v), CheckStatus::Warn);
        }
    }

    #[test]
    fn test_classify_complexity_six_or_more_fails() {
        assert_eq!(classify_complexity_status(6), CheckStatus::Fail);
        assert_eq!(classify_complexity_status(100), CheckStatus::Fail);
    }

    // ── classify_satd_status ────────────────────────────────────────────────

    #[test]
    fn test_classify_satd_zero_total_passes() {
        // Even high_severity > 0 with total == 0 is unreachable; total dominates
        assert_eq!(classify_satd_status(0, 0), CheckStatus::Pass);
    }

    #[test]
    fn test_classify_satd_with_items_no_high_severity_warns() {
        // Items present, but no high-severity → Warn
        assert_eq!(classify_satd_status(5, 0), CheckStatus::Warn);
        assert_eq!(classify_satd_status(100, 0), CheckStatus::Warn);
    }

    #[test]
    fn test_classify_satd_with_high_severity_fails() {
        // Any high-severity (with total > 0) → Fail
        assert_eq!(classify_satd_status(1, 1), CheckStatus::Fail);
        assert_eq!(classify_satd_status(10, 3), CheckStatus::Fail);
    }
}
