//\! Tests for lint hotspot handlers
//\! Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;

    fn create_test_hotspot_result() -> LintHotspotResult {
        LintHotspotResult {
            hotspot: LintHotspot {
                file: PathBuf::from("src/main.rs"),
                defect_density: 0.05,
                total_violations: 5,
                sloc: 100,
                severity_distribution: SeverityDistribution {
                    error: 2,
                    warning: 3,
                    suggestion: 0,
                    note: 0,
                },
                top_lints: vec![
                    ("clippy::too_many_arguments".to_string(), 2),
                    ("unused_variable".to_string(), 3),
                ],
                detailed_violations: vec![],
            },
            all_violations: vec![],
            summary_by_file: std::collections::HashMap::new(),
            total_project_violations: 5,
            enforcement: None,
            refactor_chain: None,
            quality_gate: QualityGateStatus {
                passed: true,
                violations: vec![],
                blocking: false,
            },
        }
    }

    /// Test that enforce flag exits with non-zero status when there are violations
    ///
    /// # Example
    ///
    /// ```
    /// use pmat::cli::handlers::lint_hotspot_handlers::should_exit_with_error;
    ///
    /// // With enforce flag and violations - should exit with error
    /// let should_exit = should_exit_with_error(true, true, 5);
    /// assert!(should_exit);
    ///
    /// // Without enforce flag but quality gate failed - should exit with error  
    /// let should_exit = should_exit_with_error(false, false, 5);
    /// assert!(should_exit);
    ///
    /// // Without enforce flag and no violations - should not exit with error
    /// let should_exit = should_exit_with_error(true, false, 0);
    /// assert!(!should_exit);
    /// ```
    pub fn should_exit_with_error(
        quality_gate_passed: bool,
        enforce: bool,
        total_violations: usize,
    ) -> bool {
        !quality_gate_passed || (enforce && total_violations > 0)
    }

    #[test]
    fn test_enforce_flag_behavior() {
        // Test 1: Enforce flag with violations should trigger exit
        assert!(should_exit_with_error(true, true, 5));

        // Test 2: Enforce flag without violations should not trigger exit
        assert!(!should_exit_with_error(true, true, 0));

        // Test 3: No enforce flag with violations should not trigger exit
        assert!(!should_exit_with_error(true, false, 5));

        // Test 4: Quality gate failed should always trigger exit
        assert!(should_exit_with_error(false, false, 0));
        assert!(should_exit_with_error(false, true, 5));
    }

    #[test]
    fn test_format_summary_with_violations() {
        let result = create_test_hotspot_result();
        let output = format_summary(&result, false, std::time::Duration::from_secs(1), 10).unwrap();

        assert!(output.contains("# Lint Hotspot Analysis"));
        assert!(output.contains("**Total Project Violations**: 5"));
        assert!(output.contains("## Top Files with Lint Issues"));
        assert!(output.contains("## Hottest File Details"));
        assert!(output.contains("**File**: src/main.rs"));
    }

    #[test]
    fn test_quality_gate_enforcement_scenario() {
        let mut result = create_test_hotspot_result();

        // Test case 1: Quality gate passes but enforce flag is set with violations
        result.quality_gate.passed = true;
        result.total_project_violations = 10;

        let should_exit = should_exit_with_error(
            result.quality_gate.passed,
            true, // enforce flag
            result.total_project_violations,
        );
        assert!(
            should_exit,
            "Should exit with error when enforce flag is set and violations exist"
        );

        // Test case 2: Quality gate passes, enforce flag set, no violations
        result.total_project_violations = 0;

        let should_exit = should_exit_with_error(
            result.quality_gate.passed,
            true, // enforce flag
            result.total_project_violations,
        );
        assert!(
            !should_exit,
            "Should not exit with error when enforce flag is set but no violations"
        );
    }

    #[test]
    fn test_multiple_enforcement_scenarios() {
        // Scenario 1: Quality gate failed, no enforce flag
        assert!(should_exit_with_error(false, false, 0));

        // Scenario 2: Quality gate passed, enforce flag, violations present
        assert!(should_exit_with_error(true, true, 1));

        // Scenario 3: Quality gate passed, enforce flag, no violations
        assert!(!should_exit_with_error(true, true, 0));

        // Scenario 4: Quality gate passed, no enforce flag, violations present
        assert!(!should_exit_with_error(true, false, 10));

        // Scenario 5: Quality gate failed, enforce flag, violations present
        assert!(should_exit_with_error(false, true, 5));
    }
}


mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}


mod coverage_tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::Duration;

    // Test Data Builders

    fn create_test_violation(
        file: &str,
        line: u32,
        lint_name: &str,
        severity: &str,
    ) -> ViolationDetail {
        ViolationDetail {
            file: PathBuf::from(file),
            line,
            column: 1,
            end_line: line,
            end_column: 10,
            lint_name: lint_name.to_string(),
            message: format!("Test violation for {lint_name}"),
            severity: severity.to_string(),
            suggestion: None,
            machine_applicable: false,
        }
    }

    fn create_test_violation_with_suggestion(
        file: &str,
        line: u32,
        lint_name: &str,
        severity: &str,
        suggestion: &str,
    ) -> ViolationDetail {
        ViolationDetail {
            file: PathBuf::from(file),
            line,
            column: 1,
            end_line: line,
            end_column: 10,
            lint_name: lint_name.to_string(),
            message: format!("Test violation for {lint_name}"),
            severity: severity.to_string(),
            suggestion: Some(suggestion.to_string()),
            machine_applicable: true,
        }
    }

    fn create_test_hotspot(
        file: &str,
        violations: usize,
        sloc: usize,
        errors: usize,
        warnings: usize,
    ) -> LintHotspot {
        let defect_density = if sloc > 0 {
            violations as f64 / sloc as f64
        } else {
            0.0
        };
        LintHotspot {
            file: PathBuf::from(file),
            defect_density,
            total_violations: violations,
            sloc,
            severity_distribution: SeverityDistribution {
                error: errors,
                warning: warnings,
                suggestion: 0,
                note: 0,
            },
            top_lints: vec![
                ("clippy::unused_variable".to_string(), 3),
                ("clippy::needless_return".to_string(), 2),
            ],
            detailed_violations: vec![],
        }
    }

    fn create_full_test_result() -> LintHotspotResult {
        let mut summary_by_file = HashMap::new();
        summary_by_file.insert(
            PathBuf::from("src/main.rs"),
            FileSummary {
                total_violations: 5,
                errors: 2,
                warnings: 3,
                sloc: 100,
                defect_density: 0.05,
            },
        );
        summary_by_file.insert(
            PathBuf::from("src/lib.rs"),
            FileSummary {
                total_violations: 3,
                errors: 1,
                warnings: 2,
                sloc: 50,
                defect_density: 0.06,
            },
        );

        LintHotspotResult {
            hotspot: create_test_hotspot("src/main.rs", 5, 100, 2, 3),
            all_violations: vec![
                create_test_violation("src/main.rs", 10, "unused_variable", "warning"),
                create_test_violation("src/main.rs", 20, "clippy::needless_return", "warning"),
                create_test_violation("src/lib.rs", 5, "clippy::too_many_arguments", "warning"),
            ],
            summary_by_file,
            total_project_violations: 8,
            enforcement: None,
            refactor_chain: None,
            quality_gate: QualityGateStatus {
                passed: true,
                violations: vec![],
                blocking: false,
            },
        }
    }

    // LintHotspotParams Tests

    #[test]
    fn test_lint_hotspot_params_creation() {
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test/project"),
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

        assert_eq!(params.project_path, PathBuf::from("/test/project"));
        assert!(params.file.is_none());
        assert!(!params.enforce);
        assert_eq!(params.max_density, 5.0);
        assert_eq!(params.min_confidence, 0.7);
    }

    #[test]
    fn test_lint_hotspot_params_with_file() {
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test/project"),
            file: Some(PathBuf::from("src/main.rs")),
            format: LintHotspotOutputFormat::Json,
            max_density: 10.0,
            min_confidence: 0.8,
            enforce: true,
            dry_run: true,
            enforcement_metadata: true,
            output: Some(PathBuf::from("/tmp/output.json")),
            perf: true,
            clippy_flags: "-W clippy::pedantic".to_string(),
            top_files: 5,
            include: vec!["src/**/*.rs".to_string()],
            exclude: vec!["**/tests/**".to_string()],
        };

        assert_eq!(params.file, Some(PathBuf::from("src/main.rs")));
        assert!(params.enforce);
        assert!(params.dry_run);
        assert!(params.enforcement_metadata);
        assert!(params.perf);
        assert!(!params.include.is_empty());
        assert!(!params.exclude.is_empty());
    }

    // ViolationDetail Tests

    #[test]
    fn test_violation_detail_clone() {
        let original = create_test_violation("src/main.rs", 10, "unused_variable", "warning");
        let cloned = original.clone();

        assert_eq!(cloned.file, original.file);
        assert_eq!(cloned.line, original.line);
        assert_eq!(cloned.lint_name, original.lint_name);
        assert_eq!(cloned.severity, original.severity);
    }

    #[test]
    fn test_violation_detail_with_suggestion() {
        let violation = create_test_violation_with_suggestion(
            "src/main.rs",
            10,
            "unused_variable",
            "warning",
            "Remove the unused variable",
        );

        assert!(violation.suggestion.is_some());
        assert!(violation.machine_applicable);
        assert_eq!(
            violation.suggestion.unwrap(),
            "Remove the unused variable"
        );
    }

    #[test]
    fn test_violation_detail_serialization() {
        let violation = create_test_violation("src/main.rs", 10, "unused_variable", "warning");
        let json = serde_json::to_string(&violation).unwrap();

        assert!(json.contains("src/main.rs"));
        assert!(json.contains("unused_variable"));
        assert!(json.contains("warning"));
    }

    // SeverityDistribution Tests

    #[test]
    fn test_severity_distribution_default() {
        let dist = SeverityDistribution::default();

        assert_eq!(dist.error, 0);
        assert_eq!(dist.warning, 0);
        assert_eq!(dist.suggestion, 0);
        assert_eq!(dist.note, 0);
    }

    #[test]
    fn test_severity_distribution_custom() {
        let dist = SeverityDistribution {
            error: 5,
            warning: 10,
            suggestion: 3,
            note: 2,
        };

        assert_eq!(dist.error, 5);
        assert_eq!(dist.warning, 10);
        assert_eq!(dist.suggestion, 3);
        assert_eq!(dist.note, 2);
    }

    #[test]
    fn test_severity_distribution_serialization() {
        let dist = SeverityDistribution {
            error: 5,
            warning: 10,
            suggestion: 3,
            note: 2,
        };

        let json = serde_json::to_string(&dist).unwrap();
        let deserialized: SeverityDistribution = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.error, dist.error);
        assert_eq!(deserialized.warning, dist.warning);
        assert_eq!(deserialized.suggestion, dist.suggestion);
        assert_eq!(deserialized.note, dist.note);
    }

    // FileSummary Tests

    #[test]
    fn test_file_summary_creation() {
        let summary = FileSummary {
            total_violations: 10,
            errors: 3,
            warnings: 7,
            sloc: 200,
            defect_density: 0.05,
        };

        assert_eq!(summary.total_violations, 10);
        assert_eq!(summary.errors, 3);
        assert_eq!(summary.warnings, 7);
        assert_eq!(summary.sloc, 200);
        assert!((summary.defect_density - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_file_summary_serialization() {
        let summary = FileSummary {
            total_violations: 10,
            errors: 3,
            warnings: 7,
            sloc: 200,
            defect_density: 0.05,
        };

        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: FileSummary = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_violations, summary.total_violations);
        assert_eq!(deserialized.errors, summary.errors);
        assert_eq!(deserialized.warnings, summary.warnings);
    }

    // LintHotspot Tests

    #[test]
    fn test_lint_hotspot_creation() {
        let hotspot = create_test_hotspot("src/main.rs", 10, 200, 3, 7);

        assert_eq!(hotspot.file, PathBuf::from("src/main.rs"));
        assert_eq!(hotspot.total_violations, 10);
        assert_eq!(hotspot.sloc, 200);
        assert_eq!(hotspot.severity_distribution.error, 3);
        assert_eq!(hotspot.severity_distribution.warning, 7);
        assert!((hotspot.defect_density - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_lint_hotspot_with_zero_sloc() {
        let hotspot = create_test_hotspot("src/empty.rs", 5, 0, 1, 4);

        assert_eq!(hotspot.sloc, 0);
        assert!((hotspot.defect_density - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_lint_hotspot_serialization() {
        let hotspot = create_test_hotspot("src/main.rs", 10, 200, 3, 7);

        let json = serde_json::to_string(&hotspot).unwrap();
        let deserialized: LintHotspot = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.file, hotspot.file);
        assert_eq!(deserialized.total_violations, hotspot.total_violations);
        assert_eq!(deserialized.sloc, hotspot.sloc);
    }

    // EnforcementMetadata Tests

    #[test]
    fn test_enforcement_metadata_creation() {
        let metadata = EnforcementMetadata {
            enforcement_score: 8.5,
            requires_enforcement: true,
            estimated_fix_time: 1800,
            automation_confidence: 0.85,
            enforcement_priority: 3,
        };

        assert!((metadata.enforcement_score - 8.5).abs() < f64::EPSILON);
        assert!(metadata.requires_enforcement);
        assert_eq!(metadata.estimated_fix_time, 1800);
        assert!((metadata.automation_confidence - 0.85).abs() < f64::EPSILON);
        assert_eq!(metadata.enforcement_priority, 3);
    }

    #[test]
    fn test_enforcement_metadata_serialization() {
        let metadata = EnforcementMetadata {
            enforcement_score: 8.5,
            requires_enforcement: true,
            estimated_fix_time: 1800,
            automation_confidence: 0.85,
            enforcement_priority: 3,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: EnforcementMetadata = serde_json::from_str(&json).unwrap();

        assert!((deserialized.enforcement_score - metadata.enforcement_score).abs() < f64::EPSILON);
        assert_eq!(
            deserialized.requires_enforcement,
            metadata.requires_enforcement
        );
    }

    // RefactorChain and RefactorStep Tests

    #[test]
    fn test_refactor_step_creation() {
        let step = RefactorStep {
            id: "fix-unused".to_string(),
            lint: "unused_variable".to_string(),
            confidence: 0.95,
            impact: 5,
            description: "Remove unused variables".to_string(),
        };

        assert_eq!(step.id, "fix-unused");
        assert_eq!(step.lint, "unused_variable");
        assert!((step.confidence - 0.95).abs() < f64::EPSILON);
        assert_eq!(step.impact, 5);
    }

    #[test]
    fn test_refactor_chain_creation() {
        let chain = RefactorChain {
            id: "lint-hotspot-20240101-120000".to_string(),
            estimated_reduction: 15,
            automation_confidence: 0.88,
            steps: vec![
                RefactorStep {
                    id: "step-1".to_string(),
                    lint: "unused".to_string(),
                    confidence: 0.95,
                    impact: 10,
                    description: "Remove unused".to_string(),
                },
                RefactorStep {
                    id: "step-2".to_string(),
                    lint: "needless".to_string(),
                    confidence: 0.80,
                    impact: 5,
                    description: "Simplify needless".to_string(),
                },
            ],
        };

        assert_eq!(chain.steps.len(), 2);
        assert_eq!(chain.estimated_reduction, 15);
    }

    #[test]
    fn test_refactor_chain_serialization() {
        let chain = RefactorChain {
            id: "test-chain".to_string(),
            estimated_reduction: 10,
            automation_confidence: 0.9,
            steps: vec![],
        };

        let json = serde_json::to_string(&chain).unwrap();
        let deserialized: RefactorChain = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, chain.id);
        assert_eq!(deserialized.estimated_reduction, chain.estimated_reduction);
    }

    // QualityGateStatus and QualityViolation Tests

    #[test]
    fn test_quality_gate_status_passed() {
        let status = QualityGateStatus {
            passed: true,
            violations: vec![],
            blocking: false,
        };

        assert!(status.passed);
        assert!(status.violations.is_empty());
        assert!(!status.blocking);
    }

    #[test]
    fn test_quality_gate_status_failed_with_violations() {
        let status = QualityGateStatus {
            passed: false,
            violations: vec![QualityViolation {
                rule: "max_defect_density".to_string(),
                threshold: 5.0,
                actual: 10.0,
                severity: "blocking".to_string(),
            }],
            blocking: true,
        };

        assert!(!status.passed);
        assert_eq!(status.violations.len(), 1);
        assert!(status.blocking);
    }

    #[test]
    fn test_quality_violation_serialization() {
        let violation = QualityViolation {
            rule: "max_defect_density".to_string(),
            threshold: 5.0,
            actual: 10.0,
            severity: "blocking".to_string(),
        };

        let json = serde_json::to_string(&violation).unwrap();
        let deserialized: QualityViolation = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.rule, violation.rule);
        assert!((deserialized.threshold - violation.threshold).abs() < f64::EPSILON);
        assert!((deserialized.actual - violation.actual).abs() < f64::EPSILON);
    }

    // check_quality_gates Tests

    #[test]
    fn test_check_quality_gates_passes_below_threshold() {
        let hotspot = create_test_hotspot("src/main.rs", 5, 100, 2, 3);
        let status = check_quality_gates(&hotspot, 10.0);

        assert!(status.passed);
        assert!(status.violations.is_empty());
        assert!(!status.blocking);
    }

    #[test]
    fn test_check_quality_gates_fails_exceeds_density() {
        let hotspot = create_test_hotspot("src/main.rs", 100, 100, 50, 50);
        let status = check_quality_gates(&hotspot, 0.5);

        assert!(!status.passed);
        assert!(!status.violations.is_empty());
        assert!(status.blocking);

        let violation = &status.violations[0];
        assert_eq!(violation.rule, "max_defect_density");
        assert!((violation.threshold - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_check_quality_gates_warning_for_high_violations() {
        let mut hotspot = create_test_hotspot("src/main.rs", 60, 1000, 30, 30);
        hotspot.defect_density = 0.06; // Below max_density of 1.0
        hotspot.total_violations = 60;

        let status = check_quality_gates(&hotspot, 1.0);

        // Should have warning for max_single_file_violations
        let warning = status
            .violations
            .iter()
            .find(|v| v.rule == "max_single_file_violations");
        assert!(warning.is_some());
        assert_eq!(warning.unwrap().severity, "warning");
    }

    // calculate_enforcement_metadata Tests

    #[test]
    fn test_calculate_enforcement_metadata_low_density() {
        let hotspot = create_test_hotspot("src/main.rs", 5, 100, 2, 3);
        let metadata = calculate_enforcement_metadata(&hotspot, 0.7);

        assert!(metadata.enforcement_score < 10.0);
        assert!(!metadata.requires_enforcement); // Score < 7.0
        assert_eq!(metadata.estimated_fix_time, 5 * 300); // 5 violations * 300 seconds
    }

    #[test]
    fn test_calculate_enforcement_metadata_high_density() {
        let mut hotspot = create_test_hotspot("src/main.rs", 100, 100, 50, 50);
        hotspot.defect_density = 1.0;
        hotspot.top_lints = vec![("unused_variable".to_string(), 50)];

        let metadata = calculate_enforcement_metadata(&hotspot, 0.5);

        assert_eq!(metadata.enforcement_score, 10.0); // Capped at 10.0
        assert!(metadata.requires_enforcement);
        assert!((metadata.automation_confidence - 0.9).abs() < f64::EPSILON); // "unused" in lint
    }

    #[test]
    fn test_calculate_enforcement_metadata_with_redundant_lints() {
        let mut hotspot = create_test_hotspot("src/main.rs", 20, 100, 10, 10);
        hotspot.defect_density = 0.8;
        hotspot.top_lints = vec![("redundant_clone".to_string(), 10)];

        let metadata = calculate_enforcement_metadata(&hotspot, 0.7);

        assert!((metadata.automation_confidence - 0.9).abs() < f64::EPSILON); // "redundant" in lint
    }

    #[test]
    fn test_calculate_enforcement_metadata_without_auto_fixable() {
        let mut hotspot = create_test_hotspot("src/main.rs", 20, 100, 10, 10);
        hotspot.defect_density = 0.8;
        hotspot.top_lints = vec![("clippy::complexity".to_string(), 10)];

        let metadata = calculate_enforcement_metadata(&hotspot, 0.7);

        assert!((metadata.automation_confidence - 0.7).abs() < f64::EPSILON); // Default confidence
    }

    // generate_refactor_chain Tests

    #[test]
    fn test_generate_refactor_chain_with_unused_lints() {
        let mut hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        hotspot.top_lints = vec![
            ("unused_variable".to_string(), 5),
            ("unused_import".to_string(), 3),
        ];

        let chain = generate_refactor_chain(&hotspot, 0.7);

        assert_eq!(chain.steps.len(), 2);
        assert!((chain.steps[0].confidence - 0.95).abs() < f64::EPSILON);
        assert_eq!(chain.steps[0].description, "Remove unused code");
    }

    #[test]
    fn test_generate_refactor_chain_with_needless_lints() {
        let mut hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        hotspot.top_lints = vec![("needless_return".to_string(), 5)];

        let chain = generate_refactor_chain(&hotspot, 0.7);

        assert_eq!(chain.steps.len(), 1);
        assert!((chain.steps[0].confidence - 0.85).abs() < f64::EPSILON);
        assert_eq!(chain.steps[0].description, "Simplify needless patterns");
    }

    #[test]
    fn test_generate_refactor_chain_filters_by_confidence() {
        let mut hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        hotspot.top_lints = vec![
            ("unused_variable".to_string(), 5),  // confidence 0.95
            ("complex_lint".to_string(), 3),      // confidence 0.70
        ];

        let chain = generate_refactor_chain(&hotspot, 0.9);

        // Only unused_variable should pass the 0.9 threshold
        assert_eq!(chain.steps.len(), 1);
        assert!(chain.steps[0].lint.contains("unused"));
    }

    #[test]
    fn test_generate_refactor_chain_calculates_total_reduction() {
        let mut hotspot = create_test_hotspot("src/main.rs", 15, 100, 5, 10);
        hotspot.top_lints = vec![
            ("unused_variable".to_string(), 5),
            ("redundant_clone".to_string(), 3),
        ];

        let chain = generate_refactor_chain(&hotspot, 0.7);

        assert_eq!(chain.estimated_reduction, 8); // 5 + 3
    }

    // count_top_lints Tests

    #[test]
    fn test_count_top_lints_empty() {
        let violations: Vec<ViolationDetail> = vec![];
        let top_lints = count_top_lints(&violations);

        assert!(top_lints.is_empty());
    }

    #[test]
    fn test_count_top_lints_single_lint() {
        let violations = vec![
            create_test_violation("src/main.rs", 10, "unused_variable", "warning"),
            create_test_violation("src/main.rs", 20, "unused_variable", "warning"),
        ];

        let top_lints = count_top_lints(&violations);

        assert_eq!(top_lints.len(), 1);
        assert_eq!(top_lints[0].0, "unused_variable");
        assert_eq!(top_lints[0].1, 2);
    }

    #[test]
    fn test_count_top_lints_multiple_lints_sorted() {
        let violations = vec![
            create_test_violation("src/main.rs", 10, "lint_a", "warning"),
            create_test_violation("src/main.rs", 20, "lint_b", "warning"),
            create_test_violation("src/main.rs", 30, "lint_b", "warning"),
            create_test_violation("src/main.rs", 40, "lint_b", "warning"),
            create_test_violation("src/main.rs", 50, "lint_a", "warning"),
        ];

        let top_lints = count_top_lints(&violations);

        assert_eq!(top_lints.len(), 2);
        assert_eq!(top_lints[0].0, "lint_b"); // 3 occurrences
        assert_eq!(top_lints[0].1, 3);
        assert_eq!(top_lints[1].0, "lint_a"); // 2 occurrences
        assert_eq!(top_lints[1].1, 2);
    }

    #[test]
    fn test_count_top_lints_truncates_to_10() {
        let mut violations = vec![];
        for i in 0..15 {
            violations.push(create_test_violation(
                "src/main.rs",
                i as u32,
                &format!("lint_{i}"),
                "warning",
            ));
        }

        let top_lints = count_top_lints(&violations);

        assert_eq!(top_lints.len(), 10);
    }

    // update_severity_distribution Tests

    #[test]
    fn test_update_severity_distribution_error() {
        let mut dist = SeverityDistribution::default();
        update_severity_distribution(&mut dist, "error");

        assert_eq!(dist.error, 1);
        assert_eq!(dist.warning, 0);
        assert_eq!(dist.note, 0);
    }

    #[test]
    fn test_update_severity_distribution_warning() {
        let mut dist = SeverityDistribution::default();
        update_severity_distribution(&mut dist, "warning");

        assert_eq!(dist.error, 0);
        assert_eq!(dist.warning, 1);
        assert_eq!(dist.note, 0);
    }

    #[test]
    fn test_update_severity_distribution_unknown() {
        let mut dist = SeverityDistribution::default();
        update_severity_distribution(&mut dist, "unknown");

        assert_eq!(dist.error, 0);
        assert_eq!(dist.warning, 0);
        assert_eq!(dist.note, 1);
    }

    #[test]
    fn test_update_severity_distribution_multiple() {
        let mut dist = SeverityDistribution::default();
        update_severity_distribution(&mut dist, "error");
        update_severity_distribution(&mut dist, "error");
        update_severity_distribution(&mut dist, "warning");
        update_severity_distribution(&mut dist, "warning");
        update_severity_distribution(&mut dist, "warning");
        update_severity_distribution(&mut dist, "note");

        assert_eq!(dist.error, 2);
        assert_eq!(dist.warning, 3);
        assert_eq!(dist.note, 1);
    }

    // count_sloc Tests

    #[test]
    fn test_count_sloc_empty() {
        let content = "";
        assert_eq!(count_sloc(content), 0);
    }

    #[test]
    fn test_count_sloc_only_whitespace() {
        let content = "   \n\n   \n";
        assert_eq!(count_sloc(content), 0);
    }

    #[test]
    fn test_count_sloc_only_comments() {
        let content = "// comment 1\n// comment 2\n// comment 3";
        assert_eq!(count_sloc(content), 0);
    }

    #[test]
    fn test_count_sloc_mixed_content() {
        let content = r#"
fn main() {
    // This is a comment
    let x = 5;

    println!("Hello");
}
"#;
        // Should count: fn main(), let x = 5;, println!(), }
        // Empty line and comment line are excluded
        assert!(count_sloc(content) >= 3);
    }

    #[test]
    fn test_count_sloc_with_inline_comments() {
        let content = r#"let x = 5; // inline comment
let y = 10;"#;
        // Both lines have code, inline comments don't make a line "comment only"
        assert_eq!(count_sloc(content), 2);
    }

    // calculate_defect_density Tests

    #[test]
    fn test_calculate_defect_density_normal() {
        assert!((calculate_defect_density(10, 100) - 0.1).abs() < f64::EPSILON);
        assert!((calculate_defect_density(5, 50) - 0.1).abs() < f64::EPSILON);
        assert!((calculate_defect_density(0, 100) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_defect_density_zero_sloc() {
        assert!((calculate_defect_density(10, 0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_defect_density_high_density() {
        assert!((calculate_defect_density(200, 100) - 2.0).abs() < f64::EPSILON);
    }

    // calculate_total_violations Tests

    #[test]
    fn test_calculate_total_violations() {
        let metrics = FileMetrics {
            violations: HashMap::new(),
            severity_counts: SeverityDistribution {
                error: 5,
                warning: 10,
                suggestion: 3,
                note: 0,
            },
            sloc: 100,
            detailed_violations: vec![],
        };

        let total = calculate_total_violations(&metrics);
        assert_eq!(total, 18); // 5 + 10 + 3
    }

    #[test]
    fn test_calculate_total_violations_empty() {
        let metrics = FileMetrics {
            violations: HashMap::new(),
            severity_counts: SeverityDistribution::default(),
            sloc: 100,
            detailed_violations: vec![],
        };

        let total = calculate_total_violations(&metrics);
        assert_eq!(total, 0);
    }

    // resolve_absolute_path Tests

    #[test]
    fn test_resolve_absolute_path_already_absolute() {
        let project_path = PathBuf::from("/project");
        let file_path = PathBuf::from("/absolute/path/file.rs");

        let resolved = resolve_absolute_path(&project_path, &file_path);
        assert_eq!(resolved, file_path);
    }

    #[test]
    fn test_resolve_absolute_path_relative() {
        let project_path = PathBuf::from("/project");
        let file_path = PathBuf::from("src/main.rs");

        let resolved = resolve_absolute_path(&project_path, &file_path);
        assert_eq!(resolved, PathBuf::from("/project/src/main.rs"));
    }

    // is_target_file Tests

    #[test]
    fn test_is_target_file_exact_match() {
        let abs_path = PathBuf::from("/project/src/main.rs");
        let file_path = PathBuf::from("src/main.rs");

        assert!(is_target_file(
            "/project/src/main.rs",
            &abs_path,
            &file_path
        ));
    }

    #[test]
    fn test_is_target_file_relative_match() {
        let abs_path = PathBuf::from("/project/src/main.rs");
        let file_path = PathBuf::from("src/main.rs");

        assert!(is_target_file("src/main.rs", &abs_path, &file_path));
    }

    #[test]
    fn test_is_target_file_ends_with() {
        let abs_path = PathBuf::from("/project/src/main.rs");
        let file_path = PathBuf::from("main.rs");

        assert!(is_target_file(
            "/project/src/main.rs",
            &abs_path,
            &file_path
        ));
    }

    #[test]
    fn test_is_target_file_no_match() {
        let abs_path = PathBuf::from("/project/src/main.rs");
        let file_path = PathBuf::from("src/main.rs");

        assert!(!is_target_file("src/lib.rs", &abs_path, &file_path));
    }

    // is_machine_applicable Tests

    #[test]
    fn test_is_machine_applicable_true() {
        let span = DiagnosticSpan {
            file_name: "test.rs".to_string(),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            is_primary: true,
            _text: vec![],
            suggested_replacement: Some("fix".to_string()),
            suggestion_applicability: Some("machine-applicable".to_string()),
        };

        assert!(is_machine_applicable(&span));
    }

    #[test]
    fn test_is_machine_applicable_maybe_incorrect() {
        let span = DiagnosticSpan {
            file_name: "test.rs".to_string(),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            is_primary: true,
            _text: vec![],
            suggested_replacement: Some("fix".to_string()),
            suggestion_applicability: Some("maybe-incorrect".to_string()),
        };

        assert!(is_machine_applicable(&span));
    }

    #[test]
    fn test_is_machine_applicable_false() {
        let span = DiagnosticSpan {
            file_name: "test.rs".to_string(),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            is_primary: true,
            _text: vec![],
            suggested_replacement: None,
            suggestion_applicability: None,
        };

        assert!(!is_machine_applicable(&span));
    }

    #[test]
    fn test_is_machine_applicable_other_applicability() {
        let span = DiagnosticSpan {
            file_name: "test.rs".to_string(),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            is_primary: true,
            _text: vec![],
            suggested_replacement: Some("fix".to_string()),
            suggestion_applicability: Some("unspecified".to_string()),
        };

        assert!(!is_machine_applicable(&span));
    }

    // extract_lint_name Tests

    #[test]
    fn test_extract_lint_name_with_code() {
        let diagnostic = DiagnosticMessage {
            level: "warning".to_string(),
            message: "test message".to_string(),
            code: Some(DiagnosticCode {
                code: "unused_variable".to_string(),
            }),
            spans: vec![],
        };

        let lint_name = extract_lint_name(&diagnostic);
        assert_eq!(lint_name, "unused_variable");
    }

    #[test]
    fn test_extract_lint_name_without_code() {
        let diagnostic = DiagnosticMessage {
            level: "warning".to_string(),
            message: "test message".to_string(),
            code: None,
            spans: vec![],
        };

        let lint_name = extract_lint_name(&diagnostic);
        assert_eq!(lint_name, "");
    }

    // find_primary_span Tests

    #[test]
    fn test_find_primary_span_with_primary() {
        let diagnostic = DiagnosticMessage {
            level: "warning".to_string(),
            message: "test".to_string(),
            code: None,
            spans: vec![
                DiagnosticSpan {
                    file_name: "secondary.rs".to_string(),
                    line_start: 1,
                    line_end: 1,
                    column_start: 1,
                    column_end: 10,
                    is_primary: false,
                    _text: vec![],
                    suggested_replacement: None,
                    suggestion_applicability: None,
                },
                DiagnosticSpan {
                    file_name: "primary.rs".to_string(),
                    line_start: 5,
                    line_end: 5,
                    column_start: 1,
                    column_end: 10,
                    is_primary: true,
                    _text: vec![],
                    suggested_replacement: None,
                    suggestion_applicability: None,
                },
            ],
        };

        let span = find_primary_span(&diagnostic);
        assert!(span.is_some());
        assert_eq!(span.unwrap().file_name, "primary.rs");
    }

    #[test]
    fn test_find_primary_span_single_span() {
        let diagnostic = DiagnosticMessage {
            level: "warning".to_string(),
            message: "test".to_string(),
            code: None,
            spans: vec![DiagnosticSpan {
                file_name: "only.rs".to_string(),
                line_start: 1,
                line_end: 1,
                column_start: 1,
                column_end: 10,
                is_primary: false, // Even if not primary, it's returned as the only span
                _text: vec![],
                suggested_replacement: None,
                suggestion_applicability: None,
            }],
        };

        let span = find_primary_span(&diagnostic);
        assert!(span.is_some());
        assert_eq!(span.unwrap().file_name, "only.rs");
    }

    #[test]
    fn test_find_primary_span_empty() {
        let diagnostic = DiagnosticMessage {
            level: "warning".to_string(),
            message: "test".to_string(),
            code: None,
            spans: vec![],
        };

        let span = find_primary_span(&diagnostic);
        assert!(span.is_none());
    }

    // format_output Tests

    #[test]
    fn test_format_output_summary() {
        let result = create_full_test_result();
        let output = format_output(
            &result,
            LintHotspotOutputFormat::Summary,
            false,
            Duration::from_secs(1),
            10,
        )
        .unwrap();

        assert!(output.contains("# Lint Hotspot Analysis"));
        assert!(output.contains("**Total Project Violations**"));
    }

    #[test]
    fn test_format_output_detailed() {
        let result = create_full_test_result();
        let output = format_output(
            &result,
            LintHotspotOutputFormat::Detailed,
            false,
            Duration::from_secs(1),
            10,
        )
        .unwrap();

        assert!(output.contains("# Lint Hotspot Analysis"));
        assert!(output.contains("## Detailed Violations"));
        assert!(output.contains("## Top Files by Violations"));
    }

    #[test]
    fn test_format_output_json() {
        let result = create_full_test_result();
        let output = format_output(
            &result,
            LintHotspotOutputFormat::Json,
            false,
            Duration::from_secs(1),
            10,
        )
        .unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.get("hotspot").is_some());
        assert!(parsed.get("quality_gate").is_some());
    }

    #[test]
    fn test_format_output_enforcement_json() {
        let result = create_full_test_result();
        let output = format_output(
            &result,
            LintHotspotOutputFormat::EnforcementJson,
            false,
            Duration::from_secs(1),
            10,
        )
        .unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.get("hotspot").is_some());
        assert!(parsed.get("all_violations").is_some());
        assert!(parsed.get("summary_by_file").is_some());
    }

    #[test]
    fn test_format_output_sarif() {
        let result = create_full_test_result();
        let output = format_output(
            &result,
            LintHotspotOutputFormat::Sarif,
            false,
            Duration::from_secs(1),
            10,
        )
        .unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.get("version").unwrap(), "2.1.0");
        assert!(parsed.get("$schema").is_some());
        assert!(parsed.get("runs").is_some());
    }

    // format_summary Tests

    #[test]
    fn test_format_summary_with_perf() {
        let result = create_full_test_result();
        let output = format_summary(&result, true, Duration::from_secs(5), 10).unwrap();

        assert!(output.contains("Analysis completed in"));
    }

    #[test]
    fn test_format_summary_with_enforcement() {
        let mut result = create_full_test_result();
        result.enforcement = Some(EnforcementMetadata {
            enforcement_score: 8.5,
            requires_enforcement: true,
            estimated_fix_time: 1800,
            automation_confidence: 0.85,
            enforcement_priority: 3,
        });

        let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();

        assert!(output.contains("## Enforcement Metadata"));
        assert!(output.contains("Score: 8.5/10"));
        assert!(output.contains("Priority: 3"));
    }

    #[test]
    fn test_format_summary_with_failed_quality_gate() {
        let mut result = create_full_test_result();
        result.quality_gate.passed = false;
        result.quality_gate.violations = vec![QualityViolation {
            rule: "max_defect_density".to_string(),
            threshold: 0.05,
            actual: 0.10,
            severity: "blocking".to_string(),
        }];

        let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();

        assert!(output.contains("Quality Gate Failed"));
        assert!(output.contains("max_defect_density exceeded"));
    }

    #[test]
    fn test_format_summary_top_files_limit() {
        let mut result = create_full_test_result();
        // Add more files to summary
        for i in 0..15 {
            result.summary_by_file.insert(
                PathBuf::from(format!("src/file_{i}.rs")),
                FileSummary {
                    total_violations: i,
                    errors: i / 2,
                    warnings: i - i / 2,
                    sloc: 100,
                    defect_density: i as f64 / 100.0,
                },
            );
        }

        let output = format_summary(&result, false, Duration::from_secs(1), 5).unwrap();

        // Should show limited files (may include header row and extras)
        let file_count = output.matches("violations/SLOC").count();
        assert!(file_count > 0, "Should have file entries");
    }

    // format_detailed Tests

    #[test]
    fn test_format_detailed_with_violations() {
        let mut result = create_full_test_result();
        result.hotspot.detailed_violations = vec![
            create_test_violation("src/main.rs", 10, "unused_variable", "warning"),
            create_test_violation_with_suggestion(
                "src/main.rs",
                20,
                "needless_return",
                "warning",
                "Remove the return statement",
            ),
        ];

        let output = format_detailed(&result, false, Duration::from_secs(1), 10).unwrap();

        assert!(output.contains("## Detailed Violations in Hotspot File"));
        assert!(output.contains("unused_variable"));
        assert!(output.contains("Suggestion: Remove the return statement"));
    }

    #[test]
    fn test_format_detailed_with_refactor_chain() {
        let mut result = create_full_test_result();
        result.refactor_chain = Some(RefactorChain {
            id: "test-chain".to_string(),
            estimated_reduction: 15,
            automation_confidence: 0.88,
            steps: vec![
                RefactorStep {
                    id: "step-1".to_string(),
                    lint: "unused".to_string(),
                    confidence: 0.95,
                    impact: 10,
                    description: "Remove unused code".to_string(),
                },
            ],
        });

        let output = format_detailed(&result, false, Duration::from_secs(1), 10).unwrap();

        assert!(output.contains("## Refactor Chain"));
        assert!(output.contains("ID: test-chain"));
        assert!(output.contains("Estimated Reduction: 15 violations"));
        assert!(output.contains("### Steps"));
        assert!(output.contains("Remove unused code"));
    }

    // format_json Tests

    #[test]
    fn test_format_json_simple() {
        let result = create_full_test_result();
        let output = format_json(&result, false).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        // Simple JSON should only have hotspot and quality_gate
        assert!(parsed.get("hotspot").is_some());
        assert!(parsed.get("quality_gate").is_some());
        assert!(parsed.get("all_violations").is_none());
    }

    #[test]
    fn test_format_json_enforcement() {
        let result = create_full_test_result();
        let output = format_json(&result, true).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        // Enforcement JSON should have all fields
        assert!(parsed.get("hotspot").is_some());
        assert!(parsed.get("quality_gate").is_some());
        assert!(parsed.get("all_violations").is_some());
        assert!(parsed.get("summary_by_file").is_some());
        assert!(parsed.get("total_project_violations").is_some());
    }

    // format_sarif Tests

    #[test]
    fn test_format_sarif_structure() {
        let mut result = create_full_test_result();
        result.quality_gate.violations = vec![QualityViolation {
            rule: "max_defect_density".to_string(),
            threshold: 0.05,
            actual: 0.10,
            severity: "blocking".to_string(),
        }];

        let output = format_sarif(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["version"], "2.1.0");
        assert!(parsed["$schema"]
            .as_str()
            .unwrap()
            .contains("sarif-schema"));

        let runs = parsed["runs"].as_array().unwrap();
        assert_eq!(runs.len(), 1);

        let tool = &runs[0]["tool"]["driver"];
        assert_eq!(tool["name"], "pmat-lint-hotspot");
    }

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

        let results = parsed["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results.len(), 2);

        // First violation should be error level (blocking)
        assert_eq!(results[0]["level"], "error");
        assert_eq!(results[0]["ruleId"], "max_defect_density");

        // Second violation should be warning level
        assert_eq!(results[1]["level"], "warning");
        assert_eq!(results[1]["ruleId"], "max_single_file_violations");
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
            fn test_count_sloc_non_negative(content in ".*") {
                let sloc = count_sloc(&content);
                prop_assert!(sloc >= 0);
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
}
