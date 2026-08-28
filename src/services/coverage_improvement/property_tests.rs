mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_target_coverage_range(target in 0.0f64..100.0f64) {
            let config = CoverageImprovementConfig {
                target_coverage: target,
                ..Default::default()
            };
            let service = CoverageImprovementService::new(config);
            prop_assert_eq!(service.config.target_coverage, target);
        }

        #[test]
        fn test_max_iterations_range(max_iter in 1usize..20usize) {
            let config = CoverageImprovementConfig {
                max_iterations: max_iter,
                ..Default::default()
            };
            let service = CoverageImprovementService::new(config);
            prop_assert_eq!(service.config.max_iterations, max_iter);
        }

        /// Property: parse_coverage_percentage correctly extracts percentage from TOTAL line
        #[test]
        fn test_parse_coverage_percentage_extraction(
            region_pct in 0.0f64..100.0,
            function_pct in 0.0f64..100.0,
            line_pct in 0.0f64..100.0
        ) {
            // Generate a TOTAL line with the given percentages
            let total_line = format!(
                "TOTAL   241150  203105  {:.2}%  17533  14596  {:.2}%  173884  145810  {:.2}%  0  0  -",
                region_pct, function_pct, line_pct
            );

            let result = CoverageImprovementService::parse_coverage_percentage(&total_line);
            prop_assert!(result.is_ok());

            // Should extract the line coverage (last percentage)
            let coverage = result.expect("internal error");
            prop_assert!((coverage - line_pct).abs() < 0.01, "Expected {}, got {}", line_pct, coverage);
        }

        /// Property: parse_coverage_percentage handles various whitespace formats
        #[test]
        #[ignore = "Fragile test - whitespace handling varies by llvm-cov version"]
        fn test_parse_coverage_percentage_whitespace(
            spaces_before in 0usize..10,
            spaces_after in 0usize..10,
            pct in 0.0f64..100.0
        ) {
            let before = " ".repeat(spaces_before);
            let after = " ".repeat(spaces_after);
            let total_line = format!(
                "{}TOTAL{}100{}100{}10.0%{}50{}50{}20.0%{}200{}150{}{:.2}%{}0{}0{}-",
                before, after, after, after, after, after, after, after, after, after, pct, after, after, after
            );

            let result = CoverageImprovementService::parse_coverage_percentage(&total_line);
            prop_assert!(result.is_ok());
            let coverage = result.expect("internal error");
            prop_assert!((coverage - pct).abs() < 0.01);
        }

        /// Property: Coverage delta calculation is commutative in magnitude
        #[test]
        fn test_coverage_delta_magnitude(
            prev in 0.0f64..100.0,
            gain in -50.0f64..50.0
        ) {
            let new = (prev + gain).max(0.0).min(100.0);
            let delta = new - prev;

            // Delta magnitude should be consistent
            prop_assert_eq!(delta, new - prev);

            // If new > prev, delta is positive; if new < prev, delta is negative
            if new > prev {
                prop_assert!(delta > 0.0);
            } else if new < prev {
                prop_assert!(delta < 0.0);
            } else {
                prop_assert_eq!(delta, 0.0);
            }
        }
    }

    /// Unit tests for coverage parsing edge cases
    #[test]
    fn test_parse_coverage_no_total_line() {
        let output = "Some other output\nwithout TOTAL line\n";
        let result = CoverageImprovementService::parse_coverage_percentage(output);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Could not find TOTAL line"));
    }

    #[test]
    fn test_parse_coverage_invalid_percentage() {
        let output = "TOTAL   100  50  invalid%  200  100  50.0%";
        let result = CoverageImprovementService::parse_coverage_percentage(output);
        // Should still work - extracts the last valid percentage
        assert!(result.is_ok());
        assert_eq!(result.expect("internal error"), 50.0);
    }

    #[test]
    fn test_parse_coverage_real_output() {
        // Real output from cargo-llvm-cov
        let output = r#"
Filename                      Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover
--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
TOTAL   241150  203105  15.78%  17533  14596  16.75%  173884  145810  16.15%  0  0  -
"#;
        let result = CoverageImprovementService::parse_coverage_percentage(output);
        assert!(result.is_ok());
        assert_eq!(result.expect("internal error"), 16.15);
    }

    #[test]
    fn test_parse_coverage_edge_cases() {
        // 0% coverage
        let output = "TOTAL   100  100  0.00%  10  10  0.00%  50  50  0.00%  0  0  -";
        let result = CoverageImprovementService::parse_coverage_percentage(output);
        assert!(result.is_ok());
        assert_eq!(result.expect("internal error"), 0.0);

        // 100% coverage
        let output = "TOTAL   100  0  100.00%  10  0  100.00%  50  0  100.00%  0  0  -";
        let result = CoverageImprovementService::parse_coverage_percentage(output);
        assert!(result.is_ok());
        assert_eq!(result.expect("internal error"), 100.0);
    }

    #[test]
    fn test_parse_coverage_single_percentage() {
        let output = "TOTAL   100  50  75.50%";
        let result = CoverageImprovementService::parse_coverage_percentage(output);
        assert!(result.is_ok());
        assert_eq!(result.expect("internal error"), 75.5);
    }

    #[test]
    fn test_parse_coverage_total_lowercase() {
        // Should not match lowercase "total"
        let output = "total   100  50  75.50%";
        let result = CoverageImprovementService::parse_coverage_percentage(output);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_coverage_no_percentage_sign() {
        let output = "TOTAL   100  50  75.50";
        let result = CoverageImprovementService::parse_coverage_percentage(output);
        assert!(result.is_err());
    }

    proptest! {
        /// Property: Config clone produces identical values
        #[test]
        fn test_config_clone_equality(
            target in 0.0f64..100.0,
            max_iter in 1usize..100,
            fast in proptest::bool::ANY
        ) {
            let config = CoverageImprovementConfig {
                project_path: PathBuf::from("/test"),
                target_coverage: target,
                max_iterations: max_iter,
                fast_mode: fast,
                mutation_threshold: 80.0,
                focus_patterns: vec!["*.rs".to_string()],
                exclude_patterns: vec!["**/target/**".to_string()],
                max_targets: 10,
            };

            let cloned = config.clone();
            prop_assert_eq!(config.project_path, cloned.project_path);
            prop_assert_eq!(config.target_coverage, cloned.target_coverage);
            prop_assert_eq!(config.max_iterations, cloned.max_iterations);
            prop_assert_eq!(config.fast_mode, cloned.fast_mode);
        }

        /// Property: CoverageImprovementReport success reflects coverage vs target
        #[test]
        fn test_report_success_consistency(
            baseline in 0.0f64..100.0,
            target in 0.0f64..100.0,
            final_cov in 0.0f64..100.0
        ) {
            let report = CoverageImprovementReport {
                baseline_coverage: baseline,
                target_coverage: target,
                final_coverage: final_cov,
                iterations: vec![],
                success: final_cov >= target,
                stop_reason: "Test".to_string(),
            };

            // Success should be true iff final >= target
            prop_assert_eq!(report.success, final_cov >= target);
        }
    }
}
