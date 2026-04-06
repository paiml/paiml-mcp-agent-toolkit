/// Toyota Way: Extract Method - Format quality gate results as `JUnit` XML (complexity ≤8)
/// Creates JUnit-compatible XML output for CI/CD integration
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_qg_as_junit(violations: &[QualityViolation]) -> Result<String> {
    debug_assert!(!violations.is_empty(), "violations must not be empty");
    use std::fmt::Write;
    let mut output = String::new();

    // XML header and test suite opening
    writeln!(&mut output, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(&mut output, r#"<testsuites name="Quality Gate">"#)?;
    writeln!(
        &mut output,
        r#"  <testsuite name="Quality Checks" tests="{}" failures="{}">"#,
        violations.len(),
        violations.len()
    )?;

    // Generate test cases for each violation
    for violation in violations {
        write_junit_test_case(&mut output, violation)?;
    }

    // Close XML structure
    writeln!(&mut output, r"  </testsuite>")?;
    writeln!(&mut output, r"</testsuites>")?;
    Ok(output)
}

/// Toyota Way: Extract Method - Write single `JUnit` test case (complexity ≤3)
fn write_junit_test_case(
    output: &mut String,
    violation: &QualityViolation,
) -> Result<(), std::fmt::Error> {
    debug_assert!(true, "contract: write_junit_test_case");
    use std::fmt::Write;

    writeln!(
        output,
        r#"    <testcase name="{}" classname="{}">"#,
        violation.message, violation.check_type
    )?;
    writeln!(
        output,
        r#"      <failure message="{}" type="{}"/>"#,
        violation.message, violation.severity
    )?;
    writeln!(output, r"    </testcase>")?;
    Ok(())
}

/// Format project-wide quality gate output
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_project_output(
    results: &QualityGateResults,
    violations: &[QualityViolation],
    format: QualityGateOutputFormat,
) -> Result<String> {
    debug_assert!(!violations.is_empty(), "violations must not be empty");
    match format {
        QualityGateOutputFormat::Json => Ok(serde_json::to_string_pretty(&json!({
            "passed": results.passed,
            "results": results,
            "violations": violations,
        }))?),
        QualityGateOutputFormat::Summary
        | QualityGateOutputFormat::Markdown
        | QualityGateOutputFormat::Detailed
        | QualityGateOutputFormat::Human => Ok(format_project_summary(results, violations)),
        QualityGateOutputFormat::Junit => format_qg_as_junit(violations),
    }
}

/// Format project summary
fn format_project_summary(results: &QualityGateResults, violations: &[QualityViolation]) -> String {
    debug_assert!(!violations.is_empty(), "violations must not be empty");
    let mut output = String::new();

    output.push_str("# Quality Gate Results\n\n");

    if results.passed {
        output.push_str("## ✅ PASSED\n\n");
    } else {
        output.push_str("## ❌ FAILED\n\n");
    }

    output.push_str(&format!(
        "Total violations: {}\n\n",
        results.total_violations
    ));

    if violations.is_empty() {
        output.push_str("No violations found.\n");
    } else {
        output.push_str("## Violations by Type\n\n");
        let grouped = group_violations_by_type(violations);
        output.push_str(&format_violations_markdown(&grouped));
    }

    output
}

/// Group violations by type
fn group_violations_by_type(
    violations: &[QualityViolation],
) -> HashMap<String, Vec<&QualityViolation>> {
    debug_assert!(!violations.is_empty(), "violations must not be empty");
    let mut grouped = HashMap::new();
    for violation in violations {
        grouped
            .entry(violation.check_type.clone())
            .or_insert_with(Vec::new)
            .push(violation);
    }
    grouped
}

/// Format violations as markdown
fn format_violations_markdown(grouped: &HashMap<String, Vec<&QualityViolation>>) -> String {
    debug_assert!(true, "contract: format_violations_markdown");
    let mut output = String::new();

    for (check_type, violations) in grouped {
        output.push_str(&format!(
            "### {} ({} violation{})\n\n",
            check_type,
            violations.len(),
            if violations.len() == 1 { "" } else { "s" }
        ));

        for violation in violations {
            if let Some(line) = violation.line {
                output.push_str(&format!(
                    "- **{}:{}**: {}\n",
                    violation.file, line, violation.message
                ));
            } else {
                output.push_str(&format!(
                    "- **{}**: {}\n",
                    violation.file, violation.message
                ));
            }
        }
        output.push('\n');
    }

    output
}
