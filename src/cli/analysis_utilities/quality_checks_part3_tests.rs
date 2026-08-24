
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod quality_checks_part3_tests {
    use super::*;

    fn create_test_results(passed: bool, total: usize) -> QualityGateResults {
        QualityGateResults {
            files_examined: 0,
            files_not_read: std::collections::BTreeMap::new(),
            checks_run: Vec::new(),
            passed,
            total_violations: total,
            blocking_violations: total,
            complexity_violations: 2,
            dead_code_violations: 1,
            satd_violations: 3,
            entropy_violations: 0,
            security_violations: 1,
            duplicate_violations: 0,
            coverage_violations: 0,
            section_violations: 0,
            provability_violations: 0,
            provability_score: Some(0.85),
            violations: vec![],
        }
    }

    fn create_test_violation(check_type: &str, message: &str) -> QualityViolation {
        QualityViolation {
            check_type: check_type.to_string(),
            message: message.to_string(),
            file: "src/test.rs".to_string(),
            line: Some(42),
            severity: "warning".to_string(),
            details: None,
        }
    }

    #[test]
    fn test_format_qg_status_badge_passed() {
        assert_eq!(format_qg_status_badge(true), "\u{2705} PASSED");
    }

    #[test]
    fn test_format_qg_status_badge_failed() {
        assert_eq!(format_qg_status_badge(false), "\u{274c} FAILED");
    }

    #[test]
    fn test_write_junit_header() {
        let mut output = String::new();
        write_junit_header(&mut output).unwrap();
        assert!(output.contains(r#"<?xml version="1.0""#));
        assert!(output.contains("<testsuites"));
    }

    #[test]
    fn test_write_junit_testsuite_start() {
        let mut output = String::new();
        write_junit_testsuite_start(&mut output, 5).unwrap();
        assert!(output.contains(r#"tests="5""#));
        assert!(output.contains(r#"failures="5""#));
    }

    #[test]
    fn test_write_junit_footer() {
        let mut output = String::new();
        write_junit_footer(&mut output).unwrap();
        assert!(output.contains("</testsuite>"));
        assert!(output.contains("</testsuites>"));
    }

    #[test]
    fn test_write_single_junit_testcase() {
        let violation = create_test_violation("complexity", "Function too complex");
        let mut output = String::new();
        write_single_junit_testcase(&mut output, &violation).unwrap();
        assert!(output.contains("<testcase"));
        assert!(output.contains("<failure"));
        assert!(output.contains("</testcase>"));
    }

    /// The sibling `ci-local` JUnit writer emitted unparseable XML because it
    /// passed child output through unescaped; this writer had the same defect in
    /// a worse form — violation fields went straight into XML *attributes* with
    /// no escaping at all. The pre-existing tests here only asserted that
    /// `<testcase` and `<failure` appear, which stays true no matter how badly
    /// the attribute values corrupt the document.
    #[test]
    fn junit_attributes_are_escaped_so_the_document_stays_parseable() {
        let violation = create_test_violation(
            "complexity",
            r#"Function "render<T>" exceeds 25 & needs work"#,
        );
        let mut output = String::new();
        write_single_junit_testcase(&mut output, &violation).unwrap();

        // The raw forms would each terminate or corrupt the attribute.
        assert!(!output.contains("render<T>"), "{output}");
        assert!(!output.contains(r#""render"#), "{output}");
        assert!(output.contains("&lt;T&gt;"), "{output}");
        assert!(output.contains("&quot;"), "{output}");
        assert!(output.contains("&amp;"), "{output}");

        // Every attribute value must be free of the delimiter that closes it.
        for (_, rest) in output.match_indices("name=\"").map(|(i, _)| (i, &output[i + 6..])) {
            let value = rest.split('"').next().expect("attribute value");
            assert!(!value.contains('<'), "raw '<' inside attribute: {value:?}");
        }
    }

    #[test]
    fn test_format_qg_as_junit() {
        let violations = vec![
            create_test_violation("complexity", "High complexity"),
            create_test_violation("satd", "TODO found"),
        ];
        let output = format_qg_as_junit(&violations).unwrap();
        assert!(output.contains("<?xml"));
        assert!(output.contains("<testsuites"));
        assert!(output.contains("</testsuites>"));
    }

    #[test]
    fn test_write_qg_human_header_passed() {
        let results = create_test_results(true, 0);
        let mut output = String::new();
        write_qg_human_header(&mut output, &results).unwrap();
        assert!(output.contains("# Quality Gate Report"));
        assert!(output.contains("\u{2705} PASSED"));
    }

    #[test]
    fn test_write_qg_human_header_failed() {
        let results = create_test_results(false, 5);
        let mut output = String::new();
        write_qg_human_header(&mut output, &results).unwrap();
        assert!(output.contains("\u{274c} FAILED"));
        assert!(output.contains("Total violations: 5"));
    }

    #[test]
    fn test_write_qg_violation_counts() {
        let results = create_test_results(false, 7);
        let mut output = String::new();
        write_qg_violation_counts(&mut output, &results).unwrap();
        assert!(output.contains("## Complexity violations: 2"));
        assert!(output.contains("## Dead code violations: 1"));
        assert!(output.contains("## Technical debt violations: 3"));
    }

    #[test]
    fn test_write_qg_violations_list() {
        let violations = vec![
            create_test_violation("complexity", "Function too complex"),
        ];
        let mut output = String::new();
        write_qg_violations_list(&mut output, &violations).unwrap();
        assert!(output.contains("## Violations:"));
        assert!(output.contains("src/test.rs:42"));
    }

    #[test]
    fn test_write_qg_violations_list_no_line() {
        let mut violation = create_test_violation("complexity", "Complex");
        violation.line = None;
        let violations = vec![violation];
        let mut output = String::new();
        write_qg_violations_list(&mut output, &violations).unwrap();
        assert!(output.contains("File: src/test.rs"));
        assert!(!output.contains(":42"));
    }

    #[test]
    fn test_format_qg_as_human() {
        let results = create_test_results(true, 2);
        let violations = vec![create_test_violation("satd", "TODO")];
        let output = format_qg_as_human(&results, &violations).unwrap();
        assert!(output.contains("# Quality Gate Report"));
        assert!(output.contains("Provability score: 0.85"));
    }

    #[test]
    fn test_format_qg_as_json() {
        let results = create_test_results(true, 1);
        let violations = vec![create_test_violation("test", "msg")];
        let output = format_qg_as_json(&results, &violations).unwrap();
        assert!(output.contains("\"results\""));
        assert!(output.contains("\"violations\""));
    }

    #[test]
    fn test_write_qg_markdown_header() {
        let results = create_test_results(false, 3);
        let mut output = String::new();
        write_qg_markdown_header(&mut output, &results).unwrap();
        assert!(output.contains("# Quality Gate Report"));
        assert!(output.contains("**Status**"));
        assert!(output.contains("**Total violations**: 3"));
    }

    #[test]
    fn test_write_qg_markdown_table_headers() {
        let mut output = String::new();
        write_qg_markdown_table_headers(&mut output).unwrap();
        assert!(output.contains("| Check Type | Violations |"));
        assert!(output.contains("|------------|------------|"));
    }

    #[test]
    fn test_format_qg_as_summary() {
        let results = create_test_results(true, 1);
        let violations = vec![create_test_violation("test", "msg")];
        let output = format_qg_as_summary(&results, &violations).unwrap();
        assert!(output.contains("Quality Gate: PASSED"));
        assert!(output.contains("Total violations: 1"));
    }

    #[test]
    fn test_write_qg_detailed_header() {
        let results = create_test_results(true, 0);
        let mut output = String::new();
        write_qg_detailed_header(&mut output, &results).unwrap();
        assert!(output.contains("# Quality Gate Detailed Report"));
    }

    #[test]
    fn test_write_qg_detailed_summary() {
        let results = create_test_results(false, 7);
        let mut output = String::new();
        write_qg_detailed_summary(&mut output, &results).unwrap();
        assert!(output.contains("## Violations by Type"));
        assert!(output.contains("- Complexity: 2"));
        assert!(output.contains("- SATD: 3"));
    }

    #[test]
    fn test_write_qg_detailed_violations() {
        let violations = vec![
            create_test_violation("complexity", "Too complex"),
        ];
        let mut output = String::new();
        write_qg_detailed_violations(&mut output, &violations).unwrap();
        assert!(output.contains("## All Violations"));
        assert!(output.contains("1. [warning]"));
    }

    #[test]
    fn test_format_qg_as_detailed() {
        let results = create_test_results(false, 1);
        let violations = vec![create_test_violation("test", "msg")];
        let output = format_qg_as_detailed(&results, &violations).unwrap();
        assert!(output.contains("# Quality Gate Detailed Report"));
        assert!(output.contains("## Violations by Type"));
        assert!(output.contains("## All Violations"));
    }

    #[test]
    fn test_format_qg_as_markdown() {
        let results = create_test_results(true, 1);
        let violations = vec![create_test_violation("complexity", "msg")];
        let output = format_qg_as_markdown(&results, &violations).unwrap();
        assert!(output.contains("# Quality Gate Report"));
        assert!(output.contains("## Violations"));
    }

    #[test]
    fn test_write_qg_markdown_violations() {
        let violations = vec![
            create_test_violation("complexity", "Complex function"),
            create_test_violation("satd", "TODO: fix"),
        ];
        let mut output = String::new();
        write_qg_markdown_violations(&mut output, &violations).unwrap();
        assert!(output.contains("| Severity | File | Line | Message |"));
        assert!(output.contains("### complexity"));
        assert!(output.contains("### satd"));
    }
}
