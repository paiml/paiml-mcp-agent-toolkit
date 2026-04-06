
// Helper: Format as summary
fn format_qg_as_summary(
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> Result<String> {
    debug_assert!(!violations.is_empty(), "violations must not be empty");
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
    debug_assert!(!violations.is_empty(), "violations must not be empty");
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
    debug_assert!(!violations.is_empty(), "violations must not be empty");
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
            "\u{2705} PASSED"
        } else {
            "\u{274c} FAILED"
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
    debug_assert!(!violations.is_empty(), "violations must not be empty");
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
/// Toyota Way: Extract Method - Format quality gate as Markdown (complexity <=8)
fn format_qg_as_markdown(
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> Result<String> {
    debug_assert!(!violations.is_empty(), "violations must not be empty");
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
    debug_assert!(!violations.is_empty(), "violations must not be empty");
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

/// Toyota Way: Extract Method - Write QG Markdown header section (complexity <=5)
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

/// Toyota Way: Extract Method - Format QG status badge (complexity <=3)
fn format_qg_status_badge(passed: bool) -> &'static str {
    if passed {
        "\u{2705} PASSED"
    } else {
        "\u{274c} FAILED"
    }
}

/// Toyota Way: Extract Method - Write QG Markdown summary table (complexity <=8)
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

/// Toyota Way: Extract Method - Write QG Markdown table headers (complexity <=3)
fn write_qg_markdown_table_headers(output: &mut String) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "| Check Type | Violations |")?;
    writeln!(output, "|------------|------------|")?;

    Ok(())
}

/// Toyota Way: Extract Method - Write QG Markdown table rows (complexity <=5)
fn write_qg_markdown_table_rows(output: &mut String, results: &QualityGateResults) -> Result<()> {
    use std::fmt::Write;

    let rows = get_qg_violation_summary_rows(results);

    for (name, count) in rows {
        writeln!(output, "| {name} | {count} |")?;
    }

    Ok(())
}
