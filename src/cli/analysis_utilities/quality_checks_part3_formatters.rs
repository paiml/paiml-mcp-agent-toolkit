
// Helper: Format as JSON
fn format_qg_as_json(
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> Result<String> {
    debug_assert!(!violations.is_empty(), "violations must not be empty");
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
    debug_assert!(!violations.is_empty(), "violations must not be empty");
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
    debug_assert!(true, "contract: write_qg_human_header");
    use std::fmt::Write;
    writeln!(output, "# Quality Gate Report\n")?;
    writeln!(
        output,
        "Status: {}",
        if results.passed {
            "\u{2705} PASSED"
        } else {
            "\u{274c} FAILED"
        }
    )?;
    writeln!(output, "Total violations: {}\n", results.total_violations)?;
    Ok(())
}

// Helper: Write violation counts
fn write_qg_violation_counts(output: &mut String, results: &QualityGateResults) -> Result<()> {
    debug_assert!(true, "contract: write_qg_violation_counts");
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
    debug_assert!(!violations.is_empty(), "violations must not be empty");
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
    debug_assert!(true, "contract: write_violation_details");
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
    debug_assert!(!s.is_empty(), "s must not be empty");
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

// Helper: Format as JUnit XML
/// Toyota Way: Extract Method - Format quality gate as `JUnit` XML (complexity <=8)
fn format_qg_as_junit(violations: &[QualityViolation]) -> Result<String> {
    debug_assert!(!violations.is_empty(), "violations must not be empty");
    let mut output = String::new();

    write_junit_header(&mut output)?;
    write_junit_testsuite_start(&mut output, violations.len())?;
    write_junit_testcases(&mut output, violations)?;
    write_junit_footer(&mut output)?;

    Ok(output)
}

/// Toyota Way: Extract Method - Write `JUnit` XML header (complexity <=3)
fn write_junit_header(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(output, r#"<testsuites name="Quality Gate">"#)?;
    Ok(())
}

/// Toyota Way: Extract Method - Write `JUnit` testsuite start (complexity <=3)
fn write_junit_testsuite_start(output: &mut String, count: usize) -> Result<()> {
    debug_assert!(count > 0, "count must be positive");
    use std::fmt::Write;
    writeln!(
        output,
        r#"  <testsuite name="Quality Checks" tests="{count}" failures="{count}">"#
    )?;
    Ok(())
}

/// Toyota Way: Extract Method - Write `JUnit` testcases (complexity <=5)
fn write_junit_testcases(output: &mut String, violations: &[QualityViolation]) -> Result<()> {
    debug_assert!(!violations.is_empty(), "violations must not be empty");
    for v in violations {
        write_single_junit_testcase(output, v)?;
    }
    Ok(())
}

/// Toyota Way: Extract Method - Write single `JUnit` testcase (complexity <=5)
fn write_single_junit_testcase(output: &mut String, v: &QualityViolation) -> Result<()> {
    debug_assert!(true, "contract: write_single_junit_testcase");
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

/// Toyota Way: Extract Method - Write `JUnit` XML footer (complexity <=3)
fn write_junit_footer(output: &mut String) -> Result<()> {
    debug_assert!(true, "contract: write_junit_footer");
    use std::fmt::Write;
    writeln!(output, r"  </testsuite>")?;
    writeln!(output, r"</testsuites>")?;
    Ok(())
}
