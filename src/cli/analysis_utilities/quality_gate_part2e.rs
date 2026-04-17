#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod quality_gate_unit_tests {
    use super::*;

    // ===================
    // get_severity_icon Tests
    // ===================

    #[test]
    fn test_get_severity_icon_error() {
        assert_eq!(get_severity_icon("error"), "🔴");
    }

    #[test]
    fn test_get_severity_icon_warning() {
        assert_eq!(get_severity_icon("warning"), "🟡");
    }

    #[test]
    fn test_get_severity_icon_other() {
        assert_eq!(get_severity_icon("info"), "🟢");
        assert_eq!(get_severity_icon("note"), "🟢");
        assert_eq!(get_severity_icon("suggestion"), "🟢");
        assert_eq!(get_severity_icon(""), "🟢");
    }

    // ===================
    // get_check_message Tests
    // ===================

    #[test]
    fn test_get_check_message_complexity() {
        let result = get_check_message(&QualityCheckType::Complexity);
        assert_eq!(result, Some("Complexity analysis"));
    }

    #[test]
    fn test_get_check_message_dead_code() {
        let result = get_check_message(&QualityCheckType::DeadCode);
        assert_eq!(result, Some("Dead code detection"));
    }

    #[test]
    fn test_get_check_message_satd() {
        let result = get_check_message(&QualityCheckType::Satd);
        assert_eq!(result, Some("Self-admitted technical debt (SATD)"));
    }

    #[test]
    fn test_get_check_message_security() {
        let result = get_check_message(&QualityCheckType::Security);
        assert_eq!(result, Some("Security vulnerabilities"));
    }

    #[test]
    fn test_get_check_message_entropy() {
        let result = get_check_message(&QualityCheckType::Entropy);
        assert_eq!(result, Some("Code entropy"));
    }

    #[test]
    fn test_get_check_message_duplicates() {
        let result = get_check_message(&QualityCheckType::Duplicates);
        assert_eq!(result, Some("Duplicate code"));
    }

    #[test]
    fn test_get_check_message_coverage() {
        let result = get_check_message(&QualityCheckType::Coverage);
        assert_eq!(result, Some("Test coverage"));
    }

    #[test]
    fn test_get_check_message_all() {
        let result = get_check_message(&QualityCheckType::All);
        assert!(result.is_none());
    }

    // ===================
    // format_report_header Tests
    // ===================

    #[test]
    fn test_format_report_header_passed() {
        let mut output = String::new();
        format_report_header(&mut output, Path::new("src/test.rs"), true);
        assert!(output.contains("Quality Gate Report: src/test.rs"));
        assert!(output.contains("PASSED"));
        assert!(output.contains("✅"));
    }

    #[test]
    fn test_format_report_header_failed() {
        let mut output = String::new();
        format_report_header(&mut output, Path::new("src/main.rs"), false);
        assert!(output.contains("Quality Gate Report: src/main.rs"));
        assert!(output.contains("FAILED"));
        assert!(output.contains("❌"));
    }

    // ===================
    // format_results_summary Tests
    // ===================

    #[test]
    fn test_format_results_summary_zeros() {
        let results = QualityGateResults::default();

        let mut output = String::new();
        format_results_summary(&mut output, &results);

        assert!(output.contains("## Summary"));
        assert!(output.contains("Total Violations: 0"));
        assert!(output.contains("Complexity Issues: 0"));
        assert!(output.contains("Dead Code: 0"));
        assert!(output.contains("Technical Debt (SATD): 0"));
        assert!(output.contains("Security Issues: 0"));
    }

    #[test]
    fn test_format_results_summary_with_violations() {
        let results = QualityGateResults {
            passed: false,
            total_violations: 10,
            complexity_violations: 3,
            dead_code_violations: 2,
            satd_violations: 4,
            security_violations: 1,
            ..Default::default()
        };

        let mut output = String::new();
        format_results_summary(&mut output, &results);

        assert!(output.contains("Total Violations: 10"));
        assert!(output.contains("Complexity Issues: 3"));
        assert!(output.contains("Dead Code: 2"));
        assert!(output.contains("Technical Debt (SATD): 4"));
        assert!(output.contains("Security Issues: 1"));
    }

    // ===================
    // QualityViolation Tests
    // ===================

    #[test]
    fn test_quality_violation_struct() {
        let violation = QualityViolation {
            check_type: "complexity".to_string(),
            severity: "error".to_string(),
            file: "src/main.rs".to_string(),
            line: Some(42),
            message: "Function too complex".to_string(),
            details: None,
        };

        assert_eq!(violation.check_type, "complexity");
        assert_eq!(violation.severity, "error");
        assert_eq!(violation.file, "src/main.rs");
        assert_eq!(violation.line, Some(42));
    }

    #[test]
    fn test_quality_violation_no_line() {
        let violation = QualityViolation {
            check_type: "dead_code".to_string(),
            severity: "warning".to_string(),
            file: "src/lib.rs".to_string(),
            line: None,
            message: "Unused function".to_string(),
            details: None,
        };

        assert!(violation.line.is_none());
    }

    // ===================
    // QualityGateResults Tests
    // ===================

    #[test]
    fn test_quality_gate_results_default() {
        let results = QualityGateResults::default();
        // Default is passed: true when no violations
        assert!(results.passed);
        assert_eq!(results.total_violations, 0);
        assert_eq!(results.complexity_violations, 0);
        assert_eq!(results.dead_code_violations, 0);
        assert_eq!(results.satd_violations, 0);
        assert_eq!(results.security_violations, 0);
        assert!(results.violations.is_empty());
    }

    #[test]
    fn test_quality_gate_results_with_values() {
        let results = QualityGateResults {
            passed: true,
            total_violations: 5,
            complexity_violations: 2,
            dead_code_violations: 1,
            satd_violations: 1,
            security_violations: 1,
            ..Default::default()
        };

        assert!(results.passed);
        assert_eq!(results.total_violations, 5);
    }

    // ===================
    // format_violations_section Tests
    // ===================

    #[test]
    fn test_format_violations_section_empty() {
        let violations: Vec<QualityViolation> = vec![];
        let mut output = String::new();
        format_violations_section(&mut output, &violations);
        assert!(output.contains("## Violations"));
    }

    #[test]
    fn test_format_violations_section_single() {
        let violations = vec![QualityViolation {
            check_type: "complexity".to_string(),
            severity: "error".to_string(),
            file: "src/main.rs".to_string(),
            line: Some(10),
            message: "Too complex".to_string(),
            details: None,
        }];

        let mut output = String::new();
        format_violations_section(&mut output, &violations);

        assert!(output.contains("## Violations"));
        assert!(output.contains("COMPLEXITY"));
        assert!(output.contains("main.rs"));
    }

    #[test]
    fn test_format_violations_section_multiple_types() {
        let violations = vec![
            QualityViolation {
                check_type: "complexity".to_string(),
                severity: "error".to_string(),
                file: "src/a.rs".to_string(),
                line: Some(10),
                message: "Complex".to_string(),
                details: None,
            },
            QualityViolation {
                check_type: "security".to_string(),
                severity: "error".to_string(),
                file: "src/b.rs".to_string(),
                line: Some(20),
                message: "Unsafe".to_string(),
                details: None,
            },
        ];

        let mut output = String::new();
        format_violations_section(&mut output, &violations);

        assert!(output.contains("COMPLEXITY"));
        assert!(output.contains("SECURITY"));
    }

    // ===================
    // format_single_violation Tests
    // ===================

    #[test]
    fn test_format_single_violation_with_line() {
        let violation = QualityViolation {
            check_type: "complexity".to_string(),
            severity: "error".to_string(),
            file: "src/main.rs".to_string(),
            line: Some(42),
            message: "Function too complex".to_string(),
            details: None,
        };

        let mut output = String::new();
        format_single_violation(&mut output, &violation);

        assert!(output.contains("🔴")); // error icon
        assert!(output.contains("main.rs"));
        assert!(output.contains("42"));
        assert!(output.contains("Function too complex"));
    }

    #[test]
    fn test_format_single_violation_without_line() {
        let violation = QualityViolation {
            check_type: "dead_code".to_string(),
            severity: "warning".to_string(),
            file: "src/lib.rs".to_string(),
            line: None,
            message: "Unused code".to_string(),
            details: None,
        };

        let mut output = String::new();
        format_single_violation(&mut output, &violation);

        assert!(output.contains("🟡")); // warning icon
        assert!(output.contains("lib.rs"));
        assert!(output.contains("Unused code"));
    }

    #[test]
    fn test_format_single_violation_no_file() {
        let violation = QualityViolation {
            check_type: "satd".to_string(),
            severity: "info".to_string(),
            file: String::new(),
            line: None,
            message: "Technical debt found".to_string(),
            details: None,
        };

        let mut output = String::new();
        format_single_violation(&mut output, &violation);

        assert!(output.contains("🟢")); // other/info icon
        assert!(output.contains("Technical debt found"));
    }

    // ===================
    // resolve_absolute_file_path Tests
    // ===================

    #[test]
    fn test_resolve_absolute_file_path_already_absolute() {
        let project = Path::new("/home/user/project");
        let file = Path::new("/home/user/project/src/main.rs");
        let result = resolve_absolute_file_path(project, file);
        assert_eq!(result, PathBuf::from("/home/user/project/src/main.rs"));
    }

    #[test]
    fn test_resolve_absolute_file_path_relative() {
        let project = Path::new("/home/user/project");
        let file = Path::new("src/main.rs");
        let result = resolve_absolute_file_path(project, file);
        assert_eq!(result, PathBuf::from("/home/user/project/src/main.rs"));
    }
}
