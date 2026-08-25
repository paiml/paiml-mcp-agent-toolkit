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
    write_qg_scope_notes(&mut output, results)?;

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
    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(output, "{}\n", c::header("# Quality Gate Report"))?;
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
        "{} {}\n",
        c::label("Total violations:"),
        c::number(&results.total_violations.to_string())
    )?;
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

/// Name the population each check measured over.
///
/// A zero next to a check is two different claims — "read it, it was clean" and
/// "never read it" — and until this line existed the human report rendered both
/// as the same word. `analyze satd` already prints its scope note; the gate,
/// which is the surface that decides pass/fail, printed nothing.
///
/// The line is printed even when nothing was skipped (#1035): "analysed 12 of
/// 12 file(s) walked" is the sentence that makes a zero mean something, and
/// printing it only on the bad case leaves the good case indistinguishable from
/// a check that never ran.
fn write_qg_scope_notes(output: &mut String, results: &QualityGateResults) -> Result<()> {
    use std::fmt::Write;
    let notes: Vec<String> = results
        .files_not_read
        .iter()
        .filter_map(|(check, census)| census.note().map(|n| format!("  {check}: {n}")))
        .collect();
    if notes.is_empty() {
        return Ok(());
    }
    writeln!(output, "\n## Scope (the population each check measured over):")?;
    for n in notes {
        writeln!(output, "{n}")?;
    }
    Ok(())
}

// Helper: Write violations list
fn write_qg_violations_list(output: &mut String, violations: &[QualityViolation]) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(output, "\n{}\n", c::subheader("## Violations:"))?;
    for v in violations {
        writeln!(
            output,
            "- [{}] {} - {}",
            qg_severity_word(&v.severity),
            v.check_type,
            v.message
        )?;
        if let Some(line) = v.line {
            writeln!(output, "  File: {}:{}", c::path(&v.file), line)?;
        } else {
            writeln!(output, "  File: {}", c::path(&v.file))?;
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
/// Toyota Way: Extract Method - Format quality gate as `JUnit` XML (complexity <=8)
fn format_qg_as_junit(violations: &[QualityViolation]) -> Result<String> {
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
    use std::fmt::Write;
    writeln!(
        output,
        r#"  <testsuite name="Quality Checks" tests="{count}" failures="{count}">"#
    )?;
    Ok(())
}

/// Toyota Way: Extract Method - Write `JUnit` testcases (complexity <=5)
fn write_junit_testcases(output: &mut String, violations: &[QualityViolation]) -> Result<()> {
    for v in violations {
        write_single_junit_testcase(output, v)?;
    }
    Ok(())
}

/// Escape a string for use inside an XML *attribute*.
///
/// Violation messages are free text assembled from source identifiers and
/// thresholds, so they routinely contain `<`, `>` and `"` — a generic parameter
/// like `Vec<T>` or a quoted function name was enough to make the whole report
/// unparseable, because these fields were interpolated raw. Control characters
/// go too: XML 1.0 allows none but TAB, LF and CR anywhere in a document.
fn xml_attr_escape(text: &str) -> String {
    text.chars()
        .filter(|c| matches!(c, '\t' | '\n' | '\r') || !c.is_control())
        .collect::<String>()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Toyota Way: Extract Method - Write single `JUnit` testcase (complexity <=5)
fn write_single_junit_testcase(output: &mut String, v: &QualityViolation) -> Result<()> {
    use std::fmt::Write;
    writeln!(
        output,
        r#"    <testcase name="{}" classname="{}">"#,
        xml_attr_escape(&v.message),
        xml_attr_escape(&v.check_type)
    )?;
    writeln!(
        output,
        r#"      <failure message="{}" type="{}"/>"#,
        xml_attr_escape(&v.message),
        xml_attr_escape(&v.severity)
    )?;
    writeln!(output, r"    </testcase>")?;
    Ok(())
}

/// Toyota Way: Extract Method - Write `JUnit` XML footer (complexity <=3)
fn write_junit_footer(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, r"  </testsuite>")?;
    writeln!(output, r"</testsuites>")?;
    Ok(())
}
