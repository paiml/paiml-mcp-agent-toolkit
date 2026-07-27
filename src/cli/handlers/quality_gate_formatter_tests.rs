#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Helper function to create test quality gate results
    fn create_test_results(passed: bool, total: usize) -> QualityGateResults {
        QualityGateResults {
            passed,
            total_violations: total,
            complexity_violations: total / 3,
            dead_code_violations: total / 3,
            satd_violations: total / 3,
            entropy_violations: 0,
            security_violations: 0,
            duplicate_violations: 0,
            coverage_violations: 0,
            section_violations: 0,
            provability_violations: 0,
            provability_score: None,
            violations: vec![],
        }
    }

    // Helper function to create test violations
    fn create_test_violations() -> Vec<QualityViolation> {
        vec![
            QualityViolation {
                check_type: "Complexity".to_string(),
                severity: "high".to_string(),
                file: "test.rs".to_string(),
                line: Some(42),
                message: "Function complexity exceeds threshold".to_string(),
                details: None,
            },
            QualityViolation {
                check_type: "SATD".to_string(),
                severity: "medium".to_string(),
                file: "test.rs".to_string(),
                line: Some(100),
                message: "TODO comment found".to_string(),
                details: None,
            },
        ]
    }

    #[test]
    fn test_format_single_file_output_json() {
        let file_path = PathBuf::from("test.rs");
        let results = create_test_results(false, 6);
        let violations = create_test_violations();

        let output = format_single_file_output(
            &file_path,
            &results,
            &violations,
            QualityGateOutputFormat::Json,
        )
        .unwrap();

        // Verify JSON structure
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["passed"], false);
        assert_eq!(parsed["file"], "test.rs");
        assert!(parsed["violations"].is_array());
    }

    #[test]
    fn test_format_single_file_output_summary() {
        let file_path = PathBuf::from("test.rs");
        let results = create_test_results(true, 0);
        let violations = vec![];

        let output = format_single_file_output(
            &file_path,
            &results,
            &violations,
            QualityGateOutputFormat::Summary,
        )
        .unwrap();

        // Verify summary contains key information
        assert!(output.contains("Quality Gate Report"));
        assert!(output.contains("✅"));
        assert!(output.contains("test.rs"));
    }

    #[test]
    fn test_format_project_output_json() {
        let results = create_test_results(false, 9);
        let violations = create_test_violations();

        let output =
            format_project_output(&results, &violations, QualityGateOutputFormat::Json).unwrap();

        // Verify JSON structure
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["passed"], false);
        assert_eq!(parsed["results"]["total_violations"], 9);
    }

    #[test]
    fn test_format_project_output_summary() {
        let results = create_test_results(true, 0);
        let violations = vec![];

        let output =
            format_project_output(&results, &violations, QualityGateOutputFormat::Summary).unwrap();

        // Verify summary format
        assert!(output.contains("Quality Gate Results"));
        assert!(output.contains("✅ PASSED"));
        assert!(output.contains("Total violations: 0"));
    }

    #[test]
    fn test_format_qg_as_junit() {
        let violations = create_test_violations();
        let output = format_qg_as_junit(&violations).unwrap();

        // Verify JUnit XML structure
        assert!(output.contains("<?xml version"));
        assert!(output.contains("<testsuites"));
        assert!(output.contains("<testsuite"));
        assert!(output.contains("<testcase"));
        assert!(output.contains("<failure"));
        assert!(output.contains("failures=\"2\""));
    }

    #[test]
    fn test_format_qg_as_junit_empty() {
        let violations = vec![];
        let output = format_qg_as_junit(&violations).unwrap();

        // Verify empty JUnit XML
        assert!(output.contains("failures=\"0\""));
        assert!(!output.contains("<testcase"));
    }

    #[test]
    fn test_group_violations_by_type() {
        let violations = vec![
            QualityViolation {
                check_type: "Complexity".to_string(),
                severity: "high".to_string(),
                file: "a.rs".to_string(),
                line: Some(1),
                message: "Complex function".to_string(),
                details: None,
            },
            QualityViolation {
                check_type: "Complexity".to_string(),
                severity: "high".to_string(),
                file: "b.rs".to_string(),
                line: Some(2),
                message: "Another complex function".to_string(),
                details: None,
            },
            QualityViolation {
                check_type: "SATD".to_string(),
                severity: "medium".to_string(),
                file: "c.rs".to_string(),
                line: Some(3),
                message: "TODO found".to_string(),
                details: None,
            },
        ];

        let grouped = group_violations_by_type(&violations);

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get("Complexity").unwrap().len(), 2);
        assert_eq!(grouped.get("SATD").unwrap().len(), 1);
    }

    #[test]
    fn test_format_single_file_summary() {
        let file_path = PathBuf::from("src/main.rs");
        let results = create_test_results(false, 3);
        let violations = create_test_violations();

        let output = format_single_file_summary(&file_path, &results, &violations);

        // Verify summary contains all key elements
        assert!(output.contains("Quality Gate Report: src/main.rs"));
        assert!(output.contains("❌"));
        assert!(output.contains("Total Violations: 3"));
        assert!(output.contains("## Violations"));
    }

    #[test]
    fn test_format_project_summary() {
        let results = create_test_results(true, 0);
        let violations = vec![];

        let output = format_project_summary(&results, &violations);

        // Verify project summary format
        assert!(output.contains("Quality Gate Results"));
        assert!(output.contains("✅ PASSED"));
        assert!(output.contains("No violations found"));
    }

    #[test]
    fn test_format_violations_markdown() {
        let violations = create_test_violations();
        let grouped = group_violations_by_type(&violations);

        let output = format_violations_markdown(&grouped);

        // Verify markdown format
        assert!(output.contains("### Complexity (1 violation)"));
        assert!(output.contains("### SATD (1 violation)"));
        assert!(output.contains("**test.rs:42**"));
        assert!(output.contains("Function complexity exceeds threshold"));
    }

    #[test]
    fn test_print_check_timing() {
        // Test that print_check_timing doesn't panic for various check types
        print_check_timing(&QualityCheckType::Complexity, 1.234);
        print_check_timing(&QualityCheckType::DeadCode, 0.567);
        print_check_timing(&QualityCheckType::All, 5.678);
        // No assertions needed - just verify no panic
    }

    #[test]
    fn test_write_junit_test_case() {
        let mut output = String::new();
        let violation = QualityViolation {
            check_type: "Complexity".to_string(),
            severity: "high".to_string(),
            file: "test.rs".to_string(),
            line: Some(42),
            message: "Function too complex".to_string(),
            details: None,
        };

        write_junit_test_case(&mut output, &violation).unwrap();

        // Verify XML structure
        assert!(output.contains("<testcase"));
        assert!(output.contains("name=\"Function too complex\""));
        assert!(output.contains("classname=\"Complexity\""));
        assert!(output.contains("<failure"));
        assert!(output.contains("type=\"high\""));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod violation_display_tests {
    use super::*;
    use crate::cli::analysis_utilities::QualityViolation;

    #[test]
    fn test_violation_with_file_and_line_displays_correctly() {
        let violation = QualityViolation {
            check_type: "complexity".to_string(),
            severity: "error".to_string(),
            file: "/path/to/src/services/complex_module.rs".to_string(),
            line: Some(42),
            message: "Function too complex".to_string(),
            details: None,
        };

        let mut output = String::new();
        add_violation_entry(&mut output, &violation);

        // Should include filename (not full path for readability)
        assert!(
            output.contains("complex_module.rs"),
            "Output should contain filename: {}",
            output
        );
        // Should include line number
        assert!(
            output.contains(":42:"),
            "Output should contain line number: {}",
            output
        );
        // Should include message
        assert!(
            output.contains("Function too complex"),
            "Output should contain message: {}",
            output
        );
    }

    #[test]
    fn test_violation_with_file_no_line_displays_correctly() {
        let violation = QualityViolation {
            check_type: "security".to_string(),
            severity: "warning".to_string(),
            file: "/path/to/src/config.rs".to_string(),
            line: None,
            message: "Potential security issue".to_string(),
            details: None,
        };

        let mut output = String::new();
        add_violation_entry(&mut output, &violation);

        // Should include filename
        assert!(
            output.contains("config.rs"),
            "Output should contain filename: {}",
            output
        );
        // Should NOT include colon-number pattern for line
        assert!(
            !output.contains(":None"),
            "Output should not contain :None: {}",
            output
        );
    }

    #[test]
    fn test_violation_without_file_displays_message() {
        let violation = QualityViolation {
            check_type: "general".to_string(),
            severity: "info".to_string(),
            file: String::new(),
            line: None,
            message: "General information".to_string(),
            details: None,
        };

        let mut output = String::new();
        add_violation_entry(&mut output, &violation);

        // Should just have message
        assert!(
            output.contains("General information"),
            "Output should contain message: {}",
            output
        );
    }

    #[test]
    fn test_severity_icons_correct() {
        assert_eq!(get_severity_icon("error"), "🔴");
        assert_eq!(get_severity_icon("warning"), "🟡");
        assert_eq!(get_severity_icon("info"), "🟢");
    }
}
