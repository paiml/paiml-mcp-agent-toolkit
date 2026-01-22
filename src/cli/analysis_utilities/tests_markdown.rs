#[cfg(test)]
mod markdown_formatting_tests {
    use super::QualityGateResults;
    use super::*;

    /// Create test quality gate results for testing
    fn create_test_quality_results(passed: bool, violations: u64) -> QualityGateResults {
        QualityGateResults {
            passed,
            total_violations: violations as usize,
            complexity_violations: (violations / 3) as usize,
            dead_code_violations: (violations / 4) as usize,
            satd_violations: (violations / 5) as usize,
            entropy_violations: (violations / 6) as usize,
            security_violations: (violations / 7) as usize,
            duplicate_violations: (violations / 8) as usize,
            coverage_violations: (violations / 9) as usize,
            section_violations: (violations / 10) as usize,
            provability_violations: (violations / 11) as usize,
            provability_score: None,
            violations: Vec::new(),
        }
    }

    #[test]
    fn test_format_status_badge_passed() {
        let badge = format_qg_status_badge(true);
        assert_eq!(badge, "✅ PASSED");
    }

    #[test]
    fn test_format_status_badge_failed() {
        let badge = format_qg_status_badge(false);
        assert_eq!(badge, "❌ FAILED");
    }

    #[test]
    fn test_write_markdown_header() {
        let mut output = String::new();
        let results = create_test_quality_results(true, 10);

        let result = write_qg_markdown_header(&mut output, &results);
        assert!(result.is_ok());

        assert!(output.contains("# Quality Gate Report"));
        assert!(output.contains("**Status**: ✅ PASSED"));
        assert!(output.contains("**Total violations**: 10"));
    }

    #[test]
    fn test_write_markdown_header_failed() {
        let mut output = String::new();
        let results = create_test_quality_results(false, 25);

        let result = write_qg_markdown_header(&mut output, &results);
        assert!(result.is_ok());

        assert!(output.contains("**Status**: ❌ FAILED"));
        assert!(output.contains("**Total violations**: 25"));
    }

    #[test]
    fn test_write_markdown_table_headers() {
        let mut output = String::new();

        let result = write_qg_markdown_table_headers(&mut output);
        assert!(result.is_ok());

        assert!(output.contains("| Check Type | Violations |"));
        assert!(output.contains("|------------|------------|"));
    }

    #[test]
    fn test_get_violation_summary_rows() {
        let results = create_test_quality_results(false, 90);
        let rows = get_qg_violation_summary_rows(&results);

        assert_eq!(rows.len(), 9);
        assert_eq!(rows[0], ("Complexity", 30)); // 90/3
        assert_eq!(rows[1], ("Dead Code", 22)); // 90/4
        assert_eq!(rows[2], ("SATD", 18)); // 90/5
        assert_eq!(rows[3], ("Entropy", 15)); // 90/6
        assert_eq!(rows[4], ("Security", 12)); // 90/7
        assert_eq!(rows[5], ("Duplicates", 11)); // 90/8
        assert_eq!(rows[6], ("Coverage", 10)); // 90/9
        assert_eq!(rows[7], ("Sections", 9)); // 90/10
        assert_eq!(rows[8], ("Provability", 8)); // 90/11
    }

    #[test]
    fn test_write_markdown_table_rows() {
        let mut output = String::new();
        let results = create_test_quality_results(false, 45);

        let result = write_qg_markdown_table_rows(&mut output, &results);
        assert!(result.is_ok());

        // Check that all violation types are included
        assert!(output.contains("| Complexity | 15 |")); // 45/3
        assert!(output.contains("| Dead Code | 11 |")); // 45/4
        assert!(output.contains("| SATD | 9 |")); // 45/5
        assert!(output.contains("| Entropy | 7 |")); // 45/6
        assert!(output.contains("| Security | 6 |")); // 45/7
        assert!(output.contains("| Duplicates | 5 |")); // 45/8
        assert!(output.contains("| Coverage | 5 |")); // 45/9
        assert!(output.contains("| Sections | 4 |")); // 45/10
        assert!(output.contains("| Provability | 4 |")); // 45/11
    }

    #[test]
    fn test_write_markdown_summary_table() {
        let mut output = String::new();
        let results = create_test_quality_results(true, 0);

        let result = write_qg_markdown_summary_table(&mut output, &results);
        assert!(result.is_ok());

        assert!(output.contains("## Summary"));
        assert!(output.contains("| Check Type | Violations |"));
        assert!(output.contains("|------------|------------|"));
        assert!(output.contains("| Complexity | 0 |"));
        assert!(output.contains("| Dead Code | 0 |"));
        assert!(output.contains("| SATD | 0 |"));
    }

    #[test]
    fn test_format_qg_as_markdown_integration() {
        let results = create_test_quality_results(false, 33);
        let violations: Vec<QualityViolation> = Vec::new();

        let output = format_qg_as_markdown(&results, &violations);
        assert!(output.is_ok());

        let markdown = output.unwrap();

        // Check all sections are present
        assert!(markdown.contains("# Quality Gate Report"));
        assert!(markdown.contains("**Status**: ❌ FAILED"));
        assert!(markdown.contains("**Total violations**: 33"));
        assert!(markdown.contains("## Summary"));
        assert!(markdown.contains("| Check Type | Violations |"));
        assert!(markdown.contains("|------------|------------|"));

        // Check specific violation counts (33 divided by denominators)
        assert!(markdown.contains("| Complexity | 11 |")); // 33/3
        assert!(markdown.contains("| Dead Code | 8 |")); // 33/4
        assert!(markdown.contains("| SATD | 6 |")); // 33/5
        assert!(markdown.contains("| Entropy | 5 |")); // 33/6
        assert!(markdown.contains("| Security | 4 |")); // 33/7
        assert!(markdown.contains("| Duplicates | 4 |")); // 33/8
        assert!(markdown.contains("| Coverage | 3 |")); // 33/9
        assert!(markdown.contains("| Sections | 3 |")); // 33/10
        assert!(markdown.contains("| Provability | 3 |")); // 33/11
    }

    #[test]
    fn test_format_qg_as_markdown_passed_state() {
        let results = create_test_quality_results(true, 0);
        let violations: Vec<QualityViolation> = Vec::new();

        let output = format_qg_as_markdown(&results, &violations);
        assert!(output.is_ok());

        let markdown = output.unwrap();

        assert!(markdown.contains("**Status**: ✅ PASSED"));
        assert!(markdown.contains("**Total violations**: 0"));

        // All violation counts should be zero
        assert!(markdown.contains("| Complexity | 0 |"));
        assert!(markdown.contains("| Dead Code | 0 |"));
        assert!(markdown.contains("| SATD | 0 |"));
        assert!(markdown.contains("| Entropy | 0 |"));
        assert!(markdown.contains("| Security | 0 |"));
        assert!(markdown.contains("| Duplicates | 0 |"));
        assert!(markdown.contains("| Coverage | 0 |"));
        assert!(markdown.contains("| Sections | 0 |"));
        assert!(markdown.contains("| Provability | 0 |"));
    }

    /// Property test: Markdown output should always be valid and complete
    #[test]
    fn test_markdown_output_completeness() {
        let empty_violations: Vec<QualityViolation> = Vec::new();
        for violations in [0, 1, 10, 50, 100, 999] {
            for passed in [true, false] {
                let results = create_test_quality_results(passed, violations);
                let output = format_qg_as_markdown(&results, &empty_violations);

                assert!(
                    output.is_ok(),
                    "Markdown formatting failed for violations={}, passed={}",
                    violations,
                    passed
                );

                let markdown = output.unwrap();

                // Essential sections must always be present
                assert!(markdown.contains("# Quality Gate Report"), "Missing header");
                assert!(markdown.contains("**Status**:"), "Missing status");
                assert!(
                    markdown.contains("**Total violations**:"),
                    "Missing total violations"
                );
                assert!(markdown.contains("## Summary"), "Missing summary section");
                assert!(
                    markdown.contains("| Check Type | Violations |"),
                    "Missing table header"
                );
                assert!(
                    markdown.contains("|------------|------------|"),
                    "Missing table separator"
                );

                // All violation types must be present
                for violation_type in [
                    "Complexity",
                    "Dead Code",
                    "SATD",
                    "Entropy",
                    "Security",
                    "Duplicates",
                    "Coverage",
                    "Sections",
                    "Provability",
                ] {
                    assert!(
                        markdown.contains(&format!("| {} |", violation_type)),
                        "Missing violation type: {}",
                        violation_type
                    );
                }
            }
        }
    }
}
