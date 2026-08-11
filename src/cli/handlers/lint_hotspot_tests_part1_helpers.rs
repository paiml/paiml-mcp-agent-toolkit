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
