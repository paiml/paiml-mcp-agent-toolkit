#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cfg_not_coverage() {
        let content = r#"
            #[cfg(not(coverage))]
            fn hidden() {}
        "#;

        let mut violations = Vec::new();
        check_cfg_patterns(Path::new("test.rs"), content, &mut violations);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern, GamingPattern::CfgNotCoverage);
        assert_eq!(violations[0].severity, Severity::Critical);
    }

    #[test]
    fn test_detect_cfg_not_tarpaulin() {
        let content = r#"
            #[cfg(not(tarpaulin))]
            mod hidden_tests {}
        "#;

        let mut violations = Vec::new();
        check_cfg_patterns(Path::new("test.rs"), content, &mut violations);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern, GamingPattern::CfgNotTarpaulin);
    }

    #[test]
    fn test_detect_lcov_exclusion() {
        let content = r#"
            // LCOV_EXCL_START
            fn uncovered_code() {
                // This won't be measured
            }
            // LCOV_EXCL_STOP
        "#;

        let mut violations = Vec::new();
        check_exclusion_comments(Path::new("test.rs"), content, &mut violations);

        assert_eq!(violations.len(), 2); // START and STOP
        assert!(violations
            .iter()
            .all(|v| v.pattern == GamingPattern::CoverageExclusionComment));
    }

    #[test]
    fn test_no_false_positives_normal_code() {
        let content = r#"
            fn normal_function() {
                println!("Hello, world!");
            }

            #[cfg_attr(coverage_nightly, coverage(off))]
            #[cfg(test)]
            mod tests {
                #[test]
                fn it_works() {
                    assert!(true);
                }
            }
        "#;

        let mut violations = Vec::new();
        check_cfg_patterns(Path::new("test.rs"), content, &mut violations);
        check_exclusion_comments(Path::new("test.rs"), content, &mut violations);

        assert!(violations.is_empty());
    }

    #[test]
    fn test_meta_falsification() {
        // The meta-check should pass (detector should find the test pattern)
        let result = run_meta_falsification(Path::new(".")).unwrap();
        assert!(result, "Meta-falsification failed: detector is broken!");
    }

    #[test]
    fn test_gaming_result_critical_check() {
        let result = GamingDetectionResult {
            violations: vec![
                GamingViolation {
                    file: PathBuf::from("test.rs"),
                    line: 1,
                    pattern: GamingPattern::CfgNotCoverage,
                    severity: Severity::Critical,
                    explanation: "Test".to_string(),
                },
                GamingViolation {
                    file: PathBuf::from("test2.rs"),
                    line: 1,
                    pattern: GamingPattern::CoverageExclusionComment,
                    severity: Severity::Warning,
                    explanation: "Test".to_string(),
                },
            ],
            files_scanned: 2,
            patterns_checked: vec![],
        };

        assert!(result.has_critical_violations());
        assert_eq!(result.critical_violations().len(), 1);
        assert_eq!(result.count_by_severity(Severity::Critical), 1);
        assert_eq!(result.count_by_severity(Severity::Warning), 1);
    }

    #[test]
    fn test_is_source_file() {
        assert!(is_source_file(Path::new("main.rs")));
        assert!(is_source_file(Path::new("kernel.cu")));
        assert!(is_source_file(Path::new("lib.py")));
        assert!(!is_source_file(Path::new("readme.md")));
        assert!(!is_source_file(Path::new("config.toml")));
    }

    #[test]
    fn test_test_deletion_detection() {
        let mut baseline = HashSet::new();
        baseline.insert(PathBuf::from("tests/unit_test.rs"));
        baseline.insert(PathBuf::from("src/lib.rs"));

        // Simulate: tests/unit_test.rs doesn't exist (deleted)
        // Note: This test assumes the file doesn't exist in the actual filesystem
        let violations = detect_test_deletions(Path::new("/nonexistent"), &baseline);

        // Both files should be flagged as missing (since /nonexistent doesn't exist)
        // But only the test file should be a TestFileDeletion
        let test_deletions: Vec<_> = violations
            .iter()
            .filter(|v| v.pattern == GamingPattern::TestFileDeletion)
            .collect();
        assert_eq!(test_deletions.len(), 1);
    }
}
