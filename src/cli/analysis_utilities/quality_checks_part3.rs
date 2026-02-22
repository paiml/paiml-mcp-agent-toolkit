
// Helper: Format as JSON
fn format_qg_as_json(
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> Result<String> {
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "results": results,
        "violations": violations,
    }))?)
}

// Helper: Format as human-readable
fn format_qg_as_human(
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    write_qg_human_header(&mut output, results)?;
    write_qg_violation_counts(&mut output, results)?;

    if let Some(score) = results.provability_score {
        writeln!(&mut output, "\nProvability score: {score:.2}")?;
    }

    if !violations.is_empty() {
        write_qg_violations_list(&mut output, violations)?;
    }

    Ok(output)
}

// Helper: Write human header
fn write_qg_human_header(output: &mut String, results: &QualityGateResults) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "# Quality Gate Report\n")?;
    writeln!(
        output,
        "Status: {}",
        if results.passed {
            "✅ PASSED"
        } else {
            "❌ FAILED"
        }
    )?;
    writeln!(output, "Total violations: {}\n", results.total_violations)?;
    Ok(())
}

// Helper: Write violation counts
fn write_qg_violation_counts(output: &mut String, results: &QualityGateResults) -> Result<()> {
    use std::fmt::Write;
    let counts = [
        ("Complexity", results.complexity_violations),
        ("Dead code", results.dead_code_violations),
        ("Technical debt", results.satd_violations),
        ("Entropy", results.entropy_violations),
        ("Security", results.security_violations),
        ("Duplicate code", results.duplicate_violations),
    ];

    for (name, count) in counts {
        if count > 0 {
            writeln!(output, "## {name} violations: {count}")?;
        }
    }
    Ok(())
}

// Helper: Write violations list
fn write_qg_violations_list(output: &mut String, violations: &[QualityViolation]) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "\n## Violations:\n")?;
    for v in violations {
        writeln!(
            output,
            "- [{}] {} - {}",
            v.severity, v.check_type, v.message
        )?;
        if let Some(line) = v.line {
            writeln!(output, "  File: {}:{}", v.file, line)?;
        } else {
            writeln!(output, "  File: {}", v.file)?;
        }
        // Show explainability details for entropy/provability violations (#226, #229)
        write_violation_details(output, v)?;
    }
    Ok(())
}

/// Write explainability details for violations that have them (#226, #229).
fn write_violation_details(output: &mut String, v: &QualityViolation) -> Result<()> {
    use std::fmt::Write;
    let Some(details) = &v.details else {
        return Ok(());
    };
    // Score factors breakdown
    if !details.score_factors.is_empty() {
        writeln!(output, "    Factors: {}", details.score_factors.join(", "))?;
    }
    // Example code snippet
    if let Some(code) = &details.example_code {
        let trimmed = code.trim();
        if !trimmed.is_empty() {
            writeln!(output, "    Example: {}", truncate_line(trimmed, 100))?;
        }
    }
    // Affected files (only if more than 1)
    if details.affected_files.len() > 1 {
        writeln!(output, "    Files: {}", details.affected_files.join(", "))?;
    }
    Ok(())
}

/// Truncate a line to max_len characters with ellipsis.
fn truncate_line(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

// Helper: Format as JUnit XML
/// Toyota Way: Extract Method - Format quality gate as `JUnit` XML (complexity ≤8)
fn format_qg_as_junit(violations: &[QualityViolation]) -> Result<String> {
    let mut output = String::new();

    write_junit_header(&mut output)?;
    write_junit_testsuite_start(&mut output, violations.len())?;
    write_junit_testcases(&mut output, violations)?;
    write_junit_footer(&mut output)?;

    Ok(output)
}

/// Toyota Way: Extract Method - Write `JUnit` XML header (complexity ≤3)
fn write_junit_header(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(output, r#"<testsuites name="Quality Gate">"#)?;
    Ok(())
}

/// Toyota Way: Extract Method - Write `JUnit` testsuite start (complexity ≤3)
fn write_junit_testsuite_start(output: &mut String, count: usize) -> Result<()> {
    use std::fmt::Write;
    writeln!(
        output,
        r#"  <testsuite name="Quality Checks" tests="{count}" failures="{count}">"#
    )?;
    Ok(())
}

/// Toyota Way: Extract Method - Write `JUnit` testcases (complexity ≤5)
fn write_junit_testcases(output: &mut String, violations: &[QualityViolation]) -> Result<()> {
    for v in violations {
        write_single_junit_testcase(output, v)?;
    }
    Ok(())
}

/// Toyota Way: Extract Method - Write single `JUnit` testcase (complexity ≤5)
fn write_single_junit_testcase(output: &mut String, v: &QualityViolation) -> Result<()> {
    use std::fmt::Write;
    writeln!(
        output,
        r#"    <testcase name="{}" classname="{}">"#,
        v.message, v.check_type
    )?;
    writeln!(
        output,
        r#"      <failure message="{}" type="{}"/>"#,
        v.message, v.severity
    )?;
    writeln!(output, r"    </testcase>")?;
    Ok(())
}

/// Toyota Way: Extract Method - Write `JUnit` XML footer (complexity ≤3)
fn write_junit_footer(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, r"  </testsuite>")?;
    writeln!(output, r"</testsuites>")?;
    Ok(())
}

// Helper: Format as summary
fn format_qg_as_summary(
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();
    writeln!(
        &mut output,
        "Quality Gate: {}",
        if results.passed { "PASSED" } else { "FAILED" }
    )?;
    writeln!(
        &mut output,
        "Total violations: {}",
        results.total_violations
    )?;

    // Show violation summary by type
    if !violations.is_empty() {
        writeln!(&mut output)?;
        write_qg_violations_summary(&mut output, violations)?;
    }

    Ok(output)
}

// Helper: Write violation summary grouped by type
fn write_qg_violations_summary(
    output: &mut String,
    violations: &[QualityViolation],
) -> Result<()> {
    use std::collections::BTreeMap;
    use std::fmt::Write;

    // Group violations by check type
    let mut by_type: BTreeMap<&str, Vec<&QualityViolation>> = BTreeMap::new();
    for v in violations {
        by_type.entry(&v.check_type).or_default().push(v);
    }

    for (check_type, type_violations) in by_type {
        writeln!(output, "## {} ({} violations)", check_type, type_violations.len())?;
        // Show all violations (no truncation) so users can see the full list
        for v in type_violations.iter() {
            if let Some(line) = v.line {
                writeln!(output, "  - {}:{} - {}", v.file, line, v.message)?;
            } else {
                writeln!(output, "  - {} - {}", v.file, v.message)?;
            }
            // Show explainability details for entropy/provability violations (#226, #229)
            write_violation_details(output, v)?;
        }
    }
    Ok(())
}

// Helper: Format as detailed
fn format_qg_as_detailed(
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> Result<String> {
    let mut output = String::new();

    write_qg_detailed_header(&mut output, results)?;
    write_qg_detailed_summary(&mut output, results)?;

    if !violations.is_empty() {
        write_qg_detailed_violations(&mut output, violations)?;
    }

    Ok(output)
}

// Helper: Write detailed header
fn write_qg_detailed_header(output: &mut String, results: &QualityGateResults) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "# Quality Gate Detailed Report\n")?;
    writeln!(
        output,
        "Status: {}",
        if results.passed {
            "✅ PASSED"
        } else {
            "❌ FAILED"
        }
    )?;
    writeln!(output, "Total violations: {}\n", results.total_violations)?;
    Ok(())
}

// Helper: Write detailed summary
fn write_qg_detailed_summary(output: &mut String, results: &QualityGateResults) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "## Violations by Type\n")?;
    let items = [
        ("Complexity", results.complexity_violations),
        ("Dead code", results.dead_code_violations),
        ("SATD", results.satd_violations),
        ("Entropy", results.entropy_violations),
        ("Security", results.security_violations),
        ("Duplicates", results.duplicate_violations),
        ("Coverage", results.coverage_violations),
        ("Sections", results.section_violations),
        ("Provability", results.provability_violations),
    ];

    for (name, count) in items {
        writeln!(output, "- {name}: {count}")?;
    }
    Ok(())
}

// Helper: Write detailed violations
fn write_qg_detailed_violations(
    output: &mut String,
    violations: &[QualityViolation],
) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "\n## All Violations\n")?;
    for (i, v) in violations.iter().enumerate() {
        writeln!(
            output,
            "{}. [{}] {}: {}",
            i + 1,
            v.severity,
            v.check_type,
            v.message
        )?;
        if let Some(line) = v.line {
            writeln!(output, "   File: {}:{}", v.file, line)?;
        } else {
            writeln!(output, "   File: {}", v.file)?;
        }
    }
    Ok(())
}

// Helper: Format as Markdown
/// Toyota Way: Extract Method - Format quality gate as Markdown (complexity ≤8)
fn format_qg_as_markdown(
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> Result<String> {
    let mut output = String::new();

    write_qg_markdown_header(&mut output, results)?;
    write_qg_markdown_summary_table(&mut output, results)?;

    // Add violations section if any exist
    if !violations.is_empty() {
        write_qg_markdown_violations(&mut output, violations)?;
    }

    Ok(output)
}

/// Write violations section in Markdown format
fn write_qg_markdown_violations(
    output: &mut String,
    violations: &[QualityViolation],
) -> Result<()> {
    use std::collections::BTreeMap;
    use std::fmt::Write;

    writeln!(output, "\n## Violations\n")?;

    // Group violations by check type
    let mut by_type: BTreeMap<&str, Vec<&QualityViolation>> = BTreeMap::new();
    for v in violations {
        by_type.entry(&v.check_type).or_default().push(v);
    }

    for (check_type, type_violations) in by_type {
        writeln!(output, "### {} ({} issues)\n", check_type, type_violations.len())?;
        writeln!(output, "| Severity | File | Line | Message |")?;
        writeln!(output, "|----------|------|------|---------|")?;

        for v in &type_violations {
            let line_str = v.line.map_or(String::from("-"), |l| l.to_string());
            // Escape pipe characters in message for markdown table
            let escaped_msg = v.message.replace('|', "\\|");
            writeln!(
                output,
                "| {} | {} | {} | {} |",
                v.severity, v.file, line_str, escaped_msg
            )?;
        }
        writeln!(output)?;
    }

    Ok(())
}

/// Toyota Way: Extract Method - Write QG Markdown header section (complexity ≤5)
fn write_qg_markdown_header(output: &mut String, results: &QualityGateResults) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "# Quality Gate Report\n")?;
    writeln!(
        output,
        "**Status**: {}\n",
        format_qg_status_badge(results.passed)
    )?;
    writeln!(
        output,
        "**Total violations**: {}\n",
        results.total_violations
    )?;

    Ok(())
}

/// Toyota Way: Extract Method - Format QG status badge (complexity ≤3)
fn format_qg_status_badge(passed: bool) -> &'static str {
    if passed {
        "✅ PASSED"
    } else {
        "❌ FAILED"
    }
}

/// Toyota Way: Extract Method - Write QG Markdown summary table (complexity ≤8)
fn write_qg_markdown_summary_table(
    output: &mut String,
    results: &QualityGateResults,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Summary\n")?;
    write_qg_markdown_table_headers(output)?;
    write_qg_markdown_table_rows(output, results)?;

    Ok(())
}

/// Toyota Way: Extract Method - Write QG Markdown table headers (complexity ≤3)
fn write_qg_markdown_table_headers(output: &mut String) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "| Check Type | Violations |")?;
    writeln!(output, "|------------|------------|")?;

    Ok(())
}

/// Toyota Way: Extract Method - Write QG Markdown table rows (complexity ≤5)
fn write_qg_markdown_table_rows(output: &mut String, results: &QualityGateResults) -> Result<()> {
    use std::fmt::Write;

    let rows = get_qg_violation_summary_rows(results);

    for (name, count) in rows {
        writeln!(output, "| {name} | {count} |")?;
    }

    Ok(())
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod quality_checks_part3_tests {
    use super::*;

    fn create_test_results(passed: bool, total: usize) -> QualityGateResults {
        QualityGateResults {
            passed,
            total_violations: total,
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
        assert_eq!(format_qg_status_badge(true), "✅ PASSED");
    }

    #[test]
    fn test_format_qg_status_badge_failed() {
        assert_eq!(format_qg_status_badge(false), "❌ FAILED");
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
        assert!(output.contains("✅ PASSED"));
    }

    #[test]
    fn test_write_qg_human_header_failed() {
        let results = create_test_results(false, 5);
        let mut output = String::new();
        write_qg_human_header(&mut output, &results).unwrap();
        assert!(output.contains("❌ FAILED"));
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
