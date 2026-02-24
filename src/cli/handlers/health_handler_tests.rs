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
}
