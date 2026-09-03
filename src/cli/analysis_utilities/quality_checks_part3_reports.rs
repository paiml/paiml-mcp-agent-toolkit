/// The verdict word, coloured by what it means. Plain text when colour is off.
///
/// `pmat quality-gate` (aliases `check`/`c`/`gate`) used to be byte-identical
/// under `--color always`, `--color never` and `--color auto` on a pty — zero
/// escapes in any of the three. The flag parsed and changed nothing, because
/// none of these formatters ever asked [`crate::cli::colors`] anything. That is
/// the same defect shape as `analyze churn`, `query`, `analyze tdg` and the
/// wasm printers; the cure is the same one, not a fourth colour authority.
fn qg_verdict_word(passed: bool) -> String {
    use crate::cli::colors as c;
    if passed {
        c::colored(c::GREEN, "PASSED")
    } else {
        c::colored(c::RED, "FAILED")
    }
}

/// A severity label coloured by the verdict rule that governs it: the two
/// verdict-bearing severities are loud, the advisory one ([`ADVISORY_SEVERITY`])
/// is dim. Plain text when colour is off, so the word itself is unchanged.
fn qg_severity_word(severity: &str) -> String {
    use crate::cli::colors as c;
    match severity {
        "error" => c::colored(c::RED, severity),
        "warning" => c::colored(c::YELLOW, severity),
        ADVISORY_SEVERITY => c::dim(severity),
        other => c::colored(c::RED, other),
    }
}

// Helper: Format as summary
fn format_qg_as_summary(
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> Result<String> {
    use crate::cli::colors as c;
    use std::fmt::Write;
    let mut output = String::new();
    writeln!(
        &mut output,
        "{} {}",
        c::label("Quality Gate:"),
        qg_verdict_word(results.passed)
    )?;
    writeln!(
        &mut output,
        "{} {}",
        c::label("Total violations:"),
        c::number(&results.total_violations.to_string())
    )?;
    // The count that decided PASSED/FAILED above. Advisory (`info`) findings are
    // listed below but are not verdict-bearing, so "PASSED" over
    // "Total violations: 1" needs this line to be readable.
    writeln!(
        &mut output,
        "{} {}",
        c::label("Blocking violations:"),
        c::number(&results.blocking_violations.to_string())
    )?;

    // The population each check measured over. This renderer — not
    // `format_qg_as_human` — is the one `pmat quality-gate` prints when no
    // `--format` is given, so it is the surface almost every reader sees, and
    // it was the one surface without the disclosure: `--format human` named the
    // files each check declined to read and the default named none of them
    // (#1035).
    write_qg_scope_notes(&mut output, results)?;

    // Show violation summary by type
    if !violations.is_empty() {
        writeln!(&mut output)?;
        write_qg_violations_summary(&mut output, violations)?;
    }

    Ok(output)
}

// Helper: Write violation summary grouped by type
fn write_qg_violations_summary(output: &mut String, violations: &[QualityViolation]) -> Result<()> {
    use crate::cli::colors as c;
    use std::collections::BTreeMap;
    use std::fmt::Write;

    // Group violations by check type
    let mut by_type: BTreeMap<&str, Vec<&QualityViolation>> = BTreeMap::new();
    for v in violations {
        by_type.entry(&v.check_type).or_default().push(v);
    }

    for (check_type, type_violations) in by_type {
        writeln!(
            output,
            "{}",
            c::subheader(&format!(
                "## {} ({} violations)",
                check_type,
                type_violations.len()
            ))
        )?;
        // Show all violations (no truncation) so users can see the full list
        for v in type_violations.iter() {
            if let Some(line) = v.line {
                writeln!(output, "  - {}:{} - {}", c::path(&v.file), line, v.message)?;
            } else {
                writeln!(output, "  - {} - {}", c::path(&v.file), v.message)?;
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
    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(output, "{}\n", c::header("# Quality Gate Detailed Report"))?;
    writeln!(
        output,
        "{} {} {}",
        c::label("Status:"),
        if results.passed {
            "\u{2705}"
        } else {
            "\u{274c}"
        },
        qg_verdict_word(results.passed)
    )?;
    writeln!(
        output,
        "{} {}",
        c::label("Total violations:"),
        c::number(&results.total_violations.to_string())
    )?;
    // The subset that decided the status above; `info` findings are advisory.
    writeln!(
        output,
        "{} {}\n",
        c::label("Blocking violations:"),
        c::number(&results.blocking_violations.to_string())
    )?;
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
        ("Duplicates", results.identical_files),
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
    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(output, "\n{}\n", c::subheader("## All Violations"))?;
    for (i, v) in violations.iter().enumerate() {
        writeln!(
            output,
            "{}. [{}] {}: {}",
            i + 1,
            qg_severity_word(&v.severity),
            v.check_type,
            v.message
        )?;
        if let Some(line) = v.line {
            writeln!(output, "   File: {}:{}", c::path(&v.file), line)?;
        } else {
            writeln!(output, "   File: {}", c::path(&v.file))?;
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
        writeln!(
            output,
            "### {} ({} issues)\n",
            check_type,
            type_violations.len()
        )?;
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
