
    #[test]
    fn test_format_sarif_with_violations() {
        let mut result = create_full_test_result();
        result.quality_gate.violations = vec![
            QualityViolation {
                rule: "max_defect_density".to_string(),
                threshold: 0.05,
                actual: 0.10,
                severity: "blocking".to_string(),
            },
            QualityViolation {
                rule: "max_single_file_violations".to_string(),
                threshold: 50.0,
                actual: 60.0,
                severity: "warning".to_string(),
            },
        ];

        let output = format_sarif(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        // #701: this used to assert `results.len() == 2`, i.e. that SARIF
        // carried only the two quality-gate breaches. That is the behaviour
        // format_sarif was fixed *away* from: building the document out of
        // gate breaches alone made a passing gate emit `"results": []` while
        // `-f detailed` listed the located clippy findings for the same run.
        // The located findings come first, the gate breaches ride along after
        // them. The fragment was never compiled, so it kept asserting the old
        // shape unchallenged.
        let results = parsed["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results.len(), result.all_violations.len() + 2);

        // Located findings first, each with a region.
        assert_eq!(results[0]["ruleId"], "unused_variable");
        assert!(results[0]["locations"][0]["physicalLocation"]["region"].is_object());

        // Then the gate breaches: blocking maps to error, everything else to warning.
        let gate = &results[result.all_violations.len()..];
        assert_eq!(gate[0]["level"], "error");
        assert_eq!(gate[0]["ruleId"], "max_defect_density");
        assert_eq!(gate[1]["level"], "warning");
        assert_eq!(gate[1]["ruleId"], "max_single_file_violations");
    }

    // LintHotspotResult Tests

    #[test]
    fn test_lint_hotspot_result_serialization() {
        let result = create_full_test_result();
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: LintHotspotResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.hotspot.file, result.hotspot.file);
        assert_eq!(
            deserialized.total_project_violations,
            result.total_project_violations
        );
        assert_eq!(
            deserialized.quality_gate.passed,
            result.quality_gate.passed
        );
    }

    #[test]
    fn test_lint_hotspot_result_with_all_fields() {
        let mut result = create_full_test_result();
        result.enforcement = Some(EnforcementMetadata {
            enforcement_score: 8.5,
            requires_enforcement: true,
            estimated_fix_time: 1800,
            automation_confidence: 0.85,
            enforcement_priority: 3,
        });
        result.refactor_chain = Some(RefactorChain {
            id: "test".to_string(),
            estimated_reduction: 10,
            automation_confidence: 0.9,
            steps: vec![],
        });

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: LintHotspotResult = serde_json::from_str(&json).unwrap();

        assert!(deserialized.enforcement.is_some());
        assert!(deserialized.refactor_chain.is_some());
    }

    // recalculate_hotspot_metrics Tests

    #[test]
    fn test_recalculate_hotspot_metrics() {
        let mut result = create_full_test_result();
        result.hotspot.detailed_violations = vec![
            create_test_violation("src/main.rs", 10, "lint_a", "warning"),
            create_test_violation("src/main.rs", 20, "lint_b", "warning"),
        ];
        result.hotspot.sloc = 100;

        recalculate_hotspot_metrics(&mut result);

        assert_eq!(result.hotspot.total_violations, 2);
        assert!((result.hotspot.defect_density - 0.02).abs() < f64::EPSILON);
    }

    #[test]
    fn test_recalculate_hotspot_metrics_zero_sloc() {
        let mut result = create_full_test_result();
        result.hotspot.detailed_violations = vec![
            create_test_violation("src/main.rs", 10, "lint_a", "warning"),
        ];
        result.hotspot.sloc = 0;

        recalculate_hotspot_metrics(&mut result);

        assert_eq!(result.hotspot.total_violations, 1);
        // defect_density should not change when sloc is 0
    }

    // should_exit_with_error Tests (comprehensive)

    #[test]
    fn test_should_exit_with_error_comprehensive() {
        let mut result = create_full_test_result();
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test"),
            file: None,
            format: LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.7,
            enforce: false,
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: String::new(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        // Quality gate passed, no enforce
        result.quality_gate.passed = true;
        result.total_project_violations = 10;
        assert!(!should_exit_with_error(&result, &params));

        // Quality gate failed
        result.quality_gate.passed = false;
        assert!(should_exit_with_error(&result, &params));
    }

    // log_analysis_start Tests

    #[test]
    fn test_log_analysis_start_non_json() {
        // This test verifies the function doesn't panic for non-JSON formats
        log_analysis_start(&LintHotspotOutputFormat::Summary);
        log_analysis_start(&LintHotspotOutputFormat::Detailed);
        log_analysis_start(&LintHotspotOutputFormat::Sarif);
    }

    #[test]
    fn test_log_analysis_start_json() {
        // This test verifies the function doesn't panic for JSON format
        log_analysis_start(&LintHotspotOutputFormat::Json);
    }

    // log_single_file_mode Tests

    #[test]
    fn test_log_single_file_mode_non_json() {
        let file_path = PathBuf::from("src/main.rs");
        log_single_file_mode(&file_path, &LintHotspotOutputFormat::Summary);
        log_single_file_mode(&file_path, &LintHotspotOutputFormat::Detailed);
    }

    #[test]
    fn test_log_single_file_mode_json() {
        let file_path = PathBuf::from("src/main.rs");
        log_single_file_mode(&file_path, &LintHotspotOutputFormat::Json);
    }

    // generate_enforcement_metadata_if_needed Tests

    #[test]
    fn test_generate_enforcement_metadata_when_requested() {
        let hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test"),
            file: None,
            format: LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.7,
            enforce: false,
            dry_run: false,
            enforcement_metadata: true, // Explicitly requested
            output: None,
            perf: false,
            clippy_flags: String::new(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let metadata = generate_enforcement_metadata_if_needed(&hotspot, &params);
        assert!(metadata.is_some());
    }

    #[test]
    fn test_generate_enforcement_metadata_when_enforce() {
        let hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test"),
            file: None,
            format: LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.7,
            enforce: true, // Enforce flag set
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: String::new(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let metadata = generate_enforcement_metadata_if_needed(&hotspot, &params);
        assert!(metadata.is_some());
    }

    #[test]
    fn test_generate_enforcement_metadata_not_requested() {
        let hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test"),
            file: None,
            format: LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.7,
            enforce: false,
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: String::new(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let metadata = generate_enforcement_metadata_if_needed(&hotspot, &params);
        assert!(metadata.is_none());
    }

    // generate_refactor_chain_if_needed Tests

    #[test]
    fn test_generate_refactor_chain_when_enforce() {
        let hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test"),
            file: None,
            format: LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.7,
            enforce: true,
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: String::new(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let chain = generate_refactor_chain_if_needed(&hotspot, &params, &None);
        assert!(chain.is_some());
    }

    #[test]
    fn test_generate_refactor_chain_when_enforcement_required() {
        let hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test"),
            file: None,
            format: LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.7,
            enforce: false,
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: String::new(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let enforcement = Some(EnforcementMetadata {
            enforcement_score: 8.0,
            requires_enforcement: true,
            estimated_fix_time: 1800,
            automation_confidence: 0.85,
            enforcement_priority: 3,
        });

        let chain = generate_refactor_chain_if_needed(&hotspot, &params, &enforcement);
        assert!(chain.is_some());
    }

    #[test]
    fn test_generate_refactor_chain_not_needed() {
        let hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test"),
            file: None,
            format: LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.7,
            enforce: false,
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: String::new(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let chain = generate_refactor_chain_if_needed(&hotspot, &params, &None);
        assert!(chain.is_none());
    }

    // Property-based Tests

    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_defect_density_never_negative(violations in 0usize..1000, sloc in 1usize..10000) {
                let density = calculate_defect_density(violations, sloc);
                prop_assert!(density >= 0.0);
            }

            #[test]
            fn test_total_violations_equals_sum(
                errors in 0usize..100,
                warnings in 0usize..100,
                suggestions in 0usize..100
            ) {
                let metrics = FileMetrics {
                    violations: HashMap::new(),
                    severity_counts: SeverityDistribution {
                        error: errors,
                        warning: warnings,
                        suggestion: suggestions,
                        note: 0,
                    },
                    sloc: 100,
                    detailed_violations: vec![],
                };

                let total = calculate_total_violations(&metrics);
                prop_assert_eq!(total, errors + warnings + suggestions);
            }

            #[test]
            // #701: this used to assert `sloc >= 0` on a `usize`, which is
            // true for every possible value and so measured nothing (clippy
            // flags it as a useless comparison). It now bounds sloc by the
            // only thing it can be: the number of lines it was counted from.
            fn test_count_sloc_bounded_by_line_count(content in ".*") {
                let sloc = count_sloc(&content);
                prop_assert!(sloc <= content.lines().count());
            }

            #[test]
            fn test_enforcement_score_bounded(violations in 1usize..100, sloc in 1usize..1000) {
                let mut hotspot = create_test_hotspot("test.rs", violations, sloc, violations / 2, violations - violations / 2);
                hotspot.defect_density = violations as f64 / sloc as f64;

                let metadata = calculate_enforcement_metadata(&hotspot, 0.7);
                prop_assert!(metadata.enforcement_score >= 0.0);
                prop_assert!(metadata.enforcement_score <= 10.0);
            }

            #[test]
            fn test_quality_gate_consistency(density in 0.0f64..10.0, threshold in 0.1f64..10.0) {
                let violations = (density * 100.0) as usize;
                let hotspot = create_test_hotspot("test.rs", violations, 100, violations / 2, violations - violations / 2);

                let status = check_quality_gates(&hotspot, threshold);

                // If density exceeds threshold, gate should fail
                if hotspot.defect_density > threshold {
                    prop_assert!(!status.passed || status.violations.iter().any(|v| v.rule == "max_defect_density"));
                }
            }
        }
    }
